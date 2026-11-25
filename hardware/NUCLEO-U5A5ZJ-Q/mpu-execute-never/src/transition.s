.syntax unified
.thumb

// User data and a separate, eight-byte-aligned XN instruction buffer.
.section .mpu_user,"aw",%nobits
.balign 8
.global mpu_work
.type mpu_work,%object
mpu_work:
    .space 8
.size mpu_work, .-mpu_work
.balign 8
.global mpu_code
.type mpu_code,%object
mpu_code:
    .space 8
.size mpu_code, .-mpu_code

.section .text.mpu_main,"ax",%progbits
.global main
.type main,%function
.thumb_func
main:
    mov r0, sp
    bl {startup}
    ldr r0, =_psp_top
    msr PSP, r0
    movs r0, #3
    msr CONTROL, r0
    isb
    b mpu_probe
.size main, .-main
.ltorg

// The flash arithmetic is the positive instruction-execution control.
// r12 carries the Thumb target in the hardware exception frame.
.section .text.mpu_probe,"ax",%progbits
.global mpu_probe
.type mpu_probe,%function
.thumb_func
mpu_probe:
    mrs r0, CONTROL
    mrs r1, IPSR
    mov r2, sp
    ldr r4, =mpu_work
    movs r6, #42
    adds r6, #1
    cmp r6, #43
    bne 1f
    ldr r12, =mpu_code+1
    ldr r3, [r12, #-1]
    ldr r5, =0x60202002
    cmp r3, r5
    bne 1f
    ldr r3, [r12, #3]
    ldr r5, =0xBF00E7FE
    cmp r3, r5
    bne 1f
    movs r6, #1
    str r6, [r4, #4]
    dsb
    isb
.global mpu_faulting_branch
mpu_faulting_branch:
    bx r12
    // If XN fails, the SRAM code changes progress to 2 and loops.
1:
    movs r6, #3
    str r6, [r4, #4]
2:
    b 2b
.size mpu_probe, .-mpu_probe
.ltorg

.section .text.mpu_memmanage,"ax",%progbits
.global MemoryManagement
.type MemoryManagement,%function
.thumb_func
MemoryManagement:
    mrs r0, PSP
    mov r1, lr
    mov r2, sp
    mrs r3, MSP
    b {record_memmanage}
.size MemoryManagement, .-MemoryManagement

.section .text.mpu_unexpected,"ax",%progbits
.global HardFault
.global DefaultHandler
.type HardFault,%function
.type DefaultHandler,%function
.thumb_func
HardFault:
.thumb_func
DefaultHandler:
    b {record_unexpected}
.size DefaultHandler, .-DefaultHandler
.size HardFault, .-HardFault
