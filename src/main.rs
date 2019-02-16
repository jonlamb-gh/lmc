#![no_std]
#![no_main]
#![feature(const_fn)]

// extern crate cortex_m;
use cortex_m_rt as rt;
// extern crate cortex_m_semihosting;
// extern crate embedded_hal;
use oxcc_nucleo_f767zi as bsp;
// extern crate panic_semihosting;

mod board;

use core::fmt::Write;
use crate::bsp::debug_console::DebugConsole;
use crate::bsp::led::{Color, Leds};
use crate::rt::{entry, exception, ExceptionFrame};
use panic_semihosting::*;

const DEBUG_WRITE_FAILURE: &str = "Failed to write to debug_console";

#[entry]
fn main() -> ! {
    panic!("HERE");
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
