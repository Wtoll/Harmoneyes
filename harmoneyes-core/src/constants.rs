use const_format::concatcp;

pub const HARMONEYES: &str = "Harmoneyes";
pub const MANUFACTURER: &str = concatcp!(HARMONEYES, " Group");
pub const USB_VENDOR_ID: u16 = 0x1209; // Thank you https://pid.codes/

pub mod cuff {
    use const_format::concatcp;

    pub const NAME: &str = concatcp!(super::HARMONEYES, " Cuff");
    pub const USB_PRODUCT_ID: u16 = 0x0001;

    pub const SERIAL_ONE: &str = "HAU001";
    pub const SERIAL_TWO: &str = "HAU002";

    pub const I2C_ADDRESS: u16 = 0x45; // Nice...
}

pub mod controller {
    use const_format::concatcp;

    pub const NAME: &str = concatcp!(super::HARMONEYES, " Controller");
    pub const USB_PRODUCT_ID: u16 = 0x0002;

    pub const SERIAL_ONE: &str = "HAO001";
    pub const SERIAL_TWO: &str = "HAO002";
}