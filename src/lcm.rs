use core::cmp;
use crate::hal::pac::TIM2;
use crate::hal::time::Hertz;
use crate::hal::timer::Timer;
use embedded_hal::timer::CountDown;
use embedded_hal::{blocking, digital};
use pwm_pca9685::{Channel, OutputLogicState, Pca9685, SlaveAddr};
// use crate::hal::timer::{Event as TimerEvent, Timer};
// use crate::hal::pac::interrupt;

const PWM_MAX: u16 = 4095;

#[derive(Debug, Clone, Copy)]
pub enum Freq {
    Continuous,
    Periodic(Hertz),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum State {
    Error,
    Off,
    On,
}

#[derive(Debug, Clone, Copy)]
pub struct Status {
    // TODO - is state useful, what fails? low-level hw bits
    state: State,
    pwm: u16,
    pwm_oe: bool,
    pwm_relay: bool,
    freq: Freq,
}

pub struct Lcm<I2C, OE, RLY> {
    pwm_drv: Pca9685<I2C>,
    pwm_oe: OE,
    pwm_relay: RLY,
    timer: Timer<TIM2>,
    pwm: u16,
    freq: Freq,
}

// TODO - make TIM/TIMER generic, but listen()/etc are not traits (yet)?
impl<I2C, OE, RLY, E> Lcm<I2C, OE, RLY>
where
    I2C: blocking::i2c::Write<Error = E>,
    E: core::fmt::Debug,
    OE: digital::StatefulOutputPin + digital::OutputPin,
    RLY: digital::StatefulOutputPin + digital::OutputPin,
{
    pub fn new(i2c: I2C, oe: OE, relay: RLY, timer: Timer<TIM2>) -> Self {
        let address = SlaveAddr::default();
        let mut lcm = Lcm {
            pwm_drv: Pca9685::new(i2c, address),
            pwm_oe: oe,
            pwm_relay: relay,
            timer,
            pwm: 0,
            freq: Freq::Continuous,
        };

        lcm.relay_disable();
        lcm.pwm_disable();

        // Max prescale = 1526 Hz
        lcm.pwm_drv.disable().unwrap();
        lcm.pwm_drv.set_prescale(3).unwrap();
        lcm.pwm_drv.enable().unwrap();

        // TODO
        lcm.pwm_drv
            .set_output_logic_state(OutputLogicState::Direct)
            .unwrap();

        lcm.pwm_drv.set_channel_full_off(Channel::All).unwrap();

        lcm
    }

    pub fn status(&self) -> Status {
        // TODO
        let state = if self.relay_enabled() {
            State::On
        } else {
            State::Off
        };

        Status {
            state,
            pwm: self.pwm(),
            pwm_oe: self.pwm_enabled(),
            pwm_relay: self.relay_enabled(),
            freq: self.freq(),
        }
    }

    pub fn set_freq(&mut self, freq: Freq) {
        self.freq = freq;

        if let Freq::Periodic(f) = self.freq {
            self.timer.start(f);
        // self.timer.listen(TimerEvent::Update);
        } else {
            // self.timer.cancel();
            // self.timer.unlisten(TimerEvent::Update);
        }
    }

    pub fn freq(&self) -> Freq {
        self.freq
    }

    pub fn set_pwm(&mut self, pwm: u16) {
        self.pwm = cmp::min(pwm, PWM_MAX);

        self.pwm_drv.set_channel_on(Channel::All, 0).unwrap();
        self.pwm_drv
            .set_channel_off(Channel::All, self.pwm)
            .unwrap();
    }

    pub fn pwm(&self) -> u16 {
        self.pwm
    }

    pub fn pwm_enabled(&self) -> bool {
        self.pwm_oe.is_set_low()
    }

    pub fn pwm_disable(&mut self) {
        self.pwm_oe.set_high();
    }

    pub fn pwm_enable(&mut self) {
        self.pwm_oe.set_low();
    }

    pub fn relay_enabled(&self) -> bool {
        self.pwm_relay.is_set_high()
    }

    pub fn relay_disable(&mut self) {
        self.pwm_relay.set_low();
    }

    pub fn relay_enable(&mut self) {
        self.pwm_relay.set_high();
    }
}

impl Status {
    pub fn state(&self) -> State {
        self.state
    }

    pub fn pwm(&self) -> u16 {
        self.pwm
    }

    pub fn pwm_oe(&self) -> bool {
        self.pwm_oe
    }

    pub fn pwm_relay(&self) -> bool {
        self.pwm_relay
    }

    pub fn freq(&self) -> Freq {
        self.freq
    }
}
