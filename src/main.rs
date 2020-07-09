#![cfg_attr(test, allow(unused_attributes))]
#![cfg_attr(target_arch = "arm", no_std)]
#![cfg_attr(target_arch = "arm", no_main)]

#[allow(unused_imports)]
use cortex_m_rt::{exception, entry};

#[cfg(target_arch = "arm")]
#[entry]
fn main() -> ! {
    use secure_bootloader_lib::{devices::implementations::Bootloader, stm32pac};
    let bootloader = Bootloader::new(
        stm32pac::Peripherals::take().unwrap(),
        cortex_m::Peripherals::take().unwrap(),
    );
    bootloader.run();
}

#[cfg(not(target_arch = "arm"))]
fn main() {}
