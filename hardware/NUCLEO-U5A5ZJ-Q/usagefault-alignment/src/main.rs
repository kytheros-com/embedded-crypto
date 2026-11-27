//! STM32U5A5 Normal-memory alignment UsageFault experiments.
//!
//! Privileged Rust configures the MPU and validates one terminal UsageFault
//! exception. Assembly owns the MSP-to-PSP switch, memory probes,
//! and pre-prologue exception-entry capture. The default build enables
//! CCR.UNALIGN_TRP; `trap-disabled` and `mandatory` select controls.
//! SWD fields are 32-bit little-endian words, bit 0 least significant.

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
const UNALIGNED: u32 = 1 << 24;
const INVSTATE: u32 = 1 << 17;
const UNALIGN_TRP: u32 = 1 << 3;
const LOAD_FIRST: u16 = if INVALID_STATE {
    0x4728
} else if MANDATORY {
    0xE9D5
} else {
    0xF8D5
};
const LOAD_SECOND: u16 = if INVALID_STATE {
    0x4770
} else if MANDATORY {
    0x6700
} else {
    0x6001
};
const FAULT_VALUE_SENTINEL: u32 = 0xA55A_5AA5;
const INVALID_STATE: bool = cfg!(feature = "invalid-state");
const MANDATORY: bool = cfg!(feature = "mandatory");
const TRAP_EXPECTED: bool = !cfg!(feature = "trap-disabled");

#[cfg(all(feature = "trap-disabled", feature = "mandatory"))]
compile_error!("trap-disabled and mandatory are mutually exclusive");

#[cfg(all(feature = "invalid-state", feature = "mandatory"))]
compile_error!("invalid-state and mandatory are mutually exclusive");

#[cfg(all(feature = "invalid-state", feature = "trap-disabled"))]
compile_error!("invalid-state and trap-disabled are mutually exclusive");
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
    ExpectedFaultAddress,
    ExpectedFirstHalfword,
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
    FirstReadback,
    SentinelAfter,
    UnexpectedIpsr,
    InitialCcr,
    ConfiguredCcr,
    HandlerCcr,
    BuildMode,
    PositiveValue,
    FaultValue,
    FailureMarker,
    ExpectedSecondHalfword,
    SecondReadback,
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
    static mpu_work: [u32; 5];
    static mpu_pattern: [u8; 12];
    static faulting_load: u16;
    static invalid_target: u16;
    static unexpected_continuation: u16;
}

global_asm!(
    include_str!("transition.s"),
    startup = sym startup,
    record_usagefault = sym record_usagefault,
    record_unexpected = sym record_unexpected,
);

#[cfg(not(any(feature = "mandatory", feature = "invalid-state")))]
global_asm!(include_str!("probe-unaligned.s"));

#[cfg(feature = "mandatory")]
global_asm!(include_str!("probe-mandatory.s"));

#[cfg(feature = "invalid-state")]
global_asm!(include_str!("probe-invstate.s"));

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
    // SRAM may be retained across resets; clear every diagnostic word.
    for word in &flt_results {
        word.store(0, Relaxed);
    }
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
    gpio.bsrr().write(|w| w.br7().set_bit());
    gpio.otyper().modify(|_, w| w.ot7().clear_bit());
    gpio.ospeedr().modify(|_, w| w.ospeed7().low_speed());
    gpio.pupdr().modify(|_, w| w.pupd7().floating());
    gpio.moder().modify(|_, w| w.mode7().output());

    record(Field::Phase, 1);
    record(
        Field::BuildMode,
        if INVALID_STATE {
            3
        } else if MANDATORY {
            2
        } else {
            u32::from(TRAP_EXPECTED)
        },
    );
    record(
        Field::ExpectedFaultAddress,
        if INVALID_STATE {
            addr_of!(invalid_target).addr() as u32
        } else {
            addr_of!(faulting_load).addr() as u32
        },
    );
    record(Field::ExpectedFirstHalfword, u32::from(LOAD_FIRST));
    record(Field::ExpectedSecondHalfword, u32::from(LOAD_SECOND));
    // SAFETY: Startup owns these aligned words before unprivileged entry.
    unsafe {
        let work = addr_of!(mpu_work).cast_mut().cast::<u32>();
        write_volatile(work, 0);
        write_volatile(work.add(1), 0);
        write_volatile(work.add(2), 0);
        write_volatile(work.add(3), FAULT_VALUE_SENTINEL);
        write_volatile(work.add(4), 0);
        let pattern = addr_of!(mpu_pattern).cast::<u8>();
        for (offset, byte) in (0x11u8..=0xCC).step_by(0x11).enumerate() {
            write_volatile(pattern.add(offset).cast_mut(), byte);
        }
        write_volatile(addr_of_mut!(protected_sentinel), SENTINEL_VALUE);
        record(
            Field::SentinelBefore,
            read_volatile(addr_of!(protected_sentinel)),
        );
    }

    configure_mpu(&mut core.MPU, &mut core.SCB);
    configure_alignment_trap(&mut core.SCB);
    record(Field::Phase, 2);
    record(Field::Phase, 3);
    if !TRAP_EXPECTED {
        // Configured control, not proof that continuation has run.
        record(Field::Status, 2);
    }
    dsb();
}

/// Configure only UNALIGN_TRP and check all other readable CCR bits.
fn configure_alignment_trap(scb: &mut SCB) {
    let initial = scb.ccr.read();
    record(Field::InitialCcr, initial);
    let requested = if TRAP_EXPECTED && !MANDATORY && !INVALID_STATE {
        initial | UNALIGN_TRP
    } else {
        initial & !UNALIGN_TRP
    };
    // SAFETY: Privileged startup is the only CCR writer in this probe.
    unsafe { scb.ccr.write(requested) };
    dsb();
    isb();
    let configured = scb.ccr.read();
    record(Field::ConfiguredCcr, configured);
    if initial & UNALIGN_TRP != 0
        || configured & UNALIGN_TRP != requested & UNALIGN_TRP
        || (configured ^ initial) & !UNALIGN_TRP != 0
    {
        fail(0x89);
    }
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
    // SAFETY: Privileged Handler mode may read the system control block.
    let handler_ccr = unsafe { &*SCB::PTR }.ccr.read();
    record(Field::HandlerCcr, handler_ccr);
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
    let entry_valid = TRAP_EXPECTED
        && handler_ipsr == 6
        && handler_control == 1
        && handler_sp == handler_msp
        && handler_msp == addr_of!(_stack_start).addr() as u32
        && exc_return == 0xFFFF_FFBC
        && psp.addr() == expected_psp
        && psp.addr() >= psp_bottom
        && psp.addr() & 7 == 0
        && cfsr & STACKING_BITS == 0
        && cfsr == if INVALID_STATE { INVSTATE } else { UNALIGNED }
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

    // SAFETY: Both halfwords are in the linked executable Flash window.
    let load = addr_of!(faulting_load);
    let first = unsafe { read_volatile(load) };
    let second = if INVALID_STATE {
        // SAFETY: The positive-control target is executable Flash.
        unsafe { read_volatile(addr_of!(invalid_target)) }
    } else {
        // SAFETY: Second halfword of the linked 32-bit load.
        unsafe { read_volatile(load.add(1)) }
    };
    let continuation = addr_of!(unexpected_continuation).addr() as u32;
    let work = addr_of!(mpu_work).cast::<u32>();
    // SAFETY: All five words are inside the configured RW user region.
    let (progress, continued, positive, zero, failure) = unsafe {
        (
            read_volatile(work),
            read_volatile(work.add(1)),
            read_volatile(work.add(2)),
            read_volatile(work.add(3)),
            read_volatile(work.add(4)),
        )
    };
    // SAFETY: Protected sentinel is in supervisor-only SRAM.
    let sentinel = unsafe { read_volatile(addr_of!(protected_sentinel)) };
    record(Field::FirstReadback, u32::from(first));
    record(Field::SecondReadback, u32::from(second));
    record(Field::ProgressMarker, progress);
    record(Field::ContinuationMarker, continued);
    record(Field::PositiveValue, positive);
    record(Field::FaultValue, zero);
    record(Field::FailureMarker, failure);
    record(Field::SentinelAfter, sentinel);

    if shcsr != 0x0005_0008
        || cfsr != if INVALID_STATE { INVSTATE } else { UNALIGNED }
        || (handler_ccr & UNALIGN_TRP != 0) != !(MANDATORY || INVALID_STATE)
        || flt_results[Field::ConfiguredCcr as usize].load(Relaxed)
            != handler_ccr
        || flt_results[Field::BuildMode as usize].load(Relaxed)
            != if INVALID_STATE {
                3
            } else if MANDATORY {
                2
            } else {
                1
            }
        || primask != 0
        || basepri != 0
        || faultmask != 0
        || flt_results[Field::Phase as usize].load(Relaxed) != 3
        || pc
            != if INVALID_STATE {
                addr_of!(invalid_target).addr() as u32
            } else {
                load.addr() as u32
            }
        || continuation
            != load.addr() as u32 + if INVALID_STATE { 2 } else { 4 }
        || first != LOAD_FIRST
        || second != LOAD_SECOND
        || xpsr & 0x0100_03FF != if INVALID_STATE { 0 } else { 0x0100_0000 }
        || r0 != 3
        || r1 != 0
        || r2 != psp_top as u32
        || progress != 1
        || continued != 0
        || positive
            != if INVALID_STATE {
                0xA55A_0007
            } else {
                0x4433_2211
            }
        || zero != FAULT_VALUE_SENTINEL
        || failure != 0
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
    systick.disable_interrupt();
    systick.set_reload(2_000_000 - 1);
    systick.clear_current();
    systick.enable_counter();
    record(Field::Status, 1);
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
    fail(0x88)
}
