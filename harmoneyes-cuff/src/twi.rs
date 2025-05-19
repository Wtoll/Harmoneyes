use log::info;
use embassy_rp::{i2c::{self}, i2c_slave::{self, Command, Error, I2cSlave}, peripherals::{I2C1, PIN_22, PIN_23}};

embassy_rp::bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

#[embassy_executor::task]
pub async fn task(
    twi: I2C1,
    scl: PIN_23,
    sda: PIN_22
) {
    let mut driver = I2cSlave::new(twi, scl, sda, Irqs, peripheral_config());

    loop {
        let mut buf = [0u8; 64];
        match driver.listen(&mut buf).await {
            Ok(Command::Write(len)) => {
                info!("Write: {:?}", &buf[..len]);

                if len >= 9 {
                    let delay = u64::from_le_bytes(buf[1..9].try_into().unwrap());
                    match buf[0] {
                        0x10 => crate::haptics::FRONT.signal(delay),
                        0x11 => crate::haptics::BACK.signal(delay),
                        0x12 => crate::haptics::LEFT.signal(delay),
                        0x13 => crate::haptics::RIGHT.signal(delay),
                        _ => {}
                    }
                }
            },
            Ok(Command::GeneralCall(len)) => { info!("General Call: {:?}", &buf[..len]); },
            Ok(Command::WriteRead(len)) => {
                info!("WriteRead: {:?}", &buf[..len]);
                let _ = driver.respond_to_read(&[0xBC]).await;
            },
            Ok(Command::Read) => {
                info!("Read");
                let _ = driver.respond_to_read(&[0xAB]).await;
            },
            Err(Error::PartialGeneralCall(len)) => { info!("Partial General: {:?}", &buf[..len]); },
            Err(Error::PartialWrite(len)) => { info!("Partial Write: {:?}", &buf[..len]); },
            Err(Error::Abort(reason)) => { info!("Abort: {:?}", reason); },
            Err(Error::InvalidResponseBufferLength) => { info!("Invalid Response Length"); },
            Err(_e) => {}
        }
    }
}

fn peripheral_config() -> i2c_slave::Config {
    let mut config = i2c_slave::Config::default();
    config.addr = harmoneyes_core::constants::cuff::I2C_ADDRESS;

    config
}