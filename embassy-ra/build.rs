use std::collections::BTreeMap;
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use ra_metapac::metadata::{self, Event};

// ============================================================================
// PERIPHERAL IRQ CONFIGURATION - Add new peripherals here!
// ============================================================================

/// Defines a peripheral's interrupt configuration
struct PeripheralIrqConfig {
    /// Cargo feature that enables this peripheral (without CARGO_FEATURE_ prefix)
    feature: &'static str,
    /// Event names this peripheral uses
    events: &'static [&'static str],
    /// Handler function to call for these events
    handler: &'static str,
}

/// GPT Timer configurations (for time driver)
const GPT_CONFIGS: &[PeripheralIrqConfig] = &[
    PeripheralIrqConfig {
        feature: "TIME_DRIVER_GPT0",
        events: &["GPT0_COUNTER_OVERFLOW", "GPT0_CAPTURE_COMPARE_A"],
        handler: "crate::time_driver::on_interrupt",
    },
    PeripheralIrqConfig {
        feature: "TIME_DRIVER_GPT1",
        events: &["GPT1_COUNTER_OVERFLOW", "GPT1_CAPTURE_COMPARE_A"],
        handler: "crate::time_driver::on_interrupt",
    },
    PeripheralIrqConfig {
        feature: "TIME_DRIVER_GPT2",
        events: &["GPT2_COUNTER_OVERFLOW", "GPT2_CAPTURE_COMPARE_A"],
        handler: "crate::time_driver::on_interrupt",
    },
    PeripheralIrqConfig {
        feature: "TIME_DRIVER_GPT3",
        events: &["GPT3_COUNTER_OVERFLOW", "GPT3_CAPTURE_COMPARE_A"],
        handler: "crate::time_driver::on_interrupt",
    },
];

/// Get all peripheral configurations
fn get_all_peripheral_configs() -> Vec<PeripheralIrqConfig> {
    let mut configs: Vec<PeripheralIrqConfig> = Vec::new();
    
    // Add GPT configs
    configs.extend(GPT_CONFIGS.iter().map(|c| PeripheralIrqConfig {
        feature: c.feature,
        events: c.events,
        handler: c.handler,
    }));
    
    // Add more peripheral configs here as needed:
    // configs.extend(get_uart_configs());
    // configs.extend(get_iic_configs());
    // configs.extend(get_spi_configs());
    // configs.extend(get_adc_configs());
    
    configs
}

// ============================================================================
// BUILD SCRIPT MAIN
// ============================================================================

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

        generate_memory_x(&out);
        generate_interrupt_bindings(&out);
    }

    println!("cargo:rerun-if-changed=build.rs");
}

/// Generate memory.x linker script
fn generate_memory_x(out: &PathBuf) {
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

/// Get enabled peripheral configs based on Cargo features
fn get_enabled_configs() -> Vec<&'static PeripheralIrqConfig> {
    // Leak the configs so they have 'static lifetime
    let configs: &'static Vec<PeripheralIrqConfig> = Box::leak(Box::new(get_all_peripheral_configs()));
    
    configs
        .iter()
        .filter(|config| env::var(format!("CARGO_FEATURE_{}", config.feature)).is_ok())
        .collect()
}

/// Collect all required events from enabled peripherals
fn get_required_events() -> Vec<&'static str> {
    get_enabled_configs()
        .iter()
        .flat_map(|config| config.events.iter().copied())
        .collect()
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
    let enabled_configs = get_enabled_configs();

    let mut code = String::new();
    writeln!(code, "// Auto-generated interrupt bindings").unwrap();
    writeln!(code, "// DO NOT EDIT - generated by build.rs").unwrap();
    writeln!(code).unwrap();

    // Generate event-to-IRQ mapping constants
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

    // Generate IEL interrupt handlers for each enabled peripheral
    // Group events by handler to generate efficient dispatch
    let mut handler_to_events: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for config in &enabled_configs {
        for event in config.events.iter() {
            handler_to_events
                .entry(config.handler)
                .or_default()
                .push(event);
        }
    }

    // Generate handlers
    for (handler, events) in &handler_to_events {
        for event_name in events {
            if let Some(&slot) = allocations.get(*event_name) {
                writeln!(code, "/// Interrupt handler for {} (IEL{})", event_name, slot).unwrap();
                writeln!(code, "#[no_mangle]").unwrap();
                writeln!(code, "#[allow(non_snake_case)]").unwrap();
                writeln!(code, "pub unsafe extern \"C\" fn IEL{}() {{", slot).unwrap();
                writeln!(code, "    {}();", handler).unwrap();
                writeln!(code, "}}").unwrap();
                writeln!(code).unwrap();
            }
        }
    }

    fs::write(out_dir.join("irq_bindings.rs"), code).unwrap();
}
