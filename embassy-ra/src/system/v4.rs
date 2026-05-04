//! Clock configuration for sysc_v4 chips: RA4M2/M3/E1/E2/T1, RA6M4/M5, RA6E1/E2, RA6T2/T3
//!
//! sysc_v4 characteristics:
//! - Max ICLK: 200 MHz (RA6E2/M4/M5), 100 MHz (RA4 sub-family)
//! - Clock sources: HOCO (16/18/20 MHz), MOCO (8 MHz), LOCO, MOSC, SOSC, PLL
//! - PLL: 16-bit PLLCCR (PLIDIV + PLSRCSEL + PLLMUL), NO hardware PLODIV
//! - Dividers: ICLK, PCLKA, PCLKB, PCLKC, PCLKD, FCLK, BCLK

use cortex_m::asm;

use super::{Clocks, ClockDiv, ClockSource, Hertz, HocoFreq, MainOscConfig, PllSource, PllInputDiv, MOCO_FREQ, LOCO_FREQ};
use super::common::sysc;

// SYSC registers not yet exposed by ra-metapac sysc_v4 PAC:
//   PLLCCR  (16-bit) at SYSC_BASE + 0x28 — PLLMUL[13:8] | PLSRCSEL[4] | PLIDIV[1:0]
//   PLLCR   (8-bit)  at SYSC_BASE + 0x2A — PLLSTP[0]
//   OSCSF bit 5 (PLLSF) — PLL stabilization flag
unsafe fn pllccr_ptr() -> *mut u16 {
    (ra_metapac::SYSC.as_ptr() as usize + 0x28) as *mut u16
}
unsafe fn pllcr_ptr() -> *mut u8 {
    (ra_metapac::SYSC.as_ptr() as usize + 0x2A) as *mut u8
}
unsafe fn oscsf_raw_ptr() -> *const u8 {
    ra_metapac::SYSC.oscsf().as_ptr() as *const u8
}
const OSCSF_PLLSF: u8 = 1 << 5;

// FCACHE.FLWT at FCACHE_BASE + 0x1C (not in PAC)
// FLWT[2:0]: 0=≤50 MHz, 1=≤100 MHz, 2=≤150 MHz, 3=>150 MHz
const FCACHE_FLWT: *mut u8 = 0x4001_C11C as *mut u8;

/// PLL configuration for sysc_v4 chips.
///
/// sysc_v4 has NO hardware PLODIV register — the VCO output feeds SCKDIVCR
/// directly. Set `iclk_div` / `pclk*_div` in `Config` to achieve the desired
/// clock ratios from the raw VCO.
#[derive(Clone, Copy, Debug)]
pub struct PllV4Config {
    pub source: PllSource,
    pub input_div: PllInputDiv,
    pub mul: super::PllMul,
}

pub const MAX_ICLK_FREQ: Hertz = Hertz(200_000_000);
pub const MAX_PCLKA_FREQ: Hertz = Hertz(100_000_000);
pub const MAX_PCLKB_FREQ: Hertz = Hertz(50_000_000);

/// Clock configuration for sysc_v4 chips (RA6E2, RA6M4/M5, RA4M2+, …)
#[derive(Clone, Copy)]
pub struct Config {
    pub source: ClockSource,
    pub hoco_freq: HocoFreq,
    pub main_osc: Option<MainOscConfig>,
    /// PLL config. No output divider — set iclk_div/pclk*_div to divide the VCO.
    pub pll: Option<PllV4Config>,
    pub iclk_div: ClockDiv,
    pub pclka_div: ClockDiv,
    pub pclkb_div: ClockDiv,
    pub pclkc_div: ClockDiv,
    pub pclkd_div: ClockDiv,
    pub fclk_div: ClockDiv,
    /// BCLK divider (field present for completeness; RA6E2 has no external bus output).
    pub bclk_div: ClockDiv,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source: ClockSource::Hoco,
            hoco_freq: crate::EMBASSY_RA_HOCO_FREQ,
            main_osc: None,
            pll: None,
            iclk_div: ClockDiv::Div1,
            pclka_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div2,
            pclkc_div: ClockDiv::Div2,
            pclkd_div: ClockDiv::Div2,
            fclk_div: ClockDiv::Div2,
            bclk_div: ClockDiv::Div1,
        }
    }
}

pub(crate) fn init(config: Config) -> Clocks {
    let sysc = sysc();

    // ── Frequency calculation ────────────────────────────────────────────────
    let source_freq = match config.source {
        ClockSource::Hoco    => Hertz(config.hoco_freq.to_hz()),
        ClockSource::Moco    => MOCO_FREQ,
        ClockSource::Loco    => LOCO_FREQ,
        ClockSource::SubOsc  => Hertz(32_768),
        ClockSource::MainOsc => config.main_osc.expect("MainOsc config required").freq,
        ClockSource::Pll => {
            let pll = config.pll.expect("PLL config required");
            let pll_input = match pll.source {
                PllSource::MainOsc => config.main_osc.expect("MainOsc required for PLL").freq,
                PllSource::Hoco    => Hertz(config.hoco_freq.to_hz()),
            };
            let input_div = match pll.input_div {
                PllInputDiv::Div1 => 1,
                PllInputDiv::Div2 => 2,
                PllInputDiv::Div3 => 3,
                PllInputDiv::Div4 => 4,
            };
            // VCO = (input / PLIDIV) × PLLMUL. Use ×2 intermediate to stay integer.
            // No PLODIV on sysc_v4 — the raw VCO feeds SCKDIVCR directly.
            let vco_x2 = pll_input.0 / input_div * pll.mul.value_x2();
            Hertz(vco_x2 / 2)
        }
    };

    let iclk  = Hertz(source_freq.0 / config.iclk_div.divisor());
    let pclka = Hertz(source_freq.0 / config.pclka_div.divisor());
    let pclkb = Hertz(source_freq.0 / config.pclkb_div.divisor());
    let pclkc = Hertz(source_freq.0 / config.pclkc_div.divisor());
    let pclkd = Hertz(source_freq.0 / config.pclkd_div.divisor());
    let fclk  = Hertz(source_freq.0 / config.fclk_div.divisor());
    let bclk  = Hertz(source_freq.0 / config.bclk_div.divisor());

    debug!("ICLK={=u32} PCLKA={=u32} PCLKB={=u32} PCLKC={=u32} PCLKD={=u32} FCLK={=u32} BCLK={=u32} (MHz)",
        iclk.0  / 1_000_000, pclka.0 / 1_000_000, pclkb.0 / 1_000_000,
        pclkc.0 / 1_000_000, pclkd.0 / 1_000_000, fclk.0  / 1_000_000,
        bclk.0  / 1_000_000,
    );

    assert!(iclk.0  <= MAX_ICLK_FREQ.0,  "ICLK exceeds maximum frequency");
    assert!(pclka.0 <= MAX_PCLKA_FREQ.0, "PCLKA exceeds maximum frequency");
    assert!(pclkb.0 <= MAX_PCLKB_FREQ.0, "PCLKB exceeds maximum frequency");

    // ── Unlock clock-register write protection ───────────────────────────────
    sysc.prcr().modify(|w| { w.set_prkey(0xA5); w.set_prc0(true); });
    debug!("prcr unlocked");

    // ── Flash wait states (must be set before raising ICLK) ─────────────────
    // RA6E2: FCACHE.FLWT (0x4001C11C). MEMWAIT is not in sysc_v4.
    let flwt: u8 = if iclk.0 <= 50_000_000 { 0 }
                   else if iclk.0 <= 100_000_000 { 1 }
                   else if iclk.0 <= 150_000_000 { 2 }
                   else { 3 };
    unsafe { FCACHE_FLWT.write_volatile(flwt); }
    asm::dsb();
    asm::isb();
    debug!("flwt={=u8}", flwt);

    // ── HOCO ─────────────────────────────────────────────────────────────────
    if matches!(config.source, ClockSource::Hoco) ||
       (matches!(config.source, ClockSource::Pll) &&
        matches!(config.pll.unwrap().source, PllSource::Hoco))
    {
        let hcfrq: u8 = match config.hoco_freq {
            HocoFreq::Mhz16 => 0b00,
            HocoFreq::Mhz18 => 0b01,
            HocoFreq::Mhz20 => 0b10,
            _ => panic!("unsupported HOCO frequency for sysc_v4"),
        };
        sysc.hococr().modify(|w| w.set_hcstp(false));
        // HOCOCR2 is not modelled in the PAC; write it via pointer offset.
        unsafe { (sysc.hococr().as_ptr() as *mut u8).add(1).write_volatile(hcfrq); }
        while !sysc.oscsf().read().hocosf() {}
        debug!("hoco stable");
    }

    // ── Main oscillator ───────────────────────────────────────────────────────
    if matches!(config.source, ClockSource::MainOsc | ClockSource::Pll) {
        let mosc = config.main_osc.expect("MainOsc config required for MainOsc/PLL source");

        // MOMCR.MODRV[4:5] — 2-bit drive capability (sysc_v4 SVD).
        //   00 = 20–24 MHz, 01 = 16–20 MHz, 10 = 8–16 MHz, 11 = ≤8 MHz
        let modrv = if      mosc.freq.0 >  20_000_000 { 0b00 }
                    else if mosc.freq.0 >  16_000_000 { 0b01 }
                    else if mosc.freq.0 >   8_000_000 { 0b10 }
                    else                              { 0b11 };
        sysc.momcr().modify(|w| {
            w.set_modrv(ra_metapac::sysc::vals::Modrv::from_bits(modrv));
            w.set_mosel(matches!(mosc.mode, super::MainOscMode::ExternalClock));
        });

        // Stabilisation wait: 8192 µs (safe for any crystal)
        sysc.moscwtcr().modify(|w| w.set_msts(ra_metapac::sysc::vals::Msts::from_bits(0x07)));

        debug!("starting mosc");
        sysc.mosccr().modify(|w| w.set_mostp(false));
        while !sysc.oscsf().read().moscsf() {}
        debug!("mosc stable");
    }

    // ── PLL ───────────────────────────────────────────────────────────────────
    // sysc_v4 PLLCCR (16-bit, offset 0x028):
    //   [1:0]  PLIDIV  — input divider (00=/1, 01=/2, 10=/3)
    //   [4]    PLSRCSEL — PLL source (0=MOSC, 1=HOCO)
    //   [13:8] PLLMUL  — VCO multiplier
    // No PLODIV field — SCKDIVCR dividers apply directly to the raw VCO output.
    if matches!(config.source, ClockSource::Pll) {
        let pll = config.pll.unwrap();

        let plidiv: u16 = match pll.input_div {
            PllInputDiv::Div1 => 0b00,
            PllInputDiv::Div2 => 0b01,
            PllInputDiv::Div3 => 0b10,
            PllInputDiv::Div4 => panic!("PLIDIV /4 not valid for sysc_v4 PLL"),
        };
        let plsrcsel: u16 = match pll.source {
            PllSource::MainOsc => 0,
            PllSource::Hoco    => 1,
        };
        let pllmul: u16 = pll.mul as u16;
        let pllccr: u16 = plidiv | (plsrcsel << 4) | (pllmul << 8);
        debug!("pllccr=0x{=u16:04x}", pllccr);

        unsafe {
            pllcr_ptr().write_volatile(0x01); // stop PLL before reconfiguring
            debug!("pll stopped");
            pllccr_ptr().write_volatile(pllccr);
            debug!("pllccr written");
            pllcr_ptr().write_volatile(0x00); // start PLL
            debug!("pll started, waiting for lock...");
            while oscsf_raw_ptr().read_volatile() & OSCSF_PLLSF == 0 {}
            debug!("pll locked");
        }
    }

    // ── Clock dividers ────────────────────────────────────────────────────────
    debug!("writing sckdivcr");
    sysc.sckdivcr().modify(|w| {
        w.set_pckd((config.pclkd_div as u8).into());
        w.set_pckc((config.pclkc_div as u8).into());
        w.set_pckb((config.pclkb_div as u8).into());
        w.set_pcka((config.pclka_div as u8).into());
        w.set_ick( (config.iclk_div  as u8).into());
        w.set_fck( (config.fclk_div  as u8).into());
    });
    debug!("sckdivcr written");

    // ── Switch clock source ───────────────────────────────────────────────────
    let cksel: u8 = match config.source {
        ClockSource::Hoco    => 0b000,
        ClockSource::Moco    => 0b001,
        ClockSource::Loco    => 0b010,
        ClockSource::MainOsc => 0b011,
        ClockSource::SubOsc  => 0b100,
        ClockSource::Pll     => 0b101,
    };
    debug!("switching to cksel={=u8}", cksel);
    sysc.sckscr().modify(|w| w.set_cksel(cksel.into()));
    asm::dsb();
    asm::isb();
    // RA6E2 user manual: wait ≥ 3 µs after SCKSCR change.
    asm::delay(1000);
    debug!("clock switched");

    // ── Lock register protection ──────────────────────────────────────────────
    sysc.prcr().modify(|w| { w.set_prkey(0xA5); w.set_prc0(false); });
    debug!("prcr locked");

    Clocks { iclk, pclkb, pclkd, fclk: Some(fclk), pclka: Some(pclka), pclkc: Some(pclkc), bclk: Some(bclk) }
}
