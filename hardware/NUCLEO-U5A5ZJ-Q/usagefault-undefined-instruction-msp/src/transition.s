.syntax unified
.thumb

// Probe progress and unexpected-continuation words; PSP stays unused.
.section .mpu_user,"aw",%nobits
.balign 8
.global mpu_work
.type mpu_work,%object
mpu_work:
    .space 8
.size mpu_work, .-mpu_work

.section .text.flt_main,"ax",%progbits
.global main
.type main,%function
.thumb_func
main:
    mov r0, sp
    bl {startup}
    // BL has returned: startup's software frame has been released.
    ldr r0, =_stack_start
    msr MSP, r0
    movs r0, #0
    msr CONTROL, r0
    isb
    b fault_probe
.size main, .-main
.ltorg

// Privileged MSP, Flash arithmetic, and SRAM store are positive controls.
.section .text.flt_probe,"ax",%progbits
.global fault_probe
.type fault_probe,%function
.thumb_func
fault_probe:
    mrs r0, CONTROL
    mrs r1, IPSR
    mov r2, sp
    ldr r4, =mpu_work
    movs r6, #42
    adds r6, #1
    cmp r6, #43
    bne probe_failure
    cmp r0, #0
    bne probe_failure
    cmp r1, #0
    bne probe_failure
    ldr r5, =_stack_start
    cmp r2, r5
    bne probe_failure
    movs r6, #1
    str r6, [r4]
    dsb
    isb
.global faulting_udf
faulting_udf:
    udf #0x42
.global unexpected_continuation
unexpected_continuation:
    movs r6, #2
    str r6, [r4, #4]
1:
    b 1b
probe_failure:
    movs r6, #3
    str r6, [r4, #4]
2:
    b 2b
.size fault_probe, .-fault_probe
.ltorg

.section .text.flt_usagefault,"ax",%progbits
.global UsageFault
.type UsageFault,%function
.thumb_func
UsageFault:
    mrs r0, MSP
    mov r1, lr
    mov r2, sp
    mrs r3, MSP
    // No push, call, or memory write before validating the shared stack.
    ldr r12, =0x20002FE0
    cmp r0, r12
    bne entry_failure
    cmp r2, r12
    bne entry_failure
    cmp r3, r12
    bne entry_failure
    ldr r12, =0xFFFFFFB8
    cmp r1, r12
    bne entry_failure
    b {record_usagefault}
entry_failure:
    // Phase remains 3 and Status remains 0; no unverified stack use.
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
