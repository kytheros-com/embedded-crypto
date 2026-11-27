# FLT-01-008 — Two one-shot expected UDF faults in one boot

**Date:** 2026-11-27  
**Lifecycle:** Implemented and statically inspected; physical execution
pending.  
**Scope:** Controlled exception-return characterization, not a
production recovery handler or POST.

## Architectural question and authorization model

Can a protected test controller regain control after a previously
characterized `UDF #0x42`, retire that one-shot expectation, and
execute a second separately armed UDF without a processor reset?
The positive operations and both UDFs are explicit assembly in one
unprivileged Thread/PSP probe. The terminal diagnostics are
supervisor-only. The second expectation is armed only by a fixed-site
SVC after the first exception has returned and the second positive
operation has completed. A final fixed-site SVC stays in Handler
mode for the success LED. No general skip-instruction API exists.

The [ARM-ARM DDI0553B.z, §C2.4.259] UDF T1 encoding is `DE42`.
[§B3.12, printed pp. 90–91] assigns UNDEFINSTR to UsageFault 6,
UFSR bit 0 (CFSR bit 16). [§B3.18, p. 105, RQLHB] selects the
faulting UDF PC. [§§B3.19, B3.22, printed pp. 106–118;
§D1.2.96, pp. 1628–1629] define the basic frame,
EXC_RETURN=`FFFFFFBC`, and exception return from Handler mode
using a POP that loads PC. The wrapper's `POP {r4,pc}` performs
exception return only after the Rust validator returns successfully.
The exact EXC_RETURN, frame bounds, active exception, xPSR and
syndrome are checked before frame inspection or PC modification.
DDI0625 was reviewed for exception-entry/return fault cases; this
test excludes stacking faults and does not attempt their recovery.

The [HW-002 SVC procedure](2026-11-23-cortex-m33-privilege-svc-procedure.md)
demonstrated cooperative SVC return but did not establish a secure
authorization boundary. Here neither an unprivileged register nor
user SRAM authorizes recovery. A descriptor in privileged-only
Supervisor SRAM stores test ID, sequence, armed state, exception 6,
exact syndrome, Thread/PSP origin, linked UDF PC, authorized linked
continuation, frame PSP, EXC_RETURN and expected completion marker.
The one-shot armed bit is consumed before return. Exact fixed SVC
return PC and opcode select the arm-second and finish transitions.
This remains a cooperating-software experiment: unprivileged code
could branch within executable Flash or alter user markers, so it
does not establish adversarial provenance or a production SVC policy.

## One-boot sequence and fixed linked sites

Assembly startup calls privileged Rust on MSP, returns with its
software frame released, selects PSP=`20002000`, CONTROL=3 and
executes ISB before entering the stackless probe. Rust initializes
the protected descriptor for test 1 and clears all diagnostics.
Every system reset restarts the protected fault count at zero.

| Step | Linked site or expected effect |
|---|---|
| Positive 1 | Check `42+1=43`; publish user marker 1 |
| Fault 1 | `UDF #0x42` at `080002F2`; expected continuation `080002F4` |
| Handler 1 | Record slot 1; clear only captured UNDEFINSTR; consume arm 1; change stacked PC to `080002F4`; return |
| After return | Publish marker 2; check `40+4=44`; publish marker 3 |
| Arm second | `SVC #0x72` at `0800030C`; return PC `0800030E`; descriptor becomes armed test 2 |
| Fault 2 | `UDF #0x42` at `08000332`; expected continuation `08000334` |
| Handler 2 | Record slot 2; clear only captured UNDEFINSTR; consume arm 2; change stacked PC to `08000334`; return |
| Final | Publish marker 5; `SVC #0x73` at `0800033C`; final Handler-mode LED loop, no SVC return |

The arm-second SVC return path publishes marker 4 before fault 2.
These addresses come from the inspected ELF, not earlier UDF images;
re-derive them for any new build. If a UDF does not fault, an
unexpected SVC or descriptor state prevents success. Duplicate
fault, wrong syndrome, wrong PC, invalid frame, missing completion,
or unapproved continuation also prevents Status=1.

## Configuration, layout and diagnostic map

Use NUCLEO-U5A5ZJ-Q, STM32U5A5ZJ revision X, Cortex-M33 r0p4,
original option bytes with TZEN=0/RDP Level 0, Rust 1.98.1 and
`thumbv8m.main-none-eabi`. Retain the three-region MPU:
Flash `08000006/08001FE1`, User `20001003/20001FE1`,
Supervisor `20002001/20002FE1`, MPU_CTRL=5, MAIR0=`44`.
MemManage and UsageFault are enabled with reset-default priorities;
no ordinary interrupt source or FP context is active. SysTick is
polled only in the terminal success path, interrupt disabled.

The 64-word supervisor record is `[20002000,20002100)`;
sentinel `A5C35A3C` is at `20002100`; protected 12-word descriptor
is `[20002104,20002134)`. All lie below the MSP reserve
`[20002800,20003000)`. The six user markers occupy
`[20001000,20001018)` and the PSP reserve begins at `20001018`.
Each basic UDF frame occupies `[20001FE0,20002000)` while its
handler is active. The second reuses that physical PSP range only
after the first exception return; each frame is first copied to a
different protected slot. The final active SVC frame also occupies
that PSP range and is **not** a surviving UDF frame.

| Record word(s) | Meaning and final acceptance |
|---|---|
| `00`–`06` | Status=1, Phase=5, FaultCount=2, SvcCount=2, PositiveMask=3, CompletionMask=5, UnexpectedIpsr=0 |
| `07`–`0B` | InitialCfsr captured; FinalCfsr=0, ArmedShcsr=`50000`, CCR captured, SentinelBefore=`A5C35A3C` |
| `0C`–`0F` | SentinelAfter=`A5C35A3C`, DescriptorState=5, DescriptorArmed=0, Error=0 |
| `10`–`23` | First 20-word UDF slot |
| `24`–`37` | Second 20-word UDF slot |
| `38`–`3F` | Reserved, startup-cleared zero |

Each slot's first eight words are the original stacked
R0,R1,R2,R3,R12,LR,PC,xPSR. The next twelve are captured
CFSR, HFSR, SHCSR, EXC_RETURN, PSP, MSP, Handler CONTROL,
Handler IPSR, linked fault PC, authorized continuation,
test ID and structural-valid marker. The first slot starts at
`20002040`, the second at `20002090`. The descriptor's twelve
words are State, Armed, TestId, Sequence, ExpectedException,
ExpectedSyndrome, ExpectedOrigin, FaultPc, ContinuationPc,
FramePsp, ExcReturn and CompletionCondition, in that order.
The final descriptor must be
`5,0,2,2,6,00010000,3,08000332,08000334,20001FE0,FFFFFFBC,5`.

For **each** slot require CFSR=`00010000` (UNDEFINSTR only),
HFSR=0, SHCSR=`00050008`, EXC_RETURN=`FFFFFFBC`,
PSP=`20001FE0`, MSP=`20003000`, Handler CONTROL=1, IPSR=6,
test ID 1 or 2, and valid=1. Stacked R0/R1/R2 must be
`3/0/20002000`; PC must match that slot's linked UDF; xPSR must
satisfy `xPSR & 010003FF == 01000000`. Retain incidental R3,
R12, LR and flags without retrospective acceptance requirements.
The slot copy is made **before** CFSR is cleared and stacked PC
is changed. It preserves the original fault PC for later SWD review;
the physical frame no longer has that PC after return.

The physical final SVC frame must have PC=`0800033E` and the
Thumb/Thread/no-padding xPSR mask. Externally read final
SHCSR/CFSR/HFSR=`00050080/0/0`; SVCALLACT is bit 7 in
[ARM-ARM §D1.2.233, p. 1841]. Require user words
`1,2,3,4,5,0`, protected sentinel unchanged, external CPUID
`410FD214` and CCR with DIV_0_TRP clear. The final repeating
three-slow-green-flash LD1 pattern is corroborating evidence,
never sufficient by itself. The two successful UDF returns and
final SVC activity occur in one boot if Status=1 under this
trusted descriptor/firmware model: startup clears the count, and
no success path exists with fewer than two retired faults.

## Physical collection and limits

Require **two independent physical RESET executions** of this
one image. Do not reset between internal stimuli. After the final
success indication on each run, read 64 supervisor words, 12
descriptor words, six user words, eight physical final SVC-frame
words, sentinel, CCR, three SCB words and CPUID: **96 observed
words per run**. The [campaign runner](campaign/README.md) records
all exact SWD commands, UTC collection times and raw outputs,
and compares each run separately. Upload the full physical Flash
range and compare it byte for byte with the reviewed image.

Before deployment inspect complete optimized disassembly, wrapper
stack checks, reachable stack consumption, `POP {r4,pc}` return,
frame-copy order, CFSR W1C order, and stacked-PC store. Ensure
the programming image and cross-host source/payload/image hashes
match. The ARM-ERRATA SDEN-756493 v14.0 r0p4 and ES0553 Rev. 6
silicon-X items previously reviewed do not match this static-priority,
aligned UDF/SVC, inactive-FP sequence; no workaround is assumed.
This experiment does not cover corrupted stacks, nested exceptions,
arbitrary operational fault recovery, adversarial caller
authentication, TrustZone routing, production PIL assurance or
protection-mechanism POST acceptance. Follow the
[board recovery policy](README.md); never change option bytes or
deploy without human authorization.
