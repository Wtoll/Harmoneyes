use embassy_futures::join::join;
use embassy_nrf::{bind_interrupts, interrupt::{self, InterruptExt, Priority}, peripherals::USBD, usb::{self, vbus_detect::SoftwareVbusDetect, Driver}};
use embassy_usb::{class::cdc_acm::{CdcAcmClass, State}, driver::EndpointError, Builder};
use defmt::info;
use static_cell::StaticCell;

static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

static ACM_STATE: StaticCell<State> = StaticCell::new();
static LOGGER_STATE: StaticCell<State> = StaticCell::new();

static VBUS_DETECT: StaticCell<SoftwareVbusDetect> = StaticCell::new();

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<USBD>;
});

#[embassy_executor::task]
pub async fn task(
    usb: USBD
) {
    // Hark, weary traveler! Take caution of a nearby softdevice. You don't want to anger it.
    interrupt::USBD.set_priority(Priority::P3);

    let vbus_detect = VBUS_DETECT.init(SoftwareVbusDetect::new(true, true));

    let driver = usb::Driver::new(usb, Irqs, &*vbus_detect);

    let mut builder = Builder::new(
        driver,
        usb_config(),
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 64])
    );

    let logger_class = CdcAcmClass::new(&mut builder, LOGGER_STATE.init(State::new()), 64);
    let serial_class = CdcAcmClass::new(&mut builder, ACM_STATE.init(State::new()), 64);

    let mut usb = builder.build();

    // Run the low-level USB interface, the ACM handler, and the logger concurrently
    join(usb.run(), join(handle_serial(serial_class), handle_logger(logger_class))).await;
}

async fn handle_logger<'a>(mut logger_class: CdcAcmClass<'a, Driver<'a, USBD, &'a SoftwareVbusDetect>>) -> ! {
    loop {
        logger_class.wait_connection().await;
        info!("Established USB connection");

        let _ = host_logger_connection(&mut logger_class).await;
        info!("Disconnected from USB");
    }
}

async fn handle_serial<'a>(mut serial_class: CdcAcmClass<'a, Driver<'a, USBD, &'a SoftwareVbusDetect>>) -> ! {
    loop {
        serial_class.wait_connection().await;
        let _ = host_serial_connection(&mut serial_class).await;
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
async fn host_serial_connection<'a>(serial_class: &mut CdcAcmClass<'a, Driver<'a, USBD, &'a SoftwareVbusDetect>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];

    loop {
        let n = serial_class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:?}", data);
        serial_class.write_packet(data).await?;
    }
}

async fn host_logger_connection<'a>(logger_class: &mut CdcAcmClass<'a, Driver<'a, USBD, &'a SoftwareVbusDetect>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];

    loop {
        let n = logger_class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:?}", data);
        logger_class.write_packet(data).await?;
    }
}

fn usb_config() -> embassy_usb::Config<'static> {
    let mut config: embassy_usb::Config<'static> = embassy_usb::Config::new(
        harmoneyes_core::constants::USB_VENDOR_ID, 
        harmoneyes_core::constants::controller::USB_PRODUCT_ID
    );

    config.manufacturer = Some(harmoneyes_core::constants::MANUFACTURER);
    config.product = Some(harmoneyes_core::constants::controller::NAME);
    config.serial_number = Some(harmoneyes_core::constants::controller::SERIAL_ONE);
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    config
}