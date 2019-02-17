#![no_std]
#![no_main]
#![feature(const_fn)]

// extern crate cortex_m;
extern crate cortex_m_rt as rt;
// extern crate cortex_m_semihosting;
// extern crate embedded_hal;
extern crate oxcc_nucleo_f767zi as bsp;
// extern crate panic_semihosting;

mod board;
mod dac_mcp4922;
mod lm;

// use crate::bsp::debug_console::DebugConsole;
use core::fmt::Write;
use crate::board::Board;
use crate::bsp::hal::prelude::*;
use crate::bsp::led::Color;
use crate::lm::Lm;
use crate::rt::{entry, exception, ExceptionFrame};

#[allow(unused_imports)]
use panic_semihosting;

#[entry]
fn main() -> ! {
    let mut board = Board::new();
    // TODO - split board components

    // Put into low-power mode by default, must enable first
    let mut lm = Lm::new(
        board.lm_dac,
        board.lm_dac_shutdown_pin,
        board.lm_dac_latch_pin,
    );

    board.leds[Color::Blue].on();

    writeln!(board.debug_console, "Starting").ok();

    lm.set_enabled(true);

    loop {
        while board.user_button.is_high() {
            board.leds[Color::Red].on();
            assert_eq!(lm.enabled(), true);
            lm.set_dac(0xFFF);
        }

        lm.set_dac(0);
        board.leds[Color::Red].off();
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
