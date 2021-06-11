use anyhow::Result;
use quote::{format_ident, quote};
use std::{fs::OpenOptions, io::Write, path::Path};

use crate::{
    memory::{ExternalMemoryMap, InternalMemoryMap, MemoryConfiguration},
    port::{Port, Subfamily},
};

use super::prettify_file;

pub fn generate<P: AsRef<Path>>(
    autogenerated_folder_path: P,
    memory_configuration: &MemoryConfiguration,
    port: &Port,
) -> Result<()> {
    let filename = autogenerated_folder_path.as_ref().join("memory_map.rs");
    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&filename)?;
    let base_index = 1usize;
    let imports = generate_imports(&memory_configuration, port)?;
    let mcu_banks = generate_mcu_banks(
        base_index,
        &memory_configuration.internal_memory_map,
        memory_configuration.golden_index,
    )?;
    let external_banks = generate_external_banks(
        memory_configuration.internal_memory_map.banks.len() + base_index,
        &memory_configuration.external_memory_map,
        memory_configuration.golden_index,
    )?;

    file.write_all(imports.as_bytes())?;
    file.write_all(mcu_banks.as_bytes())?;
    file.write_all(external_banks.as_bytes())?;
    prettify_file(filename).ok();
    Ok(())
}

fn generate_imports(memory_configuration: &MemoryConfiguration, port: &Port) -> Result<String> {
    let external_address: Vec<_> = match &memory_configuration.external_flash {
        Some(external_flash) if external_flash.name.to_lowercase().contains("n25q128a") => {
            ["blue_hal", "drivers", "micron", "n25q128a_flash", "Address"]
                .iter()
                .map(|f| format_ident!("{}", f))
                .collect()
        }
        None if *port == Port::Stm32F412 => ["blue_hal", "hal", "null", "NullAddress"]
            .iter()
            .map(|f| format_ident!("{}", f))
            .collect(),
        _ => ["usize"].iter().map(|f| format_ident!("{}", f)).collect(),
    };

    let mcu_address: Vec<_> = match port.subfamily() {
        Subfamily::Stm32f4 => ["blue_hal", "drivers", "stm32f4", "flash", "Address"]
            .iter()
            .map(|f| format_ident!("{}", f))
            .collect(),
        Subfamily::Efm32Gg11 => ["blue_hal", "drivers", "efm32gg11b", "flash", "Address"]
            .iter()
            .map(|f| format_ident!("{}", f))
            .collect(),
    };

    let code = quote! {
        //! This code is autogenerated! Don't modify it manually, as it will be overwritten
        //! in the next project build. Generation logic for this module is defined in
        //! `loadstone_config/src/codegen/memory_map.rs`
        use crate::devices::image as image;
        #[allow(unused_imports)]
        use super::pin_configuration::ExternalFlash;
        use #(#mcu_address)::* as McuAddress;
        use #(#external_address)::* as ExternalAddress;
    };
    Ok(format!("{}", code))
}

fn generate_external_banks(
    base_index: usize,
    map: &ExternalMemoryMap,
    golden_index: Option<usize>,
) -> Result<String> {
    let number_of_external_banks = map.banks.len();
    let index: Vec<u8> =
        map.banks.iter().enumerate().map(|(i, _)| (i + base_index) as u8).collect();
    let bootable = vec![false; number_of_external_banks];
    let location: Vec<u32> = map.banks.iter().map(|b| b.start_address).collect();
    let size: Vec<usize> = map.banks.iter().map(|b| (b.size_kb * 1024) as usize).collect();
    let golden: Vec<bool> =
        (0..number_of_external_banks).map(|i| Some(i + base_index) == golden_index).collect();

    let code = quote! {
        const NUMBER_OF_EXTERNAL_BANKS: usize = #number_of_external_banks;
        pub static EXTERNAL_BANKS: [image::Bank<ExternalAddress>; NUMBER_OF_EXTERNAL_BANKS] = [
            #(image::Bank {
                index: #index,
                bootable: #bootable,
                location: ExternalAddress(#location),
                size: #size,
                is_golden: #golden,
            }),*
        ];
    };
    Ok(format!("{}", code))
}

fn generate_mcu_banks(
    base_index: usize,
    map: &InternalMemoryMap,
    golden_index: Option<usize>,
) -> Result<String> {
    let number_of_mcu_banks = map.banks.len();
    let index: Vec<u8> =
        map.banks.iter().enumerate().map(|(i, _)| (i + base_index) as u8).collect();
    let bootable: Vec<bool> =
        (0..number_of_mcu_banks).map(|i| Some(i) == map.bootable_index).collect();
    let location: Vec<u32> = map.banks.iter().map(|b| b.start_address).collect();
    let size: Vec<usize> = map.banks.iter().map(|b| (b.size_kb * 1024) as usize).collect();
    let golden: Vec<bool> = (0..number_of_mcu_banks).map(|i| Some(i) == golden_index).collect();

    let code = quote! {
        const NUMBER_OF_MCU_BANKS: usize = #number_of_mcu_banks;
        pub static MCU_BANKS: [image::Bank<McuAddress>; NUMBER_OF_MCU_BANKS] = [
            #(image::Bank {
                index: #index,
                bootable: #bootable,
                location: McuAddress(#location),
                size: #size,
                is_golden: #golden,
            }),*
        ];
    };
    Ok(format!("{}", code))
}
