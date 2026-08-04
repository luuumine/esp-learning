pub mod device;
pub mod notes;
pub mod songs;

pub use device::{Buzzer, Error, VolumeType};
pub use notes::ToneValue;

use esp_hal::{
    delay::Delay,
    ledc::{LSGlobalClkSource, Ledc, channel, timer},
    peripherals::Peripherals,
};

pub fn play_pacman(peripherals: Peripherals) -> ! {
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut buzzer = Buzzer::new(
        &ledc,
        timer::Number::Timer0,
        channel::Number::Channel1,
        peripherals.GPIO4,
    );

    let delay = Delay::new();

    loop {
        buzzer.play_song(&songs::PACMAN).unwrap();

        delay.delay_millis(10_000);
    }
}
