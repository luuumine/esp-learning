#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::peripherals::Peripherals;
use esp_hal::time::{Duration, Instant};
use log::info;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    // Initialize the CPU clock and peripherals
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    circuit_1_button_led(peripherals);
}

fn circuit_1_button_led(peripherals: Peripherals) -> ! {
    let button = Input::new(peripherals.GPIO4, InputConfig::default());
    let mut led = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());

    loop {
        if button.is_high() {
            led.set_high();
            info!("led set to high");
        } else {
            led.set_low();
            info!("led set to low");
        }

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(50) {}
    }
}
