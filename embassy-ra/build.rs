use std::collections::BTreeMap;
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use ra_metapac::metadata::{self, Event};

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

        // Generate interrupt bindings based on enabled features
        generate_interrupt_bindings(&out);
    }

    println!("cargo:rerun-if-changed=build.rs");
}

/// Determines which events are needed based on enabled Cargo features
fn get_required_events() -> Vec<&'static str> {
    let mut events = Vec::new();

    // Time driver events (GPT0)
    if env::var("CARGO_FEATURE_TIME_DRIVER_GPT0").is_ok() {
        events.push("GPT0_COUNTER_OVERFLOW");
        events.push("GPT0_CAPTURE_COMPARE_A");
    }
    if env::var("CARGO_FEATURE_TIME_DRIVER_GPT1").is_ok() {
        events.push("GPT1_COUNTER_OVERFLOW");
        events.push("GPT1_CAPTURE_COMPARE_A");
    }

    // UART events
    for i in 0..10 {
        if env::var(format!("CARGO_FEATURE_SCI_UART{}", i)).is_ok() {
            events.push(Box::leak(format!("SCI{}_RXI", i).into_boxed_str()));
            events.push(Box::leak(format!("SCI{}_TXI", i).into_boxed_str()));
            events.push(Box::leak(format!("SCI{}_TEI", i).into_boxed_str()));
            events.push(Box::leak(format!("SCI{}_ERI", i).into_boxed_str()));
        }
    }

    // I2C events
    for i in 0..3 {
        if env::var(format!("CARGO_FEATURE_IIC{}", i)).is_ok() {
            events.push(Box::leak(format!("IIC{}_RXI", i).into_boxed_str()));
            events.push(Box::leak(format!("IIC{}_TXI", i).into_boxed_str()));
            events.push(Box::leak(format!("IIC{}_TEI", i).into_boxed_str()));
            events.push(Box::leak(format!("IIC{}_ERI", i).into_boxed_str()));
        }
    }

    // SPI events
    for i in 0..2 {
        if env::var(format!("CARGO_FEATURE_SPI{}", i)).is_ok() {
            events.push(Box::leak(format!("SPI{}_SPRI", i).into_boxed_str()));
            events.push(Box::leak(format!("SPI{}_SPTI", i).into_boxed_str()));
            events.push(Box::leak(format!("SPI{}_SPII", i).into_boxed_str()));
            events.push(Box::leak(format!("SPI{}_SPEI", i).into_boxed_str()));
        }
    }

    // ADC events
    for i in 0..2 {
        if env::var(format!("CARGO_FEATURE_ADC{}", i)).is_ok() {
            events.push(Box::leak(format!("ADC{}_SCAN_END", i).into_boxed_str()));
            events.push(Box::leak(format!("ADC{}_SCAN_END_B", i).into_boxed_str()));
        }
    }

    events
}

/// Allocates IRQ slots to events, respecting group constraints
fn allocate_irq_slots(required_events: &[&str]) -> BTreeMap<&'static str, u8> {
    let mut allocations: BTreeMap<&'static str, u8> = BTreeMap::new();
    let mut used_slots: Vec<bool> = vec![false; metadata::INTERRUPT_COUNT];

    // Build event lookup map
    let event_map: BTreeMap<&str, &Event> = metadata::EVENTS
        .iter()
        .map(|e| (e.name, e))
        .collect();

    // First pass: allocate events with restricted slots (grouped events)
    for event_name in required_events {
        if let Some(event) = event_map.get(*event_name) {
            if !event.irq_slots.is_empty() {
                // Find first available slot from the allowed list
                for &slot in event.irq_slots {
                    if !used_slots[slot as usize] {
                        used_slots[slot as usize] = true;
                        allocations.insert(event.name, slot);
                        break;
                    }
                }
            }
        }
    }

    // Second pass: allocate events with unrestricted slots
    for event_name in required_events {
        if allocations.contains_key(*event_name) {
            continue; // Already allocated
        }
        if let Some(event) = event_map.get(*event_name) {
            if event.irq_slots.is_empty() {
                // Find first available slot
                for (slot, used) in used_slots.iter_mut().enumerate() {
                    if !*used {
                        *used = true;
                        allocations.insert(event.name, slot as u8);
                        break;
                    }
                }
            }
        }
    }

    allocations
}

/// Generates the interrupt binding code
fn generate_interrupt_bindings(out_dir: &PathBuf) {
    let required_events = get_required_events();
    let allocations = allocate_irq_slots(&required_events);

    let mut code = String::new();
    writeln!(code, "// Auto-generated interrupt bindings").unwrap();
    writeln!(code, "// DO NOT EDIT - generated by build.rs").unwrap();
    writeln!(code).unwrap();

    // Generate event-to-IRQ mapping struct
    writeln!(code, "/// Auto-allocated IRQ slots for events").unwrap();
    writeln!(code, "pub mod irq_allocations {{").unwrap();
    for (event_name, irq_slot) in &allocations {
        let const_name = event_name.to_uppercase();
        writeln!(code, "    /// IRQ slot for {} event", event_name).unwrap();
        writeln!(code, "    pub const {}_IRQ: u8 = {};", const_name, irq_slot).unwrap();
    }
    writeln!(code, "}}").unwrap();
    writeln!(code).unwrap();

    // Generate event IDs
    writeln!(code, "/// Event IDs for ICU IELSR programming").unwrap();
    writeln!(code, "pub mod event_ids {{").unwrap();
    let event_map: BTreeMap<&str, &Event> = metadata::EVENTS
        .iter()
        .map(|e| (e.name, e))
        .collect();
    for event_name in &required_events {
        if let Some(event) = event_map.get(*event_name) {
            let const_name = event_name.to_uppercase();
            writeln!(code, "    pub const {}: u16 = {};", const_name, event.id).unwrap();
        }
    }
    writeln!(code, "}}").unwrap();
    writeln!(code).unwrap();

    // Generate interrupt handlers module for time driver
    let time_driver_enabled = env::var("CARGO_FEATURE_TIME_DRIVER_GPT0").is_ok() ||
                              env::var("CARGO_FEATURE_TIME_DRIVER_GPT1").is_ok() ||
                              env::var("CARGO_FEATURE_TIME_DRIVER_GPT2").is_ok() ||
                              env::var("CARGO_FEATURE_TIME_DRIVER_GPT3").is_ok();

    if time_driver_enabled {
        writeln!(code, "/// Auto-generated interrupt handlers for time driver").unwrap();
        writeln!(code, "pub mod time_driver_irqs {{").unwrap();
        
        // Get the GPT number from features
        let gpt_num = if env::var("CARGO_FEATURE_TIME_DRIVER_GPT0").is_ok() { 0 }
                      else if env::var("CARGO_FEATURE_TIME_DRIVER_GPT1").is_ok() { 1 }
                      else if env::var("CARGO_FEATURE_TIME_DRIVER_GPT2").is_ok() { 2 }
                      else { 3 };
        
        let overflow_event = format!("GPT{}_COUNTER_OVERFLOW", gpt_num);
        let ccmpa_event = format!("GPT{}_CAPTURE_COMPARE_A", gpt_num);
        
        // Generate IEL handlers for time driver events
        if let Some(&overflow_slot) = allocations.get(overflow_event.as_str()) {
            writeln!(code, "    /// Interrupt handler for {} (IEL{})", overflow_event, overflow_slot).unwrap();
            writeln!(code, "    #[no_mangle]").unwrap();
            writeln!(code, "    #[allow(non_snake_case)]").unwrap();
            writeln!(code, "    pub unsafe extern \"C\" fn IEL{}() {{", overflow_slot).unwrap();
            writeln!(code, "        crate::time_driver::on_interrupt();").unwrap();
            writeln!(code, "    }}").unwrap();
            writeln!(code).unwrap();
        }
        
        if let Some(&ccmpa_slot) = allocations.get(ccmpa_event.as_str()) {
            writeln!(code, "    /// Interrupt handler for {} (IEL{})", ccmpa_event, ccmpa_slot).unwrap();
            writeln!(code, "    #[no_mangle]").unwrap();
            writeln!(code, "    #[allow(non_snake_case)]").unwrap();
            writeln!(code, "    pub unsafe extern \"C\" fn IEL{}() {{", ccmpa_slot).unwrap();
            writeln!(code, "        crate::time_driver::on_interrupt();").unwrap();
            writeln!(code, "    }}").unwrap();
        }
        
        writeln!(code, "}}").unwrap();
    }

    fs::write(out_dir.join("irq_bindings.rs"), code).unwrap();
}
