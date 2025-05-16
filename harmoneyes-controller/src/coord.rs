use defmt::info;
use embassy_time::Timer;

/// A task for coordinating the distance information from nearby devices
#[embassy_executor::task]
pub async fn task() -> ! {

    loop {
        // info!("Waiting for keep-alive");
        next_timeout().await;
        // info!("Keep-alive triggered");
    }
}




fn next_timeout() -> Timer {
    Timer::after_millis(50)
}