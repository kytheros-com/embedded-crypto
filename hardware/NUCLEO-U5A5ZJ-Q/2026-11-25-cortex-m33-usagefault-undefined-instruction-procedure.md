# FLT-01-001 — Cortex-M33 undefined-instruction UsageFault

**Procedure date:** 2026-11-25
**Lifecycle:** Standalone implementation; board execution pending
**Scope:** PIL mechanism characterization, not production fault policy

## Objective, authority, and limits

Execute one labeled `UDF #0x42` in unprivileged Thread mode, after an
ordinary Flash-arithmetic and user-SRAM positive control. Determine
whether the configured processor enters privileged UsageFault and
reports an undefined instruction with a valid basic PSP frame. This
is a controlled fault, not evidence of general fault attribution or
containment. [Architecture](../../docs/assurance/architecture.md) §4.2
and [MCL FSM](../../docs/design/mcl-fsm.md) §7 define the PIL context;
[ledger](test-ledger.md) records lifecycle and coverage.

Architectural references consulted:

- [Arm Cortex-M33 Devices Generic User Guide,
  `100235_0100_01_en`, §3.13.15](https://documentation-service.arm.com/static/5f16e93d20b7cf4bc524af1d):
  UDF is permanently undefined and generates UsageFault. Its printed
  page is 3-247. The immediate does not affect fault classification.
- [Armv8-M Exception Model User Guide,
  `107706_0100_01_en`, Version 1.0, §§2.2, 2.6–2.7,
  3.1.4](https://documentation-service.arm.com/static/64c7832738511951cb7a246e):
  exception entry, basic stack frame, EXC_RETURN, and `UNDEFINSTR`.
- [ST PM0264 Rev 8, §§2.4.7, 4.3.11–4.3.13](https://www.st.com/resource/en/programming_manual/pm0264-stm32-cortexm33-mcus-programming-manual-stmicroelectronics.pdf):
  exception context, SHCSR, CFSR/UFSR, and HFSR.
- [ST RM0456 Rev 6, §§2.3.2–2.3.3,
  3.5](https://www.st.com/resource/en/reference_manual/rm0456-stm32u5-series-armbased-32bit-mcus-stmicroelectronics.pdf):
  target memory and security configuration context.
- [ST ES0553 Rev 6, device summary and core-errata summary](https://www.st.com/content/st_com/en/technical-documents/es0553-stm32u59xxx-and-stm32u5axxx-device-errata-stmicroelectronics.html):
  revision X applicability. Its listed access-permission/unaligned
  priority limitation is not exercised by this UDF probe.

The expected `0xDE42` halfword is confirmed by the linked ELF, not
inferred from the assembly mnemonic. The precise EXC_RETURN value
`0xFFFFFFBC` and handler `CONTROL=1` are expectations for this
configured, non-FP, `TZEN=0` path, consistent with prior board
observations; they are not universal UsageFault constants. `UDF`
places the faulting instruction at the stacked PC under the cited
exception model. A disagreement is non-acceptance and must be retained.
The EXC_RETURN low byte `BC` selects default callee stacking,
basic frame, interrupted Thread mode, and PSP, with security-extension
fields clear under this configuration (Arm guide §2.7).

## Configuration and memory

Use the recorded revision X NUCLEO-U5A5ZJ-Q, original option bytes
(`TZEN=0`, RDP Level 0), BOOT0 low, Rust 1.98.1,
`thumbv8m.main-none-eabi`, STM32CubeProgrammer 2.24.0, and ST-LINK
SWD. No active FP context or ordinary interrupt source is enabled.
SysTick is polled with `TICKINT=0` only after a validated fault.
Do not change option bytes, TrustZone, GTZC, DMA, or debug protection.

| MPU region | Inclusive range | Privileged | Unprivileged | RBAR/RLAR |
|---|---|---|---|---|
| Flash | `08000000–08001FFF` | RX | RX | `08000006` / `08001FE1` |
| User SRAM | `20001000–20001FFF` | RW/XN | RW/XN | `20001003` / `20001FE1` |
| Supervisor SRAM | `20002000–20002FFF` | RW/XN | none | `20002001` / `20002FE1` |

The MPU has eight regions; regions 3–7 are disabled. `MAIR0=0x44`,
`MAIR1=0`, and enabled `MPU_CTRL=0x5` use the privileged
background map. Startup enables MemManage and UsageFault, checks
`SHCSR=0x00050000` and clears captured sticky fault status before
unprivileged entry. Expected UsageFault entry adds `USGFAULTACT`
(bit 3), giving `SHCSR=0x00050008` if no other handler state is set.
The experiment uses reset-default system-handler priorities with no
ordinary interrupt source enabled; Arm's exception guide §3.1.4.1
recommends configuring priorities before enabling UsageFault for
general applications. Priority interactions remain outside this test.

User markers are at `0x20001000` (positive control) and
`0x20001004` (unexpected continuation). PSP reserve is
`[0x20001008,0x20002000)`; its top is `0x20002000`. The basic
32-byte frame is expected at `0x20001FE0`. Protected diagnostics
start at `0x20002000` and occupy 196 bytes; the protected sentinel
is at `0x200020C4`. MSP reserve is `[0x20002800,0x20003000)`.
The UDF is in permitted executable Flash. The instruction halfword
`0xDE42` is stored little-endian as bytes `42 DE`. Diagnostic words
are 32-bit little-endian; bit 0 is least significant.

The assembly entry captures original PSP, LR/EXC_RETURN, active SP,
and MSP before a Rust prologue. The handler records SHCSR, CFSR,
HFSR, masks, and entry state first. It reads the presumed PSP frame
only if EXC_RETURN is the expected basic PSP form, the pointer is
aligned and within the linker-asserted reserve, the handler is on
MSP, and no stacking error is indicated. If these checks fail,
`FrameValid=0`; frame words remain uninterpreted. It never modifies
stacked PC or returns. Privileged code reads a synthetic sentinel
before and after the fault. An unchanged word is only a local control.

## Diagnostic record and outcomes

Read 49 little-endian words at `0x20002000`. Byte offsets are hex.
Raw words must be preserved even if an internal check fails.

| Offsets | Fields and expected values |
|---|---|
| `00,04` | Status=1, phase=4 only after all internal checks. |
| `08,0C,10` | Startup CONTROL=0, IPSR=0, MSP aligned in its reserve. |
| `14,18,1C` | MPU_TYPE=`800`, initial CTRL=0, enabled CTRL=5. |
| `20,24` | MAIR0=`44`, MAIR1=0. |
| `28–3C` | Three RBAR/RLAR pairs in the table above. |
| `40,44,48,4C` | Unused-region mask=0; captured initial CFSR/HFSR; armed SHCSR=`50000`. |
| `50,54,58` | Linked UDF address; expected encoding=`DE42`; sentinel before=`A5C35A3C`. |
| `5C,60` | Positive-control marker=1; continuation marker=0. |
| `64,68,6C,70` | Handler CONTROL=1, IPSR=6, SP=MSP=`20003000`. |
| `74,78` | EXC_RETURN=`FFFFFFBC`; frame PSP=`20001FE0`. |
| `7C,80,84` | PRIMASK=0, BASEPRI=0, FAULTMASK=0. |
| `88,8C,90,94` | SHCSR=`50008`, CFSR=`00010000`, HFSR=0, frame-valid=1. |
| `98–B4` | Eight raw frame words r0,r1,r2,r3,r12,LR,PC,xPSR. |
| `B8,BC,C0` | UDF readback=`DE42`; sentinel after=`A5C35A3C`; unexpected IPSR=0. |

Expected stacked r0=3, r1=0, and r2=`0x20002000` capture
pre-fault CONTROL, IPSR, and active PSP. The stacked PC must equal
the independently inspected UDF address. Stacked xPSR must have
Thumb bit 24 set and IPSR bits [8:0] zero (interrupted Thread mode).
Other stacked registers and xPSR flags are retained raw; their
incidental values are not acceptance criteria.

Status 0 means incomplete. Nonzero failures: `81` startup state,
`83` MPU setup, `84` SHCSR/fault setup, `85` entry/frame validity,
`86` classification or controls, `87` unexpected exception, `88`
panic. A failed positive control leaves continuation marker 3;
execution beyond UDF sets it to 2. `FrameValid=0` forbids frame
interpretation. A valid diagnostic path produces a repeating
**five-pulse green LD1** pattern; failures hold LD1 on when GPIO
is available. LED behavior alone cannot establish acceptance.

Hardware acceptance requires two independently reset executions with
matching full protected and user-marker records; positive control=1;
continuation=0; sentinel unchanged; handler IPSR=6 and CONTROL=1;
EXC_RETURN=`FFFFFFBC`; valid basic PSP frame; stacked PC at the
inspected UDF; xPSR Thumb/Thread bits; CFSR exactly `0x00010000`
(`UNDEFINSTR` alone); HFSR=0; armed/active SHCSR values above;
and the five-pulse pattern. Any divergence is non-acceptance pending
analysis, not an automatic hardware security-mechanism FAIL.

## Reproducible static build and image inspection

From `hardware/NUCLEO-U5A5ZJ-Q/usagefault-undefined-instruction`:

```sh
cargo +1.98.1 fmt --all --check
cargo +1.98.1 check --release --target thumbv8m.main-none-eabi \
  --locked --offline
cargo +1.98.1 clippy --release --target thumbv8m.main-none-eabi \
  --locked --offline -- -D warnings
cargo +1.98.1 build --release --target thumbv8m.main-none-eabi \
  --locked --offline
elf=target/thumbv8m.main-none-eabi/release/nucleo-u5a5zj-q-usagefault-undefined-instruction
python3 inspect-elf.py "$elf" --payload /tmp/flt-01-001-payload.bin \
  --program-image /tmp/flt-01-001-image.bin
arm-none-eabi-readelf -h -l -S -s -W "$elf"
arm-none-eabi-objdump -d "$elf"
arm-none-eabi-objcopy -O binary "$elf" /tmp/flt-01-001-objcopy.bin
cmp /tmp/flt-01-001-objcopy.bin /tmp/flt-01-001-payload.bin
sha256sum "$elf" /tmp/flt-01-001-payload.bin \
  /tmp/flt-01-001-image.bin
```

The inspector checks the vector, load/section boundaries, MPU map
symbols, record/sentinel/stack placement, labeled 16-bit UDF, positive
control, continuation, and pre-prologue capture. The padding to the
16-byte programming granularity must consist only of `0xFF` bytes.
On macOS, after installing the pinned toolchain, target, binutils,
Python 3, and locked dependencies, repeat with:

```sh
cargo +1.98.1 fmt --all --check
cargo +1.98.1 check --release --target thumbv8m.main-none-eabi --locked
cargo +1.98.1 clippy --release --target thumbv8m.main-none-eabi \
  --locked -- -D warnings
cargo +1.98.1 build --release --target thumbv8m.main-none-eabi --locked
elf=target/thumbv8m.main-none-eabi/release/nucleo-u5a5zj-q-usagefault-undefined-instruction
python3 inspect-elf.py "$elf" --payload /tmp/flt-01-001-payload.bin \
  --program-image /tmp/flt-01-001-image.bin
arm-none-eabi-objcopy -O binary "$elf" /tmp/flt-01-001-objcopy.bin
cmp /tmp/flt-01-001-objcopy.bin /tmp/flt-01-001-payload.bin
shasum -a 256 "$elf" /tmp/flt-01-001-payload.bin \
  /tmp/flt-01-001-image.bin
```

Compare Flash payload and programming-image hashes across hosts;
ELF metadata may differ. Record all three hashes and exact source
identity before deployment.

### Static verification record — 2026-11-25

The Rust 1.98.1 release ELF has UsageFault vector `0x08000301`,
UDF address `0x080002EC`, and continuation `0x080002EE`.
Disassembly and extracted bytes show `DE42` exactly once at the
labeled probe. The inspector confirmed the three-region map,
49-word record, sentinel, and separate PSP/MSP reserves.

| Artifact | Size | SHA-256 |
|---|---:|---|
| ELF | 141,448 bytes | `fbdb9a8f5be5bceb0da0fa946f6eb47e4a0e089502bc59b01b1760761288188d` |
| Unpadded Flash payload | 2,736 bytes | `c8dbcc5ce07e86504f522cde0463f45abb7b9b82a3cf55c24cb50375b603f34d` |
| 16-byte-aligned programming image | 2,736 bytes | `c8dbcc5ce07e86504f522cde0463f45abb7b9b82a3cf55c24cb50375b603f34d` |

The payload is already 16-byte aligned; no padding bytes were needed
for this build. The inspector still rejects any unexpected load or
placement and emits a programming image rounded to that granularity.

The uncommitted source was built on repository `HEAD`
`190450cc3bac04ffb30ab189b2e55a404e466a26`. SHA-256 of
`src/main.rs` is
`f1b06afdd6b4f5358d2aae1fa994067949a7ae4d606bc242c52afe05eacac72e`;
`src/transition.s` is
`fdeeaf083f01291d071669b64e9e7f8494ab6a3c914954094c594172d1743362`;
`inspect-elf.py` is
`8d1c10276696128ae92c40709ea2110e9e12f0e5f7e3883bdf78c195c13278b7`.
These hashes identify this implementation without asserting a committed
source revision. Target-register operations use narrow `unsafe` blocks
adapted from the earlier standalone MPU experiments. Their suitability
requires human review under `AGENTS.md` before this implementation is
declared ready for deployment.

These are build observations, not board observations. Final status
and checks are recorded in the ledger.

## Pending, separately authorized board execution

Human approval for deployment is required before these steps. The
operator should confirm board identity, revision, original option
bytes (read only), recovery image, tool versions, UTC time, and
macOS image hashes. Program the independently reproduced image at
`0x08000000`, with verification enabled and automatic run disabled.
An authorized macOS operator can use the following command, after
confirming that the CLI will erase only the affected first 8 KiB page:

```sh
STM32_Programmer_CLI -c port=SWD mode=UR freq=1000 \
  -w /tmp/flt-01-001-image.bin 0x08000000 -v
```

The `-w <file> <address> -v` sequence follows
[UM2237 Rev 31 §3.2.3](https://www.st.com/resource/en/user_manual/dm00403500-stm32cubeprogrammer-software-description-stmicroelectronics.pdf).
Do not mass-erase or write option bytes. Disconnect, press
physical RESET, and observe LD1 for at least ten seconds.

Without resetting or halting, use SWD Hot Plug. CLI lengths are bytes:

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

The last read covers SHCSR, CFSR, and HFSR. Repeat all reads after a
second physical hardware reset. Preserve both complete raw dumps,
dates, image identity, LED observations, any unexpected status, and
external SCB readings in a new evidence record. Do not infer a PASS
from LED alone.

If the board fails, preserve diagnostics and use the established
[board recovery policy](README.md#5-development-safety-policy):
reconnect via SWD, try Connect Under Reset if Hot Plug fails, restore
the reviewed known-good blink image, verify programming, hardware
reset, and confirm its LD1 baseline. Leave option bytes unchanged.
No programming or board observations were performed for this
implementation task.
