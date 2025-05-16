use core::cell::OnceCell;

use defmt::warn;
use embassy_nrf::{bind_interrupts, interrupt::{self, InterruptExt}, peripherals::{P0_11, P0_12, TWISPI0}, twim::{self, Twim}};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

pub static DRIVER: Mutex<CriticalSectionRawMutex, OnceCell<Twim<'static, TWISPI0>>> = Mutex::new(OnceCell::new());

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<TWISPI0>;
});

pub async fn initialize(
    twi: TWISPI0,
    scl: P0_11,
    sda: P0_12
) {
    // Hark, weary traveler! Take caution of a nearby softdevice. You don't want to anger it.
    interrupt::TWISPI0.set_priority(interrupt::Priority::P3);

    if let Err(_) = DRIVER.lock().await.set(Twim::new(twi, Irqs, sda, scl, controller_config())) {
        warn!("Called twi::initialize when the two-wire interface was already initialized");
    }
}

fn controller_config() -> twim::Config {
    let config = twim::Config::default();

    config
}