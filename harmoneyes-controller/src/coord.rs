use defmt::info;
use embassy_time::{Duration, Instant, Ticker, Timer};

use crate::ble;

/// A task for coordinating the distance information from nearby devices
#[embassy_executor::task]
pub async fn task() -> ! {
    let period: u32 = 1000;
    let mut ticker = Ticker::every(Duration::from_millis(period as u64));

    let mut count: u32 = 0;

    loop {
        random_timeout(period).await.await;

        keep_alive(count).await;
        count += 1;

        ticker.next().await; // Keep this at the bottom of the call stack
    }
}

/// The code here will run periodically after a random duration of milliseconds anywhere from 0 to 1000
async fn keep_alive(count: u32) {
    let mut buf = [0; 242];
    for (i, char) in "Signal ".chars().enumerate() {
        buf[i] = char as u8;
    }

    for (i, byte) in count.to_ne_bytes().into_iter().enumerate() {
        buf[i + 7] = byte;
    }
    ble::OUTBOX.send(buf).await;
}

async fn random_timeout(range: u32) -> Timer {
    let now = Instant::now();
    let value = crate::rng::get().await % range;
    Timer::at(now + Duration::from_millis(value as u64))
}