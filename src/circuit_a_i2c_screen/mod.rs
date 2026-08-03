pub mod scan;
pub mod test_bus;

use alloc::vec::Vec;
use esp_hal::{
    i2c::master::{Config, I2c, I2cAddress},
    peripherals::Peripherals,
};
use log::info;

fn with_i2c<F>(peripherals: Peripherals, f: F) -> !
where
    F: FnOnce(&mut I2c<'_, esp_hal::Blocking>),
{
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let sda = peripherals.GPIO21;
    let scl = peripherals.GPIO22;

    let mut i2c = I2c::new(peripherals.I2C0, Config::default())
        .unwrap()
        .with_sda(sda)
        .with_scl(scl);

    f(&mut i2c);

    loop {}
}

pub fn run_scan(peripherals: Peripherals) -> ! {
    with_i2c(peripherals, |i2c| {
        let _ = scan::get_devices(i2c);
    })
}

pub fn run_bus_test<A>(peripherals: Peripherals, addr: A) -> !
where
    A: Into<I2cAddress>,
{
    let target_addr: I2cAddress = addr.into();
    with_i2c(peripherals, |i2c| {
        test_bus::test_write(i2c, target_addr);
        test_bus::test_read(i2c, target_addr);
    })
}

pub fn test_all_devices(peripherals: Peripherals) -> ! {
    with_i2c(peripherals, |i2c| {
        let devices = scan::get_devices(i2c);
        let mut results = Vec::new();

        for addr in devices {
            info!("testing device at address: 0x{:02x}", addr);
            let max_read = test_bus::test_read(i2c, addr);
            let max_write = test_bus::test_write(i2c, addr);
            results.push((addr, max_read, max_write));
        }

        info!("done, limits per device:");
        for (addr, max_read, max_write) in results {
            info!("0x{:02x}: {} read, {} write", addr, max_read, max_write);
        }
    })
}
