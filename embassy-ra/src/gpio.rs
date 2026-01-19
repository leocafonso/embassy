use core::convert::Infallible;
use crate::{pac, Peripheral as PeripheralType, PeripheralRef as Peri};

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

    #[inline]
    pub fn set_level(&mut self, level: Level) {
        let (port, pin) = (self.pin.port(), self.pin.pin());
        critical_section::with(|_| {
            self.set_write_protect(false);
            pac::PFS.pmn_pfs(port as usize * 16 + pin as usize).modify(|w| w.set_podr(level.into()));
            self.set_write_protect(true);
        });
    }

    fn set_write_protect(&self, protect: bool) {
        if !protect {
            pac::PFS.pwpr().write(|w| {
                w.set_b0wi(false);
                w.set_pfswe(false);
            });
            pac::PFS.pwpr().write(|w| {
                w.set_b0wi(false);
                w.set_pfswe(true);
            });
        } else {
            pac::PFS.pwpr().write(|w| {
                w.set_b0wi(false);
                w.set_pfswe(false);
            });
            pac::PFS.pwpr().write(|w| {
                w.set_b0wi(true);
                w.set_pfswe(false);
            });
        }
    }

    #[inline]
    pub fn set_as_output(&mut self) {
        let (port, pin) = (self.pin.port(), self.pin.pin());
        critical_section::with(|_| {
            self.set_write_protect(false);
            pac::PFS.pmn_pfs(port as usize * 16 + pin as usize).modify(|w| {
                w.set_pdr(true); // Output
                w.set_pmr(false); // GPIO
            });
            self.set_write_protect(true);
        });
    }

    #[inline]
    pub fn set_as_input(&mut self, pull: Pull) {
        let (port, pin) = (self.pin.port(), self.pin.pin());
        critical_section::with(|_| {
            self.set_write_protect(false);
            pac::PFS.pmn_pfs(port as usize * 16 + pin as usize).modify(|w| {
                w.set_pdr(false); // Input
                w.set_pmr(false); // GPIO
                w.set_pcr(match pull {
                    Pull::Up => true,
                    _ => false,
                });
            });
            self.set_write_protect(true);
        });
    }

    #[inline]
    pub fn set_high(&mut self) {
        self.set_level(Level::High)
    }

    #[inline]
    pub fn set_low(&mut self) {
        self.set_level(Level::Low)
    }

    #[inline]
    pub fn get_level(&self) -> Level {
        let (port, pin) = (self.pin.port(), self.pin.pin());
        if pac::PFS.pmn_pfs(port as usize * 16 + pin as usize).read().pidr() {
            Level::High
        } else {
            Level::Low
        }
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
        self.flex.get_level() == Level::High
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        self.flex.get_level() == Level::Low
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
}

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

macro_rules! impl_pin {
    ($($(#[$cfg:meta])* ($name:ident, $port:expr, $pin:expr)),* $(,)?) => {
        $(
            $(#[$cfg])*
            impl Pin for crate::peripherals::$name {}
            $(#[$cfg])*
            impl sealed::Pin for crate::peripherals::$name {
                #[inline]
                fn pin_port(&self) -> u16 {
                    ($port as u16) * 16 + ($pin as u16)
                }
            }

            $(#[$cfg])*
            impl From<crate::peripherals::$name> for AnyPin {
                fn from(_: crate::peripherals::$name) -> Self {
                    unsafe { Self::new($port, $pin) }
                }
            }
        )*
    };
}

pac::foreach_pin!(impl_pin);
