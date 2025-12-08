//! Commands issued from the M8 firmware.

#![allow(dead_code)]

/// Specifies where something should be drawn.
pub struct Position {
    x: u16,
    y: u16,
}

/// Specifies how big something should be.
pub struct Size {
    width: u16,
    height: u16,
}

/// Specifies the colour of the set of drawn pixels.
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

/// A [Command] is sent from the M8 firmware and specifies what to
/// draw and where to draw it on the display.
pub enum M8Command {
    /// A rectangle draw command
    DrawRectangle {
        pos: Position,
        size: Size,
        colour: Colour,
    },

    /// A character draw command
    DrawCharacter {
        c: u32,
        pos: Position,
        foreground: Colour,
        background: Colour,
    },

    /// A oscilloscope waveform draw command
    DrawOscilloscopeWaveform {
        colour: Colour,
        waveform: [u8; 480],
        waveform_size: u16,
    },
}

impl M8Command {
    fn process() {
        todo!()
    }
}
