use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};

use critical_section::{CriticalSection, Mutex};
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;
use crate::pac;
use crate::peripherals;
use ra_metapac::timer::{GTP, regs};

#[cfg(feature = "defmt")]
use defmt::*;
#[cfg(not(feature = "defmt"))]
use log::*;

// Include auto-generated IRQ bindings from build.rs
include!(concat!(env!("OUT_DIR"), "/irq_bindings.rs"));

// Simple interrupt number struct for NVIC operations
#[derive(Copy, Clone)]
struct IrqNum(u16);

unsafe impl cortex_m::interrupt::InterruptNumber for IrqNum {
    fn number(self) -> u16 {
        self.0
    }
}

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
        let gpt = pac::GPT0;
        let bit_width = <peripherals::GPT0 as crate::pac::Peripheral>::metadata().bit_width.unwrap_or(32);
        loop {
            let hi = self.overflow_count.load(Ordering::Acquire);
            let lo = gpt.gtcnt().read();
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
        let gpt = pac::GPT0;
        let icu = pac::ICU;
        let bit_width = <peripherals::GPT0 as crate::pac::Peripheral>::metadata().bit_width.unwrap_or(32);
        debug!("Timer driver init ({}-bit, 8MHz)", bit_width);
        // Enable GPT0 clock
        let gpt0 = unsafe { peripherals::GPT0::steal() };
        unsafe { crate::mstp::enable_clock(gpt0) };

        // Stop timer
        gpt.gtcr().modify(|w| w.set_cst(false));

        // Set period to max
        if bit_width == 32 {
            gpt.gtpr().write_value(0xFFFF_FFFF);
        } else {
            gpt.gtpr().write_value(0x0000_FFFF);
        }
        gpt.gtcnt().write_value(0);

        // Get auto-allocated IRQ slots from build.rs
        let idx_ovf = irq_allocations::GPT0_COUNTER_OVERFLOW_IRQ as usize;
        let idx_ccmpa = irq_allocations::GPT0_CAPTURE_COMPARE_A_IRQ as usize;

        // Map events to allocated IELSR slots using auto-generated event IDs
        icu.ielsr(idx_ovf).write_value(event_ids::GPT0_COUNTER_OVERFLOW as u32);
        icu.ielsr(idx_ccmpa).write_value(event_ids::GPT0_CAPTURE_COMPARE_A as u32);

        // Clear any pending interrupts in ICU
        icu.ielsr(idx_ovf).modify(|w| *w &= !(1 << 16));
        icu.ielsr(idx_ccmpa).modify(|w| *w &= !(1 << 16));

        // Enable overflow interrupt and compare match A in GPT
        gpt.gtintad().modify(|w| {
            w.set_gtintv(true); // Overflow
            w.set_gtinta(true); // Compare Match A
        });

        // Enable interrupts in NVIC using auto-allocated IRQ numbers
        unsafe {
            cortex_m::peripheral::NVIC::unmask(IrqNum(idx_ovf as u16));
            cortex_m::peripheral::NVIC::unmask(IrqNum(idx_ccmpa as u16));
        }

        // Start timer
        gpt.gtcr().modify(|w| w.set_cst(true));
    }

    fn set_alarm(&self, at: u64) {
        let gpt = pac::GPT0;
        let bit_width = <peripherals::GPT0 as crate::pac::Peripheral>::metadata().bit_width.unwrap_or(32);
        
        let now = self.now();
        if at <= now {
            gpt.gtccra().write_value(gpt.gtcnt().read());
            return;
        }

        let diff = at - now;
        let max_diff = if bit_width == 32 { 0xFFFF_FFFF } else { 0x0000_FFFF };
        
        if diff < max_diff {
            gpt.gtccra().write_value((at & max_diff) as u32);
        } else {
            gpt.gtccra().write_value(max_diff as u32);
        }
    }
}

#[allow(non_snake_case)]
pub(crate) unsafe fn on_interrupt() {
    let irq = (cortex_m::peripheral::Peripherals::steal().SCB.icsr.read() & 0x1FF) as usize - 16;
    let icu = pac::ICU;
    let mut ielsr = icu.ielsr(irq).read();
    ielsr &= !(1 << 16); // Clear IR bit
    icu.ielsr(irq).write_value(ielsr);
    let _ = icu.ielsr(irq).read(); // Read back to ensure it's cleared

    let gpt = pac::GPT0;
    let st = gpt.gtst().read();
    
    if st.tcfpo() {
        // Overflow
        gpt.gtst().write(|w| {
            w.set_tcfpo(false);
        });
        DRIVER.overflow_count.fetch_add(1, Ordering::Relaxed);
    }

    if st.tcfa() {
        // Compare match A
        gpt.gtst().write(|w| {
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