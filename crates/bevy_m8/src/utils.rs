//! This file provides general purpose functions.
use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
};

use crate::{
    display::{
        M8_DOWN, M8_EDIT, M8_KEY_COUNT, M8_LEFT, M8_OPTION, M8_RIGHT, M8_SELECT, M8_START, M8_UP,
    },
    keymap::M8KeyMap,
};

pub fn keycode_to_mask(keycodes: Vec<KeyCode>, key_map: &Res<M8KeyMap>) -> u8 {
    let mut mask = 0;
    for &keycode in keycodes.iter() {
        if keycode == key_map.edit_keycode() {
            mask |= M8_EDIT;
        } else if keycode == key_map.option_keycode() {
            mask |= M8_OPTION;
        } else if keycode == key_map.right_keycode() {
            mask |= M8_RIGHT;
        } else if keycode == key_map.left_keycode() {
            mask |= M8_LEFT;
        } else if keycode == key_map.down_keycode() {
            mask |= M8_DOWN;
        } else if keycode == key_map.up_keycode() {
            mask |= M8_UP;
        } else if keycode == key_map.select_keycode() {
            mask |= M8_SELECT;
        } else if keycode == key_map.start_keycode() {
            mask |= M8_START;
        }
    }
    mask
}

pub fn mask_to_keyboard_input(mask: u8, key_map: &Res<M8KeyMap>) -> Vec<KeyboardInput> {
    let mut keyboard_inputs = Vec::with_capacity(M8_KEY_COUNT);

    if (mask & M8_EDIT) != 0 {
        keyboard_inputs.push(KeyboardInput {
            key_code: key_map.edit_keycode(),
            logical_key: Key::Character("".into()),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    if (mask & M8_OPTION) != 0 {
        keyboard_inputs.push(KeyboardInput {
            key_code: key_map.option_keycode(),
            logical_key: Key::Character("".into()),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    if (mask & M8_RIGHT) != 0 {
        keyboard_inputs.push(KeyboardInput {
            key_code: key_map.right_keycode(),
            logical_key: Key::Character("".into()),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    if (mask & M8_LEFT) != 0 {
        keyboard_inputs.push(KeyboardInput {
            key_code: key_map.left_keycode(),
            logical_key: Key::Character("".into()),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    if (mask & M8_DOWN) != 0 {
        keyboard_inputs.push(KeyboardInput {
            key_code: key_map.down_keycode(),
            logical_key: Key::Character("".into()),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    if (mask & M8_UP) != 0 {
        keyboard_inputs.push(KeyboardInput {
            key_code: key_map.up_keycode(),
            logical_key: Key::Character("".into()),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    if (mask & M8_SELECT) != 0 {
        keyboard_inputs.push(KeyboardInput {
            key_code: key_map.select_keycode(),
            logical_key: Key::Character("".into()),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    if (mask & M8_START) != 0 {
        keyboard_inputs.push(KeyboardInput {
            key_code: key_map.start_keycode(),
            logical_key: Key::Character("".into()),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    keyboard_inputs
}
