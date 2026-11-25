//! STM32U5A5 Cortex-M33 undefined-instruction UsageFault experiment.
//!
//! Privileged Rust configures the MPU and validates one terminal UsageFault
//! exception. Assembly owns the MSP-to-PSP switch, labeled UDF probe,
//! and pre-prologue exception-entry capture. All SWD fields are 32-bit
//! little-endian words. See the dated board procedure.

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

const SENTINEL_VALUE: u32 = 0xA5C3_5A3C;
const UNDEFINSTR: u32 = 1 << 16;
const UDF_ENCODING: u16 = 0xDE42;
const MPU_TYPE: u32 = 0x800;
const MPU_CTRL: u32 = 0b101;
const MAIR0: u32 = 0x44;
const CFSR_W1C: u32 = 0x031F_BFBB;
const HFSR_W1C: u32 = (1 << 30) | (1 << 1);
const STACKING_BITS: u32 =
    (1 << 5) | (1 << 4) | (1 << 3) | (1 << 13) | (1 << 12) | (1 << 11);
const BASIC_FRAME_SIZE_IN_BYTES: usize = 32;

#[repr(usize)]
enum Field {
    Status,
    Phase,
    StartControl,
    StartIpsr,
    StartMsp,
    MpuType,
    InitialMpuCtrl,
    MpuCtrl,
    Mair0,
    Mair1,
    FlashRbar,
    FlashRlar,
    UserRbar,
    UserRlar,
    SupervisorRbar,
    SupervisorRlar,
    UnusedEnabledMask,
    InitialCfsr,
    InitialHfsr,
    ArmedShcsr,
    ExpectedUdfAddress,
    ExpectedUdfEncoding,
    SentinelBefore,
    ProgressMarker,
    ContinuationMarker,
    HandlerControl,
    HandlerIpsr,
    HandlerSp,
    HandlerMsp,
    ExcReturn,
    FramePsp,
    Primask,
    Basepri,
    Faultmask,
    Shcsr,
    Cfsr,
    Hfsr,
    FrameValid,
    StackedR0,
    StackedR1,
    StackedR2,
    StackedR3,
    StackedR12,
    StackedLr,
    StackedPc,
    StackedXpsr,
    UdfEncodingReadback,
    SentinelAfter,
    UnexpectedIpsr,
    Count,
}

// The supervisor-only record is read through SWD after a fault.
#[unsafe(no_mangle)]
static flt_results: [AtomicU32; Field::Count as usize] =
    [const { AtomicU32::new(0) }; Field::Count as usize];

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
    static mpu_work: [u32; 2];
    static faulting_udf: u16;
    static unexpected_continuation: u16;
}

global_asm!(
    include_str!("transition.s"),
    startup = sym startup,
    record_usagefault = sym record_usagefault,
    record_unexpected = sym record_unexpected,
);

fn record(field: Field, value: u32) {
    flt_results[field as usize].store(value, Relaxed);
}

fn ipsr() -> u32 {
    let value;
    // SAFETY: MRS IPSR only reads the current exception number.
    unsafe { asm!("mrs {}, IPSR", out(reg) value, options(nomem, nostack)) };
    value
}

/// Terminal failure holds LD1 on after GPIO setup, when available.
fn fail(status: u32) -> ! {
    record(Field::Status, status);
    dsb();
    // SAFETY: Privileged terminal path uses the previously configured GPIO.
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
    /// Encode a linker-asserted 32-byte aligned window with MAIR index 0.
    fn new(start: *const u8, end: *const u8, permissions: u32) -> Self {
        Self {
            rbar: start.addr() as u32 | permissions,
            rlar: ((end.addr() as u32 - 1) & !31) | 1,
        }
    }
}

/// Called once in privileged Thread mode. Returns on MSP before handoff.
extern "C" fn startup(startup_msp: u32) {
    let startup_control = control::read().bits();
    let startup_ipsr = ipsr();
    record(Field::StartControl, startup_control);
    record(Field::StartIpsr, startup_ipsr);
    record(Field::StartMsp, startup_msp);
    let msp_bottom = addr_of!(_stack_end).addr() as u32;
    let msp_top = addr_of!(_stack_start).addr() as u32;
    if startup_control != 0
        || startup_ipsr != 0
        || startup_msp < msp_bottom
        || startup_msp > msp_top
        || startup_msp & 7 != 0
    {
        fail(0x81);
    }

    // SAFETY: Startup owns these peripherals. The handlers use SCB only
    // for read-only fault capture. The success path runs after startup
    // has relinquished GPIOC and all core peripheral handles.
    let (rcc, gpio, mut core) = unsafe {
        (RCC::steal(), GPIOC::steal(), cortex_m::Peripherals::steal())
    };
    rcc.ahb2enr1().modify(|_, w| w.gpiocen().set_bit());
    let _ = rcc.ahb2enr1().read();
    gpio.bsrr().write(|w| w.bs7().set_bit());
    gpio.otyper().modify(|_, w| w.ot7().clear_bit());
    gpio.ospeedr().modify(|_, w| w.ospeed7().low_speed());
    gpio.pupdr().modify(|_, w| w.pupd7().floating());
    gpio.moder().modify(|_, w| w.mode7().output());

    record(Field::Phase, 1);
    record(
        Field::ExpectedUdfAddress,
        addr_of!(faulting_udf).addr() as u32,
    );
    record(Field::ExpectedUdfEncoding, u32::from(UDF_ENCODING));
    // SAFETY: Startup owns these aligned words before unprivileged entry.
    unsafe {
        let work = addr_of!(mpu_work).cast_mut().cast::<u32>();
        write_volatile(work, 0);
        write_volatile(work.add(1), 0);
        write_volatile(addr_of_mut!(protected_sentinel), SENTINEL_VALUE);
        record(
            Field::SentinelBefore,
            read_volatile(addr_of!(protected_sentinel)),
        );
    }

    configure_mpu(&mut core.MPU, &mut core.SCB);
    record(Field::Phase, 2);
    record(Field::Phase, 3);
    dsb();
}

/// Install and read back the complete three-region configuration.
fn configure_mpu(mpu: &mut MPU, scb: &mut SCB) {
    let kind = mpu._type.read();
    let initial_ctrl = mpu.ctrl.read();
    record(Field::MpuType, kind);
    record(Field::InitialMpuCtrl, initial_ctrl);
    if kind != MPU_TYPE || initial_ctrl != 0 {
        fail(0x83);
    }

    let cfsr = scb.cfsr.read();
    let hfsr = scb.hfsr.read();
    record(Field::InitialCfsr, cfsr);
    record(Field::InitialHfsr, hfsr);

    let regions = [
        // AP=11, XN=0. Privileged and unprivileged read/execute.
        Region::new(addr_of!(_mpu_flash_start), addr_of!(_mpu_flash_end), 6),
        // AP=01, XN=1. Privileged and unprivileged read/write.
        Region::new(addr_of!(_mpu_user_start), addr_of!(_mpu_user_end), 3),
        // AP=00, XN=1. Privileged read/write.
        Region::new(
            addr_of!(_mpu_supervisor_start),
            addr_of!(_mpu_supervisor_end),
            1,
        ),
    ];
    dsb();
    // SAFETY: The observed MPU is disabled. Startup is the only writer
    // and the linker asserts region boundaries. W1C writes clear only
    // observed, documented status bits after capturing the raw values.
    unsafe {
        scb.cfsr.write(cfsr & CFSR_W1C);
        scb.hfsr.write(hfsr & HFSR_W1C);
        mpu.mair[0].write(MAIR0);
        mpu.mair[1].write(0);
        for index in 0..8 {
            mpu.rnr.write(index);
            mpu.rlar.write(0);
        }
        for (index, region) in regions.iter().enumerate() {
            mpu.rnr.write(index as u32);
            mpu.rbar.write(region.rbar);
            mpu.rlar.write(region.rlar);
        }
    }
    scb.enable(Exception::MemoryManagement);
    scb.enable(Exception::UsageFault);
    let shcsr = scb.shcsr.read();
    record(Field::ArmedShcsr, shcsr);
    if shcsr != 0x0005_0000
        || !scb.is_enabled(Exception::MemoryManagement)
        || !scb.is_enabled(Exception::UsageFault)
        || scb.cfsr.read() != 0
        || scb.hfsr.read() != 0
    {
        fail(0x84);
    }

    // SAFETY: Flash, MSP and supervisor data are covered. Device MMIO
    // remains reachable through the privileged background map.
    unsafe { mpu.ctrl.write(MPU_CTRL) };
    dsb();
    isb();

    let ctrl = mpu.ctrl.read();
    let mair0 = mpu.mair[0].read();
    let mair1 = mpu.mair[1].read();
    record(Field::MpuCtrl, ctrl);
    record(Field::Mair0, mair0);
    record(Field::Mair1, mair1);
    let mut matches = ctrl == MPU_CTRL && mair0 == MAIR0 && mair1 == 0;
    let fields = [
        (Field::FlashRbar, Field::FlashRlar),
        (Field::UserRbar, Field::UserRlar),
        (Field::SupervisorRbar, Field::SupervisorRlar),
    ];
    for (index, ((base, limit), expected)) in
        fields.into_iter().zip(regions).enumerate()
    {
        // SAFETY: RNR selects which registers are read, with no MPU
        // region changes. The startup path is the sole selector user.
        unsafe { mpu.rnr.write(index as u32) };
        let actual = Region {
            rbar: mpu.rbar.read(),
            rlar: mpu.rlar.read(),
        };
        record(base, actual.rbar);
        record(limit, actual.rlar);
        matches &= actual == expected;
    }
    let mut unused_enabled = 0;
    for index in 3..8 {
        // SAFETY: Same selector ownership as above.
        unsafe { mpu.rnr.write(index) };
        unused_enabled |= (mpu.rlar.read() & 1) << index;
    }
    record(Field::UnusedEnabledMask, unused_enabled);
    if !matches || unused_enabled != 0 {
        fail(0x83);
    }
}

/// Snapshot sticky status before any validation or frame dereference.
fn read_fault_state() -> (u32, u32, u32) {
    // SAFETY: Runs privileged; these are read-only volatile snapshots.
    let scb = unsafe { &*SCB::PTR };
    let shcsr = scb.shcsr.read();
    let cfsr = scb.cfsr.read();
    let hfsr = scb.hfsr.read();
    record(Field::Shcsr, shcsr);
    record(Field::Cfsr, cfsr);
    record(Field::Hfsr, hfsr);
    (shcsr, cfsr, hfsr)
}

fn masks() -> (u32, u32, u32) {
    let primask: u32;
    let basepri: u32;
    let faultmask: u32;
    // SAFETY: MRS reads processor state without changing it.
    unsafe {
        asm!("mrs {}, PRIMASK", out(reg) primask,
             options(nomem, nostack));
        asm!("mrs {}, BASEPRI", out(reg) basepri,
             options(nomem, nostack));
        asm!("mrs {}, FAULTMASK", out(reg) faultmask,
             options(nomem, nostack));
    }
    (primask, basepri, faultmask)
}

/// Validate one basic PSP frame; remain in privileged Handler mode.
extern "C" fn record_usagefault(
    psp: *const u32,
    exc_return: u32,
    handler_sp: u32,
    handler_msp: u32,
) -> ! {
    let (shcsr, cfsr, hfsr) = read_fault_state();
    let (primask, basepri, faultmask) = masks();
    let handler_control = control::read().bits();
    let handler_ipsr = ipsr();
    record(Field::HandlerControl, handler_control);
    record(Field::HandlerIpsr, handler_ipsr);
    record(Field::HandlerSp, handler_sp);
    record(Field::HandlerMsp, handler_msp);
    record(Field::ExcReturn, exc_return);
    record(Field::FramePsp, psp.addr() as u32);
    record(Field::Primask, primask);
    record(Field::Basepri, basepri);
    record(Field::Faultmask, faultmask);
    record(Field::FrameValid, 0);

    let psp_top = addr_of!(_psp_top).addr();
    let psp_bottom = addr_of!(_psp_bottom).addr();
    let expected_psp = psp_top - BASIC_FRAME_SIZE_IN_BYTES;
    let entry_valid = handler_ipsr == 6
        && handler_control == 1
        && handler_sp == handler_msp
        && handler_msp == addr_of!(_stack_start).addr() as u32
        && exc_return == 0xFFFF_FFBC
        && psp.addr() == expected_psp
        && psp.addr() >= psp_bottom
        && psp.addr() & 7 == 0
        && cfsr & STACKING_BITS == 0
        && cfsr == UNDEFINSTR
        && hfsr == 0;
    if !entry_valid {
        record(Field::UnexpectedIpsr, handler_ipsr);
        fail(0x85);
    }

    // SAFETY: Exact aligned PSP in the linker-asserted user reserve,
    // basic EXC_RETURN, and no stacking fault were checked above.
    let frame: [u32; 8] = unsafe { read_volatile(psp.cast()) };
    let [r0, r1, r2, r3, r12, lr, pc, xpsr] = frame;
    record(Field::FrameValid, 1);
    record(Field::StackedR0, r0);
    record(Field::StackedR1, r1);
    record(Field::StackedR2, r2);
    record(Field::StackedR3, r3);
    record(Field::StackedR12, r12);
    record(Field::StackedLr, lr);
    record(Field::StackedPc, pc);
    record(Field::StackedXpsr, xpsr);

    // SAFETY: Both symbols are in the linked executable Flash window.
    let udf_opcode = unsafe { read_volatile(addr_of!(faulting_udf)) };
    let continuation = addr_of!(unexpected_continuation).addr() as u32;
    let work = addr_of!(mpu_work).cast::<u32>();
    // SAFETY: User control words are inside the configured RW user region.
    let (progress, continued) =
        unsafe { (read_volatile(work), read_volatile(work.add(1))) };
    // SAFETY: Protected sentinel is in supervisor-only SRAM.
    let sentinel = unsafe { read_volatile(addr_of!(protected_sentinel)) };
    record(Field::UdfEncodingReadback, u32::from(udf_opcode));
    record(Field::ProgressMarker, progress);
    record(Field::ContinuationMarker, continued);
    record(Field::SentinelAfter, sentinel);

    if shcsr & 0x0005_0008 != 0x0005_0008
        || cfsr != UNDEFINSTR
        || primask != 0
        || basepri != 0
        || faultmask != 0
        || flt_results[Field::Phase as usize].load(Relaxed) != 3
        || pc != addr_of!(faulting_udf).addr() as u32
        || continuation != pc + 2
        || udf_opcode != UDF_ENCODING
        || xpsr & 0x0100_03FF != 0x0100_0000
        || r0 != 3
        || r1 != 0
        || r2 != psp_top as u32
        || progress != 1
        || continued != 0
        || sentinel != SENTINEL_VALUE
    {
        fail(0x86);
    }

    record(Field::Phase, 4);
    indicate_success()
}

extern "C" fn record_unexpected() -> ! {
    record(Field::UnexpectedIpsr, ipsr());
    if flt_results[Field::HandlerIpsr as usize].load(Relaxed) == 0 {
        let _ = read_fault_state();
    }
    fail(0x87)
}

fn indicate_success() -> ! {
    // SAFETY: Reached only after startup relinquished these handles.
    let (gpio, mut systick) =
        unsafe { (GPIOC::steal(), cortex_m::Peripherals::steal().SYST) };
    systick.set_clock_source(SystClkSource::Core);
    systick.set_reload(500_000 - 1);
    systick.clear_current();
    systick.enable_counter();
    record(Field::Status, 1);
    dsb();
    loop {
        for _ in 0..5 {
            gpio.bsrr().write(|w| w.bs7().set_bit());
            while !systick.has_wrapped() {}
            gpio.bsrr().write(|w| w.br7().set_bit());
            while !systick.has_wrapped() {}
        }
        for _ in 0..5 {
            while !systick.has_wrapped() {}
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    fail(0x88)
}
