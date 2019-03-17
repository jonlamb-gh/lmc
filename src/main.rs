// https://crates.io/crates/pwm-pca9685
// http://wiki.sunfounder.cc/index.php?title=PCA9685_16_Channel_12_Bit_PWM_Servo_Driver

// https://www.st.com/content/ccc/resource/technical/document/user_manual/98/2e/fa/4b/e0/82/43/b7/DM00105823.pdf/files/DM00105823.pdf/jcr:content/translations/en.DM00105823.pdf

#![no_std]
#![no_main]

extern crate cortex_m_rt as rt;

// mod display;
// mod lcm;

// use core::fmt::Write;
// use crate::lcm::Lcm;
// use crate::display::Display;
use crate::rt::{entry, exception, ExceptionFrame};
use panic_semihosting;
use pwm_pca9685::{Channel, OutputLogicState, Pca9685, SlaveAddr};
use stm32f1xx_hal::{i2c::BlockingI2c, i2c::Mode, pac, prelude::*};

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().expect("Failed to take stm32::Peripherals");

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = p.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = p.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = BlockingI2c::i2c1(
        p.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Standard { frequency: 100_000 },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let address = SlaveAddr::default();
    let mut pwm = Pca9685::new(i2c, address);
    pwm.set_prescale(3).unwrap();
    pwm.enable().unwrap();
    pwm.set_channel_on(Channel::C0, 0).unwrap();

    let mut val: u16 = 0;
    loop {
        pwm.set_channel_off(Channel::C0, val).unwrap();

        val = val.wrapping_add(1);

        cortex_m::asm::delay(1000);
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
