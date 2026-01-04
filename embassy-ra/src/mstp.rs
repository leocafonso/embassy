use crate::pac;

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
        let mstp_peri = pac::peripherals::MSTP::steal();
        match mstp.register {
            "MSTPCRA" => mstp_peri.mstpcra().modify(|w| {
                w.0 &= !(1 << mstp.bit);
            }),
            "MSTPCRB" => mstp_peri.mstpcrb().modify(|w| {
                w.0 &= !(1 << mstp.bit);
            }),
            "MSTPCRC" => mstp_peri.mstpcrc().modify(|w| {
                w.0 &= !(1 << mstp.bit);
            }),
            "MSTPCRD" => mstp_peri.mstpcrd().modify(|w| {
                w.0 &= !(1 << mstp.bit);
            }),
            "MSTPCRE" => mstp_peri.mstpcre().modify(|w| {
                w.0 &= !(1 << mstp.bit);
            }),
            _ => {}
        }
    }
}

pub unsafe fn disable_clock<T: Peripheral>(_peri: T) {
    let metadata = T::metadata();
    if let Some(mstp) = metadata.mstp {
        let mstp_peri = pac::peripherals::MSTP::steal();
        match mstp.register {
            "MSTPCRA" => mstp_peri.mstpcra().modify(|w| {
                w.0 |= 1 << mstp.bit;
            }),
            "MSTPCRB" => mstp_peri.mstpcrb().modify(|w| {
                w.0 |= 1 << mstp.bit;
            }),
            "MSTPCRC" => mstp_peri.mstpcrc().modify(|w| {
                w.0 |= 1 << mstp.bit;
            }),
            "MSTPCRD" => mstp_peri.mstpcrd().modify(|w| {
                w.0 |= 1 << mstp.bit;
            }),
            "MSTPCRE" => mstp_peri.mstpcre().modify(|w| {
                w.0 |= 1 << mstp.bit;
            }),
            _ => {}
        }
    }
}
