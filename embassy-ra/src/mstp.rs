use crate::pac;
use defmt::debug;
use ra_metapac::mstp::Mstp;


// Use direct PAC constants instead of steal pattern
fn mstp() -> Mstp{
    ra_metapac::MSTP
}

#[cfg(any(
    ra0e1, ra0e2,
    ra2a1, ra2a2, ra2e1, ra2e2, ra2e3, ra2l1, ra2l2, ra2t1,
    ra4m1, ra4w1,
    ra6m1, ra6m2, ra6m3, ra6t1
))]
fn sysc() -> Sysc {
    ra_metapac::SYSC
}

// ============================================================================
// MSTP Register Access
// ============================================================================
// 
// MSTP (Module Stop) registers control power to peripherals.
// Different RA families have different MSTP register layouts:
//
// Register availability by family:
// - RA0:     MSTPCRB, MSTPCRC, MSTPCRD (MSTPCRA in SYSC)
// - RA2:     MSTPCRB, MSTPCRC, MSTPCRD (MSTPCRA in SYSC)
// - RA4/6/8: MSTPCRA, MSTPCRB, MSTPCRC, MSTPCRD, MSTPCRE (in MSTP)
//
// For families without MSTPCRA in MSTP, it's located in SYSC instead.

// ============================================================================
// MSTPCRA Access
// ============================================================================

/// Families with MSTPCRA in MSTP peripheral (TrustZone-capable devices)
#[cfg(any(
    ra4c1, ra4e1, ra4e2, ra4l1, ra4m2, ra4m3, ra4t1,
    ra6e1, ra6e2, ra6m4, ra6m5, ra6t2, ra6t3,
    ra8d1, ra8e1, ra8e2, ra8m1, ra8p1, ra8t1
))]
#[allow(unused_variables)]
unsafe fn modify_mstpcra(bit: u32, f: impl FnOnce(&mut u32)) {
    let mstp = mstp();
    mstp.mstpcra().modify(|w| {
        let mut val = w.0 as u32;
        f(&mut val);
        w.0 = val as _;
    });
    // Dummy read for synchronization
    let _ = mstp.mstpcra().read();
}

/// Families with MSTPCRA in SYSC peripheral (non-TrustZone devices)
#[cfg(any(
    ra0e1, ra0e2,
    ra2a1, ra2a2, ra2e1, ra2e2, ra2e3, ra2l1, ra2l2, ra2t1,
    ra4m1, ra4w1,
    ra6m1, ra6m2, ra6m3, ra6t1
))]
#[allow(unused_variables)]
unsafe fn modify_mstpcra(bit: u32, f: impl FnOnce(&mut u32)) {
    let sysc = sysc();
    sysc.mstpcra().modify(|w| {
        let mut val = w.0 as u32;
        f(&mut val);
        w.0 = val as _;
    });
    // Dummy read for synchronization
    let _ = sysc.mstpcra().read();
}

// ============================================================================
// MSTPCRB Access (all families have this in MSTP)
// ============================================================================

#[allow(unused_variables)]
unsafe fn modify_mstpcrb(bit: u32, f: impl FnOnce(&mut u32)) {
    let mstp = mstp();
    mstp.mstpcrb().modify(|w| {
        let mut val = w.0 as u32;
        f(&mut val);
        w.0 = val as _;
    });
    // Dummy read for synchronization
    let _ = mstp.mstpcrb().read();
}

// ============================================================================
// MSTPCRC Access (all families have this in MSTP)
// ============================================================================

#[allow(unused_variables)]
unsafe fn modify_mstpcrc(bit: u32, f: impl FnOnce(&mut u32)) {
    let mstp = mstp();
    mstp.mstpcrc().modify(|w| {
        let mut val = w.0 as u32;
        f(&mut val);
        w.0 = val as _;
    });
    // Dummy read for synchronization
    let _ = mstp.mstpcrc().read();
}

// ============================================================================
// MSTPCRD Access (all families have this in MSTP)
// ============================================================================

#[allow(unused_variables)]
unsafe fn modify_mstpcrd(bit: u32, f: impl FnOnce(&mut u32)) {
    let mstp = mstp();
    debug!("bit {}", bit);
    mstp.mstpcrd().modify(|w| {
        let mut val = w.0 as u32;
        f(&mut val);
        w.0 = val as _;
    });
    // Dummy read for synchronization
    let _ = mstp.mstpcrd().read();
    debug!("MSTPCRD after modification: {:032b}", mstp.mstpcrd().read().0);
}

// ============================================================================
// MSTPCRE Access (only RA4E/RA6/RA8 families with TrustZone)
// ============================================================================

/// Families with MSTPCRE register
#[cfg(any(
    ra4c1, ra4e1, ra4e2, ra4l1, ra4m2, ra4m3, ra4t1,
    ra6e1, ra6e2, ra6m1, ra6m2, ra6m3, ra6m4, ra6m5, ra6t1, ra6t2, ra6t3,
    ra8d1, ra8e1, ra8e2, ra8m1, ra8p1, ra8t1
))]
#[allow(unused_variables)]
unsafe fn modify_mstpcre(bit: u32, f: impl FnOnce(&mut u32)) {
    let mstp = mstp();
    mstp.mstpcre().modify(|w| {
        let mut val = w.0 as u32;
        f(&mut val);
        w.0 = val as _;
    });
    // Dummy read for synchronization
    let _ = mstp.mstpcre().read();
}

/// Families without MSTPCRE register (no-op)
#[cfg(not(any(
    ra4c1, ra4e1, ra4e2, ra4l1, ra4m2, ra4m3, ra4t1,
    ra6e1, ra6e2, ra6m1, ra6m2, ra6m3, ra6m4, ra6m5, ra6t1, ra6t2, ra6t3,
    ra8d1, ra8e1, ra8e2, ra8m1, ra8p1, ra8t1
)))]
#[allow(unused_variables)]
unsafe fn modify_mstpcre(bit: u32, f: impl FnOnce(&mut u32)) {
    // MSTPCRE not available on this family
}

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
// Clock Enable/Disable Functions
// ============================================================================

/// Enable the clock for a peripheral by clearing its MSTP bit.
/// 
/// # Safety
/// This function modifies hardware registers and should be called with
/// appropriate synchronization if multiple contexts access the same MSTP register.
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

/// Disable the clock for a peripheral by setting its MSTP bit.
/// 
/// # Safety
/// This function modifies hardware registers and should be called with
/// appropriate synchronization if multiple contexts access the same MSTP register.
pub unsafe fn disable_clock<T: Peripheral>(_peri: T) {
    let metadata = T::metadata();
    debug!("Disabling clock for {}", metadata.name);
    if let Some(mstp) = metadata.mstp {
        let bit = mstp.bit;
        debug!("MSTP register: {} bit: {}", mstp.register, bit);
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
