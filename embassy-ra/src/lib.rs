#![no_std]

pub use ra_metapac as pac;

pub use embassy_hal_internal::{impl_peripheral, Peri as PeripheralRef, PeripheralType as Peripheral};

pub mod interrupt;
pub mod mstp;

#[cfg(feature = "_time-driver")]
mod time_driver;

pub mod peripherals {
    pub use ra_metapac::peripherals::*;
}

pub mod _macros {
    pub use crate::bind_interrupts;
}

pub struct Config {
    _private: (),
}

impl Default for Config {
    fn default() -> Self {
        Self { _private: () }
    }
}

#[cfg(feature = "_time-driver")]
pub use time_driver::{Irqs, IRQS};

pub fn init(_config: Config) -> peripherals::Peripherals {
    critical_section::with(|_cs| {
        // TODO: Initialize clocks, etc.
        #[cfg(feature = "_time-driver")]
        time_driver::init(_cs);
        unsafe { peripherals::Peripherals::steal() }
    })
}
