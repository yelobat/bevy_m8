# bevy_m8 - Dirtywave M8 for Bevy.

This plugin provides the ability to make the dirtywave M8 accessible as part of
the running Bevy app.

# Building

You can build this project by doing the following:

``` shell
git clone https://github.com/yelobat/bevy_m8
cd bevy_m8
cargo build --release
```

Then you can run the client with the following command:
``` shell
target/release/bevy_m8
```

# Capabilities

## Remote Functionality

This client is controllable remotely. It uses BRP (Bevy Remote Protocol) under the hood which exposes
an API which allows you to simulate key presses.

## Custom Keybindings

The default keybindings can be overridden by inserting the `M8KeyMap` resource back into your
app, but changing the keycodes:

``` rust
use bevy::prelude::*;
use bevy_m8::{M8KeyMap, M8Plugin};

fn main() {
    App::new()
        .add_plugins(M8Plugin::default())
        .insert_resource(
            M8KeyMap::default()
                .with_left_keycode(KeyCode::ArrowLeft)
                .with_right_keycode(KeyCode::ArrowRight)
                .with_down_keycode(KeyCode::ArrowDown)
                .with_up_keycode(KeyCode::ArrowUp)
                .with_edit_keycode(KeyCode::F11)
                .with_option_keycode(KeyCode::ControlLeft)
                .with_select_keycode(KeyCode::ShiftLeft)
                .with_start_keycode(KeyCode::AltLeft),
        )
        .run();
}
```


