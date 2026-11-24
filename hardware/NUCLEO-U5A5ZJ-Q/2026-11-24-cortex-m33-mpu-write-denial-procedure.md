# Cortex-M33 MPU write-denial experiment

**Procedure date:** 2026-11-24  
**Status:** VM static verification only; Mac build and hardware run pending.

## Objective, authority, and assumptions

Test one CPU write permission: privileged code initializes and reads a
supervisor SRAM sentinel before and after MPU activation; unprivileged
Thread code writes and verifies user SRAM; one 32-bit store to the
supervisor sentinel must fault precisely without changing it. This is a
standalone PIL mechanism experiment, not production integration, FIPS
validation, a DMA isolation test, or execute-never evidence.

Use the NUCLEO-U5A5ZJ-Q Rev X board baseline: TZEN=0, RDP Level 0,
BOOT0 low, Flash boot at `0x08000000`, nominal 4 MHz reset clock,
no active floating-point context, and no enabled interrupt sources.
Keep option bytes and all non-MPU security controls unchanged. The
completed [read-denial evidence](records/2026-11-23-cortex-m33-mpu-read-denial.md)
and [privilege/SVC evidence](records/2026-11-23-cortex-m33-privilege-svc.md)
support the expected reset state and exception-return value on this
board. The project [architecture §§4.2–4.2.1](../../docs/assurance/architecture.md),
[scope](../../docs/assurance/project-scope.md),
[MCL FSM](../../docs/design/mcl-fsm.md), and
[interfaces](../../docs/design/module-interfaces.md) define the broader
PIL obligations; this experiment verifies only the stated access.
Follow the [documentation standard](../../docs/documentation-standard.md).

The memory assumptions follow RM0456 Rev 6 §§2.3.3 and 3.5. Armv8-M
Memory Model and Memory Protection User Guide §3 and PM0264 Rev 8
§§4.5.10–4.5.19 define RBAR/RLAR/MAIR and barriers. PM0264 §§2.4.7,
4.3.11–4.3.15 and Armv8-M Exception Model User Guide §3.1.2.1 define
exception entry and fault state. RBAR.AP is bits [2:1], XN bit 0;
MMFSR.DACCVIOL is bit 1. See the read-denial procedure for a noted
PM0264 diagram discrepancy. All SWD words below are little-endian;
bit 0 denotes the least-significant bit. Instruction encoding is shown
as two halfwords in execution order, with the first in bits [31:16].

## Memory, linker, and MPU configuration

| RNR | Inclusive window | Privileged | Unprivileged | RBAR | RLAR |
| --- | --- | --- | --- | --- | --- |
| 0 | Flash `0x08000000–0x08001FFF` | RX | RX | `0x08000006` | `0x08001FE1` |
| 1 | User SRAM `0x20001000–0x20001FFF` | RW, XN | RW, XN | `0x20001003` | `0x20001FE1` |
| 2 | Supervisor SRAM `0x20002000–0x20002FFF` | RW, XN | no access, XN | `0x20002001` | `0x20002FE1` |

The linker asserts 32-byte alignment, nonoverlap and supervisor data
below the MSP reserve. RBAR uses SH=00, AP=11/01/00 and XN=0/1/1.
RLAR has AttrIndx=0 and EN=1. MAIR0=`0x44` selects Normal
non-cacheable memory; MAIR1=0. Regions 3–7 are disabled. MPU_TYPE
must be `0x800`; MPU_CTRL starts at zero and becomes `0x5` (ENABLE
and PRIVDEFENA, HFNMIENA clear). Privileged background mapping permits
RCC/GPIO/SysTick access; it grants no unprivileged access to unmapped
addresses. DSB precedes configuration; DSB/ISB follow enable.
SHCSR.MEMFAULTENA (bit 16) is explicitly enabled and checked.

| Object | Linked address or range |
| --- | --- |
| User authorized word / progress | `0x20001000` / `0x20001004` |
| PSP reserve | `[0x20001008, 0x20002000)` |
| Protected diagnostics | `0x20002000`, 184 bytes |
| Protected sentinel | `0x200020B8`, four bytes |
| MSP reserve | `[0x20002800, 0x20003000)` |

## Execution and acceptance

1. Reset startup checks privileged Thread mode on MSP, configures PC7
   for LD1, initializes the sentinel to `0x53A1CE7D`, records its
   address and privileged read, and clears both user words. It records
   initial CFSR/HFSR before clearing documented W1C status bits.
2. After enabling and reading back the MPU, privileged startup reads
   the sentinel again and requires `0x53A1CE7D`. It then returns,
   allowing its MSP frame to unwind.
3. The assembly entry loads PSP=`0x20002000`, sets CONTROL=3 and
   executes ISB. The unprivileged probe samples CONTROL, IPSR and SP
   in r0–r2. It stores `0xA6074B19` to `0x20001000`, reads it back,
   compares the result, and sets progress=1 only on equality. A
   mismatch sets progress=3 and loops without accessing supervisor
   SRAM. The source value `0xC0DECAFE` is loaded into r3 and the
   linked sentinel address into r12.
4. Exactly one designated `STR.W r3,[r12]` attempts the protected
   write. If it completes, progress becomes 2 and the probe loops.
   It performs no second protected access.
5. MemManage exception 4 must enter privileged Handler mode on MSP.
   The assembly vector captures PSP, original LR/EXC_RETURN, active SP
   and MSP before Rust changes the stack. The handler records CFSR,
   HFSR and valid MMFAR before validation. It requires CONTROL=1,
   IPSR=4, both handler SP values `0x20003000`, EXC_RETURN
   `0xFFFFFFBC`, frame PSP `0x20001FE0`, CFSR exactly `0x82`
   (DACCVIOL and MMARVALID only), HFSR=0, MMFAR equal to the sentinel
   address, and armed phase 3. The exact PSP check places the basic
   32-byte frame within user SRAM and rejects an extended or padded
   frame before dereferencing it. HardFault escalation is failure.
6. The handler requires stacked PC at the named store, instruction
   halfwords `F8CC 3000`, Thumb state in xPSR, stacked r0=3,
   r1=0, r2=`0x20002000`, r3=`0xC0DECAFE`, and r12 equal to the
   sentinel address. It independently reads both user words and
   requires authorized value `0xA6074B19` and progress=1. The
   trusted instruction sequence establishes that progress follows
   the successful write/readback comparison. Finally, privileged
   Handler code reads the sentinel, records it, and requires the
   unchanged value `0x53A1CE7D` before phase=4/status=1.

The handler does not return, edit the stacked PC, or retry the store.
Status 1 drives a repeating **two-pulse** LD1 pattern with nominal
125 ms on/off steps and approximately 625 ms after each pair. This
is distinct from read denial's three pulses. Rust terminal failures
hold LD1 on after GPIO setup. An unprivileged positive-control failure
or unexpectedly completed protected store leaves LD1 off after startup;
progress 3 or 2 respectively identifies it. Early or nested faults
may prevent any diagnostic write or LED indication. LED behavior alone
is never acceptance evidence.

## Protected diagnostic record

Each offset is in bytes from `mpu_results` at `0x20002000`; each field
is one little-endian 32-bit word. The table gives acceptance values.
Values marked linked must be checked against the independently built
Mac ELF. A zero field may mean the corresponding phase was not reached.

| Offset | Field | Expected value |
| --- | --- | --- |
| `00` | Status | `1` |
| `04` | Phase | `4` |
| `08` | Startup CONTROL | `0` |
| `0C` | Startup IPSR | `0` |
| `10` | Startup MSP | within `[0x20002800,0x20003000]`, 8-byte aligned |
| `14` | MPU_TYPE | `0x800` |
| `18` | Initial MPU_CTRL | `0` |
| `1C` | Enabled MPU_CTRL | `5` |
| `20` | MAIR0 | `0x44` |
| `24` | MAIR1 | `0` |
| `28` | Flash RBAR | `0x08000006` |
| `2C` | Flash RLAR | `0x08001FE1` |
| `30` | User RBAR | `0x20001003` |
| `34` | User RLAR | `0x20001FE1` |
| `38` | Supervisor RBAR | `0x20002001` |
| `3C` | Supervisor RLAR | `0x20002FE1` |
| `40` | Unused enabled mask | `0` |
| `44` | Initial CFSR | normally `0` after reset |
| `48` | Initial HFSR | normally `0` after reset |
| `4C` | Armed SHCSR | bit 16 set, normally `0x00010000` |
| `50` | Sentinel address | linked `0x200020B8` |
| `54` | Privileged read before MPU | `0x53A1CE7D` |
| `58` | Privileged read after MPU | `0x53A1CE7D` |
| `5C` | Initial sentinel value | `0x53A1CE7D` |
| `60` | Intended prohibited write | `0xC0DECAFE` |
| `64` | Post-fault sentinel read | `0x53A1CE7D` |
| `68` | Authorized word in handler | `0xA6074B19` |
| `6C` | Progress marker in handler | `1` |
| `70` | Handler CONTROL | `1` |
| `74` | Handler IPSR | `4` |
| `78` | Handler SP at entry | `0x20003000` |
| `7C` | Handler MSP at entry | `0x20003000` |
| `80` | EXC_RETURN | `0xFFFFFFBC` |
| `84` | Frame PSP | `0x20001FE0` |
| `88` | CFSR | `0x82` |
| `8C` | HFSR | `0` |
| `90` | MMFAR | linked `0x200020B8` |
| `94` | Stacked PC | linked `0x080002E0` |
| `98` | Stacked xPSR | `(word & 0x010003FF) == 0x01000000` |
| `9C` | Stacked r0/Thread CONTROL | `3` |
| `A0` | Stacked r1/Thread IPSR | `0` |
| `A4` | Stacked r2/Thread SP | `0x20002000` |
| `A8` | Stacked r3/store source | `0xC0DECAFE` |
| `AC` | Stacked r12/destination | linked `0x200020B8` |
| `B0` | Store instruction halfwords | `0xF8CC3000` |
| `B4` | Unexpected exception IPSR | `0` |

Status 0 means incomplete. Failures: `0x81` startup state,
`0x82` privileged sentinel read, `0x83` MPU configuration,
`0x84` fault setup, `0x85` fault identity/status/frame eligibility,
`0x86` instruction/frame/positive-control mismatch, `0x87`
unexpected exception, `0x88` panic, and `0x89` changed sentinel.
Phases 0–4 mean reset, initialization, MPU enabled, probe armed,
and validated denial. The independent unprivileged words at
`0x20001000` and `0x20001004` must contain `0xA6074B19` and 1
for acceptance. They are corroborating evidence, not trusted proof.

## Build, image preparation, and static inspection

From `hardware/NUCLEO-U5A5ZJ-Q/mpu-write-denial`, use Rust 1.98.1,
`thumbv8m.main-none-eabi`, locked dependencies and release profile:

```sh
cargo +1.98.1 fmt --all --check
cargo +1.98.1 check --release --target thumbv8m.main-none-eabi --locked
cargo +1.98.1 clippy --release --target thumbv8m.main-none-eabi \
  --locked -- -D warnings
cargo +1.98.1 test --release --target thumbv8m.main-none-eabi --locked
cargo +1.98.1 test --doc --release --target thumbv8m.main-none-eabi \
  --locked
RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' \
  cargo +1.98.1 doc --release --target thumbv8m.main-none-eabi \
  --locked --no-deps
cargo +1.98.1 build --release --target thumbv8m.main-none-eabi --locked
python3 inspect-elf.py \
  target/thumbv8m.main-none-eabi/release/nucleo-u5a5zj-q-mpu-write-denial \
  --payload /tmp/mpu-write-denial-payload.bin \
  --program-image /tmp/mpu-write-denial-image.bin
arm-none-eabi-objdump -d \
  target/thumbv8m.main-none-eabi/release/nucleo-u5a5zj-q-mpu-write-denial
```

The inspection script checks ELF32/Arm entry, vector placement,
loadable Flash boundaries, allocated sections, the three 32-byte
aligned MPU windows, user and supervisor symbols, stack reserves,
record size, and designated STR encoding and probe operands. It
prints full ELF, unpadded Flash-payload and 16-byte-aligned programming
image SHA-256 and sizes. Independently reconstruct the loadable Flash
bytes from the ELF on the Mac and compare SHA-256 before programming.

### VM static-verification record, 2026-11-24

Rust 1.98.1 release build and `inspect-elf.py` checked the ELF32 Arm
entry `0x08000259`, MemManage vector `0x08000305`, the three MPU
windows and 32-byte alignment, protected record and sentinel placement,
stack reserves, absence of unintended allocated sections, and the
designated store at `0x080002E0`. GNU Arm disassembly shows
`F8CC 3000` / `str.w r3,[ip]`, with the source and destination
literals `0xC0DECAFE` and `0x200020B8`; `ip` denotes r12. The
preceding transition executes `msr PSP`, `msr CONTROL`, and `isb`.
The vector trampoline captures PSP, LR, active SP, and MSP before
branching to Rust. No runtime behavior is inferred from this check.

| VM artifact | Exact size | SHA-256 |
| --- | ---: | --- |
| Full ELF | 141,296 bytes | `98a71529a6fe6fab6fe0c6a4e386b0636ca3ccce906d8bc76ef8166fe547ee96` |
| Loadable Flash payload | 2,540 bytes | `ce8556bb06998a8666d91a2a89f8f8decca62f7d8d6057163d6682a2c46e9770` |
| 16-byte-aligned programming image | 2,544 bytes | `4659e01b51d00869c433c8512cc4cd22f88b0dc99c57b2f65f6050aa467e8939` |

`arm-none-eabi-objcopy -O binary` independently reconstructed the
2,540-byte payload from the ELF, byte-for-byte equal to the Python
inspector output. The final image appends four `0xFF` bytes. A Mac
artifact is not presumed identical.

The standalone firmware passed `cargo fmt --check`, embedded-target
`cargo check`, Clippy with warnings denied, rustdoc with broken links
denied, the release build, and the Rust source 80-character check.
`cargo test` could not build the bare-metal `test` crate on this
target; `cargo test --doc` found no library target. Neither result
constitutes a hardware test. The repository workspace checks and
host tests passed separately, including the supported
`thumbv6m-none-eabi` check for the portable crate.

## Pending hardware procedure and evidence

**No deployment is authorized by this VM procedure.** After separate
human deployment authorization and independent Mac build/inspection,
use the established ST-LINK recovery setup and commands below. Record
Mac tool versions, three hashes/sizes, disassembly, exact image used,
board identity/revision, UTC time, operator and reset method.

For a separately authorized operator, confirm a reviewed baseline
image is available, then program only the first 8 KiB Flash page
from the 16-byte-aligned image at `0x08000000`, with verification
enabled and automatic run disabled. Leave mass erase and option-byte
writes disabled. Disconnect and hardware reset. Observe LD1 for at
least ten seconds. Use SWD Hot Plug without reset or halt to read
the protected and user records. The CLI read lengths below are bytes:

```sh
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20002000 0xB8
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20001000 0x08
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0xE000ED28 0x08
```

Read-only SCB words cover CFSR, HFSR and MMFAR at `0xE000ED28`,
`0xE000ED2C`, and `0xE000ED34`; use separate reads for MMFAR:

```sh
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0xE000ED34 0x04
```

Repeat after a second hardware reset and compare complete records.
Acceptance requires matching protected and unprivileged records, the
validated two-pulse indication, all acceptance fields, and a Mac
Flash payload independently matching the programmed image. Record
any mismatch, HardFault, stalled pattern or connection failure as
non-acceptance, preserving raw data. To recover, disconnect, reset
under the established board procedure, connect under reset if Hot
Plug fails, reprogram the known-good blink image, hardware reset and
verify its baseline LD1 behavior. Read option bytes for comparison
only; do not modify them.

| Pending evidence | Observation |
| --- | --- |
| Mac build versions, ELF and image hashes/sizes | Pending |
| Board serial, revision, operator, UTC time | Pending |
| First reset LED, protected record, user words, SCB status | Pending |
| Second reset LED, protected record, user words, SCB status | Pending |
| Independent image comparison and acceptance decision | Pending |

## Limits

A static build cannot prove the MPU's behavior on silicon. A future
successful run would establish only this CPU Thread write-denial case
on the tested board and configuration. It would not establish DMA,
other bus-master isolation, arbitrary-code containment, stack-limit
enforcement, fault recovery, or FIPS validation. The experiment
intentionally stops with MemManage active; it does not test return to
Thread mode. A severe or nested fault may leave incomplete diagnostics.
