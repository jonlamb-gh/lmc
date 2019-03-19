use core::cmp;
use crate::debounce_input::DebounceInput;
use embedded_hal::adc::{Channel, OneShot};
use embedded_hal::digital::InputPin;
use nb::block;
use stm32f1xx_hal::adc::Adc;
use stm32f1xx_hal::pac::ADC1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Button {
    B0,
    B1,
    B2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AIn {
    AIN0,
    AIN1,
}

pub struct Input<BTN0, BTN1, BTN2, AIN0, AIN1> {
    btn0: BTN0,
    btn1: BTN1,
    btn2: BTN2,
    adc: Adc<ADC1>,
    ain0: AIN0,
    ain1: AIN1,
}

impl<BTN0, BTN1, BTN2, AIN0, AIN1> Input<BTN0, BTN1, BTN2, AIN0, AIN1>
where
    BTN0: InputPin,
    BTN1: InputPin,
    BTN2: InputPin,
    // TODO - make ADC generic
    AIN0: Channel<ADC1, ID = u8>,
    AIN1: Channel<ADC1, ID = u8>,
{
    pub fn new(btn0: BTN0, btn1: BTN1, btn2: BTN2, adc: Adc<ADC1>, ain0: AIN0, ain1: AIN1) -> Self {
        Input {
            btn0,
            btn1,
            btn2,
            adc,
            ain0,
            ain1,
        }
    }

    pub fn button(&self, btn: Button) -> bool {
        match btn {
            Button::B0 => self.btn0.is_low_debounce(),
            Button::B1 => self.btn1.is_low_debounce(),
            Button::B2 => self.btn2.is_low_debounce(),
        }
    }

    pub fn button_wait(&self, btn: Button) -> bool {
        if self.button(btn) {
            while self.button(btn) {}
            true
        } else {
            false
        }
    }

    pub fn ain(&mut self, ain: AIn) -> u16 {
        match ain {
            AIn::AIN0 => block!(self.adc.read(&mut self.ain0)).unwrap(),
            AIn::AIN1 => block!(self.adc.read(&mut self.ain1)).unwrap(),
        }
    }

    pub fn ain_map(&mut self, ain: AIn, out_min: u32, out_max: u32) -> u32 {
        let in_min: u32 = 0;
        let in_max: u32 = 4095;
        let x = cmp::min(self.ain(ain) as u32, in_max);

        (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
    }
}
