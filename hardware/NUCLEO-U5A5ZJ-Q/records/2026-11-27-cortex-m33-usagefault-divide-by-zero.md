# FLT-01-004 — Conditional integer divide-by-zero UsageFault evidence

**Hardware execution date:** 2026-11-27

**Result:** PASS — four physical RESET executions met their respective
prospective acceptance criteria

**Scope:** Standalone Platform Integration Layer (PIL) characterization

## Objective and evidence basis

The [prospective procedure](../2026-11-27-cortex-m33-usagefault-divide-by-zero-procedure.md)
defines separate acceptance criteria for `CCR.DIV_0_TRP=0` and `=1`.
The [source](../usagefault-divide-by-zero/) has separately compiled
images with identical assembly probes in unprivileged Thread mode on
PSP. The [ledger](../test-ledger.md) indexes this result. The procedure
remains prospective documentation and is not changed by this record.

The human operator reported both deployments, two separate physical
RESET executions per configuration, and a physical Flash upload for
each image. ChatGPT independently reviewed the supplied macOS ELF
section and symbol dumps, complete disassemblies, inspector results,
source and artifact digests, intermediate option-byte display, four
diagnostic captures, and both Flash readbacks. This record reproduces
the supplied address/value dumps; complete CLI and macOS build-check
transcripts were not deposited in the repository. Its author did not
operate the board or witness the resets.

## Board, configuration, and provenance

| Item | Operator-reported value or reviewed observation |
|---|---|
| Board; MCU | NUCLEO-U5A5ZJ-Q; STM32U5A5ZJ |
| Device ID; silicon revision | `0x481`; X |
| Processor | Cortex-M33 r0p4; CPUID `0x410FD214` in all four runs |
| NVM; bootloader | 4 MiB; version `0x92` |
| ST-LINK | V3J10M3; serial `003800353335511835383531` |
| Supply; diagnostic SWD | 3.27 V reported; 1000 kHz, Hot Plug |
| Programmer | STM32CubeProgrammer 2.24.0 |
| Security baseline | RDP Level 0; TZEN=0; no option-byte change reported |

The clean macOS source checkout was at commit
`260d8145d623af93b395406182f1846ca714de7f`, branch
`hardware/NUCLEO-U5A5ZJ-Q`, synchronized with origin at collection.
The host was macOS 27.2.1 (26B131), arm64, with rustc 1.98.1
(`48a229cea`, 2026-09-01), cargo 1.98.1 (`797e8a9bc`,
2026-08-05), Python 3.14.7, GNU objcopy 2.47.20260726, and target
`thumbv8m.main-none-eabi`.

All seven supplied macOS source digests match the Linux reference
digests in the procedure and the current tracked files:

| Source file | SHA-256 |
|---|---|
| `src/main.rs` | `6a6112168c68c6f277ecbdabd25497e3fa1de6e932bb173e35e14391d858418c` |
| `src/transition.s` | `a76d2b3ac0a4e079ab590c35a9595729ad8f025f7142012fc5c39558831525e5` |
| `inspect-elf.py` | `ac827566a481534af03e966829857d6812b146bd4d307e7763ecf101f03f59f6` |
| `Cargo.toml` | `08e3b7833f0259eefb684f070edf7aaf684b90d12272046882de06b4621ba170` |
| `Cargo.lock` | `588c37098aebf451806f1ee0383d091fa32d787bca8e2d7c9db8743a4151a36e` |
| `build.rs` | `e03c4bf416547fd9c44ee88628f3b5c8010a3e18a9930c8500f9aeac65f12804` |
| `memory.x` | `2f8e89e9efc0b6e9255535fce0dd0d305041170e2557acaf7f690afa83ed87a5` |

The earlier Linux reference builds used HEAD
`217c0003d5c5423f393fbbb9559a8686392331a9` plus then-uncommitted
FLT-01-004 implementation files. They are not attributed to the later
macOS source commit. The Linux standalone and root-workspace checks
recorded in the procedure remain Linux evidence. Complete macOS
formatting, Clippy, rustdoc, and root-workspace check transcripts were
not supplied for independent review.

| Configuration and artifact | Bytes | SHA-256 |
|---|---:|---|
| Disabled macOS ELF | 141,392 | `ef842927ce0b65883094f6c4681fbb6dfb4a4fcbc108a5a6ef4e21c46b9992ee` |
| Disabled Linux ELF | 141,380 | `2a42b72b30cd60be9f5acd90aa2d8a08a452c1c0df1abe8f75f6e2cc3daaa231` |
| Disabled payload, both hosts | 2,196 | `a7dd87ecfd734e338b9a6e503d256b4c880f1557c247ec7dc362ff8b034c3147` |
| Disabled image, both hosts | 2,208 | `48f6ab644492d11ed6fad9bd8aa197466a5b9f8e7da5e41be1dbd9e7d1f441f8` |
| Disabled physical readback | 2,208 | `48f6ab644492d11ed6fad9bd8aa197466a5b9f8e7da5e41be1dbd9e7d1f441f8` |
| Enabled macOS ELF | 141,504 | `70e323d3865d8143ebcf22c29ca3b79f19820263f6caedadc69cdf6da95739f0` |
| Enabled Linux ELF | 141,492 | `5a0e3c49586105f1f784b43d56a752981c09d9f57fbcd3e0455902d5fa65759d` |
| Enabled payload, both hosts | 3,092 | `ef9db9efe27fe04acf17695efc0a42a1e218d4a4624cb89b3221ff1929ccbed6` |
| Enabled image, both hosts | 3,104 | `8fc145b1961491968cd51e32de3d7723091826a426c92076577a91e191f81f3f` |
| Enabled physical readback | 3,104 | `8fc145b1961491968cd51e32de3d7723091826a426c92076577a91e191f81f3f` |

Each image has twelve final `FF` padding bytes. Both independent
macOS GNU objcopy payload comparisons and both ELF inspectors passed;
the inspectors' peer comparison established identical probe machine
code. Each physical upload started at `0x08000000`, and its byte-for-byte
`cmp` against the matching image passed. The disabled upload preceded
replacement by the enabled image. The macOS ELFs are each twelve
bytes larger than their Linux counterparts; the cause was not
investigated. Matching payload and image hashes establish deployable
byte agreement across hosts, not identical ELF containers.

### Intermediate read-only option-byte evidence

After the disabled runs and disabled Flash readback, and before the
enabled image was programmed, the operator ran
`stm32cli -c port=SWD mode=UR freq=1000 -ob displ`.
All 35 displayed fields were compared with the
[original option-byte record](../stm32u5a5zj-original-option-bytes.txt)
and matched:

```text
RDP            AA       BOR_LEV        0    TZEN           0
nRST_STOP       1       nRST_STDBY     1    nRST_SHDW      1
SRAM_RST        1       IWDG_SW        1    IWDG_STOP      1
IWDG_STDBY      1       WWDG_SW        1    SWAP_BANK      0
BKPRAM_ECC      1       SRAM3_ECC      1    SRAM2_ECC      1
SRAM2_RST       1       nSWBOOT0       1    nBOOT0         1
PA15_PUPEN      1       IO_VDD_HSLV    0    IO_VDDIO2_HSLV 0
NSBOOTADD0 100000       NSBOOTADD1 17F200
WRP1A_PSTRT    FF       WRP1A_PEND    0    UNLOCK_1A      1
WRP1B_PSTRT    FF       WRP1B_PEND    0    UNLOCK_1B      1
WRP2A_PSTRT    FF       WRP2A_PEND    0    UNLOCK_2A      1
WRP2B_PSTRT    FF       WRP2B_PEND    0    UNLOCK_2B      1
```

Decoded boot addresses are `0x08000000` and `0x0BF90000`.
RDP=`0xAA` is Level 0; TZEN remains zero. SRAM_RST and SRAM2_RST
remain one, allowing SRAM retention across system reset. The
option-byte display establishes BOOT0 selection policy, not physical
BOOT0 pin voltage. This capture is between deployments, not a fresh
pre-deployment measurement for the disabled image.

## Four physical RESET executions

Times below are UTC evidence-collection times on 2026-11-27, not
precise reset times. The operator reported a separate physical RESET
for each row. Each run was evaluated independently.

| Run | Collection began | Collection ended if reported | LD1 |
|---|---|---|---|
| Disabled 1 | 19:24:27 | — | Dark |
| Disabled 2 | 19:27:38 | 19:27:39, including readback | Dark |
| Enabled 1 | 19:32:56 | — | Repeating three slow green flashes, then pause |
| Enabled 2 | 19:35:14 | 19:35:15, including readback | Same pattern |

LED observations were qualitative, with no instrumental timing.
Darkness is not disabled-control acceptance by itself; flashes are
not enabled-stimulus acceptance by themselves.

### Disabled Runs 1 and 2: complete, identical raw observations

The following 58-word supervisor dump was separately supplied for
**each** disabled run and is identical in both. Status=2 means
configured control, not completed division. Completion is established
by the separate user-region and SCB observations.

```text
20002000 : 00000002 00000003 00000000 00000000
20002010 : 20003000 00000800 00000000 00000005
20002020 : 00000044 00000000 08000006 08001FE1
20002030 : 20001003 20001FE1 20002001 20002FE1
20002040 : 00000000 00000000 00000000 00050000
20002050 : 08000302 0000FBB3 A5C35A3C 00000000
20002060 : 00000000 00000000 00000000 00000000
20002070 : 00000000 00000000 00000000 00000000
20002080 : 00000000 00000000 00000000 00000000
20002090 : 00000000 00000000 00000000 00000000
200020A0 : 00000000 00000000 00000000 00000000
200020B0 : 00000000 00000000 00000000 00000000
200020C0 : 00000000 00000201 00000201 00000000
200020D0 : 00000000 00000000 00000000 00000000
200020E0 : 0000F6FC 00000000
```

These independent reads were also supplied separately and identically
for both disabled runs (11 words beyond the supervisor record):

```text
20001000 : 00000001 00000002 00000006 00000000
20001010 : 00000000
200020E8 : A5C35A3C
E000ED14 : 00000201
E000ED24 : 00050000 00000000 00000000
E000ED00 : 410FD214
```

Thus **69 observed words per disabled run** comprise 58 supervisor,
five user, one sentinel, one CCR, three SCB, and one CPUID word.
All 69 agree between Disabled Runs 1 and 2. No PSP exception-frame
read was performed or expected; residual PSP SRAM is not interpreted
as an exception frame.

### Enabled Runs 1 and 2: complete, identical raw observations

The following 58-word supervisor dump was separately supplied for
**each** enabled run and is identical in both:

```text
20002000 : 00000001 00000004 00000000 00000000
20002010 : 20003000 00000800 00000000 00000005
20002020 : 00000044 00000000 08000006 08001FE1
20002030 : 20001003 20001FE1 20002001 20002FE1
20002040 : 00000000 00000000 00000000 00050000
20002050 : 08000302 0000FBB3 A5C35A3C 00000001
20002060 : 00000000 00000001 00000006 20003000
20002070 : 20003000 FFFFFFBC 20001FE0 00000000
20002080 : 00000000 00000000 00050008 02000000
20002090 : 00000000 00000001 00000003 00000000
200020A0 : 20002000 0000002A 00000000 08001FE1
200020B0 : 08000302 29000000 0000FBB3 A5C35A3C
200020C0 : 00000000 00000201 00000211 00000211
200020D0 : 00000001 00000006 A55A5AA5 00000000
200020E0 : 0000F6FC 0000F6FC
```

These independent reads were also supplied separately and identically
for both enabled runs (19 words beyond the supervisor record):

```text
20001FE0 : 00000003 00000000 20002000 0000002A
20001FF0 : 00000000 08001FE1 08000302 29000000
20001000 : 00000001 00000000 00000006 A55A5AA5
20001010 : 00000000
200020E8 : A5C35A3C
E000ED14 : 00000211
E000ED24 : 00050008 02000000 00000000
E000ED00 : 410FD214
```

Thus **77 observed words per enabled run** comprise 58 supervisor,
eight direct physical PSP frame, five user, one sentinel, one CCR,
three SCB, and one CPUID word. All 77 agree between Enabled Runs 1
and 2. In **each** run, the eight physical words at
`[0x20001FE0,0x20002000)` match the handler copy at supervisor
offsets `98` through `B4` (`0x20002098` through `0x200020B4`).

## Complete diagnostic decoding and acceptance

Values below decode every supervisor field in the procedure's
58-word map. Zero in a disabled handler field denotes its observed
startup-cleared value and the absence of a handler invocation. Hex
values omit `0x`; all record words are little-endian 32-bit words.

| Offset | Field | Disabled, both runs | Enabled, both runs |
|---|---|---|---|
| `00` | Status | `00000002` | `00000001` |
| `04` | Phase | `00000003` | `00000004` |
| `08` | StartControl | `00000000` | `00000000` |
| `0C` | StartIpsr | `00000000` | `00000000` |
| `10` | StartMsp | `20003000` | `20003000` |
| `14` | MpuType | `00000800` | `00000800` |
| `18` | InitialMpuCtrl | `00000000` | `00000000` |
| `1C` | MpuCtrl | `00000005` | `00000005` |
| `20` | Mair0 | `00000044` | `00000044` |
| `24` | Mair1 | `00000000` | `00000000` |
| `28` | FlashRbar | `08000006` | `08000006` |
| `2C` | FlashRlar | `08001FE1` | `08001FE1` |
| `30` | UserRbar | `20001003` | `20001003` |
| `34` | UserRlar | `20001FE1` | `20001FE1` |
| `38` | SupervisorRbar | `20002001` | `20002001` |
| `3C` | SupervisorRlar | `20002FE1` | `20002FE1` |
| `40` | UnusedEnabledMask | `00000000` | `00000000` |
| `44` | InitialCfsr | `00000000` | `00000000` |
| `48` | InitialHfsr | `00000000` | `00000000` |
| `4C` | ArmedShcsr | `00050000` | `00050000` |
| `50` | ExpectedUdivAddress | `08000302` | `08000302` |
| `54` | ExpectedUdivFirst | `0000FBB3` | `0000FBB3` |
| `58` | SentinelBefore | `A5C35A3C` | `A5C35A3C` |
| `5C` | Progress copy | `00000000` | `00000001` |
| `60` | Continuation copy | `00000000` | `00000000` |
| `64` | HandlerControl | `00000000` | `00000001` |
| `68` | HandlerIpsr | `00000000` | `00000006` |
| `6C` | HandlerSp | `00000000` | `20003000` |
| `70` | HandlerMsp | `00000000` | `20003000` |
| `74` | ExcReturn | `00000000` | `FFFFFFBC` |
| `78` | FramePsp | `00000000` | `20001FE0` |
| `7C` | Primask | `00000000` | `00000000` |
| `80` | Basepri | `00000000` | `00000000` |
| `84` | Faultmask | `00000000` | `00000000` |
| `88` | Shcsr | `00000000` | `00050008` |
| `8C` | Cfsr | `00000000` | `02000000` |
| `90` | Hfsr | `00000000` | `00000000` |
| `94` | FrameValid | `00000000` | `00000001` |
| `98` | StackedR0 | `00000000` | `00000003` |
| `9C` | StackedR1 | `00000000` | `00000000` |
| `A0` | StackedR2 | `00000000` | `20002000` |
| `A4` | StackedR3 | `00000000` | `0000002A` |
| `A8` | StackedR12 | `00000000` | `00000000` |
| `AC` | StackedLr | `00000000` | `08001FE1` |
| `B0` | StackedPc | `00000000` | `08000302` |
| `B4` | StackedXpsr | `00000000` | `29000000` |
| `B8` | UdivFirstReadback | `00000000` | `0000FBB3` |
| `BC` | SentinelAfter | `00000000` | `A5C35A3C` |
| `C0` | UnexpectedIpsr | `00000000` | `00000000` |
| `C4` | InitialCcr | `00000201` | `00000201` |
| `C8` | ConfiguredCcr | `00000201` | `00000211` |
| `CC` | HandlerCcr | `00000000` | `00000211` |
| `D0` | BuildMode | `00000000` | `00000001` |
| `D4` | PositiveQuotient copy | `00000000` | `00000006` |
| `D8` | ZeroQuotient copy | `00000000` | `A55A5AA5` |
| `DC` | FailureMarker copy | `00000000` | `00000000` |
| `E0` | ExpectedUdivSecond | `0000F6FC` | `0000F6FC` |
| `E4` | UdivSecondReadback | `00000000` | `0000F6FC` |

The external user words are respectively `1,2,6,0,0` and
`1,0,6,A55A5AA5,0` for disabled and enabled runs. The user
continuation value 2 means completed division; supervisor Status=2
only means configured control. Initial CCR=`00000201` in all four
runs. Configured and external CCR=`00000201` when disabled and
`00000211` when enabled; enabled HandlerCcr=`00000211`.
The configured CCR XOR difference is exactly `00000010`, and other
readable bits are unchanged. The independent SCB reads are
`SHCSR/CFSR/HFSR=00050000/00000000/00000000` when disabled and
`00050008/02000000/00000000` when enabled. The enabled CFSR has
only DIVBYZERO (bit 25); no stacking or unstacking fault bit appears.

Both positive controls produced quotient 6. With trapping disabled,
the zero-divisor quotient store changed from its initialized
`A55A5AA5` sentinel to zero, then the continuation marker became 2.
The terminal unprivileged Thread/PSP loop is inferred from the
inspected code, markers, and absence of faults; live CONTROL and SP
were not independently sampled in that loop. With trapping enabled,
the quotient storage remained `A55A5AA5`, continuation remained 0,
and UsageFault exception 6 was entered. The unchanged quotient
storage establishes no *publication* by the continuation path. R6 is
not in the basic hardware frame, and its post-fault value was not
measured.

Enabled HandlerControl=`1` retains the interrupted Thread-mode
`nPRIV` bit; Handler mode itself executes privileged. Entry SP and
MSP were `0x20003000`, EXC_RETURN=`0xFFFFFFBC`, and the basic PSP
frame began at `0x20001FE0`. Stacked R0/R1/R2=`3/0/20002000`
are the probe-supplied CONTROL/IPSR/SP observations. Stacked
R3=`42` and R12=`0` are the controlled operands. PC=`0x08000302`
identifies the faulting UDIV, not continuation at `0x08000306`.
Stacked LR=`0x08001FE1` is incidental. Stacked xPSR=`0x29000000`
satisfies `xPSR & 0x010003FF == 0x01000000`: Thumb state,
interrupted Thread mode, and no alignment padding. Its condition
flags are preserved as observed, without an acceptance requirement.
FrameValid=1 means structural prerequisites permitted inspection of
a candidate basic frame; it is not a general authenticity guarantee.
Both instruction halfwords are `FBB3 F6FC`; the four bytes at the
fault label are `B3 FB FC F6` in little-endian Flash. All eight
physical frame words matched the corresponding diagnostic copy in
each enabled run. Neither enabled execution returned from exception.

The protected sentinel at `0x200020E8` remained `A5C35A3C` in
all four independent reads. No unexpected-handler indication or
failure marker appeared. Independent CPUID remained `410FD214` in
every run. Each run meets its own prospective criteria, and the two
runs in each configuration are word-for-word repeatable.

## Independent macOS machine-code assessment

The reviewed ELFs have Reset instruction at `0x08000258`, Thumb ELF
entry `0x08000259`, `main` at `0x080002A4`, `fault_probe` at
`0x080002C4`, and UsageFault instruction/vector at
`0x08000328`/`0x08000329`. Both builds configure PSP and CONTROL,
execute ISB, then branch to the stackless probe, which checks
CONTROL=3, IPSR=0, SP=`0x20002000`. The complete probe regions,
including literal pools, agree between configurations.

Both execute `movs r3,#42` at `0x080002DE`, `mov.w r12,#7` at
`0x080002E0`, and the positive `UDIV r6,r3,r12` at `0x080002E4`.
After `cmp r6,#6`, the probe stores the quotient, reloads R3=42 at
`0x080002EE`, initializes R12=0 at `0x080002F0`, seeds R6=`0x5A`,
and stores progress 1 at `0x080002F8`. DSB/ISB precede the fault
label at `0x08000302`. The faulting UDIV has the same `FBB3 F6FC`
encoding as the positive control. Continuation starts exactly four
bytes later, at `0x08000306`: quotient store, DSB, marker 2 store,
DSB, then terminal self-branch at `0x08000314`. The failure path
starts at `0x08000316`. CCR is addressed at `0xE000ED14`, formed
from `0xE000ED98 - 0x84`; disabled startup clears bit 4 with BIC,
enabled startup sets it with ORR. Both read-modify-write paths have
DSB/ISB, readback, and verification of the requested bit and
preservation of other readable bits.

| Path or function | Reviewed software stack consumption |
|---|---:|
| DefaultPreInit | 8 bytes |
| Startup, either build | 84 bytes, restored before PSP handoff |
| Assembly probe; UsageFault entry wrapper | 0 bytes each |
| Disabled `record_usagefault`; `record_unexpected` | 8 bytes each |
| Enabled `record_usagefault` | 72 bytes |
| Enabled `indicate_success` call | Additional 8 bytes |

The enabled handler starts at `0x0800051C` with an 8-byte push and
64-byte allocation. Its only call, at `0x080007A6`, targets
`indicate_success` at `0x08000364`, which pushes 8 more bytes.
Maximum intended terminal handler MSP use is therefore 80 bytes;
minimum predicted MSP is `0x20002FB0`, leaving 1,968 bytes of the
2,048-byte `[0x20002800,0x20003000)` reserve. Startup's 84-byte
use is the maximum across the separate startup and handler phases.
The enabled basic frame uses 32 PSP bytes. The handler reads it with
four `LDRD` operations from LR holding `0x20002000`:

```text
08000618  ldrd r2, r6, [lr, #-32]
0800061C  movs r3, #1
0800061E  ldrd r0, r5, [lr, #-24]
08000622  ldrd r1, r8, [lr, #-16]
08000626  ldrd fp, r9, [lr, #-8]
```

The subsequent stores map all eight frame words to their documented
record offsets. No intended handler-path store overlaps the PSP
frame. The disabled build's UsageFault handler is a terminal failure
path with Status=`0x85`; no LED success function is linked there.
These are static findings from the supplied optimized code under the
controlled experiment assumptions, not runtime stack high-water
measurements or bounds for arbitrary faults and nested execution.

## Architectural interpretation and limits

The local reference catalogue and exact coordinates are in the
[procedure](../2026-11-27-cortex-m33-usagefault-divide-by-zero-procedure.md)
and [references](../references.md). ARM-ARM DDI0553B.z
§C2.4.260 (p. 972) defines UDIV T1 and its conditional zero-divisor
result; §B3.12 (p. 91), CCR.DIV_0_TRP in §D1.2.9 (p. 1474),
UFSR.DIVBYZERO in §D1.2.270 (p. 1897),
GenerateIntegerZeroDivide in §E2.1.186 (p. 2002), and
IntegerZeroDivideTrappingEnabled in §E2.1.234 (p. 2023)
explain the fault choice and CFSR bit. §B3.18 (p. 105,
rule RQLHB) selects the faulting PC for a synchronous fault;
§§B3.18–B3.19 and §D1.2.96 specify exception entry and return
encoding. PM0264 Rev. 8 §§4.3.9 and 4.3.12.3 (pp. 195, 202)
describe the CCR and syndrome. PM0264 p. 39 gives a general
next-instruction return-address description; ARM-ARM RQLHB is the
specific synchronous-fault rule corroborated here. The procedure
also records the PM0264 CCR read-only wording discrepancy and the
review of ARM-ERRATA and ES0553. No applicable erratum condition
was exercised or workaround introduced.

On this tested Cortex-M33 configuration, identical assembly UDIV
instructions and operands yielded zero and continuation with the
trap disabled, and a synchronous UsageFault with DIVBYZERO only
when enabled. The fault outcome therefore depends on privileged
architectural configuration as well as instruction and operands.
The two configurations are separately compiled firmware images,
not globally identical binaries; their probe machine code is
identical. Exact frame address, EXC_RETURN, SCB values, stack use,
and LED patterns are predictions for this controlled board setup.

For future PIL design consideration only: maintain relevant
processor fault-detection configuration as a PIL invariant;
consider enabling DIV_0_TRP explicitly rather than relying on
reset defaults; interpret syndromes in the context of processor
configuration; and revisit these choices alongside UNALIGN_TRP and
other fault controls after characterization. These are non-normative
recommendations, not production requirements adopted by this record.

The following limits remain:

- The intermediate option-byte display was not taken before the
  disabled deployment. Physical BOOT0 voltage, recovery-image
  availability at programming time, and full Flash-page erase impact
  were not independently documented.
- Neither complete programming-verification transcript was supplied.
  Subsequent independent uploads and byte-for-byte comparisons passed
  for both programmed ranges.
- Exact reset timestamps, instrumental LED timing, complete macOS
  check transcripts, and runtime stack high-water measurements were
  not supplied.
- Final physical frame equality does not rule out transient writes.
  Static intended-path stack accounting does not bound arbitrary
  nested exceptions or faulting execution.
- Stacked probe-supplied values are controlled experimental ground
  truth, not authenticated production origin evidence. The unchanged
  protected sentinel does not establish comprehensive isolation.
- The disabled terminal Thread/PSP state is inferred from reviewed
  code and memory evidence; live processor registers were not
  independently captured there.
- TrustZone was disabled. Signed SDIV, arbitrary arithmetic faults,
  floating-point division, nested exceptions, stacking faults,
  deliberate physical fault induction, exception return, and recovery
  were not exercised.
- This result does not establish FIPS 140-3 validation, production PIL
  assurance, or exhaustive processor verification.
