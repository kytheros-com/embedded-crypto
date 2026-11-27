# FLT-01-006 — Mandatory alignment for LDRD

**Date:** 2026-11-27  
**Lifecycle:** Implemented and statically inspected; physical execution
pending.  
**Scope:** Standalone PIL characterization, not production POST.

## Architectural question and stimulus selection

Does `LDRD` enforce word alignment when `CCR.UNALIGN_TRP=0`?
The selected instruction is the offset, no-writeback T1
`LDRD r6,r7,[r5]`, halfwords `E9D5 6700` and little-endian bytes
`D5 E9 00 67`. R5 is an ordinary base register; the two distinct
destinations are neither SP nor PC. No prohibited register or
writeback combination is used.

[ARM-ARM, DDI0553B.z, §C2.4.85, printed pp. 683–684] defines
both loads as `MemA[address,4]` and `MemA[address+4,4]` and lists
the register restrictions. The aligned access starts at
`0x20001020`, loading `0x44332211` into R6 and `0x88776655`
into R7 from the initialized pattern. The assembly checks **both**
values before storing the positive marker. It then increments R5
to `0x20001021`, writes progress 1, synchronizes and executes the
same `LDRD` encoding. The negative span is
`[0x20001021,0x20001029)`, wholly within the 12-byte synthetic
Normal-SRAM pattern `[0x20001020,0x2000102C)`, one User MPU region.
It cannot overlap the diagnostic words, PSP reserve, supervisor
record, or sentinel.

[ARM-ARM, §D1.2.9, printed p. 1474] says UNALIGN_TRP=0 permits
unaligned ordinary LDR/LDRH/STR/STRH, while unaligned multiple
accesses always fault. The `MemA` operation of this LDRD supplies
the mandatory alignment requirement. [§B3.12, p. 91] classifies
UNALIGNED UsageFault; [§D1.2.270, p. 1898] maps it to CFSR bit 24.
[§B3.18, p. 105, RQLHB] selects the faulting instruction address
for a synchronous fault. These architectural rules predict the
syndrome; exact entry state below depends on the controlled board
configuration. The local revisions are in [references.md](references.md).

The linked positive LDRD is at `0x0800031C`; the faulting LDRD is
at `0x08000340`, followed by the continuation at `0x08000344`.
Regenerate these coordinates from any new ELF before deployment.
The stackless probe verifies CONTROL=3, IPSR=0 and PSP=`20002000`
before the aligned control. It never uses a software stack.

## Configuration, diagnostics and prospective acceptance

Use the same board, option bytes, security state, reset priorities,
MPU windows and encodings, MAIR0=`44`, 58-word supervisor layout,
five user words, and protected sentinel specified by
[FLT-01-005](2026-11-27-cortex-m33-usagefault-conditional-unaligned-procedure.md).
The common field map is in the [campaign documentation](campaign/README.md).
The assembly wrapper captures PSP/MSP/SP/EXC_RETURN without stack
writes and checks the expected stack before entering Rust. The
handler checks frame bounds/type, CFSR stacking bits, exception
state and syndrome before read-only inspection. It is terminal;
there is no exception return, retry, or resumption.

Privileged startup must observe initial CCR.UNALIGN_TRP=0, then
explicitly clear bit 3 through read-modify-write, DSB/ISB and
readback. Configured and external CCR bit 3 must remain zero;
all other readable bits must be preserved. `BuildMode=2` identifies
this image. Common startup acceptance requires CONTROL/IPSR=`0/0`,
MSP in its reserve, exact three MPU regions, MPU_CTRL=5, MAIR0=`44`,
MAIR1=0, regions 3–7 disabled, armed SHCSR=`00050000`, zero
pre-probe CFSR/HFSR, and sentinel=`A5C35A3C`.

Per physical RESET, require:

| Observation | Required value |
|---|---|
| Supervisor Status/Phase/BuildMode | `1/4/2` |
| User words `20001000`–`20001010` | `1,0,44332211,A55A5AA5,0` |
| Positive second word | R7 compared to `88776655` before marker 1 |
| External SHCSR/CFSR/HFSR | `00050008/01000000/00000000` |
| Handler CONTROL/IPSR | `1/6` |
| Handler entry SP/MSP | `20003000/20003000` |
| EXC_RETURN/FramePsp/FrameValid | `FFFFFFBC/20001FE0/1` |
| PRIMASK/BASEPRI/FAULTMASK | `0/0/0` |
| Stacked R0/R1/R2 | `3/0/20002000` |
| Stacked PC | `08000340`, not continuation `08000344` |
| Faulting halfwords | `E9D5 6700` |
| Protected sentinel; unexpected IPSR | `A5C35A3C`; 0 |

Require `xPSR & 010003FF == 01000000`: Thumb state, interrupted
Thread mode and no stack-alignment padding. Do not impose a
post-fault R6/R7 value or fixed incidental R3/R12/LR/condition
flags. `FrameValid=1` is structural prerequisite evidence, not
general authenticity. The unchanged fault-value sentinel shows the
continuation did not publish a value; it does not reveal the
post-fault destination registers.

After each of two independent physical RESET executions, observe
the repeating three-slow-green-flash LD1 pattern qualitatively,
then collect all 58 supervisor words, five user words, protected
sentinel, CCR, three SCB words, CPUID and eight direct PSP frame
words: **77 observed words** per run. Require all eight physical
frame words at `[20001FE0,20002000)` to equal the corresponding
supervisor copy at offsets `98`–`B4`. Check the two runs separately
and compare them without rejecting incidental register differences.
Upload the full physical Flash image and compare it byte for byte
before replacement. The [campaign runner](campaign/README.md)
automates read-only evidence collection and comparison but its PASS
is provisional pending independent review.

## Limitations and errata

This demonstrates mandatory alignment of the selected LDRD under
the tested Normal-memory and MPU configuration. It does not test
Device memory, another load/store multiple, crossing an MPU
boundary, stacking failure, exception return, or a production fault
policy. The relevant ARM-ERRATA SDEN-756493 v14.0 r0p4 and ES0553
Rev. 6 silicon-X entries reviewed for FLT-01-004 do not describe
this aligned-control/unaligned-Normal-SRAM LDRD path; the Device
unaligned-access erratum is a different memory class. Preserve the
[board recovery policy](README.md), original option bytes, TZEN=0,
RDP Level 0, and human-only deployment.
