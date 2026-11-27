.syntax unified
.thumb

// Progress, continuation, positive/zero quotients, failure; then PSP.
.section .mpu_user,"aw",%nobits
.balign 8
.global mpu_work
.type mpu_work,%object
mpu_work:
    .space 20
.size mpu_work, .-mpu_work

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

// All division operands and both UDIV instructions are explicit assembly.
.section .text.flt_probe,"ax",%progbits
.global fault_probe
.type fault_probe,%function
.thumb_func
fault_probe:
    mrs r0, CONTROL
    mrs r1, IPSR
    mov r2, sp
    ldr r4, =mpu_work
    cmp r0, #3
    bne probe_failure
    cmp r1, #0
    bne probe_failure
    ldr r5, =_psp_top
    cmp r2, r5
    bne probe_failure
    movs r3, #42
    mov.w r12, #7
.global positive_udiv
positive_udiv:
    udiv r6, r3, r12
    cmp r6, #6
    bne probe_failure
    str r6, [r4, #8]
    movs r3, #42
    mov.w r12, #0
    movs r6, #0x5A
    movs r5, #1
    str r5, [r4]
    dsb
    isb
.global faulting_udiv
faulting_udiv:
    udiv r6, r3, r12
.global unexpected_continuation
unexpected_continuation:
    str r6, [r4, #12]
    dsb
    movs r5, #2
    str r5, [r4, #4]
    dsb
1:
    b 1b
probe_failure:
    movs r5, #3
    str r5, [r4, #16]
    dsb
2:
    b 2b
.size fault_probe, .-fault_probe
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
