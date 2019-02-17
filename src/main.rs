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
use crate::bsp::hal::time::Hertz;
use crate::bsp::led::Color;
use crate::lm::Lm;
use crate::rt::{entry, exception, ExceptionFrame};

#[allow(unused_imports)]
use panic_semihosting;

// TODO - fix release mode Spi/DAC issues
// local panic impl over uart

#[entry]
fn main() -> ! {
    let board = Board::new();
    // TODO - split board components

    let mut pot_reader = board.pot_reader;

    let mut delay = board.delay;
    let mut dbgcon = board.debug_console;
    let mut leds = board.leds;

    // Put into low-power mode by default, must enable first
    let mut lm = Lm::new(
        board.lm_dac,
        board.lm_pulse_timer,
        board.lm_dac_shutdown_pin,
        board.lm_dac_latch_pin,
    );

    leds[Color::Blue].on();

    writeln!(dbgcon, "--- INIT ---").ok();

    loop {
        if lm.enabled() == false {
            writeln!(dbgcon, "Enabling lm").ok();
            lm.enable();
            delay.delay_ms(5_u32);
            lm.power_off();
            delay.delay_ms(50_u32);
        }

        // TODO - button debounce(...)
        let power = pot_reader.read_pot0();
        let enable = board.user_button.is_high();
        let pulse_raw = pot_reader.read_pot1();

        let pulse_freq = {
            let div = pulse_raw / 14;
            if div == 0 {
                None
            } else {
                Some(Hertz(div as u32))
            }
        };

        if enable == false {
            if lm.powered() {
                writeln!(dbgcon, "power off").ok();
            }

            lm.power_off();
            leds[Color::Red].off();
        } else {
            leds[Color::Red].on();

            if lm.powered() == false {
                writeln!(
                    dbgcon,
                    "power on - power {} - pulse_raw {} - pulse {:?}",
                    power, pulse_raw, pulse_freq,
                ).ok();
                lm.set_power_pulse(power, pulse_freq);
            }

            // TODO - cont. adjustment mode
            // lm.set_power_pulse(power, None);

            lm.update_pulse();
        }
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
