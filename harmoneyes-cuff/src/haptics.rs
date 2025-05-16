use defmt::info;
use embassy_futures::{join::join, select::{select, Either}};
use embassy_rp::{gpio::{Level, Output}, peripherals::{PIN_3, PIN_4, PIN_5, PIN_6}};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;

pub static FRONT: Signal<CriticalSectionRawMutex, u64> = Signal::new();
pub static BACK: Signal<CriticalSectionRawMutex, u64> = Signal::new();
pub static LEFT: Signal<CriticalSectionRawMutex, u64> = Signal::new();
pub static RIGHT: Signal<CriticalSectionRawMutex, u64> = Signal::new();

#[embassy_executor::task]
pub async fn task(front: PIN_3, back: PIN_4, left: PIN_5, right: PIN_6) {
    join(
        join(
            run_motor(&FRONT, Output::new(front, Level::Low)), 
            run_motor(&BACK, Output::new(back, Level::Low))
        ), 
        join(
            run_motor(&LEFT, Output::new(left, Level::Low)), 
            run_motor(&RIGHT, Output::new(right, Level::Low))
        )
    ).await;
}

async fn run_motor(signal: &Signal<CriticalSectionRawMutex, u64>, mut out: Output<'static>) {
    let mut delay: u64 = signal.wait().await;

    loop {
        out.set_high();
        info!("Turned motor on");

        match select(signal.wait(), Timer::after_millis(delay)).await {
            Either::First(new_delay) => {
                delay = new_delay;
                continue;
            },
            Either::Second(()) => {
                out.set_low();
                info!("Turned motor off");
            },
        }

        delay = signal.wait().await;
    }
}

