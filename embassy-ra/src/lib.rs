#![no_std]

pub use ra_metapac as pac;

pub use embassy_hal_internal::{impl_peripheral, Peri as PeripheralRef, PeripheralType as Peripheral};

pub mod interrupt;

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

bind_interrupts!(pub struct Irqs {
    Gpt0Ovf => time_driver::InterruptHandler<peripherals::GPT0>;
    Gpt0Ccmpa => time_driver::InterruptHandler<peripherals::GPT0>;
});

pub static IRQS: Irqs = Irqs;

pub fn init(_config: Config) -> peripherals::Peripherals {
    critical_section::with(|cs| {
        // TODO: Initialize clocks, etc.
        // #[cfg(feature = "_time-driver")]
        time_driver::init(cs);
        unsafe { peripherals::Peripherals::steal() }
    })
}
