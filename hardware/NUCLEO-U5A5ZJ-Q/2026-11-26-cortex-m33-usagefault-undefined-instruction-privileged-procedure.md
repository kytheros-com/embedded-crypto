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

Documents consulted for this procedure:

- [Arm Cortex-M33 Devices Generic User Guide, 100235_0100_01_en,
  r1p0, issue 0100-01](https://documentation-service.arm.com/static/5f16e93d20b7cf4bc524af1d),
  §2.1 CONTROL register (Table 2-16), §3.13.15 UDF. The former defines
  SPSEL and nPRIV; the latter specifies undefined-instruction UsageFault.
- [Armv8-M Exception Model User Guide, 107706_0100_01_en, Version 1.0,
  issue 0100-01](https://documentation-service.arm.com/static/64c7832738511951cb7a246e),
  §§2.2, 2.6–2.7, 3.1.4, 4.3.3: exception entry, basic frame,
  EXC_RETURN, UNDEFINSTR, and inactive FP context.
- [ST PM0264 Rev 8](https://www.st.com/resource/en/programming_manual/pm0264-stm32-cortexm33-mcus-programming-manual-stmicroelectronics.pdf),
  §§2.4.7, 4.3.11–4.3.13, as cited by FLT-01-001: exception and fault
  register descriptions. The PDF was too large for direct retrieval in
  this session; section content is inherited from the reviewed baseline
  and needs independent human confirmation.
- [ST RM0456 Rev 6](https://www.st.com/resource/en/reference_manual/rm0456-stm32u5-series-armbased-32bit-mcus-stmicroelectronics.pdf),
  §§2.3.2–2.3.3, 3.5, as cited by FLT-01-001: memory and security
  configuration. The PDF was too large for direct retrieval here; the
  revision and sections need independent human confirmation.
- [ST ES0553 Rev 6, July 2026](https://www.st.com/content/st_com/en/technical-documents/es0553-stm32u59xxx-and-stm32u5axxx-device-errata-stmicroelectronics.html),
  device summary and Core errata. The listed access-permission versus
  unaligned-device-fault priority issue does not exercise this UDF path.
- [FIRMWARE.md](FIRMWARE.md), reset MSIS clock assumption and board
  pin/timer setup; [board README](README.md), recovery policy.

Architectural guarantees: Handler mode is privileged and uses MSP;
`CONTROL.nPRIV` describes Thread-mode privilege. EXC_RETURN bits
encode interrupted mode, stack selection, security and frame type.
`UNDEFINSTR` is CFSR bit 16. The exact `0xFFFFFFBC` value, unchanged
from FLT-01-001, is an expected result for this `TZEN=0`, Thread/PSP,
non-FP configuration and is corroborated by the previous observation.
It is not a universal UsageFault value. The expectation `CONTROL=0`
in Handler mode follows the privileged Thread setting and Handler
SPSEL clearing. If documentation or hardware contradicts any value,
retain the discrepancy; do not silently change acceptance criteria.
ES0553's Core section points to a Cortex-M33 r0p4 reference, while the
consulted generic guide is r1p0. Applicability of that guide's revision
to this device requires human confirmation; the previous experiment's
hardware result corroborates the shared exception behavior on this board.

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
`08a7fb1b9fe5faac27150e4a6bcc11262550f165`, with this new
source and procedure uncommitted. `cargo fmt`,
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

The final 12 programming-image bytes are `FF` padding. The source is
uncommitted. Source-file SHA-256 digests for this build:

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
LED observations, times and any failures separately.

```sh
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20002000 0xC4
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x200020C4 0x04
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20001000 0x08
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0xE000ED24 0x0C
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
positive marker 1, continuation 0, unchanged sentinel, and expected
SCB and mask values. Compare external SCB reads with stored raw
values. Preserve and explain differences in incidental raw frame
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
