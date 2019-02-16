use crate::bsp::debug_console::DebugConsole;
use crate::bsp::hal::prelude::*;
use crate::bsp::hal::rcc::ResetConditions;
use crate::bsp::hal::serial::Serial;
use crate::bsp::hal::stm32f7x7;
use crate::bsp::led::Leds;
use crate::bsp::UserButtonPin;

use crate::bsp::hal::gpio::gpioa::{PA4, PA5, PA6, PA7};
use crate::bsp::hal::gpio::gpiod::PD12;
use crate::bsp::hal::gpio::{Output, PushPull, AF5};
use crate::bsp::hal::spi::Spi;
use crate::bsp::hal::stm32f7x7::SPI1;
use crate::dac_mcp4922::MODE as DAC_MODE;
use crate::dac_mcp4922::{Channel as DACChannel, Mcp4922};

pub type LMDacEnablePin = PD12<Output<PushPull>>;

pub type LMSpiSckPin = PA5<AF5>;
pub type LMSpiMisoPin = PA6<AF5>;
pub type LMSpiMosiPin = PA7<AF5>;
pub type LMSpiNssPin = PA4<Output<PushPull>>;

pub type LMSpi = Spi<SPI1, (PA5<AF5>, PA6<AF5>, PA7<AF5>)>;

pub type LMDac = Mcp4922<LMSpi, LMSpiNssPin>;

pub const DAC_CHANNEL: DACChannel = DACChannel::ChannelA;

pub struct Board {
    pub debug_console: DebugConsole,
    pub leds: Leds,
    pub user_button: UserButtonPin,
    // pub wdg: Iwdg<IWDG>,
    pub reset_conditions: ResetConditions,
    pub lm_dac: LMDac,
    pub lm_dac_enable: LMDacEnablePin,
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

        let mut gpioa = peripherals.GPIOA.split(&mut rcc.ahb1);
        let mut gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);
        let mut gpioc = peripherals.GPIOC.split(&mut rcc.ahb1);
        let mut gpiod = peripherals.GPIOD.split(&mut rcc.ahb1);

        let lm_dac_enable = gpiod
            .pd12
            .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);

        let lm_sck: LMSpiSckPin = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let lm_miso: LMSpiMisoPin = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let lm_mosi: LMSpiMosiPin = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let lm_nss: LMSpiNssPin = gpioa
            .pa4
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

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

        let lm_spi: LMSpi = Spi::spi1(
            peripherals.SPI1,
            (lm_sck, lm_miso, lm_mosi),
            DAC_MODE,
            1.mhz().into(),
            clocks,
            &mut rcc.apb2,
        );

        Board {
            debug_console: DebugConsole::new(serial),
            leds,
            user_button: gpioc
                .pc13
                .into_pull_down_input(&mut gpioc.moder, &mut gpioc.pupdr),
            reset_conditions,
            lm_dac: Mcp4922::new(lm_spi, lm_nss),
            lm_dac_enable,
        }
    }
}
