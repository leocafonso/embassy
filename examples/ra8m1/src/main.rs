#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_ra as hal;
use {defmt_rtt as _, panic_probe as _};
use hal::gpio::{Output, Level};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.P600, Level::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(100).await;

        info!("low");
        led.set_low();
        Timer::after_millis(100).await;
    }
}
