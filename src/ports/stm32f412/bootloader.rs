//! Concrete bootloader construction and flash bank layout for stm32f412
use crate::{devices::bootloader::Bootloader, error};
use crate::error::Error;
use blue_hal::hal::null::NullError;
use blue_hal::hal::time::Now;
use blue_hal::{drivers::{micron::n25q128a_flash,
    stm32f4::{flash, rcc::Clocks, serial, systick::SysTick}}, hal::time, stm32pac
};
use super::autogenerated::{
    self,
    BOOT_TIME_METRICS_ENABLED,
    UDPATE_SIGNAL_ENABLED,
    RECOVERY_ENABLED, devices,
    memory_map::{EXTERNAL_BANKS, MCU_BANKS},
    pin_configuration::{self, *},
};
#[cfg(feature="ecdsa-verify")]
use crate::devices::image::EcdsaImageReader as ImageReader;
#[cfg(not(feature="ecdsa-verify"))]
use crate::devices::image::CrcImageReader as ImageReader;
use super::update_signal::UpdateSignal;

impl Default for Bootloader<ExternalFlash, flash::McuFlash, Serial, SysTick, ImageReader, UpdateSignal> {
    fn default() -> Self { Self::new() }
}

impl Bootloader<ExternalFlash, flash::McuFlash, Serial, SysTick, ImageReader, UpdateSignal> {
    pub fn new() -> Self {
        let mut peripherals = stm32pac::Peripherals::take().unwrap();
        let cortex_peripherals = cortex_m::Peripherals::take().unwrap();
        let mcu_flash = flash::McuFlash::new(peripherals.FLASH).unwrap();

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
        let optional_external_flash = devices::construct_flash(qspi_pins, peripherals.QUADSPI);
        let optional_serial = devices::construct_serial(serial_pins, clocks, peripherals.USART1, peripherals.USART2, peripherals.USART6);

        let start_time = if BOOT_TIME_METRICS_ENABLED {
            Some(SysTick::now())
        } else {
            None
        };

        let update_signal = if UDPATE_SIGNAL_ENABLED {
            Some(UpdateSignal { })
        } else {
            None
        };

        Bootloader {
            mcu_flash,
            external_banks: &EXTERNAL_BANKS,
            mcu_banks: &MCU_BANKS,
            external_flash: optional_external_flash,
            serial: optional_serial,
            boot_metrics: Default::default(),
            start_time,
            recovery_enabled: RECOVERY_ENABLED,
            greeting: autogenerated::LOADSTONE_GREETING,
            _marker: Default::default(),
            update_signal,
        }
    }
}

impl error::Convertible for flash::Error {
    fn into(self) -> Error {
        match self {
            flash::Error::MemoryNotReachable => Error::DriverError("[MCU Flash] Memory not reachable"),
            flash::Error::MisalignedAccess => Error::DriverError("[MCU Flash] Misaligned memory access"),
        }
    }
}

impl error::Convertible for n25q128a_flash::Error {
    fn into(self) -> Error {
        match self {
            n25q128a_flash::Error::TimeOut => Error::DriverError("[External Flash] Operation timed out"),
            n25q128a_flash::Error::QspiError => Error::DriverError("[External Flash] Qspi error"),
            n25q128a_flash::Error::WrongManufacturerId => Error::DriverError("[External Flash] Wrong manufacturer ID"),
            n25q128a_flash::Error::MisalignedAccess => Error::DriverError("[External Flash] Misaligned memory access"),
            n25q128a_flash::Error::AddressOutOfRange => Error::DriverError("[External Flash] Address out of range"),
        }
    }
}

impl error::Convertible for NullError {
    fn into(self) -> Error { panic!("This error should never happen!") }
}

impl error::Convertible for serial::Error {
    fn into(self) -> Error {
        match self {
            serial::Error::Framing => Error::DriverError("[Serial] Framing error"),
            serial::Error::Noise => Error::DriverError("[Serial] Noise error"),
            serial::Error::Overrun => Error::DriverError("[Serial] Overrun error"),
            serial::Error::Parity => Error::DriverError("[Serial] Parity error"),
            serial::Error::Timeout => Error::DriverError("[Serial] Timeout error"),
            _ => Error::DriverError("[Serial] Unexpected serial error"),
        }
    }
}
