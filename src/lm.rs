use crate::board::{LMDac, LMDacLatchPin, LMDacShutdownPin, LMPulseTimer, DAC_CHANNEL};
use crate::bsp::hal::prelude::*;
use crate::bsp::hal::time::Hertz;
use crate::dac_mcp4922::Channel;

// TODO - move latching into the DAC module
pub struct Lm {
    dac: LMDac,
    pulse_timer: LMPulseTimer,
    shutdown_pin: LMDacShutdownPin,
    latch_pin: LMDacLatchPin,
    enabled: bool,
    power: u16,
    pulse: Option<Hertz>,
    pulse_state: bool,
}

impl Lm {
    pub fn new(
        dac: LMDac,
        pulse_timer: LMPulseTimer,
        shutdown_pin: LMDacShutdownPin,
        latch_pin: LMDacLatchPin,
    ) -> Self {
        let mut lm = Lm {
            dac,
            pulse_timer,
            shutdown_pin,
            latch_pin,
            enabled: false,
            power: 0,
            pulse: None,
            pulse_state: false,
        };

        lm.latch();
        lm.set_dac(0);
        lm.set_enabled(false);

        lm
    }

    pub fn enable(&mut self) {
        self.set_enabled(true);
    }

    pub fn set_enabled(&mut self, enable: bool) {
        if enable {
            self.shutdown_pin.set_high();
        } else {
            self.set_dac(0);
            self.shutdown_pin.set_low();
            self.power = 0;
            self.pulse = None;
            self.pulse_state = false;
        }
        self.enabled = enable;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn powered(&self) -> bool {
        if self.power == 0 {
            false
        } else {
            true
        }
    }

    pub fn pulse_state(&mut self) -> Option<bool> {
        if self.enabled {
            if self.power != 0 && self.pulse.is_some() {
                return Some(self.pulse_state);
            }
        }

        None
    }

    pub fn power_off(&mut self) {
        self.set_power_pulse(0, None);
    }

    // off if power == 0
    pub fn set_power_pulse(&mut self, power: u16, pulse: Option<Hertz>) {
        if self.enabled {
            self.set_dac(power);
            self.power = power;
            self.pulse_state = false;

            if power != 0 {
                if let Some(p) = pulse {
                    self.pulse_timer.start(p);
                    self.pulse_state = true;
                }
                self.pulse = pulse;
            } else {
                self.pulse = None;
            }
        }
    }

    // Arrange for this to be called frequently
    pub fn update_pulse(&mut self) {
        if let Some(prev_state) = self.pulse_state() {
            if self.pulse_timer.wait().is_ok() {
                if prev_state {
                    // on -> off
                    self.set_dac(0);
                } else {
                    // off -> on
                    self.set_dac(self.power);
                }

                self.pulse_state = !prev_state;
            }
        }
    }

    fn set_dac(&mut self, val: u16) {
        self.unlatch();
        let (ch_a_val, ch_b_val) = if DAC_CHANNEL == Channel::ChannelA {
            (val, 0)
        } else {
            (0, val)
        };
        self.dac
            .output_ab(ch_a_val, ch_b_val)
            .expect("TODO - DAC error");
        self.latch();
    }

    fn latch(&mut self) {
        self.latch_pin.set_low();
    }

    fn unlatch(&mut self) {
        self.latch_pin.set_high();
    }
}
