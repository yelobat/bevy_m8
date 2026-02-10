//! This file provides SLIP decoding functionality.
use bevy::{
    color::{Color, Srgba},
    log::warn,
    math::U16Vec2,
};

// // SLIP Protocol Constants.
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
pub struct SlipDecoder {
    state: State,
    buffer: Vec<u8>,
}

/// The reserved capacity for the Slip Decoder.
const SLIP_BUFFER_CAPACITY: usize = 1024;

// M8 Command Constants
const KEY_PRESS_STATE_COMMAND: u8 = 0xFB;
const DRAW_OSCILLOSCOPE_WAVEFORM_COMMAND: u8 = 0xFC;
const DRAW_CHARACTER_COMMAND: u8 = 0xFD;
const DRAW_RECTANGLE_COMMAND: u8 = 0xFE;
const SYSTEM_INFO_COMMAND: u8 = 0xFF;

/// Specifies how big something should be.
pub type Size = U16Vec2;

/// Specifies where something should be drawn.
pub type Position = U16Vec2;

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

/// The command decoder.
pub struct CommandDecoder {
    current_colour: Color,
}

#[inline]
fn u8_slice_to_color(slice: &[u8]) -> Color {
    Color::Srgba(Srgba {
        red: slice[0] as f32 / 255.0,
        green: slice[1] as f32 / 255.0,
        blue: slice[2] as f32 / 255.0,
        alpha: 1.0,
    })
}

impl SlipDecoder {
    /// Creates a new SlipDecoder.
    pub fn new() -> Self {
        Self {
            state: State::Normal,
            buffer: Vec::with_capacity(SLIP_BUFFER_CAPACITY),
        }
    }

    pub fn process_byte(&mut self, byte: u8) -> Option<Vec<u8>> {
        match self.state {
            State::Normal => match byte {
                SLIP_END => {
                    if self.buffer.is_empty() {
                        return None;
                    }

                    let packet = self.buffer.clone();
                    self.buffer.clear();
                    return Some(packet);
                }
                SLIP_ESC => {
                    self.state = State::Escaped;
                    None
                }
                _ => {
                    self.buffer.push(byte);
                    None
                }
            },
            State::Escaped => {
                match byte {
                    SLIP_ESC_END => self.buffer.push(SLIP_END),
                    SLIP_ESC_ESC => self.buffer.push(SLIP_ESC),
                    _ => {
                        self.buffer.clear();
                    }
                }
                self.state = State::Normal;
                None
            }
        }
    }
}

impl CommandDecoder {
    pub fn new() -> Self {
        Self {
            current_colour: Color::WHITE,
        }
    }

    pub fn parse(&mut self, buf: &[u8]) -> Option<M8Command> {
        if buf.is_empty() {
            return None;
        }

        let cmd_type = buf[0];
        match cmd_type {
            DRAW_CHARACTER_COMMAND => self.parse_character(buf),
            DRAW_RECTANGLE_COMMAND => self.parse_rectangle(buf),
            DRAW_OSCILLOSCOPE_WAVEFORM_COMMAND => self.parse_waveform(buf),
            SYSTEM_INFO_COMMAND => self.parse_system_info(buf),
            KEY_PRESS_STATE_COMMAND => None,
            _ => {
                warn!("Unknown M8 command: {:02X}", buf[0]);
                None
            }
        }
    }

    fn parse_rectangle(&mut self, buf: &[u8]) -> Option<M8Command> {
        let len = buf.len();

        if len < 5 {
            return None;
        }

        if len == 8 || len == 12 {
            let offset = if len == 8 { 5 } else { 9 };
            self.current_colour = u8_slice_to_color(&buf[offset..offset + 3]);
        }

        Some(M8Command::DrawRectangle {
            pos: Position {
                x: u16::from_le_bytes([buf[1], buf[2]]),
                y: u16::from_le_bytes([buf[3], buf[4]]),
            },
            size: if len >= 9 {
                Size {
                    x: u16::from_le_bytes([buf[5], buf[6]]),
                    y: u16::from_le_bytes([buf[7], buf[8]]),
                }
            } else {
                Size { x: 1, y: 1 }
            },
            colour: self.current_colour,
        })
    }

    fn parse_character(&self, buf: &[u8]) -> Option<M8Command> {
        if buf.len() != 12 {
            return None;
        }
        Some(M8Command::DrawCharacter {
            c: buf[1],
            pos: Position {
                x: u16::from_le_bytes([buf[2], buf[3]]),
                y: u16::from_le_bytes([buf[4], buf[5]]),
            },
            foreground: u8_slice_to_color(&buf[6..=8]),
            background: u8_slice_to_color(&buf[9..=11]),
        })
    }

    fn parse_waveform(&self, buf: &[u8]) -> Option<M8Command> {
        if buf.len() < 4 {
            return None;
        }
        Some(M8Command::DrawOscilloscopeWaveform {
            colour: u8_slice_to_color(&buf[1..=3]),
            waveform: buf[4..].to_vec(),
        })
    }

    fn parse_system_info(&self, buf: &[u8]) -> Option<M8Command> {
        if buf.len() < 6 {
            return None;
        }
        Some(M8Command::SystemInfo {
            hardware_type: buf[1],
            major: buf[2],
            minor: buf[3],
            patch: buf[4],
            font_mode: buf[5],
        })
    }
}
