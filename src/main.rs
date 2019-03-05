#![no_std]
#![no_main]

extern crate cortex_m_rt as rt;
extern crate stm32f407g_disc as bsp;

use core::fmt::Write;
use crate::bsp::hal::i2c::I2c;
use crate::bsp::hal::prelude::*;
use crate::bsp::hal::stm32;
use crate::rt::{entry, exception, ExceptionFrame};
use panic_semihosting;
use ssd1306::displayrotation::DisplayRotation;
use ssd1306::mode::TerminalMode;
use ssd1306::Builder;

#[entry]
fn main() -> ! {
    let peripherals = stm32::Peripherals::take().expect("Failed to take stm32::Peripherals");

    let rcc = peripherals.RCC.constrain();
    let gpiob = peripherals.GPIOB.split();

    let clocks = rcc.cfgr.sysclk(40.mhz()).freeze();

    let scl = gpiob
        .pb6
        .into_alternate_af4()
        .internal_pull_up(true)
        .set_open_drain();

    let sda = gpiob
        .pb7
        .into_alternate_af4()
        .internal_pull_up(true)
        .set_open_drain();

    let i2c = I2c::i2c1(peripherals.I2C1, (scl, sda), 400.khz(), clocks);

    // Set up the SSD1306 display at I2C address 0x3c
    let mut disp: TerminalMode<_> = Builder::new().with_i2c_addr(0x3c).connect_i2c(i2c).into();

    // Set display rotation to 180 degrees
    let _ = disp.set_rotation(DisplayRotation::Rotate180);

    // Init and clear the display
    let _ = disp.init();
    let _ = disp.clear();

    // Endless loop rendering ASCII characters all over the place
    loop {
        for c in (97..123).chain(64..91) {
            let _ = disp.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
        }
    }

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
