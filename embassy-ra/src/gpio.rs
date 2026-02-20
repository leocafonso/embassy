//! General-purpose Input/Output (GPIO)
//!
//! This driver uses the PORT registers directly for GPIO control.
//! For pin function configuration (alternate functions, etc.), PFS registers are used.

use core::convert::Infallible;
use crate::{pac, Peripheral as PeripheralType, PeripheralRef as Peri};
use ra_metapac::port::{Port, regs};
#[cfg(feature = "defmt")]
use defmt::*;
#[cfg(not(feature = "defmt"))]
use log::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Low,
    High,
}

impl From<bool> for Level {
    fn from(v: bool) -> Self {
        if v {
            Level::High
        } else {
            Level::Low
        }
    }
}

impl From<Level> for bool {
    fn from(v: Level) -> bool {
        match v {
            Level::Low => false,
            Level::High => true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pull {
    None,
    Up,
}

#[derive(Clone, Copy)]
pub struct AnyPin {
    pin_port: u16,
}

impl AnyPin {
    #[inline]
    pub unsafe fn new(port: u8, pin: u8) -> Self {
        Self {
            pin_port: (port as u16) * 16 + (pin as u16),
        }
    }

    #[inline]
    pub fn pin(&self) -> u8 {
        (self.pin_port % 16) as u8
    }

    #[inline]
    pub fn port(&self) -> u8 {
        (self.pin_port / 16) as u8
    }
}

impl PeripheralType for AnyPin {}

/// Get the PORT block for a given port number.
/// Uses Port base type which works for both Port and PortExtended since they
/// share the same register layout for basic GPIO operations.
#[inline]
fn port_block(port: u8) -> Port {
    // PORT blocks are spaced 0x20 apart from PORT0
    let base = pac::PORT0.as_ptr() as *mut u8;
    let port_ptr = unsafe { base.add(port as usize * 0x20) };
    unsafe { Port::from_ptr(port_ptr as *mut ()) }
}

/// Unlock PFS registers for writing
fn pfs_unlock() {
    pac::PFS.pwpr().write(|w| {
        w.set_b0wi(false);
        w.set_pfswe(false);
    });
    pac::PFS.pwpr().write(|w| {
        w.set_b0wi(false);
        w.set_pfswe(true);
    });
    debug!("pwpr {}", pac::PFS.pwpr().read());
}

/// Lock PFS registers to prevent accidental writes
fn pfs_lock() {
    pac::PFS.pwpr().write(|w| {
        w.set_b0wi(false);
        w.set_pfswe(false);
    });
    pac::PFS.pwpr().write(|w| {
        w.set_b0wi(true);
        w.set_pfswe(false);
    });
    debug!("pwpr {}", pac::PFS.pwpr().read());
}

/// GPIO flexible pin.
///
/// This pin can be configured as input or output.
pub struct Flex<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> Flex<'d> {
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        Self {
            pin: pin.into(),
        }
    }

    /// Set the output level using POSR (set high) or PORR (set low).
    /// These are atomic write-only registers.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        let port = port_block(self.pin.port());
        let pin_mask = 1u16 << self.pin.pin();
        
        match level {
            Level::High => {
                // POSR: writing 1 sets the pin high
                port.posr().write_value(regs::Posr(pin_mask));
            }
            Level::Low => {
                // PORR: writing 1 sets the pin low
                port.porr().write_value(regs::Porr(pin_mask));
            }
        }
    }

    /// Configure as output.
    #[inline]
    pub fn set_as_output(&mut self) {
        let (port_num, pin) = (self.pin.port(), self.pin.pin());
        
        critical_section::with(|_| {
            // Configure direction via PFS (requires write protection unlock)
            pfs_unlock();
            pac::PFS.pmn_pfs(port_num as usize * 16 + pin as usize).modify(|w| {
                w.set_pdr(true);  // Output
                w.set_pmr(false); // GPIO mode (not peripheral)
            });
            debug!("pfs {}", pac::PFS.pmn_pfs(port_num as usize * 16 + pin as usize).read());
            pfs_lock();
        });
    }

    /// Configure as input.
    #[inline]
    pub fn set_as_input(&mut self, pull: Pull) {
        let (port_num, pin) = (self.pin.port(), self.pin.pin());
        
        critical_section::with(|_| {
            // Configure direction via PFS (requires write protection unlock)
            pfs_unlock();
            pac::PFS.pmn_pfs(port_num as usize * 16 + pin as usize).modify(|w| {
                w.set_pdr(false); // Input
                w.set_pmr(false); // GPIO mode (not peripheral)
                w.set_pcr(matches!(pull, Pull::Up));
            });
            pfs_lock();
        });
    }

    /// Set pin high.
    #[inline]
    pub fn set_high(&mut self) {
        self.set_level(Level::High)
    }

    /// Set pin low.
    #[inline]
    pub fn set_low(&mut self) {
        self.set_level(Level::Low)
    }

    /// Get the current input level from PIDR.
    #[inline]
    pub fn get_level(&self) -> Level {
        let port = port_block(self.pin.port());
        let pin_mask = 1u16 << self.pin.pin();
        
        if (port.pidr().read().0 & pin_mask) != 0 {
            Level::High
        } else {
            Level::Low
        }
    }

    /// Check if input is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.get_level() == Level::High
    }

    /// Check if input is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.get_level() == Level::Low
    }
}

impl<'d> embedded_hal::digital::ErrorType for Flex<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal::digital::OutputPin for Flex<'d> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }
}

impl<'d> embedded_hal::digital::InputPin for Flex<'d> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.get_level() == Level::High)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.get_level() == Level::Low)
    }
}

/// GPIO input driver.
pub struct Input<'d> {
    pub(crate) flex: Flex<'d>,
}

impl<'d> Input<'d> {
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self {
        let mut flex = Flex::new(pin);
        flex.set_as_input(pull);
        Self { flex }
    }

    #[inline]
    pub fn is_high(&self) -> bool {
        self.flex.is_high()
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        self.flex.is_low()
    }

    #[inline]
    pub fn get_level(&self) -> Level {
        self.flex.get_level()
    }
}

impl<'d> embedded_hal::digital::ErrorType for Input<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal::digital::InputPin for Input<'d> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

/// GPIO output driver.
pub struct Output<'d> {
    pub(crate) flex: Flex<'d>,
}

impl<'d> Output<'d> {
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_level: Level) -> Self {
        let mut flex = Flex::new(pin);
        flex.set_level(initial_level);
        flex.set_as_output();
        Self { flex }
    }

    #[inline]
    pub fn set_high(&mut self) {
        self.flex.set_high()
    }

    #[inline]
    pub fn set_low(&mut self) {
        self.flex.set_low()
    }

    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.flex.set_level(level)
    }

    #[inline]
    pub fn is_set_high(&self) -> bool {
        let port = port_block(self.flex.pin.port());
        let pin_mask = 1u16 << self.flex.pin.pin();
        (port.podr().read().0 & pin_mask) != 0
    }

    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    #[inline]
    pub fn toggle(&mut self) {
        if self.is_set_high() {
            self.set_low();
        } else {
            self.set_high();
        }
    }
}

impl<'d> embedded_hal::digital::ErrorType for Output<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal::digital::OutputPin for Output<'d> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }
}

impl<'d> embedded_hal::digital::StatefulOutputPin for Output<'d> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

// ============================================================================
// Pin trait
// ============================================================================

pub(crate) mod sealed {
    pub trait Pin {
        fn pin_port(&self) -> u16;
    }
}

pub trait Pin: PeripheralType + Into<AnyPin> + sealed::Pin + Sized + 'static {
    #[inline]
    fn pin(&self) -> u8 {
        (self.pin_port() % 16) as u8
    }

    #[inline]
    fn port(&self) -> u8 {
        (self.pin_port() / 16) as u8
    }

    #[inline]
    fn degrade(self) -> AnyPin {
        AnyPin {
            pin_port: self.pin_port(),
        }
    }
}

impl Pin for AnyPin {}
impl sealed::Pin for AnyPin {
    #[inline]
    fn pin_port(&self) -> u16 {
        self.pin_port
    }
}

// Note: Pin trait implementations for individual pins (P100, P101, etc.)
// are generated in peripherals.rs by build.rs
