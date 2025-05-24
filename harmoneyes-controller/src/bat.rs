//! # Battery
//! 
//! ## Hardware
//! The Adafruit Feather nRF52840 has an on-board voltage regulator that
//! will automatically charge the battery when the USB port is powered
//! and will convert the battery voltage to a stable 3.3V for the
//! microcontroller.
//! 
//! There is also an analog input pin that is dedicated to being a
//! voltage monitor for the battery. In between the battery and the pin,
//! however, is a dual 150kOhm resistor voltage divider meaning the
//! analog pin is only reading half of the actual battery voltage.
//! This is actually great for us because the ADC is limited to a
//! maximum value of 3.3V and lowering the minimum value we're
//! trying to read allows use to transduce the voltage at a greater
//! accuracy. 
//! 
//! The lithium polymer batteries that are being have a known voltage
//! range of about 4.2V to 3.2V.
//! 
//! ### ADC
//! 
//! The analog to digital converter has a variety of configurations,
//! but in this case it's been configured to use a reference voltage
//! of 2.4V (just above 4.2V / 2) to maximize the ADC range, and a
//! 12-bit resolution.
//! 
//! ## Battery Testing
//!  
//! In testing it was found that the on-board voltage regulator will
//! turn off when the battery voltage (ADC * 2) crosses below a threshold
//! of about 3.3V and will stop charging when the battery voltage
//! crosses above a threshold of about 4.2V

use defmt::info;
use embassy_nrf::{bind_interrupts, interrupt::{self, InterruptExt, Priority}, peripherals::{P0_29, SAADC}, saadc::{self, ChannelConfig, Gain, Input, Reference, Saadc, Time}};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Ticker};

pub static BATTERY: Mutex<CriticalSectionRawMutex, Option<BatteryCharge>> = Mutex::new(None);

bind_interrupts!(struct Irqs {
    SAADC => saadc::InterruptHandler;
});

#[embassy_executor::task]
pub async fn task(
    adc: SAADC,
    pin: P0_29
) -> ! {
    // Hark, weary traveler! Take caution of a nearby softdevice. You don't want to anger it.
    interrupt::SAADC.set_priority(Priority::P3);

    let mut channel_conf = ChannelConfig::single_ended(pin.degrade_saadc());
    channel_conf.reference = Reference::INTERNAL; // The internal reference is 0.6V
    channel_conf.gain = Gain::GAIN1_4; // A gain of 1/4 means 4 * 0.6V = 2.4V
    channel_conf.time = Time::_40US; // 40μs because it's just a battery...

    let mut saadc = Saadc::new(adc, Irqs, saadc_config(), [channel_conf]);

    let mut ticker = Ticker::every(Duration::from_millis(2500));

    loop {
        // Record the battery
        let mut buf = [0; 1];
        saadc.sample(&mut buf).await;

        let bat = BatteryCharge::new(buf[0]);

        // info!("New Battery Reading {{\n    Raw: {}\n    Parsed: {} mV\n    Parsed: {}%\n}}", bat.0, bat.as_millivolts(), bat.as_ratio() * 100.0);

        BATTERY.lock().await.replace(BatteryCharge::new(buf[0]));

        ticker.next().await;
    }
}

fn saadc_config() -> saadc::Config {
    Default::default()
}








pub struct BatteryCharge(i16);

impl BatteryCharge {
    pub fn new(raw: i16) -> Self {
        Self(raw)
    }

    pub fn as_ratio(&self) -> f32 {
        // 2816 is the raw value that corresponds to 3.3V
        // (3300 / 2) * (4096 / 2400)
        // 3584 is the raw value that corresponds to 4.2V
        // (4200 / 2) * (4096 / 2400)
        // This method just uses a linear interpolation between those two values.
        (((self.0 - 2816) as f32) / ((3584 - 2816) as f32)).clamp(0.0, 1.0)
    }

    pub fn as_millivolts(&self) -> f32 {
        // 2400 is the reference voltage in mV (0.6V * 4)
        // 4096 is for the 12-bit resolution (2^12)
        // 2.0 is because of the voltage divider
        ((self.0 as f32) * (2400.0 / 4096.0)) * 2.0
    }
}