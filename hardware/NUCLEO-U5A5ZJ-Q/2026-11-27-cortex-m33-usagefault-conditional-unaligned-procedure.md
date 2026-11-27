# FLT-01-005 — Conditional unaligned Normal-memory load

**Date:** 2026-11-27  
**Lifecycle:** Implemented and statically inspected; physical execution
pending.  
**Scope:** Standalone PIL characterization, not production POST.

## Architectural question and prospective result

Does `CCR.UNALIGN_TRP` control the outcome of the **same** unaligned
ordinary `LDR.W` from authorized Normal SRAM? The two separately
compiled builds share the identical assembly probe. Privileged
startup clears bit 3 in `005-disabled` and sets it in `005-enabled`
using read-modify-write, DSB/ISB and readback. It records initial and
configured CCR and checks every other readable CCR bit unchanged.
The intended experimental variable is bit 3; do not use debugger
writes or exception return to change configuration.

[ARM-ARM, DDI0553B.z, §D1.2.9, printed p. 1474] permits ordinary
unaligned LDR when UNALIGN_TRP=0 and requires an UNALIGNED UsageFault
when it is one. [§C2.4.78, pp. 667–670] uses `MemU[address,4]`
for a non-PC destination. [§B3.12, p. 91] classifies the fault;
[§D1.2.270, p. 1898] places UNALIGNED at UFSR bit 8, CFSR bit 24.
[§B3.18, p. 105, RQLHB] stacks the faulting instruction address for
the synchronous exception. The [reference catalogue](references.md)
identifies the local ignored PDF revisions and digests. These are
architectural rules. Exact PC, frame address, EXC_RETURN, SCB values,
LED behavior and SRAM result are controlled-board predictions.

The positive `LDR.W r6,[r5]` reads bytes `11 22 33 44` from
`[0x20001020,0x20001024)` and must yield `0x44332211`.
The fault stimulus `LDR.W r6,[r5,#1]` reads bytes
`22 33 44 55` from `[0x20001021,0x20001025)` and, when permitted,
must yield `0x55443322`. The 12-byte synthetic pattern occupies
`[0x20001020,0x2000102C)` within the single User SRAM MPU region
and does not overlap the five diagnostic words
`[0x20001000,0x20001014)`, PSP reserve
`[0x20001030,0x20002000)`, supervisor record, or sentinel.
MAIR0=`0x44` defines the tested Normal-memory attributes.
The probe uses no software stack and verifies CONTROL=3, IPSR=0,
and active PSP=`0x20002000` before the positive control.

The inspected instruction halfwords are `F8D5 6000` for the aligned
positive control and `F8D5 6001` for the fault stimulus, with
little-endian fault bytes `D5 F8 01 60`. The latter begins at
`0x08000334`; its continuation is exactly four bytes later at
`0x08000338`. These addresses must be re-derived for any new ELF.

## Common configuration and diagnostic map

Use NUCLEO-U5A5ZJ-Q, STM32U5A5ZJ revision X, Cortex-M33 r0p4,
the original option bytes with TZEN=0 and RDP Level 0, Rust 1.98.1,
and `thumbv8m.main-none-eabi`. Keep the three MPU regions and
encodings from [FLT-01-004](2026-11-27-cortex-m33-usagefault-divide-by-zero-procedure.md):
Flash `08000006/08001FE1`, User `20001003/20001FE1`, Supervisor
`20002001/20002FE1`, with MPU_CTRL=5, MAIR0=`44`, MAIR1=0 and
regions 3–7 disabled. MemManage and UsageFault are enabled with
reset-default priorities. No ordinary interrupt or active FP context.

The 58-word supervisor record is `[0x20002000,0x200020E8)` and the
protected sentinel is at `0x200020E8`. Offsets `00`–`E4` retain the
FLT-01-004 layout, with these semantic replacements: at `50`,
ExpectedFaultAddress; `54`, ExpectedFirstHalfword; `B8`,
FirstReadback; `D4`, PositiveValue; `D8`, FaultValue; `E0`,
ExpectedSecondHalfword; `E4`, SecondReadback. `D0` is BuildMode
(disabled 0, enabled 1). The complete common offset map is in the
[campaign documentation](campaign/README.md); the original
[FLT-01-004 map](2026-11-27-cortex-m33-usagefault-divide-by-zero-procedure.md#supervisor-diagnostic-word-map)
defines all unchanged fields. All storage is cleared or initialized
on every reset; the fault-value word starts at `A55A5AA5`.

`FrameValid=1` only means the basic candidate frame passed structural
inspection prerequisites. Handler input is captured in stackless
assembly before Rust prologue. The handler checks original PSP,
MSP, SP, EXC_RETURN, frame bounds and type, exception number,
syndrome, and stacking-fault absence before read-only frame loads.
The handler never returns, clears CFSR, or modifies frame words.

## Prospective physical acceptance

Both builds require startup CONTROL/IPSR=`0/0`, initial MSP in the
2 KiB reserve, `MPU_TYPE=00000800`, initial MPU_CTRL=0, exact MPU
and MAIR readback above, armed SHCSR=`00050000`, no unused enabled
region, initial CFSR/HFSR captured then cleared, sentinel
`A5C35A3C`, and no unexpected IPSR or failure marker. Initial
CCR bit 3 must be zero after each physical RESET. The configured
CCR must change only that bit; external SWD CCR must match it.

| Observation | `005-disabled` | `005-enabled` |
|---|---|---|
| Supervisor Status/Phase/BuildMode | `2/3/0` | `1/4/1` |
| CCR.UNALIGN_TRP | 0 | 1 |
| User words at `20001000` | `1,2,44332211,55443322,0` | `1,0,44332211,A55A5AA5,0` |
| External SHCSR/CFSR/HFSR | `00050000/0/0` | `00050008/01000000/0` |
| LD1 | May remain dark | Repeating three slow green flashes and pause |
| Physical PSP exception frame | None expected | Basic frame `[20001FE0,20002000)` |

Disabled Status=2 means **configured control**, not completed load;
the independently read user completion and value, zero fault status,
and terminal code path establish success. Do not decode residual PSP
SRAM as a frame. LD1 darkness alone cannot establish acceptance.

Enabled acceptance additionally requires Handler CONTROL=1,
IPSR=6, entry SP/MSP=`20003000`, EXC_RETURN=`FFFFFFBC`,
FramePsp=`20001FE0`, FrameValid=1, zero PRIMASK/BASEPRI/FAULTMASK,
and exact SHCSR/CFSR/HFSR above. Stacked R0/R1/R2=`3/0/20002000`,
PC=`08000334`, and xPSR must satisfy
`xPSR & 010003FF == 01000000` (Thumb, interrupted Thread, no
alignment padding). Do not require incidental R3, R12, LR or flags.
Require halfwords `F8D5 6001`, progress 1, continuation 0,
positive value `44332211`, untouched fault-value sentinel,
unchanged protected sentinel, and no unexpected handler. Independently
read all eight physical PSP frame words after the LED indication and
compare them with supervisor offsets `98`–`B4` for that run.

Two independent physical RESET executions per build are required.
Each disabled run has 69 observed words (58 supervisor, 5 user,
sentinel, CCR, 3 SCB, CPUID); each enabled run has 77, including
the eight direct frame words. Treat each run independently, preserve
incidental differences, and compare repeats. Require independent
physical Flash upload and byte-for-byte image comparison before
replacing either configuration. The [campaign runner](campaign/README.md)
collects raw output and computes provisional comparisons; the human
operator controls all deployment and RESET actions.

## Scope and limitations

The ARM-ERRATA SDEN-756493 v14.0 r0p4 entries and ES0553 Rev. 6
silicon-X entries reviewed for FLT-01-004 do not describe this
ordinary aligned-control/unaligned-Normal-SRAM path under static
priorities and inactive FP context. In particular, an unaligned
**Device** access erratum is not evidence for this Normal-memory
test. This experiment does not establish behavior for Device memory,
cross-region accesses, arbitrary instructions, stack corruption,
exception return, recovery, TrustZone isolation, or production POST.
Review the [board recovery policy](README.md) before programming;
do not change option bytes or protection settings.
