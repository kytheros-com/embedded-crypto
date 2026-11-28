# FLT-01-005 — Conditional alignment enforcement evidence

**Hardware execution date:** 2026-11-28

**Result:** PASS against the committed prospective criteria; two independent physical RESET executions per configuration

**Scope:** Standalone Platform Integration Layer (PIL) characterization

## Basis and provenance

The [prospective
procedure](../2026-11-27-cortex-m33-usagefault-conditional-unaligned-procedure.md)
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
[`hardware/NUCLEO-U5A5ZJ-Q/usagefault-alignment/`](../usagefault-alignment/).

| File (relative to source tree) | SHA-256 |
|---|---|
| `Cargo.toml` | `ab4af690dce89cdf928ce12a3f82d180d715ccf00b9ad2bafea69b40dc23fd7d` |
| `Cargo.lock` | `542c405cb54739c747305fbe563d5115182ff8efc5c755f003b44fa9eb9354f0` |
| `build.rs` | `e03c4bf416547fd9c44ee88628f3b5c8010a3e18a9930c8500f9aeac65f12804` |
| `memory.x` | `2f8e89e9efc0b6e9255535fce0dd0d305041170e2557acaf7f690afa83ed87a5` |
| `inspect-elf.py` | `f79122f389de72117c7cfc45198457a6efec576efa35479460d768f9b63fc976` |
| `src/main.rs` | `71bfda444984ef1e5bad4969b1507d2b11f4f891b14b90ac9c01ff8ba67e1490` |
| `src/transition.s` | `bbe263ca25edb72943f578202a92b03e07d447e601afda3d8222884d7197116b` |
| `src/probe-invstate.s` | `f247019cd74b52bfe1c517e6bbbc4913a2fffbf512e6a726a799c9d3c19ee5ef` |
| `src/probe-mandatory.s` | `0198a46eb23186fdd51bf16dba9c195c782d3600b495bca2c3c9903a952f4927` |
| `src/probe-unaligned.s` | `9c9180221602b9c3cca154362bf700b649809a496780b325ef0c89ca43563bbf` |

### Artifact identities and static checks

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `005-disabled` Linux ELF | 141,388 | `fcfd075008ab9e4884c121d852d70509259fc0c7418aaf98247762f063f044df` |
| `005-disabled` macOS ELF | 141,400 | `0e9e76595dfeaddfd3e6aa24e377b6051bb9399425cbd28dcf8b5a5bf3de9237` |
| `005-disabled` payload, both hosts | 2,268 | `421c3643c91b0f637dbf6785d13d2d7a883016833784ead392c1db0a6c72d8ff` |
| `005-disabled` image, both hosts | 2,272 | `1db2b59818cc0931afde98c724787c9b2457a01db1a23a7d06ca6a7ffd61bf0c` |
| `005-disabled` physical Flash readback | 2,272 | `1db2b59818cc0931afde98c724787c9b2457a01db1a23a7d06ca6a7ffd61bf0c` |
| `005-enabled` Linux ELF | 141,496 | `327a7dbdfce0669223cbfb1b21541bef7f3e4917a1956ac623f536ef71f09a30` |
| `005-enabled` macOS ELF | 141,508 | `c9f32394aa3463e0d1afcaad60a0c126c15d471fab554668fd58e7dfc261cbce` |
| `005-enabled` payload, both hosts | 3,136 | `71463d058d72e6563350b499c50a6a7fd78fc542724c74715897337d9c485599` |
| `005-enabled` image, both hosts | 3,136 | `71463d058d72e6563350b499c50a6a7fd78fc542724c74715897337d9c485599` |
| `005-enabled` physical Flash readback | 3,136 | `71463d058d72e6563350b499c50a6a7fd78fc542724c74715897337d9c485599` |

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
| `005-disabled` | `2026-11-28T16:48:03Z` | `2026-11-28T16:48:39Z`–`2026-11-28T16:48:40Z` | `2026-11-28T16:49:47Z`–`2026-11-28T16:49:47Z` |
| `005-disabled` run 1 | RESET `2026-11-28T16:49:20Z` | LED `2026-11-28T16:49:24Z` (dark) | collection `2026-11-28T16:49:25Z` |
| `005-disabled` run 2 | RESET `2026-11-28T16:49:43Z` | LED `2026-11-28T16:49:46Z` (dark) | collection `2026-11-28T16:49:47Z` |
| `005-enabled` | `2026-11-28T16:49:47Z` | `2026-11-28T16:50:29Z`–`2026-11-28T16:50:30Z` | `2026-11-28T16:51:06Z`–`2026-11-28T16:51:07Z` |
| `005-enabled` run 1 | RESET `2026-11-28T16:50:47Z` | LED `2026-11-28T16:50:50Z` (three) | collection `2026-11-28T16:50:51Z` |
| `005-enabled` run 2 | RESET `2026-11-28T16:51:03Z` | LED `2026-11-28T16:51:05Z` (three) | collection `2026-11-28T16:51:06Z` |

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

### `005-disabled`: complete observed words

**Supervisor record at `0x20002000`** (`58` 32-bit words; index is decimal):

```text
00:00000002 01:00000003 02:00000000 03:00000000
04:20003000 05:00000800 06:00000000 07:00000005
08:00000044 09:00000000 10:08000006 11:08001FE1
12:20001003 13:20001FE1 14:20002001 15:20002FE1
16:00000000 17:00000000 18:00000000 19:00050000
20:08000334 21:0000F8D5 22:A5C35A3C 23:00000000
24:00000000 25:00000000 26:00000000 27:00000000
28:00000000 29:00000000 30:00000000 31:00000000
32:00000000 33:00000000 34:00000000 35:00000000
36:00000000 37:00000000 38:00000000 39:00000000
40:00000000 41:00000000 42:00000000 43:00000000
44:00000000 45:00000000 46:00000000 47:00000000
48:00000000 49:00000201 50:00000201 51:00000000
52:00000000 53:00000000 54:00000000 55:00000000
56:00006001 57:00000000
```

**User words at `0x20001000`** (`5` 32-bit words; index is decimal):

```text
00:00000001 01:00000002 02:44332211 03:55443322
04:00000000
```

External CCR=`00000201`, SHCSR/CFSR/HFSR=`00050000`/`00000000`/`00000000`,
CPUID=`410FD214`, protected sentinel=`A5C35A3C`.

Run 2 reproduced every one of these 69 words exactly, including incidental
stacked registers and flags. The comparison was independently recomputed from
both sets of raw SWD transcripts.

### `005-enabled`: complete observed words

**Supervisor record at `0x20002000`** (`58` 32-bit words; index is decimal):

```text
00:00000001 01:00000004 02:00000000 03:00000000
04:20003000 05:00000800 06:00000000 07:00000005
08:00000044 09:00000000 10:08000006 11:08001FE1
12:20001003 13:20001FE1 14:20002001 15:20002FE1
16:00000000 17:00000000 18:00000000 19:00050000
20:08000334 21:0000F8D5 22:A5C35A3C 23:00000001
24:00000000 25:00000001 26:00000006 27:20003000
28:20003000 29:FFFFFFBC 30:20001FE0 31:00000000
32:00000000 33:00000000 34:00050008 35:01000000
36:00000000 37:00000001 38:00000003 39:00000000
40:20002000 41:00000201 42:08000006 43:08001FE1
44:08000334 45:29000000 46:0000F8D5 47:A5C35A3C
48:00000000 49:00000201 50:00000209 51:00000209
52:00000001 53:44332211 54:A55A5AA5 55:00000000
56:00006001 57:00006001
```

**User words at `0x20001000`** (`5` 32-bit words; index is decimal):

```text
00:00000001 01:00000000 02:44332211 03:A55A5AA5
04:00000000
```

**Direct physical PSP frame at `0x20001FE0`** (`8` 32-bit words; index is decimal):

```text
00:00000003 01:00000000 02:20002000 03:00000201
04:08000006 05:08001FE1 06:08000334 07:29000000
```

External CCR=`00000209`, SHCSR/CFSR/HFSR=`00050008`/`01000000`/`00000000`,
CPUID=`410FD214`, protected sentinel=`A5C35A3C`.

Run 2 reproduced every one of these 77 words exactly, including incidental
stacked registers and flags. The comparison was independently recomputed from
both sets of raw SWD transcripts.

The aligned positive control was linked at `0800031C` in both builds. The
disabled run had startup Status=`2` and Phase=`3`: these are its deliberate
control-state markers. Successful unaligned completion is established by user
progress/continuation `1/2`, loaded value `55443322`, and absent CFSR/HFSR,
not by Status alone. Both aligned controls yielded `44332211`. Disabled and
enabled CCR were `00000201` and `00000209`; the enabled run captured
`CFSR=01000000`, `HFSR=0`, stacked PC `08000334`, and continuation marker
zero. The two images had identical probe machine code (`LDR.W` `F8D5 6001` at
`08000334`); enabled continuation was `08000338`. The enabled physical frame
matched the diagnostic copy. These observations support conditional alignment
enforcement under the two CCR configurations.

## Architectural reference

ARM-ARM DDI0553B.z §D1.2.9 (p. 1474) defines the CCR control; §C2.4.78 (pp. 667–670) defines the LDR access; §§B3.12 and D1.2.270 (pp. 91, 1898) assign UNALIGNED to CFSR bit 24. §B3.18 (p. 105, RQLHB) supplies the synchronous stacked-PC rule for this fault. The full source catalogue is in [references](../references.md).

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
