use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    peripherals::Peripherals,
    rng::Rng,
};
use log::info;

const SEQUENCE_LENGTH: usize = 25;

pub fn main(peripherals: Peripherals) -> ! {
    info!("=== SIMON STARTING ===");

    let in_conf = InputConfig::default().with_pull(Pull::Up);
    let out_conf = OutputConfig::default();

    let buttons = [
        Input::new(peripherals.GPIO15, in_conf), // note: dont use gpio2 for some reason??
        Input::new(peripherals.GPIO4, in_conf),
        Input::new(peripherals.GPIO16, in_conf),
    ];

    let mut leds = [
        Output::new(peripherals.GPIO5, Level::Low, out_conf),
        Output::new(peripherals.GPIO18, Level::Low, out_conf),
        Output::new(peripherals.GPIO19, Level::Low, out_conf),
    ];

    let rng = Rng::new();
    let delay = Delay::new();

    info!("GPIO initialization complete");

    info!("Checking buttons...");

    for (i, button) in buttons.iter().enumerate() {
        if button.is_low() {
            info!("WARNING: BUTTON {} is already PRESSED!", i);
        } else {
            info!("BUTTON {} is released", i);
        }
    }

    let mut sequence = [0u8; SEQUENCE_LENGTH];

    for value in sequence.iter_mut() {
        *value = (rng.random() % 3) as u8;
    }

    info!("Generated sequence:");

    for (i, value) in sequence.iter().enumerate() {
        info!("  [{}] = {}", i, value);
    }

    loop {
        let mut round_length = 1;
        let mut game_over = false;

        info!("Starting new game");

        while !game_over {
            info!("--- ROUND {} ---", round_length);

            info!("Showing sequence...");

            for &value in sequence[..round_length].iter() {
                let index = value as usize;

                leds[index].set_high();
                delay.delay_millis(500);

                leds[index].set_low();
                delay.delay_millis(200);
            }

            info!("Sequence finished, waiting for player");

            for &expected in sequence[..round_length].iter() {
                let expected = expected as usize;

                info!("Waiting for button {}", expected);

                let pressed = 'wait: loop {
                    for (i, button) in buttons.iter().enumerate() {
                        if button.is_low() {
                            info!("BUTTON {} PRESSED", i);

                            break 'wait i;
                        }
                    }
                };

                info!("Player pressed {}, expected {}", pressed, expected);

                leds[pressed].set_high();
                delay.delay_millis(150);
                leds[pressed].set_low();

                if pressed != expected {
                    info!("WRONG! Expected {}, got {}", expected, pressed);

                    game_over = true;
                    break;
                }

                info!("Correct!");

                while buttons[pressed].is_low() {}

                delay.delay_millis(100);
            }

            if game_over {
                info!("=== GAME OVER ===");

                for _ in 0..3 {
                    for led in leds.iter_mut() {
                        led.set_high();
                    }

                    delay.delay_millis(200);

                    for led in leds.iter_mut() {
                        led.set_low();
                    }

                    delay.delay_millis(200);
                }

                break;
            }

            if round_length == SEQUENCE_LENGTH {
                info!("=== YOU WIN! ===");

                for _ in 0..3 {
                    for led in leds.iter_mut() {
                        led.set_high();
                    }

                    delay.delay_millis(150);

                    for led in leds.iter_mut() {
                        led.set_low();
                    }

                    delay.delay_millis(150);
                }

                info!("Generating new sequence...");

                for value in sequence.iter_mut() {
                    *value = (rng.random() % 3) as u8;
                }

                for (i, value) in sequence.iter().enumerate() {
                    info!("  [{}] = {}", i, value);
                }

                break;
            }

            round_length += 1;

            delay.delay_millis(500);
        }
    }
}
