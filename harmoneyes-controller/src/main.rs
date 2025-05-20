#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

mod bat;
mod twi;
mod usb;
mod uwb;
mod softdevice;
mod coord;
mod ws;
mod ble;
mod rng;

use defmt_rtt as _;
use panic_probe as _;
use embassy_nrf as _;

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::twim::Error;
use embassy_time::{Duration, Ticker};

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize Embassy
    info!("Initializing Embassy");
    let p = embassy_nrf::init(embassy_config());

    // Initialize the softdevice
    info!("Initializing softdevice");
    softdevice::initialize(&spawner).await;

    // Spawn the ultra-wide band task
    info!("Spawning ultra-wide band task");
    spawner.must_spawn(uwb::task(
        p.SPI3,
        p.P0_14, 
        p.P0_13, 
        p.P0_15, 
        p.P0_24, 
        p.P0_25,
        p.P1_08,
        p.P0_07
    ));

    // Spawn the coordination task
    spawner.must_spawn(coord::task());

    // Initialize the two-wire interface driver
    info!("Initializing two-wire interface");
    twi::initialize(
        p.TWISPI0,
        p.P0_11, 
        p.P0_12
    ).await;

    // Spawn the battery monitor task
    info!("Spawning battery monitor task");
    spawner.must_spawn(bat::task(
        p.SAADC, 
        p.P0_29
    ));

    // Spawn the USB task
    info!("Spawning USB task");
    spawner.must_spawn(usb::task(
        p.USBD
    ));

    info!("Spawning Neopixel task");
    spawner.must_spawn(ws::task(
        p.PWM0,
        p.P0_16
    ));

    // A ticker that every 5 seconds will trigger the front motor for 50 miliseconds

    let mut ticker = Ticker::every(Duration::from_secs(5));

    let mut command: [u8; 9] = [0; 9];
    command[1..9].copy_from_slice(&u64::to_le_bytes(50));

    command[0] = 0x10; // Code for front

    loop {
        ticker.next().await;

        match twi::DRIVER.lock().await
            .get_mut().expect("Two-wire interface driver is not initialized") // SAFETY: We called twi::initialize above
            .write(harmoneyes_core::constants::cuff::I2C_ADDRESS as u8, &command).await {
                Err(Error::TxBufferTooLong) => { info!("Transmit buffer was too long") },
                Err(Error::RxBufferTooLong) => { info!("Receive buffer was too long") },
                Err(Error::Transmit) => { info!("Data transmission failed") },
                Err(Error::Receive) => { info!("Data reception failed") },
                Err(Error::BufferNotInRAM) => { info!("Buffer not in RAM") },
                Err(Error::AddressNack) => { info!("Address did not acknowledge") },
                Err(Error::DataNack) => { info!("No acknowledge after data sent") },
                Err(Error::Overrun) => { info!("Overrun") },
                Err(Error::Timeout) => { info!("Connection timed out") },
                _ => {}
            }
    }
}

fn embassy_config() -> embassy_nrf::config::Config {
    use embassy_nrf::{interrupt, config::Config};

    let mut config = Config::default();

    config.gpiote_interrupt_priority = interrupt::Priority::P2;
    config.time_interrupt_priority = interrupt::Priority::P2;

    config
}