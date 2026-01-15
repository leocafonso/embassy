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

// SYSC versions with MSTPCRA
macro_rules! impl_sysc_with_mstpcra {
    ($($version:ident),*) => {
        $(
            impl MstpRA for pac::_peripherals::$version::Sysc {
                unsafe fn modify_a(&self, _bit: u32, f: impl FnOnce(&mut u32)) {
                    self.mstpcra().modify(|w| {
                        let mut val = w.0 as u32;
                        f(&mut val);
                        w.0 = val as _;
                    })
                }
            }
        )*
    };
}

// SYSC versions without MSTPCRA (empty impl)
macro_rules! impl_sysc_without_mstpcra {
    ($($version:ident),*) => {
        $(
            impl MstpRA for pac::_peripherals::$version::Sysc {}
        )*
    };
}

// SYSC versions that have MSTPCRA
impl_sysc_with_mstpcra!(
    sysc_ra0,
    sysc_ra2a1,
    sysc_ra2a2,
    sysc_ra2e1,
    sysc_ra2e2,
    sysc_ra2l1,
    sysc_ra2t1,
    sysc_ra4m1,
    sysc_ra4w1,
    sysc_ra6m1,
    sysc_ra6m2,
    sysc_ra6t1
);

// SYSC versions that don't have MSTPCRA
impl_sysc_without_mstpcra!(
    sysc_ra4c1,
    sysc_ra4e1,
    sysc_ra4e2,
    sysc_ra4l1,
    sysc_ra4m2,
    sysc_ra4m3,
    sysc_ra4t1,
    sysc_ra6e1,
    sysc_ra6m4,
    sysc_ra6m5,
    sysc_ra6t2,
    sysc_ra6t3,
    sysc_ra8d1,
    sysc_ra8e1,
    sysc_ra8e2,
    sysc_ra8m1,
    sysc_ra8t1,
    sysc_rka8d2,
    sysc_rka8m2,
    sysc_rka8p1,
    sysc_rka8t2
);

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
                    (SYSC) => {
                        pac::peripherals::SYSC::steal().modify_a(bit, |r| *r &= !(1 << bit));
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
                    (SYSC) => {
                        pac::peripherals::SYSC::steal().modify_a(bit, |r| *r |= 1 << bit);
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
