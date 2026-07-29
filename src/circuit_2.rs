use esp_backtrace as _;
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::peripherals::Peripherals;
use esp_hal::time::{Duration, Instant};
use log::info;

pub fn main(peripherals: Peripherals) -> ! {
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
