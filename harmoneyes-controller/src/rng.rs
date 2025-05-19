use core::cell::OnceCell;

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use nrf_softdevice::Softdevice;

static RNG: Mutex<CriticalSectionRawMutex, OnceCell<SoftdeviceRng>> = Mutex::new(OnceCell::new());

static REQ_RNG: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static RNG_VAL: Signal<CriticalSectionRawMutex, u32> = Signal::new();

pub async fn initialize(spawner: &Spawner, sd: &'static Softdevice) {
    spawner.must_spawn(task(sd));
    RNG.lock().await.set(SoftdeviceRng::new()).expect("Failed to initialize RNG driver");
}

pub async fn get() -> u32 {
    RNG.lock().await.get_mut().unwrap().get().await
}

#[derive(Debug)]
pub struct SoftdeviceRng {}

impl SoftdeviceRng {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get(&mut self) -> u32 {
        REQ_RNG.signal(());
        RNG_VAL.wait().await
    }
}

#[embassy_executor::task]
async fn task(sd: &'static Softdevice) -> ! {
    loop {
        let _ = REQ_RNG.wait().await;
        let mut buf = [0u8; 4];
        loop {
            match nrf_softdevice::random_bytes(sd, &mut buf) {
                Ok(()) => break,
                Err(_) => continue,
            }
        }
        RNG_VAL.signal(u32::from_ne_bytes(buf));
        REQ_RNG.reset();
    }
}