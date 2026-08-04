use esp_hal::{
    delay::Delay,
    gpio::DriveMode::PushPull,
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
    },
    peripherals::Peripherals,
    time::Rate,
};
use log::info;

pub fn test_tones(peripherals: Peripherals) -> ! {
    let delay = Delay::new();

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut timer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    let mut buzzer = ledc.channel(channel::Number::Channel0, peripherals.GPIO4);

    timer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty8Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(440),
        })
        .unwrap();

    buzzer
        .configure(channel::config::Config {
            timer: &timer0,
            duty_pct: 0,
            drive_mode: PushPull,
        })
        .unwrap();

    let tones = [("C4", 261), ("E4", 329), ("G4", 392)];

    loop {
        for (name, freq) in tones.iter() {
            info!("playing note: {} ({} Hz)", name, freq);

            let mut timer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
            timer0
                .configure(timer::config::Config {
                    duty: timer::config::Duty::Duty8Bit,
                    clock_source: timer::LSClockSource::APBClk,
                    frequency: Rate::from_hz(*freq),
                })
                .unwrap();

            buzzer.set_duty(50).unwrap();
            delay.delay_millis(500);

            buzzer.set_duty(0).unwrap();
        }

        buzzer.set_duty(0).unwrap();
        delay.delay_millis(1500);
    }
}
