#![no_std]
#![no_main]

extern crate cortex_m_rt as rt;
extern crate stm32f1xx_hal as hal;

mod debounce_input;
mod display;
mod input;
mod lcm;

use core::fmt::Write;
use crate::display::Display;
use crate::hal::adc::Adc;
use crate::hal::gpio::State;
use crate::hal::i2c::{BlockingI2c, Mode};
use crate::hal::pac as stm32;
use crate::hal::pac::USART2;
use crate::hal::prelude::*;
use crate::hal::serial::{Rx, Serial, Tx};
use crate::hal::time::Hertz;
use crate::input::{AIn, Button, Input};
use crate::lcm::{Freq, Lcm};
use crate::rt::{entry, exception, ExceptionFrame};
use nb::block;
use panic_semihosting;

// TODO - bsp.rs with pin type mappings for the nucleo-64 board
// use crate::hal::gpioa::{PA2, PA3};

// struct DebugConsole(Serial<stm32::USART2, (PA2, PA3)>);
struct DebugConsole {
    tx: Tx<USART2>,
    _rx: Rx<USART2>,
}

impl Write for DebugConsole {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for &b in s.as_bytes() {
            block!(self.tx.write(b as _)).ok();
        }
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    let p = stm32::Peripherals::take().expect("Failed to take stm32::Peripherals");

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    let clocks = rcc
        .cfgr
        .sysclk(64.mhz())
        .pclk1(32.mhz())
        .freeze(&mut flash.acr);

    let mut afio = p.AFIO.constrain(&mut rcc.apb2);

    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = p.GPIOB.split(&mut rcc.apb2);

    let mut led = gpioa.pa5.into_push_pull_output(&mut gpioa.crl);
    led.set_low();

    // USART2
    let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let rx = gpioa.pa3;

    let serial = Serial::usart2(
        p.USART2,
        (tx, rx),
        &mut afio.mapr,
        115_200.bps(),
        clocks,
        &mut rcc.apb1,
    );

    let (tx, rx) = serial.split();
    let mut stdout = DebugConsole { tx, _rx: rx };

    // PB4, D5
    // PB5, D4
    // TODO - need an external pull-up resistor on OE or use llc
    let pwm_oe = gpiob
        .pb4
        .into_push_pull_output_with_state(&mut gpiob.crl, State::High);
    let pwm_relay = gpiob
        .pb5
        .into_push_pull_output_with_state(&mut gpiob.crl, State::Low);

    // I2C1
    let pwm_scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let pwm_sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let pwm_i2c = BlockingI2c::i2c1(
        p.I2C1,
        (pwm_scl, pwm_sda),
        &mut afio.mapr,
        Mode::Standard { frequency: 100_000 },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let mut lcm = Lcm::new(pwm_i2c, pwm_oe, pwm_relay);

    // I2C2
    let disp_scl = gpiob.pb10.into_alternate_open_drain(&mut gpiob.crh);
    let disp_sda = gpiob.pb11.into_alternate_open_drain(&mut gpiob.crh);

    let disp_i2c = BlockingI2c::i2c2(
        p.I2C2,
        (disp_scl, disp_sda),
        Mode::Standard { frequency: 100_000 },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let mut disp = Display::new(disp_i2c);

    // ADC_0, PA0, A0
    // ADC_1, PA1, A1
    // ADC_4, PA4, A2
    let ain0 = gpioa.pa0.into_analog(&mut gpioa.crl);
    let ain1 = gpioa.pa1.into_analog(&mut gpioa.crl);

    let adc = Adc::adc1(p.ADC1, &mut rcc.apb2);

    // PA10, D2
    // PA8, D7
    // PA9, D8
    let btn0_in = gpioa.pa10.into_pull_up_input(&mut gpioa.crh);
    let btn1_in = gpioa.pa8.into_pull_up_input(&mut gpioa.crh);
    let btn2_in = gpioa.pa9.into_pull_up_input(&mut gpioa.crh);

    let mut input = Input::new(btn0_in, btn1_in, btn2_in, adc, ain0, ain1);

    writeln!(stdout, "Starting").ok();

    led.set_low();
    loop {
        if input.button_wait(Button::B2) {
            if lcm.pwm_enabled() {
                lcm.pwm_disable();
            } else {
                lcm.pwm_enable();
            }
        }

        if input.button_wait(Button::B1) {
            lcm.pwm_disable();
            lcm.relay_enable();
            led.set_high();
        }

        if input.button_wait(Button::B0) {
            led.set_low();
            lcm.pwm_disable();
            lcm.relay_disable();
        }

        let pwm_sp = input.ain(AIn::AIN0);

        let raw_freq = input.ain_map(AIn::AIN1, 0, 100) as u16;
        let freq_sp = if raw_freq == 0 {
            Freq::Continuous
        } else {
            Freq::Periodic(Hertz(raw_freq as u32))
        };

        lcm.set_pwm(pwm_sp);

        lcm.set_freq(freq_sp);

        let status = lcm.status();

        disp.draw_lcm_status(&status);
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
