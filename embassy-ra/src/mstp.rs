use crate::pac;
use ra_metapac::mstp::Mstp;

fn mstp() -> Mstp {
    ra_metapac::MSTP
}

// RA2 and RA6 non-TZ: MSTPCRA lives in SYSC, not MSTP
#[cfg(any(ra2e1, ra2e2, ra2l1, ra6m1, ra6m2, ra6m3, ra6e1, ra6t1, ra6t2))]
fn sysc() -> ra_metapac::sysc::Sysc {
    ra_metapac::SYSC
}

// ============================================================================
// MSTP Register Access
// ============================================================================
//
// MSTPCRA location differs by family:
// - RA2, RA6 non-TZ: MSTPCRA is in SYSC
// - RA6 TZ (ra6e2, ra6m4, ra6m5): MSTPCRA is in MSTP

// RA6 TZ: MSTPCRA in MSTP
#[cfg(any(ra6e2, ra6m4, ra6m5))]
#[allow(unused_variables)]
unsafe fn modify_mstpcra(bit: u32, f: impl FnOnce(&mut u32)) {
    let mstp = mstp();
    mstp.mstpcra().modify(|w| {
        let mut val = w.0 as u32;
        f(&mut val);
        w.0 = val as _;
    });
    let _ = mstp.mstpcra().read();
}

// RA2, RA6 non-TZ: MSTPCRA in SYSC
#[cfg(any(ra2e1, ra2e2, ra2l1, ra6m1, ra6m2, ra6m3, ra6e1, ra6t1, ra6t2))]
#[allow(unused_variables)]
unsafe fn modify_mstpcra(bit: u32, f: impl FnOnce(&mut u32)) {
    let sysc = sysc();
    sysc.mstpcra().modify(|w| {
        let mut val = w.0 as u32;
        f(&mut val);
        w.0 = val as _;
    });
    let _ = sysc.mstpcra().read();
}

#[allow(unused_variables)]
unsafe fn modify_mstpcrb(bit: u32, f: impl FnOnce(&mut u32)) {
    let mstp = mstp();
    mstp.mstpcrb().modify(|w| {
        let mut val = w.0 as u32;
        f(&mut val);
        w.0 = val as _;
    });
    let _ = mstp.mstpcrb().read();
}

#[allow(unused_variables)]
unsafe fn modify_mstpcrc(bit: u32, f: impl FnOnce(&mut u32)) {
    let mstp = mstp();
    mstp.mstpcrc().modify(|w| {
        let mut val = w.0 as u32;
        f(&mut val);
        w.0 = val as _;
    });
    let _ = mstp.mstpcrc().read();
}

#[allow(unused_variables)]
unsafe fn modify_mstpcrd(bit: u32, f: impl FnOnce(&mut u32)) {
    let mstp = mstp();
    debug!("bit {}", bit);
    mstp.mstpcrd().modify(|w| {
        let mut val = w.0 as u32;
        f(&mut val);
        w.0 = val as _;
    });
    let _ = mstp.mstpcrd().read();
    debug!("MSTPCRD after modification: {:032b}", mstp.mstpcrd().read().0);
}

// MSTPCRE only exists on RA6 (all variants)
#[cfg(any(ra6e1, ra6e2, ra6m1, ra6m2, ra6m3, ra6m4, ra6m5, ra6t1, ra6t2))]
#[allow(unused_variables)]
unsafe fn modify_mstpcre(bit: u32, f: impl FnOnce(&mut u32)) {
    let mstp = mstp();
    mstp.mstpcre().modify(|w| {
        let mut val = w.0 as u32;
        f(&mut val);
        w.0 = val as _;
    });
    let _ = mstp.mstpcre().read();
}

// RA2 has no MSTPCRE — no-op
#[cfg(any(ra2e1, ra2e2, ra2l1))]
#[allow(unused_variables)]
unsafe fn modify_mstpcre(_bit: u32, _f: impl FnOnce(&mut u32)) {}

// ============================================================================
// Peripheral Trait for MSTP Control
// ============================================================================

pub trait SealedPeripheral {
    fn metadata() -> &'static pac::metadata::Peripheral;
}

macro_rules! impl_peripheral {
    ($($name:ident = $index:expr,)*) => {
        $(
            impl SealedPeripheral for crate::peripherals::$name {
                fn metadata() -> &'static pac::metadata::Peripheral {
                    &pac::metadata::METADATA.peripherals[$index]
                }
            }
        )*
    };
}

pac::foreach_peripheral!(impl_peripheral);

pub trait Peripheral: SealedPeripheral {}
impl<T: SealedPeripheral> Peripheral for T {}

// ============================================================================
// Clock Enable/Disable
// ============================================================================

pub unsafe fn enable_clock<T: Peripheral>(_peri: T) {
    let metadata = T::metadata();
    debug!("Enabling clock for {}", metadata.name);
    if let Some(mstp) = metadata.mstp {
        let bit = mstp.bit;
        debug!("MSTP register: {} bit: {}", mstp.register, bit);
        match mstp.register {
            "MSTPCRA" => modify_mstpcra(bit, |r| *r &= !(1 << bit)),
            "MSTPCRB" => modify_mstpcrb(bit, |r| *r &= !(1 << bit)),
            "MSTPCRC" => modify_mstpcrc(bit, |r| *r &= !(1 << bit)),
            "MSTPCRD" => modify_mstpcrd(bit, |r| *r &= !(1 << bit)),
            "MSTPCRE" => modify_mstpcre(bit, |r| *r &= !(1 << bit)),
            _ => {}
        }
    }
}

pub unsafe fn disable_clock<T: Peripheral>(_peri: T) {
    let metadata = T::metadata();
    debug!("Disabling clock for {}", metadata.name);
    if let Some(mstp) = metadata.mstp {
        let bit = mstp.bit;
        match mstp.register {
            "MSTPCRA" => modify_mstpcra(bit, |r| *r |= 1 << bit),
            "MSTPCRB" => modify_mstpcrb(bit, |r| *r |= 1 << bit),
            "MSTPCRC" => modify_mstpcrc(bit, |r| *r |= 1 << bit),
            "MSTPCRD" => modify_mstpcrd(bit, |r| *r |= 1 << bit),
            "MSTPCRE" => modify_mstpcre(bit, |r| *r |= 1 << bit),
            _ => {}
        }
    }
}
