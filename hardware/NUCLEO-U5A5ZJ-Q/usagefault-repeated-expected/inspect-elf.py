#!/usr/bin/env python3
"""Inspect FLT-01-008 linked UDF sequence and Flash image."""

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
        == {".vector_table", ".text", ".rodata", ".bss",
            ".mpu_user"},
        f"Unexpected allocated sections: {allocated}",
    )
    for name, kind, start, size in allocated:
        if name in (".vector_table", ".text", ".rodata"):
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
    require(symbols["mpu_work"] == (0x20001000, 24),
            "Wrong user marker placement")
    require(symbols["flt_results"] == (0x20002000, 256),
            "Wrong diagnostic record placement")
    require(symbols["protected_sentinel"] == (0x20002100, 4)
            and symbols["expected_fault"] == (0x20002104, 48),
            "Wrong sentinel placement")
    require(address("_stack_end") > 0x20002134,
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
    svc_vector = struct.unpack_from("<I", image, 0x2c)[0]
    require(svc_vector == address("SVCall") and svc_vector & 1,
            "Wrong SVCall vector")
    require(hardfault == address("HardFault") and
            memmanage == address("MemoryManagement") and
            busfault == address("BusFault"),
            "Unexpected fault-vector routing")
    udf1, next1 = address("udf_one"), address("cont_one")
    udf2, next2 = address("udf_two"), address("cont_two")
    svc1, svc1_next = address("svc_arm_two"), address("svc_arm_two_return")
    svc2, svc2_next = address("svc_finish"), address("svc_finish_return")
    probe_thumb, length = symbols["fault_probe"]
    probe = probe_thumb & ~1
    require(probe <= udf1 < next1 < svc1 < svc1_next < udf2 < next2
            < svc2 < svc2_next < probe + length,
            "Unexpected linked sequence ordering")
    for origin, next_pc, opcode in (
        (udf1, next1, bytes.fromhex("42 de")),
        (udf2, next2, bytes.fromhex("42 de")),
        (svc1, svc1_next, bytes.fromhex("72 df")),
        (svc2, svc2_next, bytes.fromhex("73 df")),
    ):
        require(next_pc == origin + 2, "Wrong continuation displacement")
        require(image[origin-flash_start:next_pc-flash_start] == opcode,
                "Unexpected UDF or SVC instruction bytes")
    disasm = subprocess.check_output(
        [find_binutils_tool("objdump"), "-d", str(path)], text=True
    )
    lines = disasm.splitlines()
    def region_text(start, end):
        return "\n".join(line for line in lines if
            (match := re.match(r"^\s*([0-9a-fA-F]+):", line))
            and start <= int(match[1], 16) < end)
    probe_text = region_text(probe, probe + length)
    require("push" not in probe_text and "sub\tsp" not in probe_text,
            "Probe uses software stack")
    for fragment in ("mrs\tr0, CONTROL", "mrs\tr1, IPSR",
                     "mov\tr2, sp", "cmp\tr0, #3", "cmp\tr1, #0",
                     "cmp\tr2, r5", "dsb\tsy", "isb\tsy",
                     "udf\t#66", "svc\t114", "svc\t115"):
        require(fragment in probe_text, f"Missing sequence operation: {fragment}")
    main_addr, main_len = symbols["main"]
    main_text = region_text(main_addr & ~1, (main_addr & ~1)+main_len)
    for fragment in ("bl\t", "msr\tPSP, r0", "movs\tr0, #3",
                     "msr\tCONTROL, r0", "isb\tsy"):
        require(fragment in main_text, f"Missing PSP handoff: {fragment}")
    usage_addr, usage_len = symbols["UsageFault"]
    svc_addr, svc_len = symbols["SVCall"]
    for name, start, size in (("UsageFault", usage_addr, usage_len),
                              ("SVCall", svc_addr, svc_len)):
        wrapper = region_text(start & ~1, (start & ~1) + size)
        for fragment in ("mrs\tr0, PSP", "mov\tr1, lr",
                         "mov\tr2, sp", "mrs\tr3, MSP"):
            require(fragment in wrapper,
                    f"{name} missing pre-stack capture: {fragment}")
        require("push" not in wrapper and "sub\tsp" not in wrapper,
                f"{name} adjusts unverified stack")
    for name in ("validate_usage_entry", "validate_svc_entry"):
        start = address(name)
        end = address("entry_failure")
        if name == "validate_usage_entry":
            end = address("validate_svc_entry")
        code = region_text(start, end)
        require(code.find("push\t{r4, lr}") > code.find("cmp\tr1, ip")
                and "pop\t{r4, pc}" in code,
                f"{name} lacks checked exception return")
    require("0x20001fe0" in disasm and "0x20003000" in disasm,
            "Entry constants changed")
    print(f"entry={entry:#010x} UsageFault={usagefault:#010x} "
          f"SVCall={svc_vector:#010x}")
    print(f"UDF1={udf1:#010x} continuation1={next1:#010x} "
          f"SVC-arm={svc1:#010x} UDF2={udf2:#010x} "
          f"continuation2={next2:#010x} SVC-finish={svc2:#010x}")
    print("UDF halfwords: 0xDE42 0xDE42; SVC: 0xDF72 0xDF73")
    return elf, image


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("elf", type=Path)
    parser.add_argument("--payload", type=Path)
    parser.add_argument("--program-image", type=Path)
    args = parser.parse_args()
    elf, image = inspect(args.elf, None)
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
