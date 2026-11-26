# FLT-01-002 — Privileged Thread-mode UDF UsageFault evidence

**Hardware execution date:** 2026-11-26  
**Result:** PASS — two physical-reset executions met the procedure  
**Scope:** Standalone Platform Integration Layer (PIL) mechanism test

## Objective and evidence basis

This experiment compares the privileged Thread-mode PSP execution of
`UDF #0x42` with [FLT-01-001](2026-11-25-cortex-m33-usagefault-undefined-instruction.md),
which used unprivileged Thread mode and PSP. The committed
[procedure](../2026-11-26-cortex-m33-usagefault-undefined-instruction-privileged-procedure.md)
defines the acceptance criteria. The [source](../usagefault-undefined-instruction-privileged/)
defines the 49-word record and terminal handler. The [ledger](../test-ledger.md)
indexes this result.

The human developer supplied the raw SWD values below and reported
that both independent physical RESET executions produced identical
56-word observations and the qualitative three-flash LD1 pattern.
The human developer and ChatGPT reviewed the results. This record
independently decodes the supplied values against the procedure; its
author did not operate the board or witness either reset. The only
supplied evidence-collection interval is
`2026-11-26T18:54:58Z`–`2026-11-26T18:54:59Z`. Individual reset
timestamps were not collected.

## Board, configuration, and provenance

| Item | Operator-reported value or evidence |
|---|---|
| Board; MCU | NUCLEO-U5A5ZJ-Q; STM32U5A5ZJ |
| Device ID; silicon revision | `0x481`; X |
| Processor | Cortex-M33 r0p4, independently confirmed by CPUID below |
| NVM; bootloader | 4 MiB; version `0x92` |
| ST-LINK | V3J10M3; serial `003800353335511835383531` |
| Voltage; SWD | 3.27 V; 1000 kHz |
| Programmer | STM32CubeProgrammer 2.24.0 |
| Execution | Verified programming, then two physical RESET operations |
| Security baseline | Existing `TZEN=0`; no fresh option-byte read supplied |

The documented configuration uses the original option bytes, separate
PSP and MSP reserves, three MPU regions, enabled MemManage and
UsageFault, no active FP context, and no enabled ordinary interrupt
source. The raw record confirms the MPU registers and the fault-enable
bits. Other configuration values in this paragraph are the established
procedure baseline or operator report, not new measurements from the
supplied SWD dumps. The processor revision and MCU silicon revision
are different properties.

At final evidence collection, the operator reported a clean source
checkout synchronized with its upstream branch at commit
`ec6af514bdcf7563f5d71b173183a16ea7ff10cb`. The local tracked
source at that commit has the seven reported SHA-256 digests:

| Source file | SHA-256 |
|---|---|
| `src/main.rs` | `6d7de66ddb4102e1d74a3f466ce5326c21d7d129f43514e32022b5e2f918124a` |
| `src/transition.s` | `e50de4b685017f69c9075f0a2bae7eb904b47cce74a8c9a0f3e1d10a7966c44f` |
| `inspect-elf.py` | `5fc7e885316d483b661373fd678d5cbcd628f5391034e3a5e6224d62aa3dd3fe` |
| `Cargo.toml` | `bec23f3c63502cf61c5819ab06ac8c5caed26663ea47a24dfd7cc1e4c414ffe5` |
| `Cargo.lock` | `4bac9e9d4c5b0b230d18077107f3f0c546b6280640692d884c3cf27cb0cd856e` |
| `build.rs` | `e03c4bf416547fd9c44ee88628f3b5c8010a3e18a9930c8500f9aeac65f12804` |
| `memory.x` | `2f8e89e9efc0b6e9255535fce0dd0d305041170e2557acaf7f690afa83ed87a5` |

These digests match the Linux build provenance in the procedure. The
operator reported macOS 27.2.1 (build 26B131), arm64, rustc 1.98.1
(`48a229cea`, 2026-09-01), cargo 1.98.1 (`797e8a9bc`,
2026-08-05), Python 3.14.7, GNU objcopy 2.47.20260726, and target
`thumbv8m.main-none-eabi`. No further macOS build checks or complete
build transcript were supplied.

The independent ELF inspector passed with these reported findings:

```text
flash: [0x08000000, 0x08002000) RBAR=0x08000006 RLAR=0x08001fe1
user: [0x20001000, 0x20002000) RBAR=0x20001003 RLAR=0x20001fe1
supervisor: [0x20002000, 0x20003000) RBAR=0x20002001 RLAR=0x20002fe1
entry=0x08000259 UsageFault=0x08000309 UDF=0x080002f2 continuation=0x080002f4
UDF halfword: 0xDE42
```

Independent objcopy comparison passed. The reported artifacts were:

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| macOS ELF | 141,516 | `7819724bdcc8188b152ac0211793883c1e9451c10f4b929fcee56b7a48ed14ad` |
| Flash payload | 2,612 | `9222e1b153fa080f94df02721b4eacefe46b332d8fb233cf5244ce3607b60bfd` |
| Programming image | 2,624 | `adf41ebc0bff0914d28fcb4c48b8daefc4a777d1a093768f9f346d7b0088802f` |
| Physical Flash read-back | 2,624 | `adf41ebc0bff0914d28fcb4c48b8daefc4a777d1a093768f9f346d7b0088802f` |

The payload and image hashes match the Linux reference. The macOS ELF
differs from the Linux ELF (141,500 bytes, SHA-256
`4b11caea70ca5c52ec19632d688d6aed20c722a8397389f424230c6c7ee84cb9`).
The exact cause was not investigated. The operator used
`--upload 0x08000000 2624` for the physical Flash read-back and
reported a successful byte-for-byte comparison against the
programming image. This verifies the read-back region at collection
time, not all 4 MiB of Flash. The final 12 image bytes are padding.

## Two reset executions: complete raw observations

**Execution 1 and execution 2:** each followed a separate physical
RESET. The operator reported the same complete raw values below in
both runs: 49 diagnostic-record words, one independent sentinel word,
two independent marker words, three independent SCB words, and one
independent CPUID word — **56 words per run**. After each reset, the
operator visually observed repeating groups of three slow green LD1
flashes separated by a longer dark interval. Timing was not measured.

### Protected diagnostic record — 196 bytes

```text
0x20002000 : 00000001 00000004 00000000 00000000
0x20002010 : 20003000 00000800 00000000 00000005
0x20002020 : 00000044 00000000 08000006 08001FE1
0x20002030 : 20001003 20001FE1 20002001 20002FE1
0x20002040 : 00000000 00000000 00000000 00050000
0x20002050 : 080002F2 0000DE42 A5C35A3C 00000001
0x20002060 : 00000000 00000000 00000006 20003000
0x20002070 : 20003000 FFFFFFBC 20001FE0 00000000
0x20002080 : 00000000 00000000 00050008 00010000
0x20002090 : 00000000 00000001 00000002 00000000
0x200020A0 : 20002000 20002FE1 08000006 08001FE1
0x200020B0 : 080002F2 29000000 0000DE42 A5C35A3C
0x200020C0 : 00000000
```

### Independent SWD reads

```text
0x200020C4 : A5C35A3C
0x20001000 : 00000001 00000000
0xE000ED24 : 00050008 00010000 00000000
0xE000ED00 : 410FD214
```

The independent sentinel, marker, and SCB reads match the diagnostic
record at offsets `BC`, `5C`–`60`, and `88`–`90`, respectively.
CPUID is external to the firmware record.

## Decode and acceptance assessment

Offsets are bytes from `0x20002000`. All words are 32-bit
little-endian values; bit 0 is least significant. *Incidental* words
are preserved raw without retrospective fixed-value criteria.

| Offset | Field | Raw value | Assessment |
|---|---|---|---|
| `00` | Status | `00000001` | Internal checks passed. |
| `04` | Phase | `00000004` | Validated fault path reached. |
| `08` | StartControl | `00000000` | Startup privileged Thread, MSP. |
| `0C` | StartIpsr | `00000000` | Startup Thread mode. |
| `10` | StartMsp | `20003000` | Aligned MSP at reserve top. |
| `14` | MpuType | `00000800` | Eight MPU regions reported. |
| `18` | InitialMpuCtrl | `00000000` | MPU initially disabled. |
| `1C` | MpuCtrl | `00000005` | Enabled; privileged background map. |
| `20` | Mair0 | `00000044` | Expected memory attribute. |
| `24` | Mair1 | `00000000` | No second attribute used. |
| `28` | FlashRbar | `08000006` | Flash RX region base and permissions. |
| `2C` | FlashRlar | `08001FE1` | Flash through `08001FFF`. |
| `30` | UserRbar | `20001003` | User SRAM RW/XN. |
| `34` | UserRlar | `20001FE1` | User through `20001FFF`. |
| `38` | SupervisorRbar | `20002001` | Supervisor-only SRAM RW/XN. |
| `3C` | SupervisorRlar | `20002FE1` | Supervisor through `20002FFF`. |
| `40` | UnusedEnabledMask | `00000000` | Regions 3–7 disabled. |
| `44` | InitialCfsr | `00000000` | No initial configurable fault bits. |
| `48` | InitialHfsr | `00000000` | No initial HardFault bits. |
| `4C` | ArmedShcsr | `00050000` | MemManage and UsageFault enabled. |
| `50` | ExpectedUdfAddress | `080002F2` | Linked UDF address; inspector agrees. |
| `54` | ExpectedUdfEncoding | `0000DE42` | T1 `UDF #0x42` halfword. |
| `58` | SentinelBefore | `A5C35A3C` | Synthetic protected value. |
| `5C` | ProgressMarker | `00000001` | Positive control completed. |
| `60` | ContinuationMarker | `00000000` | No continuation recorded. |
| `64` | HandlerControl | `00000000` | Handler CONTROL under this setup. |
| `68` | HandlerIpsr | `00000006` | UsageFault exception 6. |
| `6C` | HandlerSp | `20003000` | Handler uses MSP at reserve top. |
| `70` | HandlerMsp | `20003000` | Matches active handler SP. |
| `74` | ExcReturn | `FFFFFFBC` | Basic PSP frame; Thread return. |
| `78` | FramePsp | `20001FE0` | Aligned 32-byte basic frame. |
| `7C` | Primask | `00000000` | No PRIMASK inhibition. |
| `80` | Basepri | `00000000` | No BASEPRI threshold. |
| `84` | Faultmask | `00000000` | No FAULTMASK inhibition. |
| `88` | Shcsr | `00050008` | Exact enable and active bits. |
| `8C` | Cfsr | `00010000` | `UNDEFINSTR` alone. |
| `90` | Hfsr | `00000000` | No HardFault escalation. |
| `94` | FrameValid | `00000001` | Structural inspection checks passed. |
| `98` | StackedR0 | `00000002` | Probe copy: pre-fault CONTROL=2. |
| `9C` | StackedR1 | `00000000` | Probe copy: pre-fault IPSR=0. |
| `A0` | StackedR2 | `20002000` | Probe copy: pre-fault PSP top. |
| `A4` | StackedR3 | `20002FE1` | Incidental; retained raw. |
| `A8` | StackedR12 | `08000006` | Incidental; retained raw. |
| `AC` | StackedLr | `08001FE1` | Incidental; retained raw. |
| `B0` | StackedPc | `080002F2` | Labeled UDF address. |
| `B4` | StackedXpsr | `29000000` | Thumb/Thread/no padding. |
| `B8` | UdfEncodingReadback | `0000DE42` | Handler-read instruction. |
| `BC` | SentinelAfter | `A5C35A3C` | Protected value unchanged. |
| `C0` | UnexpectedIpsr | `00000000` | No unexpected handler recorded. |

The basic frame spans `[0x20001FE0,0x20002000)` inside the linked
PSP reserve. Its eight words at record offsets `98`–`B4` correspond
to stacked R0, R1, R2, R3, R12, LR, PC, and xPSR. `FrameValid=1`
means the handler's EXC_RETURN, fault-status, PSP-bound and alignment
checks permitted frame inspection; it is not complete acceptance by
itself. The later PC, xPSR, opcode, marker and status checks also pass.
For xPSR `0x29000000`, the acceptance mask
`xPSR & 0x010003FF` equals `0x01000000`: T=1, interrupted IPSR=0,
and stack-alignment padding bit 9=0. The other flags are incidental.

The positive marker confirms ordinary arithmetic and user-region
SRAM operation. That region is accessible at either privilege level.
The pre-fault CONTROL, IPSR, and PSP values in stacked R0–R2 were
placed there by the controlled probe. They are evidence about this
probe, not inherently trustworthy input to a production PIL. Handler
CONTROL, IPSR, SP/MSP, EXC_RETURN and fault registers were captured
from processor state. The independent SWD SHCSR read equals
`0x00050008` **exactly**, meeting the procedure's criterion even
though firmware's internal test only requires selected SHCSR bits.

All controlled record fields meet the procedure; no incidental
general-register or condition-flag value was added as a criterion.
The separate SWD reads agree with the corresponding recorded words,
both qualitative LED observations match the success pattern, the
operator reports identical complete 56-word results across resets,
and the programming image matches the uploaded Flash bytes. These
observations establish **PASS** for FLT-01-002.

## Processor revision and architectural interpretation

CPUID `0x410FD214` decodes as IMPLEMENTER=`0x41` (Arm),
VARIANT=`0` (revision 0), bits [19:16]=`0xF` (constant),
PARTNO=`0xD21` (Cortex-M33), and REVISION=`4` (patch 4).
It independently confirms **Cortex-M33 r0p4** on this board.
[ARM-TRM, 100230_0004_00_en, r0p4, §B2.3, printed p. B2-51].
CPUID alone establishes neither optional processor features nor
AT623/AT624 configuration, security configuration, or MCU silicon
revision X.

`CONTROL=2` before the fault selects privileged Thread mode and PSP;
exception entry selects Handler mode and MSP, leaving nPRIV=0 in
this configuration. FLT-01-001 supplied pre-fault CONTROL=3 and
observed Handler CONTROL=1. Both probes produced the predicted
UsageFault, basic PSP frame, `CFSR=0x00010000`, and
`EXC_RETURN=0xFFFFFFBC`; their labeled UDF PCs differ with the
linked images. EXC_RETURN encodes the interrupted mode and stack
selection, security and frame type, **not** interrupted Thread-mode
privilege. [ARM-ARM, DDI0553B.z, CONTROL.nPRIV and CONTROL.SPSEL,
§D1.2.13, printed p. 1482; §B3.18, printed pp. 104–105,
RWYWW, RQLHB; §D1.2.96, printed pp. 1628–1629].

The T1 UDF encoding and its undefined operation follow
[ARM-ARM, DDI0553B.z, §C2.4.259, printed p. 971]. The UsageFault
classification follows [ARM-ARM, DDI0553B.z, §B3.12,
printed p. 90, IKJPM]. The previous
[architectural review in the procedure](../2026-11-26-cortex-m33-usagefault-undefined-instruction-privileged-procedure.md#references-and-interpretation)
records the PM0264 return-address wording discrepancy and the
relevant processor and silicon errata assessment. The present
hardware observations do not erase those document-control points.

## Limits and review status

This PASS covers one UDF probe in privileged Thread mode with PSP,
on one board and documented configuration. It does not establish
fault attribution under nested exceptions, safe inspection of
arbitrary corrupted frames, TrustZone-enabled behavior, production
fault recovery, zeroization effectiveness, DMA isolation, resistance
to physical fault induction, or CMVP compliance. The reported source
review does not itself constitute approval of a production PIL.
No additional macOS checks, fresh option-byte readings, exact reset
timestamps, or instrumental LED timing are inferred.
