//! Dirtywave M8 accessible from within a bevy app.

mod command;
mod serialport;

use bevy::prelude::*;

/// If no port is defined, this is the assigned default one.
const DEFAULT_DIRTYWAVE_M8_PORT: &'static str = "/dev/ttyACM0";

/// The Dirtywave M8 Bevy Plugin.
pub struct DirtywaveM8Plugin(pub String);
impl Plugin for DirtywaveM8Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(serialport::M8SerialPlugin {
            preferred_device: self.0.clone().into(),
        });
    }
}

impl Default for DirtywaveM8Plugin {
    fn default() -> Self {
        Self(DEFAULT_DIRTYWAVE_M8_PORT.into())
    }
}
