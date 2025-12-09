//! The Dirtywave M8 serialport interaction API.

use async_channel::{Receiver, Sender};
use bevy::prelude::*;
use serialport::{SerialPort, SerialPortType};
use std::{sync::Mutex, time::Duration};

use crate::command::M8Command;

/// The maximum amount of bytes to read from the serial device in one pass.
const SERIAL_READ_SIZE: usize = 1024;

/// The delay between subsequent reads (measured in milliseconds).
const SERIAL_READ_DELAY_MS: usize = 4;

/// The timeout for reading and writing the M8 serial connection.
const SERIAL_TIMEOUT_MS: Duration = Duration::from_millis(5);

// M8 Constants
const M8_VID: u16 = 0x16C0;
const M8_PID: u16 = 0x048A;
const BAUD_RATE: u32 = 115_200;

/// Represents the connection to the M8.
#[derive(Resource)]
struct M8Connection {
    port: Mutex<Box<dyn SerialPort>>,
    buffer: [u8; 1024],
}

#[derive(Debug, Resource)]
struct M8SerialReader(Receiver<M8Command>);

#[derive(Debug, Resource)]
struct M8SerialWriter(Sender<u32>);

#[derive(Debug, Clone)]
enum M8ConnectionError {
    NoDeviceFound,
    Io(String),
    SerialPort(String),
}

#[derive(Debug, Default)]
pub struct M8SerialPlugin {
    pub preferred_device: Option<String>,
}

impl Plugin for M8SerialPlugin {
    fn build(&self, app: &mut App) {
        let mut connection =
            M8Connection::open(self.preferred_device.clone()).expect("Failed to connect to the M8");

        connection
            .send_enable_command()
            .expect("Failed to send the enable command!");
        connection.read().expect("Failed to read from the M8!");

        app.insert_resource(connection);
    }
}

impl M8Connection {
    pub fn send(&mut self, buf: &[u8]) -> Result<usize, M8ConnectionError> {
        if let Ok(mut port) = self.port.lock() {
            Ok(port
                .write(buf)
                .map_err(|e| M8ConnectionError::Io(e.to_string()))?)
        } else {
            Err(M8ConnectionError::Io("SerialPort busy".into()))
        }
    }

    fn send_enable_command(&mut self) -> Result<usize, M8ConnectionError> {
        self.send(b"E")
    }

    fn send_reset_command(&mut self) -> Result<usize, M8ConnectionError> {
        self.send(b"R")
    }

    pub fn read(&mut self) -> Result<(), M8ConnectionError> {
        self.buffer.fill(0);
        if let Ok(mut port) = self.port.lock() {
            Ok(port
                .read_exact(&mut self.buffer)
                .map_err(|e| M8ConnectionError::Io(e.to_string()))?)
        } else {
            Err(M8ConnectionError::Io("SerialPort busy".into()))
        }
    }

    pub fn open(preferred_device: Option<String>) -> Result<Self, M8ConnectionError> {
        let port_name = Self::find_port_name(preferred_device)?;

        info!("Opening M8 Serial Port at {}", port_name);

        let port = serialport::new(port_name, BAUD_RATE)
            .timeout(SERIAL_TIMEOUT_MS)
            .open()
            .map_err(|e| M8ConnectionError::SerialPort(e.to_string()))?;

        Ok(Self {
            port: Mutex::new(port),
            buffer: [0; SERIAL_READ_SIZE],
        })
    }

    fn find_port_name(preferred: Option<String>) -> Result<String, M8ConnectionError> {
        let ports = serialport::available_ports()
            .map_err(|e| M8ConnectionError::SerialPort(e.to_string()))?;

        if let Some(pref) = preferred
            && ports.iter().any(|p| p.port_name == pref)
        {
            return Ok(pref.to_string());
        }

        for port in ports {
            if let SerialPortType::UsbPort(info) = port.port_type
                && info.vid == M8_VID
                && info.pid == M8_PID
            {
                return Ok(port.port_name);
            }
        }

        Err(M8ConnectionError::NoDeviceFound)
    }
}
