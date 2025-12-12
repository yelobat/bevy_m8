use bevy::prelude::*;

use crate::{M8UpdateSystems, serial::M8Connection};

// SLIP Protocol Constants
pub const SLIP_END: u8 = 0xC0;
pub const SLIP_ESC: u8 = 0xDB;
pub const SLIP_ESC_END: u8 = 0xDC;
pub const SLIP_ESC_ESC: u8 = 0xDD;

/// SLIP Decoder State.
#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Normal,
    Escaped,
}

/// SLIP Decoder.
#[derive(Resource, Debug)]
pub struct M8SlipDecoder {
    buffer: Vec<u8>,
    state: State,
}

/// The reserved capacity for the Slip Decoder.
const SLIP_BUFFER_CAPACITY: usize = 1024;

impl M8SlipDecoder {
    /// Creates a new M8SlipDecoder.
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(SLIP_BUFFER_CAPACITY),
            state: State::Normal,
        }
    }

    /// Resets the SLIP Decoder's internal buffer and state.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.state = State::Normal;
    }

    /// Append a byte to the internal buffer.
    fn put_byte(&mut self, byte: u8) {
        self.buffer.push(byte);
        self.state = State::Normal;
    }
}

// M8 Command Constants
#[allow(dead_code)]
const KEY_PRESS_STATE_COMMAND: u8 = 0xFB;
const DRAW_OSCILLOSCOPE_WAVEFORM_COMMAND: u8 = 0xFC;
const DRAW_CHARACTER_COMMAND: u8 = 0xFD;
const DRAW_RECTANGLE_COMMAND: u8 = 0xFE;
const SYSTEM_INFO_COMMAND: u8 = 0xFF;

/// Specifies where something should be drawn.
#[derive(Debug, PartialEq)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

/// Specifies how big something should be.
#[derive(Debug, PartialEq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

/// A [Command] is sent from the M8 firmware and specifies what to
/// draw and where to draw it on the display.
#[derive(Debug, PartialEq)]
pub enum M8Command {
    /// A rectangle draw command
    DrawRectangle {
        pos: Position,
        size: Size,
        colour: Color,
    },

    /// A character draw command
    DrawCharacter {
        c: u8,
        pos: Position,
        foreground: Color,
        background: Color,
    },

    /// An oscilloscope waveform draw command
    DrawOscilloscopeWaveform { colour: Color, waveform: Vec<u8> },

    /// System Info command
    SystemInfo {
        hardware_type: u8,
        major: u8,
        minor: u8,
        patch: u8,
        font_mode: u8,
    },
}

#[inline]
fn u8_slice_to_color(slice: &[u8]) -> Color {
    let red: f32 = (slice[0] as f32) / 255.0;
    let green: f32 = (slice[1] as f32) / 255.0;
    let blue: f32 = (slice[2] as f32) / 255.0;
    let alpha: f32 = 1.0;
    Color::Srgba(Srgba {
        red,
        green,
        blue,
        alpha,
    })
}

/// The M8 command decoder.
#[derive(Debug, Resource)]
pub struct M8CommandDecoder {
    // This is the last used colour used during drawing.
    current_colour: Color,

    // The list of decoded commands in the current cycle.
    pub commands: Vec<M8Command>,
}

impl M8CommandDecoder {
    /// Create a new M8 Command Decoder.
    pub fn new() -> Self {
        Self {
            current_colour: Color::Srgba(Srgba::default()),
            commands: Vec::new(),
        }
    }

    /// Reset the Command Buffer's internal state.
    pub fn reset(&mut self) {
        self.commands.clear();
    }

    /// Processes commands that have been slip decoded.
    pub fn decode(&mut self, buf: &[u8]) -> Result<(), M8DecodeError> {
        let len = buf.len();

        let command_type = buf[0];
        let command = match command_type {
            DRAW_OSCILLOSCOPE_WAVEFORM_COMMAND => {
                if len < 4 || len > 484 {
                    Err(M8DecodeError::DrawOscilloscopeWaveformInvalidFormat)
                } else {
                    Ok(M8Command::DrawOscilloscopeWaveform {
                        colour: u8_slice_to_color(&buf[1..=3]),
                        waveform: buf[4..].to_vec(),
                    })
                }
            }
            DRAW_CHARACTER_COMMAND => {
                if len != 12 {
                    Err(M8DecodeError::DrawCharacterInvalidFormat)
                } else {
                    Ok(M8Command::DrawCharacter {
                        c: buf[1],
                        pos: Position {
                            x: u16::from_le_bytes([buf[2], buf[3]]),
                            y: u16::from_le_bytes([buf[4], buf[5]]),
                        },
                        foreground: u8_slice_to_color(&buf[6..=8]),
                        background: u8_slice_to_color(&buf[9..=11]),
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
                    colour: self.current_colour.clone(),
                }),
                8 => {
                    self.current_colour = u8_slice_to_color(&buf[5..=7]);
                    Ok(M8Command::DrawRectangle {
                        pos: Position {
                            x: u16::from_le_bytes([buf[1], buf[2]]),
                            y: u16::from_le_bytes([buf[3], buf[4]]),
                        },
                        size: Size {
                            width: 1,
                            height: 1,
                        },
                        colour: self.current_colour.clone(),
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
                    colour: self.current_colour.clone(),
                }),
                12 => {
                    self.current_colour = u8_slice_to_color(&buf[9..=11]);
                    Ok(M8Command::DrawRectangle {
                        pos: Position {
                            x: u16::from_le_bytes([buf[1], buf[2]]),
                            y: u16::from_le_bytes([buf[3], buf[4]]),
                        },
                        size: Size {
                            width: u16::from_le_bytes([buf[5], buf[6]]),
                            height: u16::from_le_bytes([buf[7], buf[8]]),
                        },
                        colour: self.current_colour.clone(),
                    })
                }
                _ => Err(M8DecodeError::DrawRectangleInvalidFormat),
            },
            SYSTEM_INFO_COMMAND => Ok(M8Command::SystemInfo {
                hardware_type: buf[1],
                major: buf[2],
                minor: buf[3],
                patch: buf[4],
                font_mode: buf[5],
            }),
            _ => Err(M8DecodeError::UnrecognizedCommand(command_type)),
        }?;

        self.commands.push(command);
        Ok(())
    }
}

#[derive(Debug, Resource)]
pub struct M8Decoder {
    pub slip_decoder: M8SlipDecoder,
    pub command_decoder: M8CommandDecoder,
}

#[derive(Debug)]
pub enum M8DecodeError {
    UnknownEscapedByte(u8),
    DrawRectangleInvalidFormat,
    DrawCharacterInvalidFormat,
    DrawOscilloscopeWaveformInvalidFormat,
    UnrecognizedCommand(u8),
}

impl M8Decoder {
    /// Create a new M8Decoder.
    pub fn new() -> Self {
        Self {
            slip_decoder: M8SlipDecoder::new(),
            command_decoder: M8CommandDecoder::new(),
        }
    }

    /// Performs the following decoding structure:
    /// Serial -> SLIP Decoder -> Command Decoder -> Commands
    pub fn decode(&mut self, buf: &[u8]) -> Result<(), M8DecodeError> {
        self.command_decoder.reset();
        buf.iter().try_for_each(|chr| self.decode_byte(*chr))
    }

    /// Performs slip decoding across the SLIP stream. Once a complete
    /// command has been sent (SLIP_END has been received), command
    /// decoding is done and pushed into the command buffer.
    fn decode_byte(&mut self, byte: u8) -> Result<(), M8DecodeError> {
        match self.slip_decoder.state {
            State::Normal => match byte {
                SLIP_END => {
                    self.command_decoder.decode(&self.slip_decoder.buffer)?;
                    self.slip_decoder.reset();
                    Ok(())
                }
                SLIP_ESC => {
                    self.slip_decoder.state = State::Escaped;
                    Ok(())
                }
                _ => {
                    self.slip_decoder.put_byte(byte);
                    Ok(())
                }
            },
            State::Escaped => match byte {
                SLIP_ESC_END => {
                    self.slip_decoder.put_byte(SLIP_END);
                    Ok(())
                }
                SLIP_ESC_ESC => {
                    self.slip_decoder.put_byte(SLIP_ESC);
                    Ok(())
                }
                _ => {
                    self.slip_decoder.reset();
                    Err(M8DecodeError::UnknownEscapedByte(byte))
                }
            },
        }
    }
}

fn decode(connection: Res<M8Connection>, mut decoder: ResMut<M8Decoder>) {
    match decoder.decode(&connection.buffer[0..connection.size]) {
        Ok(_) => (),
        Err(err) => {
            decoder.slip_decoder.reset();
            warn!("Failed to decode serial buffer: {err:?}");
        },
    };
}

/// The M8 Decoder Plugin for decoding serial data into commands.
pub struct M8DecoderPlugin;
impl Plugin for M8DecoderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(M8Decoder::new());
        app.add_systems(Update, decode.in_set(M8UpdateSystems::Decode));
    }
}

#[test]
fn decode_test() {
    let mut decoder = M8Decoder::new();
    decoder
        .decode(&[
            DRAW_RECTANGLE_COMMAND,
            0,
            0,
            0,
            0,
            SLIP_END,
            DRAW_RECTANGLE_COMMAND,
            20,
            0,
            SLIP_ESC,
            SLIP_ESC_ESC,
            0,
            40,
            0,
            50,
            0,
            SLIP_END,
            DRAW_RECTANGLE_COMMAND,
            0,
            0,
            40,
            0,
            SLIP_END,
            DRAW_RECTANGLE_COMMAND,
            40,
            0,
            0,
            0,
            SLIP_END,
            DRAW_RECTANGLE_COMMAND,
            40,
            0,
            0,
        ])
        .unwrap();

    assert_eq!(
        decoder.command_decoder.commands,
        vec![
            M8Command::DrawRectangle {
                pos: Position { x: 0, y: 0 },
                size: Size {
                    width: 1,
                    height: 1
                },
                colour: Color::Srgba(Srgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0
                })
            },
            M8Command::DrawRectangle {
                pos: Position {
                    x: 20,
                    y: SLIP_ESC as u16
                },
                size: Size {
                    width: 40,
                    height: 50,
                },
                colour: Color::Srgba(Srgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0
                })
            },
            M8Command::DrawRectangle {
                pos: Position { x: 0, y: 40 },
                size: Size {
                    width: 1,
                    height: 1
                },
                colour: Color::Srgba(Srgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0
                })
            },
            M8Command::DrawRectangle {
                pos: Position { x: 40, y: 0 },
                size: Size {
                    width: 1,
                    height: 1
                },
                colour: Color::Srgba(Srgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0
                })
            },
        ]
    );

    decoder.decode(&[0, SLIP_END]).unwrap();
    assert_eq!(
        decoder.command_decoder.commands,
        vec![M8Command::DrawRectangle {
            pos: Position { x: 40, y: 0 },
            size: Size {
                width: 1,
                height: 1
            },
            colour: Color::Srgba(Srgba {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0
            })
        }]
    );
}

#[test]
#[should_panic]
fn failed_decode_test() {
    let mut decoder = M8Decoder::new();
    decoder.decode(&[DRAW_CHARACTER_COMMAND, SLIP_END]).unwrap();
}
