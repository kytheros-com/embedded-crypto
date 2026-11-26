# NUCLEO-U5A5ZJ-Q hardware characterization ledger

**Status:** Engineering evidence index and preliminary experiment plan

**Target:** NUCLEO-U5A5ZJ-Q, STM32U5A5ZJ revision X, Cortex-M33

**Established:** 2026-11-25

## Purpose and authority

This ledger indexes small, independently reviewable experiments for the
Platform Integration Layer (PIL). Detailed procedures, raw observations,
diagnostic decodes, and image hashes remain in the linked records. The
[board inventory](README.md) records the observed hardware and original
option bytes; the [architecture](../../docs/assurance/architecture.md)
§4.2 and §5.2 assigns platform isolation, fault/reset entry, and
zeroization mechanisms to the PIL. The
[MCL FSM design](../../docs/design/mcl-fsm.md) §1, §2, and §7 defines four
coordinated logical state machines and the current fault and reset
direction. The [module interface design](../../docs/design/module-interfaces.md)
§5 describes the target-independent protection contract. The
[documentation standard](../../docs/documentation-standard.md) §2, §6,
and §7 governs traceability and evidence claims. The
[project scope](../../docs/assurance/project-scope.md) distinguishes
project evidence from module validation. The CMVP workbook named by the
architecture remains the authoritative assurance planning baseline; ledger
IDs are local hardware experiment IDs, not workbook IDs.

The design direction is to retain the four MCL state machines without
adding a `RECOVERING` state absent demonstrated need. Unexpected module
faults lead to trusted reset, preceded by required zeroization where
possible; contained application faults may be handled outside the module.
Experiments will distinguish application, primitive, MCL, and PIL origin,
and will evaluate attribution separately from containment. Uncertain
attribution or containment requires conservative treatment. Future
platforms may need mandatory emergency zeroization independent of MCL
execution. Evidence will eventually support a reviewed fault-recovery
action matrix; this ledger does not prescribe one. The current target is
Level 1 while preserving the possibility of MCL reuse in products targeting
higher levels, including Level 4. No validation claim follows from this plan.

## Identifier and status conventions

`HW-001` through `HW-005` are permanent IDs for the historical individual
experiments. New families use stable `FLT`, `TZ`, `ISO`, `REC`, and `ZER`
IDs below. Assign each future individual experiment a subordinate ID such
as `FLT-01-001`; allocate monotonically within its family, never reuse or
repurpose an ID, and retain its history when an experiment is superseded.
Family rows are coverage plans, not claims that an application exists or
that the target supports every proposed mechanism. Establish applicability
against the relevant architecture, target documentation, silicon revision,
and hardware before implementing a child experiment.

| Individual status | Meaning |
|---|---|
| `PLANNED` | Identified; firmware not implemented. |
| `IMPLEMENTED` | Firmware exists; pre-deployment verification incomplete. |
| `READY` | Build and pre-deployment inspection complete; awaiting board execution. |
| `PASS` | Valid board observations meet documented acceptance criteria. |
| `FAIL` | Valid board observations contradict documented acceptance criteria. |
| `INCONCLUSIVE` | Evidence cannot establish a result. |
| `BLOCKED` | Required hardware, documentation, tooling, or other prerequisite unavailable. |
| `N/A` | Inapplicable, with documented justification. |

A family may be described as `PARTIAL` when some related evidence exists;
that is an aggregate coverage description, not an individual status or a
family-wide pass. Record each execution separately from lifecycle status,
including failed, inconclusive, corrected, and superseded executions. A
build failure, tooling problem, or harness defect is not a hardware
security-mechanism failure. `READY` is not permission to deploy.

## Completed individual experiments

All five records concern the same revision X board with TrustZone disabled
(`TZEN=0`); the MPU and privilege experiments therefore provide no
Secure/Non-secure separation evidence. Dates below are hardware execution
dates stated in the records, not inferred from filenames. `PASS` refers
only to each record's documented acceptance criteria.

| ID | Date; status | Result and configuration | Procedure; source; evidence |
|---|---|---|---|
| `HW-001` | 2026-11-21; `PASS` | Rust 1.98.1, `thumbv8m.main-none-eabi`; 880-byte image programmed at `0x08000000`. SWD read-back matched; LD1 blinked after reset. No MPU, DMA, or TrustZone test. | [Deployment procedure](FIRMWARE.md); [bring-up source](./); [record](records/2026-11-21-initial-firmware-deployment.md) |
| `HW-002` | 2026-11-23; `PASS` | MPU disabled. Two matching reset executions showed Thread privilege/PSP transition, ignored direct unprivileged privilege restoration, SVCall handler entry, and controlled return. | [Procedure](2026-11-23-cortex-m33-privilege-svc-procedure.md); [source](privilege-svc/); [record](records/2026-11-23-cortex-m33-privilege-svc.md) |
| `HW-003` | 2026-11-23; `PASS` | Three MPU regions: Flash RX, user SRAM RW/XN, supervisor SRAM privileged-only RW/XN. Two matching reset executions: authorized user read, then prohibited supervisor read produced MemManage `DACCVIOL` with valid fault address and designated stacked load PC. | [Procedure](2026-11-23-cortex-m33-mpu-read-denial-procedure.md); [source](mpu-read-denial/); [record](records/2026-11-23-cortex-m33-mpu-read-denial.md) |
| `HW-004` | 2026-11-24; `PASS` | Same three-region MPU map. After a corrected harness was deployed, two matching reset executions showed authorized user write, MemManage `DACCVIOL` on supervisor store, and unchanged protected sentinel. | [Procedure](2026-11-24-cortex-m33-mpu-write-denial-procedure.md); [source](mpu-write-denial/); [record](records/2026-11-24-cortex-m33-mpu-write-denial.md) |
| `HW-005` | 2026-11-24; `PASS` | Same map; user SRAM XN. Two matching reset executions showed permitted SRAM data read but denied instruction fetch, MemManage `IACCVIOL`, and unchanged failure marker. | [Procedure](2026-11-25-cortex-m33-mpu-execute-never-procedure.md); [source](mpu-execute-never/); [record](records/2026-11-25-cortex-m33-mpu-execute-never.md) |

`HW-001` uses [FIRMWARE.md](FIRMWARE.md) as its pre-deployment procedure;
there is no separately named deployment procedure or separate experiment
source directory. Its record reports cross-platform agreement of the
loadable Flash payload, while complete ELFs differed. Factory-image
restoration was not tested, post-deployment option-byte comparison was not
recorded, and LED timing was not precisely measured.

`HW-002` has two identical 84-byte diagnostic records. Its evidence leaves
the exact deployed source commit and final programming-image hash
unconfirmed. The record also notes a discrepancy between ST PM0264 Rev 8's
summary of `CONTROL` access and the Armv8-M architectural description of
unprivileged reads versus ignored writes. The tested observations support
the latter behavior on this board; the documentation discrepancy needs
review when privilege management is designed.

`HW-003` has two matching diagnostic records and an independently verified
programming-image checksum. `HW-004` retains an earlier programmed run in
which LD1 stayed dark because the harness sampled MSP inside a Rust
function and incorrectly required continuous 8-byte alignment. This was
a startup-state assertion defect, not an MPU permission failure. The
corrected run captured MSP at the call boundary; two subsequent reset
executions passed without changing MPU permissions. Its evidence record
preserves both outcomes and the corrected image provenance.

`HW-005`'s evidence record is dated 2026-11-25 but explicitly gives
2026-11-24 as the hardware test date. Its `CFSR` reported `IACCVIOL`,
while `MMARVALID` was clear, so `MMFAR` was not a valid target address.
The stacked PC was `0x20001008`, the prohibited instruction-fetch target;
the initiating `BX r12` instruction was in Flash at `0x080002fc`.
This distinction must be retained in later attribution work.

The three MPU experiments use terminal privileged MemManage handlers and
test selected CPU accesses with SWD-readable diagnostics. They do not show
safe resumption, general fault attribution, arbitrary-code containment,
DMA/peripheral isolation, TrustZone enforcement, or comprehensive platform
isolation. The privilege/SVCall experiment does not establish a secure SVC
authorization boundary. The board's original option bytes are observations,
not proof of later runtime protection settings.

## Child experiments

| ID | Family | Status | Scope and evidence |
|---|---|---|---|
| `FLT-01-001` | `FLT-01` | `PASS` | On 2026-11-25, two physical hardware-reset executions produced identical 55-word observations and the expected five-flash LD1 indication. Unprivileged `UDF #0x42` entered UsageFault (IPSR 6) with `CFSR=0x00010000` (`UNDEFINSTR` only), `HFSR=0`, and stacked PC at the labeled UDF. [Procedure](2026-11-25-cortex-m33-usagefault-undefined-instruction-procedure.md); [source](usagefault-undefined-instruction/); [hardware evidence](records/2026-11-25-cortex-m33-usagefault-undefined-instruction.md). |
| `FLT-01-002` | `FLT-01` | `IMPLEMENTED` | Standalone privileged Thread-mode PSP `UDF #0x42` comparison with FLT-01-001. Linux static build and ELF inspection passed; target-specific `unsafe` review, macOS reproduction, and authorized board execution remain pending. No hardware result exists. [Procedure](2026-11-26-cortex-m33-usagefault-undefined-instruction-privileged-procedure.md); [source](usagefault-undefined-instruction-privileged/). |

The completed [evidence record](records/2026-11-25-cortex-m33-usagefault-undefined-instruction.md)
identifies committed source revision
`0f6f122dc9c64161a10e05b1af2656f443c64f94`. All seven reported
source-file hashes match that commit. The independent macOS Flash
payload and programming image each match the Linux reference: 2,736
bytes, SHA-256
`c8dbcc5ce07e86504f522cde0463f45abb7b9b82a3cf55c24cb50375b603f34d`.
The 2,736-byte physical Flash read-back has the same hash and passed a
byte-for-byte comparison with the programming image. The macOS and
Linux ELF files differ; complete ELF reproducibility is not claimed.
The full macOS build/programming transcript and individual hardware
execution timestamps were not supplied. The `PASS` status applies to
the documented hardware acceptance criteria, not to completion of human
design review of the target-specific `unsafe` code. That review's
outcome was not supplied. The result establishes neither family-wide
fault classification nor comprehensive containment or recovery.

## Preliminary experiment families

Every row below describes preliminary coverage. `PARTIAL` marks only a
related, limited historical observation or completed child experiment.
`FLT-01-001` has a board result; `FLT-01-002` has only static results.
Other UsageFault causes remain untested. No family is complete, and
applicability of future proposed
experiments remains to be established during planning. The linked
historical IDs are cross-references, not substitute acceptance results
for a family.

| Family | Investigation | Coverage and related evidence |
|---|---|---|
| `FLT-01` | UsageFault classification | `PARTIAL` hardware coverage: `FLT-01-001` passed for one unprivileged undefined instruction; `FLT-01-002` awaits review and hardware execution for privileged Thread mode. Other UsageFault causes, execution domains, containment, and recovery remain untested. |
| `FLT-02` | MemManage classification and execution-domain attribution | `PARTIAL`: `HW-003`–`HW-005` classified selected unprivileged access faults; no general attribution or recovery. |
| `FLT-03` | BusFault classification, including precise and imprecise faults where supported | Planned; applicability of each fault mode to be checked. |
| `FLT-04` | Fault escalation to HardFault | Planned; `HW-003`–`HW-005` observed no escalation for their selected faults. |
| `FLT-05` | Stack-limit violations | Planned; none. |
| `FLT-06` | Exception stacking and unstacking failures | Planned; `HW-002`–`HW-005` observed ordinary stacking, not failure paths. |
| `FLT-07` | Faults during interrupt handling, including nested interrupts | Planned; none. |
| `FLT-08` | Faults during fault handling | Planned; none. |
| `FLT-09` | Processor lockup and reset recovery | Planned; none. |
| `TZ-01` | Secure/Non-secure execution-domain attribution | Planned; historical tests used `TZEN=0`. |
| `TZ-02` | SecureFault classification and routing | Planned; historical tests used `TZEN=0`. |
| `TZ-03` | Security-domain interrupt routing | Planned; historical tests used `TZEN=0`. |
| `ISO-01` | GTZC enforcement and fault reporting | Planned; none. |
| `ISO-02` | DMA isolation | Planned; none. |
| `REC-01` | Reset behavior and memory retention | `PARTIAL`: `HW-001`–`HW-005` used hardware resets; no systematic retention or reset-cause study. |
| `REC-02` | Watchdog recovery | Planned; none. |
| `ZER-01` | Reliable memory zeroization | Planned; none. |
| `ZER-02` | Interrupted zeroization recovery | Planned; none. |

## Prerequisites and open questions

Before each child experiment, identify the applicable Arm architecture
revision and section, ST reference/programming manual revision and section,
device errata, board revision, option-byte baseline, toolchain, debugger,
reset/recovery path, and any test-specific hardware constraints. Determine
whether the proposed mechanism and diagnostic registers are implemented on
this silicon. A configuration change, especially TrustZone enablement or
other option-byte changes, requires separate review and authorization;
the existing `TZEN=0` evidence cannot answer Secure/Non-secure questions.

The principal architectural questions are whether diagnostics can reliably
distinguish application, primitive, MCL, and PIL faults; when exception
frames are valid; whether a fault stays contained; and which trusted reset
and zeroization mechanisms still work after nested faults, lockup, or
interrupted cleanup. Retained SRAM and reset-cause reliability also need
evidence. These questions inform the still-open reset/fault realization
(Q-008) and the formal module boundary (Q-001); the ledger does not settle
either workbook question. Current evidence is limited to selected,
controlled, reversible experiments on one board and silicon revision.

## Evidence convention for future child experiments

Each child has a linked procedure, source directory when implemented, and
append-only hardware evidence record. Record, as applicable:

- Stable ID, objective, specific architectural question, and expected
  observations with explicit acceptance criteria.
- Applicable architecture and target references with revision and section;
  identify separately guarantees, observed behavior, and inferences.
- Board/silicon and software prerequisites; processor, security, privilege,
  MPU/GTZC/DMA, interrupt, and reset configuration.
- Source revision and review provenance; toolchain, linker layout, generated
  code, build commands, image size/hash, and programmed-image provenance.
- Each execution's date, setup, raw register and memory observations,
  decoded values, independent ground truth, repeatability, outcome, and
  rationale. Preserve unsuccessful and superseded executions.
- Limitations, unresolved questions, and implications for PIL design.

For faults, additionally record the **known originating execution domain**
and induced condition; exception handler entered; exception-stack
availability and validity; execution domain and fault class inferred from
diagnostics; whether containment is established, partial, or uncertain;
and which recovery mechanisms remained available. Ground truth should be
established independently from the intended instruction, location, and
configuration, rather than inferred only from the handler's classification.
An unknown or ambiguous attribution is a valid result. Do not replace it
with an assumed domain or fault class. Separate a fault's observed outcome
from the child's lifecycle status and from a harness or tooling failure.

## Incremental workflow

1. Define one narrow question and explicit acceptance criteria.
2. Implement a minimal standalone experimental application with synthetic
   data, never real keys or other sensitive security parameters.
3. Review source, linker layout, generated code, and build artifacts as
   applicable; record provenance.
4. Obtain human approval before deploying to physical hardware.
5. Execute on the board and retain raw observations.
6. Analyze observations against the acceptance criteria, including
   inconclusive or unexpected results.
7. Preserve evidence and update this ledger without erasing prior outcomes.
8. Select the next experiment using accumulated evidence.

Experimental firmware remains separate from production PIL code. Successful
architectural exception tests do not establish resistance to arbitrary
hardware corruption or physical fault-induction attacks. All conclusions
remain subject to human engineering review.
