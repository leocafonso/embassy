#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_ra as hal;

use hal::gpio::{Output, Level};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());
    info!("Hello World!");

    // Create LED output on P104
    //let mut led = Output::new(p.P207, Level::Low);
    let mut led = Output::new(p.P400, Level::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_micros(500).await;

        info!("low");
        led.set_low();
        Timer::after_micros(500).await;
    }
}
