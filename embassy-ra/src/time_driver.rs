use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};

use critical_section::{CriticalSection, Mutex};
use cortex_m::interrupt::InterruptNumber;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;
use defmt::*;
use crate::interrupt::{Event, InterruptRegistry, InterruptExt};
use crate::peripherals;
use crate::Irqs;

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
        let gpt = unsafe { peripherals::GPT0::steal() };
        loop {
            let hi = self.overflow_count.load(Ordering::Acquire);
            let lo = gpt.gtcnt().read() as u16;
            let hi2 = self.overflow_count.load(Ordering::Acquire);
            if hi == hi2 {
                return (hi as u64) << 16 | (lo as u64);
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
        let gpt = unsafe { peripherals::GPT0::steal() };
        let mstp = unsafe { peripherals::MSTP::steal() };
        let icu = unsafe { peripherals::ICU::steal() };
        debug!("Timer driver init (16-bit, 8MHz)");
        // Enable GPT0 clock
        // MSTPCRE bit 31 is GPT0
        mstp.mstpcre().modify(|w| w.set_mstpe31(false));

        // Stop timer
        gpt.gtcr().modify(|w| w.set_cst(false));

        // Set period to max 16-bit
        gpt.gtpr().write_value(0x0000_FFFF);
        gpt.gtcnt().write_value(0);

        // Get assigned IELs
        use crate::interrupt::typelevel::Interrupt;
        let irq_ovf = <Irqs as InterruptRegistry<crate::interrupt::events::Gpt0Ovf>>::Interrupt::IRQ;
        let irq_ccmpa = <Irqs as InterruptRegistry<crate::interrupt::events::Gpt0Ccmpa>>::Interrupt::IRQ;

        let idx_ovf = irq_ovf.number() as usize;
        let idx_ccmpa = irq_ccmpa.number() as usize;

        // Map Gpt0Ovf (0x14) to assigned IEL
        icu.ielsr(idx_ovf).write_value(crate::interrupt::events::Gpt0Ovf::ID as u32);
        // Map Gpt0Ccmpa (0x15) to assigned IEL
        icu.ielsr(idx_ccmpa).write_value(crate::interrupt::events::Gpt0Ccmpa::ID as u32);

        // Clear any pending interrupts in ICU
        icu.ielsr(idx_ovf).modify(|w| *w &= !(1 << 16));
        icu.ielsr(idx_ccmpa).modify(|w| *w &= !(1 << 16));

        // Enable overflow interrupt and compare match A in GPT
        gpt.gtintad().modify(|w| {
            w.set_gtintv(true); // Overflow
            w.set_gtinta(true); // Compare Match A
        });

        // Enable interrupts in NVIC
        unsafe {
            irq_ovf.enable();
            irq_ccmpa.enable();
        }

        // Start timer
        gpt.gtcr().modify(|w| w.set_cst(true));
    }

    fn set_alarm(&self, at: u64) {
        let gpt = unsafe { peripherals::GPT0::steal() };
        
        let now = self.now();
        if at <= now {
            gpt.gtccra().write_value(gpt.gtcnt().read());
            return;
        }

        let diff = at - now;
        if diff < 0xFFFF {
            // Target is in the current or next 16-bit cycle.
            gpt.gtccra().write_value((at & 0xFFFF) as u32);
        } else {
            // Too far away, let overflow handle it.
            gpt.gtccra().write_value(0xFFFF);
        }
    }
}

pub struct InterruptHandler<T>(core::marker::PhantomData<T>);

impl<T> crate::interrupt::Handler<crate::interrupt::events::Gpt0Ovf>
    for InterruptHandler<T>
{
    unsafe fn on_interrupt() {
        on_interrupt();
    }
}

impl<T> crate::interrupt::Handler<crate::interrupt::events::Gpt0Ccmpa>
    for InterruptHandler<T>
{
    unsafe fn on_interrupt() {
        on_interrupt();
    }
}

#[allow(non_snake_case)]
pub(crate) unsafe fn on_interrupt() {
    let irq = (cortex_m::peripheral::Peripherals::steal().SCB.icsr.read() & 0x1FF) as usize - 16;
    let icu = peripherals::ICU::steal();
    let mut ielsr = icu.ielsr(irq).read();
    ielsr &= !(1 << 16); // Clear IR bit
    icu.ielsr(irq).write_value(ielsr);
    let _ = icu.ielsr(irq).read(); // Read back to ensure it's cleared

    let gpt = peripherals::GPT0::steal();
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