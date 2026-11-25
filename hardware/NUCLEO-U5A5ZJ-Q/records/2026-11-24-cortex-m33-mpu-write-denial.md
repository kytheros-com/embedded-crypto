# Cortex-M33 MPU Write-Denial Experiment — 2026-11-24

**Project:** embedded-crypto  
**Hardware:** ST NUCLEO-U5A5ZJ-Q, STM32U5A5ZJ Revision X  
**Experiment:** MPU enforcement of privileged-only SRAM write permissions  
**Date:** 2026-11-24  
**Result:** **PASS — Two successful, identical hardware-reset executions**

## 1. Objective

Verify that the Arm Cortex-M33 MPU prevents unprivileged Thread code from modifying supervisor-owned SRAM while permitting authorized writes to unprivileged SRAM.

The experiment tests the following properties:

1. Privileged code can initialize and read a protected sentinel before and after MPU activation.
2. Unprivileged Thread code can write to and read back an authorized SRAM word.
3. An attempted unprivileged store to supervisor-only SRAM generates a precise MemManage data-access violation.
4. The fault handler can identify the instruction, attempted value, and target address.
5. The protected sentinel remains unchanged after the rejected store.
6. The results are repeatable across independent hardware resets.

This experiment extends the successful MPU read-denial experiment of 2026-11-23.

## 2. Firmware and Verification

**Repository:** `kytheros-com/embedded-crypto`  
**Branch:** `hardware/NUCLEO-U5A5ZJ-Q`  
**Source revision at final review:** `9029430013d360d898cda2ed529520220fc19ea1`  
**Application:** `hardware/NUCLEO-U5A5ZJ-Q/mpu-write-denial/`

The firmware was built using Rust 1.98.1 for `thumbv8m.main-none-eabi`, with the release profile and locked dependencies.

The ELF inspection script verified the MPU region boundaries, allocated sections, exception vectors, sentinel placement, stack bounds, and precise instruction encoding of the prohibited store.

Independent Linux VM and macOS builds produced identical Flash payload and programming-image SHA-256 digests.

| Artifact | Size | SHA-256 |
|---|---:|---|
| Flash payload | 2,536 bytes | `22b437421660cbe1eee5df492e5a9e352ec754b7caaa5954536c9926e58b0e27` |
| Programming image | 2,544 bytes | `762d7fdd0987f46a1500939b3039e4b961465566c3b9066abe7379182cf9f611` |
| macOS ELF | 141,308 bytes | `93dbe8d56f2f65019c2fe29f55981697a029e6383e75d57b3447cf2741a1ab25` |

The programming image contains eight trailing `0xFF` padding bytes to satisfy the 16-byte Flash programming granularity.

The Flash payload and programming image matched the independent Linux reference build exactly.

The verified image was programmed at `0x08000000` using STM32CubeProgrammer 2.24.0 and ST-LINK SWD.

## 3. MPU Configuration

The MPU configuration was preserved from the previously successful read-denial experiment.

| Region | Address range | Privileged | Unprivileged |
|---|---|---|---|
| Flash | `0x08000000–0x08001FFF` | RX | RX |
| User SRAM | `0x20001000–0x20001FFF` | RW, XN | RW, XN |
| Supervisor SRAM | `0x20002000–0x20002FFF` | RW, XN | No access |

The hardware reported:

```text
MPU_TYPE = 0x00000800
MPU_CTRL = 0x00000005
MAIR0    = 0x00000044
MAIR1    = 0x00000000

Flash RBAR      = 0x08000006
Flash RLAR      = 0x08001FE1
User RBAR       = 0x20001003
User RLAR       = 0x20001FE1
Supervisor RBAR = 0x20002001
Supervisor RLAR = 0x20002FE1

Unused enabled-region mask = 0
SHCSR = 0x00010000
```

These values match the intended configuration. MemManage faults were explicitly enabled.

## 4. Initial Failure and Correction

The initial firmware programmed and verified successfully, but LD1 remained dark after reset.

The first diagnostic record began:

```text
0x20002000 : 00000081 00000000 00000000 00000000
0x20002010 : 20002FA4 00000000 00000000 00000000
```

The status `0x81` indicated failure of the startup-state validation.

Startup CONTROL and IPSR were both zero, confirming privileged Thread mode without an active exception. However, the recorded MSP was:

```text
0x20002FA4
```

This is 4-byte aligned but not 8-byte aligned.

The original code sampled MSP within the Rust `startup()` function, after its compiler-generated prologue. The firmware incorrectly required 8-byte alignment at that internal instruction point.

AAPCS32 requires 8-byte stack alignment at public function interfaces, but only 4-byte alignment throughout ordinary function execution. The Cortex-M33 does not enforce continuous 8-byte MSP alignment.

Therefore, this was a **test-harness defect**, not an observed MPU failure.

The correction captured MSP immediately before the assembly-to-Rust call:

```asm
main:
    mov r0, sp
    bl {startup}
```

The Rust function declaration was changed to accept the captured value:

```rust
extern "C" fn startup(startup_msp: u32)
```

The existing 8-byte alignment and stack-bounds checks were retained.

The corrected firmware was rebuilt, independently inspected, compared across Linux and macOS, and reprogrammed.

The resulting diagnostic record reported:

```text
Startup MSP = 0x20003000
```

This is the expected, 8-byte-aligned MSP at the function-call boundary.

The initial failure is retained as evidence of a corrected test assumption. No MPU permission changes were necessary.

## 5. Successful Hardware Executions

After installing the corrected firmware, the board displayed the expected repeating **two-flash green LD1 pattern**.

The protected diagnostic record, unprivileged working words, and protected sentinel were read through SWD Hot Plug.

The board was then reset using its physical RESET button, and all three memory reads were repeated.

**Both hardware-reset executions produced identical results.**

### 5.1 Protected Diagnostic Record

The following 184-byte record was obtained in both runs:

```text
Address: 0x20002000
Size:    184 bytes

0x20002000 : 00000001 00000004 00000000 00000000
0x20002010 : 20003000 00000800 00000000 00000005
0x20002020 : 00000044 00000000 08000006 08001FE1
0x20002030 : 20001003 20001FE1 20002001 20002FE1
0x20002040 : 00000000 00000000 00000000 00010000
0x20002050 : 200020B8 53A1CE7D 53A1CE7D 53A1CE7D
0x20002060 : C0DECAFE 53A1CE7D A6074B19 00000001
0x20002070 : 00000001 00000004 20003000 20003000
0x20002080 : FFFFFFBC 20001FE0 00000082 00000000
0x20002090 : 200020B8 080002E4 29000000 00000003
0x200020A0 : 00000000 20002000 C0DECAFE 200020B8
0x200020B0 : F8CC3000 00000000
```

All 46 diagnostic fields matched the acceptance conditions in the experimental procedure.

### 5.2 Unprivileged Working Memory

Both runs produced:

```text
Address: 0x20001000
Size:    8 bytes

0x20001000 : A6074B19 00000001
```

The first word is the value written and read back by unprivileged Thread code.

The second word is the progress marker indicating that the authorized write/readback comparison succeeded before the prohibited store.

The marker remained `1`; it did not advance to the value `2`, which would indicate that the prohibited store completed.

### 5.3 Independent Protected Sentinel Read

An additional direct SWD read of the protected sentinel was performed after each execution.

Both reads returned:

```text
Address: 0x200020B8
Size:    4 bytes

0x200020B8 : 53A1CE7D
```

The sentinel retained its original value.

This agrees with the privileged fault handler's independently recorded post-fault sentinel value.

## 6. Fault Analysis

### 6.1 Authorized Write

The unprivileged Thread executed an ordinary store followed by a readback from authorized SRAM.

The handler subsequently observed:

```text
Authorized word = 0xA6074B19
Progress marker = 0x00000001
```

The expected readback comparison had succeeded before the prohibited store.

**Result: PASS.**

### 6.2 Prohibited Store

The prohibited instruction was:

```asm
str.w r3, [r12]
```

The linked instruction address and encoding were:

```text
PC       = 0x080002E4
Encoding = F8CC 3000
```

The hardware-stacked register values were:

```text
r3  = 0xC0DECAFE
r12 = 0x200020B8
```

Thus, unprivileged code attempted to write `0xC0DECAFE` to the protected sentinel at `0x200020B8`.

The processor reported:

```text
CFSR  = 0x00000082
HFSR  = 0x00000000
MMFAR = 0x200020B8
```

The CFSR value sets:

- `DACCVIOL`: a data-access violation occurred.
- `MMARVALID`: MMFAR contains a valid fault address.

No unrelated CFSR fault-status bits were set, and no HardFault escalation was recorded.

The stacked PC identified the designated prohibited store.

**Result: PASS.**

### 6.3 Protected Sentinel Integrity

The privileged handler recorded:

```text
Initial sentinel     = 0x53A1CE7D
Before MPU           = 0x53A1CE7D
After MPU            = 0x53A1CE7D
After prohibited STR = 0x53A1CE7D
```

The separate SWD read independently corroborated the post-fault value.

The intended prohibited write value, `0xC0DECAFE`, was never observed in the sentinel.

**Result: PASS.**

### 6.4 Exception Handling and Stack State

The handler recorded:

```text
Handler CONTROL = 0x00000001
Handler IPSR    = 0x00000004
Handler SP      = 0x20003000
Handler MSP     = 0x20003000
EXC_RETURN      = 0xFFFFFFBC
Frame PSP       = 0x20001FE0
Stacked xPSR    = 0x29000000
```

These observations agree with execution in privileged MemManage Handler mode using MSP, following interruption of unprivileged Thread code using PSP.

The Thread PSP immediately before exception entry was:

```text
0x20002000
```

The difference between this value and the hardware-stacked frame address is:

```text
0x20002000 - 0x20001FE0 = 32 bytes
```

This matches the expected eight-word basic exception frame, entirely within the authorized user SRAM region.

The handler remained active in its terminal indication loop and did not resume the faulting instruction.

**Result: PASS.**

## 7. Repeatability and Acceptance

The final acceptance criteria were evaluated against both hardware-reset executions.

| Criterion | Result |
|---|---|
| Matching independent Flash payloads | PASS |
| Successful Flash programming and verification | PASS |
| Correct MPU configuration | PASS |
| Privileged sentinel access | PASS |
| Authorized unprivileged write/readback | PASS |
| Deliberate prohibited store identified | PASS |
| Precise MemManage data-access violation | PASS |
| Correct fault address | PASS |
| Correct stacked instruction and operands | PASS |
| No HardFault escalation | PASS |
| Correct exception frame and stack selection | PASS |
| Protected sentinel unchanged | PASS |
| No unexpected exception | PASS |
| Terminal two-flash indication | PASS |
| Identical protected diagnostic records across two resets | PASS |
| Identical user SRAM observations across two resets | PASS |
| Identical independent sentinel observations across two resets | PASS |

**Overall experimental result: PASS.**

## 8. Security Conclusions and Limitations

The experiment establishes, on the tested STM32U5A5 Revision X hardware and MPU configuration, that:

- An authorized unprivileged CPU write to user SRAM completed and was successfully read back.
- An attempted unprivileged CPU store to supervisor-only SRAM generated the expected MemManage data-access violation.
- The fault status and hardware exception frame identified the intended store instruction, source value, and destination address.
- The protected sentinel remained unchanged.
- The observations were repeatable across two hardware resets.

Together with the previous read-denial experiment, this provides hardware evidence for both read and write isolation of the tested supervisor-owned SRAM locations.

The result does **not** establish comprehensive containment of arbitrary unprivileged software. In particular, it does not demonstrate DMA isolation, protection against privileged-code compromise, execute-never enforcement, peripheral isolation, stack-overflow protection, or recovery from a MemManage exception.

The experiment uses a terminal fault handler rather than returning to unprivileged execution. TrustZone and irreversible security settings remained outside the experimental scope.

## 9. Next Experiment

The next recommended hardware experiment is verification of MPU execute-never enforcement for SRAM.

That experiment should use a separate standalone application and independently validate:

- Successful execution from an authorized executable memory region.
- A deliberate instruction-fetch attempt from an execute-never SRAM region.
- The expected instruction-access violation fault status.
- The stacked fault context and destination address.
- Safe terminal handling and repeatability across hardware resets.

The read-denial and write-denial applications should be preserved unchanged as successful reference experiments.

---

**Final disposition:** The Cortex-M33 MPU write-denial experiment passed. Two independent hardware-reset executions demonstrated that the configured MPU denied the designated unprivileged store to supervisor-only SRAM while preserving the protected sentinel. The initial startup-alignment test-harness defect was identified, corrected, and retained in the record as part of the engineering evidence.
