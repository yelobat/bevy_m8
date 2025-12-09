//! Dirtywave M8 accessible from within a bevy app.

mod command;
mod serialport;
mod slip;

use bevy::prelude::*;

/// The states of operation for which the Plugin will operate.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
enum DirtywaveM8UpdateSystems {
    Input,
    SerialRead,
    DisplayRender,
    Update,
}

/// If no port is defined, this is the assigned default one.
const DEFAULT_DIRTYWAVE_M8_PORT: &'static str = "/dev/ttyACM0";

/// The Dirtywave M8 Bevy Plugin.
pub struct DirtywaveM8Plugin(pub String);
impl Plugin for DirtywaveM8Plugin {
    fn build(&self, app: &mut App) {
        // Configure the M8 Update SystemSet
        app.configure_sets(
            Update,
            (
                DirtywaveM8UpdateSystems::Input,
                DirtywaveM8UpdateSystems::SerialRead,
                DirtywaveM8UpdateSystems::DisplayRender,
                DirtywaveM8UpdateSystems::Update,
            ),
        );

        // Add the Serial Interaction Plugin.
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
