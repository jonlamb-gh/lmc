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
// use crate::bsp::led::{Color, Leds};
use crate::rt::{entry, exception, ExceptionFrame};
use board::Board;
use core::fmt::Write;
use lm::Lm;
use panic_semihosting;

#[entry]
fn main() -> ! {
    let mut board = Board::new();
    // TODO - split board components

    let mut lm = Lm::new(board.lm_dac, board.lm_dac_enable);

    writeln!(board.debug_console, "Here").ok();

    lm.set_enabled(true);

    lm.set_dac(0x00F);

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
