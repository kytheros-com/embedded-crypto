.syntax unified
.thumb

// Progress, continuation, positive/fault values, failure; then pattern.
.section .mpu_user,"aw",%nobits
.balign 8
.global mpu_work
.type mpu_work,%object
mpu_work:
    .space 20
.size mpu_work, .-mpu_work
.space 12
.global mpu_pattern
.type mpu_pattern,%object
mpu_pattern:
    .space 12
.size mpu_pattern, .-mpu_pattern

.section .text.flt_main,"ax",%progbits
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
    b fault_probe
.size main, .-main
.ltorg

.section .text.flt_usagefault,"ax",%progbits
.global UsageFault
.type UsageFault,%function
.thumb_func
UsageFault:
    mrs r0, PSP
    mov r1, lr
    mov r2, sp
    mrs r3, MSP
    // Check the separate PSP frame and handler MSP before Rust stack use.
    ldr r12, =0x20001FE0
    cmp r0, r12
    bne entry_failure
    ldr r12, =0x20003000
    cmp r2, r12
    bne entry_failure
    cmp r3, r12
    bne entry_failure
    ldr r12, =0xFFFFFFBC
    cmp r1, r12
    bne entry_failure
    b {record_usagefault}
entry_failure:
    // No verified software stack; Status remains non-success.
    b entry_failure
.size UsageFault, .-UsageFault
.ltorg

.section .text.flt_unexpected,"ax",%progbits
.global HardFault
.global MemoryManagement
.global BusFault
.global DefaultHandler
.type HardFault,%function
.type MemoryManagement,%function
.type BusFault,%function
.type DefaultHandler,%function
.thumb_func
HardFault:
.thumb_func
MemoryManagement:
.thumb_func
BusFault:
.thumb_func
DefaultHandler:
    b {record_unexpected}
.size DefaultHandler, .-DefaultHandler
.size BusFault, .-BusFault
.size MemoryManagement, .-MemoryManagement
.size HardFault, .-HardFault
