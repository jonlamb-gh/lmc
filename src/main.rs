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

// use crate::bsp::debug_console::DebugConsole;
// use crate::bsp::led::{Color, Leds};
use crate::rt::{entry, exception, ExceptionFrame};
use board::Board;
use core::fmt::Write;
use panic_semihosting;

#[entry]
fn main() -> ! {
    let mut board = Board::new();
    // TODO - split board components

    writeln!(board.debug_console, "Here").ok();

    // TODO
    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
