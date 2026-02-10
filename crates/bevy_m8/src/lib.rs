//! Dirtywave M8 accessible from within a bevy app.

mod assets;
mod audio;
mod decoder;
mod display;
mod keymap;
mod remote;
mod serial;
mod utils;

use bevy::prelude::*;
pub use keymap::M8KeyMap;

/// Dirtywave M8 accessible from within a bevy app.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
pub enum M8LoadingState {
    #[default]
    Loading,
    Running,
}

/// If no port is defined, this is the assigned default one.
const DEFAULT_M8_PORT: &'static str = "/dev/ttyACM0";

/// The M8 Bevy Plugin.
pub struct M8Plugin(pub String);
impl Plugin for M8Plugin {
    fn build(&self, app: &mut App) {
        // Add the Serial Interaction Plugin.
        app.add_plugins((
            serial::M8SerialPlugin {
                preferred_device: self.0.clone().into(),
            },
            display::M8DisplayPlugin,
            remote::M8RemotePlugin::default(),
            keymap::M8KeyMapPlugin,
            assets::M8AssetsPlugin,
            audio::M8AudioPlugin,
        ));
    }
}

impl Default for M8Plugin {
    fn default() -> Self {
        Self(DEFAULT_M8_PORT.into())
    }
}
