//! Standalone LD1 bring-up for the NUCLEO-U5A5ZJ-Q.
//!
//! Assumes reset-time clocks and default active-high PC7 board routing.
//! SysTick is polled at the nominal 4 MHz core clock. See FIRMWARE.md.

#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

use core::panic::PanicInfo;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::entry;
use stm32u5::stm32u5a5::{GPIOC, RCC};

const CORE_CLOCK_HZ: u32 = 4_000_000;
const HALF_PERIOD_TICKS: u32 = CORE_CLOCK_HZ / 2;

#[entry]
fn main() -> ! {
    // SAFETY: This is the only entry point. No handlers or other code acquire
    // RCC, GPIOC or the core peripheral handles. Each is taken once here.
    let (rcc, gpio, mut systick) = unsafe {
        (
            RCC::steal(),
            GPIOC::steal(),
            cortex_m::Peripherals::steal().SYST,
        )
    };

    rcc.ahb2enr1().modify(|_, w| w.gpiocen().set_bit());
    // The readback provides the clock-enable delay before GPIO access.
    let _ = rcc.ahb2enr1().read();

    // Preload the inactive level before enabling the PC7 output driver.
    gpio.bsrr().write(|w| w.br7().set_bit());
    gpio.otyper().modify(|_, w| w.ot7().clear_bit());
    gpio.ospeedr().modify(|_, w| w.ospeed7().low_speed());
    gpio.pupdr().modify(|_, w| w.pupd7().floating());
    gpio.moder().modify(|_, w| w.mode7().output());

    // Reset leaves TICKINT clear. Poll COUNTFLAG without a SysTick handler.
    systick.set_clock_source(SystClkSource::Core);
    systick.set_reload(HALF_PERIOD_TICKS - 1);
    systick.clear_current();
    systick.enable_counter();

    loop {
        while !systick.has_wrapped() {}
        gpio.bsrr().write(|w| w.bs7().set_bit());
        while !systick.has_wrapped() {}
        gpio.bsrr().write(|w| w.br7().set_bit());
    }
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        cortex_m::asm::nop();
    }
}
