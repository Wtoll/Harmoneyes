use core::future::pending;

use nrf_softdevice::{ble::{advertisement_builder::{AdvertisementDataType, ExtendedAdvertisementBuilder}, peripheral}, Softdevice};

#[embassy_executor::task]
pub async fn task(sd: &'static Softdevice) -> ! {

    let ad = peripheral::NonconnectableAdvertisement::NonscannableUndirected {
        adv_data: &ExtendedAdvertisementBuilder::new()
            .raw(AdvertisementDataType::from_u8(0x2A), &[])
            .build()
    };


    pending().await
}