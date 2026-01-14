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
    const IRQ_NUMBERS: &'static [u8];
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
        ($(($name:ident, $id:tt, ($($irq_num:tt),*)),)*) => {
            $(
                pub struct $name;
                impl Event for $name {
                    const ID: u16 = $id;
                    const IRQ_NUMBERS: &'static [u8] = &[$($irq_num),*];
                }
            )*
        };
    }

    crate::pac::foreach_event!(event_structs);
}

/// Const function to check if an IRQ number is valid for an event.
/// Returns true if the event has no restrictions (empty list) or if the IRQ is in the list.
pub const fn is_valid_irq_for_event(irq_numbers: &[u8], irq: u8) -> bool {
    // Empty list means unrestricted
    if irq_numbers.is_empty() {
        return true;
    }
    // Check if IRQ is in the allowed list
    let mut i = 0;
    while i < irq_numbers.len() {
        if irq_numbers[i] == irq {
            return true;
        }
        i += 1;
    }
    false
}

/// Simplified bind_interrupts macro that requires explicit IRQ specification.
/// For grouped events (RA2 family), the IRQ must be one of the allowed positions.
/// For unrestricted events, any IRQ can be used.
///
/// Usage:
/// ```ignore
/// bind_interrupts!(struct Irqs {
///     IEL0 => TimerHandler, Gpt0CounterOverflow;
///     IEL4 => UartHandler, SciUart0Rxi;
/// });
/// ```
///
/// Note: For RA2 devices with grouped interrupts, ensure the IRQ slot is valid for the event.
/// The Event::IRQ_NUMBERS constant contains the allowed IELSR indices for each event.
/// Using an invalid slot will compile but the interrupt won't fire correctly at runtime.
#[macro_export]
macro_rules! bind_interrupts {
    ($vis:vis struct $name:ident {
        $(
            $irq:ident => $handler:ty, $event:ident;
        )*
    }) => {
        $vis struct $name;

        $(
            #[no_mangle]
            unsafe extern "C" fn $irq() {
                <$handler as $crate::interrupt::Handler<$crate::interrupt::events::$event>>::on_interrupt();
            }

            unsafe impl $crate::interrupt::Binding<$crate::interrupt::events::$event, $handler> for $name {}

            impl $crate::interrupt::InterruptRegistry<$crate::interrupt::events::$event> for $name {
                type Interrupt = $crate::interrupt::typelevel::$irq;
            }
        )*
    };
}
