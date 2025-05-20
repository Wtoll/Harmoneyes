use core::{pin::pin, slice, str};
use defmt::info;
use embassy_futures::join::join;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Ticker, Timer};
use futures::future::{select, Either};
use nrf_softdevice::{ble::{advertisement_builder::{AdvertisementDataType, ExtendedAdvertisementBuilder, ExtendedAdvertisementPayload}, central, peripheral, Phy, PhySet}, Softdevice};

pub static OUTBOX: Channel<CriticalSectionRawMutex, [u8; 242], 1> = Channel::new();

#[embassy_executor::task]
pub async fn task(sd: &'static Softdevice) {
    let advertise = pin!(advertise(sd));
    let listen = pin!(listen(sd));

    join(advertise, listen).await;
}

async fn listen(sd: &'static Softdevice) {
    match central::scan(sd, &scan_config(), |params| {
        // We exclusively use nonconnectable and nonscannable advertising
        if params.type_.connectable() == 0 && params.type_.scannable() == 0 {
            let mut data = unsafe { slice::from_raw_parts(params.data.p_data, params.data.len as usize) };
            // Every packet has a header that is the mesh AD type followed by the magic string "Harmoneyes"
            if data.len() > 12 && data[1..12] == [0x2A, 0x48, 0x61, 0x72, 0x6d, 0x6f, 0x6e, 0x65, 0x79, 0x65, 0x73] {
                let data = &data[12..];
                info!("Harmoneyes Data: {}", data);
            }
        }
        return None::<()>
    }).await {
        Ok(_) => info!("We have a problem"),
        Err(_) => info!("We have a problem"),
    };
}

async fn advertise(sd: &'static Softdevice) {
    loop {
        let message = OUTBOX.receive().await;
        info!("Sending a new message");
        
        let ad = peripheral::NonconnectableAdvertisement::ExtendedNonscannableUndirected {
            set_id: 0,
            anonymous: false,
            adv_data: &build_ad_data(message)
        };

        let _ = peripheral::advertise(sd, ad, &peripheral_config()).await;
    }
}

fn scan_config() -> central::ScanConfig<'static> {
    let mut config = central::ScanConfig::default();

    config.extended = true;
    config.phys = PhySet::Coded;
    config.interval = 500;

    config
}

fn build_ad_data(message: [u8; 242]) -> ExtendedAdvertisementPayload {
    let mut raw = [0; 252];

    for (i, char) in "Harmoneyes".chars().enumerate() {
        raw[i] = char as u8;
    }

    for (i, byte) in message.into_iter().enumerate() {
        raw[i + 10] = byte;
    }

    ExtendedAdvertisementBuilder::new()
        .raw(AdvertisementDataType::from_u8(0x2A), &raw)
        .build()
}

fn peripheral_config() -> peripheral::Config {
    let mut config = peripheral::Config::default();

    config.interval = 50;
    config.timeout = Some(10);
    config.primary_phy = Phy::Coded;
    config.secondary_phy = Phy::Coded;

    config
}