# Cortex-M33 privilege and SVCall experiment

**Procedure date** 2026-11-23  
**Status** Draft for engineering review. VM build and static inspection complete.
Target execution pending.

Demonstrate reset-time privileged Thread mode on MSP, a one-way software
transition to unprivileged Thread mode on PSP, the ignored direct attempt
to clear `CONTROL.nPRIV`, and a synchronous SVCall that permits this
cooperative test to resume privileged Thread execution on the same PSP.
The expected values derive from the processor architecture. This procedure
contains fields for a future run. Preserve completed evidence in a separate
dated record under `records/`.

## Initial conditions

| Item | Required configuration |
| --- | --- |
| Board | NUCLEO-U5A5ZJ-Q, product `NUU5A5ZJQ$AT2`, MB1549-U5A5ZJQ-C04 |
| MCU | STM32U5A5ZJ, device ID `0x481`, revision X, Cortex-M33 |
| Boot | Hardware reset, BOOT0 low, boot from user Flash at `0x08000000`, no bank swap |
| Security | `TZEN=0`, RDP Level 0, effective MPU disabled, eight regions available |
| Memory | 4 MiB user Flash `0x08000000–0x083FFFFF`, 768 KiB SRAM1 `0x20000000–0x200BFFFF` |
| LED and clock | Green user LD1, active-high PC7, SB21 closed and SB23 open, nominal reset MSIS 4 MHz |
| Debug | Existing STLINK-V3E SWD setup, STM32CubeProgrammer 2.24.0 on Mac |

Check the [hardware inventory](README.md), [blink firmware](FIRMWARE.md),
and [prior deployment record](records/2026-11-21-initial-firmware-deployment.md)
for wiring, option-byte baseline, and recovery. Check option bytes read-only
and use a hardware reset for each run. The firmware assumes reset interrupt
enables and masks, the boot path above, and no active floating-point context.
It uses the soft-float target. SysTick is polled with TICKINT left clear.
The experiment leaves MPU, GTZC, DMA, option bytes, Flash protection, debug,
watchdogs, and boot configuration untouched.

## Processor sequence

Bit numbers start at the least significant bit. `CONTROL.nPRIV` is bit 0
and `CONTROL.SPSEL` is bit 1. The unprivileged probe uses `MRS CONTROL`
and `MRS IPSR` and reads active R13 directly. The zero IPSR read is
consistent with Thread mode, which the handler independently checks in
the stacked xPSR and EXC_RETURN. In Handler mode, IPSR is the active
exception number and the processor executes privileged on MSP even
when `CONTROL.nPRIV=1` still describes the interrupted Thread's privilege.

1. The runtime initializes `.bss` and VTOR, then calls assembly `main`.
   The entry checks `CONTROL=0`, `IPSR=0`, and active SP at the linked
   initial MSP. An ordinary AAPCS call sets LD1 solid and returns on MSP.
   Assembly writes the exclusive top of the aligned 2 KiB PSP storage
   to PSP, writes `CONTROL=3`, executes ISB, and branches without a
   live Rust frame. PSP and the descending MSP occupy disjoint space.
2. The unprivileged probe records `CONTROL=3`, `IPSR=0`, and active SP
   equal to PSP top. It uses no GPIO and consumes no software stack.
3. The probe writes `CONTROL=2` with MSR, executes ISB, and expects
   `CONTROL` to remain 3. The unprivileged write is architecturally
   ignored. No fault is expected.
4. `svc #0x35` stacks the basic eight-word frame on PSP and enters
   exception 11 on MSP. The handler records original LR and checks
   IPSR, CONTROL, active SP against MSP and the original MSP top,
   EXC_RETURN common fields, and phase status. Before reading the
   frame, it checks PSP against the exact expected aligned frame base
   (PSP top minus 32 bytes). It checks stacked IPSR, Thumb state and
   alignment bit in xPSR, then matches the stacked return PC against
   the single linked call site before reading the preceding halfword.
   Both the call site and opcode `0xDF35` must match.
5. The handler clears `CONTROL.nPRIV` while privileged and executes
   ISB. It returns with `BX LR` using the original EXC_RETURN. The
   processor unstacks from PSP and restores Thread mode and SPSEL
   from EXC_RETURN. The frame does not contain CONTROL or nPRIV.
   The saved PC resumes immediately after SVC, with `CONTROL=2`,
   `IPSR=0`, and active SP back at PSP top. The privileged Rust LED
   loop then runs on PSP.

Armv8-M DDI0553A.i §B3.4 explicitly allows unprivileged CONTROL reads
and specifies that unprivileged writes are ignored. PM0264 Rev 8 Table 2
labels CONTROL access as privileged without distinguishing reads from
writes. The experiment follows the access rules in Arm §B3.4. For IPSR
and exception entry/return, see Arm §§B3.5.1, B3.18–B3.19 and B3.22, and
PM0264 §§2.1.3.8 and 2.4.7. The raw EXC_RETURN is retained in RAM.
The handler checks `(EXC_RETURN & ~0x41) == 0xFFFFFFBC`, covering the
prefix and reserved fields, default stacking, basic frame, Thread
mode and PSP. Bits 6 and 0 are architecture security-state fields or
reserved fields depending on the security extension. The handler
leaves those hardware-generated bits intact. Record the full word
observed with TZEN=0. This procedure makes no cross-security-state claim.
[Armv8-M Architecture Reference Manual](https://documentation-service.arm.com/static/5f8f019ef86e16515cdbe64c) · [Cortex-M33 Devices Generic User Guide, §§2.1, 2.3.7 and 3.13](https://documentation-service.arm.com/static/5f16e93d20b7cf4bc524af1d) · [ST PM0264 Rev 8](https://www.st.com/resource/en/programming_manual/pm0264-stm32-cortexm33-mcus-and-mpus-programming-manual-stmicroelectronics.pdf)

The memory layout follows the board baseline and RM0456 Rev 6 §2.3.3,
Table 7 and §3.5.3. GPIO uses the baseline PC7 configuration and clock
gate described in RM0456 §§11.8.30 and 13.4.
The test uses aligned loads, the reset clock and no sleep-on-exit.
ES0553 describes limitations involving unaligned device access and
Flash reads around interrupt return under other timing conditions.
[ST RM0456 Rev 6](https://www.st.com/resource/en/reference_manual/rm0456-stm32u5-series-armbased-32bit-mcus-stmicroelectronics.pdf) · [ST ES0553 device errata](https://www.st.com/content/st_com/en/technical-documents/es0553-stm32u59xxx-and-stm32u5axxx-device-errata-stmicroelectronics_asset/chunk-001-topic-es-description.html)

## Implementation and review scope

`privilege-svc/` is a standalone Cargo workspace. It uses `no_std` and
no allocator, with the same locked direct dependency versions as blink
(`cortex-m` 0.7.10, `cortex-m-rt` 0.7.7 with `set-vtor`, and
`stm32u5` 0.16.0 with `rt` and `stm32u5a5`). The runtime provides reset
and core vectors. The PAC provides device vectors and register APIs.

- `transition.s` is included by `global_asm!`. It owns the RAM symbols
  and emits explicit stores for every observation, stack switch and
  exception transition. Rust has no mutable static references or unsafe
  attributes. Runtime assembly and dependency MMIO internals also form
  part of the trusted execution path.
- The `cortex-m` helper `enter_unprivileged_psp` requires valid executable
  code, an eight-byte-aligned stack top, and a sufficiently large backing
  stack. Its implementation writes PSP, sets the two CONTROL bits,
  executes ISB and branches. The experiment implements the same sequence
  in a stackless entry to establish the absence of a live MSP Rust frame
  independently of compiler prologue and inlining decisions.
- `cortex-m-rt` documents its `main` ABI and allows its linker exception
  aliases to be overridden by defining the corresponding symbols. The
  experiment defines Thumb `SVCall` directly, with the runtime's vector
  table and linker script. This gives access to hardware LR before any
  compiler trampoline. HardFault and the runtime default exception
  target also have stackless definitions. The handler leaves LR, r4–r11,
  and the hardware frame intact. The core restores r0–r3 and r12.
- `main.rs` has two unsafe blocks. The startup block acquires RCC and
  GPIOC after startup validation and relinquishes its local handles on
  return. The final block acquires GPIOC again and takes SysTick via the
  core peripheral handle on the one-way success path. No other code
  acquires these peripherals and handlers access no MMIO.
- In the inspected release, the startup LED call returns after its
  eight-byte MSP frame, the probe and SVCall handler use no software
  stack, the basic exception frame occupies 32 bytes of PSP, and the
  final Rust LED loop uses an eight-byte PSP frame and makes no calls.
  The 2 KiB PSP reservation is specific to this image. It provides no
  architectural stack-limit enforcement.
  [cortex-m 0.7.10 assembly source](https://docs.rs/cortex-m/0.7.10/src/cortex_m/asm.rs.html) · [cortex-m-rt 0.7.7 runtime documentation](https://docs.rs/cortex-m-rt/0.7.7/cortex_m_rt/) · [cortex-m-rt 0.7.7 linker source](https://docs.rs/crate/cortex-m-rt/0.7.7/source/link.x.in)

Architecture §§2 and 4.2–4.2.1 define the project's PIL responsibilities.
The project scope, MCL FSM and interface design keep cryptographic
services, authoritative state and the protection contract outside this
firmware. The cooperative probe and handler demonstrate processor behavior
only. With the MPU disabled, unprivileged code can write the phase state,
diagnostics and stacks in ordinary SRAM. The SVC opcode, fixed call-site
check and phase marker do not constitute secure authorization. The SVC
mechanism is not yet a security boundary and the RAM records are not
tamper-resistant. This experiment provides no evidence of hardware
isolation, fault containment, zeroization or validation.
[Architecture](../../docs/assurance/architecture.md) · [Scope](../../docs/assurance/project-scope.md) · [MCL FSM](../../docs/design/mcl-fsm.md) · [Module interfaces](../../docs/design/module-interfaces.md) · [Documentation standard](../../docs/documentation-standard.md)

## VM build and static inspection

Run from the board directory. Record the repository revision and retain the
exact reviewed source and lockfile, including any working tree changes.
The captured build used base revision
`4f7ec5d5d2d5a1ef9ffbfa50dc47fd46dbce56af` plus the experiment files,
Rust 1.98.1 (commit `48a229cea`, LLVM 22.1.8),
`thumbv8m.main-none-eabi`, and GNU Arm binutils 2.42. The board's
`.cargo/config.toml` provides `-Tlink.x`.

```sh
git rev-parse HEAD
rustc +1.98.1 -Vv
cargo +1.98.1 fetch --manifest-path privilege-svc/Cargo.toml --locked \
  --target thumbv8m.main-none-eabi

cd privilege-svc
cargo +1.98.1 fmt --all --check
awk 'length($0)>80 { print FILENAME,FNR; failed=1 }
  END { exit failed }' build.rs src/main.rs src/transition.s
cargo +1.98.1 check --release --locked --offline
cargo +1.98.1 clippy --release --locked --offline -- -D warnings
cargo +1.98.1 build --release --locked --offline
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" \
  cargo +1.98.1 doc --no-deps --locked --offline
cargo +1.98.1 tree --locked --offline --edges normal,build

elf=target/thumbv8m.main-none-eabi/release/nucleo-u5a5zj-q-privilege-svc
flash=target/privilege-svc-flash.bin
readelf -h -l -S -A -s -W "$elf"
readelf -x .vector_table "$elf"
arm-none-eabi-objdump -dC "$elf"
arm-none-eabi-objcopy -O binary --gap-fill 0xff \
  -j .vector_table -j .text "$elf" "$flash"
wc -c "$elf" "$flash"
sha256sum "$elf" "$flash"
```

Ensure the selected sections account for every file-backed Flash
`PT_LOAD` segment. This release contains two contiguous segments and
zero bytes of initialized RAM data. The 1,524-byte binary is their
byte-for-byte concatenation, with no padding through the end of the
Flash page. The explicit section list avoids including a zero-size
RAM section in the binary's address range.

| Item | Observed VM release ELF |
| --- | --- |
| Format | ELF32 little-endian Armv8-M Mainline, Thumb, EABI5 soft-float |
| Entry and reset vector | `0x08000259` including Thumb bit |
| Initial MSP | `0x200C0000` in first vector word |
| SVCall | Code `0x0800037C`, vector slot `0x0800002C` contains `0x0800037D` |
| `.vector_table` | Flash `0x08000000`, 600 bytes |
| `.text` | Flash `0x08000258`, 924 bytes |
| `.rodata`, `.data`, `.gnu.sgstubs` | Zero bytes each |
| `.bss` | RAM `0x20000000`, 84 bytes, results |
| `.uninit` | RAM `0x20000058`, 2,048 bytes, PSP top `0x20000858` exclusive |
| Flash payload span | `[0x08000000, 0x080005F4)`, within first 8 KiB page |
| ELF size | 137,560 bytes |
| ELF SHA-256 | `6bdbc21ce145d553d5a71e8f81b25d42efce3f628a995de74479f246af355406` |
| Flash-payload SHA-256 | `5fee694e0b21d59135b536c17c6190f1facac67b865e50ea0acf9a31a8ff0588` |

The linked instructions show reset calling `main` at `0x080002A4`.
The startup transition emits `MSR PSP,r2` at `0x080002D4`,
`MSR CONTROL,r1` with input 3 at `0x080002DA`, and ISB at
`0x080002DE`. The direct unprivileged write with input 2 is at
`0x08000324`. ISB and readback follow at `0x08000328` and
`0x0800032C`. The call instruction at `0x0800033C` is the halfword
`DF35`. The handler writes zero to CONTROL at `0x08000414`,
executes ISB at `0x08000418`, and uses `BX LR` at `0x08000420`.
The linked continuation samples CONTROL and IPSR at `0x08000340`
and `0x08000346`.

## Proposed Mac run after engineering approval

1. Transfer the reviewed ELF with an `.elf` extension. Compare its
   SHA-256 on the Mac with `shasum -a 256`. Check availability of the
   reviewed blink ELF and matching Mac-local factory backup before
   replacing the current application. Record which image is to be used
   for recovery.
2. Connect STM32CubeProgrammer via the established SWD Hot Plug setup.
   Confirm the device and compare the option bytes to the board capture.
   Open the reviewed ELF and verify that programming starts at
   `0x08000000` and spans only the first 8 KiB page. Program with
   verification enabled, automatic run disabled, and page erase.
   Keep mass erase and option-byte programming disabled. If the tool
   rejects ELF segment alignment, stop for review. Any binary used
   for programming needs to account for the 16-byte programming
   granularity, with erased-state `0xFF` tail padding if needed.
   Record the actual programmed artifact and its digest.
3. Disconnect and hardware reset. Watch LD1 for at least ten seconds.
   Success is two roughly 125 ms green pulses separated by 125 ms
   off, followed by roughly 875 ms off, repeated. A failed check keeps
   LD1 solid after GPIO startup. A failure before GPIO setup may
   leave it dark, and a fault after blinking starts may freeze either
   LED level. Diagnose a missing pair pattern through SWD.
4. Read the SRAM results using Hot Plug after the LED behavior
   stabilizes. Use the Memory & File view at the ELF symbol
   `experiment_results` for 84 bytes, or use the read-only CLI below
   with the address from the actual ELF. Hot Plug preserves the
   current execution and records. Read twice if the status is still
   changing. Reset clears the record through runtime `.bss` init,
   so collect it before retrying.
5. Repeat the hardware reset, LED observation, and RAM read. A pass
   requires the expected pair pattern and all expected words,
   including status 5 and no unexpected exception, on both runs.
   Read and compare option bytes again without writing them.
   [STM32CubeProgrammer documentation](https://dev.st.com/stm32cube-docs/prog/) · [ST CLI memory reads and SWD modes](https://dev.st.com/stm32cube-docs/prog/2.23.0/en/docs/markup/CubeProg_Command_Lines.html)

```sh
STM32_Programmer_CLI -c port=SWD mode=HOTPLUG -r32 0x20000000 0x54
```

### SWD result layout

Offsets are bytes from `experiment_results`. Each entry is one
32-bit little-endian word. Values below are expectations for the
inspected ELF.

| Offset | Observation | Expected |
| --- | --- | --- |
| `0x00` | Status | `5` |
| `0x04` | Startup CONTROL | `0` |
| `0x08` | Startup IPSR | `0` |
| `0x0C` | Startup active SP | `0x200C0000` |
| `0x10` | Unprivileged CONTROL | `3` |
| `0x14` | Unprivileged IPSR | `0` |
| `0x18` | Unprivileged active SP | `0x20000858` |
| `0x1C` | CONTROL after direct MSR | `3` |
| `0x20` | Handler CONTROL before clearing nPRIV | `1` |
| `0x24` | Handler IPSR | `11` |
| `0x28` | Handler active SP | `0x200C0000` |
| `0x2C` | Handler MSP | `0x200C0000` |
| `0x30` | Hardware EXC_RETURN | Masked value `0xFFFFFFBC` as defined above |
| `0x34` | PSP at handler entry | `0x20000838` |
| `0x38` | Stacked PC | `0x0800033E` |
| `0x3C` | Stacked xPSR | `(word & 0x010003FF) == 0x01000000` |
| `0x40` | SVC instruction halfword, zero-extended | `0x0000DF35` |
| `0x44` | Returned Thread CONTROL | `2` |
| `0x48` | Returned Thread IPSR | `0` |
| `0x4C` | Returned Thread active SP | `0x20000858` |
| `0x50` | Unexpected exception IPSR | `0` |

Status `0` means startup incomplete. Intermediate codes `1–4`
denote startup, unprivileged Thread, ignored write, and accepted
SVCall respectively. Code `5` means the return checks passed.
Accept overall success only with the paired LED pattern as well.
Codes `0x81–0x85` mean the corresponding phase failed. Code `0x86`
means an unexpected exception reached the fault/default handler.
Its IPSR gives the exception number. A failure branches into the
`experiment_halted` spin loop. A fault before the handler can execute
may leave the record incomplete. A fault after status 5 can replace
it with `0x86`. Nonzero intermediate status alone is insufficient.
If SWD can still read the system registers, capture CFSR at
`0xE000ED28` and HFSR at `0xE000ED2C` before resetting. Interpret
them using PM0264 Rev 8 §§4.3.12–4.3.13.

## Recovery and record fields

Preserve the Hot Plug dump before reset if possible. If it fails, use
the previously demonstrated hardware Connect Under Reset procedure, halt
and record any available state, then restore the reviewed blink ELF
using the board baseline's programming and verification workflow.
Check its familiar half-second blink and read-only option-byte parity.
If restoring the factory image instead, follow the
[baseline recovery procedure](FIRMWARE.md#proposed-mac-deployment-after-review)
with the verified Mac-local backup. Keep the pre-reset diagnostic
capture distinct from any read after recovery connection or reset.

| Evidence field | Actual observation |
| --- | --- |
| Operator, UTC time, board ID/revision | |
| Source revision, working tree, exact tool versions | |
| ELF and programmed artifact SHA-256, verified Flash range | |
| Read-only option-byte comparisons before and after | |
| LED pattern and timing, first run and repeat | |
| Raw 84-byte SWD dumps, both runs | |
| Decoded processor fields, status, full EXC_RETURN | |
| Failure diagnostics or recovery actions and result | |
| Phase-by-phase and overall pass/fail with evidence links | |
| Reviewer and review disposition | |
