#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_ra as hal;
use hal::gpio::{Output, Level};
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".ofs0"]
#[no_mangle]
pub static OFS0: u32 = 0xFFFF_FFFF;

#[link_section = ".osis"]
#[no_mangle]
pub static ID_CODE: [u32; 4] = [0xFFFF_FFFF; 4];

#[link_section = ".ofs1_sec"]
#[no_mangle]
pub static OFS1: u32 = 0xFFFF_FDFF;

#[link_section = ".bps_sec"]
#[no_mangle]
pub static BSP0: u32 = 0xFFFF_FFFF;

#[link_section = ".pbps_sec"]
#[no_mangle]
pub static PBPS0: u32 = 0xFFFF_FFFF;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());
    info!("Hello World!");

    // Create LED output on P104
    let mut led = Output::new(p.P104, Level::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(100).await;

        info!("low");
        led.set_low();
        Timer::after_millis(100).await;
    }
}


