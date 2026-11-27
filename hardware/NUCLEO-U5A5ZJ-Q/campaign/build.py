#!/usr/bin/env python3
"""Build and statically inspect the FLT-01-005..008 campaign images."""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys

BOARD = Path(__file__).resolve().parents[1]
ROOT = BOARD.parents[1]
TARGET = "thumbv8m.main-none-eabi"
ALIGN = BOARD / "usagefault-alignment"
REPEAT = BOARD / "usagefault-repeated-expected"
CONFIGS = {
    "005-disabled": (ALIGN, "disabled", ["--no-default-features",
                                   "--features", "trap-disabled"]),
    "005-enabled": (ALIGN, "enabled", []),
    "006": (ALIGN, "mandatory", ["--no-default-features",
                              "--features", "mandatory"]),
    "007": (ALIGN, "invalid-state", ["--no-default-features",
                                  "--features", "invalid-state"]),
    "008": (REPEAT, "repeated", []),
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def tool(name):
    for candidate in ("arm-none-eabi-" + name, name):
        path = shutil.which(candidate)
        if path:
            return path
    raise RuntimeError(f"GNU {name} is unavailable")


def run(command, cwd, log, env=None):
    command = [str(part) for part in command]
    result = subprocess.run(command, cwd=cwd, env=env,
                            text=True, capture_output=True, check=False)
    with log.open("a", encoding="utf-8") as file:
        file.write("$ " + " ".join(command) + "\n")
        file.write(result.stdout)
        file.write(result.stderr)
        file.write(f"exit={result.returncode}\n\n")
    if result.returncode:
        raise RuntimeError(f"Command failed ({result.returncode}): "
                           + " ".join(command) + f"; see {log}")
    return result.stdout


def symbol_addresses(readelf):
    result = {}
    for line in readelf.splitlines():
        fields = line.split()
        if len(fields) == 8 and fields[0].rstrip(":").isdigit():
            result[fields[-1]] = int(fields[1], 16)
    return result


def source_files(source):
    return [source / name for name in (
        "Cargo.toml", "Cargo.lock", "build.rs", "memory.x",
        "inspect-elf.py", "src/main.rs", "src/transition.s",
    )] + sorted(source.glob("src/probe-*.s"))


def compare_reference(item, prior, name):
    for label in ("sources_sha256", "payload", "image"):
        if item[label] != prior[label]:
            raise RuntimeError(f"Cross-host {label} mismatch: {name}")


def version(command):
    result = subprocess.run([str(part) for part in command],
                            capture_output=True, text=True, check=True)
    return result.stdout.splitlines()[0]


def host_provenance(cargo):
    return {
        "os": platform.system(),
        "os_release": platform.release(),
        "os_version": platform.version(),
        "macos_version": platform.mac_ver()[0] or None,
        "architecture": platform.machine(),
        "rustc": version(["rustc", "+1.98.1", "--version"]),
        "cargo": version([cargo, "+1.98.1", "--version"]),
        "python": sys.version.splitlines()[0],
        "gnu_readelf": version([tool("readelf"), "--version"]),
        "gnu_objdump": version([tool("objdump"), "--version"]),
        "gnu_objcopy": version([tool("objcopy"), "--version"]),
    }


def check_rust_lines(log):
    sources = [*ROOT.glob("crates/**/*.rs"),
               *ALIGN.glob("src/**/*.rs"),
               *REPEAT.glob("src/**/*.rs")]
    violations = [
        f"{path}:{number}: {len(line)} characters (limit 80)"
        for path in sources
        for number, line in enumerate(path.read_text().splitlines(), 1)
        if len(line) > 80
    ]
    with log.open("a", encoding="utf-8") as file:
        file.write("Rust source line-length check: "
                   f"{len(sources)} files, {len(violations)} violations\n")
        for violation in violations:
            file.write(violation + "\n")
    if violations:
        raise RuntimeError(f"Rust line-length check failed; see {log}")


def build_one(name, output, cargo, reference=None):
    source, mode, features = CONFIGS[name]
    directory = output / name
    directory.mkdir(parents=True)
    log = directory / "build.log"
    target_dir = output / "targets" / name
    common = [cargo, "+1.98.1"]
    args = ["--locked", "--offline", "--target", TARGET,
            "--target-dir", str(target_dir), *features]
    run([*common, "fmt", "--all", "--check"], source, log)
    run([*common, "check", "--release", *args], source, log)
    run([*common, "clippy", "--release", *args, "--", "-D", "warnings"],
        source, log)
    env = dict(os.environ)
    env["RUSTDOCFLAGS"] = "-D rustdoc::broken_intra_doc_links"
    run([*common, "doc", "--release", "--no-deps", *args],
        source, log, env)
    run([*common, "build", "--release", *args], source, log)
    package = ("nucleo-u5a5zj-q-usagefault-alignment"
               if source == ALIGN else
               "nucleo-u5a5zj-q-usagefault-repeated-expected")
    elf = target_dir / TARGET / "release" / package
    if not elf.is_file():
        raise RuntimeError(f"Missing ELF: {elf}")
    shutil.copyfile(elf, directory / "firmware.elf")
    readelf = run([tool("readelf"), "-h", "-l", "-S", "-s", "-W",
                   elf], source, log)
    (directory / "readelf.txt").write_text(readelf, encoding="utf-8")
    disasm = run([tool("objdump"), "-d", elf], source, log)
    (directory / "disassembly.txt").write_text(disasm,
                                                encoding="utf-8")
    inspect = ["python3", source / "inspect-elf.py", elf]
    if source == ALIGN:
        inspect += ["--mode", mode]
    if name == "005-enabled":
        inspect += ["--peer", output / "005-disabled" / "firmware.elf"]
    inspect += ["--payload", directory / "payload.bin",
                "--program-image", directory / "image.bin"]
    inspection = run(inspect, source, log)
    (directory / "inspection.txt").write_text(inspection,
                                                encoding="utf-8")
    run([tool("objcopy"), "-O", "binary", elf,
         directory / "objcopy.bin"], source, log)
    payload = (directory / "payload.bin").read_bytes()
    image = (directory / "image.bin").read_bytes()
    if payload != (directory / "objcopy.bin").read_bytes():
        raise RuntimeError(f"Independent objcopy mismatch: {name}")
    if image[:len(payload)] != payload or len(image) % 16:
        raise RuntimeError(f"Invalid padded image: {name}")
    if image[len(payload):] != b"\xff" * (len(image) - len(payload)):
        raise RuntimeError(f"Invalid padding bytes: {name}")
    source_hashes = {
        str(path.relative_to(source)): digest(path.read_bytes())
        for path in source_files(source)
    }
    symbols = symbol_addresses(readelf)
    names = ("faulting_load", "unexpected_continuation", "invalid_target",
             "flt_results", "protected_sentinel", "mpu_work",
             "_psp_bottom", "_psp_top", "UsageFault", "SVCall",
             "expected_fault", "udf_one", "cont_one", "udf_two",
             "cont_two", "svc_arm_two", "svc_arm_two_return",
             "svc_finish", "svc_finish_return")
    selected = {key: symbols[key] for key in names if key in symbols}
    item = {
        "configuration": name,
        "mode": mode,
        "source": str(source.relative_to(ROOT)),
        "sources_sha256": source_hashes,
        "symbols": selected,
        "elf": {"bytes": elf.stat().st_size,
                "sha256": digest(elf.read_bytes())},
        "payload": {"bytes": len(payload), "sha256": digest(payload)},
        "image": {"bytes": len(image), "sha256": digest(image)},
        "objcopy_compare": "PASS",
        "inspection": inspection,
    }
    if reference is not None:
        prior = reference["configurations"][name]
        compare_reference(item, prior, name)
    print(name, item["image"])
    return item


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--reference-manifest", type=Path)
    args = parser.parse_args()
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    output = args.output or Path("/tmp") / f"flt-01-005-008-{stamp}"
    output = output.resolve()
    if output.exists():
        raise SystemExit(f"Output already exists: {output}")
    output.mkdir(parents=True)
    reference = (json.loads(args.reference_manifest.read_text())
                 if args.reference_manifest else None)
    log = output / "root-checks.log"
    cargo = shutil.which("cargo")
    if cargo is None:
        raise SystemExit("cargo unavailable")
    check_rust_lines(log)
    for command in (
        [cargo, "+1.98.1", "fmt", "--all", "--check"],
        [cargo, "+1.98.1", "check", "--workspace", "--all-targets",
         "--locked", "--offline"],
        [cargo, "+1.98.1", "clippy", "--workspace", "--all-targets",
         "--locked", "--offline", "--", "-D", "warnings"],
        [cargo, "+1.98.1", "test", "--workspace", "--all-targets",
         "--locked", "--offline"],
        [cargo, "+1.98.1", "test", "--doc", "--workspace", "--locked",
         "--offline"],
        [cargo, "+1.98.1", "check", "--workspace", "--locked",
         "--offline", "--target", "thumbv6m-none-eabi"],
    ):
        run(command, ROOT, log)
    env = dict(os.environ)
    env["RUSTDOCFLAGS"] = "-D rustdoc::broken_intra_doc_links"
    run([cargo, "+1.98.1", "doc", "--workspace", "--no-deps",
         "--locked", "--offline"], ROOT, log, env)
    configs = {}
    for name in CONFIGS:
        configs[name] = build_one(name, output, cargo, reference)
    manifest = {
        "created_utc": datetime.now(timezone.utc).isoformat(),
        "host": host_provenance(cargo),
        "campaign_scripts_sha256": {
            "build.py": digest(Path(__file__).read_bytes()),
            "runner.py": digest((BOARD / "campaign/runner.py").read_bytes()),
        },
        "git_head": run(["git", "rev-parse", "HEAD"], ROOT, log).strip(),
        "git_status": run(["git", "status", "--short", "--branch"],
                          ROOT, log),
        "configurations": configs,
    }
    (output / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print("Manifest:", output / "manifest.json")


if __name__ == "__main__":
    main()
