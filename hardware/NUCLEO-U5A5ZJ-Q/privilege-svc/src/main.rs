//! Cooperative Cortex-M33 privilege experiment for the NUCLEO-U5A5ZJ-Q.
//!
//! The stack transitions and little-endian SWD records live in transition.s.
//! Rust drives LD1 at privileged startup and after validated return.
//! See the dated experiment procedure for the reset assumptions, register
//! contracts, and limits of this experiment with the MPU disabled.

#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

use core::arch::global_asm;
use core::panic::PanicInfo;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt as _;
use stm32u5::stm32u5a5::{GPIOC, RCC};

global_asm!(
    include_str!("transition.s"),
    initialize_led = sym initialize_led,
    indicate_success = sym indicate_success,
);

/// Set LD1 solid before the test, while still in privileged Thread mode.
extern "C" fn initialize_led() {
    // SAFETY: Assembly calls once after validating privileged Thread mode
    // on MSP. RCC and GPIOC have no other live owners and neither handle
    // escapes this function.
    let (rcc, gpio) = unsafe { (RCC::steal(), GPIOC::steal()) };

    rcc.ahb2enr1().modify(|_, w| w.gpiocen().set_bit());
    let _ = rcc.ahb2enr1().read();
    gpio.bsrr().write(|w| w.bs7().set_bit());
    gpio.otyper().modify(|_, w| w.ot7().clear_bit());
    gpio.ospeedr().modify(|_, w| w.ospeed7().low_speed());
    gpio.pupdr().modify(|_, w| w.pupd7().floating());
    gpio.moder().modify(|_, w| w.mode7().output());
}

/// Blink LD1 in pairs after validated privileged Thread return on PSP.
/// The reset MSIS clock is nominally 4 MHz. SysTick is polled.
extern "C" fn indicate_success() -> ! {
    // SAFETY: Startup has returned and relinquished GPIOC. This path is
    // entered once. No handler acquires GPIOC or a core peripheral handle.
    let (gpio, mut systick) =
        unsafe { (GPIOC::steal(), cortex_m::Peripherals::steal().SYST) };

    systick.set_clock_source(SystClkSource::Core);
    systick.set_reload(500_000 - 1);
    systick.clear_current();
    systick.enable_counter();

    loop {
        for _ in 0..2 {
            gpio.bsrr().write(|w| w.bs7().set_bit());
            while !systick.has_wrapped() {}
            gpio.bsrr().write(|w| w.br7().set_bit());
            while !systick.has_wrapped() {}
        }
        for _ in 0..6 {
            while !systick.has_wrapped() {}
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    // No current Rust path can panic. If it does, stop the blink.
    loop {
        cortex_m::asm::nop();
    }
}
