.syntax unified
.thumb

.section .mpu_user,"aw",%nobits
.balign 8
.global mpu_work
.type mpu_work,%object
mpu_work:
    .space 24
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

// A single boot executes both UDF instructions. No probe stack use.
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
    movs r6, #42
    adds r6, #1
    cmp r6, #43
    bne probe_failure
    movs r6, #1
    str r6, [r4, #0]
    dsb
    isb
.global udf_one
udf_one:
    udf #0x42
.global cont_one
cont_one:
    movs r6, #2
    str r6, [r4, #4]
    dsb
    movs r6, #40
    adds r6, #4
    cmp r6, #44
    bne probe_failure
    movs r6, #3
    str r6, [r4, #8]
    dsb
.global svc_arm_two
svc_arm_two:
    svc #0x72
.global svc_arm_two_return
svc_arm_two_return:
    movs r6, #4
    str r6, [r4, #12]
    dsb
    mrs r0, CONTROL
    mrs r1, IPSR
    mov r2, sp
    cmp r0, #3
    bne probe_failure
    cmp r1, #0
    bne probe_failure
    ldr r5, =_psp_top
    cmp r2, r5
    bne probe_failure
    isb
.global udf_two
udf_two:
    udf #0x42
.global cont_two
cont_two:
    movs r6, #5
    str r6, [r4, #16]
    dsb
.global svc_finish
svc_finish:
    svc #0x73
.global svc_finish_return
svc_finish_return:
    b probe_failure
probe_failure:
    movs r6, #3
    str r6, [r4, #20]
    dsb
1:
    b 1b
.size fault_probe, .-fault_probe
.ltorg

// Capture and check the original PSP/MSP/EXC_RETURN before stack use.
.section .text.flt_usagefault,"ax",%progbits
.global UsageFault
.type UsageFault,%function
.thumb_func
UsageFault:
    mrs r0, PSP
    mov r1, lr
    mov r2, sp
    mrs r3, MSP
    b validate_usage_entry
.size UsageFault, .-UsageFault

.global SVCall
.type SVCall,%function
.thumb_func
SVCall:
    mrs r0, PSP
    mov r1, lr
    mov r2, sp
    mrs r3, MSP
    b validate_svc_entry
.size SVCall, .-SVCall

validate_usage_entry:
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
    push {{r4, lr}}
    bl {record_usagefault}
    pop {{r4, pc}}

validate_svc_entry:
    ldr r12, =0x20001FE0
    cmp r0, r12
    bne entry_failure
    ldr r12, =0x20003000
    cmp r2, r12
    bne entry_failure
    cmp r3, r12
    ldr r12, =0xFFFFFFBC
    bne entry_failure
    cmp r1, r12
    bne entry_failure
    push {{r4, lr}}
    bl {record_svc}
    pop {{r4, pc}}

entry_failure:
    b entry_failure
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
