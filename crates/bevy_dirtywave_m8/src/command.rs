//! Commands issued from the M8 firmware.

use bevy::prelude::*;
use crate::slip;

// M8 Command Constants
#[allow(dead_code)]
const KEY_PRESS_STATE_COMMAND: u8 = 0xFB;
const DRAW_OSCILLOSCOPE_WAVEFORM_COMMAND: u8 = 0xFC;
const DRAW_CHARACTER_COMMAND: u8 = 0xFD;
const DRAW_RECTANGLE_COMMAND: u8 = 0xFE;
const SYSTEM_INFO_COMMAND: u8 = 0xFF;

/// Specifies where something should be drawn.
#[derive(Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

/// Specifies how big something should be.
#[derive(Debug)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

/// Specifies the colour of the set of drawn pixels.
#[derive(Debug, Clone)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum M8CommandError {
    InvalidCommand,
    DrawRectangleInvalidFormat,
    DrawCharacterInvalidFormat,
    DrawOscilloscopeWaveformInvalidFormat,
    SystemInfoInvalidFormat,
    UnrecognizedCommand,
}

/// A [Command] is sent from the M8 firmware and specifies what to
/// draw and where to draw it on the display.
#[derive(Debug)]
pub enum M8Command {
    /// A rectangle draw command
    DrawRectangle {
        pos: Position,
        size: Size,
        colour: Colour,
    },

    /// A character draw command
    DrawCharacter {
        c: u8,
        pos: Position,
        foreground: Colour,
        background: Colour,
    },

    /// An oscilloscope waveform draw command
    DrawOscilloscopeWaveform { colour: Colour, waveform: Vec<u8> },

    /// System Info command
    SystemInfo {
        hardware_type: u8,
        major: u8,
        minor: u8,
        patch: u8,
        font_mode: u8,
    },
}

impl M8Command {
    /// From a byte slice, and the current set colour.
    fn from_bytes_with_context(
        buf: &[u8],
        current_colour: &mut Colour,
    ) -> Result<Self, M8CommandError> {
        let len = buf.len();
        let command_type = buf[0];
        match command_type {
            DRAW_OSCILLOSCOPE_WAVEFORM_COMMAND => {
                if len < 4 || len > 484 {
                    Err(M8CommandError::InvalidCommand)
                } else {
                    let colour = Colour {
                        r: buf[1],
                        g: buf[2],
                        b: buf[3],
                    };
                    Ok(M8Command::DrawOscilloscopeWaveform {
                        colour,
                        waveform: buf[4..].to_vec(),
                    })
                }
            }
            DRAW_CHARACTER_COMMAND => {
                if len != 12 {
                    Err(M8CommandError::InvalidCommand)
                } else {
                    Ok(M8Command::DrawCharacter {
                        c: buf[1],
                        pos: Position {
                            x: u16::from_le_bytes([buf[2], buf[3]]),
                            y: u16::from_le_bytes([buf[4], buf[5]]),
                        },
                        foreground: Colour {
                            r: buf[6],
                            g: buf[7],
                            b: buf[8],
                        },
                        background: Colour {
                            r: buf[9],
                            g: buf[10],
                            b: buf[11],
                        },
                    })
                }
            }
            DRAW_RECTANGLE_COMMAND => match buf.len() {
                5 => Ok(M8Command::DrawRectangle {
                    pos: Position {
                        x: u16::from_le_bytes([buf[1], buf[2]]),
                        y: u16::from_le_bytes([buf[3], buf[4]]),
                    },
                    size: Size {
                        width: 1,
                        height: 1,
                    },
                    colour: current_colour.clone(),
                }),
                8 => {
                    *current_colour = Colour {
                        r: buf[5],
                        g: buf[6],
                        b: buf[7],
                    };
                    Ok(M8Command::DrawRectangle {
                        pos: Position {
                            x: u16::from_le_bytes([buf[1], buf[2]]),
                            y: u16::from_le_bytes([buf[3], buf[4]]),
                        },
                        size: Size {
                            width: 1,
                            height: 1,
                        },
                        colour: current_colour.clone(),
                    })
                }
                9 => Ok(M8Command::DrawRectangle {
                    pos: Position {
                        x: u16::from_le_bytes([buf[1], buf[2]]),
                        y: u16::from_le_bytes([buf[3], buf[4]]),
                    },
                    size: Size {
                        width: u16::from_le_bytes([buf[5], buf[6]]),
                        height: u16::from_le_bytes([buf[7], buf[8]]),
                    },
                    colour: current_colour.clone(),
                }),
                12 => {
                    *current_colour = Colour {
                        r: buf[5],
                        g: buf[6],
                        b: buf[7],
                    };
                    Ok(M8Command::DrawRectangle {
                        pos: Position {
                            x: u16::from_le_bytes([buf[1], buf[2]]),
                            y: u16::from_le_bytes([buf[3], buf[4]]),
                        },
                        size: Size {
                            width: u16::from_le_bytes([buf[5], buf[6]]),
                            height: u16::from_le_bytes([buf[7], buf[8]]),
                        },
                        colour: current_colour.clone(),
                    })
                }
                _ => Err(M8CommandError::InvalidCommand),
            },
            SYSTEM_INFO_COMMAND => Ok(M8Command::SystemInfo {
                hardware_type: buf[1],
                major: buf[2],
                minor: buf[3],
                patch: buf[4],
                font_mode: buf[5],
            }),
            _ => Err(M8CommandError::UnrecognizedCommand),
        }
    }
}

/// The M8 command decoder.
#[derive(Debug, Resource)]
pub struct M8CommandDecoder {
    // This is the last used colour used during drawing.
    current_colour: Colour,

    // The list of decoded commands in the current cycle.
    commands: Vec<M8Command>,
}

impl M8CommandDecoder {
    /// Create a new M8 Command Decoder.
    pub fn new() -> Self {
        Self {
            current_colour: Colour { r: 0, g: 0, b: 0 },
            commands: Vec::new(),
        }
    }

    /// Processes commands that have been slip decoded.
    pub fn process(&mut self, buf: &[u8]) -> Result<(), M8CommandError> {
        self.commands.clear();
        buf.split(|&byte| byte == slip::SLIP_END)
            .filter(|buf| !buf.is_empty())
            .try_for_each(|buf| {
                let command = M8Command::from_bytes_with_context(buf, &mut self.current_colour)?;
                self.commands.push(command);
                Ok(())
            })
    }
}

pub struct M8CommandDecoderPlugin;
impl Plugin for M8CommandDecoderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(M8CommandDecoder::new());
    }
}

#[test]
fn decode_test() {
    let mut decoder = M8CommandDecoder::new();
    decoder
        .process(&[DRAW_RECTANGLE_COMMAND, 20, 0, 40, 0, slip::SLIP_END])
        .unwrap();
    let rectangle = decoder.commands.get(0).unwrap();
    match &rectangle {
        M8Command::DrawRectangle { pos, .. } => {
            assert_eq!(pos.x, 20);
            assert_eq!(pos.y, 40);
        }
        _ => panic!(),
    }

    decoder
        .process(&[
            DRAW_RECTANGLE_COMMAND,
            20,
            0,
            40,
            0,
            slip::SLIP_END,
            DRAW_CHARACTER_COMMAND,
            65,
            20,
            0,
            40,
            0,
            0,
            1,
            2,
            255,
            254,
            253,
            slip::SLIP_END,
        ])
        .unwrap();

    let character = decoder.commands.get(1).unwrap();
    match &character {
        M8Command::DrawCharacter {
            c,
            pos,
            foreground,
            background,
        } => {
            assert_eq!(c, &65);
            assert_eq!(pos.x, 20);
            assert_eq!(pos.y, 40);
            assert_eq!(foreground.r, 0);
            assert_eq!(foreground.g, 1);
            assert_eq!(foreground.b, 2);
            assert_eq!(background.r, 255);
            assert_eq!(background.g, 254);
            assert_eq!(background.b, 253);
        }
        _ => panic!(),
    }
}

#[test]
#[should_panic]
fn decode_panic_test() {
    let mut decoder = M8CommandDecoder::new();
    decoder
        .process(&[DRAW_RECTANGLE_COMMAND, 20, 20, 0, slip::SLIP_END])
        .unwrap();
}
