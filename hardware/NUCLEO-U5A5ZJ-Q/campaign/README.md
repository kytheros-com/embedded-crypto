# FLT-01-005–008 shared build and physical campaign

**Status:** Prospective characterization infrastructure; no board run.

The [ledger](../test-ledger.md) records lifecycle. Each child has its
own procedure and acceptance criteria:

| Child | Prospective procedure | Build configuration |
|---|---|---|
| FLT-01-005 | [Conditional unaligned load](../2026-11-27-cortex-m33-usagefault-conditional-unaligned-procedure.md) | `005-disabled`, `005-enabled` |
| FLT-01-006 | [Mandatory alignment](../2026-11-27-cortex-m33-usagefault-mandatory-alignment-procedure.md) | `006` |
| FLT-01-007 | [Invalid instruction state](../2026-11-27-cortex-m33-usagefault-invalid-state-procedure.md) | `007` |
| FLT-01-008 | [Repeated expected fault](../2026-11-27-cortex-m33-usagefault-repeated-expected-procedure.md) | `008` |

The first four configurations share
[`usagefault-alignment/`](../usagefault-alignment/) for MPU startup,
the 58-word supervisor record, GPIO indication, exception-frame checks,
and image extraction. Each assembly probe and exception acceptance
condition is independently selected and inspected. FLT-01-008 uses
[`usagefault-repeated-expected/`](../usagefault-repeated-expected/)
because its descriptor, two diagnostic slots, SVC orchestration and
exception return have different review requirements. Completed
experiments are not dependencies of the new source tree.

### Shared FLT-01-005–007 diagnostic word map

The 58-word supervisor record begins at `20002000`. Each offset below
is hexadecimal bytes from that base. Entries grouped on one row
appear in the listed order, one 32-bit word each. These definitions
are shared code; each procedure sets its own expected values.

| Offset(s) | Field(s), in word order |
|---|---|
| `00,04,08,0C,10` | Status, Phase, StartControl, StartIpsr, StartMsp |
| `14,18,1C,20,24` | MpuType, InitialMpuCtrl, MpuCtrl, Mair0, Mair1 |
| `28,2C,30,34,38,3C` | FlashRbar, FlashRlar, UserRbar, UserRlar, SupervisorRbar, SupervisorRlar |
| `40,44,48,4C` | UnusedEnabledMask, InitialCfsr, InitialHfsr, ArmedShcsr |
| `50,54,58` | ExpectedFaultAddress, ExpectedFirstHalfword, SentinelBefore |
| `5C,60,64,68` | ProgressMarker, ContinuationMarker, HandlerControl, HandlerIpsr |
| `6C,70,74,78` | HandlerSp, HandlerMsp, ExcReturn, FramePsp |
| `7C,80,84,88,8C,90,94` | Primask, Basepri, Faultmask, Shcsr, Cfsr, Hfsr, FrameValid |
| `98,9C,A0,A4,A8,AC,B0,B4` | StackedR0, StackedR1, StackedR2, StackedR3, StackedR12, StackedLr, StackedPc, StackedXpsr |
| `B8,BC,C0` | FirstReadback, SentinelAfter, UnexpectedIpsr |
| `C4,C8,CC,D0` | InitialCcr, ConfiguredCcr, HandlerCcr, BuildMode |
| `D4,D8,DC,E0,E4` | PositiveValue, FaultValue, FailureMarker, ExpectedSecondHalfword, SecondReadback |

For FLT-01-007, `ExpectedFaultAddress` names the invalid-state
**target**, while `FirstReadback` reads the initiating `BX` and
`SecondReadback` reads target `BX LR`. For FLT-01-005/006,
the two readback words are the two halfwords of the faulting load.
Startup clears all 58 words; disabled-control handler fields stay
zero. The protected sentinel follows the record at `200020E8`.
The five user words are progress, continuation, positive value,
fault value and failure marker at `20001000` through `20001010`.

## Build and static verification

Run on Linux, retain its manifest, then repeat on macOS with
`--reference-manifest` pointing to the transferred Linux manifest.
Use a fresh output directory. The script runs root-workspace checks
and checks every standalone configuration before building release
ELFs in distinct target directories. It saves full readelf and
disassembly output, inspector output, payloads, images, independent
GNU objcopy binaries, logs, source hashes, and artifact hashes.
It stops on a mismatched cross-host source, payload, or image hash.

```sh
python3 hardware/NUCLEO-U5A5ZJ-Q/campaign/build.py \
  --output /tmp/flt-01-005-008-linux

# On the independently reproduced macOS checkout:
python3 hardware/NUCLEO-U5A5ZJ-Q/campaign/build.py \
  --output /tmp/flt-01-005-008-macos \
  --reference-manifest /path/to/flt-01-005-008-linux/manifest.json
```

The macOS command is the consolidated build and static-verification
command. The manifest records the checked Git HEAD and working-tree
status. The inspector derives Flash payloads from ELF load segments,
requires 16-byte-aligned programming images padded only with `FF`,
checks vector routing, MPU windows, stack/record separation, and exact
probe instruction bytes. The build script compares each payload with
independent GNU objcopy output. The two FLT-01-005 builds require an
exact probe byte comparison. A human must review the complete
disassemblies, including reachable stack and frame-access paths.
The completed [Linux static-verification report](static-verification.md)
records the reference artifact digests, linked sites, negative tests,
and intended-path stack assessment for this implementation state.

## Physical runner, after human review

Specify the actual STM32CubeProgrammer executable path; Python does
not expand shell aliases. Inspect the command plan without board
access using `--dry-run`:

```sh
python3 hardware/NUCLEO-U5A5ZJ-Q/campaign/runner.py --campaign \
  --manifest /tmp/flt-01-005-008-macos/manifest.json \
  --reference-manifest /path/to/flt-01-005-008-linux/manifest.json \
  --programmer /path/to/STM32_Programmer_CLI \
  --output /path/to/new/flt-01-005-008-evidence --dry-run
```

After reviewing the exact images, commands, recovery image and Flash
page impact, remove `--dry-run` for the consolidated physical campaign
command. Programming requires a typed, image-specific authorization
phrase for **each** configuration. The runner does not write option
bytes, mass erase, unlock, recover, or operate the reset switch.
It requires the operator to press physical RESET and report LD1 for
each of two executions per configuration. FLT-01-008 has two faults
within each one-boot execution and must not be reset between them.

Before each deployment the runner verifies the image size/hash,
board identity, all 35 original option-byte fields, and CPUID using
read-only commands. It uses STM32CubeProgrammer `-w ... -v` only after
typed authorization. After every run it retains independent SWD
reads of supervisor data, user data, sentinel, CCR, SCB and CPUID,
plus a direct frame or descriptor read when required. It uploads the
full reviewed programming range and compares bytes before moving to
the next image. Any missing read, nonzero command exit, baseline
discrepancy, unexpected diagnostic, or Flash mismatch stops the
campaign without retry. Failed and completed raw files remain in
their distinct directories.

`--select 006,007`, `--resume 007:2`, and `--evidence-only` provide
selection, continuation and read-only collection. A resumed second
run requires the first accepted run file. Evidence-only mode verifies
the current Flash image by readback before collecting data. The
`--build-only` mode delegates to the consolidated build script.

The compact submission file is `summary.json` in the evidence output
directory. Every command has a complete raw `.txt` transcript with
UTC start/end times and exact arguments; each run has `decoded.json`.
Submit `summary.json`, all `decoded.json` files, the manifest, and
the raw transcript directory for independent review. Runner PASS is
**provisional**, not a hardware evidence record or engineering approval.
Each run also retains `operator-observation.json` with the times the
RESET confirmation and LED description were entered. Those times
are not measurements of the physical RESET edge or LED timing.

The board's [recovery policy](../README.md) and original
[option-byte record](../stm32u5a5zj-original-option-bytes.txt)
remain applicable. No TrustZone, RDP, GTZC, DMA, or debug-protection
change is authorized. Option-byte display establishes selection
policy, not BOOT0 pin voltage. Report unavailable recovery or
unverified Flash erase impact before programming.
