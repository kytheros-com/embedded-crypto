.syntax unified
.thumb

// Word offsets into experiment_results. State is cooperative and writable.
.equ STATUS,            0
.equ START_CONTROL,     4
.equ START_IPSR,        8
.equ START_SP,         12
.equ THREAD_CONTROL,   16
.equ THREAD_IPSR,      20
.equ THREAD_SP,        24
.equ DIRECT_CONTROL,   28
.equ HANDLER_CONTROL,  32
.equ HANDLER_IPSR,     36
.equ HANDLER_SP,       40
.equ HANDLER_MSP,      44
.equ EXC_RETURN,       48
.equ FRAME_PSP,        52
.equ FRAME_PC,         56
.equ FRAME_XPSR,       60
.equ SVC_OPCODE,       64
.equ RETURN_CONTROL,   68
.equ RETURN_IPSR,      72
.equ RETURN_SP,        76
.equ FAULT_IPSR,       80
.equ RECORD_SIZE_IN_BYTES, 84

.equ SVC_OPERATION, 0x35
.equ PSP_SIZE_IN_BYTES, 2048
.equ BASIC_FRAME_SIZE_IN_BYTES, 32

.section .bss.experiment_results,"aw",%nobits
.balign 4
.global experiment_results
.type experiment_results,%object
experiment_results:
    .space RECORD_SIZE_IN_BYTES
.size experiment_results, .-experiment_results

.section .uninit.experiment_psp,"aw",%nobits
.balign 8
.global experiment_psp
.type experiment_psp,%object
experiment_psp:
    .space PSP_SIZE_IN_BYTES
experiment_psp_top:
.size experiment_psp, .-experiment_psp

// cortex-m-rt calls main with AAPCS after reset initialization.
.section .text.experiment_main,"ax",%progbits
.global main
.type main,%function
.thumb_func
main:
    ldr r3, =experiment_results
    movs r0, #0x81
    mrs r1, CONTROL
    str r1, [r3, #START_CONTROL]
    mrs r2, IPSR
    str r2, [r3, #START_IPSR]
    mov r12, sp
    str r12, [r3, #START_SP]
    cbnz r1, 1f
    cbnz r2, 1f
    ldr r2, =_stack_start
    cmp r12, r2
    beq 2f
1:
    b experiment_fail
2:
    // AAPCS call on MSP. It returns and removes its frame before the switch.
    bl {initialize_led}
    ldr r3, =experiment_results
    movs r0, #1
    str r0, [r3, #STATUS]
    ldr r2, =experiment_psp_top
    msr PSP, r2
    movs r1, #3
    msr CONTROL, r1
    isb
    b experiment_thread
.size main, .-main
.ltorg

// This function never uses the stack directly before its SVC frame.
.section .text.experiment_thread,"ax",%progbits
.type experiment_thread,%function
.thumb_func
experiment_thread:
    movs r0, #0x82
    mrs r1, CONTROL
    str r1, [r3, #THREAD_CONTROL]
    mrs r2, IPSR
    str r2, [r3, #THREAD_IPSR]
    mov r12, sp
    str r12, [r3, #THREAD_SP]
    cmp r1, #3
    bne experiment_fail
    cmp r2, #0
    bne experiment_fail
    ldr r2, =experiment_psp_top
    cmp r12, r2
    bne experiment_fail
    movs r0, #2
    str r0, [r3, #STATUS]

    // Unprivileged MSR CONTROL is ignored. Request PSP with nPRIV clear.
    movs r0, #0x83
    movs r1, #2
    msr CONTROL, r1
    isb
    mrs r1, CONTROL
    str r1, [r3, #DIRECT_CONTROL]
    cmp r1, #3
    bne experiment_fail
    movs r0, #3
    str r0, [r3, #STATUS]
    svc #SVC_OPERATION
.global experiment_svc_return
experiment_svc_return:
    movs r0, #0x85
    mrs r1, CONTROL
    str r1, [r3, #RETURN_CONTROL]
    mrs r2, IPSR
    str r2, [r3, #RETURN_IPSR]
    mov r12, sp
    str r12, [r3, #RETURN_SP]
    cmp r1, #2
    bne experiment_fail
    cmp r2, #0
    bne experiment_fail
    ldr r2, =experiment_psp_top
    cmp r12, r2
    bne experiment_fail
    ldr r1, [r3, #STATUS]
    cmp r1, #4
    bne experiment_fail
    movs r0, #5
    str r0, [r3, #STATUS]
    b {indicate_success}
.size experiment_thread, .-experiment_thread
.ltorg

// cortex-m-rt link.x supplies a weak alias for the SVCall vector. This
// definition overrides it and receives the hardware LR before any prologue.
// Only hardware-stacked caller-saved registers are used. LR stays intact.
.section .text.experiment_svcall,"ax",%progbits
.global SVCall
.type SVCall,%function
.thumb_func
SVCall:
    ldr r3, =experiment_results
    movs r0, #0x84
    mrs r1, CONTROL
    str r1, [r3, #HANDLER_CONTROL]
    mrs r2, IPSR
    str r2, [r3, #HANDLER_IPSR]
    mov r12, sp
    str r12, [r3, #HANDLER_SP]
    mrs r12, MSP
    str r12, [r3, #HANDLER_MSP]
    str lr, [r3, #EXC_RETURN]
    cmp r2, #11
    bne experiment_fail
    cmp r1, #1
    bne experiment_fail
    cmp sp, r12
    bne experiment_fail
    ldr r2, =_stack_start
    cmp r12, r2
    bne experiment_fail

    // Require a basic Thread/PSP return with default stacking. Check the
    // common EXC_RETURN fields, excluding S and ES, and reuse hardware LR.
    bic r1, lr, #0x41
    mvn r2, #0x43
    cmp r1, r2
    bne experiment_fail
    ldr r1, [r3, #STATUS]
    cmp r1, #3
    bne experiment_fail

    // Require exactly the basic frame at the aligned PSP top before loads.
    mrs r12, PSP
    str r12, [r3, #FRAME_PSP]
    ldr r2, =experiment_psp_top - BASIC_FRAME_SIZE_IN_BYTES
    cmp r12, r2
    bne experiment_fail
    ldr r1, [r12, #24]
    str r1, [r3, #FRAME_PC]
    ldr r2, [r12, #28]
    str r2, [r3, #FRAME_XPSR]
    // T set, no stack alignment padding, interrupted IPSR zero.
    ldr r12, =0x010003ff
    ands r2, r12
    cmp r2, #0x01000000
    bne experiment_fail
    ldr r2, =experiment_svc_return
    cmp r1, r2
    bne experiment_fail
    ldrh r1, [r1, #-2]
    str r1, [r3, #SVC_OPCODE]
    movw r2, #(0xdf00 | SVC_OPERATION)
    cmp r1, r2
    bne experiment_fail

    movs r1, #0
    msr CONTROL, r1
    isb
    movs r1, #4
    str r1, [r3, #STATUS]
    bx lr
.size SVCall, .-SVCall
.ltorg

.section .text.experiment_fail,"ax",%progbits
.type experiment_fail,%function
.thumb_func
experiment_fail:
    str r0, [r3, #STATUS]
    dsb
.global experiment_halted
experiment_halted:
    b experiment_halted
.size experiment_fail, .-experiment_fail

// Keep failure handling independent of the interrupted stack frame.
// No fault recovery or MMIO access is attempted from a fault handler.
.section .text.experiment_fault,"ax",%progbits
.global HardFault
.global DefaultHandler
.type HardFault,%function
.type DefaultHandler,%function
.thumb_func
HardFault:
.thumb_func
DefaultHandler:
    ldr r3, =experiment_results
    mrs r1, IPSR
    str r1, [r3, #FAULT_IPSR]
    movs r0, #0x86
    b experiment_fail
.size HardFault, .-HardFault
.size DefaultHandler, .-DefaultHandler
.ltorg
