use core::future::pending;

use embassy_nrf::peripherals::{P0_29, SAADC};

#[embassy_executor::task]
pub async fn task(
    _adc: SAADC,
    _pin: P0_29
) -> ! {
    loop { pending::<()>().await; }
}