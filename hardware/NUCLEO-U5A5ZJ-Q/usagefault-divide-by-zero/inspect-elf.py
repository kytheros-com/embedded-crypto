#!/usr/bin/env python3
"""Inspect FLT-01-004 placement, UDIV probe, and Flash images."""

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


def inspect(path, mode):
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

    require(address("_psp_bottom") == 0x20001018,
            "Unexpected PSP bottom")
    require(address("_psp_top") == 0x20002000,
            "Unexpected PSP top")
    require(address("_stack_end") == 0x20002800,
            "Unexpected MSP bottom")
    require(address("_stack_start") == 0x20003000,
            "Unexpected MSP top")
    require(symbols["mpu_work"] == (0x20001000, 20),
            "Wrong user marker placement")
    require(symbols["flt_results"] == (0x20002000, 232),
            "Wrong diagnostic record placement")
    require(symbols["protected_sentinel"] == (0x200020e8, 4),
            "Wrong sentinel placement")
    require(address("_stack_end") > 0x200020ec,
            "Supervisor record overlaps MSP reserve")
    require(address("_psp_top") - 32 >= address("_psp_bottom"),
            "Basic frame exceeds PSP reserve")
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
    positive = address("positive_udiv")
    udf = address("faulting_udiv")
    continuation = address("unexpected_continuation")
    probe, length = symbols["fault_probe"]
    require(probe - 1 <= positive < udf < probe - 1 + length,
            "UDIV instructions outside Flash probe")
    require(continuation == udf + 4,
            "Continuation not four bytes after UDIV")
    opcode = bytes.fromhex("b3 fb fc f6")
    for label, location in (("positive", positive), ("faulting", udf)):
        require(image[location - flash_start:location - flash_start + 4]
                == opcode, f"{label} UDIV opcode is not FBB3 F6FC")
    disasm = subprocess.check_output(
        [find_binutils_tool("objdump"), "-d", str(path)], text=True
    )
    main_text = disasm.split("<main>:", 1)[1].split(
        "<fault_probe>:", 1
    )[0]
    handoff = ("bl\t", "msr\tPSP, r0", "movs\tr0, #3",
               "msr\tCONTROL, r0", "isb\tsy", "<fault_probe>")
    previous = -1
    for instruction in handoff:
        position = main_text.find(instruction, previous + 1)
        require(position > previous, f"Missing PSP handoff: {instruction}")
        previous = position
    require("0x20002000" in main_text and "msr\tMSP" not in main_text,
            "Unexpected Thread stack selection")
    probe_text = disasm.split("<fault_probe>:", 1)[1].split(
        "<UsageFault>:", 1)[0]
    for instruction in (
        "mrs\tr0, CONTROL", "mrs\tr1, IPSR", "mov\tr2, sp",
        "cmp\tr0, #3", "cmp\tr1, #0", "cmp\tr2, r5",
        "movs\tr3, #42", "mov.w\tip, #7", "<positive_udiv>",
        "udiv\tr6, r3, ip", "cmp\tr6, #6",
        "str\tr6, [r4, #8]", "mov.w\tip, #0",
        "movs\tr6, #90", "str\tr5, [r4, #0]",
        "dsb\tsy", "isb\tsy", "<faulting_udiv>",
        "str\tr6, [r4, #12]", "str\tr5, [r4, #4]",
        "str\tr5, [r4, #16]",
    ):
        require(instruction in probe_text,
                f"Missing probe instruction: {instruction}")
    require(probe_text.count("udiv\tr6, r3, ip") == 2,
            "Expected exactly two identical UDIV probes")
    require("push" not in probe_text and "sub\tsp" not in probe_text,
            "Probe uses software stack")
    handler = disasm.split("<UsageFault>:", 1)[1].split(
        "<BusFault>:", 1)[0]
    for instruction in (
        "mrs\tr0, PSP", "mov\tr1, lr", "mov\tr2, sp",
        "mrs\tr3, MSP",
        "cmp\tr0, ip", "cmp\tr2, ip", "cmp\tr3, ip",
        "cmp\tr1, ip", "<entry_failure>",
    ):
        require(instruction in handler,
                f"Missing entry capture: {instruction}")
    require("push" not in handler and "sub\tsp" not in handler
            and "0x20001fe0" in handler and "0x20003000" in handler
            and "mvn.w\tip, #67" in handler,
            "Entry wrapper does not validate stacklessly")
    require("orr.w\tr4, r4, #65536" in disasm and
            "orr.w\tr4, r4, #262144" in disasm and
            "cmp.w\tr4, #327680" in disasm,
            "Missing fault enables and SHCSR readback check")
    require("17record_usagefault>" in handler,
            "No handler transfer after entry checks")
    startup_label = next(
        line for line in disasm.splitlines()
        if line.endswith("7startup>:")
    )
    startup_text = disasm.split(startup_label, 1)[1].split(
        "<DefaultPreInit>:", 1
    )[0]
    for instruction in (
        "movw\tr0, #60824", "movt\tr0, #57344",
        "ldr.w\tr3, [r0, #-132]",
        "[r0, #-132]", "dsb\tsy", "isb\tsy",
    ):
        require(instruction in startup_text,
                f"Missing CCR RMW/readback: {instruction}")
    require(re.search(r"str\.w\tr3, \[r\d+, #196\]", startup_text)
            and re.search(r"str\.w\tr0, \[r\d+, #200\]",
                          startup_text),
            "Initial/configured CCR diagnostic stores are missing")
    if mode == "enabled":
        require("orr.w\tr6, r3, #16" in startup_text,
                "Trap-enabled build does not set DIV_0_TRP")
    else:
        require("bic.w\tr3, r3, #16" in startup_text,
                "Trap-disabled build does not clear DIV_0_TRP")
        require(re.search(r"str\tr0, \[r\d+, #0\]", startup_text),
                "Control build lacks configured status")
    record_label = next(
        line for line in disasm.splitlines()
        if line.endswith("17record_usagefault>:")
    )
    record_text = disasm.split(record_label, 1)[1].split(
        startup_label, 1
    )[0]
    if mode == "enabled":
        frame_lines = [
            line for line in record_text.splitlines()
            if re.search(r"\[lr, #-(32|24|16|8)\]", line)
        ]
        require("sub\tsp, #64" in record_text and
                len(frame_lines) == 4 and
                all("ldrd\t" in line for line in frame_lines) and
                all(f"[lr, #-{offset}]" in record_text
                    for offset in (32, 24, 16, 8)) and
                record_text.count("bl\t") == 1 and
                "16indicate_success>" in record_text,
                "Reassess frame loads and handler stack/call path")
    else:
        require("16indicate_success>" not in disasm and
                "bl\t" not in record_text,
                "Disabled fault path might signal success")
    print(f"mode={mode} entry={entry:#010x} "
          f"UsageFault={usagefault:#010x} "
          f"positive={positive:#010x} faulting={udf:#010x} "
          f"continuation={continuation:#010x}")
    print("UDIV halfwords: 0xFBB3 0xF6FC")
    return elf, image, image[probe - 1 - flash_start:
                             probe - 1 - flash_start + length]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("elf", type=Path)
    parser.add_argument("--mode", required=True,
                        choices=("enabled", "disabled"))
    parser.add_argument("--peer", type=Path)
    parser.add_argument("--payload", type=Path)
    parser.add_argument("--program-image", type=Path)
    args = parser.parse_args()
    elf, image, probe = inspect(args.elf, args.mode)
    if args.peer:
        _, _, peer_probe = inspect(args.peer,
                                   "disabled" if args.mode == "enabled"
                                   else "enabled")
        require(probe == peer_probe,
                "Probe machine code differs between configurations")
        print("Identical probe machine code: PASS")
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
