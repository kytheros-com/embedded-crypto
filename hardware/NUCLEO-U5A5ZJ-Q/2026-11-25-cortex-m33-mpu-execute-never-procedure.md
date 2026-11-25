# Cortex-M33 MPU execute-never experiment

**Procedure date:** 2026-11-25  
**Status:** Static implementation and verification; hardware run pending.

## Objective and authority

Test whether the STM32U5A5 MPU blocks unprivileged instruction fetch
from user SRAM while allowing unprivileged data reads from the same
region and instruction execution from Flash. This is a standalone PIL
mechanism test, not production integration or FIPS validation. The
[architecture](../../docs/assurance/architecture.md),
[scope](../../docs/assurance/project-scope.md),
[MCL FSM](../../docs/design/mcl-fsm.md),
[interfaces](../../docs/design/module-interfaces.md), and
[documentation standard](../../docs/documentation-standard.md)
provide the project context. The completed
[read-denial](records/2026-11-23-cortex-m33-mpu-read-denial.md) and
[write-denial](records/2026-11-24-cortex-m33-mpu-write-denial.md)
records support the reset, MPU and exception-entry baseline.

Board assumptions match those records: TZEN=0, RDP Level 0, BOOT0 low,
Flash boot at `0x08000000`, nominal 4 MHz reset clock, no active FP
context, and no enabled interrupt source. RM0456 Rev 6 §§2.3.3, 3.5
cover this memory map. Armv8-M Memory Model and Memory Protection User
Guide §3 and PM0264 Rev 8 §§4.5.10–4.5.19 describe RBAR/RLAR/MAIR,
XN, and barriers. PM0264 §§2.4.7, 4.3.11–4.3.15 and Armv8-M Exception
Model User Guide §3.1.2.1 describe exception entry and status.
RBAR.AP occupies bits [2:1], XN bit 0; MMFSR.IACCVIOL is bit 0.
All diagnostic words are little-endian. Bit 0 is least significant.
Thumb code below is listed in halfword execution order; each halfword
is stored little-endian in SRAM.

## Memory and controls

| Region | Inclusive addresses | Privileged | Unprivileged | RBAR/RLAR |
| --- | --- | --- | --- | --- |
| Flash | `08000000–08001FFF` | RX | RX | `08000006` / `08001FE1` |
| User SRAM | `20001000–20001FFF` | RW, XN | RW, XN | `20001003` / `20001FE1` |
| Supervisor SRAM | `20002000–20002FFF` | RW, XN | no access, XN | `20002001` / `20002FE1` |

MAIR0=`0x44`; MAIR1=0; MPU_TYPE=`0x800`; MPU_CTRL starts at
zero and becomes `0x5`. Regions 3–7 are disabled. MemManage is
explicitly enabled and read back. The Flash linker length remains
8 KiB. User marker words occupy `0x20001000–0x20001007`; the
8-byte-aligned instruction buffer occupies `0x20001008–0x2000100F`.
The PSP reserve starts at `0x20001010` and ends at `0x20002000`.
Protected diagnostics occupy 168 bytes from `0x20002000`; the MSP
reserve is `[0x20002800,0x20003000)`.

Privileged startup writes these four Thumb halfwords into user SRAM:
`2002` (`MOVS r0,#2`), `6020` (`STR r0,[r4]`), `E7FE` (`B .`),
`BF00` (`NOP`). If XN enforcement fails, the code writes 2 to the
failure marker at `0x20001000` and loops. It does not touch protected
memory. Startup reads both 32-bit code words back before MPU enable.
The Flash probe executes `MOVS/ADDS/CMP` as the Flash execution
control, reads and compares both SRAM code words, and sets progress
at `0x20001004` to 1 only after all controls pass. A mismatch sets
progress to 3 and loops. It then executes DSB/ISB and `BX r12`,
with r12=`0x20001009` (Thumb bit set). The attempted instruction
fetch address is `0x20001008` after Thumb-state normalization.

## Expected fault and acceptance

The expected MemManage fault is exception 4, CFSR exactly
`0x00000001` (IACCVIOL only), HFSR=0, and MMARVALID clear.
MMFAR is therefore not an instruction-fetch address and its value
is not an acceptance criterion. The assembly handler captures PSP,
original LR/EXC_RETURN, active SP, and MSP before a Rust prologue.
The handler requires CONTROL=1, IPSR=4, active SP and MSP both
`0x20003000`, EXC_RETURN=`0xFFFFFFBC`, and basic frame PSP
`0x20001FE0`. The exact PSP and fault-status checks precede frame
reads. It requires stacked PC=`0x20001008`, stacked r12=`0x20001009`,
stacked r0=3, r1=0, r2=`0x20002000`, and the Thumb xPSR bit set.
It also requires both expected SRAM code words, progress=1,
failure=0, and the Flash `BX r12` encoding `0x4760`.

The handler never returns to Thread mode. Status 1 and phase 4
indicate a validated fault and drive repeating **four-pulse** LD1
indication, distinct from the prior three- and two-pulse patterns.
Terminal privileged failures hold LD1 on; an unprivileged control
failure leaves it off with progress=3. A missing fault that executes
the SRAM code leaves failure=2 and no success pattern. An early or
nested fault may leave incomplete diagnostics. LED alone is not
acceptance evidence.

The protected record is 42 consecutive little-endian 32-bit words at
`0x20002000`. Offsets below are byte offsets. Unlisted values must
still be retained in the raw SWD capture.

| Offsets | Expected fields |
| --- | --- |
| `00,04` | status=1, phase=4 |
| `08,0C,10` | startup CONTROL=0, IPSR=0, aligned MSP in reserve |
| `14,18,1C` | MPU_TYPE=`800`, initial CTRL=0, enabled CTRL=5 |
| `20,24` | MAIR0=`44`, MAIR1=0 |
| `28–3C` | Flash, user, supervisor RBAR/RLAR as table above |
| `40,44,48,4C` | unused mask=0, initial CFSR/HFSR captured, SHCSR bit 16 set |
| `50,54,58` | target=`20001008`, code words=`60202002`,`BF00E7FE` |
| `5C,60` | progress=1, failure=0 |
| `64,68,6C,70` | handler CONTROL=1, IPSR=4, SP=MSP=`20003000` |
| `74,78` | EXC_RETURN=`FFFFFFBC`, frame PSP=`20001FE0` |
| `7C,80,84` | CFSR=1, HFSR=0, MMFAR unused (recorded zero) |
| `88,8C` | stacked PC=`20001008`, xPSR Thumb bit set |
| `90,94,98` | stacked r0=3, r1=0, r2=`20002000` |
| `9C,A0,A4` | stacked r12=`20001009`, BX opcode=`4760`, unexpected IPSR=0 |

Status 0 is incomplete. Failure codes `81` startup state, `82`
code initialization, `83` MPU setup, `84` fault setup, `85` fault
identity, `86` frame/control mismatch, `87` unexpected exception,
`88` panic. Phases 0–4 mark reset, initialization, MPU enabled,
probe armed, and validated denial. The user marker and code bytes
should be captured independently through SWD.

## Static build and inspection

From `hardware/NUCLEO-U5A5ZJ-Q/mpu-execute-never`, use Rust 1.98.1:

```sh
cargo +1.98.1 fmt --all --check
cargo +1.98.1 check --release --target thumbv8m.main-none-eabi --locked
cargo +1.98.1 clippy --release --target thumbv8m.main-none-eabi \
  --locked -- -D warnings
cargo +1.98.1 build --release --target thumbv8m.main-none-eabi --locked
python3 inspect-elf.py \
  target/thumbv8m.main-none-eabi/release/nucleo-u5a5zj-q-mpu-execute-never \
  --payload /tmp/mpu-xn-payload.bin \
  --program-image /tmp/mpu-xn-image.bin
```

The inspector checks ELF32/Arm, Flash and SRAM load boundaries,
allocated sections, MPU symbols and encodings, vectors, stacks,
record and code-buffer placement, code-word materialization, Flash
control instructions, branch encoding, target literal, and handler
capture. It prints full ELF, unpadded Flash payload, and 16-byte
aligned programming-image sizes and SHA-256 values. Static inspection
cannot establish silicon fault behavior.

### Static verification record, 2026-11-25

Rust 1.98.1 release build and `inspect-elf.py` passed. The inspector
found the MemManage vector at `0x08000315`, Flash branch at
`0x080002FC`, Thumb target `0x20001009`, aligned SRAM code buffer at
`0x20001008`, and exactly the expected allocated sections. It
confirmed the Flash region ends at `0x08002000`, the 168-byte
supervisor record, stack separation, code-word literals, and the
`BX r12` encoding. No physical hardware was used.

| Artifact | Exact size | SHA-256 |
| --- | ---: | --- |
| Full ELF | 141,280 bytes | `f40d5a7a7fea32c3fb78cc9eed947202e3365e488fd970e1114aa3858b103ae8` |
| Flash payload | 2,604 bytes | `35a1c5ad9e61f06b7f93760fb584a60b72c0d48843b6e98cf6a4d36f58680bae` |
| 16-byte-aligned programming image | 2,608 bytes | `837e6c67c6c51fad89b9b36f1f48e46676456f589a7543c6a22de9a08e8c8538` |

`arm-none-eabi-objcopy -O binary` independently produced a payload
byte-for-byte equal to the inspector output. The programming image
adds four `0xFF` padding bytes.

The standalone `cargo fmt --check`, embedded-target `cargo check`,
Clippy with warnings denied, rustdoc with broken links denied, release
build, ELF inspection, and Rust 80-character source check passed.
`cargo test` cannot build the bare-metal `test` crate; `cargo test
--doc` reports no library target. Repository workspace formatting,
check, Clippy, tests, doctests, rustdoc, CI line-length check, and
supported `thumbv6m-none-eabi` check passed separately. These are
static and host results, not a hardware PASS.

## Pending SWD observation and recovery

**No deployment is authorized by this procedure.** After separate
human deployment authorization, an operator should independently
build and inspect the image, program only the first 8 KiB Flash page
with verification and automatic run disabled, then disconnect and
hardware reset. Keep option bytes, TrustZone, RDP, DMA, and GTZC
unchanged. Observe LD1 for at least ten seconds. Use SWD Hot Plug
without reset or halt to read 168 bytes at `0x20002000`, 16 bytes at
`0x20001000`, and CFSR/HFSR at `0xE000ED28/2C`. Read MMFAR only
for diagnostic completeness. Repeat after a second hardware reset.
Record board revision, serial, UTC time, operator, tool versions,
image hashes, raw words, LED behavior, and both reset outcomes.

Any mismatch, HardFault, stalled LED, or connection failure is
non-acceptance. Preserve the raw records. For recovery, disconnect,
reset using the established board procedure, connect under reset if
Hot Plug fails, reprogram the known-good blink image, hardware reset,
and confirm its baseline LD1 behavior. Option bytes may be read for
comparison only. Do not infer a hardware PASS from static evidence.
A successful future run would cover only this CPU Thread instruction
fetch under the tested configuration; it would not establish DMA or
other bus-master isolation, arbitrary-code containment, fault
recovery, or FIPS validation.
