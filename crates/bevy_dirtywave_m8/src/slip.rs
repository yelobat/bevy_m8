//! Slip Implementation based on: <https://github.com/laamaa/m8c/src/backends/slip.c>

use bevy::prelude::*;

use crate::{DirtywaveM8UpdateSystems, serial::M8Connection};

// SLIP Protocol Constants
pub const SLIP_END: u8 = 0xC0;
pub const SLIP_ESC: u8 = 0xDB;
pub const SLIP_ESC_END: u8 = 0xDC;
pub const SLIP_ESC_ESC: u8 = 0xDD;

/// Errors that can occur during SLIP decoding.
#[derive(Debug, Clone, PartialEq)]
pub enum SlipError {
    BufferOverflow,
    InvalidPacket,
    UnknownEscapedByte(u8),
}

/// SLIP Decoder State.
#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Normal,
    Escaped,
}

/// SLIP Decoder.
#[derive(Resource, Debug)]
pub struct SlipDecoder {
    pub buffer: Vec<u8>,
    capacity: usize,
    state: State,
}

fn slip_decode(connection: Res<M8Connection>, mut decoder: ResMut<SlipDecoder>) {
    decoder
        .decode(&connection.buffer)
        .expect("Slip Decoding Failed");
}

impl SlipDecoder {
    /// Creates a new SlipDecoder with maximum [capacity].
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
            state: State::Normal,
        }
    }

    /// Resets the SLIP Decoder's internal buffer and state.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.state = State::Normal;
    }

    /// Append a byte to the internal buffer, respecting the maximum capacity.
    fn put_byte(&mut self, byte: u8) -> Result<(), SlipError> {
        if self.buffer.len() >= self.capacity {
            self.reset();
            Err(SlipError::BufferOverflow)
        } else {
            self.buffer.push(byte);
            self.state = State::Normal;
            Ok(())
        }
    }

    pub fn decode(&mut self, stream: &[u8]) -> Result<(), SlipError> {
        self.reset(); // <- Not sure if this line should be added.
        Ok(stream.iter().try_for_each(|chr| self.decode_byte(*chr))?)
    }

    fn decode_byte(&mut self, byte: u8) -> Result<(), SlipError> {
        match self.state {
            State::Normal => match byte {
                // NOTE Might actually keep this around
                // and keep the entire rendering logic contained
                // here instead of the current approach.
                // FIXME Might actually move everything into here
                // as it is more natural and easier to handle the
                // command decoding here. At this point, you have
                // the command inside of the slip buffer. So you
                // can decode the slip buffer at this point in
                // time since you know it is a valid command
                // (should be atleast) and you can then update
                // the display accordingly. Perhaps this should
                // actually be moved into as a system rather than
                // a traditional function, and the organization of
                // this repository should be made more monolithic.
                //SLIP_END => {
                //    // This is where the current message should actually
                //    // be processed.
                //    //self.reset();
                //    Ok(())
                //}
                SLIP_ESC => {
                    self.state = State::Escaped;
                    Ok(())
                }
                _ => {
                    self.put_byte(byte)?;
                    Ok(())
                }
            },
            State::Escaped => match byte {
                SLIP_ESC_END => {
                    self.put_byte(SLIP_END)?;
                    Ok(())
                }
                SLIP_ESC_ESC => {
                    self.put_byte(SLIP_ESC)?;
                    Ok(())
                }
                _ => {
                    self.reset();
                    return Err(SlipError::UnknownEscapedByte(byte));
                }
            },
        }
    }
}

pub struct M8SlipDecoderPlugin;
impl Plugin for M8SlipDecoderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SlipDecoder::new(1024));
        app.add_systems(
            Update,
            slip_decode.in_set(DirtywaveM8UpdateSystems::SlipDecode),
        );
    }
}

#[test]
fn decoder_init_test() {
    let decoder = SlipDecoder::new(1024);
    assert_eq!(decoder.buffer, []);
    assert_eq!(decoder.state, State::Normal);
    assert_eq!(decoder.capacity, 1024);
}

#[test]
fn decoder_insertion_test() {
    let mut decoder = SlipDecoder::new(1024);
    decoder.put_byte(b'c').unwrap();
    assert_eq!(decoder.buffer, [b'c']);
    assert_eq!(decoder.buffer.len(), 1);
    decoder.put_byte(b'd').unwrap();
    assert_eq!(decoder.buffer, [b'c', b'd']);
    assert_eq!(decoder.buffer.len(), 2);
    decoder.put_byte(b'e').unwrap();
    assert_eq!(decoder.buffer, [b'c', b'd', b'e']);
    assert_eq!(decoder.buffer.len(), 3);
}

#[test]
fn decoder_decode_test() {
    let mut decoder = SlipDecoder::new(1024);
    decoder.decode(&[SLIP_ESC, SLIP_ESC_END]).unwrap();
    assert_eq!(decoder.buffer, [SLIP_END]);
    decoder.decode(&[SLIP_ESC, SLIP_ESC_END, SLIP_END]).unwrap();
    assert_eq!(decoder.buffer, []);
    decoder
        .decode(&[
            SLIP_ESC,
            SLIP_ESC_END,
            b'\xfe',
            b'\xff',
            b'\x77',
            SLIP_ESC,
            SLIP_ESC_ESC,
        ])
        .unwrap();
    assert_eq!(
        decoder.buffer,
        [SLIP_END, b'\xfe', b'\xff', b'\x77', SLIP_ESC]
    );
}

#[test]
#[should_panic]
fn decoder_invalid_esc_test() {
    let mut decoder = SlipDecoder::new(1024);
    decoder.decode(&[SLIP_ESC, b'\x00']).unwrap();
}
