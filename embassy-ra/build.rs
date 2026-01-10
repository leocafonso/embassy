use std::env;
use std::fs;
use std::path::PathBuf;
use ra_metapac::metadata;

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let chip_name = env::vars()
        .map(|(a, _)| a)
        .filter(|x| x.starts_with("CARGO_FEATURE_R7") || x.starts_with("CARGO_FEATURE_R8"))
        .next()
        .map(|x| x.strip_prefix("CARGO_FEATURE_").unwrap().to_ascii_lowercase().replace('_', "-"));

    if let Some(chip_name) = chip_name {
        println!("cargo:rustc-cfg={}", chip_name);
        println!("cargo:rustc-cfg=ra");

        use std::fmt::Write;
        let mut memory_x = String::new();

        let flash_region = metadata::MEMORY.iter().find(|r| r.name == "FLASH").unwrap();
        let mut overlaps = Vec::new();
        for region in metadata::MEMORY {
            if region.name.starts_with("OPTION_SETTING_") {
                if region.address >= flash_region.address && region.address < flash_region.address + flash_region.size {
                    overlaps.push(region);
                }
            }
        }

        if !overlaps.is_empty() {
            let last_region = overlaps.iter().max_by_key(|r| r.address + r.size).unwrap();
            let end_addr = last_region.address + last_region.size;
            writeln!(memory_x, "_stext = 0x{:08x};", end_addr).unwrap();
        }

        writeln!(memory_x, "MEMORY").unwrap();
        writeln!(memory_x, "{{").unwrap();

        for region in metadata::MEMORY {
            let size_str = if region.size >= 1024 && region.size % 1024 == 0 {
                format!("{}K", region.size / 1024)
            } else {
                format!("{}", region.size)
            };

            writeln!(
                memory_x,
                "    {} : ORIGIN = 0x{:08x}, LENGTH = {}",
                region.name, region.address, size_str
            )
            .unwrap();
        }
        writeln!(memory_x, "}}").unwrap();

        writeln!(memory_x, "SECTIONS").unwrap();
        writeln!(memory_x, "{{").unwrap();

        for region in metadata::MEMORY {
            if region.name.starts_with("OPTION_SETTING_") {
                let section_name = region.name.strip_prefix("OPTION_SETTING_").unwrap().to_lowercase();
                if overlaps.iter().any(|o| o.name == region.name) {
                    writeln!(
                        memory_x,
                        "    .{} 0x{:08x} : {{ KEEP(*(.{})) }} > FLASH",
                        section_name, region.address, section_name
                    )
                    .unwrap();
                } else {
                    writeln!(memory_x, "    .{} : {{ KEEP(*(.{})) }} > {}", section_name, section_name, region.name)
                        .unwrap();
                }
            }
        }
        writeln!(memory_x, "}}").unwrap();

        fs::write(out.join("memory.x"), memory_x).unwrap();
        println!("cargo:rustc-link-search={}", out.display());
        println!("cargo:rerun-if-changed=memory.x");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
