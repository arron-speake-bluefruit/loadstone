use crate::{port, Configuration};
use anyhow::Result;
use quote::quote;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use super::prettify_file;
mod stm32;

/// Generates the `pin_configuration.rs` module, which contains pin definitions
/// alternate function assignments for a particular loadstone build.
pub fn generate<P: AsRef<Path>>(
    autogenerated_folder_path: P,
    configuration: &Configuration,
) -> Result<()> {
    let filename = autogenerated_folder_path.as_ref().join("pin_configuration.rs");
    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&filename)?;

    match configuration.port.subfamily() {
        port::Subfamily::Stm32f4 => stm32::generate_stm32f4_pins(configuration, &mut file)?,
        port::Subfamily::Efm32Gg11 => generate_efm32gg(configuration, &mut file)?,
        port::Subfamily::Maxim3263 => generate_maxim3263(configuration, &mut file)?,
    };
    prettify_file(filename).ok();
    Ok(())
}

fn generate_efm32gg(_configuration: &Configuration, file: &mut File) -> Result<()> {
    let code = quote! {
        pub use blue_hal::hal::null::NullFlash as ExternalFlash;
    };
    file.write_all(format!("{}", code).as_bytes())?;
    Ok(())
}

fn generate_maxim3263(_configuration: &Configuration, _file: &mut File) -> Result<()> {
    Ok(())
}
