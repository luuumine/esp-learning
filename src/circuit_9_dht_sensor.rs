use embedded_dht_rs::dht11::Dht11;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::{DriveMode, Flex, OutputConfig, Pull};
use esp_hal::peripherals::Peripherals;
use esp_hal::time::{Duration, Instant};
use log::{error, info};

pub fn main(peripherals: Peripherals) -> ! {
    let mut dht11_pin = Flex::new(peripherals.GPIO4);
    dht11_pin.apply_output_config(
        &OutputConfig::default()
            .with_drive_mode(DriveMode::OpenDrain)
            .with_pull(Pull::None),
    );
    dht11_pin.set_output_enable(true);
    dht11_pin.set_input_enable(true);
    dht11_pin.set_high();

    let mut dht11 = Dht11::new(dht11_pin, Delay::new());

    loop {
        match dht11.read() {
            Ok(reading) => info!(
                "Temperature: {}°C\nHumidity {}%\n",
                reading.temperature, reading.humidity
            ),
            Err(e) => {
                error!("Error: {:?}", e)
            }
        }

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(2000) {}
    }
}
