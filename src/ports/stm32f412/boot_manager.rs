//! Concrete boot manager construction and flash bank layout
//! for stm32f412
use crate::devices::{boot_manager::BootManager, cli::Cli};
use blue_hal::{drivers::stm32f4::{flash, rcc::Clocks, systick::SysTick}, hal::time, stm32pac};

use super::autogenerated::{self, devices, memory_map::{EXTERNAL_BANKS, MCU_BANKS}, pin_configuration::{self, *}, UDPATE_SIGNAL_ENABLED};
#[cfg(feature="ecdsa-verify")]
use crate::devices::image::EcdsaImageReader as ImageReader;
#[cfg(not(feature="ecdsa-verify"))]
use crate::devices::image::CrcImageReader as ImageReader;
use super::update_signal::{UpdateSignalWriter, initialize_rtc_backup_domain};

impl Default for BootManager<flash::McuFlash, ExternalFlash, Serial, ImageReader, UpdateSignalWriter> {
    fn default() -> Self { Self::new() }
}

impl BootManager<flash::McuFlash, ExternalFlash, Serial, ImageReader, UpdateSignalWriter> {
    pub fn new() -> Self {
        let mut peripherals = stm32pac::Peripherals::take().unwrap();
        let cortex_peripherals = cortex_m::Peripherals::take().unwrap();
        let mcu_flash = flash::McuFlash::new(peripherals.FLASH).unwrap();

        initialize_rtc_backup_domain(&mut peripherals.RCC, &mut peripherals.PWR);

        let (serial_pins, qspi_pins) = pin_configuration::pins(
                peripherals.GPIOA,
                peripherals.GPIOB,
                peripherals.GPIOC,
                peripherals.GPIOD,
                peripherals.GPIOE,
                peripherals.GPIOF,
                peripherals.GPIOG,
                peripherals.GPIOH,
                &mut peripherals.RCC,
            );
        let clocks = Clocks::hardcoded(peripherals.RCC);
        SysTick::init(cortex_peripherals.SYST, clocks);
        SysTick::wait(time::Seconds(1)); // Gives time for the flash chip to stabilize after powerup

        let serial = devices::construct_serial(
            serial_pins,
            clocks,
            peripherals.USART1,
            peripherals.USART2,
            peripherals.USART6)
            .expect("Demo app can't function without serial!");
        let cli = Cli::new(serial).unwrap();
        let external_flash = devices::construct_flash(qspi_pins, peripherals.QUADSPI);

        let update_signal = if UDPATE_SIGNAL_ENABLED {
            let rtc = peripherals.RTC;
            Some(UpdateSignalWriter::new(rtc))
        } else {
            None
        };

        BootManager {
            external_flash,
            mcu_flash,
            external_banks: &EXTERNAL_BANKS,
            mcu_banks: &MCU_BANKS,
            cli: Some(cli),
            boot_metrics: None,
            greeting: Some(autogenerated::DEMO_APP_GREETING),
            _marker: Default::default(),
            update_signal,
        }
    }
}
