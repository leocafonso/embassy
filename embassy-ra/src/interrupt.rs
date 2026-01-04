pub use embassy_hal_internal::interrupt::{InterruptExt, Priority};

macro_rules! irq_mod {
    ($($irq:ident = $num:expr,)*) => {
        embassy_hal_internal::interrupt_mod!($($irq),*);
    };
}
crate::pac::foreach_interrupt!(irq_mod);

pub use self::interrupt::*;

pub trait Event {
    const ID: u16;
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

    macro_rules! event_structs {
        ($($name:ident = $id:expr,)*) => {
            $(
                pub struct $name;
                impl Event for $name {
                    const ID: u16 = $id;
                }
            )*
        };
    }

    crate::pac::foreach_event!(event_structs);
}

#[macro_export]
macro_rules! bind_interrupts {
    ($vis:vis struct $name:ident {
        $(
            $event:ident => $handler:ty;
        )*
    }) => {
        $vis struct $name;

        $crate::pac::foreach_interrupt!($crate::bind_interrupts, @foreach $name, $($event => $handler;)*);
    };

    (@foreach $name:ident, $($event:ident => $handler:ty;)* { $($irq:ident = $num:expr,)* }) => {
        $crate::bind_interrupts!(@inner $name, ($($irq)*), $($event => $handler;)*);
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
