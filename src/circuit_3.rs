use esp_backtrace as _;
use esp_hal::ledc::channel::{self, ChannelIFace};
use esp_hal::ledc::timer::{self, TimerIFace};
use esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed};
use esp_hal::peripherals::Peripherals;
use esp_hal::time::Rate;
use log::info;

pub fn main(peripherals: Peripherals) -> ! {
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
