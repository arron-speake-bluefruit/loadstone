use std::{fs::{File, OpenOptions}, path::Path};
use anyhow::Result;
use crate::{Configuration, port};

use super::prettify_file;
mod stm32;

pub fn generate<P: AsRef<Path>>(
    autogenerated_folder_path: P,
    configuration: &Configuration,
) -> Result<()> {
    let filename = autogenerated_folder_path.as_ref().join("pin_configuration.rs");
    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&filename)?;

    match configuration.port.subfamily() {
        port::Subfamily::Stm32f4 => stm32::generate_stm32f4_pins(configuration, &mut file)?,
        port::Subfamily::Efm32Gg11 => generate_efm32gg(configuration, &mut file)?,
    };
    prettify_file(filename).ok();
    Ok(())
}

fn generate_efm32gg(_configuration: &Configuration, _file: &mut File) -> Result<()> {
    todo!()
}

