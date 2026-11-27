# FLT-01-003 — Privileged Thread-mode MSP UsageFault

**Date:** 2026-11-27

**Lifecycle:** IMPLEMENTED; human review and physical execution pending

**Scope:** Standalone PIL hardware characterization

## Objective and reference basis

Execute the same T1 `UDF #0x42` as [FLT-01-002](2026-11-26-cortex-m33-usagefault-undefined-instruction-privileged-procedure.md),
with privileged Thread mode using MSP (`CONTROL=0`). The exception
frame and handler software stack share the MSP allocation. This tests
successful exception entry, not corruption, exhaustion, recovery, or
production fault handling. The project architecture §§4.2, 5.2 assigns
platform fault mechanisms to PIL; the MCL FSM §7 leaves recovery open.
The [reference catalogue](references.md) identifies the ignored local
manual revisions and their SHA-256 digests. The cited PDF page numbers
below were checked in those local copies.

- [ARM-ARM, DDI0553B.z, §B3.4, printed p. 73, RGKVQ, RSNGJ,
  RHVGB, INMBL;
  CONTROL.SPSEL, CONTROL.nPRIV, CONTROL.FPCA, §D1.2.13,
  printed pp. 1481–1482]: privileged CONTROL write, synchronization,
  MSP selection, and inactive FP context.
- [ARM-ARM, DDI0553B.z, §B3.18, printed pp. 104–105, RWYWW,
  RQLHB]: exception sequence and faulting instruction return address
  for a synchronous fault. [§B3.19, printed pp. 106–109, RQDKQ,
  RMFPR, RPWWG, RSGKY, RPBVF]: descending stack, basic state frame,
  conditional FP context, and alignment.
- [ARM-ARM, DDI0553B.z, §D1.2.96, printed pp. 1628–1629]:
  EXC_RETURN Mode=Thread, SPSEL=MSP, FType=basic; with the observed
  `TZEN=0` configuration, predict `0xFFFFFFB8`.
- [ARM-ARM, DDI0553B.z, §C2.4.259, printed p. 971]: T1 UDF
  encoding `0xDE00 | imm8`; `imm8=0x42` yields `0xDE42`.
- [ARM-EXCEPTIONS, 107706_0100_01_en, §§2.6–2.7,
  printed pp. 25–27]: explanatory basic frame and EXC_RETURN.
  [ARM-TRM, 100230_0004_00_en, r0p4, §§A1.9, B2.1,
  printed pp. A1-29, B2-44]: processor identity and CPUID.
- [PM0264, Rev. 8, §2.1.3.8, printed pp. 18–19; §§2.4.7.1–2.4.7.2,
  printed pp. 36–41; §§4.3.11–4.3.12.3, printed pp. 197–203]:
  stack selection, exception frame, SHCSR, and UFSR. [RM0456,
  Rev. 7, §§2.3.2–2.3.3, printed pp. 143, 149; §§3.5.6–3.5.7,
  printed pp. 168–169]: STM32U5A SRAM and TZEN integration.
- [ARM-ERRATA, SDEN-756493 v14.0, printed pp. 10–11,
  errata 1080541, 1435973, 2219175]; [ES0553, Rev. 6,
  Table 2 and §2.1.1, printed pp. 2, 5]: checked r0p4 and
  silicon-X limitations. The unaligned Device access, interrupt
  priority update, and partial VLLDM conditions do not occur here.

The architecture specifies the MSP choice, full descending stack,
eight-word basic state frame when FPCA=0, and the synchronous UDF PC.
The exact frame address, EXC_RETURN value, fault status, and LED result
are predictions for this board configuration, not universal UsageFault
properties. FLT-01-002 independently measured CPUID=`0x410FD214`;
each new run still reads CPUID. PM0264 Rev. 8 p. 39 broadly describes
the stacked return address as the next instruction. This differs from
the specific ARM-ARM synchronous-fault rule RQLHB. The acceptance PC
follows RQLHB; retain the wording discrepancy for human review.

## Configuration and stack ownership

Use NUCLEO-U5A5ZJ-Q, STM32U5A5ZJ silicon revision X, Cortex-M33
r0p4, original option bytes (`TZEN=0`, RDP Level 0), Rust 1.98.1,
and `thumbv8m.main-none-eabi`. Leave exception priorities at reset
defaults. Enable MemManage and UsageFault, no ordinary interrupt
sources and no active FP context. Preserve MPU_CTRL=5, MAIR0=`0x44`,
MAIR1=0, and disabled regions 3–7:

| Region | Half-open interval | RBAR | RLAR |
|---|---|---|---|
| Flash | `08000000–08002000` | `08000006` | `08001FE1` |
| User SRAM | `20001000–20002000` | `20001003` | `20001FE1` |
| Supervisor SRAM | `20002000–20003000` | `20002001` | `20002FE1` |

The record is `[20002000,200020C4)`, sentinel at `200020C4`, and
MSP reserve `[20002800,20003000)`. The PSP reserve remains allocated
at `[20001008,20002000)` but is never selected or used as a frame
pointer. The expected hardware frame is `[20002FE0,20003000)`, wholly
inside the MSP reserve and 0xF18 bytes above the sentinel's end.
The handler starts with SP=`20002FE0`; full descending software
allocations occupy addresses below it. All SWD words and the UDF
halfword are little-endian; bit 0 is the least significant bit.

Assembly calls Rust startup, waits for its return and released frame,
then sets MSP=`20003000`, CONTROL=0, executes ISB, and branches to
the stackless probe. The probe checks CONTROL=0, IPSR=0 and active
SP=`20003000`, performs the arithmetic and SRAM-store controls, and
puts their observed CONTROL/IPSR/SP in R0/R1/R2 before UDF. Those
values are controlled experimental ground truth, not trusted
production diagnostics. Its following continuation store must remain
unexecuted. The UsageFault wrapper captures original MSP, SP and LR
before stack use. It checks MSP=SP=`20002FE0` and
LR=`0xFFFFFFB8` in stackless assembly; mismatch loops with Status=0
and Phase=3, so an incomplete record cannot be accepted. On match it
passes the captured frame pointer explicitly to Rust. Rust checks
fault state and bounds before a volatile read-only copy of eight
words. No mutable frame reference, return, retry, or resumption occurs.

The inspected release code has a 24-byte `record_usagefault` prologue
(8-byte push, 16-byte subtraction), and its only reachable call on
success adds an 8-byte `indicate_success` push. The failure path is
inlined. The deepest observed SP is therefore `20002FC0`, leaving
1984 bytes above reserve bottom `20002800`; all writes through SP
are below the frame. The assembly-to-Rust SP is eight-byte aligned.
This is a static assessment of this exact optimized ELF's reachable
instructions, not a general bound under arbitrary changes, interrupts,
stack corruption, or a different compiler. Reinspect each rebuilt ELF.

## Diagnostic layout and acceptance before execution

The 49-word layout retains FLT-01-002 offsets. Offset `78` is now
**FrameMsp**, the original hardware-frame address, rather than PSP.
`FrameValid=1` means the handler allowed inspection of a candidate
basic frame; complete acceptance additionally requires all controls.

| Byte offsets | Field and expected value |
|---|---|
| `00,04` | Status=1, Phase=4. |
| `08,0C,10` | Startup CONTROL=0, IPSR=0, MSP aligned in reserve. |
| `14,18,1C` | MPU_TYPE=`800`, initial CTRL=0, enabled CTRL=5. |
| `20,24` | MAIR0=`44`, MAIR1=0. |
| `28–3C` | Three RBAR/RLAR pairs above. |
| `40,44,48,4C` | Unused-region mask=0; initial CFSR/HFSR raw; armed SHCSR=`50000`. |
| `50,54,58` | Linked UDF=`080002F2`; encoding=`DE42`; sentinel before=`A5C35A3C`. |
| `5C,60` | Positive marker=1; continuation=0. |
| `64,68,6C,70` | Handler CONTROL=0, IPSR=6, entry SP=MSP=`20002FE0`. |
| `74,78` | EXC_RETURN=`FFFFFFB8`; FrameMsp=`20002FE0`. |
| `7C,80,84` | PRIMASK=0, BASEPRI=0, FAULTMASK=0. |
| `88,8C,90,94` | SHCSR=`00050008`, CFSR=`00010000`, HFSR=0, FrameValid=1. |
| `98–B4` | Stacked R0=0, R1=0, R2=`20003000`; R3/R12/LR raw; PC=`080002F2`; xPSR as below. |
| `B8,BC,C0` | UDF readback=`DE42`; sentinel after=`A5C35A3C`; unexpected IPSR=0. |

Stacked xPSR must have Thumb bit 24 set, interrupted Thread IPSR
bits [8:0] clear, and alignment bit 9 clear. General-purpose R3,
R12, LR and condition flags have no fixed acceptance values. External
SHCSR/CFSR/HFSR must equal captured values. Directly read eight words
at `20002FE0` after each success indication and require word-for-word
equality with record offsets `98–B4`. This corroborates preservation
while the handler uses MSP; it cannot prove absence of transient writes
or behavior under stack corruption. Sentinel, markers, SCB status,
and CPUID=`410FD214` are independent reads. Each run contains 64
observed words: 49 record, one sentinel, two markers, three SCB,
one CPUID, eight physical frame words. Three slow green LD1 flashes
(nominal 500 ms on/off, extended dark interval) corroborate success
but cannot establish PASS. Solid LD1 indicates terminal failure where
GPIO is usable; the stackless entry failure may remain dark.

## macOS build and static verification

Run from repository root with pinned Rust, target, Python 3, and Arm
GNU binutils. The inspector derives Flash bytes independently from
ELF program headers, verifies routing and placement, transition,
probe, UDF, stackless capture/check, and image padding. Review the
complete disassembly and stack paths on each host build.

```sh
cd hardware/NUCLEO-U5A5ZJ-Q/usagefault-undefined-instruction-msp
rustc +1.98.1 --version
cargo +1.98.1 --version
python3 --version
arm-none-eabi-objdump --version | head -1
cargo +1.98.1 fmt --all --check
cargo +1.98.1 check --release --target thumbv8m.main-none-eabi --locked
cargo +1.98.1 clippy --release --target thumbv8m.main-none-eabi \
  --locked -- -D warnings
RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' \
  cargo +1.98.1 doc --release --no-deps \
  --target thumbv8m.main-none-eabi --locked
cargo +1.98.1 build --release --target thumbv8m.main-none-eabi --locked
elf=target/thumbv8m.main-none-eabi/release/nucleo-u5a5zj-q-usagefault-undefined-instruction-msp
python3 inspect-elf.py "$elf" --payload /tmp/flt-01-003-payload.bin \
  --program-image /tmp/flt-01-003-image.bin
arm-none-eabi-readelf -h -l -S -s -W "$elf"
arm-none-eabi-objdump -d "$elf"
arm-none-eabi-objcopy -O binary "$elf" /tmp/flt-01-003-objcopy.bin
cmp /tmp/flt-01-003-payload.bin /tmp/flt-01-003-objcopy.bin
shasum -a 256 src/main.rs src/transition.s inspect-elf.py \
  Cargo.toml Cargo.lock build.rs memory.x "$elf" \
  /tmp/flt-01-003-payload.bin /tmp/flt-01-003-image.bin
git rev-parse HEAD
git status --short --branch
```

The current Linux reference build reports UsageFault vector
`08000309`, UDF=`080002F2`, continuation=`080002F4`, encoding
`42 DE`, and a 2,624-byte programming image. Cross-host ELF bytes
can differ; require a reviewed payload and image match before Flash.
On 2026-11-27, repository HEAD was
`d0c4d0ed5d94bedd5cf72de74a89c2f0b808c3e7`, with this experiment
uncommitted. Rust/Cargo 1.98.1, Python 3.12.3 and GNU Arm
binutils 2.42 were observed. Standalone formatting, target release
check, Clippy with warnings denied, rustdoc broken-link check, release
build, ELF inspection, and byte-for-byte GNU objcopy comparison passed.
Temporary ELF copies with a changed UDF byte and an MSP-to-PSP write
were each rejected by the inspector; the final ELF was unmodified.
Root-workspace formatting, check, Clippy, tests, doctests, rustdoc,
CI line length, and supported thumbv6m check passed. These are static
and host-side observations only.
An additional exploratory `thumbv6m` check with `--all-targets` failed
because repository integration tests require `std` on that bare-metal
target. The supported CI command omits `--all-targets` and passed;
no diagnostic or test was suppressed.

| Artifact | Size in bytes | SHA-256 |
|---|---:|---|
| ELF | 141,516 | `c72af70d1ff78a08171ffa0752eb63f4cfeecee593159b04937a2e5ef9835b30` |
| Flash payload | 2,624 | `ba6ab1858d92fbfc8009a4b7c54610933300c4f60bb64ce77f4ea9a3cd8adce1` |
| Programming image | 2,624 | `ba6ab1858d92fbfc8009a4b7c54610933300c4f60bb64ce77f4ea9a3cd8adce1` |

| Source file | SHA-256 |
|---|---|
| `src/main.rs` | `ef90e360e4e15f40d56f14d42fce0b86bc607ee32e6a058d6d870e8f729898cd` |
| `src/transition.s` | `ff27c92e5eda836e8d706242eeaad55d4096fabc9ce9df00703293a9ea0580f6` |
| `inspect-elf.py` | `064f7916f44d5c01a00736e6b5b91e34b039b971e246c096b02a3e20aadad47f` |
| `Cargo.toml` | `c25fe8a2c4db617a1966048bdca204619236c967e97170229cb942c40fc38739` |
| `Cargo.lock` | `fc2349e9fccd381d56bfaaadd6ec2360bec77a9b3f424e0125f2fed3691ed338` |
| `build.rs` | `e03c4bf416547fd9c44ee88628f3b5c8010a3e18a9930c8500f9aeac65f12804` |
| `memory.x` | `c0bc3b0b75924e76956d473c8e045afae7f03f597129bf387ef93dfa2e2f67b8` |

## Human-operated board sequence

No physical board operation is authorized by this procedure alone.
Before deployment, record board ID, revision X, CPUID, BOOT0 low,
power, recovery image and tool versions. Read option bytes with
`STM32_Programmer_CLI -c port=SWD mode=UR freq=1000 -ob displ` and
compare read-only with [original settings](stm32u5a5zj-original-option-bytes.txt)
including TZEN=0/RDP Level 0. Confirm the reviewed 2,624-byte image
and the Flash page impact. The operator may then program with verify
and no automatic run:

```sh
STM32_Programmer_CLI -c port=SWD mode=UR freq=1000 \
  -w /tmp/flt-01-003-image.bin 0x08000000 -v
```

Disconnect, press **physical RESET**, observe LD1 for at least ten
seconds, then run this **consolidated first-run read block** without
resetting or halting. Save all raw output and the observation time.

```sh
date -u '+%Y-%m-%dT%H:%M:%SZ'
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x20002000 0xC4
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x20002FE0 0x20
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x200020C4 0x04
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x20001000 0x08
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0xE000ED24 0x0C
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0xE000ED00 0x04
```

Press **physical RESET** independently a second time. Again observe
LD1 for at least ten seconds. Run this **consolidated second-run
evidence block** and retain its output separately from run one:

```sh
date -u '+%Y-%m-%dT%H:%M:%SZ'
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x20002000 0xC4
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x20002FE0 0x20
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x200020C4 0x04
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x20001000 0x08
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0xE000ED24 0x0C
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0xE000ED00 0x04
git rev-parse HEAD
git status --short --branch
rustc +1.98.1 --version
cargo +1.98.1 --version
python3 --version
arm-none-eabi-objcopy --version | head -1
STM32_Programmer_CLI --version
shasum -a 256 src/main.rs src/transition.s inspect-elf.py \
  Cargo.toml Cargo.lock build.rs memory.x "$elf" \
  /tmp/flt-01-003-payload.bin /tmp/flt-01-003-image.bin
stat -f '%N %z bytes' "$elf" /tmp/flt-01-003-payload.bin \
  /tmp/flt-01-003-image.bin
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  --upload 0x08000000 2624 /tmp/flt-01-003-flash-readback.bin
shasum -a 256 /tmp/flt-01-003-flash-readback.bin
cmp /tmp/flt-01-003-image.bin /tmp/flt-01-003-flash-readback.bin
date -u '+%Y-%m-%dT%H:%M:%SZ'
```

Treat each run independently against the complete criteria above;
preserve unexpected raw observations. Require direct frame-to-record
equality, not merely the LED pattern. If Hot Plug fails, use Connect
Under Reset and the [board recovery policy](README.md#5-development-safety-policy):
restore the reviewed known-good blink image, verify programming,
press physical RESET, and confirm LD1 baseline. Leave option bytes
unchanged. Hardware PASS and production acceptance require human review.
