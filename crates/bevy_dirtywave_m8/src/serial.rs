//! The Dirtywave M8 serialport interaction API.

use async_channel::{Receiver, Sender};
use bevy::prelude::*;
use serialport::{SerialPort, SerialPortType};
use std::sync::Mutex;

use crate::{DirtywaveM8UpdateSystems, command::M8Command};

/// The maximum amount of bytes to read from the serial device in one pass.
const SERIAL_READ_SIZE: usize = 1024;

// M8 Constants
const M8_VID: u16 = 0x16C0;
const M8_PID: u16 = 0x048A;
const BAUD_RATE: u32 = 115_200;

/// Represents the connection to the M8.
#[derive(Resource)]
struct M8Connection {
    port: Mutex<Box<dyn SerialPort>>,
    size: usize,
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

fn read(mut connection: ResMut<M8Connection>) {
    match connection.read() {
        Ok(size) => info!(
            "M8 Serial Buffer: {:?} has size: {:?}",
            connection.buffer,
            connection.size
        ),
        Err(err) => warn!("M8 Serial Error: {:?}", err),
    }
}

#[derive(Debug, Default)]
pub struct M8SerialPlugin {
    pub preferred_device: Option<String>,
}

impl Plugin for M8SerialPlugin {
    fn build(&self, app: &mut App) {
        let connection =
            M8Connection::open(self.preferred_device.clone()).expect("Failed to connect to the M8");
        app.insert_resource(connection);
        app.add_systems(Update, read.in_set(DirtywaveM8UpdateSystems::SerialRead));
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

    pub fn read(&mut self) -> Result<usize, M8ConnectionError> {
        self.buffer.fill(0);
        if let Ok(mut port) = self.port.lock() {
            self.size = port
                .read(&mut self.buffer)
                .map_err(|e| M8ConnectionError::Io(e.to_string()))?;
            Ok(self.size)
        } else {
            Err(M8ConnectionError::Io("SerialPort busy".into()))
        }
    }

    pub fn open(preferred_device: Option<String>) -> Result<Self, M8ConnectionError> {
        let port_name = Self::find_port_name(preferred_device)?;

        info!("Opening M8 Serial Port at {}", port_name);

        let port = serialport::new(port_name, BAUD_RATE)
            .open()
            .map_err(|e| M8ConnectionError::SerialPort(e.to_string()))?;

        Ok(Self {
            port: Mutex::new(port),
            buffer: [0; SERIAL_READ_SIZE],
            size: 0,
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
