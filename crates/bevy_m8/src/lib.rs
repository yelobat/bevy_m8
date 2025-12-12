//! Dirtywave M8 accessible from within a bevy app.

mod decoder;
mod display;
mod serial;

use bevy::prelude::*;

/// The states of operation for which the Plugin will operate.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
enum M8UpdateSystems {
    Input,
    SerialRead,
    Decode,
    DisplayRender,
    Update,
}

/// If no port is defined, this is the assigned default one.
const DEFAULT_M8_PORT: &'static str = "/dev/ttyACM0";

/// The M8 Bevy Plugin.
pub struct M8Plugin(pub String);
impl Plugin for M8Plugin {
    fn build(&self, app: &mut App) {
        // Configure the M8 Update SystemSet
        app.configure_sets(
            Update,
            (
                M8UpdateSystems::Input,
                M8UpdateSystems::SerialRead,
                M8UpdateSystems::Decode,
                M8UpdateSystems::DisplayRender,
                M8UpdateSystems::Update,
            )
                .chain(),
        );

        // Add the Serial Interaction Plugin.
        app.add_plugins((
            serial::M8SerialPlugin {
                preferred_device: self.0.clone().into(),
            },
            display::M8DisplayPlugin,
            decoder::M8DecoderPlugin,
        ));
    }
}

impl Default for M8Plugin {
    fn default() -> Self {
        Self(DEFAULT_M8_PORT.into())
    }
}
