use embedded_dht_rs::dht11::Dht11;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{DriveMode, Flex, OutputConfig, Pull},
    i2c::master::I2c,
};
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::BufferedGraphicsMode, prelude::*};

pub fn init_screen<'a>(
    i2c: I2c<'a, esp_hal::Blocking>,
) -> Ssd1306<
    I2CInterface<I2c<'a, esp_hal::Blocking>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
> {
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();
    display
}

pub fn init_dht11<'a>(mut pin: Flex<'a>, delay: Delay) -> Dht11<Flex<'a>, Delay> {
    pin.apply_output_config(
        &OutputConfig::default()
            .with_drive_mode(DriveMode::OpenDrain)
            .with_pull(Pull::None),
    );
    pin.set_output_enable(true);
    pin.set_input_enable(true);
    pin.set_high();

    Dht11::new(pin, delay)
}
