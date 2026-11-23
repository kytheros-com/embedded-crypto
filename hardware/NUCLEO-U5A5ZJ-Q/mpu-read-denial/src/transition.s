.syntax unified
.thumb

.section .mpu_user,"aw",%nobits
.balign 8
.global mpu_work
.type mpu_work,%object
mpu_work:
    .space 8       // Authorized test word and untrusted progress marker.
.size mpu_work, .-mpu_work

// The runtime calls this symbol on MSP. Startup returns before SPSEL changes.
.section .text.mpu_main,"ax",%progbits
.global main
.type main,%function
.thumb_func
main:
    bl {startup}
    ldr r0, =_psp_top
    msr PSP, r0
    movs r0, #3
    msr CONTROL, r0
    isb
    b mpu_probe
.size main, .-main
.ltorg

// The hardware frame preserves the positive control and address registers.
.section .text.mpu_probe,"ax",%progbits
.global mpu_probe
.type mpu_probe,%function
.thumb_func
mpu_probe:
    mrs r0, CONTROL
    mrs r1, IPSR
    mov r2, sp
    ldr r4, =mpu_work
    ldr r3, [r4]
    ldr r5, ={working_value}
    cmp r3, r5
    bne 1f
    movs r5, #1
    str r5, [r4, #4]
    ldr r12, ={sentinel}
.global mpu_faulting_load
mpu_faulting_load:
    ldr.w r6, [r12] // Preserve the authorized read in stacked r3.
    movs r5, #2    // The forbidden load completed: failure, even if data is 0.
    b 2f
1:
    movs r5, #3    // Positive control mismatch; the forbidden load was skipped.
2:
    str r5, [r4, #4]
    dsb
3:
    b 3b
.size mpu_probe, .-mpu_probe
.ltorg

// Runtime linker override. Capture the original hardware LR, active SP and
// both stack pointers before any Rust prologue or call can change them.
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

// No frame dereference on unexpected exception paths.
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
