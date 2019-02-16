use crate::board::{LMDac, LMDacEnablePin, DAC_CHANNEL};
use crate::bsp::hal::prelude::*;

pub struct Lm {
    dac: LMDac,
    dac_enable_pin: LMDacEnablePin,
    enabled: bool,
}

impl Lm {
    pub fn new(dac: LMDac, dac_enable_pin: LMDacEnablePin) -> Self {
        let mut lm = Lm {
            dac,
            dac_enable_pin,
            enabled: false,
        };

        lm.set_enabled(false);

        lm
    }

    // TODO - is it active low?
    pub fn set_enabled(&mut self, enable: bool) {
        if enable {
            self.dac_enable_pin.set_low();
        } else {
            self.dac_enable_pin.set_high();
        }
        self.enabled = enable;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_dac(&mut self, value: u16) {
        if self.enabled {
            self.dac.output(value, DAC_CHANNEL).expect("TODO")
        }
    }
}
