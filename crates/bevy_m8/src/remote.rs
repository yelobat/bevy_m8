//! The Dirtywave M8 remote interaction API.

use std::{
    collections::VecDeque,
    net::{IpAddr, Ipv4Addr},
};

use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    remote::{RemotePlugin, http::RemoteHttpPlugin},
};

use crate::{keymap::M8KeyMap, utils::mask_to_keyboard_input};

/// The M8 Events that can be triggered remotely.
#[derive(Event, Reflect, Default)]
#[reflect(Event, Default)]
enum M8Event {
    #[default]
    Disconnect,
    Enable,
    Reset,
    KeyHold(u8),
    KeyPress(u8),
    KeyRelease(u8),
}

/// The M8 Event Queue used to schedule KeyboardInput events to
/// be handled in subsequent frames.
#[derive(Resource, Default)]
struct M8KeyboardEventQueue(VecDeque<KeyboardInput>);

fn input_from_event(
    event: On<M8Event>,
    key_map: Res<M8KeyMap>,
    mut event_queue: ResMut<M8KeyboardEventQueue>,
    mut keyboard_events: MessageWriter<KeyboardInput>,
) {
    match *event {
        M8Event::Disconnect => todo!(),
        M8Event::Enable => todo!(),
        M8Event::Reset => todo!(),
        M8Event::KeyHold(mask) => {
            // TODO If repeated KeyHold events are sent to the same keyboard inputs
            // this could could issues here. Should probably check
            // to see if the keyboard input is already in the queue before
            // adding it back in.
            for keyboard_input in mask_to_keyboard_input(mask, &key_map).iter() {
                keyboard_events.write(keyboard_input.clone());
            }
        }
        M8Event::KeyPress(mask) => {
            for keyboard_input in mask_to_keyboard_input(mask, &key_map).iter() {
                keyboard_events.write(keyboard_input.clone());
                event_queue.0.push_back(KeyboardInput {
                    state: ButtonState::Released,
                    ..keyboard_input.clone()
                });
            }
        }
        M8Event::KeyRelease(mask) => {
            for keyboard_input in mask_to_keyboard_input(mask, &key_map).iter() {
                keyboard_events.write(KeyboardInput {
                    state: ButtonState::Released,
                    ..keyboard_input.clone()
                });
            }
        }
    };
}

fn flush_keyboard_event_queue(
    mut event_queue: ResMut<M8KeyboardEventQueue>,
    mut keyboard_events: MessageWriter<KeyboardInput>,
) {
    while let Some(keyboard_input) = event_queue.0.pop_back() {
        keyboard_events.write(keyboard_input.clone());
    }
}

/// Default port with which bevy_m8 remote functionality
/// runs on.
const DEFAULT_PORT: u16 = 3030;

/// Default address with which bevy_m8 remote functionality
/// runs on.
const DEFAULT_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

/// This plugin provides remote capabilities.
pub struct M8RemotePlugin {
    address: IpAddr,
    port: u16,
}

impl Default for M8RemotePlugin {
    fn default() -> Self {
        Self {
            address: DEFAULT_ADDRESS,
            port: DEFAULT_PORT,
        }
    }
}

#[allow(unused)]
impl M8RemotePlugin {
    pub fn with_address(self, address: impl Into<IpAddr>) -> Self {
        Self {
            address: address.into(),
            ..self
        }
    }

    pub fn with_port(self, port: u16) -> Self {
        Self { port, ..self }
    }
}

impl Plugin for M8RemotePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RemotePlugin::default());
        app.add_plugins(
            RemoteHttpPlugin::default()
                .with_address(self.address)
                .with_port(self.port),
        );
        app.add_observer(input_from_event);
        app.add_systems(Update, flush_keyboard_event_queue);
        app.insert_resource(M8KeyboardEventQueue::default());
        app.register_type::<M8Event>();
    }
}
