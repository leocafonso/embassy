pub use embassy_hal_internal::interrupt::{InterruptExt, Priority};

embassy_hal_internal::interrupt_mod!(
    IEL0, IEL1, IEL2, IEL3, IEL4, IEL5, IEL6, IEL7,
    IEL8, IEL9, IEL10, IEL11, IEL12, IEL13, IEL14, IEL15,
    IEL16, IEL17, IEL18, IEL19, IEL20, IEL21, IEL22, IEL23,
    IEL24, IEL25, IEL26, IEL27, IEL28, IEL29, IEL30, IEL31
);

pub use self::interrupt::*;

pub trait Event {
    const ID: u8;
}

pub trait InterruptRegistry<E: Event> {
    type Interrupt: self::typelevel::Interrupt;
}

pub trait Handler<E: Event> {
    unsafe fn on_interrupt();
}

pub unsafe trait Binding<E: Event, H: Handler<E>> {}

pub mod events {
    use super::Event;

    pub struct Gpt0Ovf;
    impl Event for Gpt0Ovf {
        const ID: u8 = 0xc6;
    }

    pub struct Gpt0Ccmpa;
    impl Event for Gpt0Ccmpa {
        const ID: u8 = 0xc0;
    }
}

#[macro_export]
macro_rules! bind_interrupts {
    ($vis:vis struct $name:ident {
        $(
            $event:ident => $handler:ty;
        )*
    }) => {
        $vis struct $name;

        $crate::bind_interrupts!(@inner $name, (IEL0 IEL1 IEL2 IEL3 IEL4 IEL5 IEL6 IEL7 IEL8 IEL9 IEL10 IEL11 IEL12 IEL13 IEL14 IEL15 IEL16 IEL17 IEL18 IEL19 IEL20 IEL21 IEL22 IEL23 IEL24 IEL25 IEL26 IEL27 IEL28 IEL29 IEL30 IEL31), $($event => $handler;)*);
    };

    (@inner $name:ident, ($iel:ident $($rest_iel:ident)*), $event:ident => $handler:ty; $($rest:tt)*) => {
        #[no_mangle]
        unsafe extern "C" fn $iel() {
            <$handler as $crate::interrupt::Handler<$crate::interrupt::events::$event>>::on_interrupt();
        }

        unsafe impl $crate::interrupt::Binding<$crate::interrupt::events::$event, $handler> for $name {}

        impl $crate::interrupt::InterruptRegistry<$crate::interrupt::events::$event> for $name {
            type Interrupt = $crate::interrupt::typelevel::$iel;
        }

        $crate::bind_interrupts!(@inner $name, ($($rest_iel)*), $($rest)*);
    };

    (@inner $name:ident, ($($iel:ident)*), ) => {};
}
