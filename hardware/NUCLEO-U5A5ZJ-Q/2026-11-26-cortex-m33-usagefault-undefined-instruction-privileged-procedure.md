# FLT-01-002 — Privileged Thread-mode undefined-instruction UsageFault

**Procedure date:** 2026-11-26  
**Lifecycle:** IMPLEMENTED; human review and board execution pending  
**Scope:** Standalone Platform Integration Layer (PIL) mechanism test

## Objective and authority

Compare with [FLT-01-001](2026-11-25-cortex-m33-usagefault-undefined-instruction-procedure.md):
execute the same `UDF #0x42` using PSP in privileged Thread mode. The
only intended architectural change before the probe is `CONTROL=2`
(`SPSEL=1`, `nPRIV=0`) instead of `3`. A privileged Thread probe using
MSP is outside this experiment. No production fault handling or recovery
is implemented. The architecture §4.2 and §5.2 assigns fault-entry
mechanisms to the PIL; the MCL FSM §7 leaves recovery realization open.
The [ledger](test-ledger.md) records lifecycle, not hardware acceptance.

## References and interpretation

The revision-specific identifiers and SHA-256 digests for the ignored
local manuals are in the [reference catalogue](references.md). The
following sections were inspected for this experiment:

- [ARM-ARM, DDI0553B.z, §B3.4, printed p. 73, RGKVQ, RSNGJ,
  RHVGB, INMBL]: CONTROL access, synchronization and stack selection.
  [ARM-ARM, DDI0553B.z, CONTROL.SPSEL and CONTROL.nPRIV,
  §D1.2.13, printed p. 1482]: bit meanings and reset values.
- [ARM-ARM, DDI0553B.z, §C2.4.259, printed p. 971]: UDF T1
  encoding `0xDE00 | imm8`, ignored immediate, and `UNDEFINED`
  operation. For `imm8=0x42`, the halfword is `0xDE42`.
- [ARM-ARM, DDI0553B.z, §B3.12, printed p. 90, IKJPM;
  §B3.18, printed pp. 104–105, RWYWW, RQLHB;
  §B3.19, printed pp. 106–109, RMFPR, RPWWG, RPBVF;
  §D1.2.96, printed pp. 1628–1629]: UsageFault classification,
  exception entry, the basic frame, faulting PC and EXC_RETURN.
- [ARM-TRM, 100230_0004_00_en, r0p4, §A1.9, printed p. A1-29;
  §B2.1, printed p. B2-44]: r0p4 CPUID `0x410FD214` at
  `0xE000ED00`. The actual board value remains to be read by SWD.
- [ARM-DGUI, 100235_0004_00_en, r0p4, §3.12.15,
  printed p. 3-240]: UDF programming guidance.
  [ARM-EXCEPTIONS, 107706_0100_01_en, Version 1.0, Issue 01,
  §§2.6–2.7, printed pp. 25–27]: basic frame and EXC_RETURN.
- [PM0264, Rev. 8, §2.1.3.8, printed pp. 18–19]: CONTROL and
  the immediate ISB after changing stack selection.
  [PM0264, Rev. 8, §§2.4.7.1–2.4.7.2, printed pp. 36–41;
  §§4.3.11–4.3.12.3, printed pp. 197–203]: frame, EXC_RETURN,
  SHCSR and CFSR/UFSR fields.
- [RM0456, Rev. 7, §2.3.2, Figure 5, printed p. 143;
  §2.3.3, Table 7, printed p. 149]: STM32U59x/U5Ax memory map
  and SRAM1. [RM0456, Rev. 7, §§3.5.6–3.5.7,
  printed pp. 168–169]: TZEN configuration and privilege isolation
  when TrustZone is deactivated.
- [ARM-ERRATA, SDEN-756493 v14.0, summary printed pp. 10–11,
  errata 1080541, 1435973, 2219175] and
  [ES0553, Rev. 6, Table 2 and §2.1.1, printed pp. 2, 5]:
  revision-X and r0p4 limitations checked for this path. Their
  stated unaligned Device-memory, interrupt-priority-update and
  partial VLLDM conditions do not match this controlled UDF probe.
- [FIRMWARE.md](FIRMWARE.md): reset MSIS clock assumption and LED
  setup. [Board README](README.md): recovery policy.

Architectural guarantees: `CONTROL=2` selects PSP and privileged
Thread mode; Handler mode remains privileged and uses MSP. Exception
entry clears CONTROL.SPSEL. UDF generates an undefined-instruction
UsageFault; `UNDEFINSTR` occupies CFSR bit 16. For a synchronous
exception, the stacked PC is the faulting instruction, not the next
instruction. The eight-word basic frame is expected when FPCA is
clear and stacking succeeds. [ARM-ARM, DDI0553B.z, §B3.8,
printed p. 78, RTXRW, IDMLS; §B3.18, printed pp. 104–105,
RWYWW, RQLHB; §B3.19, printed pp. 106–109, RPWWG, RPBVF].

Experimental predictions: startup CONTROL=0, pre-fault CONTROL=2,
Handler CONTROL=0, PSP frame at `0x20001FE0`, and
EXC_RETURN=`0xFFFFFFBC` follow this reset, aligned PSP, `TZEN=0`,
Thread/PSP and inactive-FP configuration. FLT-01-001 previously
observed the same EXC_RETURN value. The exact value is not universal
to UsageFault. CPUID=`0x410FD214` is a TRM-based processor-revision
prediction, pending an independent board read. The probe's stacked
R0/R1/R2 are controlled copies of pre-fault state, not independent
fault-register observations or trusted production PIL input.

PM0264 Rev. 8 §2.4.7.1, printed p. 39, broadly calls the stacked
return address the next instruction. This conflicts with the specific
ARM-ARM synchronous-exception rule RQLHB; UDF's stacked-PC acceptance
criterion follows RQLHB. Preserve this wording discrepancy for review.
The catalogue also records a genuine ES0553 Rev. 6 publication-date
discrepancy: the local PDF says June 2026, whereas the publisher HTML
page was labelled July 2026. Neither discrepancy is silently resolved
by a hardware observation.

## Configuration and diagnostics

Use the NUCLEO-U5A5ZJ-Q STM32U5A5ZJ revision X with original option
bytes, `TZEN=0`, RDP Level 0, BOOT0 low, Rust 1.98.1, and
`thumbv8m.main-none-eabi`. Do not alter option bytes, TrustZone,
GTZC, DMA, or debug protection. No active FP context or ordinary
interrupt source is enabled. MemManage and UsageFault are enabled.
SysTick is polled only after internal checks and has its interrupt
explicitly disabled. Reset-default priorities are used for this
controlled test; priority interaction is outside scope.

| MPU region | Inclusive address range | Privileged | Unprivileged | RBAR / RLAR |
|---|---|---|---|---|
| Flash | `08000000–08001FFF` | RX | RX | `08000006` / `08001FE1` |
| User SRAM | `20001000–20001FFF` | RW/XN | RW/XN | `20001003` / `20001FE1` |
| Supervisor SRAM | `20002000–20002FFF` | RW/XN | none | `20002001` / `20002FE1` |

The existing map has regions 3–7 disabled, MAIR0=`44`, MAIR1=0,
MPU_CTRL=5, and privileged background mapping. Marker words are at
`20001000` and `20001004`; PSP reserve is `[20001008,20002000)`.
The expected basic frame starts at `20001FE0`. The 49-word, 196-byte
record is at `20002000`, sentinel at `200020C4`, and MSP reserve at
`[20002800,20003000)`. UDF halfword `DE42` is bytes `42 DE` in
little-endian Flash. SWD record words are little-endian 32-bit values;
bit 0 is least significant. These placements match FLT-01-001.

Assembly captures original PSP, LR/EXC_RETURN, SP and MSP before any
Rust prologue. The handler records raw SHCSR, CFSR, HFSR, masks and
entry state. It dereferences the proposed PSP frame only after checking
EXC_RETURN, fault status, alignment and linker-asserted bounds. It is
terminal: no PC modification, exception return, retry or recovery.
Stacked R0 is a probe-supplied copy of pre-fault CONTROL; stacked R1
and R2 similarly copy IPSR and active SP. They are useful controlled
experimental evidence, not inherently trustworthy production PIL input.
Handler CONTROL, IPSR, stack pointers and status registers are read
from processor state. A sentinel match is a local control only.

| Byte offsets | Field and expected value |
|---|---|
| `00,04` | Status=1, phase=4 after internal checks. |
| `08,0C,10` | Startup CONTROL=0, IPSR=0, MSP aligned in reserve. |
| `14,18,1C` | MPU_TYPE=`800`, initial CTRL=0, enabled CTRL=5. |
| `20,24` | MAIR0=`44`, MAIR1=0. |
| `28–3C` | Three RBAR/RLAR pairs above. |
| `40,44,48,4C` | Unused region mask=0; initial CFSR/HFSR raw; armed SHCSR=`50000`. |
| `50,54,58` | Linked UDF address; expected encoding=`DE42`; sentinel before=`A5C35A3C`. |
| `5C,60` | Positive marker=1; continuation marker=0. |
| `64,68,6C,70` | Handler CONTROL=0, IPSR=6, SP=MSP=`20003000`. |
| `74,78` | EXC_RETURN=`FFFFFFBC`; frame PSP=`20001FE0`. |
| `7C,80,84` | PRIMASK=0, BASEPRI=0, FAULTMASK=0. |
| `88,8C,90,94` | SHCSR=`50008`, CFSR=`00010000`, HFSR=0, frame-valid=1. |
| `98–B4` | Stacked R0=2, R1=0, R2=`20002000`; raw R3, R12, LR, PC, xPSR. |
| `B8,BC,C0` | UDF readback=`DE42`; sentinel after=`A5C35A3C`; unexpected IPSR=0. |

The stacked PC must equal the independently inspected labeled UDF
address. Stacked xPSR bit 24 must be set (Thumb); IPSR bits [8:0]
must be zero (interrupted Thread), and alignment padding bit 9 must
be clear. Other general registers and condition flags are retained
raw but have no fixed-value acceptance criterion. Status 0 means
incomplete; nonzero failures `81` startup, `83` MPU, `84` fault
setup, `85` entry/frame, `86` classification/controls, `87`
unexpected exception, `88` panic. Marker 3 means the positive
control failed; marker 2 means execution continued past UDF.
`FrameValid=0` forbids interpreting frame fields.
`FrameValid=1` means the handler's EXC_RETURN, fault-status, PSP-bound
and alignment checks permitted inspection of a candidate basic frame.
It does not mean that the subsequent PC, xPSR, opcode, marker and
status checks passed or that the experiment was accepted.

The probe's ordinary SRAM store targets the user RW region, which is
accessible at either privilege level. It verifies an ordinary data
operation, while pre-fault privilege is indicated by the probe's
`MRS CONTROL` result in stacked R0 and the inspected transition.
The direct Handler-mode register reads do not independently recover
the interrupted Thread-mode nPRIV bit.

Successful checks yield repeating groups of **three green LD1 flashes**.
The reset MSIS/CPU clock assumption is 4 MHz with unity prescaling.
SysTick uses core clock and reload `1,999,999`: 2,000,000 cycles or
nominally 500 ms per wrap, below its 24-bit counter maximum of
16,777,216 cycles. Each flash is nominally 500 ms on, followed by
500 ms off; four extra dark wraps follow each three-flash group, so
there are nominally 2.5 seconds dark between groups. Loop, GPIO and
oscillator effects are unmeasured; no timing accuracy is claimed.
Failure holds LD1 on when GPIO is available. Dark or inconsistent
behavior is incomplete/unexpected. LED is corroboration, not an
experiment identifier or sufficient acceptance evidence.

## Static build and independent ELF inspection

Use the following **single macOS build and static-verification block**
from the repository root. Install pinned Rust, target, Python 3, and
Arm GNU binutils beforehand. Build only this standalone experiment;
root workspace checks are separate repository regression gates.

```sh
cd hardware/NUCLEO-U5A5ZJ-Q/usagefault-undefined-instruction-privileged
rustc +1.98.1 --version
cargo +1.98.1 --version
python3 --version
arm-none-eabi-readelf --version | head -1
arm-none-eabi-objdump --version | head -1
cargo +1.98.1 fmt --all --check
cargo +1.98.1 check --release --target thumbv8m.main-none-eabi --locked
cargo +1.98.1 clippy --release --target thumbv8m.main-none-eabi \
  --locked -- -D warnings
RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' \
  cargo +1.98.1 doc --release --no-deps \
  --target thumbv8m.main-none-eabi --locked
cargo +1.98.1 build --release --target thumbv8m.main-none-eabi --locked
elf=target/thumbv8m.main-none-eabi/release/nucleo-u5a5zj-q-usagefault-undefined-instruction-privileged
python3 inspect-elf.py "$elf" --payload /tmp/flt-01-002-payload.bin \
  --program-image /tmp/flt-01-002-image.bin
arm-none-eabi-readelf -h -l -S -s -W "$elf"
arm-none-eabi-objdump -d "$elf"
arm-none-eabi-objcopy -O binary "$elf" /tmp/flt-01-002-objcopy.bin
cmp /tmp/flt-01-002-objcopy.bin /tmp/flt-01-002-payload.bin
shasum -a 256 src/main.rs src/transition.s inspect-elf.py \
  Cargo.toml Cargo.lock build.rs memory.x "$elf" \
  /tmp/flt-01-002-payload.bin /tmp/flt-01-002-image.bin
```

The inspector checks vector routing, loadable Flash and SRAM bounds,
all MPU windows and encodings, stack/record/sentinel positions, ordered
`PSP`/`CONTROL=2`/`ISB` transition, probe controls, UDF halfword and
address, continuation, and pre-prologue exception capture. It derives
Flash payload from ELF program headers and pads the programming image
with `FF` to 16-byte granularity. Compare `objcopy` payload byte for
byte. Independently review the disassembly and all source, including
`unsafe` MMIO and frame access. Cross-host ELF hashes may differ;
compare payload and programming image bytes and hashes.

### Linux static verification observed 2026-11-26

Rust 1.98.1, Cargo 1.98.1, Python 3.12.3, and GNU Arm binutils
2.42 were observed. Repository HEAD was
`08a7fb1b9fe5faac27150e4a6bcc11262550f165`; at that build the
new source and procedure were uncommitted. They were subsequently
committed as `03e8992`. `cargo fmt`,
target release `check`, Clippy with `-D warnings`, release build,
rustdoc broken-link check, and ELF inspector passed. The release ELF
has UsageFault vector `08000309`, UDF `080002F2`, continuation
`080002F4`, and bytes `42 DE` at the UDF. Record and placement checks
passed. A temporary ELF copy with the transition changed to
`CONTROL=3` was rejected by the inspector at its ordered handoff
check; the release artifact was not changed. The separate root-workspace
regression results are reported in
the task completion report. These are static observations only.

| Artifact | Size in bytes | SHA-256 |
|---|---:|---|
| ELF | 141,500 | `4b11caea70ca5c52ec19632d688d6aed20c722a8397389f424230c6c7ee84cb9` |
| Flash payload | 2,612 | `9222e1b153fa080f94df02721b4eacefe46b332d8fb233cf5244ce3607b60bfd` |
| Programming image | 2,624 | `adf41ebc0bff0914d28fcb4c48b8daefc4a777d1a093768f9f346d7b0088802f` |

The final 12 programming-image bytes are `FF` padding. Source-file
SHA-256 digests for this build:

| File | SHA-256 |
|---|---|
| `src/main.rs` | `6d7de66ddb4102e1d74a3f466ce5326c21d7d129f43514e32022b5e2f918124a` |
| `src/transition.s` | `e50de4b685017f69c9075f0a2bae7eb904b47cce74a8c9a0f3e1d10a7966c44f` |
| `inspect-elf.py` | `5fc7e885316d483b661373fd678d5cbcd628f5391034e3a5e6224d62aa3dd3fe` |
| `Cargo.toml` | `bec23f3c63502cf61c5819ab06ac8c5caed26663ea47a24dfd7cc1e4c414ffe5` |
| `Cargo.lock` | `4bac9e9d4c5b0b230d18077107f3f0c546b6280640692d884c3cf27cb0cd856e` |
| `build.rs` | `e03c4bf416547fd9c44ee88628f3b5c8010a3e18a9930c8500f9aeac65f12804` |
| `memory.x` | `2f8e89e9efc0b6e9255535fce0dd0d305041170e2557acaf7f690afa83ed87a5` |

Collect fresh source hashes and Git status using the block below.

## Separately authorized hardware procedure

Physical deployment requires separate human authorization and review
of target-specific `unsafe` code. Before deployment, confirm board
identity, silicon revision, original option bytes by read-only check,
BOOT0 low, supply, recovery image, tools, and Flash page impact.
Only an authorized operator may program the image at `0x08000000` with
verification and automatic run disabled. The established CLI command is:

```sh
STM32_Programmer_CLI -c port=SWD mode=UR freq=1000 \
  -w /tmp/flt-01-002-image.bin 0x08000000 -v
```

Disconnect, press **physical RESET**, observe LD1 for at least ten
seconds, and collect the following exact Hot Plug reads without
resetting or halting. CLI read lengths are bytes. Repeat the complete
read sequence after a second independent **physical RESET**. Keep
both raw 49-word records, sentinel, marker, external SCB reads,
CPUID, LED observations, times and any failures separately. The
independent CPUID read is expected to return `0x410FD214` for r0p4;
retain an unexpected value rather than substituting the prediction.

```sh
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20002000 0xC4
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x200020C4 0x04
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20001000 0x08
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0xE000ED24 0x0C
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0xE000ED00 0x04
```

After the second run, use this **consolidated post-deployment evidence
block** from the experiment directory. Retain its output with both raw
run dumps. `--upload` saves physical Flash; `-r8` is not used for a
binary file. The size is the programming-image size in bytes.

```sh
date -u '+%Y-%m-%dT%H:%M:%SZ'
git rev-parse HEAD
git status --short --branch
rustc +1.98.1 --version
cargo +1.98.1 --version
python3 --version
arm-none-eabi-objcopy --version | head -1
STM32_Programmer_CLI --version
shasum -a 256 src/main.rs src/transition.s inspect-elf.py \
  Cargo.toml Cargo.lock build.rs memory.x
elf=target/thumbv8m.main-none-eabi/release/nucleo-u5a5zj-q-usagefault-undefined-instruction-privileged
stat -f '%N %z bytes' "$elf" /tmp/flt-01-002-payload.bin \
  /tmp/flt-01-002-image.bin
shasum -a 256 "$elf" /tmp/flt-01-002-payload.bin \
  /tmp/flt-01-002-image.bin
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20002000 0xC4
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x200020C4 0x04
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20001000 0x08
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0xE000ED24 0x0C
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0xE000ED00 0x04
image_size_in_bytes=$(stat -f '%z' /tmp/flt-01-002-image.bin)
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  --upload 0x08000000 "$image_size_in_bytes" \
  /tmp/flt-01-002-flash-readback.bin
stat -f '%N %z bytes' /tmp/flt-01-002-flash-readback.bin
shasum -a 256 /tmp/flt-01-002-flash-readback.bin
cmp /tmp/flt-01-002-image.bin /tmp/flt-01-002-flash-readback.bin
date -u '+%Y-%m-%dT%H:%M:%SZ'
```

Hardware acceptance requires both reset executions to meet all
controlled fields in the table, including `CONTROL=2` in stacked R0,
handler CONTROL=0, the inspected UDF PC, Thumb/Thread/no-padding xPSR,
positive marker 1, continuation 0, unchanged sentinel, expected
SCB and mask values, and independent CPUID=`0x410FD214`. Require
the exact active SHCSR=`0x00050008` in the captured record and
external SWD read. Firmware's internal success check tests only that
the required SHCSR bits are set; it does not reject extra active or
pending bits. Compare external SCB reads with stored raw values.
Preserve and explain differences in incidental raw frame
values; do not force them to match FLT-01-001 or each other. The
three-flash LED group corroborates but cannot establish PASS alone.
Unexpected results are preserved for analysis, not automatically
classified as hardware security failure.

If Hot Plug fails, use Connect Under Reset. The established
[board recovery policy](README.md#5-development-safety-policy) is to
restore the reviewed known-good blink image, verify programming,
press physical RESET and confirm its LD1 baseline. Leave option
bytes unchanged. No hardware operation or hardware PASS occurred in
this implementation task. Human engineering review remains pending.
