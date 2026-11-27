# FLT-01-004 — Conditional integer divide-by-zero UsageFault

**Date:** 2026-11-27

**Lifecycle:** IMPLEMENTED; human review and physical execution pending

**Scope:** Standalone PIL hardware characterization

## Objective and authority

Execute the same explicit unsigned `UDIV r6, r3, r12` with divisor
zero in two separately built configurations. First run the
`trap-disabled` control with CCR.DIV_0_TRP=0; then run the default
trap-enabled stimulus with CCR.DIV_0_TRP=1. In both, the assembly
probe runs in unprivileged Thread mode on PSP. The configured CCR bit
is the intended experimental variable. No debugger write, SVC,
exception return, retry, or recovery changes that bit. The
[ledger](test-ledger.md) records lifecycle, not hardware acceptance.
The [architecture](../../docs/assurance/architecture.md) §§4.2, 5.2
assigns platform exception mechanisms to PIL; the
[MCL FSM](../../docs/design/mcl-fsm.md) §7 leaves recovery realization
open. This is not a production handler.

## Checked local references and interpretation

The ignored local PDFs and their revisions/digests are identified in
the [reference catalogue](references.md). These coordinates were
checked in the local copies:

- [ARM-ARM, DDI0553B.z, §C2.4.260, printed p. 972]: UDIV T1
  encoding, unsigned quotient, and the zero-divisor branch. The
  specified register fields yield halfwords `FBB3 F6FC` for
  `UDIV r6,r3,r12`, stored as bytes `B3 FB FC F6` in little-endian
  Flash. The zero-divisor branch writes result zero when trapping is
  disabled; with trapping enabled it calls GenerateIntegerZeroDivide.
- [ARM-ARM, DDI0553B.z, §B3.12, printed p. 91]: divide-by-zero
  UsageFault classification when CCR.DIV_0_TRP=1.
  [CCR.DIV_0_TRP, §D1.2.9, printed p. 1474]: bit 4 and warm-reset
  value zero. [UFSR.DIVBYZERO, §D1.2.270, printed p. 1897]: UFSR
  bit 9, corresponding to CFSR bit 25 (`0x02000000`).
- [ARM-ARM, DDI0553B.z, §E2.1.186, printed p. 2002]:
  GenerateIntegerZeroDivide sets DIVBYZERO and creates UsageFault.
  [§E2.1.234, printed p. 2023]: IntegerZeroDivideTrappingEnabled
  reads CCR.DIV_0_TRP. [§B3.18, printed p. 105, RQLHB]: a
  synchronous fault stacks its faulting instruction address.
- [ARM-ARM, DDI0553B.z, §§B3.18–B3.19, printed pp. 104–109,
  RWYWW, RQDKQ, RPWWG, RSGKY, RPBVF; §D1.2.96,
  printed pp. 1628–1629]: exception entry, full-descending basic
  frame, inactive FP context, alignment, and EXC_RETURN.
- [PM0264, Rev. 8, §§4.3.9, 4.3.12.3, printed pp. 195, 202]:
  DIV_0_TRP control and DIVBYZERO status. [ARM-TRM,
  100230_0004_00_en, r0p4, §B2.3, printed p. B2-51]: CPUID.
- [ARM-ERRATA, SDEN-756493 v14.0, summary printed pp. 10–11,
  errata 1080541, 1435973, 2219175]; [ES0553, Rev. 6,
  Table 2 and §2.1.1, printed pp. 2, 5]: applicable r0p4 and
  silicon-X limitations were reviewed. Their unaligned Device
  access, interrupt-priority update, and partial VLLDM conditions
  do not match this aligned integer instruction path, static
  exception-priority setup, and inactive FP context. No workaround
  is introduced.

The UDIV zero-divisor behavior and synchronous fault PC are
architectural rules. Exact frame address, EXC_RETURN, SCB values,
stack use, and LED indication are predictions for the controlled
board configuration. PM0264 Rev. 8 p. 39 broadly calls the stacked
return address the next instruction; ARM-ARM rule RQLHB specifically
selects the causing instruction for this synchronous fault. Keep
that wording discrepancy visible during review.
PM0264 Rev. 8 §4.3.9, printed p. 194 calls CCR a read-only register,
despite describing writable DIV_0_TRP on p. 195. ARM-ARM
DDI0553B.z §D1.2.9, printed p. 1471 defines privileged word
read/write access. This is a documentation discrepancy; the build
uses the architectural read-modify-write contract and requires
physical CCR readback before acceptance.

## Board configuration and memory layout

Use NUCLEO-U5A5ZJ-Q, STM32U5A5ZJ silicon revision X,
Cortex-M33 r0p4, original option bytes, TZEN=0, RDP Level 0,
Rust 1.98.1, `thumbv8m.main-none-eabi`. No active FP context or
ordinary interrupt source is enabled. MemManage and UsageFault are
enabled with reset-default priorities. SysTick is only polled after
the enabled-trap handler accepts; its interrupt remains disabled.
Preserve MPU_CTRL=5, MAIR0=`0x44`, MAIR1=0, regions 3–7 disabled:

| Region | Half-open interval | RBAR | RLAR |
|---|---|---|---|
| Flash | `08000000–08002000` | `08000006` | `08001FE1` |
| User SRAM | `20001000–20002000` | `20001003` | `20001FE1` |
| Supervisor SRAM | `20002000–20003000` | `20002001` | `20002FE1` |

Supervisor record `[20002000,200020E8)` has 58 little-endian
32-bit words, bit 0 least significant. Sentinel is at `200020E8`.
MSP reserve remains `[20002800,20003000)`.
User diagnostic words occupy `[20001000,20001014)`; the aligned PSP
reserve begins at `20001018` and ends at `20002000`. The enabled
basic frame is predicted at `[20001FE0,20002000)`. The disabled
run creates no frame. Neither record nor sentinel overlaps either
stack reserve. Startup explicitly clears all 58 record words and
initializes all five user words and the sentinel on every reset;
it does not rely on retained SRAM being erased.

| User address | Meaning | Startup | Disabled acceptance | Enabled acceptance |
|---|---|---:|---:|---:|
| `20001000` | Progress | 0 | 1 | 1 |
| `20001004` | Continuation | 0 | 2 | 0 |
| `20001008` | Positive quotient | 0 | 6 | 6 |
| `2000100C` | Zero quotient | `A55A5AA5` | 0 | `A55A5AA5` |
| `20001010` | Failure marker | 0 | 0 | 0 |

Assembly captures pre-stimulus CONTROL/IPSR/SP in R0/R1/R2,
checks `3/0/20002000`, then executes `42 / 7` with R3=42,
R12=7, R6 destination. It requires quotient 6 and stores it.
It resets R3=42 and R12=0, seeds R6=`0x5A`, stores progress 1,
executes DSB and ISB, then runs the second, identically encoded
UDIV at the linked fault label. Completion stores actual R6 at
`2000100C`, executes DSB, publishes marker 2 and executes DSB
before a terminal unprivileged loop. Probe failure stores marker 3
at `20001010` and loops. There is no probe stack use. The stacked
R0/R1/R2 copies are controlled experimental ground truth, not
independently trusted production diagnostics. The stacked R6
destination is not captured and has no acceptance value.

Privileged startup reads and records initial CCR, modifies only
DIV_0_TRP bit 4, executes DSB/ISB, reads configured CCR, and rejects
wrong bit or changes to any other readable CCR bit. Require initial
CCR.DIV_0_TRP=0 after physical RESET, configured bit 0 for the
disabled build and bit 1 for enabled; retain all other CCR bits as
observed rather than assuming a complete fixed CCR word.

## Supervisor diagnostic word map

The first 49 offsets retain the FLT-01-001 frame/status meaning where
applicable. `ExpectedUdivFirst` and `UdivFirstReadback` replace the
old UDF halfword fields; new fields follow at `C4`–`E4`.
Fields not written by a handler remain zero in the disabled control
because startup clears the record. In that control, external user
memory and SCB reads establish completion; supervisor Status=2 means
**configured control**, not completed division. Phase remains 3.
Unexpected exceptions overwrite Status with a failure code. Enabled
success requires Status=1 and Phase=4.

| Offset | Field | Disabled expectation | Enabled expectation |
|---|---|---|---|
| `00` | Status | 2 | 1 |
| `04` | Phase | 3 | 4 |
| `08` | StartControl | 0 | 0 |
| `0C` | StartIpsr | 0 | 0 |
| `10` | StartMsp | aligned in MSP reserve | same |
| `14` | MpuType | `00000800` | same |
| `18` | InitialMpuCtrl | 0 | same |
| `1C` | MpuCtrl | 5 | same |
| `20` | Mair0 | `44` | same |
| `24` | Mair1 | 0 | same |
| `28,2C` | FlashRbar/Rlar | `08000006/08001FE1` | same |
| `30,34` | UserRbar/Rlar | `20001003/20001FE1` | same |
| `38,3C` | SupervisorRbar/Rlar | `20002001/20002FE1` | same |
| `40` | UnusedEnabledMask | 0 | same |
| `44,48` | InitialCfsr/Hfsr | observed raw; cleared before probe | same |
| `4C` | ArmedShcsr | `00050000` | same |
| `50` | ExpectedUdivAddress | `08000302` | same |
| `54` | ExpectedUdivFirst | `0000FBB3` | same |
| `58` | SentinelBefore | `A5C35A3C` | same |
| `5C,60` | Progress/Continuation copy | 0/0; handler not entered | 1/0 |
| `64` | HandlerControl | 0; handler not entered | 1 |
| `68` | HandlerIpsr | 0 | 6 |
| `6C,70` | HandlerSp/Msp | 0/0 | `20003000/20003000` |
| `74` | ExcReturn | 0 | `FFFFFFBC` |
| `78` | FramePsp | 0 | `20001FE0` |
| `7C,80,84` | Primask/Basepri/Faultmask | 0/0/0 | 0/0/0 |
| `88` | Shcsr | 0; handler not entered | `00050008` |
| `8C` | Cfsr | 0 | `02000000` |
| `90` | Hfsr | 0 | 0 |
| `94` | FrameValid | 0 | 1 |
| `98` | StackedR0 | 0; no frame | 3 |
| `9C` | StackedR1 | 0; no frame | 0 |
| `A0` | StackedR2 | 0; no frame | `20002000` |
| `A4` | StackedR3 | 0; no frame | 42 |
| `A8` | StackedR12 | 0; no frame | 0 |
| `AC` | StackedLr | 0; no frame | incidental raw |
| `B0` | StackedPc | 0; no frame | `08000302` |
| `B4` | StackedXpsr | 0; no frame | Thumb/Thread/no padding |
| `B8` | UdivFirstReadback | 0 | `0000FBB3` |
| `BC` | SentinelAfter | 0 | `A5C35A3C` |
| `C0` | UnexpectedIpsr | 0 | 0 |
| `C4` | InitialCcr | raw; bit 4 zero | same |
| `C8` | ConfiguredCcr | initial with bit 4 clear | initial with bit 4 set |
| `CC` | HandlerCcr | 0; handler not entered | equals configured CCR |
| `D0` | BuildMode | 0 | 1 |
| `D4` | PositiveQuotient copy | 0; handler not entered | 6 |
| `D8` | ZeroQuotient copy | 0; handler not entered | `A55A5AA5` |
| `DC` | FailureMarker copy | 0 | 0 |
| `E0` | ExpectedUdivSecond | `0000F6FC` | same |
| `E4` | UdivSecondReadback | 0 | `0000F6FC` |

The current ELF inspector derives the linked UDIV address, verifies
both four-byte instructions, and checks continuation=`08000306`.
The table's fixed addresses apply only to the inspected artifacts;
if a new build moves them, update the prospective comparison before
deployment. The complete enabled frame at offsets `98`–`B4` is
R0,R1,R2,R3,R12,LR,PC,xPSR. xPSR must satisfy
`xPSR & 0x010003FF == 0x01000000`: Thumb bit 24 set,
interrupted IPSR bits [8:0] zero, and alignment bit 9 zero.
No fixed criterion applies to incidental LR or condition flags.
`FrameValid=1` permits inspection of a candidate basic frame; it
does not by itself establish full acceptance or authenticity.

## Prospective physical acceptance

**Disabled control:** require supervisor Status=2, Phase=3,
BuildMode=0, expected startup/MPU fields, CCR initial bit 4 zero,
configured/external CCR bit 4 zero with other readable bits
preserved. Require user words `1,2,6,0,0`, external
SHCSR/CFSR/HFSR=`00050000,00000000,00000000`, unchanged sentinel,
and no unexpected handler indicator. The Thread remains unprivileged
on PSP in its terminal loop, based on inspected code and controlled
handoff. No exception frame is expected; residual PSP SRAM must not
be decoded as one. LD1 may remain dark; darkness alone proves
nothing. Common startup drives LD1 off before either probe; no
privileged transition is used to signal disabled-control completion.
Status=2 is not a misleading success signal because all
external completion conditions are mandatory.

**Enabled stimulus:** require supervisor Status=1, Phase=4,
BuildMode=1, expected startup/MPU fields and user words
`1,0,6,A55A5AA5,0`. Require initial CCR bit 4 zero,
configured/handler/external CCR bit 4 one and other readable bits
unchanged. Handler CONTROL=1, IPSR=6, entry SP/MSP=`20003000`,
EXC_RETURN=`FFFFFFBC`, FramePsp=`20001FE0`, FrameValid=1,
PRIMASK=BASEPRI=FAULTMASK=0. Require exact
SHCSR/CFSR/HFSR=`00050008,02000000,00000000` with no other
stacking/unstacking fault bits, stacked R0/R1/R2/R3/R12
=`3,0,20002000,42,0`, stacked PC=`08000302`, xPSR mask as
above, both UDIV halfwords, unchanged sentinel, and unexpected
IPSR=0. Directly read the eight physical PSP frame words after the
success indication and compare each with record offsets `98`–`B4`.
Three slow green LD1 flashes (nominally 500 ms on/off and longer
dark interval, repeated) corroborate but cannot establish PASS.
Failure holds LD1 solid when GPIO and the stack remain usable;
the stackless entry failure loops without a complete record.

Both builds require independent external user, sentinel, CCR,
SHCSR/CFSR/HFSR and CPUID reads. CPUID should remain
`410FD214`. Treat each of two physical RESET executions per build
independently, four total. Preserve differences in incidental
registers and flags. Reproducibility supports but does not replace
each run's acceptance comparison. Each disabled run has **69**
observed words: 58 record + 5 user + 1 sentinel + 1 CCR + 3 SCB
+ 1 CPUID. Each enabled run has **77**, adding 8 direct frame words.

## macOS build, static inspection, and provenance

From the repository root, use this consolidated block. Keep target
directories and images distinct. Review both complete disassemblies,
source, target-specific `unsafe` access, and static stack paths before
deployment. Compare source and deployable payload/image digests
across hosts; ELF hashes may differ. Run root-workspace regression
checks separately.

```sh
cd hardware/NUCLEO-U5A5ZJ-Q/usagefault-divide-by-zero
rustc +1.98.1 --version
cargo +1.98.1 --version
python3 --version
arm-none-eabi-objcopy --version | head -1
cargo +1.98.1 fmt --all --check
cargo +1.98.1 check --release --locked --target thumbv8m.main-none-eabi \
  --target-dir target/trap-disabled --no-default-features \
  --features trap-disabled
cargo +1.98.1 clippy --release --locked \
  --target thumbv8m.main-none-eabi --target-dir target/trap-disabled \
  --no-default-features --features trap-disabled -- -D warnings
RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' \
  cargo +1.98.1 doc --release --no-deps --locked \
  --target thumbv8m.main-none-eabi --target-dir target/trap-disabled \
  --no-default-features --features trap-disabled
cargo +1.98.1 build --release --locked \
  --target thumbv8m.main-none-eabi --target-dir target/trap-disabled \
  --no-default-features --features trap-disabled
cargo +1.98.1 check --release --locked --target thumbv8m.main-none-eabi \
  --target-dir target/trap-enabled
cargo +1.98.1 clippy --release --locked \
  --target thumbv8m.main-none-eabi --target-dir target/trap-enabled \
  -- -D warnings
RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' \
  cargo +1.98.1 doc --release --no-deps --locked \
  --target thumbv8m.main-none-eabi --target-dir target/trap-enabled
cargo +1.98.1 build --release --locked \
  --target thumbv8m.main-none-eabi --target-dir target/trap-enabled
elf_disabled=target/trap-disabled/thumbv8m.main-none-eabi/release/nucleo-u5a5zj-q-usagefault-divide-by-zero
elf_enabled=target/trap-enabled/thumbv8m.main-none-eabi/release/nucleo-u5a5zj-q-usagefault-divide-by-zero
python3 inspect-elf.py "$elf_disabled" --mode disabled \
  --peer "$elf_enabled" --payload /tmp/flt-01-004-disabled-payload.bin \
  --program-image /tmp/flt-01-004-disabled-image.bin
python3 inspect-elf.py "$elf_enabled" --mode enabled \
  --peer "$elf_disabled" --payload /tmp/flt-01-004-enabled-payload.bin \
  --program-image /tmp/flt-01-004-enabled-image.bin
arm-none-eabi-readelf -h -l -S -s -W "$elf_disabled"
arm-none-eabi-readelf -h -l -S -s -W "$elf_enabled"
arm-none-eabi-objdump -d "$elf_disabled"
arm-none-eabi-objdump -d "$elf_enabled"
arm-none-eabi-objcopy -O binary "$elf_disabled" \
  /tmp/flt-01-004-disabled-objcopy.bin
arm-none-eabi-objcopy -O binary "$elf_enabled" \
  /tmp/flt-01-004-enabled-objcopy.bin
cmp /tmp/flt-01-004-disabled-payload.bin \
  /tmp/flt-01-004-disabled-objcopy.bin
cmp /tmp/flt-01-004-enabled-payload.bin \
  /tmp/flt-01-004-enabled-objcopy.bin
shasum -a 256 src/main.rs src/transition.s inspect-elf.py \
  Cargo.toml Cargo.lock build.rs memory.x "$elf_disabled" \
  "$elf_enabled" /tmp/flt-01-004-disabled-payload.bin \
  /tmp/flt-01-004-disabled-image.bin \
  /tmp/flt-01-004-enabled-payload.bin \
  /tmp/flt-01-004-enabled-image.bin
git rev-parse HEAD
git status --short --branch
```

The current Linux reference builds have identical probe machine
code, positive UDIV=`080002E4`, faulting UDIV=`08000302`,
continuation=`08000306`, and vector=`08000329`. Enabled handler
prologue uses 72 bytes (8-byte push and 64-byte allocation); its
only success call adds an 8-byte push, for a controlled maximum
of 80 MSP bytes and minimum MSP=`20002FB0`. Startup uses 84 bytes
and restores them before the PSP handoff. Disabled startup also
uses 84 bytes; its unexpected UsageFault path has an 8-byte push
and cannot call the LED success function. The enabled frame loads
are four read-only `LDRD` instructions from LR holding PSP top,
with offsets -32, -24, -16, -8. This is static code inspection,
not a runtime stack high-water measurement or arbitrary-fault bound.
The build used repository HEAD
`217c0003d5c5423f393fbbb9559a8686392331a9` plus uncommitted
FLT-01-004 files, Rust/Cargo 1.98.1, Python 3.12.3 and GNU Arm
binutils 2.42. Independent GNU objcopy comparison passed for both
payloads. Temporary modified ELF copies with a changed faulting
UDIV byte and changed zero-divisor initialization were rejected by
the inspector; the release artifacts were unmodified. Both
standalone configurations passed formatting, target release check,
Clippy with warnings denied, rustdoc broken-link validation, and
release build. `cargo test --doc` could not run for this binary-only
package because it has no library target; root-workspace doctests
passed. The root workspace also passed formatting, check, Clippy,
tests, rustdoc, line-length, and supported thumbv6m checks.

| Configuration and artifact | Bytes | SHA-256 |
|---|---:|---|
| Enabled ELF | 141,492 | `5a0e3c49586105f1f784b43d56a752981c09d9f57fbcd3e0455902d5fa65759d` |
| Enabled payload | 3,092 | `ef9db9efe27fe04acf17695efc0a42a1e218d4a4624cb89b3221ff1929ccbed6` |
| Enabled programming image | 3,104 | `8fc145b1961491968cd51e32de3d7723091826a426c92076577a91e191f81f3f` |
| Disabled ELF | 141,380 | `2a42b72b30cd60be9f5acd90aa2d8a08a452c1c0df1abe8f75f6e2cc3daaa231` |
| Disabled payload | 2,196 | `a7dd87ecfd734e338b9a6e503d256b4c880f1557c247ec7dc362ff8b034c3147` |
| Disabled programming image | 2,208 | `48f6ab644492d11ed6fad9bd8aa197466a5b9f8e7da5e41be1dbd9e7d1f441f8` |

Each programming image has twelve final `FF` padding bytes to reach
a 16-byte multiple. These hashes are a Linux reference for cross-host review,
not a guarantee of identical macOS ELF files.

| Source file | SHA-256 |
|---|---|
| `src/main.rs` | `6a6112168c68c6f277ecbdabd25497e3fa1de6e932bb173e35e14391d858418c` |
| `src/transition.s` | `a76d2b3ac0a4e079ab590c35a9595729ad8f025f7142012fc5c39558831525e5` |
| `inspect-elf.py` | `ac827566a481534af03e966829857d6812b146bd4d307e7763ecf101f03f59f6` |
| `Cargo.toml` | `08e3b7833f0259eefb684f070edf7aaf684b90d12272046882de06b4621ba170` |
| `Cargo.lock` | `588c37098aebf451806f1ee0383d091fa32d787bca8e2d7c9db8743a4151a36e` |
| `build.rs` | `e03c4bf416547fd9c44ee88628f3b5c8010a3e18a9930c8500f9aeac65f12804` |
| `memory.x` | `2f8e89e9efc0b6e9255535fce0dd0d305041170e2557acaf7f690afa83ed87a5` |

## Human-operated board sequence

Physical operation requires human review and authorization. Follow
the [board recovery policy](README.md#5-development-safety-policy).
Verify the board identity, silicon revision X, original option bytes
including TZEN=0/RDP Level 0, BOOT0 selection policy, supply, tools,
recovery image, and Flash page impact before writing. The following
pre-deployment commands are read-only; retain their raw outputs:

```sh
STM32_Programmer_CLI -c port=SWD mode=UR freq=1000
STM32_Programmer_CLI -c port=SWD mode=UR freq=1000 -ob displ
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0xE000ED00 0x04
```

Run disabled first. Require reviewed cross-host payload/image digests
before each write. Program with verification and no automatic run.
Disconnect and press **physical RESET** twice independently; after
each reset collect the disabled read block below. Retain separate
raw outputs, observation times, and qualitative LD1 observations.
Before replacing this image, upload and compare physical Flash using
the disabled readback block.

```sh
STM32_Programmer_CLI -c port=SWD mode=UR freq=1000 \
  -w /tmp/flt-01-004-disabled-image.bin 0x08000000 -v
```

**Per disabled RESET run — 69 words, read lengths in bytes:**

```sh
date -u '+%Y-%m-%dT%H:%M:%SZ'
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x20002000 0xE8
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x20001000 0x14
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x200020E8 0x04
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0xE000ED14 0x04
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0xE000ED24 0x0C
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0xE000ED00 0x04
```

**Disabled image physical readback before replacement:**

```sh
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  --upload 0x08000000 2208 /tmp/flt-01-004-disabled-readback.bin
shasum -a 256 /tmp/flt-01-004-disabled-readback.bin
cmp /tmp/flt-01-004-disabled-image.bin \
  /tmp/flt-01-004-disabled-readback.bin
```

Only after retaining the disabled evidence, program the enabled
image. Disconnect and press **physical RESET** twice independently;
observe the LED success pattern and collect the enabled read block
after each reset. The frame read is required after the success
indication, while the handler remains active. Compare its eight
words with supervisor offsets `98`–`B4` for that same run. Upload
and compare physical Flash before completing evidence collection.

```sh
STM32_Programmer_CLI -c port=SWD mode=UR freq=1000 \
  -w /tmp/flt-01-004-enabled-image.bin 0x08000000 -v
```

**Per enabled RESET run — 77 words, read lengths in bytes:**

```sh
date -u '+%Y-%m-%dT%H:%M:%SZ'
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x20002000 0xE8
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x20001FE0 0x20
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x20001000 0x14
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0x200020E8 0x04
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0xE000ED14 0x04
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0xE000ED24 0x0C
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 -r32 0xE000ED00 0x04
```

**Enabled image physical readback:**

```sh
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  --upload 0x08000000 3104 /tmp/flt-01-004-enabled-readback.bin
shasum -a 256 /tmp/flt-01-004-enabled-readback.bin
cmp /tmp/flt-01-004-enabled-image.bin \
  /tmp/flt-01-004-enabled-readback.bin
git rev-parse HEAD
git status --short --branch
date -u '+%Y-%m-%dT%H:%M:%SZ'
```

The exact readback sizes above come from this Linux static build;
if macOS produces a different reviewed image size, substitute its
actual size consistently and record the discrepancy before writing.
If Hot Plug fails, use Connect Under Reset and restore the reviewed
known-good blink image, verify it, press physical RESET, and confirm
its LD1 baseline. Leave option bytes unchanged. No board operation
or hardware PASS occurred during implementation.
