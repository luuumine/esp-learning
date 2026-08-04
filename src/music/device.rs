use core::fmt::Debug;

use esp_hal::{
    clock::Clocks,
    delay::Delay,
    gpio::{AnyPin, DriveMode, Level, Output, OutputConfig, OutputPin},
    ledc::{
        Ledc, LowSpeed,
        channel::{self, Channel, ChannelIFace},
        timer::{self, Timer, TimerIFace},
    },
    time::Rate,
};

use super::notes::ToneValue;

/// Errors from Buzzer
#[derive(Debug)]
pub enum Error {
    Channel(channel::Error),
    Timer(timer::Error),
    VolumeNotSet,
    VolumeOutOfRange,
    LengthMismatch,
}

impl From<channel::Error> for Error {
    fn from(error: channel::Error) -> Self {
        Error::Channel(error)
    }
}

impl From<timer::Error> for Error {
    fn from(error: timer::Error) -> Self {
        Error::Timer(error)
    }
}

#[derive(Debug)]
pub enum VolumeType {
    OnOff,
    Duty,
}

struct Volume<'d> {
    volume_pin: AnyPin<'d>,
    volume_type: VolumeType,
    level: u8,
}

/// A buzzer instance driven by Ledc
pub struct Buzzer<'a> {
    timer: Timer<'a, LowSpeed>,
    channel_number: channel::Number,
    output_pin: AnyPin<'a>,
    delay: Delay,
    volume: Option<Volume<'a>>,
}

impl<'a> Buzzer<'a> {
    pub fn new(
        ledc: &'a Ledc,
        timer_number: timer::Number,
        channel_number: channel::Number,
        output_pin: impl OutputPin + 'a,
    ) -> Self {
        let timer = ledc.timer(timer_number);
        Self {
            timer,
            channel_number,
            output_pin: output_pin.degrade(),
            delay: Delay::new(),
            volume: None::<Volume>,
        }
    }

    pub fn with_volume(mut self, volume_pin: impl OutputPin + 'a, volume_type: VolumeType) -> Self {
        self.volume = Some(Volume {
            volume_pin: volume_pin.degrade(),
            volume_type,
            level: 50,
        });
        self
    }

    pub fn set_volume(&mut self, level: u8) -> Result<(), Error> {
        if let Some(ref mut volume) = self.volume {
            match volume.volume_type {
                VolumeType::OnOff => {
                    Output::new(
                        unsafe { volume.volume_pin.clone_unchecked() },
                        if level != 0 { Level::High } else { Level::Low },
                        OutputConfig::default(),
                    );
                    Ok(())
                }
                VolumeType::Duty => match level {
                    0..=99 => {
                        volume.level = level;
                        if !self.timer.is_configured() {
                            self.timer.configure(timer::config::Config {
                                duty: timer::config::Duty::Duty11Bit,
                                clock_source: timer::LSClockSource::APBClk,
                                frequency: Rate::from_hz(20_000),
                            })?;
                        }

                        let mut channel = Channel::new(self.channel_number, unsafe {
                            volume.volume_pin.clone_unchecked()
                        });
                        channel
                            .configure(channel::config::Config {
                                timer: &self.timer,
                                duty_pct: level,
                                drive_mode: DriveMode::PushPull,
                            })
                            .map_err(|e| e.into())
                    }
                    100 => {
                        Output::new(
                            unsafe { volume.volume_pin.clone_unchecked() },
                            Level::High,
                            OutputConfig::default(),
                        );
                        Ok(())
                    }
                    _ => Err(Error::VolumeOutOfRange),
                },
            }
        } else {
            Err(Error::VolumeNotSet)
        }
    }

    pub fn mute(&self) {
        if !self.timer.is_configured() {
            return;
        }
        let mut channel = Channel::new(self.channel_number, unsafe {
            self.output_pin.clone_unchecked()
        });

        channel
            .configure(channel::config::Config {
                timer: &self.timer,
                duty_pct: 0,
                drive_mode: DriveMode::PushPull,
            })
            .unwrap()
    }

    pub fn play(&mut self, frequency: u32) -> Result<(), Error> {
        if frequency == 0 {
            self.mute();
            return Ok(());
        }

        let mut result = 0;
        let mut value = Clocks::get().apb_clock / Rate::from_hz(frequency);

        while value > 1 && result < 14 {
            value >>= 1;
            result += 1;
        }

        self.timer.configure(timer::config::Config {
            duty: timer::config::Duty::try_from(result).unwrap(),
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(frequency),
        })?;

        let mut channel = Channel::new(self.channel_number, unsafe {
            self.output_pin.clone_unchecked()
        });
        channel.configure(channel::config::Config {
            timer: &self.timer,
            duty_pct: self.volume.as_ref().map_or(50, |v| v.level),
            drive_mode: DriveMode::PushPull,
        })?;

        Ok(())
    }

    pub fn play_tones<const T: usize>(
        &mut self,
        sequence: [u32; T],
        timings: [u32; T],
    ) -> Result<(), Error> {
        for (frequency, timing) in sequence.iter().zip(timings.iter()) {
            self.play(*frequency)?;
            self.delay.delay_millis(*timing);
            self.mute();
        }
        self.mute();
        Ok(())
    }

    pub fn play_tones_from_slice(
        &mut self,
        sequence: &[u32],
        timings: &[u32],
    ) -> Result<(), Error> {
        if sequence.len() != timings.len() {
            return Err(Error::LengthMismatch);
        }

        for (frequency, timing) in sequence.iter().zip(timings.iter()) {
            self.play(*frequency)?;
            self.delay.delay_millis(*timing);
            self.mute();
        }
        self.mute();
        Ok(())
    }

    pub fn play_song(&mut self, tones: &[ToneValue]) -> Result<(), Error> {
        for tone in tones {
            self.play(tone.frequency)?;
            self.delay.delay_millis(tone.duration);
            self.mute();
        }
        self.mute();
        Ok(())
    }
}
