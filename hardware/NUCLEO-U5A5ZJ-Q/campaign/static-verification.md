# FLT-01-005–008 Linux static verification

**Date:** 2026-11-27  
**Status:** Static verification only; no physical execution or hardware PASS.

Linux tools: rustc 1.98.1 (`48a229cea`, 2026-09-01), cargo 1.98.1
(`797e8a9bc`, 2026-08-05), Python 3.12.3, and GNU Arm binutils
2.42 (`2.42-1ubuntu1+23`). The macOS host must record its own tools.

This report records one local Linux build at Git HEAD `4c2e8276f2fffaeea7be9a6dbc0b9e33047f33ed` with the new implementation files uncommitted. The manifest and full logs remain under `/tmp/flt-01-005-008-verification-20261127`. Reproduce the command in [README](README.md) on a clean macOS checkout and require source, payload, and image digest agreement before deployment. ELF hashes can differ between hosts.

## Build and image results

All five configurations passed standalone formatting, release target check, Clippy with denied warnings, rustdoc link validation, release build, ELF inspection, and independent GNU objcopy payload comparison. All root-workspace formatting, check, Clippy, tests, doctests, rustdoc, and `thumbv6m-none-eabi` check passed. The CI Rust line-length check passed for 35 files with zero violations.

| Configuration | ELF bytes / SHA-256 | Payload bytes / SHA-256 | Image bytes / SHA-256 | Fault/continuation |
|---|---|---|---|---|
| `005-disabled` | 141388 / `fcfd075008ab9e4884c121d852d70509259fc0c7418aaf98247762f063f044df` | 2268 / `421c3643c91b0f637dbf6785d13d2d7a883016833784ead392c1db0a6c72d8ff` | 2272 / `1db2b59818cc0931afde98c724787c9b2457a01db1a23a7d06ca6a7ffd61bf0c` | `08000334` / `08000338` |
| `005-enabled` | 141496 / `327a7dbdfce0669223cbfb1b21541bef7f3e4917a1956ac623f536ef71f09a30` | 3136 / `71463d058d72e6563350b499c50a6a7fd78fc542724c74715897337d9c485599` | 3136 / `71463d058d72e6563350b499c50a6a7fd78fc542724c74715897337d9c485599` | `08000334` / `08000338` |
| `006` | 141496 / `c9c2ae534ac025aa8965506ab2d52f6c889fe03307ecc661e489a7fd46071553` | 3148 / `37d2a5613d4db7b102a43d82ecddae17ccf23c99c935ddc0d53ce2038b702bb4` | 3152 / `bb31a1d1ce0fd9114129c8c931534be5ad4872b0e72c3187f145f53ea1c4a0aa` | `08000340` / `08000344` |
| `007` | 141560 / `8005b079bac900a9d35e7c36c3d7ef6c13fd8a07e828230df2af796e0b32342a` | 3164 / `857543c547ad357e3fb845eba15006c0685966f74b6070673fab2ba3c3f303c9` | 3168 / `708d7d027e2f8860829bf6a091b93040b83ea044d64e83141ab7a119eeb500c6` | `08000338` / `0800033A` |
| `008` | 142260 / `166c7f68f05c1771cde195048014f6ac0c41ec735175cdf68d21ec328b01acdd` | 3804 / `4e9f22be4e5c927bb79fa1cb2dc71e32392a102335bc49ec8e3ad40255dce922` | 3808 / `43b92b3ee6288a04fcd44b41f7e991257b3708e96752d9e84859d5df1fd3d58e` | `080002F2` / `080002F4` |

The Flash payloads fit `[08000000,08002000)` and all images are padded with only `FF` to 16-byte lengths. The padding lengths are 4, 0, 4, 4, and 4 bytes in table order. The two FLT-01-005 probe regions are byte-identical. Exact verified stimuli: `LDR.W` `F8D5 6001` (005), `LDRD` `E9D5 6700` (006), `BX r5` `4728` to target `0800034E` (007), and two `UDF #0x42` halfwords `DE42` at `080002F2` and `08000332` (008). FLT-01-008 also has SVC `DF72` at `0800030C` and `DF73` at `0800033C`.

## Source SHA-256

The two source trees are independently reproducible from these digests. Shared files are listed once. All paths below are relative to the named source tree.

| Source tree | File | SHA-256 |
|---|---|---|
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-alignment` | `Cargo.toml` | `ab4af690dce89cdf928ce12a3f82d180d715ccf00b9ad2bafea69b40dc23fd7d` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-alignment` | `Cargo.lock` | `542c405cb54739c747305fbe563d5115182ff8efc5c755f003b44fa9eb9354f0` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-alignment` | `build.rs` | `e03c4bf416547fd9c44ee88628f3b5c8010a3e18a9930c8500f9aeac65f12804` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-alignment` | `memory.x` | `2f8e89e9efc0b6e9255535fce0dd0d305041170e2557acaf7f690afa83ed87a5` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-alignment` | `inspect-elf.py` | `f79122f389de72117c7cfc45198457a6efec576efa35479460d768f9b63fc976` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-alignment` | `src/main.rs` | `71bfda444984ef1e5bad4969b1507d2b11f4f891b14b90ac9c01ff8ba67e1490` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-alignment` | `src/transition.s` | `bbe263ca25edb72943f578202a92b03e07d447e601afda3d8222884d7197116b` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-alignment` | `src/probe-invstate.s` | `f247019cd74b52bfe1c517e6bbbc4913a2fffbf512e6a726a799c9d3c19ee5ef` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-alignment` | `src/probe-mandatory.s` | `0198a46eb23186fdd51bf16dba9c195c782d3600b495bca2c3c9903a952f4927` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-alignment` | `src/probe-unaligned.s` | `9c9180221602b9c3cca154362bf700b649809a496780b325ef0c89ca43563bbf` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-repeated-expected` | `Cargo.toml` | `4312a593fa1b1b01c02183e3c5d647229488605bcce50d26ae86102bfad78654` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-repeated-expected` | `Cargo.lock` | `4331b8c77499189019a75b3addea07cbe2709587121416662b35d2b1b40c9a2b` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-repeated-expected` | `build.rs` | `e03c4bf416547fd9c44ee88628f3b5c8010a3e18a9930c8500f9aeac65f12804` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-repeated-expected` | `memory.x` | `2f8e89e9efc0b6e9255535fce0dd0d305041170e2557acaf7f690afa83ed87a5` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-repeated-expected` | `inspect-elf.py` | `3557483696d10510e4a8a13fd119ec2bc0219b6e074774563918eb1069b47df3` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-repeated-expected` | `src/main.rs` | `f515231df3b8bd51d6a2f2bd3823ac80bc67be96ff74537029b5ac52ecd2fdec` |
| `hardware/NUCLEO-U5A5ZJ-Q/usagefault-repeated-expected` | `src/transition.s` | `b5e053e5624569b0b9b891737800b98481d409e1bda935498a6f435618966e53` |

The shared host tools have SHA-256 `2d6ade26054855db139bc6453183be8f8f3c3d72c1d0f466d94f3fc90120711b`
for `campaign/build.py` and
`cf864cbb1849bc955a6cde840684cf04503ee989b92009f1b4b1f290a7ec762f`
for `campaign/runner.py`.

## Optimized intended-path stack assessment

Complete generated disassembly for each configuration was inspected. The assembly probes and entry wrappers make no stack allocation before their explicit entry checks. The alignment-tree startup pushes 36 bytes and allocates 48 bytes, totaling 84 bytes; it restores that allocation before the PSP handoff. `DefaultPreInit` pushes eight bytes.

| Configuration | Successful terminal handler MSP consumption | Predicted minimum MSP | Free in 2048-byte reserve |
|---|---:|---|---:|
| `005-disabled` | No successful exception handler; terminal Thread loop | `20003000` after startup | 2048 bytes |
| `005-enabled` and `006` | 8-byte push + 56-byte local allocation + 8-byte LED push = 72 bytes | `20002FB8` | 1976 bytes |
| `007` | 8-byte push + 64-byte local allocation + 8-byte LED push = 80 bytes | `20002FB0` | 1968 bytes |
| `008` | 8-byte wrapper push + 56-byte SVC handler frame + 32-byte `memcmp` frame = 96 bytes | `20002FA0` | 1952 bytes |

For FLT-01-008, the UsageFault path also uses the 8-byte wrapper push plus a 56-byte Rust frame (64 total); the final SVC-to-LED path uses 72 bytes. Its startup pushes 20 bytes and releases them before Thread/PSP entry. Each hardware basic frame uses 32 bytes of PSP at `[20001FE0,20002000)`; the second UDF and final SVC reuse that range only after exception return. All intended Handler-to-Rust and Handler-to-helper call sites are eight-byte aligned. This is a static assessment of these optimized machine-code paths under the controlled assumptions, not a runtime high-water measurement or an arbitrary-fault/nesting bound. The unexpected-exception path still presumes usable MSP.

## Inspector and host-runner checks

Four temporary ELF copies were modified at the selected stimulus instruction (005 enabled, 006, 007, 008). Each inspector rejected the modified copy. The release ELFs were not modified. Python bytecode compilation passed for both inspectors and both campaign scripts. The runner dry-run generated the expected five command plans without hardware access.

No macOS build, physical RESET run, SWD observation, physical Flash readback, or option-byte measurement was performed during this Linux static verification. A host runner PASS remains provisional until independent engineering review.

The applicable errata review used [ARM-ERRATA,
SDEN-756493 v14.0, errata 1080541, 1435973 and
2219175](../references.md) and [ES0553 Rev. 6, Table 2 and
§2.1.1, printed pp. 2, 5](../references.md). The identified
unaligned Device-memory, interrupt-priority update and partial
floating-point load conditions do not match these Normal-SRAM,
static-priority, inactive-FP stimuli. The architectural predictions
remain prospective until physical evidence is reviewed.
