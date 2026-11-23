# Cortex-M33 Privilege and SVCall Experiment — 2026-11-23

**Project:** Kytheros Embedded Cryptography  
**Hardware:** ST NUCLEO-U5A5ZJ-Q  
**Experiment:** Privilege transitions, SVCall exception entry, and controlled privilege restoration  
**Date:** 2026-11-23  
**Overall result:** PASS — Two consecutive hardware-reset executions produced identical diagnostic records.

## 1. Objective

Verify the Cortex-M33 privilege and exception mechanisms intended to support the Kytheros Platform Integration Layer (PIL).

Specifically, demonstrate:

1. Reset-time execution in privileged Thread mode using MSP.
2. Deliberate transition to unprivileged Thread mode using PSP.
3. Inability of unprivileged Thread code to restore privilege through a direct write to `CONTROL`.
4. Privileged Handler-mode execution in response to an `SVC` instruction.
5. Correct exception stacking and preservation of the exception-return mechanism.
6. Authorized restoration of privileged Thread execution through the SVCall handler.
7. Successful continuation of execution on PSP following exception return.

The experiment is a processor-behavior test, not a demonstration of memory isolation or a complete security boundary.

## 2. Hardware and Software Configuration

| Property | Value |
|---|---|
| Development board | NUCLEO-U5A5ZJ-Q |
| Product identifier | `NUU5A5ZJQ$AT2` |
| Board assembly | `MB1549-U5A5ZJQ-C04` |
| MCU | STM32U5A5ZJ |
| Silicon revision | X |
| Processor | Arm Cortex-M33 |
| Device ID | `0x481` |
| TrustZone | Disabled (`TZEN=0` baseline) |
| Readout protection | Level 0 baseline |
| MPU | Disabled; eight regions implemented |
| Firmware target | `thumbv8m.main-none-eabi` |
| Rust toolchain | 1.98.1 |
| Programming/debug interface | STLINK-V3E, SWD |
| Programmer | STM32CubeProgrammer 2.24.0 |
| Firmware source | `hardware/NUCLEO-U5A5ZJ-Q/privilege-svc/` |

The firmware was developed and built on the GCP development VM. Hardware deployment and observation were performed using the separately maintained Mac development environment.

The experiment does not intentionally configure the MPU, GTZC, DMA, TrustZone, watchdogs, or security option bytes.

### Reference Build Artifacts

The Codex build procedure records the following reference artifacts:

| Artifact | Recorded value |
|---|---|
| ELF SHA-256 | `6bdbc21ce145d553d5a71e8f81b25d42efce3f628a995de74479f246af355406` |
| Unpadded Flash payload SHA-256 | `5fee694e0b21d59135b536c17c6190f1facac67b865e50ea0acf9a31a8ff0588` |
| Flash payload length | 1,524 bytes |
| Flash origin | `0x08000000` |
| Initial MSP | `0x200C0000` |
| Dedicated PSP top | `0x20000858` |
| Results structure | `0x20000000`, 84 bytes |

These are the reference build values documented before deployment. The hardware observations below independently demonstrate successful execution of the expected instruction sequence. The exact deployed source commit and final programming-image hash should be added to the provenance record when independently confirmed.

## 3. Experimental Procedure

The firmware was deployed to the STM32U5A5 and started using the development board's hardware RESET button.

After execution, the diagnostic structure was read through a nonintrusive SWD Hot Plug connection.

Command:

```sh
stm32cli \
  -c port=SWD mode=HOTPLUG freq=1000 \
  -r32 0x20000000 0x54
```

The read covers 84 bytes (21 consecutive 32-bit words).

The board was then reset a second time, and the same memory region was read again.

**Both executions produced identical 84-byte diagnostic records.**

The distinctive green LED success pattern continued operating.

The LED timing was assessed visually rather than measured with external instrumentation.

## 4. Raw Hardware Observations

The following output was obtained during the first run and reproduced identically following the second hardware reset.

```text
Reading 32-bit memory content

Size     : 84 Bytes
Address  : 0x20000000

0x20000000 : 00000005 00000000 00000000 200C0000
0x20000010 : 00000003 00000000 20000858 00000003
0x20000020 : 00000001 0000000B 200C0000 200C0000
0x20000030 : FFFFFFBC 20000838 0800033E 29000000
0x20000040 : 0000DF35 00000002 00000000 20000858
0x20000050 : 00000000
```

## 5. Decoded Diagnostic Results

All values are 32-bit words. Offsets are relative to `experiment_results` at `0x20000000`.

| Offset | Observation | Measured | Result |
|---|---|---|---|
| `0x00` | Final status | `0x00000005` | PASS |
| `0x04` | Startup `CONTROL` | `0x00000000` | PASS |
| `0x08` | Startup `IPSR` | `0x00000000` | PASS |
| `0x0C` | Startup stack pointer | `0x200C0000` | PASS |
| `0x10` | Unprivileged `CONTROL` | `0x00000003` | PASS |
| `0x14` | Unprivileged `IPSR` | `0x00000000` | PASS |
| `0x18` | Unprivileged stack pointer | `0x20000858` | PASS |
| `0x1C` | `CONTROL` after direct write | `0x00000003` | PASS |
| `0x20` | SVCall handler `CONTROL` | `0x00000001` | PASS |
| `0x24` | SVCall handler `IPSR` | `0x0000000B` | PASS |
| `0x28` | Handler active stack pointer | `0x200C0000` | PASS |
| `0x2C` | Handler MSP | `0x200C0000` | PASS |
| `0x30` | `EXC_RETURN` | `0xFFFFFFBC` | PASS |
| `0x34` | PSP during handler execution | `0x20000838` | PASS |
| `0x38` | Stacked return PC | `0x0800033E` | PASS |
| `0x3C` | Stacked `xPSR` | `0x29000000` | PASS |
| `0x40` | SVC instruction encoding | `0x0000DF35` | PASS |
| `0x44` | Returned Thread `CONTROL` | `0x00000002` | PASS |
| `0x48` | Returned Thread `IPSR` | `0x00000000` | PASS |
| `0x4C` | Returned Thread stack pointer | `0x20000858` | PASS |
| `0x50` | Unexpected exception number | `0x00000000` | PASS |

The final status of `5` indicates that the firmware completed its expected sequence and passed its internal checks. The other diagnostic words provide independent evidence of the relevant processor states.

## 6. Analysis

### 6.1 Privileged Startup

The initial processor state was:

```text
CONTROL = 0x00000000
IPSR    = 0x00000000
SP      = 0x200C0000
```

These values establish that the application began in privileged Thread mode using MSP.

The stack pointer matches the exclusive upper boundary of the configured 768 KiB SRAM1 region.

**Result: PASS.**

### 6.2 Transition to Unprivileged Thread Mode

Following the deliberate privilege transition, the recorded values were:

```text
CONTROL = 0x00000003
IPSR    = 0x00000000
SP      = 0x20000858
```

For `CONTROL = 3`:

- Bit 0 (`nPRIV`) is set: Thread mode is unprivileged.
- Bit 1 (`SPSEL`) is set: Thread mode uses PSP.

The zero `IPSR` confirms execution in Thread mode.

The stack pointer matches the top of the dedicated 2 KiB PSP storage.

**Result: PASS.**

### 6.3 Attempted Direct Privilege Restoration

While executing in unprivileged Thread mode, the firmware attempted to write `CONTROL = 2`, clearing `nPRIV`.

The subsequent read returned:

```text
CONTROL = 0x00000003
```

The processor ignored the attempted write, leaving Thread mode unprivileged.

This demonstrates the architecturally specified restriction on direct modification of `CONTROL` from unprivileged execution.

No processor fault was required or observed.

**Result: PASS.**

### 6.4 SVCall Exception Entry

The unprivileged test executed:

```asm
svc #0x35
```

The diagnostic record contains the instruction encoding:

```text
0xDF35
```

Inside the exception handler:

```text
CONTROL = 0x00000001
IPSR    = 0x0000000B
SP      = 0x200C0000
MSP     = 0x200C0000
```

The `IPSR` value of 11 identifies the SVCall exception.

The handler executes in privileged Handler mode using MSP, even though the recorded `CONTROL.nPRIV` value remains set.

This demonstrates the distinction between Thread-mode privilege configuration and the processor's privileged exception-handler execution.

**Result: PASS.**

### 6.5 Exception Frame and Return State

The recorded exception-return information was:

```text
EXC_RETURN = 0xFFFFFFBC
Frame PSP  = 0x20000838
Stacked PC = 0x0800033E
Stacked xPSR = 0x29000000
SVC opcode = 0x0000DF35
```

The difference between the pre-exception PSP and stacked-frame PSP is:

```text
0x20000858 - 0x20000838 = 0x20 = 32 bytes
```

This agrees with the eight-word basic exception frame expected by the experiment.

The stacked return PC points to the instruction immediately following the SVC instruction at `0x0800033C`.

The stacked `xPSR` has its Thumb-state bit set and indicates an interrupted Thread-mode context. The stack-alignment indicator is clear, consistent with the observed frame size.

The complete hardware-generated `EXC_RETURN` value was `0xFFFFFFBC`.

This satisfies the firmware's documented check:

```text
(EXC_RETURN & ~0x41) == 0xFFFFFFBC
```

The observation is consistent with basic-frame exception return to Thread mode using PSP under this board's TrustZone-disabled configuration.

**Result: PASS.**

### 6.6 Controlled Restoration of Privileged Thread Execution

The privileged SVCall handler deliberately cleared `CONTROL.nPRIV` before returning through the hardware exception-return mechanism.

Following exception return, the firmware recorded:

```text
CONTROL = 0x00000002
IPSR    = 0x00000000
SP      = 0x20000858
```

These values establish that execution resumed in:

- Privileged Thread mode (`nPRIV = 0`).
- Thread mode rather than Handler mode (`IPSR = 0`).
- PSP-based execution (`SPSEL = 1`).

The processor successfully returned from the exception and continued executing the application's success-indication loop.

**Result: PASS.**

### 6.7 Unexpected Exception Handling

The diagnostic field reserved for unexpected exception identity contained:

```text
FAULT_IPSR = 0x00000000
```

The firmware did not report entry into its unexpected-exception handler.

This result is consistent with the successful execution of the expected sequence. It is not an exhaustive test of fault handling.

**Result: PASS.**

## 7. External Execution Observation

Following deployment and hardware reset, the green user LED LD1 displayed the expected repeating two-pulse pattern.

The pattern continued while the diagnostic registers were inspected through SWD.

After a second hardware reset, the same pattern was observed, and the complete diagnostic structure matched the first run.

This provides repeatability evidence for two consecutive executions from hardware reset.

The nominal success pattern is derived from a 4 MHz clock assumption and polled SysTick delays. No oscilloscope, logic analyzer, or other independent timing measurement was performed.

**Execution and repeatability: PASS.**

## 8. Documentation Discrepancy

During implementation, Codex identified a discrepancy in the description of `CONTROL` register access.

ST PM0264 Revision 8 summarizes `CONTROL` as requiring privileged access, without clearly separating read and write permissions.

The Armv8-M Architecture Reference Manual specifies that unprivileged code may read `CONTROL`, while attempted writes by unprivileged code are ignored.

The experiment followed the Arm architectural definition.

The observed values confirm the expected behavior for the tested read and write operations on this STM32U5A5 Revision X device.

This discrepancy should remain documented when implementing and reviewing future PIL privilege-management code.

## 9. Security Conclusions and Limitations

The experiment provides direct hardware evidence that the tested Cortex-M33 privilege mechanisms behave as expected.

**Established properties:**

- Privileged Thread execution can transition into unprivileged Thread execution.
- Separate MSP and PSP stacks can support the tested transition.
- Unprivileged Thread code cannot directly clear `CONTROL.nPRIV`.
- SVC transfers execution to privileged Handler mode.
- The handler can inspect the exception frame and associated processor state.
- Trusted handler code can restore privileged Thread execution through the tested exception-return path.
- The implemented transition sequence completed successfully and repeatably.

**Properties not established:**

- Protection of privileged SRAM from unprivileged code.
- Isolation of primitive working buffers, stacks, or executable memory.
- Protection of supervisor authorization records against malicious modification.
- Enforcement of a secure SVC authorization policy.
- Isolation of memory-mapped peripherals.
- DMA or other bus-master isolation.
- TrustZone Secure/Nonsecure transitions.
- Fault containment, recovery, or zeroization.
- Resistance to physical fault injection or side-channel attacks.

In particular, the MPU remained disabled throughout the experiment. Unprivileged code could therefore access ordinary SRAM containing experimental state and other unprotected data.

The SVC handler's call-site and phase checks validate the cooperative execution sequence, but do not yet constitute a hardware-enforced authorization boundary.

The successful experiment must not be interpreted as proof that untrusted or malicious cryptographic primitive code is contained.

## 10. Assurance Summary

| Test objective | Result |
|---|---|
| Privileged Thread startup | PASS |
| MSP identification | PASS |
| PSP initialization and selection | PASS |
| Unprivileged Thread execution | PASS |
| Direct privilege-restoration restriction | PASS |
| SVC privileged Handler entry | PASS |
| MSP use by exception handler | PASS |
| Exception-frame structure | PASS |
| SVC instruction/call-site consistency | PASS |
| Controlled privileged Thread restoration | PASS |
| Normal execution following exception return | PASS |
| Unexpected-exception check | PASS |
| LED success indication | PASS |
| Repeatability across two hardware resets | PASS |
| Enforced memory isolation | NOT TESTED |
| Secure/Nonsecure isolation | NOT TESTED |
| DMA isolation | NOT TESTED |

**Overall experimental result: PASS.**

## 11. Recommended Follow-Up

The next experiment should introduce the Cortex-M33 MPU while retaining the established privilege-transition mechanism.

The initial MPU test should demonstrate that unprivileged code can access explicitly permitted memory but cannot read or write privileged-only memory.

It should also establish controlled MemManage fault reporting, safe exception handling, and protection of privileged supervisor state.

The existing assembly-based experiment should be preserved as a low-level architectural reference. Future production PIL implementations should preferentially use established Rust abstractions and minimize custom assembly where practical.

## 12. References

- [Armv8-M Architecture Reference Manual](https://documentation-service.arm.com/static/5f8f019ef86e16515cdbe64c)
- [ST PM0264 — Cortex-M33 Programming Manual](https://www.st.com/resource/en/programming_manual/pm0264-stm32-cortexm33-mcus-and-mpus-programming-manual-stmicroelectronics.pdf)
- [ST RM0456 — STM32U5 Reference Manual](https://www.st.com/resource/en/reference_manual/rm0456-stm32u5-series-armbased-32bit-mcus-stmicroelectronics.pdf)
- [ST ES0553 — STM32U59xxx/U5Axxx Errata](https://www.st.com/content/st_com/en/technical-documents/es0553-stm32u59xxx-and-stm32u5axxx-device-errata-stmicroelectronics.html)
- `hardware/NUCLEO-U5A5ZJ-Q/2026-11-23-cortex-m33-privilege-svc-procedure.md`
- `hardware/NUCLEO-U5A5ZJ-Q/privilege-svc/src/transition.s`
- `hardware/NUCLEO-U5A5ZJ-Q/privilege-svc/src/main.rs`
- `hardware/NUCLEO-U5A5ZJ-Q/records/2026-11-21-initial-firmware-deployment.md`
- `docs/assurance/architecture.md`

---

**Final conclusion:** The STM32U5A5 Revision X Cortex-M33 successfully demonstrated the expected Thread/Handler privilege transitions, stack selection, exception stacking, and SVC-mediated privilege restoration in two repeatable hardware runs. This establishes a verified processor-mechanism baseline for subsequent PIL memory-isolation experiments, without yet establishing a security boundary.
