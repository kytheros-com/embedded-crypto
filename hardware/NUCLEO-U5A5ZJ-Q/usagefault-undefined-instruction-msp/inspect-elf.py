#!/usr/bin/env python3
"""Inspect FLT-01-003 placement, shared MSP, UDF, and Flash images."""

import argparse
import hashlib
from pathlib import Path
import struct
import subprocess
import shutil
import re

def find_binutils_tool(name):
    """Locate a GNU binutils tool, preferring the ARM cross-toolchain."""
    candidates = (f"arm-none-eabi-{name}", name)

    for candidate in candidates:
        executable = shutil.which(candidate)
        if executable is not None:
            return executable

    raise SystemExit(
        f"Cannot find {name}. Tried: {', '.join(candidates)}"
    )

def require(condition, message):
    if not condition:
        raise SystemExit(message)


def inspect(path):
    elf = path.read_bytes()
    require(elf[:7] == b"\x7fELF\x01\x01\x01", "Expected LE ELF32")
    header = struct.unpack_from("<16sHHIIIIIHHHHHH", elf)
    require(header[1:3] == (2, 40), "Expected an executable Arm ELF")
    entry, phoff, phentsize, phnum = header[4], header[5], header[9], header[10]
    require(phentsize == 32, "Unexpected program-header format")
    flash_start, flash_end = 0x08000000, 0x08002000
    image = bytearray()
    previous_end = flash_start
    headers = [
        struct.unpack_from("<8I", elf, phoff + index * phentsize)
        for index in range(phnum)
    ]
    for kind, _, virtual, physical, size, memory_size, _, _ in headers:
        if kind != 1:
            continue
        require(virtual == physical and size <= memory_size,
                "Unexpected load mapping or memory size")
        if memory_size:
            require(
                (flash_start <= physical < physical + memory_size
                 <= flash_end)
                or (0x20001000 <= physical < physical + memory_size
                    <= 0x20002000)
                or (0x20002000 <= physical < physical + memory_size
                    <= 0x20002800),
                "Unexpected loadable segment",
            )
    loads = sorted(
        (physical, offset, size)
        for kind, offset, _, physical, size, *_ in headers
        if kind == 1 and size
    )
    require(bool(loads), "No file-backed load segments")
    for start, offset, size in loads:
        require(
            previous_end <= start and start + size <= flash_end,
            "Overlapping or non-Flash file-backed load segment",
        )
        require(offset + size <= len(elf), "Truncated load segment")
        image.extend(b"\xff" * (start - previous_end))
        image.extend(elf[offset:offset + size])
        previous_end = start + size
    require(loads[0][0] == flash_start, "Missing payload at Flash origin")

    sections = subprocess.check_output(
        [find_binutils_tool("readelf"), "-SW", str(path)], text=True
    )
    allocated = []
    for line in sections.splitlines():
        match = re.match(
            r"\s*\[\s*\d+\]\s+(\S+)\s+(\S+)\s+"
            r"([0-9a-fA-F]+)\s+[0-9a-fA-F]+\s+"
            r"([0-9a-fA-F]+)\s+\S+\s+(\S+)", line
        )
        if match and "A" in match[5] and int(match[4], 16):
            allocated.append((match[1], match[2], int(match[3], 16),
                              int(match[4], 16)))
    require(
        {item[0] for item in allocated}
        == {".vector_table", ".text", ".bss", ".mpu_user"},
        f"Unexpected allocated sections: {allocated}",
    )
    for name, kind, start, size in allocated:
        if name in (".vector_table", ".text"):
            require(kind == "PROGBITS" and
                    flash_start <= start < start + size <= flash_end,
                    f"{name}: unexpected Flash placement")
        else:
            region_start, region_end = (
                (0x20001000, 0x20002000) if name == ".mpu_user"
                else (0x20002000, 0x20002800)
            )
            require(kind == "NOBITS" and
                    region_start <= start < start + size <= region_end,
                    f"{name}: unexpected SRAM placement")

    symbols = {}
    for line in subprocess.check_output(
        [find_binutils_tool("readelf"), "-sW", str(path)], text=True
    ).splitlines():
        fields = line.split()
        if len(fields) == 8 and fields[0].rstrip(":").isdigit():
            symbols[fields[7]] = int(fields[1], 16), int(fields[2])

    def address(name):
        return symbols[name][0]

    expected_regions = (
        ("flash", flash_start, flash_end, 3, 0, 0x08000006, 0x08001FE1),
        ("user", 0x20001000, 0x20002000, 1, 1, 0x20001003, 0x20001FE1),
        ("supervisor", 0x20002000, 0x20003000, 0, 1, 0x20002001, 0x20002FE1),
    )
    previous_end = 0
    for name, start, end, ap, xn, rbar, rlar in expected_regions:
        require(
            (address(f"_mpu_{name}_start"), address(f"_mpu_{name}_end"))
            == (start, end),
            f"{name}: unexpected linker window",
        )
        require(previous_end <= start < end, f"{name}: overlapping window")
        require((start | end) & 31 == 0, f"{name}: alignment failure")
        require(
            (rbar & ~31, (rlar | 31) + 1) == (start, end)
            and (rbar >> 1) & 3 == ap
            and rbar & 1 == xn
            and (rbar >> 3) & 3 == 0
            and rlar & 31 == 1,
            f"{name}: incorrect RBAR/RLAR decode",
        )
        print(f"{name}: [{start:#010x}, {end:#010x}) "
              f"RBAR={rbar:#010x} RLAR={rlar:#010x}")
        previous_end = end

    require(address("_psp_bottom") == 0x20001008,
            "Unexpected PSP bottom")
    require(address("_psp_top") == 0x20002000,
            "Unexpected PSP top")
    require(address("_stack_end") == 0x20002800,
            "Unexpected MSP bottom")
    require(address("_stack_start") == 0x20003000,
            "Unexpected MSP top")
    require(address("_stack_start") - 32 == 0x20002fe0,
            "Unexpected basic frame start")
    require(address("_stack_end") <= 0x20002fe0 and
            0x20002fe0 >= 0x200020c8,
            "Frame overlaps data or exceeds MSP reserve")
    require(symbols["mpu_work"] == (0x20001000, 8),
            "Wrong user marker placement")
    require(symbols["flt_results"] == (0x20002000, 196),
            "Wrong diagnostic record placement")
    require(symbols["protected_sentinel"] == (0x200020c4, 4),
            "Wrong sentinel placement")
    require(address("__vector_table") == flash_start,
            "Wrong vector base")
    msp, reset = struct.unpack_from("<II", image)
    hardfault, memmanage, busfault, usagefault = struct.unpack_from(
        "<4I", image, 0x0c
    )
    require(msp == address("_stack_start"), "Wrong initial MSP")
    require(entry == reset == address("Reset") and entry & 1,
            "Wrong reset vector")
    require(usagefault == address("UsageFault") and usagefault & 1,
            "Wrong UsageFault vector")
    require(hardfault == address("HardFault") and
            memmanage == address("MemoryManagement") and
            busfault == address("BusFault"),
            "Unexpected fault-vector routing")
    udf = address("faulting_udf")
    continuation = address("unexpected_continuation")
    probe, length = symbols["fault_probe"]
    require(probe - 1 <= udf < probe - 1 + length,
            "UDF outside Flash probe")
    require(continuation == udf + 2, "Continuation not after UDF")
    require(image[udf - flash_start:udf - flash_start + 2]
            == b"\x42\xde", "UDF opcode is not 0xDE42")
    disasm = subprocess.check_output(
        [find_binutils_tool("objdump"), "-d", str(path)], text=True
    )
    main_text = disasm.split("<main>:", 1)[1].split(
        "<fault_probe>:", 1
    )[0]
    transition = (
        "bl\t", "msr\tMSP, r0", "movs\tr0, #0",
        "msr\tCONTROL, r0", "isb\tsy", "<fault_probe>",
    )
    previous = -1
    for instruction in transition:
        position = main_text.find(instruction, previous + 1)
        require(position > previous,
                f"Missing ordered privileged MSP handoff: {instruction}")
        previous = position
    require("msr\tPSP" not in main_text and
            "movs\tr0, #2" not in main_text and
            "movs\tr0, #3" not in main_text,
            "Unexpected PSP or unprivileged transition")
    require("0x20003000" in main_text and
            main_text.find("bl\t") < main_text.find("msr\tMSP, r0"),
            "MSP top is not established after startup returns")
    probe_text = disasm.split("<fault_probe>:", 1)[1].split(
        "<UsageFault>:", 1)[0]
    for instruction in (
        "mrs\tr0, CONTROL", "mrs\tr1, IPSR", "mov\tr2, sp",
        "adds\tr6, #1", "cmp\tr0, #0", "cmp\tr1, #0",
        "cmp\tr2, r5", "str\tr6, [r4, #0]", "dsb\tsy",
        "isb\tsy", "udf\t#66", "str\tr6, [r4, #4]",
    ):
        require(instruction in probe_text,
                f"Missing probe instruction: {instruction}")
    require(probe_text.count("udf\t#66") == 1,
            "Expected exactly one UDF probe")
    require("<probe_failure>" in probe_text and
            "movs\tr6, #3" in probe_text and
            "movs\tr6, #2" in probe_text,
            "Missing positive-control failure or continuation path")
    handler = disasm.split("<UsageFault>:", 1)[1].split(
        "<BusFault>:", 1)[0]
    for instruction in (
        "mrs\tr0, MSP", "mov\tr1, lr", "mov\tr2, sp",
        "mrs\tr3, MSP", "cmp\tr0, ip", "cmp\tr2, ip",
        "cmp\tr3, ip", "cmp\tr1, ip", "<entry_failure>",
    ):
        require(instruction in handler,
                f"Missing entry capture: {instruction}")
    require("orr.w\tr4, r4, #65536" in disasm and
            "orr.w\tr4, r4, #262144" in disasm and
            "cmp.w\tr4, #327680" in disasm,
            "Missing fault enables and SHCSR readback check")
    require("mrs\tr0, PSP" not in handler and
            "push" not in handler.split("<entry_failure>:", 1)[0],
            "Entry uses PSP or adjusts stack before checking MSP")
    require("0x20002fe0" in handler and "mvn.w\tip, #71" in handler
            and "b.w\t" in handler and "b.n\t" in handler,
            "Missing exact stackless entry values or terminal branch")
    rust_handler = disasm.split("17record_usagefault>:", 1)[1].split(
        "7startup>:", 1
    )[0]
    require("push\t{r7, lr}" in rust_handler and
            "sub\tsp, #16" in rust_handler,
            "Reassess compiled handler stack use")
    require(rust_handler.count("bl\t") == 1 and
            "16indicate_success>" in rust_handler,
            "Reassess reachable handler calls")
    print(f"entry={entry:#010x} UsageFault={usagefault:#010x} "
          f"UDF={udf:#010x} continuation={continuation:#010x}")
    print("UDF halfword: 0xDE42")
    return elf, image


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("elf", type=Path)
    parser.add_argument("--payload", type=Path)
    parser.add_argument("--program-image", type=Path)
    args = parser.parse_args()
    elf, image = inspect(args.elf)
    padded = image + b"\xff" * (-len(image) % 16)
    for label, data in (("ELF", elf), ("Flash payload", image),
                        ("16-byte-aligned programming image", padded)):
        print(f"{label}: {len(data)} bytes, SHA-256 "
              f"{hashlib.sha256(data).hexdigest()}")
    if args.payload:
        args.payload.write_bytes(image)
    if args.program_image:
        args.program_image.write_bytes(padded)


if __name__ == "__main__":
    main()
