use crate::{
    devices::implementations::flash::micron_n25q128a::MicronN25q128a,
    drivers::{
        gpio::{GpioExt, *},
        qspi::{mode, QuadSpi, self},
        rcc::RccExt,
        serial::{self, UsartAf, UsartExt},
    },
    hal::{self, serial::Write},
    pin_configuration::*,
    stm32pac::{Peripherals, USART6},
};

// Flash pins and typedefs
type QspiPins = (Pb2<AF9>, Pg6<AF10>, Pf8<AF10>, Pf9<AF10>, Pf7<AF9>, Pf6<AF9>);
type Qspi = QuadSpi<QspiPins, mode::Single>;
type Flash = MicronN25q128a<Qspi>;

// Serial pins and typedefs
type UsartPins = (Pg14<UsartAf>, Pg9<UsartAf>);
type Serial = serial::Serial<USART6, UsartPins>;

/// Top level Bootloader type for the stm32f412 Discovery board
pub struct Bootloader {
    flash: Flash,
    serial: Serial,
}

impl Bootloader {
    pub fn new(mut peripherals: Peripherals) -> Bootloader {
        let gpiob = peripherals.GPIOB.split(&mut peripherals.RCC);
        let gpiog = peripherals.GPIOG.split(&mut peripherals.RCC);
        let gpiof = peripherals.GPIOF.split(&mut peripherals.RCC);
        let clocks = peripherals
            .RCC
            .constrain()
            .use_hse(hal::time::MegaHertz(180))
            .sysclk(hal::time::MegaHertz(180))
            .hclk(hal::time::MegaHertz(84))
            .pclk1(hal::time::MegaHertz(42))
            .pclk2(hal::time::MegaHertz(84))
            .require_pll48clk()
            .freeze();
        let serial_config = serial::config::Config::default().baudrate(hal::time::Bps(115_200));
        let serial_pins = (gpiog.pg14, gpiog.pg9);
        let serial = peripherals.USART6.constrain(serial_pins, serial_config, clocks).unwrap();

        let qspi_config = qspi::Config::<mode::Single>::default().with_flash_size(24).unwrap();
        let qspi_pins = (gpiob.pb2, gpiog.pg6, gpiof.pf8, gpiof.pf9, gpiof.pf7, gpiof.pf6);
        let qspi = Qspi::from_config(peripherals.QUADSPI, qspi_pins, qspi_config).unwrap();
        let flash = Flash::new(qspi).unwrap();

        Bootloader { flash, serial }
    }

    pub fn run(mut self) -> ! {
        uprintln!(self.serial, "Secure Bootloader Startup");
        loop {}
    }
}
