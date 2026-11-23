# Cortex-M33 MPU read-denial experiment

**Procedure date** 2026-11-23  
**Status** Draft for engineering review. VM build and static inspection
completed. Hardware execution and the acceptance decision are pending.

The objective is to show that privileged Thread code can read a word in
supervisor SRAM and that an unprivileged Thread can read its own working
word but receives a precise MemManage data-access fault on a single
32-bit load from the supervisor word. Acceptance requires independent
agreement between the LED pattern, protected SRAM diagnostics, and the
linked instruction. Write-denial and execute-never behavior are outside
this run. Write a separate dated evidence record after the hardware run.

## Configuration and references

Use NUCLEO-U5A5ZJ-Q product `NUU5A5ZJQ$AT2`, assembly
MB1549-U5A5ZJQ-C04, with STM32U5A5ZJ revision X, device ID `0x481`,
TZEN=0 and RDP Level 0. Start by hardware reset with BOOT0 low,
booting user Flash at `0x08000000`, without bank swap. Use the
existing PC7/LD1 routing, nominal 4 MHz reset MSIS clock, disabled
interrupt sources, reset exception priority and mask state, and no
active floating-point context. The initial MPU_TYPE and MPU_CTRL
expected from the board baseline are `0x00000800` and `0`.
Use the existing STLINK-V3E SWD configuration and compare option
bytes read-only before and after the run.
[Board inventory](README.md) · [Blink baseline](FIRMWARE.md) ·
[Privilege/SVC procedure](2026-11-23-cortex-m33-privilege-svc-procedure.md) ·
[Privilege/SVC board evidence](records/2026-11-23-cortex-m33-privilege-svc.md)

The project PIL contract defines the eventual isolation, exception and
memory-ownership requirements. This experiment tests one CPU read
permission on one board. The project architecture, scope and interfaces
define the broader boundary and the CMVP workbook remains the planning
authority.
[Architecture §§4.2–4.2.1](../../docs/assurance/architecture.md#42-platform-integration-layer-pil) ·
[Project scope](../../docs/assurance/project-scope.md) ·
[MCL FSM](../../docs/design/mcl-fsm.md) ·
[Module interfaces](../../docs/design/module-interfaces.md) ·
[Documentation standard](../../docs/documentation-standard.md)

The memory map follows RM0456 Rev 6 §§2.3.3 and 3.5 and the board
baseline. For MPU configuration and barriers, use Armv8-M Memory Model
and Memory Protection User Guide §3 and PM0264 Rev 8 §§4.5.10–4.5.19.
For exception identity, stacking and fault registers, use PM0264
§§2.4.7, 4.3.11–4.3.15 and the Armv8-M Exception Model User Guide
§3.1.2.1. PM0264's RBAR diagram caption and one MMFSR diagram
transcription can disagree with the register tables. The implementation
uses RBAR.AP[2:1], RBAR.XN[0], and MMFSR.DACCVIOL[1] as specified
in the tables and Arm guides. For the prior CONTROL read/write
discrepancy, see the completed SVC evidence.
[RM0456](https://www.st.com/resource/en/reference_manual/rm0456-stm32u5-series-armbased-32bit-mcus-stmicroelectronics.pdf) ·
[PM0264](https://www.st.com/resource/en/programming_manual/pm0264-stm32-cortexm33-mcus-and-mpus-programming-manual-stmicroelectronics.pdf) ·
[Arm MPU guide](https://documentation-service.arm.com/static/64c783b138511951cb7a2575) ·
[Arm exception guide](https://documentation-service.arm.com/static/64c7832738511951cb7a246e) ·
[STM32 revision errata](https://www.st.com/content/st_com/en/technical-documents/es0553-stm32u59xxx-and-stm32u5axxx-device-errata-stmicroelectronics.html)

## Memory map and expected protection

The MCU has 4 MiB of user Flash at `0x08000000` and 768 KiB
of SRAM1 at `0x20000000`. This image uses the first Flash page and
two 4 KiB SRAM1 windows. Ends in the table are inclusive. Bit 0 is
the least significant bit.

| RNR | Window | Inclusive address range | Privileged | Unprivileged | RBAR | RLAR |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | Flash | `0x08000000–0x08001FFF` | Read, execute | Read, execute | `0x08000006` | `0x08001FE1` |
| 1 | User SRAM | `0x20001000–0x20001FFF` | Read, write, XN | Read, write, XN | `0x20001003` | `0x20001FE1` |
| 2 | Supervisor SRAM | `0x20002000–0x20002FFF` | Read, write, XN | No access, XN | `0x20002001` | `0x20002FE1` |

The linker defines each window start and exclusive end on 32-byte
boundaries. RBAR holds the base plus SH=00, AP=11/01/00 respectively,
and the XN bit. RLAR encodes address bits [31:5] of the inclusive
end, AttrIndx=0, and EN=1. Its effective inclusive end has low five
bits set. MAIR0 is `0x00000044`, selecting Normal inner and outer
non-cacheable memory at index 0, with non-shareable Normal memory
in each RBAR. MAIR1 is zero. Regions 3–7 are disabled and their
enable bits are checked.

MPU_CTRL is `0x5` (ENABLE and PRIVDEFENA set, HFNMIENA clear).
PRIVDEFENA supplies the default map to privileged accesses outside
the explicit windows so the LD1 GPIO and RCC remain available with
only three regions. Explicit regions take priority over this fallback,
which supplies no permission to unprivileged requests. System Control
Space has architecture-defined privileged access rules. HFNMIENA=0
means the MPU does not enforce these mappings in HardFault, NMI,
or execution at the corresponding negative priority. Those vectors
lead to terminal failure handling.

| Item | Linked placement |
| --- | --- |
| User working word / untrusted progress | `0x20001000` / `0x20001004` |
| PSP reserve | `[0x20001008, 0x20002000)`, 4,088 bytes |
| Supervisor record `mpu_results` | `0x20002000`, 164 bytes |
| Supervisor sentinel `mpu_sentinel` | `0x200020A4`, four bytes, value `0x53A1CE7D` |
| MSP reserve | `[0x20002800, 0x20003000)`, 2,048 bytes |

## Procedure and fault acceptance

1. The runtime sets VTOR and zeroes supervisor `.bss`. Startup
   checks privileged Thread mode on MSP and sets LD1 solid.
   It initializes and reads the sentinel, then initializes the
   user working word to `0xA6074B19` and clears the progress word.
2. Confirm reset MPU state. Record initial CFSR and HFSR before
   clearing documented sticky bits using direct W1C writes.
   The CFSR clear mask is `0x031FBFBB`; HFSR clears observed
   FORCED and VECTTBL only. Require both registers zero before
   arming the probe. Explicitly enable MemManage via
   SHCSR.MEMFAULTENA, bit 16, and verify its readback.
3. Apply DSB before changing permissions, then DSB and ISB after
   MPU enable. Read back MPU_CTRL, MAIR0/1, all three RBAR/RLAR
   pairs, and disabled region enable bits. Read the sentinel again
   with the MPU enabled to prove privileged access under the
   tested mapping. Abort on any mismatch.
4. Rust startup returns and unwinds its ordinary MSP frame. The
   assembly entry writes the aligned PSP top, writes CONTROL=3,
   executes ISB, and branches to the probe. The probe samples
   CONTROL, IPSR and active SP, reads the allowed word, compares
   it to the expected constant and stores progress 1 in the
   authorized region. It executes exactly one aligned
   `LDR.W r6,[r12]` targeting `mpu_sentinel`. If the load
   completes, it stores progress 2 and spins. If the working
   word comparison fails, it stores progress 3 and spins
   without reaching the sentinel load. Neither spin uses MMIO.
5. Expected MemManage enters exception 4 privileged on MSP.
   The trampoline captures PSP, EXC_RETURN from LR, active SP,
   and MSP before the Rust prologue. The handler records CFSR,
   HFSR and MMFAR when valid and checks `CFSR=0x82` exactly
   (DACCVIOL and MMARVALID only), `HFSR=0`, MMFAR equal to
   the sentinel, phase 3, and the expected mode and stack.
   Use exact EXC_RETURN `0xFFFFFFBC` based on the completed
   experiment on this board. Reject unexpected frame format
   or stacking faults before dereferencing PSP.
6. Verify that the stacked PC is the linked faulting load and
   that Flash contains the expected halfwords `F8DC 6000`.
   Verify stacked xPSR Thumb state, basic unpadded Thread
   frame, and stacked r0/r1/r2/r3/r12 values from the probe.
   The denied load targets r6, leaving the positive-control
   read in stacked r3. The handler never consumes the denied
   value. On acceptance, publish phase 4 and status 1 and
   run the terminal triple-pulse indicator. No exception
   return, stacked PC edit, or retry is performed.

After startup the usual failure indication is solid LD1.
A fault before LED setup may leave LD1 dark. An unexpected
exception during indication stops the pattern and may leave LD1
at either level. The expected repeating pattern is three green
pulses, nominally 125 ms each, with 125 ms off between pulses
and about 1 s off after the third pulse. Observe repeated groups
after startup. The indicator polls SysTick, with TICKINT clear.
MemManage remains active while the indicator runs, so the test
cannot establish return safety or useful continued Thread operation.
More severe or nested faults may prevent the diagnostic routine
from running. HardFault or any other exception is a failure.

## Diagnostic record

Offsets are bytes from `mpu_results`. Each field is a 32-bit
little-endian word. Rust uses atomic stores for the SRAM
observations and a DSB before the terminal loop. Reset clears
the protected record through `.bss` initialization.

| Offset | Field | Expected successful-denial value |
| --- | --- | --- |
| `00` | Status | `1` |
| `04` | Phase | `4` |
| `08` | Startup CONTROL | `0` |
| `0C` | Startup IPSR | `0` |
| `10` | Startup MSP after prologue | `0x20002FA8` |
| `14` | MPU_TYPE | `0x00000800` |
| `18` | Initial MPU_CTRL | `0` |
| `1C` | Enabled MPU_CTRL readback | `5` |
| `20` | MAIR0 | `0x44` |
| `24` | MAIR1 | `0` |
| `28` | Flash RBAR | `0x08000006` |
| `2C` | Flash RLAR | `0x08001FE1` |
| `30` | User RBAR | `0x20001003` |
| `34` | User RLAR | `0x20001FE1` |
| `38` | Supervisor RBAR | `0x20002001` |
| `3C` | Supervisor RLAR | `0x20002FE1` |
| `40` | Enable mask for regions 3–7 | `0` |
| `44` | Initial CFSR before W1C | Normally `0` after reset |
| `48` | Initial HFSR before W1C | Normally `0` after reset |
| `4C` | Armed SHCSR | MEMFAULTENA bit 16 set, normally `0x00010000` |
| `50` | Expected sentinel address | `0x200020A4` |
| `54` | Privileged read before MPU enable | `0x53A1CE7D` |
| `58` | Privileged read after MPU enable | `0x53A1CE7D` |
| `5C` | Handler CONTROL | `1` |
| `60` | Handler IPSR | `4` |
| `64` | Handler active SP at entry | `0x20003000` |
| `68` | Handler MSP at entry | `0x20003000` |
| `6C` | EXC_RETURN | `0xFFFFFFBC` |
| `70` | Hardware frame PSP | `0x20001FE0` |
| `74` | CFSR including MMFSR | `0x00000082` |
| `78` | HFSR | `0` |
| `7C` | MMFAR if valid, zero otherwise | `0x200020A4` |
| `80` | Stacked PC | `0x080002DC` |
| `84` | Stacked xPSR | `(word & 0x010003FF) == 0x01000000` |
| `88` | Thread CONTROL, stacked r0 | `3` |
| `8C` | Thread IPSR read, stacked r1 | `0` |
| `90` | Thread active SP, stacked r2 | `0x20002000` |
| `94` | Authorized value, stacked r3 | `0xA6074B19` |
| `98` | Sentinel operand, stacked r12 | `0x200020A4` |
| `9C` | Fault instruction, first halfword in bits [31:16] | `0xF8DC6000` |
| `A0` | Unexpected IPSR | `0` |

Phases 0–4 mean reset/incomplete, privileged initialization,
MPU readback complete, armed, and validated denial respectively.
Status 0 means no terminal result. Nonzero failure statuses are
`0x81` startup mode/MSP, `0x82` privileged sentinel, `0x83`
MPU configuration, `0x84` fault setup, `0x85` exception
identity/flags/address or frame eligibility, `0x86` frame or
instruction validation, `0x87` unexpected vector, and `0x88` panic.
For `0x85`, frame-only fields may remain zero because reading the
frame first could use an invalid PSP. The user progress word is
corroborating evidence, outside supervisor protection. Progress 2
or 3 is a failure independently of any missing privileged result.

## Build, static inspection, and image creation

Run from the board directory. Rust 1.98.1 (`48a229cea`, LLVM
22.1.8), target `thumbv8m.main-none-eabi`, Python 3, and GNU Arm
binutils 2.42 were used on the VM. The board Cargo configuration
supplies `-Tlink.x`. The standalone workspace locks
`cortex-m` 0.7.10, `cortex-m-rt` 0.7.7 and `stm32u5` 0.16.0.
The VM build used base commit
`bb6a1edda5d6998123d4ab58f8c6714eb9c306b6` plus the experiment files.

```sh
git rev-parse HEAD
rustc +1.98.1 -Vv
cd mpu-read-denial
target=thumbv8m.main-none-eabi
cargo +1.98.1 fetch --locked --target "$target"
cargo +1.98.1 fmt --all --check
awk 'length($0)>80 {print FILENAME,FNR; failed=1}
  END {exit failed}' build.rs src/*
cargo +1.98.1 check --release --locked --offline --target "$target"
cargo +1.98.1 clippy --release --locked --offline --target "$target" \
  -- -D warnings
cargo +1.98.1 build --release --locked --offline --target "$target"
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" \
  cargo +1.98.1 doc --no-deps --locked --offline --target "$target"
cargo +1.98.1 tree --locked --offline --edges normal,build
elf=target/$target/release/nucleo-u5a5zj-q-mpu-read-denial
readelf -h -l -S -A -s -W "$elf"
readelf -x .vector_table "$elf"
arm-none-eabi-objdump -dC "$elf"
python3 inspect-elf.py "$elf" --payload target/payload.bin \
  --program-image target/program.bin
arm-none-eabi-objcopy -O binary -j .vector_table -j .text "$elf" \
  target/objcopy.bin
cmp target/payload.bin target/objcopy.bin
sha256sum "$elf" target/payload.bin target/program.bin
```

The script verifies the linked windows and independently decodes
their documented RBAR/RLAR words, checks vector and load bytes, and
extracts the programmed span from file-backed Flash PT_LOAD segments.
It rejects overlaps and fills gaps or final programming alignment
with erased-state `0xFF`. The VM checked that a one-byte mutation
of either the MemManage vector or the load opcode is rejected.
In this image two adjacent Flash segments generate the raw payload,
which matched the GNU objcopy output byte for byte. Four erased
bytes pad the programming image to the 16-byte granularity.

| ELF property | Observed on VM |
| --- | --- |
| Entry / Reset code | `0x08000259` / `0x08000258` |
| Vector table | `0x08000000`, 600 bytes, initial MSP `0x20003000` |
| MemManage vector | `0x08000010` contains `0x080002FD` |
| MemManage code / Rust validator | `0x080002FC` / `0x080004C4` |
| Faulting load / Thumb halfwords | `0x080002DC` / `F8DC 6000` |
| `.text` | `0x08000258`, 1,876 bytes |
| `.bss` / `.mpu_user` | 168 bytes at `0x20002000` / 4,096 bytes at `0x20001000`, both NOBITS |
| Other allocatable sections | `.rodata`, `.data`, `.uninit` and `.gnu.sgstubs` have size zero |
| ELF SHA-256, 141,288 bytes | `f53d7cc5ba86823d998b688e978228e46312c651e3e75bde5bc3880c74e87f43` |
| Flash-payload SHA-256, 2,476 bytes | `931ec1170de2bb21431c6df28fd866bf9eb72c6a125599daf9cf63eaadb23bb3` |
| Programming-image SHA-256, 2,480 bytes | `db78782d0f08e769a19e7a60b396ef0c98d59f8fd612ca34185a176b52562ce8` |

The disassembly shows MPU_CTRL enable at `0x0800085A` followed
by DSB/ISB at `0x0800085E/0x08000862`. The startup Rust frame
uses 88 bytes on MSP and returns before `MSR PSP` at
`0x080002AA`, `MSR CONTROL` at `0x080002B0`, and ISB at
`0x080002B4`. The assembly probe consumes no software stack.
Hardware is expected to stack 32 bytes on PSP. The MemManage
trampoline passes four AAPCS register arguments (PSP, original
LR, active SP, and MSP) and tail-branches to Rust on MSP.
The validator and its terminal indicator use eight bytes of MSP
each in the inspected release. Neither path returns.

The runtime's exception attribute uses a Rust trampoline with no
MemManage frame argument and no contract to preserve entry LR
until an application reads it. The code uses the runtime's supported
linker aliases for MemoryManagement, HardFault and DefaultHandler.
The stack-switch shim avoids leaving a live Rust MSP frame.
The probe assembly pins one word load and its label and preserves
the sampled values in architecturally stacked registers. Rust
uses one small inline `MRS IPSR` because the dependency has no
IPSR register reader. The assembly has no allocator and no
Rust mutable static references.
[cortex-m register APIs](https://docs.rs/cortex-m/0.7.10/cortex_m/register/) ·
[cortex-m-rt linker aliases](https://docs.rs/crate/cortex-m-rt/0.7.7/source/link.x.in)

Application unsafe uses comprise the unique exported static symbols
and assembly/linker symbol declarations, IPSR read, exclusive
peripheral acquisition at startup and at success, volatile
initialization of the assembly-owned user words, MPU and W1C
volatile writes, read-only access to SCB on fault paths, and volatile
frame and opcode reads after validation. The runtime and PAC also
use their own low-level MMIO and startup code. SWD reads and
handler code remain outside the threat model. The experiment
neither configures nor tests DMA, GTZC, debug access controls,
TrustZone, watchdogs, option bytes, or Flash security. The linker
reserves the stacks without programming stack-limit registers.
It does not establish containment for arbitrary code, a
production authorization policy, recovery, or zeroization.

## Reviewed Mac run and recovery

1. Transfer the approved `target/program.bin` and its ELF.
   Verify both SHA-256 values on the Mac with `shasum -a 256`.
   Confirm a known-good blink or privilege/SVC image is available
   before overwriting the current application.
2. Via STM32CubeProgrammer 2.24.0 and the established SWD wiring,
   confirm board identity and read-only option-byte parity. Open
   the 2,480-byte programming binary and set its start address
   to `0x08000000`. Its span is within the first 8 KiB page.
   Erase/program that page with verification enabled and automatic
   run disabled. Leave mass erase and option-byte writes disabled.
   Record the programmed digest and verification result.
3. Disconnect, hardware reset, and observe LD1 for at least ten
   seconds. Connect Hot Plug without resetting or halting and
   capture the protected record and both user words as below.
   Check the offsets against the matching ELF. The CLI sizes are
   bytes. Repeat after a second hardware reset.
4. Accept only when both runs show the triple-pulse pattern,
   status 1, phase 4, all configured MPU fields, the expected
   precise fault at the designated load and sentinel address,
   expected stack/frame values, and zero unexpected IPSR.
   Capture and explain any stale startup fault bits. Record
   results as pending or failed if the expected data are absent.
   [STM32CubeProgrammer documentation](https://dev.st.com/stm32cube-docs/prog/) ·
   [ST SWD CLI and memory reads](https://dev.st.com/stm32cube-docs/prog/2.23.0/en/docs/markup/CubeProg_Command_Lines.html)

```sh
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20002000 0xA4
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20001000 0x08
```

On unexpected results, preserve the raw dump and LED observations
before reset. If Hot Plug fails, use the previously demonstrated
hardware Connect Under Reset path with the existing SWD wiring
and record any available state. Restore a reviewed baseline ELF
using its established page programming and verification procedure.
Hardware reset, verify the baseline LED behavior, and compare
option bytes read-only. Recovery resets the volatile MPU state.
[Baseline recovery procedure](FIRMWARE.md#proposed-mac-deployment-after-review)

| Evidence field | Actual observation |
| --- | --- |
| Operator, UTC time, board/revision, reset method | |
| Source revision and working tree, tool versions | |
| ELF, payload and programming-image digests, Flash verification | |
| Read-only option-byte comparisons | |
| LED pattern and approximate timing, both runs | |
| Raw protected and user-region SWD reads, both runs | |
| Decoded MPU, fault, frame and instruction evidence, both runs | |
| Unexpected behavior, recovery actions and evidence | |
| Run 1, run 2, overall pass/fail and reviewer disposition | |
