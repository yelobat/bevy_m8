//! This file provides key map functionality.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

/// The Key map resource for defining
/// the key bindings for interaction with
/// the M8.
#[allow(unused)]
#[derive(Resource)]
pub struct M8KeyMap {
    edit: KeyCode,
    option: KeyCode,
    right: KeyCode,
    left: KeyCode,
    up: KeyCode,
    down: KeyCode,
    select: KeyCode,
    start: KeyCode,
}

impl Default for M8KeyMap {
    fn default() -> Self {
        Self {
            edit: KeyCode::KeyZ,
            option: KeyCode::KeyX,
            right: KeyCode::KeyF,
            left: KeyCode::KeyB,
            up: KeyCode::KeyP,
            down: KeyCode::KeyN,
            select: KeyCode::ControlLeft,
            start: KeyCode::ShiftLeft,
        }
    }
}

#[allow(unused)]
impl M8KeyMap {
    pub fn edit_keycode(&self) -> KeyCode {
        self.edit
    }

    pub fn option_keycode(&self) -> KeyCode {
        self.option
    }

    pub fn right_keycode(&self) -> KeyCode {
        self.right
    }

    pub fn left_keycode(&self) -> KeyCode {
        self.left
    }

    pub fn up_keycode(&self) -> KeyCode {
        self.up
    }

    pub fn down_keycode(&self) -> KeyCode {
        self.down
    }

    pub fn select_keycode(&self) -> KeyCode {
        self.select
    }

    pub fn start_keycode(&self) -> KeyCode {
        self.start
    }
    pub fn with_edit_keycode(self, keycode: KeyCode) -> Self {
        Self {
            edit: keycode,
            ..self
        }
    }

    pub fn with_option_keycode(self, keycode: KeyCode) -> Self {
        Self {
            option: keycode,
            ..self
        }
    }

    pub fn with_right_keycode(self, keycode: KeyCode) -> Self {
        Self {
            right: keycode,
            ..self
        }
    }

    pub fn with_left_keycode(self, keycode: KeyCode) -> Self {
        Self {
            left: keycode,
            ..self
        }
    }

    pub fn with_up_keycode(self, keycode: KeyCode) -> Self {
        Self {
            up: keycode,
            ..self
        }
    }

    pub fn with_down_keycode(self, keycode: KeyCode) -> Self {
        Self {
            down: keycode,
            ..self
        }
    }

    pub fn with_select_keycode(self, keycode: KeyCode) -> Self {
        Self {
            select: keycode,
            ..self
        }
    }

    pub fn with_start_keycode(self, keycode: KeyCode) -> Self {
        Self {
            start: keycode,
            ..self
        }
    }
}

/// The Key Map plugin, providing a means
/// of controlling the key bindings used
/// in the app.
pub struct M8KeyMapPlugin;

impl Plugin for M8KeyMapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(M8KeyMap::default());
    }
}
