#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::ledc::channel::{self, ChannelIFace};
use esp_hal::ledc::timer::{self, TimerIFace};
use esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed};
use esp_hal::main;
use esp_hal::peripherals::Peripherals;
use esp_hal::time::{Duration, Instant, Rate};
use log::info;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    // Initialize the CPU clock and peripherals
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    circuit_4_motion_sensor(peripherals);
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

fn circuit_3_pwm(peripherals: Peripherals) -> ! {
    let led_pin = peripherals.GPIO4;

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut timer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    timer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty8Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(5),
        })
        .unwrap();

    let mut channel0 = ledc.channel(channel::Number::Channel0, led_pin);
    channel0
        .configure(channel::config::Config {
            timer: &timer0,
            duty_pct: 0,
            drive_mode: esp_hal::gpio::DriveMode::PushPull,
        })
        .unwrap();

    loop {
        info!("starting fade up");
        channel0.start_duty_fade(0, 100, 1000).unwrap();
        while channel0.is_duty_fade_running() {}
        info!("starting fade down");
        channel0.start_duty_fade(100, 0, 1000).unwrap();
        while channel0.is_duty_fade_running() {}
    }
}

fn circuit_4_motion_sensor(peripherals: Peripherals) -> ! {
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
