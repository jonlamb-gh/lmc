// TODO

use crate::bsp::debug_console::DebugConsole;
use crate::bsp::hal::prelude::*;
use crate::bsp::hal::rcc::ResetConditions;
use crate::bsp::hal::serial::Serial;
use crate::bsp::hal::stm32f7x7;
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
    pub fn new() -> Self {
        let reset_conditions = ResetConditions::read_and_clear();

        let mut core_peripherals =
            cortex_m::Peripherals::take().expect("Failed to take cortex_m::Peripherals");
        let peripherals =
            stm32f7x7::Peripherals::take().expect("Failed to take stm32f7x7::Peripherals");

        core_peripherals.SCB.enable_icache();
        core_peripherals
            .SCB
            .enable_dcache(&mut core_peripherals.CPUID);

        let mut flash = peripherals.FLASH.constrain();
        let mut rcc = peripherals.RCC.constrain();

        let mut gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);
        let mut gpioc = peripherals.GPIOC.split(&mut rcc.ahb1);
        let mut gpiod = peripherals.GPIOD.split(&mut rcc.ahb1);

        let led_r = gpiob
            .pb14
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        let led_g = gpiob
            .pb0
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        let led_b = gpiob
            .pb7
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

        let usart3_tx = gpiod.pd8.into_af7(&mut gpiod.moder, &mut gpiod.afrh);
        let usart3_rx = gpiod.pd9.into_af7(&mut gpiod.moder, &mut gpiod.afrh);

        // default clock configuration runs at 16 MHz
        // let clocks = rcc.cfgr.freeze(&mut flash.acr);

        // TODO - enable OverDrive to get 216 MHz
        // configure maximum clock frequency at 200 MHz
        let clocks = rcc.cfgr.freeze_max(&mut flash.acr);

        let mut leds = Leds::new(led_r, led_g, led_b);
        for led in leds.iter_mut() {
            led.off();
        }

        // USART3 is routed up to the same USB port as the stlink
        // shows up as /dev/ttyACM0 for me
        let serial = Serial::usart3(
            peripherals.USART3,
            (usart3_tx, usart3_rx),
            115_200.bps(),
            clocks,
            &mut rcc.apb1,
        );

        Board {
            debug_console: DebugConsole::new(serial),
            leds,
            user_button: gpioc
                .pc13
                .into_pull_down_input(&mut gpioc.moder, &mut gpioc.pupdr),
            reset_conditions,
        }
    }
}
