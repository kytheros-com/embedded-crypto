# Initial Firmware Deployment — 2026-11-21

**Project:** Kytheros Embedded Cryptography  
**Hardware:** ST NUCLEO-U5A5ZJ-Q  
**Branch:** `hardware/NUCLEO-U5A5ZJ-Q`  
**Result:** PASS

## 1. Objective

Establish a reproducible workflow for compiling, programming, and executing bare-metal Rust firmware on the STM32U5A5ZJ development board.

Demonstrate basic GPIO operation, SysTick timing, Flash programming, and independent verification of programmed firmware.

## 2. Hardware Configuration

| Property | Value |
|---|---|
| Board product ID | `NUU5A5ZJQ$AT2` |
| Board assembly | `MB1549-U5A5ZJQ-C04` |
| MCU | STM32U5A5ZJ |
| Silicon revision | X |
| Processor | Arm Cortex-M33 |
| Debug interface | STLINK-V3E / SWD |
| Readout protection | Level 0 (original baseline) |
| TrustZone | Disabled (original baseline) |

The original option-byte configuration was captured before firmware development and is preserved separately.

Connect-under-reset debugging had previously been demonstrated successfully.

## 3. Factory Firmware Preservation

Before programming, the complete 4 MiB user Flash contents were captured twice using STM32CubeProgrammer.

The two independently captured images were compared byte for byte and found identical.

**Factory Flash SHA-256:**

`c58a081913d5767ff4960c18a4e2b9dc1d532515696c9dafc35c5cad5d81f021`

The backups are stored outside the Git repository.

Restoration from these images has not yet been tested.

## 4. Firmware Implementation

The initial application is a standalone Rust `no_std` program that toggles GPIO PC7, connected to the green user LED LD1.

The application uses the reset-time clock configuration and polls SysTick to produce approximately 0.5 seconds on and 0.5 seconds off.

No application interrupts, DMA, MPU configuration, TrustZone configuration, or watchdog initialization are required.

Build target: `thumbv8m.main-none-eabi`  
Rust toolchain: `1.98.1`

## 5. Independent Build Verification

The firmware was compiled independently on:

- GCP development VM: Linux x86-64.
- MacBook Air: macOS ARM64.

Both builds used the same Rust toolchain version, target architecture, and locked Cargo dependencies.

The complete ELF files had different SHA-256 values:

| Build environment | ELF SHA-256 |
|---|---|
| GCP VM | `30c689567bd5774b529818ba96431d35f34c996a29ece9a11c1feafd5c96764b` |
| MacBook Air | `e8bc100a3000770912d37245af17706f8ab78d4c072f85d5c31b15042f98e32e` |

The ELF loadable Flash segments were independently extracted and hashed.

Both builds produced identical executable Flash contents:

| Segment | Address | Size |
|---|---|---|
| Vector table | `0x08000000` | 600 bytes |
| Executable code | `0x08000258` | 272 bytes |

**Canonical unpadded Flash-payload SHA-256:**

`eee0c5d3b3c38eef00f87ca4199a759ab801c516b57d1dabd48cf0b6cee11abe`

**Result:** Cross-platform reproducibility of the loadable Flash payload demonstrated for this application.

This result does not establish reproducibility of all ELF metadata or independently validate the compiler.

## 6. Firmware Programming

The Mac-built firmware was used for deployment.

The two contiguous ELF load segments were combined into a binary image, padded with eight bytes of `0xFF` to satisfy the STM32U5 Flash programming granularity.

Programming image length: **880 bytes**

Programming address: `0x08000000`

Programming interface: STM32CubeProgrammer 2.24.0 using ST-LINK over SWD.

The operation replaced the initial portion of the factory application. No option-byte programming was requested.

## 7. Independent Flash Verification

Following programming, the first 880 bytes of MCU Flash were read back through SWD.

The initial read-back attempt failed because CubeProgrammer rejected the output filename extension. After correcting the filename to end in `.bin`, the read succeeded.

The read-back image was compared with the programming image.

**Results:**

- Byte-for-byte comparison: PASS.
- SHA-256 comparison: PASS.
- Read-back length: 880 bytes.

The comparison establishes that the programmed region matches the reference binary. It does not constitute verification of the remaining Flash contents.

## 8. Execution Verification

The development board was reset using its physical RESET button.

**Observed behavior:**

- The previous factory RGB LED demonstration was replaced.
- Green user LED LD1 blinked repeatedly.
- The observed blinking behavior was consistent with the application's intended operation.

**Result:** PASS.

The observation demonstrates successful boot, execution of the application, GPIO configuration, and operation of the polled SysTick timing loop.

Precise timing accuracy was not measured.

## 9. Assurance Summary

| Test | Result |
|---|---|
| Complete factory Flash backup | PASS |
| Independent backup comparison | PASS |
| SWD connect under reset | PASS |
| Rust cross-compilation | PASS |
| Independent Flash-payload reproducibility | PASS |
| MCU Flash programming | PASS |
| Independent Flash read-back comparison | PASS |
| Boot and GPIO execution | PASS |
| Recovery by restoring factory firmware | Not tested |
| Post-deployment option-byte comparison | Not recorded |
| MPU, privilege, and TrustZone protection | Not tested |
| DMA isolation | Not tested |

## 10. Conclusions

The initial hardware bring-up milestone was completed successfully.

The project now has a demonstrated development workflow connecting independently reproducible Rust builds to physical firmware execution and read-back verification.

The result provides preliminary evidence useful for developing future firmware provenance and reproducible-build procedures.

No cryptographic functionality or security isolation claims are established by this test.

Future work will expand hardware testing toward the Platform Integration Layer (PIL), including privilege transitions, memory protection, interrupt control, and DMA isolation.

**Overall result: PASS — Initial firmware deployment and execution verified.**
