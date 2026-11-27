.syntax unified
.thumb
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
    ldr r5, =invalid_target + 1
    ldr lr, =positive_return + 1
.global positive_load
positive_load:
    bx r5
positive_return:
    ldr r6, =0xA55A0007
    str r6, [r4, #8]
    movs r6, #1
    str r6, [r4]
    ldr r5, =invalid_target + 1
    bic.w r5, r5, #1
    dsb
    isb
.global faulting_load
faulting_load:
    bx r5
.global unexpected_continuation
unexpected_continuation:
    movs r6, #2
    str r6, [r4, #4]
    dsb
1:
    b 1b
probe_failure:
    movs r6, #3
    str r6, [r4, #16]
    dsb
2:
    b 2b
.global invalid_target
invalid_target:
    bx lr
.size fault_probe, .-fault_probe
.ltorg
