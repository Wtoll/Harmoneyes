use core::future::pending;

use embassy_nrf::peripherals::USBD;

#[embassy_executor::task]
pub async fn task(
    _usb: USBD
) -> ! {
    loop { pending::<()>().await; }
}