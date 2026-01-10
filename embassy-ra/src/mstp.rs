use crate::pac;

#[allow(unused_variables)]
trait MstpRA {
    unsafe fn modify_a(&self, bit: u32, f: impl FnOnce(&mut u32)) {}
}
#[allow(unused_variables)]
trait MstpRB {
    unsafe fn modify_b(&self, bit: u32, f: impl FnOnce(&mut u32)) {}
}
#[allow(unused_variables)]
trait MstpRC {
    unsafe fn modify_c(&self, bit: u32, f: impl FnOnce(&mut u32)) {}
}
#[allow(unused_variables)]
trait MstpRD {
    unsafe fn modify_d(&self, bit: u32, f: impl FnOnce(&mut u32)) {}
}
#[allow(unused_variables)]
trait MstpRE {
    unsafe fn modify_e(&self, bit: u32, f: impl FnOnce(&mut u32)) {}
}

// MSTP versions
impl MstpRA for pac::_peripherals::mstp_v1::Mstp {}
impl MstpRA for pac::_peripherals::mstp_v2::Mstp {
    unsafe fn modify_a(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcra().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}
impl MstpRA for pac::_peripherals::mstp_v3::Mstp {}

impl MstpRB for pac::_peripherals::mstp_v1::Mstp {
    unsafe fn modify_b(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcrb().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}
impl MstpRB for pac::_peripherals::mstp_v2::Mstp {
    unsafe fn modify_b(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcrb().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}
impl MstpRB for pac::_peripherals::mstp_v3::Mstp {
    unsafe fn modify_b(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcrb().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}

impl MstpRC for pac::_peripherals::mstp_v1::Mstp {
    unsafe fn modify_c(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcrc().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}
impl MstpRC for pac::_peripherals::mstp_v2::Mstp {
    unsafe fn modify_c(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcrc().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}
impl MstpRC for pac::_peripherals::mstp_v3::Mstp {
    unsafe fn modify_c(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcrc().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}

impl MstpRD for pac::_peripherals::mstp_v1::Mstp {
    unsafe fn modify_d(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcrd().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}
impl MstpRD for pac::_peripherals::mstp_v2::Mstp {
    unsafe fn modify_d(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcrd().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}
impl MstpRD for pac::_peripherals::mstp_v3::Mstp {
    unsafe fn modify_d(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcrd().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}

impl MstpRE for pac::_peripherals::mstp_v1::Mstp {}
impl MstpRE for pac::_peripherals::mstp_v2::Mstp {
    unsafe fn modify_e(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcre().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}
impl MstpRE for pac::_peripherals::mstp_v3::Mstp {}

// SYSTEM versions
impl MstpRA for pac::_peripherals::system_v1::Sysc {
    unsafe fn modify_a(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcra().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}
impl MstpRA for pac::_peripherals::system_v2::System {
    unsafe fn modify_a(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcra().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}
impl MstpRA for pac::_peripherals::system_v3::Sysc {}
impl MstpRA for pac::_peripherals::system_v4::System {
    unsafe fn modify_a(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
        self.mstpcra().modify(|w| {
            let mut val = w.0 as u32;
            f(&mut val);
            w.0 = val as _;
        })
    }
}
impl MstpRA for pac::_peripherals::system_v5::Sysc {}

pub trait SealedPeripheral {
    fn metadata() -> &'static pac::metadata::Peripheral;
}

macro_rules! impl_peripheral {
    ($($name:ident = $index:expr,)*) => {
        $(
            impl SealedPeripheral for pac::peripherals::$name {
                fn metadata() -> &'static pac::metadata::Peripheral {
                    &pac::metadata::PERIPHERALS[$index]
                }
            }
        )*
    };
}

pac::foreach_peripheral!(impl_peripheral);

pub trait Peripheral: SealedPeripheral {}
impl<T: SealedPeripheral> Peripheral for T {}

pub unsafe fn enable_clock<T: Peripheral>(_peri: T) {
    let metadata = T::metadata();
    if let Some(mstp) = metadata.mstp {
        let bit = mstp.bit;
        match mstp.register {
            "MSTPCRA" => {
                macro_rules! call {
                    (MSTP) => {
                        pac::peripherals::MSTP::steal().modify_a(bit, |r| *r &= !(1 << bit));
                    };
                    (SYSTEM) => {
                        pac::peripherals::SYSTEM::steal().modify_a(bit, |r| *r &= !(1 << bit));
                    };
                    ($other:ident) => {};
                }
                macro_rules! runner {
                    ($($name:ident = $index:expr,)*) => {
                        $( call!($name); )*
                    };
                }
                pac::foreach_peripheral!(runner);
            }
            "MSTPCRB" => {
                macro_rules! call {
                    (MSTP) => {
                        pac::peripherals::MSTP::steal().modify_b(bit, |r| *r &= !(1 << bit));
                    };
                    ($other:ident) => {};
                }
                macro_rules! runner {
                    ($($name:ident = $index:expr,)*) => {
                        $( call!($name); )*
                    };
                }
                pac::foreach_peripheral!(runner);
            }
            "MSTPCRC" => {
                macro_rules! call {
                    (MSTP) => {
                        pac::peripherals::MSTP::steal().modify_c(bit, |r| *r &= !(1 << bit));
                    };
                    ($other:ident) => {};
                }
                macro_rules! runner {
                    ($($name:ident = $index:expr,)*) => {
                        $( call!($name); )*
                    };
                }
                pac::foreach_peripheral!(runner);
            }
            "MSTPCRD" => {
                macro_rules! call {
                    (MSTP) => {
                        pac::peripherals::MSTP::steal().modify_d(bit, |r| *r &= !(1 << bit));
                    };
                    ($other:ident) => {};
                }
                macro_rules! runner {
                    ($($name:ident = $index:expr,)*) => {
                        $( call!($name); )*
                    };
                }
                pac::foreach_peripheral!(runner);
            }
            "MSTPCRE" => {
                macro_rules! call {
                    (MSTP) => {
                        pac::peripherals::MSTP::steal().modify_e(bit, |r| *r &= !(1 << bit));
                    };
                    ($other:ident) => {};
                }
                macro_rules! runner {
                    ($($name:ident = $index:expr,)*) => {
                        $( call!($name); )*
                    };
                }
                pac::foreach_peripheral!(runner);
            }
            _ => {}
        }
    }
}

pub unsafe fn disable_clock<T: Peripheral>(_peri: T) {
    let metadata = T::metadata();
    if let Some(mstp) = metadata.mstp {
        let bit = mstp.bit;
        match mstp.register {
            "MSTPCRA" => {
                macro_rules! call {
                    (MSTP) => {
                        pac::peripherals::MSTP::steal().modify_a(bit, |r| *r |= 1 << bit);
                    };
                    (SYSTEM) => {
                        pac::peripherals::SYSTEM::steal().modify_a(bit, |r| *r |= 1 << bit);
                    };
                    ($other:ident) => {};
                }
                macro_rules! runner {
                    ($($name:ident = $index:expr,)*) => {
                        $( call!($name); )*
                    };
                }
                pac::foreach_peripheral!(runner);
            }
            "MSTPCRB" => {
                macro_rules! call {
                    (MSTP) => {
                        pac::peripherals::MSTP::steal().modify_b(bit, |r| *r |= 1 << bit);
                    };
                    ($other:ident) => {};
                }
                macro_rules! runner {
                    ($($name:ident = $index:expr,)*) => {
                        $( call!($name); )*
                    };
                }
                pac::foreach_peripheral!(runner);
            }
            "MSTPCRC" => {
                macro_rules! call {
                    (MSTP) => {
                        pac::peripherals::MSTP::steal().modify_c(bit, |r| *r |= 1 << bit);
                    };
                    ($other:ident) => {};
                }
                macro_rules! runner {
                    ($($name:ident = $index:expr,)*) => {
                        $( call!($name); )*
                    };
                }
                pac::foreach_peripheral!(runner);
            }
            "MSTPCRD" => {
                macro_rules! call {
                    (MSTP) => {
                        pac::peripherals::MSTP::steal().modify_d(bit, |r| *r |= 1 << bit);
                    };
                    ($other:ident) => {};
                }
                macro_rules! runner {
                    ($($name:ident = $index:expr,)*) => {
                        $( call!($name); )*
                    };
                }
                pac::foreach_peripheral!(runner);
            }
            "MSTPCRE" => {
                macro_rules! call {
                    (MSTP) => {
                        pac::peripherals::MSTP::steal().modify_e(bit, |r| *r |= 1 << bit);
                    };
                    ($other:ident) => {};
                }
                macro_rules! runner {
                    ($($name:ident = $index:expr,)*) => {
                        $( call!($name); )*
                    };
                }
                pac::foreach_peripheral!(runner);
            }
            _ => {}
        }
    }
}
