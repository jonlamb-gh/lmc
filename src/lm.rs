use crate::board::{LMDac, LMDacLatchPin, LMDacShutdownPin, DAC_CHANNEL};
use crate::bsp::hal::prelude::*;
use crate::dac_mcp4922::Channel;

// TODO - move latching into the DAC module
pub struct Lm {
    dac: LMDac,
    shutdown_pin: LMDacShutdownPin,
    latch_pin: LMDacLatchPin,
    enabled: bool,
}

impl Lm {
    pub fn new(dac: LMDac, shutdown_pin: LMDacShutdownPin, latch_pin: LMDacLatchPin) -> Self {
        let mut lm = Lm {
            dac,
            shutdown_pin,
            latch_pin,
            enabled: false,
        };

        lm.set_enabled(false);
        lm.latch();

        lm
    }

    pub fn enable(&mut self) {
        self.set_enabled(true);
    }

    pub fn set_enabled(&mut self, enable: bool) {
        if enable {
            self.shutdown_pin.set_high();
        } else {
            self.shutdown_pin.set_low();
        }
        self.enabled = enable;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_dac(&mut self, value: u16) {
        if self.enabled {
            self.unlatch();

            let (ch_a_val, ch_b_val) = if DAC_CHANNEL == Channel::ChannelA {
                (value, 0)
            } else {
                (0, value)
            };

            self.dac.output_ab(ch_a_val, ch_b_val).expect("TODO");
            self.latch();
        }
    }

    fn latch(&mut self) {
        self.latch_pin.set_low();
    }

    fn unlatch(&mut self) {
        self.latch_pin.set_high();
    }
}
