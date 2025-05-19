
use defmt::info;
use dw3000_ng::{hl::RxQuality, time::Instant, SingleBufferReceiving, DW3000};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_futures::select::{select, Either};
use embassy_nrf::{bind_interrupts, gpio::{Input, Level, Output, OutputDrive, Pull}, interrupt::{self, InterruptExt, Priority}, peripherals::{P0_07, P0_13, P0_14, P0_15, P0_24, P0_25, P1_08, SPI3}, spim::{self, Spim}};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::Timer;

static POLL_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<SPI3>;
});

#[embassy_executor::task]
pub async fn task(
    spi: SPI3,
    sck: P0_14,
    mosi: P0_13,
    miso: P0_15,
    cs: P0_24,
    irq: P0_25,
    exton: P1_08,
    reset: P0_07,
) -> ! {
    // Hark, weary traveler! Take caution of a nearby softdevice. You don't want to anger it.
    interrupt::SPIM3.set_priority(Priority::P3);

    // Start with the device off by holding the reset pin low.
    let mut reset = Output::new(reset, Level::Low, OutputDrive::Standard0Disconnect1);

    let mut exton = Input::new(exton, Pull::Down);
    let mut irq = Input::new(irq, Pull::Down);

    let spi: Mutex<CriticalSectionRawMutex, _> = Mutex::new(Spim::new(spi, Irqs, sck, miso, mosi, spi_config()));
    let cs = Output::new(cs, Level::High, OutputDrive::Standard);
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
    match select(irq.wait_for_high(), Timer::after_millis(50)).await {
        Either::First(_) => info!("Received SPIRDY Interrupt"),
        Either::Second(_) => info!("SPIRDY Interrupt took too long. Moving forward anyways."),
    };
    
    // Initialize the DWM3000
    info!("Initializing the DWM3000");
    let mut dwm = dwm
        .init().await.expect("Failed to initialize DWM3000")
        .config(dw_config(), embassy_time::Delay).await.expect("Failed to configure DWM3000");

    let (_pan, address) = dwm.get_address().await.expect("Failed to get DWM3000 Address");
    info!("DWM3000 Address is {}", address);

    loop {
        dwm.enable_rx_interrupts().await.expect("Failed to enable rx interrupts");

        let mut rx = dwm.receive(dw_config()).await.expect("Failed to enter receiving mode");

        let (buf, len, _inst, _qual) = loop {
            info!("trying to receive a packet");

            let mut buf = [0u8; 128];
            match try_receive_packet(&mut rx, select(irq.wait_for_high(), Timer::after_secs(5)), &mut buf).await {
                Some(inner) => break (buf, inner.0, inner.1, inner.2),
                None => {
                    info!("Failed to receive packet... trying again");
                    continue
                }
            }
        };

        info!("Received packet! {}", buf[..len]);

        dwm = rx.finish_receiving().await.expect("Failed to leave receiving mode");
    }
}








async fn try_receive_packet<T, I>(dw: &mut DW3000<T, SingleBufferReceiving>, irq: I, buf: &mut [u8]) -> Option<(usize, Instant, RxQuality)>
where
    T: embedded_hal_async::spi::SpiDevice,
    I: Future<Output = Either<(), ()>>,
    <T as embedded_hal_async::spi::ErrorType>::Error: defmt::Format
{
    
    match dw.r_wait_buf(buf).await {
        Ok(inner) => Some(inner),
        Err(nb::Error::WouldBlock) => {
            irq.await;
            match dw.r_wait_buf(buf).await {
                Ok(inner) => Some(inner),
                Err(nb::Error::Other(e)) => {
                    info!("{}", e);
                    info!("There was an error trying to receive a packet");
                    None
                }
                _ => None
            }
        },
        Err(nb::Error::Other(e)) => {
            info!("{}", e);
            info!("There was an error trying to receive a packet");
            None
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