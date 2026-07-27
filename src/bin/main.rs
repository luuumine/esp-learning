#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
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

    circuit_2_potentiometer(peripherals);
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

fn circuit_2_potentiometer(peripherals: Peripherals) -> ! {
    let mut adc_config = AdcConfig::new();

    let mut potentiometer = adc_config.enable_pin(peripherals.GPIO4, Attenuation::_11dB);

    let mut adc = Adc::new(peripherals.ADC2, adc_config);

    loop {
        let pot_value: u16 = loop {
            if let Ok(val) = adc.read_oneshot(&mut potentiometer) {
                break val;
            }
        };

        info!("Potentiometer value: {}", pot_value);

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
    }
}
