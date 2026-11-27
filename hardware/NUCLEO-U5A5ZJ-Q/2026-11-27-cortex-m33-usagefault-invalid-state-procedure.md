# FLT-01-007 — Invalid instruction-set state on BX target

**Date:** 2026-11-27  
**Lifecycle:** Implemented and statically inspected; physical execution
pending.  
**Scope:** Standalone PIL characterization, not production POST.

## Architectural question and controls

What fault and stacked state result when `BX r5` targets executable
Flash with address bit 0 clear? The positive control first loads
the **same** linked target with bit 0 set (`0x0800034F`), branches
with `BX r5`, executes target `BX LR`, and returns to the labeled
positive continuation. Only then does it publish user positive
value `A55A0007`. For the negative stimulus, assembly reloads the
target and explicitly clears bit 0 with `BIC.W` before the second
`BX r5`. The instruction at target `0x0800034E` remains `BX LR`
(`0x4770`) inside executable Flash. The initiating positive and
negative BX halfwords are both `0x4728`; the negative branch is
at `0x08000338`, and its unreachable continuation is
`0x0800033A`. These are linked-image coordinates, to be
re-derived for a new build.

[ARM-ARM, DDI0553B.z, §B3.5.2, printed p. 75, RSQLX] requires an
INVSTATE UsageFault on an attempt to execute with EPSR.T=0 on a
Main Extension PE. [§C2.4.29, pp. 584–585] says BX uses target
bit 0 for instruction state and places the fault on the instruction
at the target address when bit 0 is zero. [§B3.12, p. 90] classifies
INVSTATE; [§D1.2.270, pp. 1898–1899] maps it to UFSR bit 1,
CFSR bit 17 (`00020000`). [§B3.18, p. 105, RQLHB] stacks the
instruction causing a synchronous exception. Here that is the
invalid-state fetch at **target** `0800034E`, not the initiating
BX at `08000338`. The supplemental [DDI0624 instruction-fetch
cases](references.md), workbook `V8MFETCH!A16:X16`, were reviewed
for a non-Secure
Main Extension INVSTATE fetch with T=0 and no MPU or bus fault;
no TrustZone transition is attempted.
The documented prediction is a stacked PC at the target and a
stacked EPSR.T bit of zero. This differs deliberately from the
ordinary Thumb-state acceptance mask used by the UDF experiments.

The assembly probe verifies CONTROL=3, IPSR=0, active
PSP=`20002000` and performs no stack access. Both target and
continuation are explicit linked symbols; there is no invalid
instruction encoding at the target. No PC rewrite, exception
return or recovery is attempted.

## Configuration and prospective acceptance

Use NUCLEO-U5A5ZJ-Q, STM32U5A5ZJ revision X, Cortex-M33 r0p4,
original option bytes, TZEN=0, RDP Level 0, Rust 1.98.1 and
`thumbv8m.main-none-eabi`. Retain the three MPU regions, Normal
User SRAM, supervisor-only diagnostics, stack reserves, reset
exception priorities, inactive FP context and disabled ordinary
interrupts from [FLT-01-005](2026-11-27-cortex-m33-usagefault-conditional-unaligned-procedure.md).
The common 58-word [diagnostic field map](campaign/README.md)
uses BuildMode=3. Privileged startup records CCR and leaves
UNALIGN_TRP cleared through the common read-modify-write and
readback path; this bit does not cause the invalid-state fault.

Per physical RESET, require exact startup/MPU fields, armed
SHCSR=`00050000`, initial CCR.UNALIGN_TRP=0, configured/external
CCR with bit 3 clear and other readable bits preserved, and:

| Observation | Required value |
|---|---|
| Supervisor Status/Phase/BuildMode | `1/4/3` |
| User progress/continuation/positive/fault/failure | `1,0,A55A0007,A55A5AA5,0` |
| External SHCSR/CFSR/HFSR | `00050008/00020000/00000000` |
| Handler CONTROL/IPSR | `1/6` |
| Handler entry SP/MSP | `20003000/20003000` |
| EXC_RETURN/FramePsp/FrameValid | `FFFFFFBC/20001FE0/1` |
| PRIMASK/BASEPRI/FAULTMASK | `0/0/0` |
| Stacked R0/R1/R2 | `3/0/20002000` |
| Stacked PC | `0800034E`, not initiating BX `08000338` |
| Stacked xPSR mask | `xPSR & 010003FF == 00000000` |
| Branch and target halfwords | `4728` and `4770` |
| Sentinel; unexpected IPSR | `A5C35A3C`; 0 |

The xPSR mask requires **T=0**, interrupted IPSR=0 and no
alignment padding. Other flags and incidental stacked R3/R12/LR
have no fixed acceptance value. The handler checks exact PSP,
MSP, SP, EXC_RETURN, bounds, basic-frame type, UsageFault number,
exact syndrome and absence of stacking bits before read-only
frame inspection. `FrameValid=1` is only a structural marker.
The terminal handler never modifies the frame or returns.

After each of two independent physical RESET executions, note the
three-slow-green-flash LD1 indication, then read all 58 supervisor
words, five user words, sentinel, CCR, three SCB registers, CPUID,
and eight physical PSP frame words: **77 observed words** per run.
Require every direct frame word to match supervisor offsets
`98`–`B4`. Check each run independently and preserve incidental
differences. Read back the entire physical programming-image range
and compare it byte for byte before replacement. The
[campaign runner](campaign/README.md) provides provisional
comparisons; independent engineering review remains necessary.

## Errata and limits

ARM-ERRATA SDEN-756493 v14.0 and ES0553 Rev. 6 have been reviewed
for the tested Cortex-M33 r0p4 and STM32U5A5 silicon X. The
previously identified unaligned Device access, interrupt-priority
update and partial FP-state erratum conditions do not match this
static-priority, integer BX, inactive-FP, executable-Flash path.
This result would not establish general instruction-fetch
precedence, arbitrary target attribution, TrustZone routing,
adversarial control-flow integrity, exception return or production
fault recovery. Follow the [board recovery policy](README.md);
do not change option bytes or physical protection settings.
