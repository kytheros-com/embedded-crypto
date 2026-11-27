#!/usr/bin/env python3
"""Operator-gated FLT-01-005..008 deployment and SWD evidence runner.

PASS here is provisional pending independent engineering review.
Every external command retains its complete stdout and stderr.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

from build import BOARD, CONFIGS

BASELINE = BOARD / "stm32u5a5zj-original-option-bytes.txt"
HOTPLUG = ["-c", "port=SWD", "mode=HOTPLUG", "freq=1000"]
UNDER_RESET = ["-c", "port=SWD", "mode=UR", "freq=1000"]
CPUID = 0x410F_D214
SENTINEL = 0xA5C3_5A3C
QUOTIENT_SENTINEL = 0xA55A_5AA5
OPTION_NAMES = (
    "RDP", "BOR_LEV", "TZEN", "nRST_STOP", "nRST_STDBY",
    "nRST_SHDW", "SRAM_RST", "IWDG_SW", "IWDG_STOP",
    "IWDG_STDBY", "WWDG_SW", "SWAP_BANK", "BKPRAM_ECC",
    "SRAM3_ECC", "SRAM2_ECC", "SRAM2_RST", "nSWBOOT0",
    "nBOOT0", "PA15_PUPEN", "IO_VDD_HSLV", "IO_VDDIO2_HSLV",
    "NSBOOTADD0", "NSBOOTADD1", "WRP1A_PSTRT", "WRP1A_PEND",
    "UNLOCK_1A", "WRP1B_PSTRT", "WRP1B_PEND", "UNLOCK_1B",
    "WRP2A_PSTRT", "WRP2A_PEND", "UNLOCK_2A", "WRP2B_PSTRT",
    "WRP2B_PEND", "UNLOCK_2B",
)


def utc():
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def command(programmer, arguments, folder, name):
    path = folder / f"{name}.txt"
    if path.exists():
        raise RuntimeError(f"Refusing to overwrite evidence: {path}")
    full = [programmer, *map(str, arguments)]
    started = utc()
    result = subprocess.run(full, text=True, capture_output=True,
                            check=False)
    ended = utc()
    path.write_text(
        f"start_utc={started}\ncommand={json.dumps(full)}\n"
        f"exit={result.returncode}\nend_utc={ended}\n"
        f"--- stdout ---\n{result.stdout}\n"
        f"--- stderr ---\n{result.stderr}\n",
        encoding="utf-8",
    )
    if result.returncode:
        raise RuntimeError(f"Programmer command failed; see {path}")
    return result.stdout + result.stderr


def option_fields(text):
    text = re.sub(r"\x1b\[[0-9;]*m", "", text)
    return {
        key: int(value, 16)
        for key, value in re.findall(
            r"^\s*([A-Za-z][A-Za-z0-9_]+)\s*:\s*0x([0-9A-Fa-f]+)",
            text, re.MULTILINE,
        )
    }


def parse_words(output, address, count):
    words = {}
    for line in output.splitlines():
        match = re.match(
            r"^\s*(?:0x)?([0-9A-Fa-f]{8})\s*:\s*"
            r"((?:(?:0x)?[0-9A-Fa-f]{8}\s*)+)$", line,
        )
        if match:
            start = int(match[1], 16)
            values = [int(x, 16) for x in
                      re.findall(r"(?:0x)?([0-9A-Fa-f]{8})", match[2])]
            for index, value in enumerate(values):
                location = start + 4 * index
                if location in words:
                    raise RuntimeError("Duplicate SWD word address")
                words[location] = value
    expected = {address + 4 * index for index in range(count)}
    if set(words) != expected:
        raise RuntimeError(
            f"Incomplete SWD read at {address:#010x}: "
            f"expected {count}, observed {len(words)} words"
        )
    return [words[address + 4 * index] for index in range(count)]


def read_words(programmer, folder, name, address, count):
    output = command(
        programmer, [*HOTPLUG, "-r32", f"0x{address:08X}",
                     f"0x{count * 4:X}"], folder, name,
    )
    return parse_words(output, address, count)


def baseline_check(programmer, folder, connection):
    folder.mkdir(parents=True, exist_ok=False)
    info = command(programmer, connection, folder, "board-info")
    if not re.search(r"Device\s*ID[^\n]*0x0*481\b", info, re.I):
        raise RuntimeError("Device ID 0x481 not established")
    if not re.search(r"\b(?:Revision|Rev)\b[^\n]*\bX\b", info, re.I):
        raise RuntimeError("Silicon revision X not established")
    if "003800353335511835383531" not in info:
        raise RuntimeError("Expected ST-LINK serial not established")
    option_text = command(programmer, [*connection, "-ob", "displ"],
                          folder, "option-bytes")
    original = option_fields(BASELINE.read_text(encoding="utf-8"))
    current = option_fields(option_text)
    if any(key not in original or key not in current
           or original[key] != current[key] for key in OPTION_NAMES):
        raise RuntimeError("Option-byte baseline differs or is incomplete")
    cpuid = read_words(programmer, folder, "cpuid", 0xE000_ED00, 1)
    if cpuid != [CPUID]:
        raise RuntimeError("Cortex-M33 r0p4 CPUID mismatch")


def expect(condition, message):
    if not condition:
        raise RuntimeError(message)


def accept_alignment(name, symbols, record, user, frame,
                     sentinel, ccr, scb, cpuid):
    mode = CONFIGS[name][1]
    enabled = mode != "disabled"
    fault_pc = (symbols["invalid_target"] if mode == "invalid-state"
                else symbols["faulting_load"])
    bit = 8 if mode == "enabled" else 0
    initial, configured, handler = record[49:52]
    expect(record[0:4] == ([1, 4, 0, 0] if enabled
                           else [2, 3, 0, 0]), "Status/phase/start")
    expect(record[4:10] == [0x20003000, 0x800, 0, 5, 0x44, 0],
           "Startup MSP/MPU/MAIR")
    expect(record[10:16] == [0x08000006, 0x08001FE1,
                             0x20001003, 0x20001FE1,
                             0x20002001, 0x20002FE1],
           "MPU region encodings")
    expect(record[16:20] == [0, 0, 0, 0x50000],
           "Unused MPU/fault status/armed SHCSR")
    expect(record[20] == fault_pc and record[22] == SENTINEL,
           "Fault address or initial sentinel")
    first, second = ((0x4728, 0x4770) if mode == "invalid-state"
                     else (0xE9D5, 0x6700) if mode == "mandatory"
                     else (0xF8D5, 0x6001))
    expect(record[21] == first and record[56] == second,
           "Expected instruction halfwords")
    expect(initial & 8 == 0 and configured == (initial | bit),
           "Initial/configured CCR")
    expect(ccr == configured and record[52] == (3 if mode == "invalid-state"
              else 2 if mode == "mandatory" else 1 if enabled else 0),
           "External CCR or build mode")
    expect(record[48] == 0 and user[4] == 0,
           "Unexpected exception or probe failure")
    expect(sentinel == SENTINEL and cpuid == CPUID,
           "Sentinel or CPUID")
    positive = 0xA55A0007 if mode == "invalid-state" else 0x44332211
    expected_user = ([1, 0, positive, QUOTIENT_SENTINEL, 0]
                     if enabled else [1, 2, positive, 0x55443322, 0])
    expect(user == expected_user, "User progress/results")
    expected_scb = ([0x50008, 0x00020000, 0] if mode == "invalid-state"
                    else [0x50008, 0x01000000, 0] if enabled
                    else [0x50000, 0, 0])
    expect(scb == expected_scb, "Independent SCB syndrome")
    if not enabled:
        expect(frame is None and handler == 0
               and all(value == 0 for value in record[23:49])
               and record[53:56] == [0, 0, 0] and record[57] == 0,
               "Disabled handler fields must stay startup-cleared")
        return
    expect(frame is not None and len(frame) == 8,
           "Direct PSP frame missing")
    expect(record[23:37] == [1, 0, 1, 6, 0x20003000, 0x20003000,
                             0xFFFFFFBC, 0x20001FE0, 0, 0, 0,
                             expected_scb[0], expected_scb[1], 0],
           "Handler entry and fault status")
    expect(record[37] == 1 and record[38:46] == frame,
           "Frame validity or physical-copy mismatch")
    expect(frame[0:3] == [3, 0, 0x20002000]
           and frame[6] == fault_pc,
           "Stacked probe ground truth or PC")
    expected_xpsr = 0 if mode == "invalid-state" else 0x01000000
    expect(frame[7] & 0x010003FF == expected_xpsr,
           "Stacked Thumb/Thread/alignment state")
    expect(record[46:48] == [first, SENTINEL]
           and record[51] == handler == configured
           and record[53:56] == [positive, QUOTIENT_SENTINEL, 0]
           and record[57] == second,
           "Handler copies, instruction readback or sentinel")


def accept_repeated(symbols, record, user, frame, descriptor,
                    sentinel, ccr, scb, cpuid):
    expect(record[0:7] == [1, 5, 2, 2, 3, 5, 0],
           "Single-boot completion counters")
    expect(record[7] == 0 and record[9] == 0x50000
           and record[10] == ccr and ccr & 8 == 0
           and record[11:16] == [SENTINEL, SENTINEL, 5, 0, 0],
           "Startup state, sentinel or descriptor retirement")
    expect(record[8] == 0 and scb == [0x50080, 0, 0],
           "Sticky syndrome was not cleared or final SVC state wrong")
    expect(user == [1, 2, 3, 4, 5, 0] and sentinel == SENTINEL
           and cpuid == CPUID, "User sequence, sentinel or CPUID")
    expect(descriptor[0:7] == [5, 0, 2, 2, 6, 0x10000, 3]
           and descriptor[7:12] == [symbols["udf_two"],
                                    symbols["cont_two"],
                                    0x20001FE0, 0xFFFFFFBC, 5],
           "Trusted descriptor final state")
    for index, fault, continuation, test_id in (
        (16, symbols["udf_one"], symbols["cont_one"], 1),
        (36, symbols["udf_two"], symbols["cont_two"], 2),
    ):
        slot = record[index:index + 20]
        expect(slot[0:3] == [3, 0, 0x20002000]
               and slot[6] == fault
               and slot[7] & 0x010003FF == 0x01000000,
               "Expected UDF slot frame")
        expect(slot[8:16] == [0x10000, 0, 0x50008,
                              0xFFFFFFBC, 0x20001FE0,
                              0x20003000, 1, 6]
               and slot[16:20] == [fault, continuation, test_id, 1],
               "Expected UDF slot metadata")
    expect(frame is not None and len(frame) == 8
           and frame[6] == symbols["svc_finish_return"]
           and frame[7] & 0x010003FF == 0x01000000,
           "Final physical SVC frame")


def collect(programmer, manifest, name, folder, led):
    symbols = manifest["configurations"][name]["symbols"]
    repeat = name == "008"
    record = read_words(programmer, folder, "supervisor-record",
                        symbols["flt_results"], 64 if repeat else 58)
    frame = None
    if name != "005-disabled":
        frame = read_words(programmer, folder, "physical-frame",
                           0x20001FE0, 8)
    user = read_words(programmer, folder, "user-record",
                      symbols["mpu_work"], 6 if repeat else 5)
    descriptor = (read_words(programmer, folder, "descriptor",
                             symbols["expected_fault"], 12)
                  if repeat else None)
    sentinel = read_words(programmer, folder, "sentinel",
                          symbols["protected_sentinel"], 1)[0]
    ccr = read_words(programmer, folder, "ccr", 0xE000ED14, 1)[0]
    scb = read_words(programmer, folder, "scb", 0xE000ED24, 3)
    cpuid = read_words(programmer, folder, "cpuid", 0xE000ED00, 1)[0]
    if repeat:
        accept_repeated(symbols, record, user, frame, descriptor,
                        sentinel, ccr, scb, cpuid)
    else:
        accept_alignment(name, symbols, record, user, frame,
                         sentinel, ccr, scb, cpuid)
    expected_led = "dark" if name == "005-disabled" else "three"
    expect(led == expected_led, "Operator LED observation differs")
    return {
        "configuration": name, "result": "PROVISIONAL PASS",
        "led": led, "record_words": len(record), "user_words": len(user),
        "direct_frame_words": len(frame or []),
        "record": record, "user": user, "frame": frame,
        "descriptor": descriptor, "ccr": ccr, "scb": scb,
        "cpuid": cpuid, "sentinel": sentinel,
    }


def readback(programmer, manifest_path, manifest, name, folder):
    folder.mkdir(parents=True, exist_ok=False)
    image = manifest_path.parent / name / "image.bin"
    expected = manifest["configurations"][name]["image"]
    if image.stat().st_size != expected["bytes"] or sha(image) != expected["sha256"]:
        raise RuntimeError("Reviewed programming image changed")
    upload = folder / "physical-readback.bin"
    command(programmer, [*HOTPLUG, "--upload", "0x08000000",
                         str(expected["bytes"]), upload],
            folder, "upload")
    if not upload.is_file() or upload.read_bytes() != image.read_bytes():
        raise RuntimeError("Physical Flash readback differs from image")
    (folder / "comparison.json").write_text(json.dumps({
        "result": "PASS", "bytes": expected["bytes"],
        "sha256": sha(upload), "utc": utc(),
    }, indent=2) + "\n", encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--build-only", action="store_true")
    mode.add_argument("--campaign", action="store_true")
    mode.add_argument("--evidence-only", action="store_true")
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--reference-manifest", type=Path)
    parser.add_argument("--programmer", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--select", default=",".join(CONFIGS))
    parser.add_argument("--resume", help="Configuration:run, e.g. 006:2")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    if args.build_only:
        from build import main as build_main
        sys.argv = [sys.argv[0]]
        if args.output:
            sys.argv += ["--output", str(args.output)]
        if args.reference_manifest:
            sys.argv += ["--reference-manifest",
                         str(args.reference_manifest)]
        build_main()
        return
    if args.manifest is None or args.programmer is None:
        parser.error("--manifest and --programmer are required")
    if args.campaign and args.reference_manifest is None:
        parser.error("campaign requires --reference-manifest")
    manifest_path = args.manifest.resolve()
    manifest_data = json.loads(manifest_path.read_text())
    selected = args.select.split(",")
    if any(name not in CONFIGS for name in selected):
        parser.error("Unknown configuration in --select")
    if args.reference_manifest:
        reference = json.loads(args.reference_manifest.read_text())
        for name in selected:
            ours = manifest_data["configurations"][name]
            theirs = reference["configurations"][name]
            for key in ("sources_sha256", "payload", "image"):
                if ours[key] != theirs[key]:
                    raise RuntimeError(f"Cross-host {key} mismatch: {name}")
    resume_name, resume_run = None, 1
    if args.resume:
        resume_name, number = args.resume.split(":", 1)
        resume_run = int(number)
        if resume_name not in selected or resume_run not in (1, 2):
            parser.error("Invalid --resume target")
        skipped = selected[:selected.index(resume_name)]
        selected = selected[selected.index(resume_name):]
    else:
        skipped = []
    output = (args.output or manifest_path.parent / "evidence").resolve()
    if args.dry_run:
        for name in selected:
            item = manifest_data["configurations"][name]
            symbols = item["symbols"]
            connection = HOTPLUG if args.evidence_only else UNDER_RESET
            print(name, "precheck", [str(args.programmer),
                  *connection])
            print(name, "option bytes", [str(args.programmer),
                  *connection, "-ob", "displ"])
            if args.campaign:
                print(name, "program", [str(args.programmer),
                      *UNDER_RESET, "-w",
                      str(manifest_path.parent / name / "image.bin"),
                      "0x08000000", "-v"])
            reads = [
                (symbols["flt_results"], 64 if name == "008" else 58),
                (symbols["mpu_work"], 6 if name == "008" else 5),
                (symbols["protected_sentinel"], 1),
                (0xE000ED14, 1), (0xE000ED24, 3),
                (0xE000ED00, 1),
            ]
            if name != "005-disabled":
                reads.append((0x20001FE0, 8))
            if name == "008":
                reads.append((symbols["expected_fault"], 12))
            for address, count in reads:
                print(name, "read", [str(args.programmer), *HOTPLUG,
                      "-r32", f"0x{address:08X}", f"0x{count * 4:X}"])
            print(name, "upload", [str(args.programmer), *HOTPLUG,
                  "--upload", "0x08000000", item["image"]["bytes"],
                  "<readback.bin>"])
        return
    if output.exists() and not args.resume:
        raise RuntimeError("Evidence output exists; use a new directory")
    output.mkdir(parents=True, exist_ok=True)
    for name in skipped:
        comparison = output / name / "final-readback" / "comparison.json"
        if (not comparison.is_file()
                or json.loads(comparison.read_text()).get("result") != "PASS"):
            raise RuntimeError(
                f"Cannot resume past {name} without accepted Flash readback"
            )
    if not args.programmer.is_file() or not os.access(args.programmer,
                                                     os.X_OK):
        raise RuntimeError("Configured programmer path is not executable")
    programmer = str(args.programmer.resolve())
    summary_path = output / "summary.json"
    summary = (json.loads(summary_path.read_text())["runs"]
               if args.resume and summary_path.is_file() else [])
    for name in selected:
        folder = output / name
        folder.mkdir(exist_ok=True)
        item = manifest_data["configurations"][name]
        image = manifest_path.parent / name / "image.bin"
        if (image.stat().st_size != item["image"]["bytes"]
                or sha(image) != item["image"]["sha256"]):
            raise RuntimeError(f"Programming image mismatch: {name}")
        precheck = folder / "predeployment"
        if precheck.exists():
            precheck = folder / ("predeployment-" +
                                 datetime.now(timezone.utc).strftime(
                                     "%Y%m%dT%H%M%SZ"))
        baseline_check(programmer, precheck,
                       HOTPLUG if args.evidence_only else UNDER_RESET)
        first = resume_run if name == resume_name else 1
        already_programmed = (folder / "programming.txt").is_file()
        if args.campaign and first == 1 and not already_programmed:
            print(name, item["image"], "at", image)
            phrase = f"PROGRAM {name} {item['image']['sha256'][:12]}"
            if input(f"Type {phrase} to authorize programming: ") != phrase:
                raise RuntimeError("Programming not authorized")
            command(programmer,
                    [*UNDER_RESET, "-w", image, "0x08000000", "-v"],
                    folder, "programming")
        else:
            readback(programmer, manifest_path, manifest_data, name,
                     folder / ("preexisting-flash-check-" +
                               datetime.now(timezone.utc).strftime(
                                   "%Y%m%dT%H%M%SZ")))
        captures = []
        if first == 2:
            prior = folder / "run-1" / "decoded.json"
            if not prior.is_file():
                raise RuntimeError("Resume run 2 requires accepted run 1")
            prior_result = json.loads(prior.read_text())
            if prior_result.get("result") != "PROVISIONAL PASS":
                raise RuntimeError("Prior run 1 is not accepted")
            captures.append(prior_result)
        for number in range(first, 3):
            run_folder = folder / f"run-{number}"
            run_folder.mkdir(exist_ok=False)
            if input(f"{name} run {number}: press physical RESET, "
                     "wait for indication, then type RESET: ") != "RESET":
                raise RuntimeError("Physical RESET not confirmed")
            reset_confirmation = utc()
            led = input("LD1 observation (dark or three): ").strip()
            observed = {
                "reset_confirmation_recorded_utc": reset_confirmation,
                "led_observation_recorded_utc": utc(), "led": led,
            }
            (run_folder / "operator-observation.json").write_text(
                json.dumps(observed, indent=2) + "\n", encoding="utf-8")
            result = collect(programmer, manifest_data, name,
                             run_folder, led)
            result.update(observed)
            result["collection_completed_utc"] = utc()
            (run_folder / "decoded.json").write_text(
                json.dumps(result, indent=2) + "\n", encoding="utf-8")
            captures.append(result)
            print(name, number, result["result"])
        if len(captures) == 2:
            differences = [i for i, (a, b) in
                           enumerate(zip(captures[0]["record"],
                                         captures[1]["record"])) if a != b]
        else:
            differences = []
        readback(programmer, manifest_path, manifest_data,
                 name, folder / "final-readback")
        summary = [item for item in summary
                   if item["configuration"] != name]
        summary.append({"configuration": name, "runs": len(captures),
                        "record_differing_offsets": [4 * i for i in
                                                     differences],
                        "physical_flash": "PASS",
                        "acceptance": "PROVISIONAL PASS"})
        summary_path.write_text(
            json.dumps({"utc": utc(), "runs": summary}, indent=2) + "\n",
            encoding="utf-8")
    print("Compact summary:", output / "summary.json")
    print("Complete raw command output:", output)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"Campaign stopped: {error}", file=sys.stderr)
        raise
