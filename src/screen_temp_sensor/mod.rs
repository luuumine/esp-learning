use alloc::format;
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_10X20},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::Flex,
    i2c::master::{Config, I2c},
    peripherals::Peripherals,
    time::{Duration, Instant},
};
use log::{error, info};

mod hardware;
use hardware::*;

pub fn main(peripherals: Peripherals) -> ! {
    esp_alloc::heap_allocator!(size: 72 * 1024);
    let delay = Delay::new();

    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO21)
        .with_scl(peripherals.GPIO22);

    let mut display = init_screen(i2c);
    let mut dht11 = init_dht11(Flex::new(peripherals.GPIO4), delay);

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(BinaryColor::On)
        .build();

    loop {
        let uptime_secs = Instant::now().duration_since_epoch().as_secs();
        let mins = uptime_secs / 60;
        let secs = uptime_secs % 60;
        let time_str = format!("Up: {:02}:{:02}", mins, secs);

        let delay_start = Instant::now();

        match dht11.read() {
            Ok(reading) => {
                info!(
                    "Time: {} | Temp: {}°C | Hum: {}%",
                    time_str, reading.temperature, reading.humidity
                );

                display.clear(BinaryColor::Off).unwrap();

                let text = format!(
                    "{}\nTemp: {}C\nHum:  {}%",
                    time_str, reading.temperature, reading.humidity
                );

                Text::with_alignment(&text, Point::new(64, 16), text_style, Alignment::Center)
                    .draw(&mut display)
                    .unwrap();

                display.flush().unwrap();
            }
            Err(e) => {
                error!("Error reading DHT11: {:?}", e);
            }
        }

        while delay_start.elapsed() < Duration::from_millis(1000) {}
    }
}
