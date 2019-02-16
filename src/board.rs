// TODO

use crate::bsp::debug_console::DebugConsole;
use crate::bsp::hal::rcc::ResetConditions;
use crate::bsp::led::Leds;
use crate::bsp::UserButtonPin;

pub struct Board {
    pub debug_console: DebugConsole,
    pub leds: Leds,
    pub user_button: UserButtonPin,
    // pub wdg: Iwdg<IWDG>,
    pub reset_conditions: ResetConditions,
}

impl Board {
    // pub fn new() -> Self {}
}
