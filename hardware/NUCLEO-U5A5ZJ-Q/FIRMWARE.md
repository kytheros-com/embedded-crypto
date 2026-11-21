# NUCLEO-U5A5ZJ-Q LD1 firmware

This standalone bring-up application is for product `NUU5A5ZJQ$AT2`,
assembly MB1549-U5A5ZJQ-C04, with STM32U5A5ZJ silicon revision X. The
[hardware inventory](README.md) records the baseline and option bytes. The
firmware assumes normal boot from internal Flash at `0x08000000` after a
target reset, with BOOT0 low, TZEN=0, RDP Level 0, and the default board
routing (SB21 closed, SB23 open).

LD1 starts off, then repeats roughly 0.5 seconds on and 0.5 seconds off.
The actual period depends on the MSIS oscillator. LD1 is the green user
LED, distinct from the green power indicator LD5.

## Implementation and hardware references

- UM2861 Rev 10, §7.5 and the C04 schematic (sheet 5) specify the active-high
  PC7 connection. [UM2861](https://www.st.com/resource/en/user_manual/um2861-stm32u5-nucleo144-boards-mb1549-stmicroelectronics.pdf)
  · [C04 schematic](https://www.st.com/resource/en/schematic_pack/mb1549-u5a5ziq-c04-schematic.pdf)
- RM0456 Rev 6, §11.8.30 and §13.4 specify the clock gate and GPIO fields.
  The PAC uses RCC at `0x46020C00` and GPIOC at `0x42020800`.
  The firmware enables only `RCC_AHB2ENR1.GPIOCEN` (bit 2) and reads the
  enable register back before accessing GPIOC. It preloads PC7 low and
  configures PC7 as low-speed push-pull output with no pull resistor.
  Writes to GPIOC BSRR use bit 7 to set the pin and bit 23 to reset it.
  Register bit numbers start at the least-significant bit.
  [RM0456](https://www.st.com/resource/en/reference_manual/rm0456-stm32u5-series-armbased-32bit-mcus-stmicroelectronics.pdf)
  · [ST device header](https://github.com/STMicroelectronics/cmsis-device-u5/blob/main/Include/stm32u5a5xx.h)
  · [ST GPIOC clock-enable macro](https://github.com/STMicroelectronics/stm32u5xx-hal-driver/blob/main/Inc/stm32u5xx_hal_rcc.h)
- The reset clock assumption is the default MSIS range of 4 MHz and
  unity CPU/AHB prescaling. The application selects the core clock for
  SysTick, writes a reload of 1,999,999 (2,000,000 cycles per wrap),
  clears the current value, and polls COUNTFLAG. TICKINT stays clear
  from reset. Launching the ELF over a running application can violate
  these assumptions.
  [ST RCC documentation](https://dev.st.com/stm32cube-docs/stm32u5-hal2/2.0.0-beta.1.1/docs/drivers/hal_drivers/rcc/hal_rcc_how_to_use.html)
  · [PM0264, SysTick registers](https://www.st.com/resource/en/programming_manual/pm0264-stm32-cortexm33-mcus-programming-manual-stmicroelectronics.pdf)
- The linker uses the 4 MiB user Flash window `0x08000000–0x083FFFFF`
  and SRAM1 alone, `0x20000000–0x200BFFFF` (768 KiB). The initial stack
  pointer is the exclusive end of SRAM1, `0x200C0000`, and the stack
  grows downward. The little-endian vector table begins at the Flash
  origin. The runtime initializes RAM sections and sets VTOR to the
  linked vector table.
  [AN5766, Tables 3–4](https://www.st.com/content/st_com/en/technical-documents/an5766-migrating-within-stm32u5-series-microcontrollers_asset/chunk-003-topic-an5766-flash-memory.html)
  · [DS13543](https://www.st.com/resource/en/datasheet/stm32u5a5aj.pdf)
  · [ES0553, revision X errata](https://www.st.com/content/st_com/en/technical-documents/es0553-stm32u59xxx-and-stm32u5axxx-device-errata-stmicroelectronics.html)

The standalone workspace keeps the board dependencies and lint settings
local. The application is `no_std` and allocation-free. `cortex-m-rt`
provides reset and exception vectors with the device vectors from the
`stm32u5` PAC. `cortex-m` provides SysTick and the panic loop's NOP.
The only application `unsafe` block acquires the register handles once
in the sole entry point. Register access and startup also rely on the
dependencies' internal unsafe code and assembly. A Rust panic stops
blinking by spinning in a NOP loop with no logging or reset. A fault
uses the runtime's default handlers.

## Build and inspect on the VM

Run from the repository root with Rust 1.98.1. The board directory sets
the default build target and linker flag.

```sh
rustup target add --toolchain 1.98.1 thumbv8m.main-none-eabi
cd hardware/NUCLEO-U5A5ZJ-Q
cargo fetch --locked --target thumbv8m.main-none-eabi
cargo fmt --all --check
cargo clippy --release --locked --offline -- -D warnings
cargo build --release --locked --offline
cargo tree --locked --offline --edges normal,build
find src build.rs -type f -name '*.rs' -exec awk '
  length($0) > 80 { print FILENAME, FNR, length($0); failed = 1 }
  END { exit failed }
' {} +

elf="$PWD/target/thumbv8m.main-none-eabi/release/nucleo-u5a5zj-q-blink"
readelf -h -l -S -W "$elf"
readelf -x .vector_table "$elf"
readelf -A -s -W "$elf"
size -A "$elf"
sha256sum "$elf"
```

### VM release build results

| Item | Observed result |
| --- | --- |
| ELF | 32-bit little-endian Arm, Armv8-M Mainline, soft-float ABI |
| Reset entry | `0x08000259` (Thumb bit set, code at `0x08000258`) |
| Vector table | `0x08000000`, 600 bytes, first words `0x200C0000` and `0x08000259` |
| Code | `.text` at `0x08000258`, 272 bytes |
| Data | `.rodata`, `.data`, `.bss` and `.uninit` are each 0 bytes |
| Flash payload | 872 bytes in two adjacent load segments, within the first 8 KiB page |
| ELF file size | 71,788 bytes |
| SHA-256 | `30c689567bd5774b529818ba96431d35f34c996a29ece9a11c1feafd5c96764b` |

The locked direct dependencies are `cortex-m` 0.7.10, `cortex-m-rt`
0.7.7, and `stm32u5` 0.16.0. Formatting, the 80-character Rust line
limit, release Clippy with warnings denied, release build, and ELF
inspection passed on the VM. The root workspace host and bare-metal CI
checks passed independently.

## Proposed Mac deployment after review

1. Transfer the reviewed ELF to the Mac with an `.elf` extension. Use
   `shasum -a 256` on the Mac to compare it with the VM build's checksum.
   Confirm the local 4 MiB factory backup is available and matches
   `c58a081913d5767ff4960c18a4e2b9dc1d532515696c9dafc35c5cad5d81f021`.
2. Connect with the SWD Hot Plug setup from the inventory. Confirm MCU
   device ID `0x481` and revision X. Read the option bytes and compare
   them with the captured baseline.
3. Open the ELF in STM32CubeProgrammer, confirm the image starts at
   `0x08000000`, and download with programming verification enabled.
   Keep automatic run disabled and use the required erase of the first
   8 KiB page. Leave full-chip erase and option-byte programming disabled.
   Review the programmer's handling of the STM32U5 16-byte write
   granularity. The second ELF segment starts at `0x08000258` and the
   payload ends at `0x08000368` (exclusive). If the tool rejects the ELF
   on alignment, stop and prepare a
   reviewed contiguous binary padded with erased-state bytes before retrying.
4. Confirm programming verification succeeds. Disconnect and reset
   the target with the board's reset button or power cycle so startup
   runs from the hardware reset state. Observe LD1 for ten cycles
   (approximately ten seconds). Confirm that blinking resumes after
   another reset and record the observed period, ELF checksum, and
   read-only option-byte comparison.

For recovery, reconnect through the documented SWD setup. If Hot Plug
fails, use a temporary Connect Under Reset session for recovery, with
human approval and the existing board wiring. Resolve connection or
verification failures before continuing. To restore the factory image,
select the verified Mac-local backup as a raw binary at `0x08000000`
and erase/program the user Flash pages it spans. Verify the download,
then read back all 4 MiB and compare its SHA-256 with the original
backup checksum. Reset and confirm the original LED sequence and the
read-only option-byte comparison.
[STM32CubeProgrammer UM2237](https://www.st.com/resource/en/user_manual/dm00403500-stm32cubeprogrammer-software-description-stmicroelectronics.pdf)

Deployment, LED timing, and recovery remain pending human review and
on-board verification.
