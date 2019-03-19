use core::fmt::Write;
use crate::lcm::{Freq, State, Status};
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use embedded_hal::blocking;
use heapless::consts::U32;
use heapless::String;
use ssd1306::mode::GraphicsMode;
use ssd1306::prelude::*;
use ssd1306::Builder;

const SLAVE_ADDRESS: u8 = 0x3C;

pub struct Display<I2C>
where
    I2C: blocking::i2c::Write,
{
    drv: GraphicsMode<I2cInterface<I2C>>,
}

impl<I2C> Display<I2C>
where
    I2C: embedded_hal::blocking::i2c::Write,
{
    // TODO - error/result handling
    pub fn new(i2c: I2C) -> Self {
        let mut drv: GraphicsMode<I2cInterface<I2C>> = Builder::new()
            .with_size(DisplaySize::Display128x32)
            .with_rotation(DisplayRotation::Rotate180)
            .with_i2c_addr(SLAVE_ADDRESS)
            .connect_i2c(i2c)
            .into();

        drv.init().unwrap();
        drv.clear();

        let (w, h) = drv.get_dimensions();
        assert_eq!(w, 128);
        assert_eq!(h, 32);

        drv.flush().unwrap();

        Display { drv }
    }

    pub fn draw_lcm_status(&mut self, status: &Status) {
        // TODO - custom fmt for Status

        self.drv.clear();

        let mut value_str: String<U32> = String::new();

        value_str.clear();
        write!(value_str, " PWM: {}", status.pwm()).ok();

        self.drv.draw(
            Font6x8::render_str(&value_str)
                .translate(Coord::new(0, 0))
                .into_iter(),
        );

        value_str.clear();
        match status.state() {
            State::Error => write!(value_str, "STAT: ERR").ok(),
            State::Off => write!(value_str, "STAT: OFF").ok(),
            State::On => write!(value_str, "STAT: ON").ok(),
        };

        self.drv.draw(
            Font6x8::render_str(&value_str)
                .translate(Coord::new(68, 0))
                .into_iter(),
        );

        value_str.clear();
        match status.freq() {
            Freq::Continuous => write!(value_str, "FREQ: CONT").ok(),
            Freq::Periodic(freq) => write!(value_str, "FREQ: {}", freq.0).ok(),
        };

        self.drv.draw(
            Font6x8::render_str(&value_str)
                .translate(Coord::new(0, 12))
                .into_iter(),
        );

        value_str.clear();
        match status.pwm_oe() {
            true => write!(value_str, "  OE: ON").ok(),
            false => write!(value_str, "  OE: OFF").ok(),
        };

        self.drv.draw(
            Font6x8::render_str(&value_str)
                .translate(Coord::new(0, 24))
                .into_iter(),
        );

        value_str.clear();
        match status.pwm_relay() {
            true => write!(value_str, " RLY: ON").ok(),
            false => write!(value_str, " RLY: OFF").ok(),
        };

        self.drv.draw(
            Font6x8::render_str(&value_str)
                .translate(Coord::new(68, 24))
                .into_iter(),
        );

        self.drv.flush().unwrap();
    }
}
