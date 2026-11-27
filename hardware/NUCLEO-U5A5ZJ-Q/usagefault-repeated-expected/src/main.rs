//! Two one-shot expected UDF faults during one boot, with controlled return.
//!
//! A protected descriptor authorizes only the two linked UDF sites and their
//! linked continuations. This is characterization firmware, not recovery.
//! Diagnostic words are little-endian u32 values, bit 0 least significant.

#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use cortex_m::asm::{dsb, isb, nop};
use cortex_m::peripheral::scb::Exception;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::{MPU, SCB};
use cortex_m::register::control;
use cortex_m_rt as _;
use stm32u5::stm32u5a5::{GPIOC, RCC};

const SENTINEL: u32 = 0xA5C3_5A3C;
const UNDEFINSTR: u32 = 0x0001_0000;
const SHCSR_ARMED: u32 = 0x0005_0000;
const SHCSR_FAULT: u32 = 0x0005_0008;
const SHCSR_SVC: u32 = 0x0005_0080;
const CFSR_W1C: u32 = 0x031F_BFBB;
const HFSR_W1C: u32 = (1 << 30) | (1 << 1);
const STACKING_BITS: u32 =
    (1 << 5) | (1 << 4) | (1 << 3) | (1 << 13) | (1 << 12) | (1 << 11);
const FRAME_ADDRESS: u32 = 0x2000_1FE0;
const EXC_RETURN: u32 = 0xFFFF_FFBC;
const RESULT_WORDS: usize = 64;
const SLOT_WORDS: usize = 20;
const SLOT_ONE: usize = 16;
const SLOT_TWO: usize = SLOT_ONE + SLOT_WORDS;

#[repr(usize)]
enum Field {
    Status,
    Phase,
    FaultCount,
    SvcCount,
    PositiveMask,
    CompletionMask,
    UnexpectedIpsr,
    InitialCfsr,
    FinalCfsr,
    ArmedShcsr,
    Ccr,
    SentinelBefore,
    SentinelAfter,
    DescriptorState,
    DescriptorArmed,
    Error,
}

#[repr(usize)]
enum Desc {
    State,
    Armed,
    TestId,
    Sequence,
    ExpectedException,
    ExpectedSyndrome,
    ExpectedOrigin,
    FaultPc,
    ContinuationPc,
    FramePsp,
    ExcReturn,
    CompletionCondition,
    Count,
}

// Slot words: frame R0,R1,R2,R3,R12,LR,PC,xPSR; CFSR,HFSR,
// SHCSR,EXC_RETURN,PSP,MSP,CONTROL,IPSR,fault PC,continuation,
// test ID,valid. Slots are never reused during a boot.
#[unsafe(no_mangle)]
static flt_results: [AtomicU32; RESULT_WORDS] =
    [const { AtomicU32::new(0) }; RESULT_WORDS];

#[unsafe(no_mangle)]
static expected_fault: [AtomicU32; Desc::Count as usize] =
    [const { AtomicU32::new(0) }; Desc::Count as usize];

#[unsafe(no_mangle)]
static mut protected_sentinel: u32 = 0;

unsafe extern "C" {
    static _mpu_flash_start: u8;
    static _mpu_flash_end: u8;
    static _mpu_user_start: u8;
    static _mpu_user_end: u8;
    static _mpu_supervisor_start: u8;
    static _mpu_supervisor_end: u8;
    static _stack_start: u8;
    static _stack_end: u8;
    static _psp_bottom: u8;
    static _psp_top: u8;
    static mpu_work: [u32; 6];
    static udf_one: u16;
    static cont_one: u16;
    static udf_two: u16;
    static cont_two: u16;
    static svc_arm_two_return: u16;
    static svc_finish_return: u16;
}

global_asm!(
    include_str!("transition.s"),
    startup = sym startup,
    record_usagefault = sym record_usagefault,
    record_svc = sym record_svc,
    record_unexpected = sym record_unexpected,
);

fn put(field: Field, value: u32) {
    flt_results[field as usize].store(value, Relaxed);
}

fn desc(field: Desc) -> u32 {
    expected_fault[field as usize].load(Relaxed)
}

fn set_desc(field: Desc, value: u32) {
    expected_fault[field as usize].store(value, Relaxed);
}

fn ipsr() -> u32 {
    let value;
    // SAFETY: MRS reads the current exception number.
    unsafe { asm!("mrs {}, IPSR", out(reg) value, options(nomem, nostack)) };
    value
}

fn masks_clear() -> bool {
    let primask: u32;
    let basepri: u32;
    let faultmask: u32;
    // SAFETY: MRS only reads processor masks.
    unsafe {
        asm!("mrs {}, PRIMASK", out(reg) primask,
             options(nomem, nostack));
        asm!("mrs {}, BASEPRI", out(reg) basepri,
             options(nomem, nostack));
        asm!("mrs {}, FAULTMASK", out(reg) faultmask,
             options(nomem, nostack));
    }
    primask == 0 && basepri == 0 && faultmask == 0
}

fn fail(code: u32) -> ! {
    put(Field::Error, code);
    put(Field::Status, code);
    dsb();
    // SAFETY: Terminal path is privileged with GPIO configured by startup.
    unsafe { GPIOC::steal() }
        .bsrr()
        .write(|w| w.bs7().set_bit());
    loop {
        nop();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Region {
    rbar: u32,
    rlar: u32,
}

impl Region {
    fn new(start: *const u8, end: *const u8, permissions: u32) -> Self {
        Self {
            rbar: start.addr() as u32 | permissions,
            rlar: ((end.addr() as u32 - 1) & !31) | 1,
        }
    }
}

fn arm(test_id: u32, position: u32, fault: u32, next: u32, completion: u32) {
    set_desc(Desc::Armed, 0);
    set_desc(Desc::TestId, test_id);
    set_desc(Desc::Sequence, position);
    set_desc(Desc::ExpectedException, 6);
    set_desc(Desc::ExpectedSyndrome, UNDEFINSTR);
    set_desc(Desc::ExpectedOrigin, 3);
    set_desc(Desc::FaultPc, fault);
    set_desc(Desc::ContinuationPc, next);
    set_desc(Desc::FramePsp, FRAME_ADDRESS);
    set_desc(Desc::ExcReturn, EXC_RETURN);
    set_desc(Desc::CompletionCondition, completion);
    set_desc(Desc::State, if test_id == 1 { 1 } else { 3 });
    dsb();
    set_desc(Desc::Armed, 1);
    dsb();
    put(Field::DescriptorState, desc(Desc::State));
    put(Field::DescriptorArmed, 1);
}

/// Privileged startup returns to the assembly MSP-to-PSP handoff.
extern "C" fn startup(startup_msp: u32) {
    for word in &flt_results {
        word.store(0, Relaxed);
    }
    for word in &expected_fault {
        word.store(0, Relaxed);
    }
    if control::read().bits() != 0
        || ipsr() != 0
        || startup_msp < addr_of!(_stack_end).addr() as u32
        || startup_msp > addr_of!(_stack_start).addr() as u32
        || startup_msp & 7 != 0
    {
        fail(0x81);
    }

    // SAFETY: Startup owns RCC, GPIO, MPU and SCB until it returns.
    let (rcc, gpio, mut core) = unsafe {
        (RCC::steal(), GPIOC::steal(), cortex_m::Peripherals::steal())
    };
    rcc.ahb2enr1().modify(|_, w| w.gpiocen().set_bit());
    let _ = rcc.ahb2enr1().read();
    gpio.bsrr().write(|w| w.br7().set_bit());
    gpio.otyper().modify(|_, w| w.ot7().clear_bit());
    gpio.ospeedr().modify(|_, w| w.ospeed7().low_speed());
    gpio.pupdr().modify(|_, w| w.pupd7().floating());
    gpio.moder().modify(|_, w| w.mode7().output());
    put(Field::Phase, 1);

    // SAFETY: Startup initializes all user words and protected sentinel.
    unsafe {
        let work = addr_of!(mpu_work).cast_mut().cast::<u32>();
        for index in 0..6 {
            write_volatile(work.add(index), 0);
        }
        write_volatile(addr_of_mut!(protected_sentinel), SENTINEL);
    }
    put(Field::SentinelBefore, SENTINEL);
    configure_mpu(&mut core.MPU, &mut core.SCB);
    let ccr = core.SCB.ccr.read();
    put(Field::Ccr, ccr);
    if ccr & (1 << 4) != 0 || !masks_clear() {
        fail(0x82);
    }
    arm(
        1,
        1,
        addr_of!(udf_one).addr() as u32,
        addr_of!(cont_one).addr() as u32,
        2,
    );
    put(Field::Phase, 2);
}

fn configure_mpu(mpu: &mut MPU, scb: &mut SCB) {
    if mpu._type.read() != 0x800 || mpu.ctrl.read() != 0 {
        fail(0x83);
    }
    let cfsr = scb.cfsr.read();
    let hfsr = scb.hfsr.read();
    put(Field::InitialCfsr, cfsr);
    dsb();
    // SAFETY: Startup owns disabled MPU; writes clear observed status.
    unsafe {
        scb.cfsr.write(cfsr & CFSR_W1C);
        scb.hfsr.write(hfsr & HFSR_W1C);
        mpu.mair[0].write(0x44);
        mpu.mair[1].write(0);
        for index in 0..8 {
            mpu.rnr.write(index);
            mpu.rlar.write(0);
        }
    }
    let regions = [
        Region::new(addr_of!(_mpu_flash_start), addr_of!(_mpu_flash_end), 6),
        Region::new(addr_of!(_mpu_user_start), addr_of!(_mpu_user_end), 3),
        Region::new(
            addr_of!(_mpu_supervisor_start),
            addr_of!(_mpu_supervisor_end),
            1,
        ),
    ];
    for (index, region) in regions.iter().enumerate() {
        // SAFETY: Only startup selects and writes the MPU region.
        unsafe {
            mpu.rnr.write(index as u32);
            mpu.rbar.write(region.rbar);
            mpu.rlar.write(region.rlar);
        }
    }
    scb.enable(Exception::MemoryManagement);
    scb.enable(Exception::UsageFault);
    let shcsr = scb.shcsr.read();
    put(Field::ArmedShcsr, shcsr);
    if shcsr != SHCSR_ARMED || scb.cfsr.read() != 0 || scb.hfsr.read() != 0 {
        fail(0x84);
    }
    // SAFETY: Privileged startup enables the checked MPU configuration.
    unsafe { mpu.ctrl.write(5) };
    dsb();
    isb();
    if mpu.ctrl.read() != 5
        || mpu.mair[0].read() != 0x44
        || mpu.mair[1].read() != 0
    {
        fail(0x83);
    }
    for (index, region) in regions.iter().enumerate() {
        // SAFETY: Only startup selects the MPU region number.
        unsafe { mpu.rnr.write(index as u32) };
        if mpu.rbar.read() != region.rbar || mpu.rlar.read() != region.rlar {
            fail(0x83);
        }
    }
    for index in 3..8 {
        // SAFETY: Only startup selects the MPU region number.
        unsafe { mpu.rnr.write(index) };
        if mpu.rlar.read() & 1 != 0 {
            fail(0x83);
        }
    }
}

fn frame_prerequisites(
    psp: *const u32,
    exc: u32,
    sp: u32,
    msp: u32,
    exception: u32,
    cfsr: u32,
    hfsr: u32,
) -> bool {
    psp.addr() as u32 == FRAME_ADDRESS
        && psp.addr() >= addr_of!(_psp_bottom).addr()
        && psp.addr() + 32 == addr_of!(_psp_top).addr()
        && psp.addr() & 7 == 0
        && exc == EXC_RETURN
        && sp == 0x2000_3000
        && msp == sp
        && ipsr() == exception
        && control::read().bits() == 1
        && masks_clear()
        && cfsr & STACKING_BITS == 0
        && hfsr == 0
}

fn user_words() -> [u32; 6] {
    // SAFETY: All six initialized words lie inside the user MPU region.
    unsafe { read_volatile(addr_of!(mpu_work)) }
}

/// Validate, record, retire and redirect one specifically armed UDF.
extern "C" fn record_usagefault(psp: *mut u32, exc: u32, sp: u32, msp: u32) {
    // SAFETY: Privileged Handler mode reads the SCB syndrome.
    let scb = unsafe { &*SCB::PTR };
    let shcsr = scb.shcsr.read();
    let cfsr = scb.cfsr.read();
    let hfsr = scb.hfsr.read();
    let state = desc(Desc::State);
    let test_id = desc(Desc::TestId);
    let (slot, linked_fault, linked_next, expected_count) =
        if state == 1 && test_id == 1 {
            (
                SLOT_ONE,
                addr_of!(udf_one).addr() as u32,
                addr_of!(cont_one).addr() as u32,
                0,
            )
        } else if state == 3 && test_id == 2 {
            (
                SLOT_TWO,
                addr_of!(udf_two).addr() as u32,
                addr_of!(cont_two).addr() as u32,
                1,
            )
        } else {
            fail(0x85)
        };
    if !frame_prerequisites(psp, exc, sp, msp, 6, cfsr, hfsr)
        || shcsr != SHCSR_FAULT
        || cfsr != UNDEFINSTR
        || desc(Desc::Armed) != 1
        || desc(Desc::Sequence) != test_id
        || desc(Desc::ExpectedException) != 6
        || desc(Desc::ExpectedSyndrome) != UNDEFINSTR
        || desc(Desc::ExpectedOrigin) != 3
        || desc(Desc::FaultPc) != linked_fault
        || desc(Desc::ContinuationPc) != linked_next
        || desc(Desc::FramePsp) != FRAME_ADDRESS
        || desc(Desc::ExcReturn) != EXC_RETURN
        || desc(Desc::CompletionCondition) != test_id * 3 - 1
        || flt_results[Field::FaultCount as usize].load(Relaxed)
            != expected_count
        || linked_next != linked_fault + 2
    {
        fail(0x85);
    }
    // SAFETY: Exact basic PSP frame and no stacking fault were checked.
    let frame: [u32; 8] = unsafe { read_volatile(psp.cast()) };
    if frame[0] != 3
        || frame[1] != 0
        || frame[2] != addr_of!(_psp_top).addr() as u32
        || frame[6] != linked_fault
        || frame[7] & 0x0100_03FF != 0x0100_0000
    {
        fail(0x86);
    }
    // SAFETY: The descriptor's linked fault address is in Flash.
    if unsafe { read_volatile(linked_fault as *const u16) } != 0xDE42 {
        fail(0x86);
    }
    for (index, word) in frame.into_iter().enumerate() {
        flt_results[slot + index].store(word, Relaxed);
    }
    for (index, word) in [
        cfsr,
        hfsr,
        shcsr,
        exc,
        psp.addr() as u32,
        msp,
        control::read().bits(),
        6,
        linked_fault,
        linked_next,
        test_id,
        1,
    ]
    .into_iter()
    .enumerate()
    {
        flt_results[slot + 8 + index].store(word, Relaxed);
    }
    put(Field::FaultCount, expected_count + 1);
    dsb();
    // SAFETY: W1C of only the captured UNDEFINSTR syndrome.
    unsafe { scb.cfsr.write(UNDEFINSTR) };
    dsb();
    isb();
    if scb.cfsr.read() != 0 || scb.hfsr.read() != 0 {
        fail(0x86);
    }
    // SAFETY: The checked frame's PC slot alone is changed to the
    // descriptor's exact linked continuation. All other words stay put.
    unsafe { write_volatile(psp.add(6), linked_next) };
    dsb();
    set_desc(Desc::Armed, 0);
    set_desc(Desc::State, if test_id == 1 { 2 } else { 4 });
    put(Field::DescriptorArmed, 0);
    put(Field::DescriptorState, desc(Desc::State));
    put(Field::FinalCfsr, scb.cfsr.read());
    put(Field::Phase, if test_id == 1 { 3 } else { 4 });
    dsb();
}

/// SVC is fixed-site orchestration, not an unprivileged authorization API.
extern "C" fn record_svc(psp: *const u32, exc: u32, sp: u32, msp: u32) {
    // SAFETY: Privileged Handler mode reads SCB state.
    let scb = unsafe { &*SCB::PTR };
    let cfsr = scb.cfsr.read();
    let hfsr = scb.hfsr.read();
    if !frame_prerequisites(psp, exc, sp, msp, 11, cfsr, hfsr)
        || cfsr != 0
        || scb.shcsr.read() != SHCSR_SVC
        || desc(Desc::Armed) != 0
    {
        fail(0x87);
    }
    // SAFETY: Exact basic PSP frame checked above.
    let frame: [u32; 8] = unsafe { read_volatile(psp.cast()) };
    let pc = frame[6];
    if frame[7] & 0x0100_03FF != 0x0100_0000 || pc & 1 != 0 {
        fail(0x87);
    }
    let work = user_words();
    if desc(Desc::State) == 2
        && pc == addr_of!(svc_arm_two_return).addr() as u32
        && work[..3] == [1, 2, 3]
        && work[3..] == [0, 0, 0]
        && flt_results[Field::FaultCount as usize].load(Relaxed) == 1
    {
        // SAFETY: The preceding fixed Flash halfword is the SVC site.
        if unsafe { read_volatile((pc - 2) as *const u16) } != 0xDF72 {
            fail(0x87);
        }
        put(Field::PositiveMask, 3);
        put(Field::CompletionMask, 2);
        put(Field::SvcCount, 1);
        arm(
            2,
            2,
            addr_of!(udf_two).addr() as u32,
            addr_of!(cont_two).addr() as u32,
            5,
        );
        return;
    }
    if desc(Desc::State) == 4
        && pc == addr_of!(svc_finish_return).addr() as u32
        && work == [1, 2, 3, 4, 5, 0]
        && flt_results[Field::FaultCount as usize].load(Relaxed) == 2
        && flt_results[SLOT_ONE + 19].load(Relaxed) == 1
        && flt_results[SLOT_TWO + 19].load(Relaxed) == 1
    {
        // SAFETY: The preceding fixed Flash halfword is the SVC site.
        if unsafe { read_volatile((pc - 2) as *const u16) } != 0xDF73 {
            fail(0x87);
        }
        put(Field::PositiveMask, 3);
        put(Field::CompletionMask, 5);
        put(Field::SvcCount, 2);
        // SAFETY: Privileged Handler mode reads supervisor sentinel.
        let sentinel = unsafe { read_volatile(addr_of!(protected_sentinel)) };
        put(Field::SentinelAfter, sentinel);
        if sentinel != SENTINEL {
            fail(0x87);
        }
        set_desc(Desc::State, 5);
        put(Field::DescriptorState, 5);
        put(Field::Phase, 5);
        indicate_success()
    }
    fail(0x87)
}

extern "C" fn record_unexpected() -> ! {
    put(Field::UnexpectedIpsr, ipsr());
    fail(0x88)
}

fn indicate_success() -> ! {
    // SAFETY: Terminal Handler mode owns GPIO and polled SysTick.
    let (gpio, mut systick) =
        unsafe { (GPIOC::steal(), cortex_m::Peripherals::steal().SYST) };
    systick.set_clock_source(SystClkSource::Core);
    systick.disable_interrupt();
    systick.set_reload(2_000_000 - 1);
    systick.clear_current();
    systick.enable_counter();
    put(Field::Status, 1);
    dsb();
    loop {
        for _ in 0..3 {
            gpio.bsrr().write(|w| w.bs7().set_bit());
            while !systick.has_wrapped() {}
            gpio.bsrr().write(|w| w.br7().set_bit());
            while !systick.has_wrapped() {}
        }
        for _ in 0..4 {
            while !systick.has_wrapped() {}
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    fail(0x89)
}
