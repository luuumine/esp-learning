#![allow(unused)]

/// Generate a ToneValue slice from Song beats and tempo
#[macro_export]
macro_rules! song {
    ($tempo:expr, [$(($note:expr, $duration:expr)),*]) => {
        {
            // 240s per whole note * 12 multiplier
            const WHOLENOTE: u32 = (240_000 * 12) / $tempo;
            [
                $(
                    // Use $crate to give the absolute path to the struct
                    $crate::music::notes::ToneValue { frequency: $note, duration: WHOLENOTE / $duration },
                )*
            ]
        }
    };
}

mod pacman;
pub use pacman::PACMAN;
