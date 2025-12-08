use bevy::prelude::*;
use bevy_dirtywave_m8::DirtywaveM8Plugin;

fn main() {
    App::new().add_plugins(DirtywaveM8Plugin::default()).run();
}
