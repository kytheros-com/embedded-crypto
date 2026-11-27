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
    ldr r5, =mpu_pattern
.global positive_load
positive_load:
    ldrd r6, r7, [r5]
    ldr r8, =0x44332211
    cmp r6, r8
    bne probe_failure
    ldr r8, =0x88776655
    cmp r7, r8
    bne probe_failure
    str r6, [r4, #8]
    adds r5, #1
    movs r7, #1
    str r7, [r4]
    dsb
    isb
.global faulting_load
faulting_load:
    ldrd r6, r7, [r5]
.global unexpected_continuation
unexpected_continuation:
    str r6, [r4, #12]
    dsb
    movs r7, #2
    str r7, [r4, #4]
    dsb
1:
    b 1b
probe_failure:
    movs r7, #3
    str r7, [r4, #16]
    dsb
2:
    b 2b
.size fault_probe, .-fault_probe
.ltorg
