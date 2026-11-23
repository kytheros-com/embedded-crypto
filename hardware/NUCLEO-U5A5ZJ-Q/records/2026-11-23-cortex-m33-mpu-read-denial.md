# Cortex-M33 MPU Read-Denial Experiment — 2026-11-23

**Project:** embedded-crypto  
**Maintainer:** Kytheros LLC  
**Hardware:** ST NUCLEO-U5A5ZJ-Q  
**Experiment:** MPU-enforced denial of unprivileged SRAM access  
**Date:** 2026-11-23  
**Overall result:** **PASS — Two hardware-reset executions produced identical diagnostic records.**

## 1. Objective

Verify that the Arm Cortex-M33 Memory Protection Unit (MPU) enforces a privilege boundary between application working memory and supervisor-owned memory.

The experiment was designed to establish five properties:

1. Privileged Thread code can configure and enable the MPU.
2. Privileged code can read supervisor-owned SRAM before and after MPU activation.
3. Unprivileged Thread code can read authorized working memory.
4. An unprivileged read of supervisor-owned SRAM generates a MemManage data-access violation.
5. The privileged MemManage handler can identify the prohibited instruction, fault address, and relevant processor state.

The experiment builds upon the successful privilege/SVCall experiment documented in `2026-11-23-cortex-m33-privilege-svc.md`.

The test exercises one deliberately prohibited read. It does not establish comprehensive protection against arbitrary malicious code.

## 2. Hardware and Software Configuration

| Property | Value |
|---|---|
| Development board | NUCLEO-U5A5ZJ-Q |
| Product identifier | `NUU5A5ZJQ$AT2` |
| Board assembly | `MB1549-U5A5ZJQ-C04` |
| Microcontroller | STM32U5A5ZJ |
| Silicon revision | X |
| Processor | Arm Cortex-M33 |
| Device ID | `0x481` |
| Flash capacity | 4 MiB |
| SRAM1 capacity | 768 KiB |
| TrustZone baseline | Disabled (`TZEN=0`) |
| MPU regions implemented | 8 |
| Programming/debug interface | STLINK-V3E, SWD |
| ST-LINK firmware | V3J10M3 |
| STM32CubeProgrammer | 2.24.0 |
| SWD frequency | 1,000 kHz |
| Reported target voltage | 3.27 V |
| Development target | `thumbv8m.main-none-eabi` |
| Rust toolchain | 1.98.1 |

Firmware source:

`hardware/NUCLEO-U5A5ZJ-Q/mpu-read-denial/`

Experimental procedure:

`hardware/NUCLEO-U5A5ZJ-Q/2026-11-23-cortex-m33-mpu-read-denial-procedure.md`

The software was developed and initially built on the Linux development VM. A separate release build was performed on the Mac connected to the development board.

The experiment did not intentionally modify TrustZone, GTZC, DMA configuration, security option bytes, debug protection, or readout-protection settings.

## 3. Firmware Provenance

The initial MPU experiment implementation was committed as:

`f0a7e89c4253a3f8e774ff2b4bab2b4d203753e7`

Commit description: `Experiment with MPU protections`

The reference Linux build and independently generated Mac build produced identical Flash payload and programming-image SHA-256 values.

| Artifact | Size | SHA-256 |
|---|---:|---|
| Flash payload | 2,476 bytes | `931ec1170de2bb21431c6df28fd866bf9eb72c6a125599daf9cf63eaadb23bb3` |
| Programming image | 2,480 bytes | `db78782d0f08e769a19e7a60b396ef0c98d59f8fd612ca34185a176b52562ce8` |

The programming image includes four trailing `0xFF` bytes to satisfy the STM32U5 Flash programming granularity.

The complete ELF files were not byte-identical between Linux and macOS, as anticipated. The matching Flash payload establishes byte-for-byte agreement of the generated loadable Flash image.

The Linux reference ELF SHA-256 was:

`f53d7cc5ba86823d998b688e978228e46312c651e3e75bde5bc3880c74e87f43`

The final programming-image checksum was verified on the Mac before deployment.

## 4. Configured MPU Memory Map

The firmware configured three MPU regions using the Armv8-M `RBAR`, `RLAR`, and `MAIR` registers.

| Region | Address range (inclusive) | Privileged access | Unprivileged access |
|---|---|---|---|
| Flash | `0x08000000–0x08001FFF` | Read/execute | Read/execute |
| User SRAM | `0x20001000–0x20001FFF` | Read/write, XN | Read/write, XN |
| Supervisor SRAM | `0x20002000–0x20002FFF` | Read/write, XN | No access |

The recorded MPU register values were:

```text
MPU_TYPE = 0x00000800
MPU_CTRL = 0x00000005
MAIR0    = 0x00000044
MAIR1    = 0x00000000

Region 0:
    RBAR = 0x08000006
    RLAR = 0x08001FE1

Region 1:
    RBAR = 0x20001003
    RLAR = 0x20001FE1

Region 2:
    RBAR = 0x20002001
    RLAR = 0x20002FE1

Unused region enable mask = 0
```

The recorded configuration agrees with the intended MPU permissions and memory boundaries.

`MPU_CTRL = 5` enables the MPU and privileged background memory mapping (`PRIVDEFENA`). The privileged background map does not grant unprivileged code access to otherwise unmapped memory.

The MPU attributes use Normal, non-cacheable memory through MAIR index 0.

### Significant SRAM Locations

| Item | Address |
|---|---|
| Authorized working word | `0x20001000` |
| Unprivileged progress marker | `0x20001004` |
| Unprivileged PSP top | `0x20002000` |
| Privileged diagnostic record | `0x20002000` |
| Protected sentinel | `0x200020A4` |
| Privileged MSP top | `0x20003000` |

The supervisor-owned sentinel was initialized to:

`0x53A1CE7D`

The unprivileged working word was initialized to:

`0xA6074B19`

## 5. Experimental Procedure

The verified release firmware was programmed at Flash address `0x08000000` using the established SWD connection.

Following a hardware reset, the application executed its MPU initialization, privilege transition, authorized memory read, and deliberately prohibited memory read.

The expected successful outcome was a repeating three-flash pattern on the green LD1 indicator.

After the first execution, the diagnostic record and unprivileged working memory were read using SWD Hot Plug without resetting the processor.

The board was reset, and the procedure was repeated.

**The diagnostic memory contents were identical across both runs.**

The green LED displayed the repeating three-flash success pattern.

No externally calibrated timing measurement was performed.

### SWD Commands

Protected diagnostic record:

```sh
stm32cli \
  -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20002000 0xA4
```

Unprivileged working memory:

```sh
stm32cli \
  -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20001000 0x08
```

The first command reads 164 bytes (41 words). The second reads eight bytes (two words).

## 6. Raw Hardware Observations

### 6.1 Privileged Diagnostic Record

The following 164-byte record was obtained after execution and reproduced identically following the second hardware reset.

```text
Reading 32-bit memory content

Size     : 164 Bytes
Address  : 0x20002000

0x20002000 : 00000001 00000004 00000000 00000000
0x20002010 : 20002FA8 00000800 00000000 00000005
0x20002020 : 00000044 00000000 08000006 08001FE1
0x20002030 : 20001003 20001FE1 20002001 20002FE1
0x20002040 : 00000000 00000000 00000000 00010000
0x20002050 : 200020A4 53A1CE7D 53A1CE7D 00000001
0x20002060 : 00000004 20003000 20003000 FFFFFFBC
0x20002070 : 20001FE0 00000082 00000000 200020A4
0x20002080 : 080002DC 29000000 00000003 00000000
0x20002090 : 20002000 A6074B19 200020A4 F8DC6000
0x200020A0 : 00000000
```

### 6.2 Unprivileged Working Memory

The following values were also reproduced after reset.

```text
Reading 32-bit memory content

Size     : 8 Bytes
Address  : 0x20001000

0x20001000 : A6074B19 00000001
```

The first word is the authorized working value.

The second word is the progress marker indicating successful verification of the authorized word before the prohibited read was attempted.

A progress value of `2` would have indicated that the prohibited load completed. This did not occur.

## 7. Decoded Diagnostic Record

All 41 diagnostic words were compared with the expected values in the experimental procedure.

Offsets are byte offsets relative to `0x20002000`.

| Offset | Field | Measured | Result |
|---|---|---|---|
| `0x00` | Final status | `0x00000001` | PASS |
| `0x04` | Final phase | `0x00000004` | PASS |
| `0x08` | Startup CONTROL | `0` | PASS |
| `0x0C` | Startup IPSR | `0` | PASS |
| `0x10` | Startup MSP observation | `0x20002FA8` | PASS |
| `0x14` | MPU_TYPE | `0x00000800` | PASS |
| `0x18` | Initial MPU_CTRL | `0` | PASS |
| `0x1C` | Enabled MPU_CTRL | `5` | PASS |
| `0x20` | MAIR0 | `0x44` | PASS |
| `0x24` | MAIR1 | `0` | PASS |
| `0x28` | Flash RBAR | `0x08000006` | PASS |
| `0x2C` | Flash RLAR | `0x08001FE1` | PASS |
| `0x30` | User SRAM RBAR | `0x20001003` | PASS |
| `0x34` | User SRAM RLAR | `0x20001FE1` | PASS |
| `0x38` | Supervisor SRAM RBAR | `0x20002001` | PASS |
| `0x3C` | Supervisor SRAM RLAR | `0x20002FE1` | PASS |
| `0x40` | Unused enabled-region mask | `0` | PASS |
| `0x44` | Initial CFSR | `0` | PASS |
| `0x48` | Initial HFSR | `0` | PASS |
| `0x4C` | SHCSR | `0x00010000` | PASS |
| `0x50` | Expected fault address | `0x200020A4` | PASS |
| `0x54` | Sentinel read before MPU | `0x53A1CE7D` | PASS |
| `0x58` | Sentinel read after MPU | `0x53A1CE7D` | PASS |
| `0x5C` | Handler CONTROL | `1` | PASS |
| `0x60` | Handler IPSR | `4` | PASS |
| `0x64` | Handler active SP at entry | `0x20003000` | PASS |
| `0x68` | Handler MSP at entry | `0x20003000` | PASS |
| `0x6C` | EXC_RETURN | `0xFFFFFFBC` | PASS |
| `0x70` | Hardware frame PSP | `0x20001FE0` | PASS |
| `0x74` | CFSR | `0x00000082` | PASS |
| `0x78` | HFSR | `0` | PASS |
| `0x7C` | MMFAR | `0x200020A4` | PASS |
| `0x80` | Stacked PC | `0x080002DC` | PASS |
| `0x84` | Stacked xPSR | `0x29000000` | PASS |
| `0x88` | Thread CONTROL reading | `3` | PASS |
| `0x8C` | Thread IPSR reading | `0` | PASS |
| `0x90` | Thread SP reading | `0x20002000` | PASS |
| `0x94` | Authorized working value | `0xA6074B19` | PASS |
| `0x98` | Faulting address operand | `0x200020A4` | PASS |
| `0x9C` | Faulting instruction | `0xF8DC6000` | PASS |
| `0xA0` | Unexpected exception identity | `0` | PASS |

**All 41 diagnostic fields agree with their specified acceptance conditions.**

## 8. Analysis

### 8.1 Privileged Startup and MPU Initialization

The startup observations were:

```text
CONTROL = 0
IPSR    = 0
MSP     = 0x20002FA8
```

These values establish that startup executed in privileged Thread mode.

The observed MSP is 88 bytes below the configured stack top, consistent with the stack frame of the Rust startup function.

The initial MPU configuration was:

```text
MPU_TYPE = 0x00000800
MPU_CTRL = 0x00000000
```

This agrees with the previously established processor baseline: eight MPU regions implemented, MPU initially disabled.

Following configuration, the recorded MPU control value was:

```text
MPU_CTRL = 0x00000005
```

The register readbacks matched all three intended regions, and regions 3–7 remained disabled.

The supervisor sentinel was successfully read both before and after MPU activation:

```text
Before MPU: 0x53A1CE7D
After MPU:  0x53A1CE7D
```

This confirms that the configured MPU permissions did not prevent the intended privileged accesses.

**Result: PASS.**

### 8.2 Authorized Unprivileged SRAM Access

Immediately before the prohibited access, the unprivileged thread recorded:

```text
CONTROL = 3
IPSR    = 0
SP      = 0x20002000
```

`CONTROL = 3` indicates unprivileged Thread mode using PSP.

The authorized working value was successfully read and verified:

```text
Address: 0x20001000
Value:   0xA6074B19
```

The progress marker at `0x20001004` was set to `1`, confirming that execution passed the authorized-memory check and reached the deliberate prohibited-access sequence.

This is an important positive control: the MPU did not simply prevent all unprivileged SRAM access.

**Result: PASS.**

### 8.3 Prohibited Supervisor SRAM Read

The test then attempted a 32-bit read from:

```text
0x200020A4
```

This address lies inside the supervisor MPU region:

```text
0x20002000–0x20002FFF
```

That region is configured for privileged read/write access, with unprivileged access denied.

The processor reported:

```text
CFSR  = 0x00000082
HFSR  = 0x00000000
MMFAR = 0x200020A4
```

The MemManage fault-status bits set in `CFSR = 0x82` are:

- `DACCVIOL` (bit 1): data-access violation.
- `MMARVALID` (bit 7): valid MemManage fault address.

No other CFSR fault-status bits were set.

The reported faulting address exactly matches the protected sentinel address.

`HFSR = 0` indicates that the event did not escalate to a HardFault.

**Result: PASS.**

### 8.4 Faulting Instruction Verification

The hardware-stacked PC was:

```text
0x080002DC
```

The linked instruction at that address is:

```asm
ldr.w r6, [r12]
```

The diagnostic record contains the expected instruction encoding:

```text
F8DC 6000
```

The stacked address operand was:

```text
r12 = 0x200020A4
```

Thus, three independently relevant observations agree:

1. The stacked PC identifies the designated load instruction.
2. The instruction reads from the address contained in `r12`.
3. The processor's MMFAR identifies that same address.

The unprivileged progress marker remained `1`, rather than advancing to `2`.

These observations demonstrate that the deliberately prohibited read triggered the expected MemManage fault before the successful-load continuation executed.

**Result: PASS.**

### 8.5 Privileged MemManage Handler Execution

The handler recorded:

```text
CONTROL = 1
IPSR    = 4
SP      = 0x20003000
MSP     = 0x20003000
```

`IPSR = 4` identifies the MemManage exception.

The handler executed using MSP in privileged Handler mode.

As established in the preceding SVC experiment, `CONTROL.nPRIV = 1` reflects the Thread-mode privilege configuration; Handler mode is privileged regardless of this value.

The configuration register also showed:

```text
SHCSR = 0x00010000
```

This confirms that the MemManage exception was explicitly enabled.

The handler entered its expected validation path rather than the unexpected-exception path.

**Result: PASS.**

### 8.6 Exception Frame and Stack Integrity

The recorded exception-return state was:

```text
EXC_RETURN = 0xFFFFFFBC
Frame PSP  = 0x20001FE0
Stacked PC = 0x080002DC
Stacked xPSR = 0x29000000
```

The pre-exception PSP was:

```text
0x20002000
```

The difference is:

```text
0x20002000 - 0x20001FE0 = 0x20 = 32 bytes
```

This agrees with the expected eight-word basic exception frame.

The frame lies entirely inside the unprivileged SRAM region, ending immediately below its exclusive upper boundary.

The recorded xPSR satisfies the experiment's validation mask:

```text
0x29000000 & 0x010003FF = 0x01000000
```

This confirms the expected Thumb-state bit, interrupted Thread context, and absence of stack-alignment padding.

The observed `EXC_RETURN` value matches the previously verified exception-return configuration on this device.

**Result: PASS.**

### 8.7 Terminal Success Indication

After validating the fault identity, address, instruction, stack frame, and processor state, the handler recorded:

```text
Status = 1
Phase  = 4
Unexpected IPSR = 0
```

It then entered the terminal three-flash LD1 indication loop.

The repeating three-flash pattern was observed on the physical board.

The indicator runs in the fault handler and does not return to the faulting thread.

Consequently, the experiment demonstrates fault detection and terminal handling, not recovery or continuation after a fault.

**Result: PASS.**

## 9. Repeatability

The experiment was executed following two hardware resets.

After each execution, the diagnostic record and unprivileged working memory were read through SWD.

The memory contents were identical across the two runs, and the expected repeating three-flash pattern was observed.

No differences were reported in the MPU configuration, fault-status registers, exception frame, or progress marker.

**Repeatability result: PASS — Two matching hardware-reset executions.**

## 10. Security Conclusions

The experiment establishes the following properties on the tested STM32U5A5 Revision X hardware:

- The Cortex-M33 MPU can be configured with separate privileged and unprivileged SRAM access permissions.
- Privileged code retained access to the supervisor sentinel after the MPU was enabled.
- Unprivileged Thread code successfully accessed its authorized working memory.
- An unprivileged read of supervisor-only SRAM was denied.
- The denial produced a precise MemManage data-access violation.
- The fault-status registers identified the expected type and address of violation.
- The hardware-stacked PC identified the designated prohibited load.
- The processor entered the privileged MemManage handler using MSP.
- The handler successfully validated the fault and preserved diagnostic evidence.
- The observed behavior was repeatable across two hardware resets.

These observations provide direct hardware evidence for **CPU-enforced read isolation between the tested unprivileged and supervisor SRAM regions**.

They establish a useful architectural foundation for implementing the project's Platform Integration Layer.

### Properties Not Yet Established

This test does not demonstrate:

- Denial of unprivileged writes to supervisor memory.
- Execute-never enforcement for SRAM.
- Isolation against arbitrary or malicious unprivileged programs.
- Protection against corruption of MPU configuration by compromised privileged code.
- Safe exception recovery or controlled resumption of an unprivileged context.
- Unprivileged access restrictions on memory-mapped peripherals.
- DMA or other bus-master isolation.
- TrustZone Secure/Nonsecure separation.
- Stack overflow prevention or stack-limit enforcement.
- Fault recovery, cancellation, or zeroization.
- Side-channel or physical-fault resistance.
- Production readiness or FIPS 140-3 validation.

The privileged background memory map remained enabled.

Debug access through SWD was available throughout the experiment. The diagnostic records were readable by the external debugger; this is intentional and does not imply that unprivileged CPU code could access the same memory.

The test relies upon a carefully constructed instruction sequence and a statically validated MPU configuration. Its successful outcome does not prove containment for arbitrary software.

## 11. Assurance Summary

| Test objective | Result |
|---|---|
| Privileged Thread startup | PASS |
| MPU type and initial state | PASS |
| MPU configuration and readback | PASS |
| Privileged supervisor SRAM access | PASS |
| Unprivileged Thread execution | PASS |
| Authorized unprivileged SRAM read | PASS |
| Denial of prohibited supervisor SRAM read | PASS |
| MemManage exception identity | PASS |
| Precise data-access violation status | PASS |
| Fault address validation | PASS |
| Faulting instruction validation | PASS |
| Exception frame and stack validation | PASS |
| Terminal success indication | PASS |
| No unexpected exception reported | PASS |
| Repeatability across two hardware resets | PASS |
| Unprivileged write denial | NOT TESTED |
| Execute-never enforcement | NOT TESTED |
| Safe post-fault recovery | NOT TESTED |
| DMA isolation | NOT TESTED |
| TrustZone isolation | NOT TESTED |

**Overall experimental result: PASS.**

## 12. Recommended Follow-Up

Preserve this firmware, its verified programming-image checksum, and the hardware observations as the reference MPU read-denial experiment.

The next experiments should separately verify:

1. Denial of unprivileged writes to supervisor-owned SRAM.
2. Execute-never enforcement for data memory.
3. Controlled MemManage fault recovery, including authorization checks and appropriate treatment of the faulting context.
4. Protection of supervisor-owned configuration and state during more adversarial unprivileged execution.

Each experiment should have independently defined acceptance criteria and preserve a distinct hardware evidence record.

Production PIL code should minimize custom assembly where established and auditable processor abstractions are sufficient.

## 13. References

- [MPU read-denial experimental procedure](../2026-11-23-cortex-m33-mpu-read-denial-procedure.md)
- [MPU experiment Rust implementation](../mpu-read-denial/src/main.rs)
- [MPU experiment assembly](../mpu-read-denial/src/transition.s)
- [MPU experiment memory layout](../mpu-read-denial/memory.x)
- [ELF inspection script](../mpu-read-denial/inspect-elf.py)
- [Previous privilege/SVC experiment record](2026-11-23-cortex-m33-privilege-svc.md)
- [Board firmware and recovery procedure](../FIRMWARE.md)
- [Project architecture](../../../docs/assurance/architecture.md)
- [Armv8-M Architecture Reference Manual](https://developer.arm.com/documentation/ddi0553/latest)
- [ST PM0264 — Cortex-M33 Programming Manual](https://www.st.com/resource/en/programming_manual/pm0264-stm32-cortexm33-mcus-and-mpus-programming-manual-stmicroelectronics.pdf)
- [ST RM0456 — STM32U5 Reference Manual](https://www.st.com/resource/en/reference_manual/rm0456-stm32u5-series-armbased-32bit-mcus-stmicroelectronics.pdf)

---

**Final conclusion:** Two repeatable executions on STM32U5A5 Revision X hardware demonstrated that the configured Cortex-M33 MPU permits authorized unprivileged SRAM access while denying a read from supervisor-owned SRAM. The resulting MemManage exception identified the correct violation, fault address, and instruction. This provides verified hardware evidence for the tested CPU memory-isolation property, without implying comprehensive containment or production-level assurance.
