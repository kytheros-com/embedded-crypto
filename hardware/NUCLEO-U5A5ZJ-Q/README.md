# NUCLEO-U5A5ZJ-Q — Hardware Inventory and Bring-Up Record

**Date:** 2026-11-21  
**Status:** Initial hardware identification completed; no target firmware or security configuration changes made.

## 1. Hardware Identification

| Property | Observed Value |
|---|---|
| Development board | ST NUCLEO-U5A5ZJ-Q |
| Board PCB/assembly revision | MB1549-U5A5ZJQ-C04 (PCB C, assembly 04) |
| Product identification | NUU5A5ZJQ$AT2 |
| Target MCU | STM32U5A5ZJ |
| Processor core | Arm Cortex-M33 (Armv8-M Mainline) |
| Device ID | `0x481` |
| Silicon revision | Rev X |
| Nonvolatile memory | 4 MB |
| Bootloader version | `0x92` |
| Measured target voltage | 3.27 V |
| Onboard debugger | STLINK-V3E |
| ST-LINK serial number | `003800353335511835383531` |
| ST-LINK firmware | `V3J10M3` |

Identification was obtained through STM32CubeProgrammer using an SWD Hot Plug connection.

## 2. Development Environment

| Component | Configuration |
|---|---|
| Host computer | Apple MacBook Air (Apple Silicon) |
| Operating system | macOS 27.2.1 |
| Programming software | STM32CubeProgrammer 2.24.0 |
| Debug interface | SWD |
| Connection mode | Hot Plug |
| Reported SWD frequency | 8 MHz |
| USB connection | Via Dell U2725QE monitor USB hub |
| ST-LINK virtual COM port | `/dev/cu.usbmodem1143203` (may change) |

### USB Connectivity Observations

Direct USB connection between the MacBook Air and ST-LINK produced intermittent `libusb` errors, including:

- `0xe00002d8` — device not ready.
- `0xe00002ed` — device not responding.

Connecting the board through a downstream USB-C port in the Dell U2725QE monitor's retractable USB panel resolved these errors during initial identification.

STM32CubeProgrammer subsequently enumerated ST-LINK and successfully connected to the target MCU through SWD.

This establishes a working communication configuration, although longer-term reliability remains to be demonstrated.

## 3. Initial Board Behavior

Before any programming:

- LD4 illuminated red when ST-LINK was idle.
- LD5 illuminated green.
- LD1, LD2, and LD3 displayed a repeating green, blue, and red sequence.
- The observed LED activity was consistent with the board's factory demonstration firmware.
- The ST-LINK USB mass-storage interface appeared as `NOD_U5A5ZJ`.

The identity and contents of the factory application firmware have not been independently verified.

## 4. Original Option-Byte Configuration

The original configuration was captured using STM32CubeProgrammer's read-only `-ob displ` operation.

The complete original output is preserved in:

`stm32u5a5zj-original-option-bytes.txt`

### Important Settings

| Option | Original Value | Interpretation |
|---|---|---|
| `RDP` | `0xAA` | Readout protection Level 0 |
| `TZEN` | `0` | TrustZone disabled |
| `BOR_LEV` | `0` | Brownout reset Level 0 |
| `SWAP_BANK` | `0` | Flash banks not swapped |
| `IWDG_SW` | `1` | Software-controlled independent watchdog |
| `WWDG_SW` | `1` | Software-controlled window watchdog |
| `SRAM_RST` | `1` | SRAM1, SRAM3, and SRAM4 not erased on system reset |
| `SRAM2_RST` | `1` | SRAM2 not erased on system reset |
| `BKPRAM_ECC` | `1` | Backup RAM ECC disabled |
| `SRAM2_ECC` | `1` | SRAM2 ECC disabled |
| `SRAM3_ECC` | `1` | SRAM3 ECC disabled |
| `NSBOOTADD0` | `0x100000` | Boot address `0x08000000` |
| `NSBOOTADD1` | `0x17F200` | Boot address `0x0BF90000` |

All four recorded Flash write-protection ranges are inactive, with page start `0xFF` and page end `0x00`.

The configuration is consistent with an unrestricted development environment.

These are observations of the option bytes, not evidence of current runtime MPU, GTZC, or DMA protection settings.

## 5. Development Safety Policy

Until explicitly reviewed and authorized:

- Do not change readout protection.
- Do not enable TrustZone.
- Do not change Flash security or write-protection option bytes.
- Do not activate irreversible debug restrictions.
- Do not upgrade ST-LINK firmware.
- Do not modify hardware jumpers or power configurations without first documenting their original positions.
- Prefer protection experiments based on reversible runtime register configurations.
- Establish and test a recovery procedure before performing experiments that may disrupt execution or debug access.

In particular, **RDP Level 2 must not be enabled on this development board**.

Any future option-byte modification requires a documented assessment of reversibility, possible mass erase, debug accessibility, and recovery requirements.

## 6. Initial Milestone Results

| Milestone | Result |
|---|---|
| USB enumeration on macOS | PASS |
| ST-LINK identification | PASS |
| Virtual COM port enumeration | PASS |
| SWD target connection | PASS |
| MCU device identification | PASS |
| Silicon revision identification | PASS |
| Original option-byte capture | PASS |
| Target Flash programming | Not attempted |
| Target firmware execution under our control | Not attempted |
| Runtime memory-protection testing | Not attempted |
| DMA isolation testing | Not attempted |
| TrustZone activation | Not attempted |

The PASS results above refer only to the observed initial bring-up operations. They do not imply general hardware reliability or security validation.

## 7. Next Steps

The next development milestone is a minimal, reproducible bare-metal Rust application that can be flashed, executed, observed, and replaced without modifying security option bytes.

Initial tasks include confirming board assembly revision, establishing the build and flashing toolchain, testing the UART communication interface, and recording the first successful target-side execution.

Memory-protection and DMA-isolation experiments will follow only after the basic toolchain and hardware recovery procedures are established.

## 8. References

- ST UM2861 — STM32U5 Nucleo-144 boards user manual.
- ST RM0456 — STM32U5 reference manual.
- ST PM0264 — Cortex-M33 programming manual.
- ST ES0553 — STM32U59xxx/U5Axxx device errata, applicable to silicon revision X.
- Original board configuration capture: `stm32u5a5zj-original-option-bytes.txt`.

**Assurance status:** Internal development evidence only. No FIPS 140-3 or cryptographic module validation claim is implied.
