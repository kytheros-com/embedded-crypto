# FLT-01-003 — Privileged Thread-mode MSP UDF UsageFault evidence

**Hardware execution date:** 2026-11-27

**Result:** PASS — two physical-reset executions met the procedure

**Scope:** Standalone Platform Integration Layer (PIL) mechanism test

## Objective and evidence basis

This experiment executes T1 `UDF #0x42` in privileged Thread mode
using MSP. Unlike [FLT-01-002](2026-11-26-cortex-m33-usagefault-undefined-instruction-privileged.md),
the basic exception frame and handler software stack occupy the same
MSP reserve. The committed [procedure](../2026-11-27-cortex-m33-usagefault-undefined-instruction-msp-procedure.md)
defines prospective acceptance criteria. The [source](../usagefault-undefined-instruction-msp/)
defines the terminal handler and 49-word record; the
[ledger](../test-ledger.md) indexes this result.

The human operator reported firmware deployment, two independent
physical RESET executions, and a physical Flash upload. The supplied
raw values below were reviewed against the procedure; this record's
author did not operate the board or witness either reset. ChatGPT's
reported independent macOS review included the supplied complete
readelf symbol, section and program-header dump, disassembly, ELF
inspector output, source and artifact digests, option-byte display,
and both hardware captures. Those complete macOS transcripts were
not deposited in this repository. The full macOS build-check transcript
was not supplied, so individual macOS check results are not inferred.

## Board, configuration, and provenance

| Item | Operator-reported value or evidence |
|---|---|
| Board; MCU | NUCLEO-U5A5ZJ-Q; STM32U5A5ZJ |
| Device ID; silicon revision | `0x481`; X |
| Processor | Cortex-M33 r0p4, confirmed by the independent CPUID reads below |
| Device name; NVM; bootloader | STM32U5xx; 4 MBytes; version `0x92` |
| ST-LINK | V3J10M3; serial `003800353335511835383531` |
| Voltage; SWD | 3.27 V; 1000 kHz |
| Programmer | STM32CubeProgrammer 2.24.0 |
| Execution | Operator-reported deployment and two physical RESET operations |
| Security baseline | Fresh read-only option-byte display matched the original record; no option-byte modification |

Before programming, the operator obtained a fresh read-only option-byte
display with `stm32cli -c port=SWD mode=UR freq=1000 -ob displ`.
All displayed fields matched the [original option-byte record](../stm32u5a5zj-original-option-bytes.txt):
RDP=`0xAA` (Level 0), TZEN=0, SRAM_RST=1, SRAM2_RST=1,
SWAP_BANK=0, nSWBOOT0=1, nBOOT0=1, and decoded
NSBOOTADD0=`0x08000000`. The original write-protection ranges remained
inactive and unchanged. This establishes the option-byte BOOT0
selection policy, not the physical BOOT0 pin voltage. No separately
measured BOOT0 voltage, recovery-image availability check, Flash
page-impact check, or programming-verification transcript was supplied.
Successful deployment was reported by the operator and corroborated by
the later physical Flash read-back.

At final evidence collection, the operator reported a clean macOS
checkout on `hardware/NUCLEO-U5A5ZJ-Q` at commit
`2f21a2df76ff17d50c896c98fb4a4b2913996411`. Its seven supplied
source SHA-256 digests match the current tracked source and the Linux
reference-build digests recorded in the prospective procedure:

| Source file | SHA-256 |
|---|---|
| `src/main.rs` | `ef90e360e4e15f40d56f14d42fce0b86bc607ee32e6a058d6d870e8f729898cd` |
| `src/transition.s` | `ff27c92e5eda836e8d706242eeaad55d4096fabc9ce9df00703293a9ea0580f6` |
| `inspect-elf.py` | `064f7916f44d5c01a00736e6b5b91e34b039b971e246c096b02a3e20aadad47f` |
| `Cargo.toml` | `c25fe8a2c4db617a1966048bdca204619236c967e97170229cb942c40fc38739` |
| `Cargo.lock` | `fc2349e9fccd381d56bfaaadd6ec2360bec77a9b3f424e0125f2fed3691ed338` |
| `build.rs` | `e03c4bf416547fd9c44ee88628f3b5c8010a3e18a9930c8500f9aeac65f12804` |
| `memory.x` | `c0bc3b0b75924e76956d473c8e045afae7f03f597129bf387ef93dfa2e2f67b8` |

The macOS host was version 27.2.1, build 26B131, arm64. Reported
tools were rustc 1.98.1 (`48a229cea`, 2026-09-01), cargo 1.98.1
(`797e8a9bc`, 2026-08-05), Python 3.14.7, GNU objcopy
2.47.20260726, and target `thumbv8m.main-none-eabi`. The Linux
reference ELF was built earlier from the then-uncommitted implementation
at repository HEAD `d0c4d0ed5d94bedd5cf72de74a89c2f0b808c3e7`;
its provenance is not retrospectively assigned to the macOS commit.

The reported macOS ELF inspector output was:

```text
flash: [0x08000000, 0x08002000) RBAR=0x08000006 RLAR=0x08001fe1
user: [0x20001000, 0x20002000) RBAR=0x20001003 RLAR=0x20001fe1
supervisor: [0x20002000, 0x20003000) RBAR=0x20002001 RLAR=0x20002fe1
entry=0x08000259 UsageFault=0x08000309 UDF=0x080002f2 continuation=0x080002f4
UDF halfword: 0xDE42
```

The inspected macOS machine code showed startup using 84 bytes of
software stack and restoring its allocation before return. Assembly
then set MSP=`0x20003000`, CONTROL=0, executed ISB and branched to
the stackless probe. The UsageFault wrapper at `0x08000308` captured
and validated MSP, SP and EXC_RETURN without stack writes; its failure
branch loops without entering Rust. `record_usagefault` at
`0x080004F4` pushes eight bytes and allocates sixteen more. Its only
explicit call is at `0x080006DE` to `indicate_success` at
`0x0800033C`, which pushes eight bytes and makes no further call or
stack adjustment. All function-entry SPs on the intended path are
eight-byte aligned. The maximum handler software stack use is 32
bytes. With the 32-byte basic hardware frame, total consumption is
64 bytes: minimum MSP=`0x20002FC0`, leaving 1,984 bytes of the
2,048-byte reserve `[0x20002800,0x20003000)`. No intended handler-path
store overlaps `[0x20002FE0,0x20003000)`.

The reviewed frame-inspection instructions were:

```text
080005E0: ldrd sl, r2, [lr, #-32]
080005E4: movs r6, #1
080005E6: ldrd r8, r3, [lr, #-24]
080005EA: ldrd r4, r5, [lr, #-16]
080005EE: ldrd r0, r1, [lr, #-8]
```

Here LR holds `0x20003000`; these are read-only software inspection
loads, not architectural exception unstacking. This is a static
assessment of the exact supplied optimized machine code under the
controlled assumptions, not a runtime high-water measurement or a
bound for arbitrary faults and nested exceptions.

Independent GNU objcopy payload comparison passed. The reported
artifacts were:

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| macOS ELF | 141,528 | `c03f0bf0fa32c9b557e61f0a056ff1d95612d1aefb54332e9e1c40626d6bff62` |
| Linux reference ELF | 141,516 | `c72af70d1ff78a08171ffa0752eb63f4cfeecee593159b04937a2e5ef9835b30` |
| Linux and macOS Flash payloads | 2,624 each | `ba6ab1858d92fbfc8009a4b7c54610933300c4f60bb64ce77f4ea9a3cd8adce1` |
| Linux and macOS programming images | 2,624 each | `ba6ab1858d92fbfc8009a4b7c54610933300c4f60bb64ce77f4ea9a3cd8adce1` |
| Physical Flash read-back | 2,624 | `ba6ab1858d92fbfc8009a4b7c54610933300c4f60bb64ce77f4ea9a3cd8adce1` |

The ELF files differ by 12 bytes; the reason was not investigated.
The Flash payload fills the 2,624-byte programming image, so no
padding was needed. The operator reported `--upload` of 2,624 bytes
from `0x08000000` and a byte-for-byte comparison with the programming
image. This verifies that Flash range at collection time, not the
entire 4-MByte NVM.

## Two physical RESET executions: complete raw observations

**Run 1:** first independent physical RESET, qualitative repeating
three-slow-green-LD1-flash pattern with a longer pause. Evidence
collection began `2026-11-27T17:03:03Z`.

### Run 1 diagnostic record — 196 bytes, 49 words

```text
0x20002000 : 00000001 00000004 00000000 00000000
0x20002010 : 20003000 00000800 00000000 00000005
0x20002020 : 00000044 00000000 08000006 08001FE1
0x20002030 : 20001003 20001FE1 20002001 20002FE1
0x20002040 : 00000000 00000000 00000000 00050000
0x20002050 : 080002F2 0000DE42 A5C35A3C 00000001
0x20002060 : 00000000 00000000 00000006 20002FE0
0x20002070 : 20002FE0 FFFFFFB8 20002FE0 00000000
0x20002080 : 00000000 00000000 00050008 00010000
0x20002090 : 00000000 00000001 00000000 00000000
0x200020A0 : 20003000 20002FE1 08000006 08001FE1
0x200020B0 : 080002F2 29000000 0000DE42 A5C35A3C
0x200020C0 : 00000000
```

### Run 1 independent SWD reads — 15 words

```text
0x20002FE0 : 00000000 00000000 20003000 20002FE1
0x20002FF0 : 08000006 08001FE1 080002F2 29000000
0x200020C4 : A5C35A3C
0x20001000 : 00000001 00000000
0xE000ED24 : 00050008 00010000 00000000
0xE000ED00 : 410FD214
```

**Run 2:** second independent physical RESET, the same qualitative
three-flash pattern. Evidence collection began
`2026-11-27T17:06:46Z` and ended
`2026-11-27T17:06:48Z`.

### Run 2 diagnostic record — 196 bytes, 49 words

```text
0x20002000 : 00000001 00000004 00000000 00000000
0x20002010 : 20003000 00000800 00000000 00000005
0x20002020 : 00000044 00000000 08000006 08001FE1
0x20002030 : 20001003 20001FE1 20002001 20002FE1
0x20002040 : 00000000 00000000 00000000 00050000
0x20002050 : 080002F2 0000DE42 A5C35A3C 00000001
0x20002060 : 00000000 00000000 00000006 20002FE0
0x20002070 : 20002FE0 FFFFFFB8 20002FE0 00000000
0x20002080 : 00000000 00000000 00050008 00010000
0x20002090 : 00000000 00000001 00000000 00000000
0x200020A0 : 20003000 20002FE1 08000006 08001FE1
0x200020B0 : 080002F2 29000000 0000DE42 A5C35A3C
0x200020C0 : 00000000
```

### Run 2 independent SWD reads — 15 words

```text
0x20002FE0 : 00000000 00000000 20003000 20002FE1
0x20002FF0 : 08000006 08001FE1 080002F2 29000000
0x200020C4 : A5C35A3C
0x20001000 : 00000001 00000000
0xE000ED24 : 00050008 00010000 00000000
0xE000ED00 : 410FD214
```

These are collection timestamps, not exact RESET times. LED periods
were not measured. The supplied raw address/value dumps do not imply
that omitted CLI transcript lines were saved as repository files.
Each run contains 64 observed words: 49 diagnostic, eight direct
frame, one sentinel, two markers, three SCB, and one CPUID. All 64
words agree between the two runs.

## Diagnostic decode and acceptance

Offsets are bytes from `0x20002000`. Each word is a 32-bit
little-endian value, with bit 0 least significant. Incidental values
are retained without new fixed acceptance criteria.

| Offset | Field | Raw value | Assessment |
|---|---|---|---|
| `00` | Status | `00000001` | Internal checks passed. |
| `04` | Phase | `00000004` | Validated fault path reached. |
| `08` | StartControl | `00000000` | Startup privileged Thread/MSP. |
| `0C` | StartIpsr | `00000000` | Startup Thread mode. |
| `10` | StartMsp | `20003000` | Aligned MSP at reserve top. |
| `14` | MpuType | `00000800` | Eight MPU regions reported. |
| `18` | InitialMpuCtrl | `00000000` | MPU initially disabled. |
| `1C` | MpuCtrl | `00000005` | MPU enabled with privileged background map. |
| `20` | Mair0 | `00000044` | Expected memory attribute. |
| `24` | Mair1 | `00000000` | No second attribute used. |
| `28` | FlashRbar | `08000006` | Expected Flash base and permissions. |
| `2C` | FlashRlar | `08001FE1` | Flash through `08001FFF`. |
| `30` | UserRbar | `20001003` | User SRAM RW/XN. |
| `34` | UserRlar | `20001FE1` | User through `20001FFF`. |
| `38` | SupervisorRbar | `20002001` | Supervisor-only SRAM RW/XN. |
| `3C` | SupervisorRlar | `20002FE1` | Supervisor through `20002FFF`. |
| `40` | UnusedEnabledMask | `00000000` | Regions 3–7 disabled. |
| `44` | InitialCfsr | `00000000` | No initial configurable fault bits. |
| `48` | InitialHfsr | `00000000` | No initial HardFault bits. |
| `4C` | ArmedShcsr | `00050000` | MemManage and UsageFault enabled. |
| `50` | ExpectedUdfAddress | `080002F2` | Linked UDF; inspector agrees. |
| `54` | ExpectedUdfEncoding | `0000DE42` | T1 `UDF #0x42`. |
| `58` | SentinelBefore | `A5C35A3C` | Synthetic protected value. |
| `5C` | ProgressMarker | `00000001` | Arithmetic and SRAM-store controls completed. |
| `60` | ContinuationMarker | `00000000` | No continuation recorded. |
| `64` | HandlerControl | `00000000` | Handler CONTROL in this setup. |
| `68` | HandlerIpsr | `00000006` | UsageFault exception number 6. |
| `6C` | HandlerSp | `20002FE0` | Entry SP equals frame bottom. |
| `70` | HandlerMsp | `20002FE0` | Original entry MSP equals SP. |
| `74` | ExcReturn | `FFFFFFB8` | Basic MSP frame; interrupted Thread. |
| `78` | FrameMsp | `20002FE0` | Captured basic-frame address. |
| `7C` | Primask | `00000000` | No PRIMASK inhibition. |
| `80` | Basepri | `00000000` | No BASEPRI threshold. |
| `84` | Faultmask | `00000000` | No FAULTMASK inhibition. |
| `88` | Shcsr | `00050008` | Exact enables and UsageFault active bit. |
| `8C` | Cfsr | `00010000` | `UNDEFINSTR` alone. |
| `90` | Hfsr | `00000000` | No HardFault escalation. |
| `94` | FrameValid | `00000001` | Candidate-frame inspection prerequisites passed. |
| `98` | StackedR0 | `00000000` | Probe copy: pre-fault CONTROL=0. |
| `9C` | StackedR1 | `00000000` | Probe copy: pre-fault IPSR=0. |
| `A0` | StackedR2 | `20003000` | Probe copy: active MSP before fault. |
| `A4` | StackedR3 | `20002FE1` | Incidental; retained raw. |
| `A8` | StackedR12 | `08000006` | Incidental; retained raw. |
| `AC` | StackedLr | `08001FE1` | Incidental; retained raw. |
| `B0` | StackedPc | `080002F2` | Labeled UDF, not continuation. |
| `B4` | StackedXpsr | `29000000` | Thumb/Thread/no alignment padding. |
| `B8` | UdfEncodingReadback | `0000DE42` | Handler-read instruction. |
| `BC` | SentinelAfter | `A5C35A3C` | Protected value unchanged. |
| `C0` | UnexpectedIpsr | `00000000` | No unexpected handler recorded. |

The expected eight-word basic frame spans
`[0x20002FE0,0x20003000)` inside the MSP reserve. In each run, the
independent physical words at `0x20002FE0`–`0x20002FFC` equal the
handler's diagnostic copy at record offsets `98`–`B4`, word for
word. The independent sentinel, markers, and SCB reads equal record
offsets `BC`, `5C`–`60`, and `88`–`90`, respectively. The independent
CPUID read is external to the record. For stacked xPSR
`0x29000000`, `xPSR & 0x010003FF = 0x01000000`: Thumb bit 24 is
set; interrupted IPSR bits [8:0] and alignment bit 9 are clear.
Stacked R3, R12, LR and condition flags remain incidental.

The stacked PC is the inspector-identified UDF at `0x080002F2`,
not the continuation at `0x080002F4`. Marker 1 confirms the positive
arithmetic/SRAM-store controls; continuation marker 0 shows no
recorded execution past UDF. Exact SHCSR=`0x00050008`,
CFSR=`0x00010000` (`UNDEFINSTR` only), and HFSR=0 show the expected
UsageFault classification without observed HardFault escalation.
The sentinel remained `0xA5C35A3C`. `FrameValid=1` means the
candidate basic frame passed the handler's structural inspection
prerequisites; it is not a general authenticity guarantee or complete
acceptance by itself. Status=1 and Phase=4 follow the additional PC,
xPSR, opcode, marker and status checks.

CPUID `0x410FD214` decodes as IMPLEMENTER=`0x41` (Arm),
VARIANT=`0` (revision 0), bits [19:16]=`0xF`,
PARTNO=`0xD21` (Cortex-M33), and REVISION=`4` (patch 4):
Cortex-M33 r0p4. [ARM-TRM, 100230_0004_00_en, r0p4, §B2.3,
printed p. B2-51]. This processor identity is distinct from the
MCU's silicon revision X.

All prospective controlled-field criteria are satisfied separately
in both runs. The direct frame-to-copy match, independent status
reads, two qualitative LED observations, reproducible 64-word
captures, and exact physical Flash read-back support **PASS** for
FLT-01-003. No incidental register or flag value was made into a
retrospective criterion.

## Architectural interpretation and limits

`CONTROL=0` before UDF selects privileged Thread mode and MSP;
CONTROL.FPCA=0 indicates no active FP context. The assembly checks
and inspected code establish the intended probe and exception-entry
stack choice under cooperating experiment software. Armv8-M
architectural exception entry uses a full descending stack and
creates the eight-word basic state frame in this configuration;
EXC_RETURN=`0xFFFFFFB8` encodes interrupted Thread/MSP and basic
frame type. A synchronous UDF stacks the faulting instruction PC.
[ARM-ARM, DDI0553B.z, §B3.4, printed p. 73, RGKVQ, RSNGJ,
RHVGB, INMBL; §B3.18, printed pp. 104–105, RWYWW, RQLHB;
§B3.19, printed pp. 106–109, RQDKQ, RMFPR, RPWWG, RSGKY,
RPBVF; §D1.2.96, printed pp. 1628–1629; §C2.4.259,
printed p. 971]. The [procedure's reference review](../2026-11-27-cortex-m33-usagefault-undefined-instruction-msp-procedure.md#objective-and-reference-basis)
also records the applicable processor and silicon errata assessment.

PM0264 Rev. 8 p. 39 broadly describes the stacked return address as
the next instruction. That wording remains discrepant with ARM-ARM
rule RQLHB for a synchronous fault. The observed stacked UDF PC
supports the specific architectural rule in this experiment; it does
not resolve the document-control discrepancy.

This result establishes successful exception entry and controlled
read-only frame inspection while the handler uses the same MSP
allocation. The independent final frame read corroborates frame
preservation while the terminal handler is active; it cannot prove
the absence of transient overwrites. It does not establish safe
inspection of corrupted or incomplete frames, stack-exhaustion
tolerance, nested-exception bounds, exception return or recovery,
production handler suitability, general execution-domain
attribution, TrustZone isolation, or Level 4 fault-induction
resistance. The unexpected-exception handler still assumes a usable
MSP before its software prologue.

The architecture calls for PIL-enforced isolation and trusted fault
entry, but this experiment does not show those broader invariants
have been implemented. The three MPU-region settings and fault-enable
bits were observed for this run; the code's 8-byte call alignment and
R0/R1/R2 probe convention are cooperating-software ABI choices, not
independently enforced security properties. The probe placed
pre-fault CONTROL, IPSR and SP into R0/R1/R2 as controlled ground
truth. Those stacked values are not independently trustworthy
diagnostics for adversarial production software. Human engineering
review remains necessary before using this evidence in a PIL design;
PASS does not imply cryptographic-module validation or production
readiness.
