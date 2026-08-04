use alloc::format;
use embedded_graphics::{
    mono_font::{
        MonoTextStyleBuilder,
        ascii::{FONT_6X10, FONT_10X20},
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Baseline, Text},
};
use esp_hal::{
    delay::Delay,
    i2c::master::I2c,
    time::{Duration, Instant},
};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

pub fn hello_world(i2c: &mut I2c<'_, esp_hal::Blocking>) {
    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();
}

pub fn gm(i2c: &mut I2c<'_, esp_hal::Blocking>) {
    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(BinaryColor::On)
        .build();

    Text::with_alignment("gm chat", Point::new(64, 32), text_style, Alignment::Center)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();
}

pub fn count(i2c: &mut I2c<'_, esp_hal::Blocking>) {
    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(BinaryColor::On)
        .build();

    let mut next_frame = Instant::now();

    for i in 0..u32::MAX {
        next_frame += Duration::from_millis(500);

        display.clear(BinaryColor::Off).unwrap();

        Text::with_alignment(
            &format!("{}", i),
            Point::new(64, 32),
            text_style,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();

        while Instant::now() < next_frame {}
        display.flush().unwrap();
    }
}
