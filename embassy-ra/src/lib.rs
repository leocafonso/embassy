#![no_std]

pub use ra_metapac as pac;

pub mod peripherals {
    pub use ra_metapac::peripherals::*;
}

pub struct Config {
    _private: (),
}

impl Default for Config {
    fn default() -> Self {
        Self { _private: () }
    }
}

pub fn init(_config: Config) -> peripherals::Peripherals {
    // TODO: Initialize clocks, etc.
    unsafe { peripherals::Peripherals::steal() }
}
