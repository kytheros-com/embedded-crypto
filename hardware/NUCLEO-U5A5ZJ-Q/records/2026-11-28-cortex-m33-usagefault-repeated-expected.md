# FLT-01-008 — Repeated expected-fault recovery evidence

**Hardware execution date:** 2026-11-28

**Result:** PASS against the committed prospective criteria; two independent physical RESET executions per configuration

**Scope:** Standalone Platform Integration Layer (PIL) characterization

## Basis and provenance

The [prospective
procedure](../2026-11-27-cortex-m33-usagefault-repeated-expected-procedure.md)
supplies the acceptance criteria; this completed record does not revise them.
The [campaign protocol](../campaign/README.md), [historical Linux static
report](../campaign/static-verification.md), and [test
ledger](../test-ledger.md) are distinct records. The deployed source revision
was `4014a41cbe8877efd543db5bac1efc5c29a4fd45` on `hardware/NUCLEO-U5A5ZJ-Q`.
The historical static report retains its earlier uncommitted-build provenance.

The Linux reference manifest SHA-256 was
`d72264d5a50af2c2aa85e4588658e9a67f02b3f3a9d4ae9e22d0bc77a40ff532`; macOS
manifest SHA-256 was
`d49efe27c666d56ad5ae0634d7796afce58e50472fb9ca92fda72b25e488b056`; physical
archive SHA-256 was
`61bddde604095ed8335bdce6682111b15da12d93c9891a6d72fb4c7f6feef804`; campaign
summary SHA-256 was
`0ec548c8d9c0863a51c078cde52d9f43225d03f0f9213f9427f7e27310a04672`. All four
were independently verified before review. The raw archives remain
operator-held; complete observed values are transcribed below.

Linux: x86_64, Linux 7.0.0-1018-gcp, rustc/cargo 1.98.1, Python 3.12.3, GNU
Arm binutils 2.42 (`2.42-1ubuntu1+23`). macOS: arm64, macOS 27.2.1 (Darwin
27.2.0), rustc/cargo 1.98.1, Python 3.14.7, GNU Arm binutils 2.47.20260726.
The macOS working tree contained an **untracked Linux reference manifest** at
`hardware/NUCLEO-U5A5ZJ-Q/campaign/manifest.json`; no tracked firmware source
modification is indicated. Both manifests record matching campaign-script
digests: `build.py`
`6c77526372d8da23f21b41df700a4aa2ead789313268dbfabb5b6398db621a01` and
`runner.py`
`c339c49a6687fa3fd76910d92d3d485e97d970c5d270d53e89b5fad03250d88d`.

The board-info transcripts identify NUCLEO-U5A5ZJ-Q, STM32U5A5ZJ device ID
`0x481`, revision X, 4 MiB NVM, Cortex-M33 r0p4 (`CPUID=410FD214`), ST-LINK
V3J10M3 serial `003800353335511835383531`, STM32CubeProgrammer 2.24.0, 3.27 V
reported, and 1000 kHz SWD. Each predeployment display matched all 35 fields
in the [original option-byte
baseline](../stm32u5a5zj-original-option-bytes.txt), including RDP Level 0 and
TZEN=0.

### Source-file SHA-256 at the deployed revision

The source tree is
[`hardware/NUCLEO-U5A5ZJ-Q/usagefault-repeated-expected/`](../usagefault-repeated-expected/).

| File (relative to source tree) | SHA-256 |
|---|---|
| `Cargo.toml` | `4312a593fa1b1b01c02183e3c5d647229488605bcce50d26ae86102bfad78654` |
| `Cargo.lock` | `4331b8c77499189019a75b3addea07cbe2709587121416662b35d2b1b40c9a2b` |
| `build.rs` | `e03c4bf416547fd9c44ee88628f3b5c8010a3e18a9930c8500f9aeac65f12804` |
| `memory.x` | `2f8e89e9efc0b6e9255535fce0dd0d305041170e2557acaf7f690afa83ed87a5` |
| `inspect-elf.py` | `3557483696d10510e4a8a13fd119ec2bc0219b6e074774563918eb1069b47df3` |
| `src/main.rs` | `f515231df3b8bd51d6a2f2bd3823ac80bc67be96ff74537029b5ac52ecd2fdec` |
| `src/transition.s` | `b5e053e5624569b0b9b891737800b98481d409e1bda935498a6f435618966e53` |

### Artifact identities and static checks

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `008` Linux ELF | 142,260 | `166c7f68f05c1771cde195048014f6ac0c41ec735175cdf68d21ec328b01acdd` |
| `008` macOS ELF | 142,272 | `584d52983cf5a3df669085c0d267fb623f42a659170dd2ed763982ca85552f15` |
| `008` payload, both hosts | 3,804 | `4e9f22be4e5c927bb79fa1cb2dc71e32392a102335bc49ec8e3ad40255dce922` |
| `008` image, both hosts | 3,808 | `43b92b3ee6288a04fcd44b41f7e991257b3708e96752d9e84859d5df1fd3d58e` |
| `008` physical Flash readback | 3,808 | `43b92b3ee6288a04fcd44b41f7e991257b3708e96752d9e84859d5df1fd3d58e` |

All five Linux rebuilds reproduced their original Linux ELF, payload, and
image digests. For each configuration, macOS reproduced the Linux payload and
programming-image digest and size; macOS ELF containers differed. Both hosts
recorded passing ELF inspection and independent GNU objcopy payload
comparisons; the macOS archive contains readelf, disassembly, and inspector
outputs. Every physical readback binary was independently hashed and byte
compared with the reviewed image. Generated build artifacts are not retained
in Git.

### Evidence collection times (UTC)

| Configuration / event | Predeployment option bytes or RESET | Programming or LED report | Readback or collection |
|---|---|---|---|
| `008` | `2026-11-28T16:53:15Z` | `2026-11-28T16:53:43Z`–`2026-11-28T16:53:43Z` | `2026-11-28T16:54:20Z`–`2026-11-28T16:54:20Z` |
| `008` run 1 | RESET `2026-11-28T16:54:01Z` | LED `2026-11-28T16:54:04Z` (three) | collection `2026-11-28T16:54:05Z` |
| `008` run 2 | RESET `2026-11-28T16:54:16Z` | LED `2026-11-28T16:54:18Z` (three) | collection `2026-11-28T16:54:20Z` |

The ten run directories contain no reported failed execution or manual retry.
The 95 raw programmer transcripts each have zero exit status. Programming
transcripts state that internal memory sector 0 was erased and verification
completed. The complete campaign ended at `2026-11-28T16:54:20Z`.

## Physical observations

The values below are unsigned 32-bit hexadecimal words. Addresses are byte
addresses; diagnostic word index zero begins at the stated base, with
successive words in increasing address order. Instruction halfwords are shown
in architectural hexadecimal notation; the stored Flash bytes use
little-endian order. Each configuration was independently reset twice.

### `008`: complete observed words

**Supervisor record at `0x20002000`** (`64` 32-bit words; index is decimal):

```text
00:00000001 01:00000005 02:00000002 03:00000002
04:00000003 05:00000005 06:00000000 07:00000000
08:00000000 09:00050000 10:00000201 11:A5C35A3C
12:A5C35A3C 13:00000005 14:00000000 15:00000000
16:00000003 17:00000000 18:20002000 19:00000001
20:20002001 21:20002100 22:080002F2 23:29000000
24:00010000 25:00000000 26:00050008 27:FFFFFFBC
28:20001FE0 29:20003000 30:00000001 31:00000006
32:080002F2 33:080002F4 34:00000001 35:00000001
36:00000003 37:00000000 38:20002000 39:00000001
40:20002001 41:20002100 42:08000332 43:69000000
44:00010000 45:00000000 46:00050008 47:FFFFFFBC
48:20001FE0 49:20003000 50:00000001 51:00000006
52:08000332 53:08000334 54:00000002 55:00000001
56:00000000 57:00000000 58:00000000 59:00000000
60:00000000 61:00000000 62:00000000 63:00000000
```

**User words at `0x20001000`** (`6` 32-bit words; index is decimal):

```text
00:00000001 01:00000002 02:00000003 03:00000004
04:00000005 05:00000000
```

**Direct physical PSP frame at `0x20001FE0`** (`8` 32-bit words; index is decimal):

```text
00:00000003 01:00000000 02:20002000 03:00000001
04:20002001 05:20002100 06:0800033E 07:29000000
```

**Protected expected-fault descriptor at `0x20002104`** (`12` 32-bit words; index is decimal):

```text
00:00000005 01:00000000 02:00000002 03:00000002
04:00000006 05:00010000 06:00000003 07:08000332
08:08000334 09:20001FE0 10:FFFFFFBC 11:00000005
```

External CCR=`00000201`, SHCSR/CFSR/HFSR=`00050080`/`00000000`/`00000000`,
CPUID=`410FD214`, protected sentinel=`A5C35A3C`.

Run 2 reproduced every one of these 96 words exactly, including incidental
stacked registers and flags. The comparison was independently recomputed from
both sets of raw SWD transcripts.

The fixed SVC sites were `0800030C` (arm second, return `0800030E`) and
`0800033C` (finish, physical frame PC `0800033E`). The 64-word record contains
both complete historical UsageFault slots: words 16–35 are fault 1 and words
36–55 are fault 2. Each slot preserves the original eight-word frame before
syndrome clearing and PC replacement, followed by CFSR, HFSR, SHCSR,
EXC_RETURN, PSP, entry MSP, Handler CONTROL, Handler IPSR, linked fault PC,
authorized continuation, test ID, and structural-valid marker. The first fault
PC/continuation were `080002F2`/`080002F4`; the second were
`08000332`/`08000334`. Both report `CFSR=00010000`, `HFSR=0`,
`SHCSR=00050008`, `EXC_RETURN=FFFFFFBC`, `PSP=20001FE0`, `MSP=20003000`,
CONTROL=1, IPSR=6. Stacked xPSR was `29000000` then `69000000`.

The final protected descriptor words are
`5,0,2,2,6,00010000,3,08000332,08000334,20001FE0,FFFFFFBC,5`.
Status/Phase=`1/5`, FaultCount/SvcCount=`2/2`,
PositiveMask/CompletionMask=`3/5`, DescriptorState/Armed=`5/0`, and Error=0.
Initial and final CFSR are zero; all eight reserved record words (56–63) are
zero. User words are `1,2,3,4,5,0`; the protected sentinel is unchanged. Final
external SHCSR/CFSR/HFSR are `00050080/0/0`.

The directly read physical PSP frame is the **final active SVC frame**, with
PC=`0800033E`; it is not a surviving UDF frame. The two UDF frames are
historical protected copies. The observations support two separately
authorized, controlled UsageFault returns in one boot, with expected
sticky-syndrome clearing and one-shot descriptor retirement between stimuli.
This is experimental support for multiple controlled protection-mechanism
self-tests without a reset between individual tests.

A candidate future PIL power-on self-test (POST) pattern is: establish a
protected one-shot expected-fault descriptor; execute a controlled stimulus;
validate exception metadata and structural frame prerequisites; preserve
original diagnostics; clear and verify the expected sticky syndrome; install
the prescribed continuation and consume authorization before return; then
verify continued execution before arming another test. This experiment did not
implement a production POST. Controlled recovery here does not establish safe
recovery from unexpected operational faults.

## Architectural reference

ARM-ARM DDI0553B.z §C2.4.259 defines UDF T1; §B3.12 (pp. 90–91) assigns UNDEFINSTR to CFSR bit 16; §B3.18 (p. 105, RQLHB) governs faulting UDF PC. §§B3.19, B3.22 (pp. 106–118) and §D1.2.96 (pp. 1628–1629) describe the basic frame and exception return; §D1.2.233 (p. 1841) defines final SVCALLACT status. The full source catalogue is in [references](../references.md).

## Interpretation and limits

These are controlled experiments on this board with TrustZone disabled. They
do not establish general security isolation, production readiness, or FIPS
validation. Structural frame checks do not authenticate an arbitrary frame;
cooperating test software is assumed, so adversarial caller authentication is
not established. Corrupted stacks, stacking or unstacking failures, nested
exceptions, and lockup were outside scope. The intended-path static stack
assessment in the historical static report is not an arbitrary-fault or
nesting bound.

Physical RESET confirmations and LED descriptions are operator reports, not
instrumented measurements. Option bytes were read before each image
deployment, not after the final run. Flash upload covered the reviewed
programming-image interval from `0x08000000`, not the entire erase page. The
programming transcripts report erasure of internal memory sector 0.
Recovery-image availability was not independently established in this campaign
review. No operational fault-handling policy change follows from this
evidence. Human engineering acceptance is still required.
