use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};

use critical_section::{CriticalSection, Mutex};
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;
use ra_metapac::gpt;

#[cfg(feature = "defmt")]
use defmt::*;
#[cfg(not(feature = "defmt"))]
use log::*;

// Include auto-generated IRQ bindings from build.rs
include!(concat!(env!("OUT_DIR"), "/irq_bindings.rs"));

struct TimeDriverInterruptHandler;

// ============================================================================
// GPT channel selection via cfg
// ============================================================================

#[cfg(time_driver_gpt0)]
mod gpt_cfg {
    pub type T = crate::peripherals::GPT0;
    pub const GPT: crate::pac::gpt::Gpt = unsafe { 
        crate::pac::gpt::Gpt::from_ptr(crate::pac::GPT0.as_ptr()) 
    };
    pub use super::irq_allocations::{GPT0_COUNTER_OVERFLOW_IRQ as OVF_IRQ, GPT0_CAPTURE_COMPARE_A_IRQ as CCMPA_IRQ};
    pub use super::event_ids::{GPT0_COUNTER_OVERFLOW as OVF_EVENT, GPT0_CAPTURE_COMPARE_A as CCMPA_EVENT};
}

#[cfg(time_driver_gpt1)]
mod gpt_cfg {
    pub type T = crate::peripherals::GPT1;
    pub const GPT: crate::pac::gpt::Gpt = unsafe { 
        crate::pac::gpt::Gpt::from_ptr(crate::pac::GPT1.as_ptr()) 
    };
    pub use super::irq_allocations::{GPT1_COUNTER_OVERFLOW_IRQ as OVF_IRQ, GPT1_CAPTURE_COMPARE_A_IRQ as CCMPA_IRQ};
    pub use super::event_ids::{GPT1_COUNTER_OVERFLOW as OVF_EVENT, GPT1_CAPTURE_COMPARE_A as CCMPA_EVENT};
}

#[cfg(time_driver_gpt2)]
mod gpt_cfg {
    pub type T = crate::peripherals::GPT2;
    pub const GPT: crate::pac::gpt::Gpt = unsafe { 
        crate::pac::gpt::Gpt::from_ptr(crate::pac::GPT2.as_ptr()) 
    };
    pub use super::irq_allocations::{GPT2_COUNTER_OVERFLOW_IRQ as OVF_IRQ, GPT2_CAPTURE_COMPARE_A_IRQ as CCMPA_IRQ};
    pub use super::event_ids::{GPT2_COUNTER_OVERFLOW as OVF_EVENT, GPT2_CAPTURE_COMPARE_A as CCMPA_EVENT};
}

#[cfg(time_driver_gpt3)]
mod gpt_cfg {
    pub type T = crate::peripherals::GPT3;
    pub const GPT: crate::pac::gpt::Gpt = unsafe { 
        crate::pac::gpt::Gpt::from_ptr(crate::pac::GPT3.as_ptr()) 
    };
    pub use super::irq_allocations::{GPT3_COUNTER_OVERFLOW_IRQ as OVF_IRQ, GPT3_CAPTURE_COMPARE_A_IRQ as CCMPA_IRQ};
    pub use super::event_ids::{GPT3_COUNTER_OVERFLOW as OVF_EVENT, GPT3_CAPTURE_COMPARE_A as CCMPA_EVENT};
}

#[cfg(time_driver_gpt4)]
mod gpt_cfg {
    pub type T = crate::peripherals::GPT4;
    pub const GPT: crate::pac::gpt::Gpt = unsafe { 
        crate::pac::gpt::Gpt::from_ptr(crate::pac::GPT4.as_ptr()) 
    };
    pub use super::irq_allocations::{GPT4_COUNTER_OVERFLOW_IRQ as OVF_IRQ, GPT4_CAPTURE_COMPARE_A_IRQ as CCMPA_IRQ};
    pub use super::event_ids::{GPT4_COUNTER_OVERFLOW as OVF_EVENT, GPT4_CAPTURE_COMPARE_A as CCMPA_EVENT};
}

#[cfg(time_driver_gpt5)]
mod gpt_cfg {
    pub type T = crate::peripherals::GPT5;
    pub const GPT: crate::pac::gpt::Gpt = unsafe { 
        crate::pac::gpt::Gpt::from_ptr(crate::pac::GPT5.as_ptr()) 
    };
    pub use super::irq_allocations::{GPT5_COUNTER_OVERFLOW_IRQ as OVF_IRQ, GPT5_CAPTURE_COMPARE_A_IRQ as CCMPA_IRQ};
    pub use super::event_ids::{GPT5_COUNTER_OVERFLOW as OVF_EVENT, GPT5_CAPTURE_COMPARE_A as CCMPA_EVENT};
}

// Re-export selected GPT configuration
#[allow(unused_imports)]
use gpt_cfg::*;

fn regs() -> gpt::Gpt {
    gpt_cfg::GPT
}

fn icu() -> ra_metapac::icu::Icu {
    ra_metapac::ICU
}

// Simple interrupt number struct for NVIC operations
#[derive(Copy, Clone)]
struct IrqNum(u16);

unsafe impl cortex_m::interrupt::InterruptNumber for IrqNum {
    fn number(self) -> u16 {
        self.0
    }
}

macro_rules! impl_time_driver_event_handlers {
    ($ovf:ident, $ccmpa:ident) => {
        impl crate::interrupt::Handler<crate::interrupt::events::$ovf> for TimeDriverInterruptHandler {
            unsafe fn on_interrupt() {
                on_interrupt();
            }
        }
        impl crate::interrupt::Handler<crate::interrupt::events::$ccmpa> for TimeDriverInterruptHandler {
            unsafe fn on_interrupt() {
                on_interrupt();
            }
        }

        crate::bind_interrupts!(struct TimeDriverIrqs {
            $ovf => TimeDriverInterruptHandler;
            $ccmpa => TimeDriverInterruptHandler;
        });

        const _: () = {
            let _ = core::mem::size_of::<TimeDriverIrqs>();
        };
    };
}

#[cfg(time_driver_gpt0)]
impl_time_driver_event_handlers!(Gpt0CounterOverflow, Gpt0CaptureCompareA);
#[cfg(time_driver_gpt1)]
impl_time_driver_event_handlers!(Gpt1CounterOverflow, Gpt1CaptureCompareA);
#[cfg(time_driver_gpt2)]
impl_time_driver_event_handlers!(Gpt2CounterOverflow, Gpt2CaptureCompareA);
#[cfg(time_driver_gpt3)]
impl_time_driver_event_handlers!(Gpt3CounterOverflow, Gpt3CaptureCompareA);
#[cfg(time_driver_gpt4)]
impl_time_driver_event_handlers!(Gpt4CounterOverflow, Gpt4CaptureCompareA);
#[cfg(time_driver_gpt5)]
impl_time_driver_event_handlers!(Gpt5CounterOverflow, Gpt5CaptureCompareA);
#[cfg(time_driver_gpt6)]
impl_time_driver_event_handlers!(Gpt6CounterOverflow, Gpt6CaptureCompareA);
#[cfg(time_driver_gpt7)]
impl_time_driver_event_handlers!(Gpt7CounterOverflow, Gpt7CaptureCompareA);
#[cfg(time_driver_gpt8)]
impl_time_driver_event_handlers!(Gpt8CounterOverflow, Gpt8CaptureCompareA);
#[cfg(time_driver_gpt9)]
impl_time_driver_event_handlers!(Gpt9CounterOverflow, Gpt9CaptureCompareA);

struct TimerDriver {
    overflow_count: AtomicU32,
    queue: Mutex<RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimerDriver = TimerDriver {
    overflow_count: AtomicU32::new(0),
    queue: Mutex::new(RefCell::new(Queue::new()))
});

impl Driver for TimerDriver {
    fn now(&self) -> u64 {
        let r = regs();
        let bit_width = gpt_bit_width();
        loop {
            let hi = self.overflow_count.load(Ordering::Acquire);
            let lo = r.gtcnt().read();
            let hi2 = self.overflow_count.load(Ordering::Acquire);
            if hi == hi2 {
                if bit_width == 32 {
                    return (hi as u64) << 32 | (lo as u64);
                } else {
                    return (hi as u64) << 16 | (lo as u64);
                }
            }
        }
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();
            if queue.schedule_wake(at, waker) {
                let now = self.now();
                let next = queue.next_expiration(now);
                self.set_alarm(next);
            }
        })
    }
}

impl TimerDriver {
    fn init(&'static self, _cs: CriticalSection) {
        let r = regs();
        let icu = icu();
        let bit_width = gpt_bit_width();
        debug!("Timer driver init ({}-bit, 8MHz)", bit_width);
        
        // Enable GPT clock using type T
        unsafe { crate::mstp::enable_clock(*T::steal()) };

        // Stop timer
        r.gtcr().modify(|w| w.set_cst(false));

        // Set period to max
        if bit_width == 32 {
            r.gtpr().write_value(0xFFFF_FFFF);
        } else {
            r.gtpr().write_value(0x0000_FFFF);
        }
        r.gtcnt().write_value(0);

        // Get auto-allocated IRQ slots from build.rs (now uses selected GPT channel)
        let idx_ovf = gpt_cfg::OVF_IRQ as usize;
        let idx_ccmpa = gpt_cfg::CCMPA_IRQ as usize;

        // Map events to allocated IELSR slots using auto-generated event IDs
        icu.ielsr(idx_ovf).write_value(gpt_cfg::OVF_EVENT as u32);
        icu.ielsr(idx_ccmpa).write_value(gpt_cfg::CCMPA_EVENT as u32);

        debug!("OVF event={}, irq={}", gpt_cfg::OVF_EVENT, idx_ovf);
        debug!("CCMPA event={}, irq={}", gpt_cfg::CCMPA_EVENT, idx_ccmpa);

        // Clear any pending interrupts in ICU
        icu.ielsr(idx_ovf).modify(|w| *w &= !(1 << 16));
        icu.ielsr(idx_ccmpa).modify(|w| *w &= !(1 << 16));

        // Enable interrupts in NVIC using auto-allocated IRQ numbers
        unsafe {
            cortex_m::peripheral::NVIC::unmask(IrqNum(idx_ovf as u16));
            cortex_m::peripheral::NVIC::unmask(IrqNum(idx_ccmpa as u16));
        }

        // Start timer
        r.gtcr().modify(|w| w.set_cst(true));
    }

    fn set_alarm(&self, at: u64) {
        let r = regs();
        let bit_width = gpt_bit_width();
        
        let now = self.now();
        if at <= now {
            r.gtccra().write_value(r.gtcnt().read());
            return;
        }

        let diff = at - now;
        let max_diff = if bit_width == 32 { 0xFFFF_FFFF } else { 0x0000_FFFF };
        
        if diff < max_diff {
            r.gtccra().write_value((at & max_diff) as u32);
        } else {
            r.gtccra().write_value(max_diff as u32);
        }
    }
}

#[allow(non_snake_case)]
pub(crate) unsafe fn on_interrupt() {
    let irq = (cortex_m::peripheral::Peripherals::steal().SCB.icsr.read() & 0x1FF) as usize - 16;
    let icu = icu();
    let mut ielsr = icu.ielsr(irq).read();
    ielsr &= !(1 << 16); // Clear IR bit
    icu.ielsr(irq).write_value(ielsr);
    let _ = icu.ielsr(irq).read(); // Read back to ensure it's cleared

    let r = regs();
    let st = r.gtst().read();
    
    if st.tcfpo() {
        // Overflow
        r.gtst().write(|w| {
            w.set_tcfpo(false);
        });
        DRIVER.overflow_count.fetch_add(1, Ordering::Relaxed);
    }

    if st.tcfa() {
        // Compare match A
        r.gtst().write(|w| {
            w.set_tcfa(false);
        });
    }

    // Always trigger queue check
    critical_section::with(|cs| {
        let mut queue = DRIVER.queue.borrow(cs).borrow_mut();
        let now = DRIVER.now();
        let next = queue.next_expiration(now);
        DRIVER.set_alarm(next);
    })
}

pub(crate) fn init(cs: CriticalSection) {
    DRIVER.init(cs)
}

fn gpt_bit_width() -> u32 {
    // TODO: Get from peripheral metadata when traits are set up
    // For now, assume 16-bit for GPT16E variants
    16
}