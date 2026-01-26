//! The Dirtywave M8 serialport interaction API.

// FIXME This is where all the problems currently lie.
// Right now the reading from this port is just not producing
// good data, it times out frequently and it ends up corrupting the
// stream causing casscading errors, not sure why this is happening.
//
// It could be because I'm not using a separate thread that performs
// the reading from the serial port, but I don't see why I would need
// this.
//
// Should perhaps look at this example:
// https://github.com/bevyengine/bevy/blob/main/examples/async_tasks/async_channel_pattern.rs
// checkout into a new branch called async-serial-read-feature
// and attempt an asynchronous implementation to see if it fixes any issues. Although, I don't
// see why making it asynchronous will miraculously fix these issues.

use bevy::{diagnostic::LogDiagnosticsPlugin, prelude::*};
use crossbeam_channel::{Receiver, Sender, unbounded};
use serialport::SerialPortType;
use std::{thread, time::Duration};

use crate::decoder::{CommandDecoder, M8Command, SlipDecoder};

/// The maximum amount of bytes to read from the serial device in one pass.
const SERIAL_READ_SIZE: usize = 1024;

// M8 Constants
const M8_VID: u16 = 0x16C0;
const M8_PID: u16 = 0x048A;
const BAUD_RATE: u32 = 115_200;

/// Represents the connection to the M8.
#[derive(Resource)]
pub struct M8Connection {
    pub rx: Receiver<M8Command>,
    pub tx: Sender<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum M8ConnectionError {
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
        let (to_bevy, from_serial) = unbounded::<M8Command>();
        let (to_serial, from_bevy) = unbounded::<Vec<u8>>();

        let port_name = M8Connection::find_port_name(self.preferred_device.clone())
            .expect("Could not find M8 Tracker.");

        thread::spawn(move || {
            let mut port = serialport::new(port_name, BAUD_RATE)
                .timeout(Duration::from_millis(10))
                .parity(serialport::Parity::None)
                .stop_bits(serialport::StopBits::One)
                .flow_control(serialport::FlowControl::None)
                .data_bits(serialport::DataBits::Eight)
                .open()
                .expect("Failed to open M8 port");

            if let Err(e) = port.write_all(b"E") {
                error!("Failed to send Enable command: {:?}", e);
            } else {
                info!("Sent Enable command ('E') to M8");
            }

            thread::sleep(Duration::from_millis(60));

            if let Err(e) = port.write_all(b"R") {
                error!("Failed to send Reset/Refresh command: {:?}", e);
            } else {
                info!("Sent Reset/Refresh command ('R') to M8");
            }

            let mut slip_decoder = SlipDecoder::new();
            let mut command_decoder = CommandDecoder::new();
            let mut read_buffer = [0u8; SERIAL_READ_SIZE];

            loop {
                match port.read(&mut read_buffer) {
                    Ok(count) if count > 0 => {
                        for i in 0..count {
                            if let Some(packet) = slip_decoder.process_byte(read_buffer[i]) {
                                if let Some(cmd) = command_decoder.parse(&packet) {
                                    to_bevy.send(cmd).ok();
                                }
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                    Err(e) => error!("Serial Read Error: {:?}", e),
                }
                if let Ok(msg) = from_bevy.try_recv() {
                    if let Err(e) = port.write_all(&msg) {
                        error!("Serial Write Error: {:?}", e);
                    }
                }
            }
        });

        app.add_plugins(LogDiagnosticsPlugin::default());
        app.insert_resource(M8Connection {
            rx: from_serial,
            tx: to_serial,
        });
    }
}

impl M8Connection {
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
