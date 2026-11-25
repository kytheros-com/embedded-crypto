# FLT-01-001 — Cortex-M33 undefined-instruction UsageFault evidence

**Hardware execution date:** 2026-11-25

**Result:** PASS — two identical hardware-reset executions

**Scope:** Standalone Platform Integration Layer (PIL) mechanism experiment

## 1. Objective and evidence basis

Determine whether one labeled `UDF #0x42` in unprivileged Thread mode
raises UsageFault and whether the privileged handler can record the
classification and interrupted context. The
[procedure](../2026-11-25-cortex-m33-usagefault-undefined-instruction-procedure.md)
defines the acceptance criteria and architectural references. The
[standalone source](../usagefault-undefined-instruction/) defines the
record layout and handler checks. The
[hardware ledger](../test-ledger.md) indexes this result. The
[architecture](../../../docs/assurance/architecture.md) §4.2 assigns
fault entry to the PIL; the [MCL FSM](../../../docs/design/mcl-fsm.md)
§7 governs future module fault policy, which this experiment does not
implement.

The human operator supplied the four raw SWD dumps below and reported
that both physical RESET executions produced the same dumps and the
repeating five-flash green LD1 pattern. The analysis here decodes the
supplied observations; the author of this record did not operate the
board or independently witness the executions.
The later source, build, and physical Flash provenance was collected
at `2026-11-25T22:11:19Z`. This is the evidence collection time;
individual hardware execution times were not supplied. The LD1
observations were visual, without external timing measurement.

## 2. Board, configuration, and image provenance

| Item | Operator-reported value |
|---|---|
| Board | NUCLEO-U5A5ZJ-Q |
| MCU and core | STM32U5A5ZJ, Cortex-M33 |
| Device ID; silicon revision | `0x481`; X |
| ST-LINK serial number | `003800353335511835383531` |
| ST-LINK firmware | V3J10M3 |
| STM32CubeProgrammer | 2.24.0 |
| Reported voltage | 3.27 V |
| SWD frequency and diagnostic mode | 1000 kHz; Hot Plug |
| Bootloader; NVM capacity | `0x92`; 4 MiB |
| Execution method | Physical RESET button, twice |
| TrustZone/security baseline | Existing `TZEN=0` option-byte configuration; no change reported |

The established three-region MPU map was captured in the diagnostic
record: executable Flash `0x08000000–0x08001FFF`, user SRAM
`0x20001000–0x20001FFF` RW/XN, and supervisor SRAM
`0x20002000–0x20002FFF` privileged-only RW/XN. The implementation
uses separate user PSP and supervisor MSP storage, no active FP
context, and no enabled ordinary interrupt source. The record's
`MPU_TYPE=0x00000800`, `MPU_CTRL=0x00000005`, `MAIR0=0x44`,
`MAIR1=0`, six RBAR/RLAR values, and unused-region mask match the
procedure. An option-byte readback for these two executions was not
supplied; `TZEN=0` is the established experimental baseline and
operator-reported unchanged configuration.

### Committed source and macOS environment

At evidence collection, the operator reported branch
`hardware/NUCLEO-U5A5ZJ-Q`, clean `git status --short`, and `HEAD`
`0f6f122dc9c64161a10e05b1af2656f443c64f94`, matching its
remote-tracking branch. The exact commit subject is
`"Undefined Instruction fault testing"`; author and commit timestamps are both
`2026-11-25T21:36:05Z`. These commit times are not the hardware
execution times. The seven macOS SHA-256 values below were verified
against the named commit's blobs and the current tracked files:

| Committed source file | SHA-256 |
|---|---|
| `src/main.rs` | `f1b06afdd6b4f5358d2aae1fa994067949a7ae4d606bc242c52afe05eacac72e` |
| `src/transition.s` | `fdeeaf083f01291d071669b64e9e7f8494ab6a3c914954094c594172d1743362` |
| `inspect-elf.py` | `8d1c10276696128ae92c40709ea2110e9e12f0e5f7e3883bdf78c195c13278b7` |
| `memory.x` | `2f8e89e9efc0b6e9255535fce0dd0d305041170e2557acaf7f690afa83ed87a5` |
| `build.rs` | `e03c4bf416547fd9c44ee88628f3b5c8010a3e18a9930c8500f9aeac65f12804` |
| `Cargo.toml` | `9e645e891e98641cfc2170dd02047ca3336ac34abf709bf04526f18162d1c601` |
| `Cargo.lock` | `e15069e0c1e39f7078583e05c9aec48dee80739f49bfc707f0b2bae091b793cc` |

The first three hashes also match the original procedure's static
verification record. The operator supplied these environment values:

| Property | macOS observation |
|---|---|
| Operating system; build | macOS 27.2.1; 26B131 |
| Host architecture; Rust host target | arm64; `aarch64-apple-darwin` |
| Firmware target | `thumbv8m.main-none-eabi` |
| Rust | 1.98.1; compiler commit `48a229ceaefd4985c50990b14116b6d856af0985` (2026-09-01) |
| LLVM | 22.1.8 |
| Cargo | 1.98.1; commit `797e8a9bc` (2026-08-05) |
| GNU Binutils objcopy | 2.47.20260726 |
| Python | 3.14.7 |

The operator independently built and inspected the firmware on
macOS before deployment. The complete command transcript was not
supplied, so no additional command executions are inferred.

### Build artifacts and physical Flash read-back

| Artifact | Linux reference | Independent macOS / physical result |
|---|---|---|
| ELF | 141,448 bytes; `fbdb9a8f5be5bceb0da0fa946f6eb47e4a0e089502bc59b01b1760761288188d` | 141,460 bytes; `71b0cde2a1e0c3e3f95f399290e18ba0b1342820bdfd822276d5f26dcad1572b` |
| Flash payload | 2,736 bytes; `c8dbcc5ce07e86504f522cde0463f45abb7b9b82a3cf55c24cb50375b603f34d` | 2,736 bytes; same SHA-256 |
| Programming image | 2,736 bytes; same SHA-256 | 2,736 bytes; same SHA-256 |
| Physical Flash read-back | Not a Linux artifact | 2,736 bytes; same SHA-256 |

The Linux values are from the original static procedure. The
operator independently reported the macOS values and successful
byte-for-byte comparisons of payload against programming image and
physical read-back against programming image. The complete ELF files
differ in size and digest. The Flash payloads and images agree by
SHA-256; no complete-ELF reproducibility or cause for the ELF
difference is claimed. `2736 = 171 × 16`, so the programming image
needed no padding bytes.

After the executions, the operator uploaded the programmed region:

```sh
stm32cli -c port=SWD mode=HOTPLUG freq=1000 \
  --upload 0x08000000 2736 /tmp/flt-01-001-readback.bin
```

The upload returned 2,736 bytes. The operator's SHA-256 comparison
and `cmp` against the programming image both passed. This establishes
agreement for the read-back region at evidence collection time, not
for the rest of Flash or for compiler correctness. The independent
ELF inspection in the procedure located the UDF at `0x080002EC`
with encoding `0xDE42` and the UsageFault vector at `0x08000301`.

## 3. Raw hardware observations

**Execution 1 and execution 2:** each followed a separate physical
RESET. Each showed the repeating five-flash green LD1 indication.
All four dumps below were **identical in both executions**: 49
protected diagnostic words, one independent sentinel word, two user
marker words, and three independent SCB words — **55 words / 220
bytes** total per execution. The dumps are reproduced exactly as
supplied. No differences were reported.

### Protected diagnostic record — 196 bytes

```text
0x20002000 : 00000001 00000004 00000000 00000000
0x20002010 : 20003000 00000800 00000000 00000005
0x20002020 : 00000044 00000000 08000006 08001FE1
0x20002030 : 20001003 20001FE1 20002001 20002FE1
0x20002040 : 00000000 00000000 00000000 00050000
0x20002050 : 080002EC 0000DE42 A5C35A3C 00000001
0x20002060 : 00000000 00000001 00000006 20003000
0x20002070 : 20003000 FFFFFFBC 20001FE0 00000000
0x20002080 : 00000000 00000000 00050008 00010000
0x20002090 : 00000000 00000001 00000003 00000000
0x200020A0 : 20002000 20002FE1 08000006 08001FE1
0x200020B0 : 080002EC 29000000 0000DE42 A5C35A3C
0x200020C0 : 00000000
```

### Independent protected sentinel — 4 bytes

```text
0x200020C4 : A5C35A3C
```

### Independent user progress markers — 8 bytes

```text
0x20001000 : 00000001 00000000
```

### Independent SCB readings — 12 bytes

```text
0xE000ED24 : 00050008 00010000 00000000
```

## 4. Decode of all 49 diagnostic fields

Offsets are bytes from `0x20002000`; values are raw 32-bit
little-endian words. Fields marked *incidental* are retained without
adding an acceptance criterion. The final `Count` enum variant is a
length marker, not a recorded word.

| Offset | Field | Raw value | Decode or role |
|---|---|---|---|
| `00` | Status | `00000001` | Internal checks passed. |
| `04` | Phase | `00000004` | Validated fault path reached. |
| `08` | StartControl | `00000000` | Privileged Thread startup, MSP selected. |
| `0C` | StartIpsr | `00000000` | Startup in Thread mode. |
| `10` | StartMsp | `20003000` | Initial MSP at supervisor stack top. |
| `14` | MpuType | `00000800` | Eight MPU regions reported. |
| `18` | InitialMpuCtrl | `00000000` | MPU initially disabled. |
| `1C` | MpuCtrl | `00000005` | MPU enabled with privileged background map. |
| `20` | Mair0 | `00000044` | Configured memory attribute. |
| `24` | Mair1 | `00000000` | No second attribute used. |
| `28` | FlashRbar | `08000006` | Flash RX region base/permissions. |
| `2C` | FlashRlar | `08001FE1` | Flash region through `08001FFF`. |
| `30` | UserRbar | `20001003` | User SRAM RW/XN base/permissions. |
| `34` | UserRlar | `20001FE1` | User region through `20001FFF`. |
| `38` | SupervisorRbar | `20002001` | Supervisor-only SRAM base/permissions. |
| `3C` | SupervisorRlar | `20002FE1` | Supervisor region through `20002FFF`. |
| `40` | UnusedEnabledMask | `00000000` | Regions 3–7 disabled. |
| `44` | InitialCfsr | `00000000` | No initial configurable fault bits. |
| `48` | InitialHfsr | `00000000` | No initial HardFault bits. |
| `4C` | ArmedShcsr | `00050000` | MemManage and UsageFault enabled. |
| `50` | ExpectedUdfAddress | `080002EC` | Linked UDF address from firmware. |
| `54` | ExpectedUdfEncoding | `0000DE42` | Expected 16-bit UDF encoding. |
| `58` | SentinelBefore | `A5C35A3C` | Synthetic protected value before fault. |
| `5C` | ProgressMarker | `00000001` | Positive control completed. |
| `60` | ContinuationMarker | `00000000` | No unexpected continuation recorded. |
| `64` | HandlerControl | `00000001` | Handler-mode CONTROL under tested configuration. |
| `68` | HandlerIpsr | `00000006` | UsageFault exception number 6. |
| `6C` | HandlerSp | `20003000` | Active handler SP at MSP top. |
| `70` | HandlerMsp | `20003000` | Handler used separate supervisor MSP. |
| `74` | ExcReturn | `FFFFFFBC` | Basic PSP frame, interrupted Thread mode. |
| `78` | FramePsp | `20001FE0` | Basic 32-byte frame starts here. |
| `7C` | Primask | `00000000` | No PRIMASK inhibition recorded. |
| `80` | Basepri | `00000000` | No BASEPRI threshold recorded. |
| `84` | Faultmask | `00000000` | No FAULTMASK inhibition recorded. |
| `88` | Shcsr | `00050008` | Enabled bits plus active UsageFault bit 3. |
| `8C` | Cfsr | `00010000` | `UNDEFINSTR` bit 16 alone. |
| `90` | Hfsr | `00000000` | No HardFault escalation recorded. |
| `94` | FrameValid | `00000001` | Handler's frame prerequisites passed. |
| `98` | StackedR0 | `00000003` | Pre-fault CONTROL=3: unprivileged Thread/PSP. |
| `9C` | StackedR1 | `00000000` | Pre-fault IPSR=0: Thread mode. |
| `A0` | StackedR2 | `20002000` | Pre-fault active PSP at its top. |
| `A4` | StackedR3 | `20002FE1` | Incidental; retained raw. |
| `A8` | StackedR12 | `08000006` | Incidental; retained raw. |
| `AC` | StackedLr | `08001FE1` | Incidental; retained raw. |
| `B0` | StackedPc | `080002EC` | Address of labeled faulting UDF. |
| `B4` | StackedXpsr | `29000000` | Thumb, C, and Q bits set; IPSR bits [8:0] and stack-align bit 9 clear. |
| `B8` | UdfEncodingReadback | `0000DE42` | Handler-read instruction halfword. |
| `BC` | SentinelAfter | `A5C35A3C` | Protected sentinel unchanged. |
| `C0` | UnexpectedIpsr | `00000000` | No unexpected exception recorded. |

The eight words at offsets `98–B4` form the basic exception frame
at `0x20001FE0`: r0, r1, r2, r3, r12, LR, PC, and xPSR, in that
order. The frame spans `[0x20001FE0,0x20002000)`. Its SP is
8-byte aligned and inside the linked PSP reserve. The xPSR value
`0x29000000` has T=1, interrupted IPSR=0, and stack-alignment
indicator bit 9=0. C and Q are also set; these flags were not
acceptance criteria.

The independently read sentinel matches both record copies. The
independent user marker read matches the record. Independent SCB
`SHCSR/CFSR/HFSR` words at `0xE000ED24` exactly match the handler
snapshot at offsets `88/8C/90`.

## 5. Result against acceptance criteria

| Criterion | Observation | Result |
|---|---|---|
| Two separate hardware-reset executions | All 55 words identical; five-flash LD1 repeated | PASS |
| Positive control, no continuation | Markers 1 and 0; status 1, phase 4 | PASS |
| Configured fault handler | Armed SHCSR=`00050000`; active=`00050008`; IPSR=6 | PASS |
| Fault classification | CFSR=`00010000` (`UNDEFINSTR` only); HFSR=0 | PASS |
| Privileged handler and original entry | CONTROL=1; SP=MSP=`20003000`; EXC_RETURN=`FFFFFFBC` | PASS |
| Basic PSP exception frame | FrameValid=1; PSP=`20001FE0`; stacked pre-fault PSP=`20002000` | PASS |
| Instruction attribution | Stacked PC=`080002EC` matches independently inspected UDF address; halfword=`DE42` | PASS |
| Interrupted state | Stacked xPSR T=1, IPSR=0, no alignment padding indicated | PASS |
| Local sentinel control | Before, after, and separate SWD read all `A5C35A3C` | PASS |
| Independent diagnostic corroboration | External SHCSR/CFSR/HFSR match stored words | PASS |

The firmware's internal SHCSR check uses a mask. The observed
`SHCSR=0x00050008` exactly meets the stricter documented hardware
acceptance criterion.

| Memory region | Bytes | Matching words across executions |
|---|---:|---:|
| Protected diagnostic record | 196 | 49/49 |
| Protected sentinel | 4 | 1/1 |
| User progress markers | 8 | 2/2 |
| SCB fault registers | 12 | 3/3 |
| Total | 220 | 55/55 |

**Disposition: PASS — two successful, identical hardware-reset
executions, independently reproduced Flash payload, and verified
physical Flash read-back.** Both runs met the documented hardware
acceptance criteria under this configuration. Five-pulse LD1 is a
corroborating indication, not the sole basis for the result. The
identical dumps establish repeatability for these recorded words and
observations only.

## 6. Comparison and limitations

In [HW-005](2026-11-25-cortex-m33-mpu-execute-never.md), a Flash
`BX r12` attempted a prohibited SRAM instruction fetch. The stacked
PC was the prohibited SRAM fetch target `0x20001008`, while the
initiating branch was in Flash at `0x080002FC`. Here, `UNDEFINSTR`
identifies a fault in the UDF instruction itself, and the stacked PC
is that UDF's Flash address `0x080002EC`. A stacked PC must therefore
be interpreted with fault classification and architectural context.
The combination of status, exception-entry context, and independently
known instruction location permits attribution of this specific
synchronous fault. These two experiments do not establish a universal
fault-attribution algorithm.

This result covers one deliberately induced undefined instruction in
unprivileged Thread mode, on the reported revision X board and
`TZEN=0` baseline. It does not establish general execution-domain
attribution, TrustZone-aware routing, comprehensive fault containment,
safe exception return or recovery, privileged Thread-mode behavior,
nested exception or stacking-failure behavior, resistance to arbitrary
hardware corruption or physical fault induction, compiler correctness,
complete ELF reproducibility, or FIPS 140-3 compliance or validation.
The unchanged sentinel is a local control, not proof of comprehensive
memory integrity. The terminal handler did not attempt exception return.

The complete historical macOS build/programming transcript, individual
hardware execution timestamps, operator identifier, and contemporaneous
option-byte readback were not supplied. The read-back covers only the
2,736-byte programmed region at evidence collection time. These limits
prevent full independent reconstruction of the deployed session. Human
engineering review remains required before treating this record as
approved project evidence.
