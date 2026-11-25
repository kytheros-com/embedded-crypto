# Cortex-M33 MPU Execute-Never Experiment — Hardware Evidence

**Experiment:** MPU execute-never enforcement for unprivileged instruction fetches  
**Hardware test date:** 2026-11-24  
**Procedure:** `2026-11-25-cortex-m33-mpu-execute-never-procedure.md`  
**Result:** **PASS — Two successful, identical hardware-reset executions**

## 1. Objective

Verify that the Arm Cortex-M33 MPU prevents unprivileged Thread code from executing instructions in an execute-never SRAM region while allowing authorized data access to that same region.

The experiment tests five principal properties:

1. Unprivileged Thread code can execute instructions from authorized Flash.
2. Unprivileged Thread code can read valid Thumb instructions stored in user SRAM.
3. Attempted execution of those SRAM instructions causes a MemManage instruction-access violation.
4. The processor reports the expected fault status and exception context, and the SRAM payload's failure marker remains unchanged.
5. The results are repeatable across hardware resets.

This experiment extends the successful MPU read-denial and write-denial experiments.

## 2. Hardware and Software Configuration

**Board:** ST NUCLEO-U5A5ZJ-Q  
**MCU:** STM32U5A5ZJ, Cortex-M33, Revision X  
**Device ID:** `0x481`  
**ST-LINK firmware:** `V3J10M3`  
**STM32CubeProgrammer:** `v2.24.0`  
**Observed supply voltage:** `3.27 V`  
**SWD frequency:** `1000 kHz`  
**Diagnostic connection mode:** Hot Plug  

**Repository:** `kytheros-com/embedded-crypto`  
**Branch:** `hardware/NUCLEO-U5A5ZJ-Q`  
**Implementation commit:** `b65940fc76b5c057748ed55232d89aecb092b990`  
**Application:** `hardware/NUCLEO-U5A5ZJ-Q/mpu-execute-never/`

The verified firmware was programmed at Flash address `0x08000000`.

### Binary verification

The independent Linux and macOS builds produced identical Flash payloads and programming images.

| Artifact | Size | SHA-256 |
|---|---:|---|
| Flash payload | 2,604 bytes | `35a1c5ad9e61f06b7f93760fb584a60b72c0d48843b6e98cf6a4d36f58680bae` |
| Programming image | 2,608 bytes | `837e6c67c6c51fad89b9b36f1f48e46676456f589a7543c6a22de9a08e8c8538` |
| macOS ELF | 141,308 bytes | `8b3e2cca43e3df5c6cb0b1fd373b947ddef4644cccf06338b19f256e53dfecd9` |

The four additional programming-image bytes are `0xFF` padding.

The Python ELF inspector passed on macOS and confirmed:

```text
flash: [0x08000000, 0x08002000)
  RBAR=0x08000006 RLAR=0x08001fe1

user: [0x20001000, 0x20002000)
  RBAR=0x20001003 RLAR=0x20001fe1

supervisor: [0x20002000, 0x20003000)
  RBAR=0x20002001 RLAR=0x20002fe1

mpu_results: 0x20002000, 168 bytes

entry     = 0x08000259
MemManage = 0x08000315
branch    = 0x080002fc
target    = 0x20001009

SRAM Thumb halfwords: 2002 6020 E7FE BF00
```

The deployed Flash bytes were therefore identical to the independently inspected reference image.

## 3. MPU Configuration

The experiment retained the previously validated three-region MPU configuration.

| Region | Address range | Privileged | Unprivileged |
|---|---|---|---|
| Flash | `0x08000000–0x08001FFF` | RX | RX |
| User SRAM | `0x20001000–0x20001FFF` | RW, XN | RW, XN |
| Supervisor SRAM | `0x20002000–0x20002FFF` | RW, XN | No access |

The diagnostic record confirms:

```text
MPU_TYPE     = 0x00000800
Initial CTRL = 0x00000000
Enabled CTRL = 0x00000005
MAIR0        = 0x00000044
MAIR1        = 0x00000000
Unused mask  = 0x00000000
SHCSR        = 0x00010000
```

All three RBAR/RLAR pairs matched their expected values. Regions 3–7 were disabled.

MemManage exceptions were enabled before entering unprivileged Thread mode.

## 4. Test Instruction Sequence

The authorized user SRAM region contains two control words followed by an eight-byte instruction buffer.

| Address | Purpose |
|---|---|
| `0x20001000` | Failure marker |
| `0x20001004` | Positive-control progress marker |
| `0x20001008` | Start of execute-never instruction buffer |

The instruction buffer contains the following Thumb halfwords:

```text
Address       Halfword  Instruction
0x20001008    0x2002    MOVS r0, #2
0x2000100A    0x6020    STR r0, [r4]
0x2000100C    0xE7FE    B .
0x2000100E    0xBF00    NOP
```

If execution were permitted, the payload would write `2` to the failure marker at `0x20001000` and enter an infinite loop.

The Flash-based positive control first executed arithmetic instructions and read back the two SRAM instruction words. After the comparisons succeeded, it wrote `1` to the progress marker.

The probe then executed `DSB`, `ISB`, and:

```asm
bx r12
```

The branch operand was:

```text
r12 = 0x20001009
```

Bit 0 specifies Thumb state. The corresponding instruction-fetch address is `0x20001008`.

The MPU was expected to reject the instruction fetch before the SRAM payload could modify the failure marker.

## 5. Hardware Execution

The verified firmware was flashed successfully, and the board was reset.

The board displayed the expected **repeating four-flash green LD1 pattern**.

The following observations were collected using SWD Hot Plug:

- The protected 168-byte diagnostic record at `0x20002000`.
- The 16-byte user SRAM control and instruction area at `0x20001000`.

The board was then reset again, and both reads were repeated.

The **four-flash LD1 pattern was observed after both resets**.

Both protected diagnostic records were identical, and both user SRAM records were identical.

### 5.1 First hardware execution — Protected diagnostic record

```text
Address: 0x20002000
Size:    168 bytes

0x20002000 : 00000001 00000004 00000000 00000000
0x20002010 : 20003000 00000800 00000000 00000005
0x20002020 : 00000044 00000000 08000006 08001FE1
0x20002030 : 20001003 20001FE1 20002001 20002FE1
0x20002040 : 00000000 00000000 00000000 00010000
0x20002050 : 20001008 60202002 BF00E7FE 00000001
0x20002060 : 00000000 00000001 00000004 20003000
0x20002070 : 20003000 FFFFFFBC 20001FE0 00000001
0x20002080 : 00000000 00000000 20001008 29000000
0x20002090 : 00000003 00000000 20002000 20001009
0x200020A0 : 00004760 00000000
```

### 5.2 First hardware execution — User SRAM

```text
Address: 0x20001000
Size:    16 bytes

0x20001000 : 00000000 00000001 60202002 BF00E7FE
```

### 5.3 Second hardware execution — Protected diagnostic record

After the second hardware reset:

```text
Address: 0x20002000
Size:    168 bytes

0x20002000 : 00000001 00000004 00000000 00000000
0x20002010 : 20003000 00000800 00000000 00000005
0x20002020 : 00000044 00000000 08000006 08001FE1
0x20002030 : 20001003 20001FE1 20002001 20002FE1
0x20002040 : 00000000 00000000 00000000 00010000
0x20002050 : 20001008 60202002 BF00E7FE 00000001
0x20002060 : 00000000 00000001 00000004 20003000
0x20002070 : 20003000 FFFFFFBC 20001FE0 00000001
0x20002080 : 00000000 00000000 20001008 29000000
0x20002090 : 00000003 00000000 20002000 20001009
0x200020A0 : 00004760 00000000
```

### 5.4 Second hardware execution — User SRAM

```text
Address: 0x20001000
Size:    16 bytes

0x20001000 : 00000000 00000001 60202002 BF00E7FE
```

**Repeatability result:** All 42 protected diagnostic words and all four user SRAM words were identical across the two executions.

## 6. Fault Analysis

### 6.1 Positive controls

The recorded user SRAM values were:

```text
Failure marker  = 0x00000000
Progress marker = 0x00000001
Instruction 0   = 0x60202002
Instruction 1   = 0xBF00E7FE
```

The progress marker confirms successful completion of the Flash arithmetic control and SRAM instruction-word comparisons.

The unprivileged Thread could read the SRAM instruction bytes as ordinary data.

The failure marker remained zero, showing that the SRAM payload's intended store did not complete.

**Result: PASS.**

### 6.2 Instruction-access violation

The fault status was:

```text
CFSR  = 0x00000001
HFSR  = 0x00000000
MMFAR = unavailable (recorded as zero)
```

CFSR bit 0 (`IACCVIOL`) was set, indicating a MemManage instruction-access violation.

No additional CFSR fault-status bits were set, and no HardFault escalation was recorded.

The `MMARVALID` bit was clear. Accordingly, MMFAR was not treated as a valid fault-address source.

Instead, the handler validated the hardware-stacked PC:

```text
Stacked PC = 0x20001008
```

This matches the start of the execute-never instruction buffer.

The stacked branch operand was:

```text
Stacked r12 = 0x20001009
```

The handler also recorded the Flash branch opcode:

```text
0x4760  (BX r12)
```

Together, these observations establish that the processor attempted the intended Thumb branch and generated an instruction-access violation at the target SRAM instruction address.

**Result: PASS.**

### 6.3 Exception context and stack selection

The protected record contained:

```text
Handler CONTROL = 0x00000001
Handler IPSR    = 0x00000004
Handler SP      = 0x20003000
Handler MSP     = 0x20003000

EXC_RETURN      = 0xFFFFFFBC
Frame PSP       = 0x20001FE0

Stacked xPSR    = 0x29000000
Thread CONTROL  = 0x00000003
Thread IPSR     = 0x00000000
Thread SP       = 0x20002000
```

These values agree with a MemManage exception interrupting unprivileged Thread execution using PSP and entering privileged Handler mode using MSP.

The exception frame occupied:

```text
0x20002000 - 0x20001FE0 = 32 bytes
```

This is the expected basic eight-word hardware exception frame.

The stacked xPSR has the Thumb-state bit set and reports no active Thread exception.

The corrected startup MSP capture also succeeded:

```text
Startup MSP = 0x20003000
```

The fault handler remained active in its terminal indication loop and did not resume execution at the faulting SRAM address.

**Result: PASS.**

## 7. Acceptance Results

| Acceptance criterion | Result |
|---|---|
| Independent Flash image reproducibility | PASS |
| ELF structural verification | PASS |
| Expected MPU region configuration | PASS |
| Correct startup MSP alignment | PASS |
| Authorized unprivileged Flash execution | PASS |
| Authorized unprivileged SRAM data reads | PASS |
| Correct SRAM Thumb instruction bytes | PASS |
| Valid Thumb-state branch operand | PASS |
| MemManage exception number 4 | PASS |
| CFSR exactly `0x00000001` | PASS |
| HFSR zero | PASS |
| MMARVALID clear | PASS |
| Stacked PC at intended SRAM target | PASS |
| Stacked r12 contains correct branch operand | PASS |
| Correct `BX r12` opcode | PASS |
| Correct PSP/MSP exception behavior | PASS |
| Positive-control progress marker remains `1` | PASS |
| SRAM execution failure marker remains `0` | PASS |
| Terminal status/phase `1`/`4` | PASS |
| Four-flash LD1 pattern | PASS |
| Identical diagnostics after second reset | PASS |
| Identical user SRAM after second reset | PASS |
| Four-flash LD1 pattern after second reset | PASS |

**Overall experimental result: PASS.**

## 8. Security Conclusions

The experiment provides repeatable hardware evidence that the configured Cortex-M33 MPU rejects unprivileged instruction fetches from the tested execute-never SRAM region.

The result is supported by mutually consistent observations:

1. Authorized Flash instructions executed successfully.
2. The SRAM instruction buffer was accessible through authorized data reads.
3. The attempted branch used a valid Thumb-state target.
4. The hardware-stacked PC identified the intended SRAM instruction address.
5. The processor reported `IACCVIOL`, rather than a data-access or instruction-state fault.
6. The SRAM payload did not modify its failure marker.
7. Both hardware executions produced identical records and the expected LED indication.

Together with the completed read-denial and write-denial experiments, this establishes three independently tested MPU protection properties:

| Property | Hardware result |
|---|---|
| Denial of unauthorized CPU data reads | PASS |
| Denial of unauthorized CPU data writes | PASS |
| Denial of CPU instruction fetches from XN SRAM | PASS |

These are results for the tested MPU configuration, selected accesses, and STM32U5A5 Revision X hardware.

They do **not** constitute a proof of arbitrary-code containment, complete memory isolation, or resistance to privileged-code compromise. They also do not establish DMA isolation, peripheral isolation, TrustZone enforcement, behavior under all exception and interrupt conditions, or compliance with FIPS 140-3.

The experiment deliberately terminates in its privileged MemManage handler. It does not test recovery and resumption of unprivileged execution after a memory-protection fault.

## 9. Disposition and Next Steps

The MPU execute-never experiment passed its static and hardware acceptance criteria.

The firmware should be retained unchanged as a reference test, together with its procedure and the raw hardware evidence.

The next recommended investigation is controlled recovery from a MemManage exception, including preservation of the protected execution environment, cleanup of the faulting context, and prevention of unauthorized privilege transitions.

Such recovery should be developed and evaluated in a separate standalone experiment rather than modifying the successful read-denial, write-denial, or execute-never applications.

---

**Final disposition: PASS.** Two hardware-reset executions demonstrated that the Cortex-M33 MPU denied the designated unprivileged SRAM instruction fetch, raised the expected MemManage instruction-access violation, preserved the failure marker, and entered the intended privileged terminal handler. Both executions produced identical SWD diagnostic records and the expected four-flash LED indication.
