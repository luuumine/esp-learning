use esp_hal::i2c::master::{I2c, I2cAddress};
use log::debug;

pub fn test_write<A>(i2c: &mut I2c<'_, esp_hal::Blocking>, addr: A) -> usize
where
    A: Into<I2cAddress>,
{
    let target_addr: I2cAddress = addr.into();
    let data = [0u8; 1024];
    let mut limit = 0;

    debug!("testing writes to device...");
    for bytes in 1..=data.len() {
        if let Err(e) = i2c.write(target_addr, &data[0..bytes]) {
            debug!("write failed at {} bytes: {:?}", bytes, e);
            break;
        }
        limit = bytes;
    }
    limit
}

pub fn test_read<A>(i2c: &mut I2c<'_, esp_hal::Blocking>, addr: A) -> usize
where
    A: Into<I2cAddress>,
{
    let target_addr: I2cAddress = addr.into();
    let mut data = [0u8; 1024];
    let mut limit = 0;

    debug!("testing reads from device...");
    for bytes in 1..=data.len() {
        if let Err(e) = i2c.read(target_addr, &mut data[0..bytes]) {
            debug!("read failed at {} bytes: {:?}", bytes, e);
            break;
        }
        limit = bytes;
    }
    limit
}
