use esp_backtrace as _;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::peripherals::Peripherals;
use esp_hal::time::Instant;
use log::info;

pub fn main(peripherals: Peripherals) -> ! {
    let buzzer_pin = peripherals.GPIO26;
    let motion_sensor_pin = peripherals.GPIO27;

    let mut buzzing = false;

    let mut previous = Instant::now();

    let interval_ms = 200;

    // let button = Input::new(peripherals.GPIO4, InputConfig::default());
    let mut buzzer = Output::new(buzzer_pin, Level::Low, OutputConfig::default());
    let motion_sensor = Input::new(motion_sensor_pin, InputConfig::default());

    loop {
        if motion_sensor.is_high() {
            buzzer.set_high();
            if !buzzing {
                info!("motion detected!")
            }
            previous = Instant::now();
            buzzing = true;
        } else {
            if previous.elapsed().as_millis() > interval_ms {
                buzzer.set_low();
                if buzzing {
                    info!("motion not detected for {} ms", interval_ms)
                }
                buzzing = false;
            }
        }
    }
}
