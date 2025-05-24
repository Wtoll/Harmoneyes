//! Hi, because I know I didn't find a lot of great documentation on how to use this library, in case you're using this for reference
//! please be aware that while this code has been tested and does work on actual hardware, it is not pretty. This device is meant as a
//! proof of concept to demonstrate the technology and a lot of work would need to be done for it to be in a state where it could
//! actually be deployed, but feel free to use this as a jumping off point.

use defmt::{debug, info};
use dw3000_ng::{hl::{RxQuality, SendTime}, time::Instant, Ready, SingleBufferReceiving, DW3000};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_nrf::{bind_interrupts, gpio::{Input, Level, Output, OutputDrive, Pull}, interrupt::{self, InterruptExt, Priority}, peripherals::{P0_07, P0_13, P0_14, P0_15, P0_24, P0_25, P1_08, SPI3}, spim::{self, Spim}};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex, signal::Signal};
use embassy_time::{Duration, Timer};

static POLL_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

pub static DISTANCES: Channel<CriticalSectionRawMutex, u64, 20> = Channel::new();

static RESET: Signal<CriticalSectionRawMutex, ()> = Signal::new();

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<SPI3>;
});

enum Mode {
    Driver,
    Listener
}

#[embassy_executor::task]
pub async fn task(
    mut spi: SPI3,
    mut sck: P0_14,
    mut mosi: P0_13,
    mut miso: P0_15,
    mut cs: P0_24,
    mut irq: P0_25,
    mut exton: P1_08,
    mut reset: P0_07,
) -> ! {
    // Hark, weary traveler! Take caution of a nearby softdevice. You don't want to anger it.
    interrupt::SPIM3.set_priority(Priority::P3);

    loop {
        // Start with the device off by holding the reset pin low.
        let mut reset = Output::new(&mut reset, Level::Low, OutputDrive::Standard0Disconnect1);

        let mut exton = Input::new(&mut exton, Pull::Down);
        let mut irq = Input::new(&mut irq, Pull::Down);

        let spi: Mutex<CriticalSectionRawMutex, _> = Mutex::new(Spim::new(&mut spi, Irqs, &mut sck, &mut miso, &mut mosi, spi_config()));
        let cs = Output::new(&mut cs, Level::High, OutputDrive::Standard);
        let device = SpiDevice::new(&spi, cs);

        let dwm = DW3000::new(device);

        // Make sure we really are pulling the reset pin low
        reset.set_low();
        
        // Wait at least 50ms for the reset
        Timer::after_millis(50).await;

        // Turn the device on.
        reset.set_high(); 

        // Wait for the device to power on
        info!("Waiting for DWM3000 to turn on");
        exton.wait_for_high().await;
        info!("DWM3000 turned on!");

        // Wait for the SPIRDY interrupt indicating that we can start communicating over spi
        info!("Waiting for DWM3000 to move into IDLE_RC state");
        match embassy_futures::select::select(irq.wait_for_high(), Timer::after_millis(50)).await {
            embassy_futures::select::Either::First(_) => info!("Received SPIRDY Interrupt"),
            embassy_futures::select::Either::Second(_) => info!("SPIRDY Interrupt took too long. Moving forward anyways."),
        };
        
        // Initialize the DWM3000
        info!("Initializing the DWM3000");
        let mut dwm = dwm
            .init().await.expect("Failed to initialize DWM3000")
            .config(dw_config(), embassy_time::Delay).await.expect("Failed to configure DWM3000");

        let (_pan, address) = dwm.get_address().await.expect("Failed to get DWM3000 Address");
        info!("DWM3000 Address is {}", address);

        // Turn off the SPIRDY interrupt (really this is just to be safe)
        dwm.disable_interrupts().await.expect("Failed to disable all interrupts");

        let mut mode = Mode::Listener;

        let mut counter = 0;

        let mut last_tx: Option<Instant> = None;
        let mut last_rx: Option<Instant> = None;

        loop {
            // Unfortunately some errors with the DW3000 require an entire chip reset in order to go back to functioning properly
            if RESET.signaled() {
                info!("An unrecoverable error occured in the DWM3000... Restarting");
                RESET.reset();
                break;
            }

            match mode {
                Mode::Driver => {
                    // Send one message to get things started
                    mode = Mode::Listener;

                    let (returned_dw, res) = must_transmit(dwm, &mut irq, &[]).await;
                    dwm = returned_dw;

                    continue;
                },
                Mode::Listener => {

                    // Try to receive a packet
                    let (returned_dw, res) = must_receive(dwm, &mut irq, Duration::from_millis(50)).await; // From testing with a basic ping pong 10ms timeout with no artificial turn-around delay achieved about a 0.15% timeout to response rate which seems pretty good.
                    dwm = returned_dw;

                    match res {
                        Some((buf, len, rx_inst, qual)) => {

                            if last_tx.is_some() && rx_inst.value() > last_tx.unwrap().value() && len > 12 {
                                let last_tx_rx_interval: u64 = rx_inst.value() - last_tx.unwrap().value();

                                let mut buf_frame = [0u8; 8];
                                for (i, byte) in buf[len-12..len-4].iter().enumerate() {
                                    if i < buf_frame.len() {
                                        buf_frame[7-i] = *byte;
                                    }
                                }

                                let foreign_rx_tx_interval = u64::from_ne_bytes(buf_frame);
                                if foreign_rx_tx_interval != 0 && last_tx_rx_interval > foreign_rx_tx_interval {
                                    DISTANCES.send((last_tx_rx_interval - foreign_rx_tx_interval) / 2).await;
                                }
                            }

                            Timer::after(Duration::from_millis(3)).await;

                            let mut payload = [0u8; 8];

                            if last_rx.is_some() && last_tx.is_some() && last_tx.unwrap().value() > last_rx.unwrap().value() {
                                let last_rx_tx_interval: u64 = last_tx.unwrap().value() - last_rx.unwrap().value();

                                for (i, byte) in last_rx_tx_interval.to_ne_bytes().into_iter().enumerate() {
                                    payload[7-i] = byte;
                                }
                            }

                            // We got a packet! Time to respond.
                            let (returned_dw, tx_inst) = must_transmit(dwm, &mut irq, &payload).await;
                            dwm = returned_dw;

                            if counter < 100 {
                                counter += 1;
                            } else {
                                info!("100 packets exchanged");
                                counter = 0;
                            }

                            last_tx = Some(tx_inst);
                            last_rx = Some(rx_inst);
                        },
                        None => {
                            // If the timeout was triggered swap to driver mode.
                            mode = Mode::Driver;
                            continue;
                        },
                    }
                },
            }
        }
    }
}

/// This function will just loop the transmit function until it doesn't return an error. Transmission errors are pretty rare, so
/// this is usually sufficient to do the job, as any errors you do encounter are probably a much more serious issue.
async fn must_transmit<'a, T>(mut dw: DW3000<T, Ready>, irq_pin: &mut Input<'a>, data: &[u8]) -> (DW3000<T, Ready>, Instant)
where
    T: embedded_hal_async::spi::SpiDevice,
    <T as embedded_hal_async::spi::ErrorType>::Error: defmt::Format
{
    let mut counter = 0;

    loop {
        if counter < 10 {
            counter += 1;
        } else {
            RESET.signal(());
            return (dw, Instant::new(0).unwrap())
        }

        let (returned_dw, res) = transmit(dw, irq_pin.wait_for_high(), data).await;
        dw = returned_dw;

        match res {
            Some(inst) => return (dw, inst),
            None => {
                continue;
            },
        }
    }
}

async fn transmit<T, I, O>(mut dw: DW3000<T, Ready>, irq: I, data: &[u8]) -> (DW3000<T, Ready>, Option<Instant>)
where
    T: embedded_hal_async::spi::SpiDevice,
    I: Future<Output = O>,
    <T as embedded_hal_async::spi::ErrorType>::Error: defmt::Format
{
    // Disable all interrupts just in case someone was lazy
    dw.disable_interrupts().await.expect("Failed to disable all interrupts");
    // Enable transmitting interrupts on the irq pin.
    dw.enable_tx_interrupts().await.expect("Failed to enable transmitter interrupts");
    // Put the device into transmitting mode.
    let mut tx = dw.send(data, SendTime::Now, dw_config()).await.expect("Failed to enter transmitting mode");

    let response = match tx.s_wait().await {
        // If the transmitter immediately returns a value then return that
        Ok(inner) => Some(inner),
        // If the transmitter immediately returns an error then return nothing
        Err(nb::Error::Other(e)) => {
            info!("Error in transmitting: {}", e);
            None
        },
        // If the transmitter needs to wait...
        Err(nb::Error::WouldBlock) => {
            // ...then wait for the interrupt...
            irq.await;
            // ...then if the transmitter returns a value return it.
            tx.s_wait().await.ok()
        },
    };

    // Take the device out of transmitting mode.
    let mut dw = tx.finish_sending().await.expect("Failed to leave transmitting mode");
    // Disable all interrupts.
    dw.disable_interrupts().await.expect("Failed to disable transmitter interrupts");

    (dw, response)
}

async fn try_receive<T, I>(rx: &mut DW3000<T, SingleBufferReceiving>, irq: I) -> Result<([u8; 128], usize, Instant, RxQuality), dw3000_ng::Error<T>>
where
    T: embedded_hal_async::spi::SpiDevice,
    I: Future<Output = ()>,
    <T as embedded_hal_async::spi::ErrorType>::Error: defmt::Format
{
    // Allocate a buffer for the received message.
    let mut buf = [0u8; 128];

    match rx.r_wait_buf(&mut buf).await {
        // If the receiver immediately returns a value then return that
        Ok(inner) => Ok((buf, inner.0, inner.1, inner.2)),
        // If the receiver immediately returns an error then return nothing
        Err(nb::Error::Other(e)) => Err(e),
        // If the receiver needs to wait...
        Err(nb::Error::WouldBlock) => {
            // ...then wait for the interrupt...
            irq.await;
            // ...then if the receiver returns a value return it.
            match rx.r_wait_buf(&mut buf).await {
                Ok(inner) => Ok((buf, inner.0, inner.1, inner.2)),
                Err(nb::Error::Other(e)) => Err(e),
                Err(nb::Error::WouldBlock) => unreachable!()
            }
        }
    }
}

/// This function will loop until it successfully receives a packet. In order to do so, however, it must take ownership of the device, so
/// in the event that there were no packets to be receieved it would just destroy the device. To solve for this you must pass it a timeout
/// duration after which point it will return the device to you. The `Option` is only ever a `None` in the event that this occurs.
async fn must_receive<'a, T>(mut dwm: DW3000<T, Ready>, irq_pin: &mut Input<'a>, time_out: Duration) -> (DW3000<T, Ready>, Option<([u8; 128], usize, Instant, RxQuality)>)
where
    T: embedded_hal_async::spi::SpiDevice,
    <T as embedded_hal_async::spi::ErrorType>::Error: defmt::Format
{
    use embassy_futures::select::{select, Either};

    let after = embassy_time::Instant::now() + time_out;

    loop {
        // Enable receiver interrupts
        dwm.enable_rx_interrupts().await.expect("Failed to enable receiver interrupts");
        // Enter receiving mode
        let mut rx = dwm.receive(dw_config()).await.expect("Failed to enter receiving mode");

        let res = select(try_receive(&mut rx, irq_pin.wait_for_high()), Timer::at(after)).await;
        // Put the device back where we found it
        dwm = rx.finish_receiving().await.expect("Failed to leave receiving mode");
        // Disable the interrupts we enabled
        dwm.disable_interrupts().await.expect("Failed to disable receiver interrupts");

        match res {
            Either::First(Ok(inner)) => return (dwm, Some(inner)),
            Either::First(Err(e)) => {
                info!("Error in receiving: {}", e);
                RESET.signal(());
                return (dwm, None);
            },
            Either::Second(()) => {
                info!("Receiver timed out");
                return (dwm, None);
            }
        }
    }
}

fn dw_config() -> dw3000_ng::Config {
    let config = dw3000_ng::Config::default();

    config
}

fn spi_config() -> spim::Config {
    let config = spim::Config::default();

    config
}