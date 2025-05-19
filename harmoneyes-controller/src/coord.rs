use defmt::info;
use embassy_time::{Duration, Instant, Timer};

/// A task for coordinating the distance information from nearby devices
#[embassy_executor::task]
pub async fn task() -> ! {
    loop {
        let next = random_timeout(1000).await; // Keep this at the top of the call stack

        keep_alive().await;

        next.await; // Keep this at the bottom of the call stack
    }
}

/// The code here will run periodically after a random duration of milliseconds anywhere from 0 to 1000
async fn keep_alive() {
    info!("Random keep-alive");
}

async fn random_timeout(range: u32) -> Timer {
    let now = Instant::now();
    let value = crate::rng::get().await % range;
    Timer::at(now + Duration::from_millis(value as u64))
}