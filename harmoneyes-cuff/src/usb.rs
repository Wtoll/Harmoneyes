
use embassy_rp::{bind_interrupts, peripherals::USB, usb::{self, Driver}};
use embassy_usb::{class::cdc_acm::{CdcAcmClass, State}, driver::EndpointError};
use futures::future::join;
use log::info;
use static_cell::StaticCell;

static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

static ACM_STATE: StaticCell<State> = StaticCell::new();
static LOGGER_STATE: StaticCell<State> = StaticCell::new();

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

#[embassy_executor::task]
pub async fn task(
    usb: USB
) {
    let driver = Driver::new(usb, Irqs);

    let mut builder = embassy_usb::Builder::new(
        driver,
        usb_config(),
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 64])
    );

    let logger = CdcAcmClass::new(&mut builder, LOGGER_STATE.init(State::new()), 64);
    let logger_fut = embassy_usb_logger::with_class!(1024, log::LevelFilter::Trace, logger);

    let acm = CdcAcmClass::new(&mut builder, ACM_STATE.init(State::new()), 64);

    let mut usb = builder.build();

    // Run the low-level USB interface, the ACM handler, and the logger concurrently
    join(usb.run(), join(handle_acm(acm), logger_fut)).await;
}

async fn handle_acm(mut acm: CdcAcmClass<'static, Driver<'static, USB>>) -> ! {
    loop {
        acm.wait_connection().await;
        info!("Established USB connection");

        let _ = host_connection(&mut acm).await;
        info!("Disconnected from USB");
    }
}

struct Disconnected;

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

/// Handles serial data communication to another device connected over USB.
async fn host_connection(acm: &mut CdcAcmClass<'static, Driver<'static, USB>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];

    loop {
        let n = acm.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:?}", data);
        acm.write_packet(data).await?;
    }
}

fn usb_config() -> embassy_usb::Config<'static> {
    let mut config = embassy_usb::Config::new(
        harmoneyes_core::constants::USB_VENDOR_ID, 
        harmoneyes_core::constants::cuff::USB_PRODUCT_ID
    );

    config.manufacturer = Some(harmoneyes_core::constants::MANUFACTURER);
    config.product = Some(harmoneyes_core::constants::cuff::NAME);
    config.serial_number = Some(harmoneyes_core::constants::cuff::SERIAL_ONE);

    config
}