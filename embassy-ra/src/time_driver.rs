/// Embassy time driver for Renesas RA using AGT0 (Asynchronous General-purpose Timer).
///
/// # Design
///
/// AGT0 is a 16-bit **down-counter** that:
///   - Counts from the reload value (0xFFFF) down to 0x0000
///   - On underflow reloads automatically and fires AGT0_INT
///   - Has a Compare Match A register (AGTCMA) that fires AGT0_COMPARE_A when
///     the counter == AGTCMA value
///
/// ## Time representation
///
/// We define one *period* = 2^15 ticks (half of the 16-bit range), mirroring
/// the mspm0 driver approach.  The 16-bit AGT counter runs from 0xFFFF down;
/// `calc_now` converts (period, counter) into a monotonic u64.
///
/// ## Interrupts
///
/// - **AGT0_INT** (underflow, every 65536 ticks): increments `period`, and
///   re-enables Compare Match A if the alarm falls within the coming window.
/// - **AGT0_COMPARE_A**: fires the alarm — wakes expired queue entries.
///
/// ## Clock source
///
/// AGTMR1.TCK = 0b100 → AGTLCLK (sub-clock oscillator, 32.768 kHz).
/// AGTMR2.CKS = 0b000 → /1 divider.
/// This matches the `tick-hz-32_768` feature used by the ra2e1 example.
use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};

use critical_section::{CriticalSection, Mutex};
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;
use ra_metapac::agt::Agt;
use ra_metapac::icu::Icu;

use crate::{interrupt, mstp, pac, peripherals};

use defmt::*;

// Include auto-generated IRQ bindings from build.rs
include!(concat!(env!("OUT_DIR"), "/irq_bindings.rs"));

// ============================================================================
// AGT channel selection
// ============================================================================

mod agt_cfg {
    use super::{pac, peripherals, Agt};
    pub type T = peripherals::AGT0;
    pub const AGT: Agt = pac::AGT0;
    pub use super::irq_allocations::AGT0_INT_IRQ as INT_IRQ;
    pub use super::irq_allocations::AGT0_COMPARE_A_IRQ as CMPA_IRQ;
    pub use super::event_ids::AGT0_INT as INT_EVENT;
    pub use super::event_ids::AGT0_COMPARE_A as CMPA_EVENT;
}

#[inline(always)]
fn regs() -> Agt {
    agt_cfg::AGT
}

#[inline(always)]
fn icu() -> Icu {
    ra_metapac::ICU
}

/// Thin wrapper so NVIC calls compile.
#[derive(Copy, Clone)]
struct IrqNum(u16);
unsafe impl cortex_m::interrupt::InterruptNumber for IrqNum {
    fn number(self) -> u16 {
        self.0
    }
}

// ============================================================================
// Interrupt handler wiring
// ============================================================================

struct TimeDriverInterruptHandler;

// Bind both AGT0_INT and AGT0_COMPARE_A to the shared on_interrupt() function.
impl interrupt::Handler<interrupt::events::Agt0Int> for TimeDriverInterruptHandler {
    unsafe fn on_interrupt() {
        on_interrupt();
    }
}

impl interrupt::Handler<interrupt::events::Agt0CompareA> for TimeDriverInterruptHandler {
    unsafe fn on_interrupt() {
        on_interrupt();
    }
}

crate::bind_interrupts!(struct TimeDriverIrqs {
    Agt0Int      => TimeDriverInterruptHandler;
    Agt0CompareA => TimeDriverInterruptHandler;
});

const _: () = {
    let _ = core::mem::size_of::<TimeDriverIrqs>();
};

// ============================================================================
// Time math
// ============================================================================

/// Convert (period, raw AGT down-counter value) → monotonic tick count.
///
/// AGT underflows every 65536 ticks, so one period = 65536 ticks.
///   elapsed_in_period = 0xFFFF - counter   (0 … 65535)
///   now = period * 65536 + elapsed
///
/// The retry loop in `now()` handles the race between reading `period` and
/// reading `counter` without needing a mutex or XOR trick.
#[inline]
fn calc_now(period: u32, counter: u16) -> u64 {
    ((period as u64) << 16) | (0xFFFFu16.wrapping_sub(counter)) as u64
}

// ============================================================================
// Driver struct
// ============================================================================

struct TimxDriver {
    /// Number of 2^15-tick periods elapsed since boot.
    period: AtomicU32,
    /// Alarm timestamp in ticks.  u64::MAX = no alarm pending.
    alarm: Mutex<Cell<u64>>,
    /// Waker queue.
    queue: Mutex<RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimxDriver = TimxDriver {
    period: AtomicU32::new(0),
    alarm:  Mutex::new(Cell::new(u64::MAX)),
    queue:  Mutex::new(RefCell::new(Queue::new())),
});

// ============================================================================
// Driver trait
// ============================================================================

impl Driver for TimxDriver {
    fn now(&self) -> u64 {
        let r = regs();
        let mut retries = 0u32;
        loop {
            let period = self.period.load(Ordering::Relaxed);
            compiler_fence(Ordering::Acquire);
            // AGT.agt() read returns the live counter (down-counting).
            let counter = r.agt().read();
            compiler_fence(Ordering::Acquire);
            let period2 = self.period.load(Ordering::Relaxed);
            if period == period2 {
                let now = calc_now(period, counter);
                if retries > 0 {
                    debug!("now() retried {} times: period={} counter={} -> {}", retries, period, counter, now);
                } else {
                    debug!("now() period={} counter={} -> {}", period, counter, now);
                }
                return now;
            }
            retries += 1;
            debug!("now() retry #{}: period={} != period2={}", retries, period, period2);
            // Period bumped while reading → retry.
        }
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        debug!("schedule_wake: at={}", at);
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();
            if queue.schedule_wake(at, waker) {
                debug!("schedule_wake: queue updated, calling next_expiration");
                let mut next = queue.next_expiration(self.now());
                debug!("schedule_wake: next_expiration returned {}", next);
                while !self.set_alarm(cs, next) {
                    debug!("schedule_wake: set_alarm returned false, retrying");
                    next = queue.next_expiration(self.now());
                    debug!("schedule_wake: next_expiration returned {}", next);
                }
            } else {
                debug!("schedule_wake: queue.schedule_wake returned false (no rearm needed)");
            }
        });
    }
}

// ============================================================================
// Internal helpers
// ============================================================================

impl TimxDriver {
    fn init(&'static self, _cs: CriticalSection) {
        let r = regs();
        let icu = icu();

        // 1. Release AGT0 from module stop.
        // SAFETY: we are in the one-time init, no other code uses AGT0.
        unsafe {
            mstp::enable_clock(*agt_cfg::T::steal())
        };

        // 2. Stop the counter before touching configuration registers.
        r.agtcr().write(|w| w.0 = 0);
        while r.agtcr().read().tcstf() == ra_metapac::agt::vals::Tcstf::from_bits(1) {}

        // 3. Timer mode, AGTLCLK source (/1) → 32.768 kHz ticks.
        r.agtmr1().write(|w| {
            w.set_tmod(ra_metapac::agt::vals::Tmod::from_bits(1)); // timer mode
            w.set_tck(ra_metapac::agt::vals::Tck::from_bits(4));   // AGTLCLK
        });
        r.agtmr2().write(|w| {
            w.set_cks(ra_metapac::agt::vals::Cks::from_bits(0)); // /1
        });

        // 4. Write reload value.  Per the RA2E1 manual: "the write value is
        //    written to the reload register; the read value is read from the
        //    counter."  Writing 0xFFFF makes the counter start at 0xFFFF.
        r.agt().write_value(0xFFFF);

        // 5. AGTCMSR = 0: Compare Match A disabled at startup.
        r.agtcmsr().write(|w| w.0 = 0);

        // 6. Program ICU: map events to the build-assigned IEL slots.
        let int_slot  = agt_cfg::INT_IRQ  as usize;
        let cmpa_slot = agt_cfg::CMPA_IRQ as usize;

        icu.ielsr(int_slot).write(|w| {
            w.set_iels(ra_metapac::icu::vals::Iels::from_bits(agt_cfg::INT_EVENT));
            w.set_ir(false);
        });
        icu.ielsr(cmpa_slot).write(|w| {
            w.set_iels(ra_metapac::icu::vals::Iels::from_bits(agt_cfg::CMPA_EVENT));
            w.set_ir(false);
        });

        // 7. Unmask both IRQs in the NVIC.
        unsafe {
            cortex_m::peripheral::NVIC::unmask(IrqNum(int_slot  as u16));
            cortex_m::peripheral::NVIC::unmask(IrqNum(cmpa_slot as u16));
        }

        // 8. Start the counter.
        r.agtcr().modify(|w| {
            w.set_tstart(ra_metapac::agt::vals::Tstart::from_bits(1))
        });
    }

    /// Called on every underflow (AGT0_INT, every 65536 ticks = 1 period).
    /// Increments the period counter and re-enables Compare Match A if the
    /// pending alarm falls within the next window.
    fn next_period(&self) {
        let r = regs();

        let period = self.period.load(Ordering::Relaxed) + 1;
        self.period.store(period, Ordering::Relaxed);

        critical_section::with(|cs| {
            let at = self.alarm.borrow(cs).get();
            // AGTCMA writes are buffered: the compare register is loaded at the
            // underflow.  Enable TCMEA one period before alarm_period so the
            // buffered AGTCMA value is clocked in at the underflow from
            // (alarm_period-1) → alarm_period, and CmpA fires in alarm_period.
            let alarm_period = (at >> 16) as u32;
            let enable = alarm_period > 0 && period == alarm_period - 1;
            debug!("next_period: period={} alarm_at={} alarm_period={} enable={}",
                   period, at, alarm_period, enable);
            if enable {
                r.agtcmsr().modify(|w| {
                    w.set_tcmea(ra_metapac::agt::vals::Tcmea::from_bits(1))
                });
            }
        });
    }

    /// Arms the Compare Match A alarm for `timestamp`.
    /// Returns `false` if the timestamp has already passed (caller retries).
    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let r = regs();

        // u64::MAX is the "no pending alarm" sentinel.  Writing it into AGTCMA
        // would produce 0x0000, which coincides with the underflow counter value
        // and causes a spurious Compare Match A interrupt every period.
        if timestamp == u64::MAX {
            self.disarm(cs);
            return true;
        }

        self.alarm.borrow(cs).set(timestamp);

        let t = self.now();
        if timestamp <= t {
            self.disarm(cs);
            return false;
        }

        // AGTCMA value = 0xFFFF - (timestamp % 65536).
        // This is the counter value at which Compare Match A fires within the
        // alarm's period.  `timestamp & 0xFFFF` == `timestamp % 65536`.
        let target_counter = 0xFFFFu16.wrapping_sub((timestamp & 0xFFFF) as u16);
        r.agtcma().write_value(target_counter);
        debug!("set_alarm: AGTCMA={} (ts={} & 0xFFFF = {})",
               target_counter, timestamp, timestamp & 0xFFFF);

        // AGTCMA writes while the counter is running are buffered: they only
        // take effect at the next underflow (per the RA2E1 manual).
        // So CmpA will actually fire in period (alarm_period), but the compare
        // register is loaded at the underflow from (alarm_period-1) → alarm_period.
        // We must enable TCMEA one period *before* alarm_period so the buffer
        // flushes at the right underflow.
        let current_period = self.period.load(Ordering::Relaxed);
        let alarm_period   = (timestamp >> 16) as u32;
        // TCMEA should be on whenever current_period == alarm_period - 1
        // (so it arms at the next underflow) OR if alarm_period == 0 (edge case).
        let enable_now = alarm_period > 0 && current_period == alarm_period - 1
            || alarm_period == 0 && current_period == 0;
        debug!("set_alarm: ts={} alarm_period={} cur_period={} tcmea={}",
               timestamp, alarm_period, current_period, enable_now);
        r.agtcmsr().modify(|w| {
            w.set_tcmea(ra_metapac::agt::vals::Tcmea::from_bits(
                if enable_now { 1 } else { 0 },
            ))
        });

        // Race check: did now() pass the timestamp between our first check and
        // writing AGTCMA?
        let t2 = self.now();
        if timestamp <= t2 {
            debug!("set_alarm: RACE - timestamp {} <= t2 {}, disarming", timestamp, t2);
            self.disarm(cs);
            return false;
        }

        true
    }

    /// Disarms Compare Match A and clears the stored alarm.
    #[inline]
    fn disarm(&self, cs: CriticalSection) {
        let r = regs();
        r.agtcmsr().modify(|w| {
            w.set_tcmea(ra_metapac::agt::vals::Tcmea::from_bits(0))
        });
        self.alarm.borrow(cs).set(u64::MAX);
    }

    /// Called from the Compare Match A ISR: fire expired wakers, re-arm.
    fn trigger_alarm(&self, cs: CriticalSection) {
        let now = self.now();
        debug!("trigger_alarm: now={}", now);
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(now);
        debug!("trigger_alarm: next_expiration={}", next);
        while !self.set_alarm(cs, next) {
            let now2 = self.now();
            debug!("trigger_alarm: retry now={}", now2);
            next = self.queue.borrow(cs).borrow_mut().next_expiration(now2);
            debug!("trigger_alarm: next_expiration={}", next);
        }
    }
}

// ============================================================================
// Shared ISR (called from both IEL handlers)
// ============================================================================

pub(crate) unsafe fn on_interrupt() {
    // Read the active exception number from ICSR to know which slot fired.
    let irq = (cortex_m::peripheral::Peripherals::steal().SCB.icsr.read() & 0x1FF) as usize - 16;
    // Clear the ICU pending flag before processing to allow re-triggering.
    interrupt::clear_icu_ir(irq);

    let r = regs();

    if irq == agt_cfg::INT_IRQ as usize {
        debug!("IRQ: Underflow (slot {})", irq);
        // Underflow: clear TUNDF, advance period.
        r.agtcr().modify(|w| {
            w.set_tundf(ra_metapac::agt::vals::Tundf::from_bits(0))
        });
        DRIVER.next_period();
    } else if irq == agt_cfg::CMPA_IRQ as usize {
        // Compare Match A: disable match, clear flag, fire alarm.
        debug!("IRQ: Compare Match A (slot {})", irq);
        r.agtcmsr().modify(|w| {
            w.set_tcmea(ra_metapac::agt::vals::Tcmea::from_bits(0))
        });
        r.agtcr().modify(|w| {
            w.set_tcmaf(ra_metapac::agt::vals::Tcmaf::from_bits(0))
        });
        critical_section::with(|cs| {
            DRIVER.trigger_alarm(cs);
        });
    }
}

// ============================================================================
// Init
// ============================================================================

pub(crate) fn init(cs: CriticalSection) {
    DRIVER.init(cs);
}
