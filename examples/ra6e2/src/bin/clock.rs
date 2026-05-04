#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_ra as hal;

use hal::gpio::{Output, Level};
use hal::system::{
    Config, ClockSource, MainOscConfig, MainOscMode, Hertz,
    PllV4Config, PllSource, PllInputDiv, PllMul,
    ClockDiv,
};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // EK-RA6E2: 24 MHz crystal on X1.
    // MOSC(24 MHz) / PLIDIV(3) = 8 MHz → PLL×25 = 200 MHz VCO → iclk_div(Div1) → ICLK=200 MHz
    let clk_config = Config {
        source: ClockSource::Pll,
        main_osc: Some(MainOscConfig { freq: Hertz::mhz(24), mode: MainOscMode::Crystal }),
        pll: Some(PllV4Config {
            source: PllSource::MainOsc,
            input_div: PllInputDiv::Div3,
            mul: PllMul::Mul25_0,
        }),
        iclk_div: ClockDiv::Div1,
        pclka_div: ClockDiv::Div2,
        pclkb_div: ClockDiv::Div4,
        pclkc_div: ClockDiv::Div4,
        pclkd_div: ClockDiv::Div2,
        ..Default::default()
    };
    // ICLK=200 MHz, PCLKA=100 MHz, PCLKB=50 MHz, PCLKC=50 MHz, PCLKD=100 MHz

    let p = hal::init(hal::Config { system: clk_config });
    info!("Hello World! Running at 200 MHz via PLL");

    // Create LED output on P104
    let mut led = Output::new(p.P207, Level::Low);
    
    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(1000).await;

        info!("low");
        led.set_low();
        Timer::after_millis(1000).await;
    }
}


