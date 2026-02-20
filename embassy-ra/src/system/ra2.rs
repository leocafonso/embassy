//! Clock configuration for RA6 family (RA2E1, RA2E2, RA2E3)

use defmt::debug;

use ra_metapac::sysc::Sysc;
use super::{Clocks, ClockDiv, ClockSource, Hertz, MainOscConfig, MOCO_FREQ, LOCO_FREQ};

// Use direct PAC access
fn sysc() -> Sysc {
    ra_metapac::SYSC
}

/// Maximum ICLK frequency for RA2 family
pub const MAX_ICLK_FREQ: Hertz = Hertz(48_000_000);

/// Maximum PCLKB frequency
pub const MAX_PCLKB_FREQ: Hertz = Hertz(32_000_000);

/// Maximum PCLKB frequency
pub const MAX_PCLKD_FREQ: Hertz = Hertz(64_000_000);


/// Clock configuration for RA2 family
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// Clock source selection
    pub source: ClockSource,
    
    /// Main oscillator configuration (if MOSC is used)
    pub main_osc: Option<MainOscConfig>,
    
    /// System clock (ICLK) divider
    pub iclk_div: ClockDiv,
    
    /// Peripheral clock B (PCLKB) divider
    pub pclkb_div: ClockDiv,
    
    /// Peripheral clock D (PCLKD) divider
    pub pclkd_div: ClockDiv,

    /// Flash clock (FCLK) divider
    pub fclk_div: ClockDiv,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source: ClockSource::Hoco,
            main_osc: None,
            iclk_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div2,
            pclkd_div: ClockDiv::Div1,
            fclk_div: ClockDiv::Div2,
        }
    }
}

impl Config {
    /// Create configuration using HOCO (frequency determined by Option Bytes)
    pub const fn hoco() -> Self {
        Self {
            source: ClockSource::Hoco,
            main_osc: None,
            iclk_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div2,
            pclkd_div: ClockDiv::Div1,
            fclk_div: ClockDiv::Div2,
        }
    }
    
    /// Create configuration using MOSC
    pub const fn mosc(main_osc: MainOscConfig) -> Self {
        Self {
            source: ClockSource::MainOsc,
            main_osc: Some(main_osc),
            iclk_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div2,
            pclkd_div: ClockDiv::Div1,
            fclk_div: ClockDiv::Div2,
        }
    }
    
    /// Set ICLK divider
    pub const fn iclk_div(mut self, div: ClockDiv) -> Self {
        self.iclk_div = div;
        self
    }
    
    /// Set FCLK divider
    pub const fn fclk_div(mut self, div: ClockDiv) -> Self {
        self.fclk_div = div;
        self
    }
    
    /// Set all peripheral clock dividers at once
    pub const fn pclk_divs(
        mut self,
        pclkb: ClockDiv,
        pclkd: ClockDiv,
    ) -> Self {
        self.pclkb_div = pclkb;
        self.pclkd_div = pclkd;
        self
    }
}

/// Initialize clocks with the given configuration
///
/// # Safety
/// This function should only be called once during system initialization.
pub(crate) fn init(config: Config) -> Clocks {
    let sysc = sysc();
    
    // Calculate source frequency
    let source_freq = match config.source {
        ClockSource::Hoco => {
             // For RA2, HOCO frequency is set in Option Bytes (OFS1 at 0x0000_0408)
             // Bits [14:12] determine frequency
             // 000: 24MHz, 010: 32MHz, 100: 48MHz, 101: 64MHz
             let ofs1 = unsafe { *(0x0000_0408 as *const u32) };
             match (ofs1 >> 12) & 0b111 {
                 0b000 => Hertz(24_000_000),
                 0b010 => Hertz(32_000_000),
                 0b100 => Hertz(48_000_000),
                 0b101 => Hertz(64_000_000),
                 _ => Hertz(48_000_000), // Default safe fallback
             }
        },
        ClockSource::Moco => MOCO_FREQ,
        ClockSource::Loco => LOCO_FREQ,
        ClockSource::MainOsc => {
            config.main_osc.expect("MainOsc config required").freq
        }
        ClockSource::SubOsc => Hertz(32_768),
        ClockSource::Pll => panic!("PLL not supported on RA2"),
    };
    
    // Calculate output frequencies
    let iclk = Hertz(source_freq.0 / config.iclk_div.divisor());
    let pclkb = Hertz(source_freq.0 / config.pclkb_div.divisor());
    let pclkd = Hertz(source_freq.0 / config.pclkd_div.divisor());
    let fclk = Hertz(source_freq.0 / config.fclk_div.divisor());
    
    debug!("Clock frequencies: ICLK={} MHz, PCLKB={} MHz, PCLKD={} MHz",
        iclk.0 / 1_000_000,
        pclkb.0 / 1_000_000,
        pclkd.0 / 1_000_000,
    );

    // Validate frequencies
    assert!(iclk.0 <= MAX_ICLK_FREQ.0, "ICLK exceeds maximum frequency");
    assert!(pclkb.0 <= MAX_PCLKB_FREQ.0, "PCLKB exceeds maximum frequency");
    assert!(pclkd.0 <= MAX_PCLKD_FREQ.0, "PCLKD exceeds maximum frequency");
    
    // Unlock register protection for clock configuration
    sysc.prcr().modify(|w| {
        w.set_prkey(0xA5);  // Write key
        w.set_prc0(true);   // Enable writing to clock registers
        w.set_prc1(true);   // Enable writing to clock registers (some might need PRC1)
    });

    // Configure SCKDIVCR (System Clock Division Control Register)
    // RA2E1: Bits 24-26: FCK, 28-30: ICK, 8-10: PCKB, 0-2: PCKD.
    sysc.sckdivcr().modify(|w| {
        w.set_pckd((config.pclkd_div as u8).into());
        // w.set_pckc(...) // No PCLKC on RA2
        w.set_pckb((config.pclkb_div as u8).into());
        // w.set_pcka(...) // No PCLKA on RA2
        w.set_ick((config.iclk_div as u8).into());
        w.set_fck((config.fclk_div as u8).into());
    });
    
    // Configure SCKSCR (System Clock Source Control Register)
    let cksel: u8 = match config.source {
        ClockSource::Hoco => 0b000,
        ClockSource::Moco => 0b001,
        ClockSource::Loco => 0b010,
        ClockSource::MainOsc => 0b011,
        ClockSource::SubOsc => 0b100,
        ClockSource::Pll => panic!("PLL not supported on RA2"),
    };
    
    // TODO: Add proper clock switching sequence:
    // 1. Enable target oscillator/PLL
    // 2. Wait for stabilization
    // 3. Switch clock source
    // 4. Disable unused oscillators (optional, for power saving)
    
    sysc.sckscr().modify(|w| {
        w.set_cksel(cksel.into());
    });
    
    // Lock register protection
    sysc.prcr().modify(|w| {
        w.set_prkey(0xA5);
        w.set_prc0(false);
        w.set_prc1(false);
    });
    
    Clocks {
        iclk,
        pclkb,
        pclkd,
        fclk: Some(fclk),
        pclka: None,
        pclkc: None,
        bclk: None,
    }
}
