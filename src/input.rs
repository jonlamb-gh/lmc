use embedded_hal::adc::Channel;
use embedded_hal::digital::InputPin;
use stm32f1xx_hal::pac::ADC1;

pub enum Button {
    B0,
    B1,
    B2,
}

pub struct Input<BTN0, BTN1, BTN2, AIN0, AIN1> {
    btn0: BTN0,
    btn1: BTN1,
    btn2: BTN2,
    ain0: AIN0,
    ain1: AIN1,
}

impl<BTN0, BTN1, BTN2, AIN0, AIN1> Input<BTN0, BTN1, BTN2, AIN0, AIN1>
where
    BTN0: InputPin,
    BTN1: InputPin,
    BTN2: InputPin,
    // TODO - make ADC generic
    AIN0: Channel<ADC1>,
    AIN1: Channel<ADC1>,
{
    pub fn new(btn0: BTN0, btn1: BTN1, btn2: BTN2, ain0: AIN0, ain1: AIN1) -> Self {
        Input {
            btn0,
            btn1,
            btn2,
            ain0,
            ain1,
        }
    }

    pub fn button(&mut self, button: Button) -> bool {
        // TODO - debounce input
        false
    }

    // pub fn ain(...) -> u16
}
