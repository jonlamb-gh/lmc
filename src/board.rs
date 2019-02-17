use crate::bsp::debug_console::DebugConsole;
use crate::bsp::hal::adc::Adc;
use crate::bsp::hal::adc::Channel as AdcChannel;
use crate::bsp::hal::adc::Prescaler as AdcPrescaler;
use crate::bsp::hal::adc::Resolution as AdcResolution;
use crate::bsp::hal::adc::SampleTime as AdcSampleTime;
use crate::bsp::hal::delay::Delay;
use crate::bsp::hal::gpio::gpioa::{PA4, PA5, PA6, PA7};
use crate::bsp::hal::gpio::gpiod::{PD12, PD13};
use crate::bsp::hal::gpio::{Output, PushPull, AF5};
use crate::bsp::hal::prelude::*;
use crate::bsp::hal::rcc::ResetConditions;
use crate::bsp::hal::serial::Serial;
use crate::bsp::hal::spi::Spi;
use crate::bsp::hal::stm32f7x7;
use crate::bsp::hal::stm32f7x7::{ADC1, SPI1, TIM2};
use crate::bsp::hal::timer::Timer;
use crate::bsp::led::Leds;
use crate::bsp::UserButtonPin;
use crate::bsp::{AnalogInput0Pin, AnalogInput1Pin};
use crate::dac_mcp4922::MODE as DAC_MODE;
use crate::dac_mcp4922::{Channel as DACChannel, Mcp4922};

pub type LMPulseTimer = Timer<TIM2>;

pub type LMDacShutdownPin = PD12<Output<PushPull>>;
pub type LMDacLatchPin = PD13<Output<PushPull>>;

pub type LMSpiSckPin = PA5<AF5>;
pub type LMSpiMisoPin = PA6<AF5>;
pub type LMSpiMosiPin = PA7<AF5>;
pub type LMSpiNssPin = PA4<Output<PushPull>>;

pub type LMSpi = Spi<SPI1, (PA5<AF5>, PA6<AF5>, PA7<AF5>)>;

pub type LMDac = Mcp4922<LMSpi, LMSpiNssPin>;

// ADC1 pins, potentiometer 0/1
pub type PotSensor0Pin = AnalogInput0Pin;
pub type PotSensor1Pin = AnalogInput1Pin;

// pub type Button0Pin = PB4<Output<PushPull>>;
// pub type Button1Pin = PD11<Output<PushPull>>;
// pub type Button2Pin = PE2<Output<PushPull>>;

pub const DAC_CHANNEL: DACChannel = DACChannel::ChannelA;

pub const ADC_PRESCALER: AdcPrescaler = AdcPrescaler::Prescaler4;
pub const ADC_SAMPLE_TIME: AdcSampleTime = AdcSampleTime::Cycles480;
pub const ADC_RESOLUTION: AdcResolution = AdcResolution::Bits12;

pub struct Board {
    pub debug_console: DebugConsole,
    pub leds: Leds,
    pub user_button: UserButtonPin,
    pub delay: Delay,
    // pub wdg: Iwdg<IWDG>,
    pub reset_conditions: ResetConditions,
    // TODO - sub structs for pins/etc
    pub lm_dac: LMDac,
    pub lm_pulse_timer: LMPulseTimer,
    pub lm_dac_shutdown_pin: LMDacShutdownPin,
    pub lm_dac_latch_pin: LMDacLatchPin,
    //
    pub pot_reader: PotReader,
}

// Owns ADC1
pub struct PotReader {
    pot0_pin: PotSensor0Pin,
    pot1_pin: PotSensor1Pin,
    adc1: Adc<ADC1>,
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
        let mut c_adc = peripherals.C_ADC;

        let mut gpioa = peripherals.GPIOA.split(&mut rcc.ahb1);
        let mut gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);
        let mut gpioc = peripherals.GPIOC.split(&mut rcc.ahb1);
        let mut gpiod = peripherals.GPIOD.split(&mut rcc.ahb1);

        let lm_dac_shutdown_pin = gpiod
            .pd12
            .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);
        let lm_dac_latch_pin = gpiod
            .pd13
            .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);

        let lm_sck: LMSpiSckPin = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let lm_miso: LMSpiMisoPin = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let lm_mosi: LMSpiMosiPin = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let lm_nss: LMSpiNssPin = gpioa
            .pa4
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

        let pot0_pin = gpioa
            .pa3
            .into_analog_input(&mut gpioa.moder, &mut gpioa.pupdr);
        let pot1_pin = gpioc
            .pc0
            .into_analog_input(&mut gpioc.moder, &mut gpioc.pupdr);

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

        // TODO - this can be moved into the HAL once it's aware of the clocks
        let adc_clock = match ADC_PRESCALER {
            AdcPrescaler::Prescaler2 => clocks.pclk2().0 / 2,
            AdcPrescaler::Prescaler4 => clocks.pclk2().0 / 4,
            AdcPrescaler::Prescaler6 => clocks.pclk2().0 / 6,
            AdcPrescaler::Prescaler8 => clocks.pclk2().0 / 8,
        };
        assert!(adc_clock <= 30_000_000);

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
            4.mhz().into(),
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
            delay: Delay::new(core_peripherals.SYST, clocks),
            lm_dac: Mcp4922::new(lm_spi, lm_nss),
            lm_pulse_timer: LMPulseTimer::tim2(peripherals.TIM2, 1.hz(), clocks, &mut rcc.apb1),
            lm_dac_shutdown_pin,
            lm_dac_latch_pin,
            pot_reader: PotReader {
                pot0_pin,
                pot1_pin,
                adc1: Adc::adc1(
                    peripherals.ADC1,
                    &mut c_adc,
                    &mut rcc.apb2,
                    ADC_PRESCALER,
                    ADC_RESOLUTION,
                ),
            },
        }
    }
}

impl PotReader {
    pub fn read_pot0(&mut self) -> u16 {
        // AnalogInput0Pin, PA3, ADC123_IN3
        self.adc1.read(AdcChannel::Adc123In3, ADC_SAMPLE_TIME)
    }

    pub fn read_pot1(&mut self) -> u16 {
        // AnalogInput1Pin, PC0, ADC123_IN10
        self.adc1.read(AdcChannel::Adc123In10, ADC_SAMPLE_TIME)
    }
}
