#![no_std]

pub use ra_metapac as pac;

pub use embassy_hal_internal::{impl_peripheral, Peri as PeripheralRef, PeripheralType as Peripheral};

#[cfg(feature = "gpio")]
pub mod gpio;
pub mod interrupt;
pub mod mstp;
pub mod system;

// Include auto-generated peripherals definitions
// This expands to: pub mod peripherals { ... with GPT0, Peripherals, etc. }
include!(concat!(env!("OUT_DIR"), "/peripherals.rs"));
// Re-export for convenient access
pub use peripherals::*;

#[cfg(feature = "_time-driver")]
mod time_driver;

pub mod _macros {
    pub use crate::bind_interrupts;
}

/// System configuration
pub struct Config {
    /// Clock configuration
    pub system: system::Config,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            system: Default::default(),
        }
    }
}

pub fn init(config: Config) -> Peripherals {
    critical_section::with(|_cs| {
        // Initialize clocks
        let clocks = system::init(config.system);
        unsafe { system::set_freqs(clocks) };
        
        #[cfg(feature = "_time-driver")]
        time_driver::init(_cs);
        
        unsafe { Peripherals::steal() }
    })
}

