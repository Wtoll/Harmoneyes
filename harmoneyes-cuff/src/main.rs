#![no_std]
#![no_main]

mod haptics;
mod twi;
mod usb;
mod ws;

use panic_probe as _;
use defmt_rtt as _;

use log::info;
use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize Embassy
    info!("Initializing Embassy");
    let p = embassy_rp::init(embassy_config());

    // Spawn the two-wire interface task
    info!("Spawning two-wire interface task");
    spawner.must_spawn(twi::task(
        p.I2C1,
        p.PIN_23,
        p.PIN_22
    ));

    // Spawn the haptics task
    info!("Spawning haptics task");
    spawner.must_spawn(haptics::task(
        p.PIN_3,
        p.PIN_4,
        p.PIN_5,
        p.PIN_6
    ));

    // Spawn the USB task
    info!("Spawning USB task");
    spawner.must_spawn(usb::task(p.USB));

    info!("Spawning Neopixel task");
    spawner.must_spawn(ws::task(
        p.PIO0,
        p.DMA_CH0,
        p.PIN_12,
        p.PIN_11
    ));
}

fn embassy_config() -> embassy_rp::config::Config {
    Default::default()
}