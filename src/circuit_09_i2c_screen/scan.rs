use alloc::vec::Vec;
use esp_hal::i2c::master::I2c;
use log::info;

pub fn get_devices(i2c: &mut I2c<'_, esp_hal::Blocking>) -> Vec<u8> {
    info!("Scanning I2C bus...");
    let mut found = Vec::new();

    let data = [0u8; 1024];

    for addr in 1..=127 {
        if i2c.write(addr, &data).is_ok() {
            info!("found device at address: 0x{:02x}", addr);
            found.push(addr);
        }
    }

    info!(
        "scan complete: found {} device{}",
        found.len(),
        if found.len() == 1 { "" } else { "s" }
    );
    found
}
