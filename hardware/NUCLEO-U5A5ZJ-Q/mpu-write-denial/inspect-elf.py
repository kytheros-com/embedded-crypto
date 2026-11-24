#!/usr/bin/env python3
"""Verify this experiment's linked placement and derive its Flash images."""

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

    require(address("_psp_bottom") == 0x20001008, "Unexpected PSP bottom")
    require(address("_psp_top") == 0x20002000, "Unexpected PSP top")
    require(address("_stack_end") == 0x20002800, "Unexpected MSP bottom")
    require(address("_stack_start") == 0x20003000, "Unexpected MSP top")
    require(symbols["mpu_work"] == (0x20001000, 8), "Wrong work placement")
    require(symbols["mpu_results"][1] == 184, "Unexpected record size")
    for name in ("mpu_results", "mpu_sentinel"):
        start, size = symbols[name]
        require(0x20002000 <= start < start + size <= 0x20002800,
                f"{name}: outside privileged data range")
        print(f"{name}: {start:#010x}, {size} bytes")

    require(address("__vector_table") == flash_start, "Wrong vector base")
    msp, reset = struct.unpack_from("<II", image)
    memmanage, = struct.unpack_from("<I", image, 0x10)
    require(msp == address("_stack_start"), "Incorrect vector MSP")
    require(entry == reset == address("Reset") and entry & 1,
            "Incorrect reset vector")
    require(memmanage == address("MemoryManagement") and memmanage & 1,
            "Incorrect MemManage vector")
    pc = address("mpu_faulting_store")
    probe, probe_size = symbols["mpu_probe"]
    require(probe - 1 <= pc and pc + 4 <= probe - 1 + probe_size,
            "Faulting store lies outside the probe")
    require(
        image[pc - flash_start:pc - flash_start + 4] ==
        bytes.fromhex("ccf80030"),
        "Wrong faulting store encoding",
    )
    disassembly = subprocess.check_output(
        [find_binutils_tool("objdump"), "-d", str(path)], text=True
    )
    for instruction in (
        "msr\tPSP, r0", "msr\tCONTROL, r0", "isb\tsy",
        "mrs\tr0, CONTROL", "mrs\tr1, IPSR", "mov\tr2, sp",
        "str\tr5, [r4, #0]", "ldr\tr3, [r4, #0]",
        "str\tr6, [r4, #4]", "str.w\tr3, [ip]",
        "mrs\tr0, PSP", "mov\tr1, lr", "mrs\tr3, MSP",
    ):
        require(instruction in disassembly,
                f"Missing instruction: {instruction}")
    probe_text = disassembly.split("<mpu_probe>:", 1)[1].split(
        "<MemoryManagement>:", 1
    )[0]
    require(probe_text.count("str.w\tr3, [ip]") == 1,
            "Expected exactly one prohibited store")
    source_literal = re.search(
        r"ldr\s+r3, \[pc, #\d+\]\s+@ \(([0-9a-f]+)",
        probe_text,
    )
    destination_literal = re.search(
        r"ldr\.w\s+ip, \[pc, #\d+\]\s+@ ([0-9a-f]+)",
        probe_text,
    )
    require(source_literal is not None and destination_literal is not None,
            "Cannot locate probe literal loads")
    def flash_word(address_in_bytes):
        require(flash_start <= address_in_bytes <= flash_end - 4,
                "Probe literal outside Flash")
        return struct.unpack_from("<I", image,
                                  address_in_bytes - flash_start)[0]
    require(flash_word(int(source_literal[1], 16)) == 0xC0DECAFE,
            "Incorrect prohibited source literal")
    require(flash_word(int(destination_literal[1], 16)) ==
            address("mpu_sentinel"), "Incorrect destination literal")
    print(f"entry={entry:#010x} MemManage={memmanage:#010x} "
          f"store={pc:#010x} (F8CC 3000)")
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
