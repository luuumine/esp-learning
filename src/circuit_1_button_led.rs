use esp_backtrace as _;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::peripherals::Peripherals;
use esp_hal::time::{Duration, Instant};
use log::info;

pub fn main(peripherals: Peripherals) -> ! {
    let button = Input::new(peripherals.GPIO4, InputConfig::default());
    let mut led = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());

    loop {
        if button.is_high() {
            if led.is_set_low() {
                info!("setting led to high");
            }
            led.set_high();
        } else {
            if led.is_set_high() {
                info!("setting led to low");
            }
            led.set_low();
        }

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(50) {}
    }
}
