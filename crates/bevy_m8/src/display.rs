use std::ops::Add;

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    math::{U16Vec2, u16vec2},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::WindowResolution,
};

use crate::{
    M8LoadingState,
    assets::M8Assets,
    decoder::{M8Command, Position, Size},
    serial::M8Connection,
};

pub const DISPLAY_WIDTH: u32 = 320;
pub const DISPLAY_HEIGHT: u32 = 240;

/// The title used for the Display window.
const TITLE: &'static str = "Bevy M8";

#[derive(Resource)]
pub struct M8Display {
    display: Handle<Image>,
    background: Color,
}

fn setup_display(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: DISPLAY_WIDTH,
            height: DISPLAY_HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::nearest();

    let handle = images.add(image);
    commands.insert_resource(M8Display {
        display: handle.clone(),
        background: Color::default(),
    });
    commands.spawn(Sprite {
        image: handle.clone(),
        ..default()
    });

    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: DISPLAY_WIDTH as f32,
                height: DISPLAY_HEIGHT as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn draw_rectangle(display: &mut Image, pos: Position, size: Size, colour: Color) {
    for y in pos.y..pos.y + size.y {
        for x in pos.x..pos.x + size.x {
            if x < DISPLAY_WIDTH as u16 && y < DISPLAY_HEIGHT as u16 {
                display.set_color_at(x.into(), y.into(), colour).unwrap();
            }
        }
    }
}

fn draw_character(
    display: &mut Image,
    font: &Image,
    c: u8,
    pos: Position,
    foreground: Color,
    background: Color,
) {
    const GLYPH_WIDTH: u32 = 5;
    const GLYPH_HEIGHT: u32 = 7;
    const TEXT_OFFSET_Y: u16 = 3;
    if c == 32 {
        draw_rectangle(
            display,
            pos.add(u16vec2(0, TEXT_OFFSET_Y)),
            U16Vec2::new(GLYPH_WIDTH as u16, GLYPH_HEIGHT as u16),
            background,
        );
        return;
    }

    let id = c.saturating_sub(33) as u32;
    let src_x_start = id * GLYPH_WIDTH;

    for y in 0..GLYPH_HEIGHT {
        for x in 0..GLYPH_WIDTH {
            let is_on = font
                .get_color_at(src_x_start + x, y)
                .map(|p| p.luminance() > 0.5)
                .unwrap_or(false);

            let final_colour = if is_on {
                foreground
            } else {
                background
            };

            let dx = pos.x as u32 + x;
            let dy = pos.y as u32 + y + TEXT_OFFSET_Y as u32;

            if dx < DISPLAY_WIDTH && dy < DISPLAY_HEIGHT {
                if is_on {
                    display.set_color_at(dx, dy, final_colour).unwrap();
                } else if foreground != background {
                    display.set_color_at(dx, dy, background).unwrap();
                }
            }
        }
    }
}

fn draw_waveform(display: &mut Image, colour: Color, waveform: Vec<u8>, background: Color) {
    const WAVEFORM_MAX_HEIGHT: u32 = 16;
    let start_x = 0;

    for x in start_x..DISPLAY_WIDTH {
        for y in 0..=WAVEFORM_MAX_HEIGHT {
            display.set_color_at(x, y, background).unwrap();
        }
    }

    let draw_start_x = 0;
    for (i, &val) in waveform.iter().enumerate() {
        let clamped_y = (val as u32).min(WAVEFORM_MAX_HEIGHT);
        let x = draw_start_x + i as u32;

        if x < DISPLAY_WIDTH {
            display.set_color_at(x, clamped_y, colour).unwrap();
        }
    }
}

fn render(
    connection: Res<M8Connection>,
    mut display: ResMut<M8Display>,
    m8_assets: Res<M8Assets>,
    mut images: ResMut<Assets<Image>>,
) {
    let images_ptr: *mut Assets<Image> = &mut *images;
    unsafe {
        let display_image = (*images_ptr).get_mut(&display.display);
        let font = (*images_ptr).get(&m8_assets.font_small);

        if let (Some(display_image), Some(font)) = (display_image, font) {
            while let Ok(cmd) = connection.rx.try_recv() {
                match cmd {
                    M8Command::DrawRectangle { pos, size, colour } => {
                        if pos.x == 0
                            && pos.y <= 0
                            && size.x == DISPLAY_WIDTH as u16
                            && size.y >= DISPLAY_HEIGHT as u16
                        {
                            display.background = colour;
                        }

                        draw_rectangle(display_image, pos, size, colour);
                    }
                    M8Command::DrawCharacter {
                        c,
                        pos,
                        foreground,
                        background,
                    } => {
                        draw_character(display_image, font, c, pos, foreground, background);
                    }
                    M8Command::DrawOscilloscopeWaveform { colour, waveform } => {
                        draw_waveform(display_image, colour, waveform, display.background);
                    }
                    M8Command::SystemInfo {
                        hardware_type,
                        major,
                        minor,
                        patch,
                        font_mode,
                    } => (),
                }
            }
        }
    }
}

const M8_EDIT: u8 = 1 << 0;
const M8_OPTION: u8 = 1 << 1;
const M8_RIGHT: u8 = 1 << 2;
const M8_START: u8 = 1 << 3;
const M8_SELECT: u8 = 1 << 4;
const M8_DOWN: u8 = 1 << 5;
const M8_UP: u8 = 1 << 6;
const M8_LEFT: u8 = 1 << 7;

fn input(keys: Res<ButtonInput<KeyCode>>, connection: Res<M8Connection>, mut prev_mask: Local<u8>) {
    if keys.just_pressed(KeyCode::KeyE) {
        let _ = connection.tx.send(vec![b'E']);
    }

    if keys.just_pressed(KeyCode::KeyR) {
        let _ = connection.tx.send(vec![b'R']);
    }

    let mut mask: u8 = 0;
    if keys.pressed(KeyCode::KeyZ) {
        mask |= M8_EDIT;
    }
    if keys.pressed(KeyCode::KeyX) {
        mask |= M8_OPTION;
    }
    if keys.pressed(KeyCode::KeyF) {
        mask |= M8_RIGHT;
    }
    if keys.pressed(KeyCode::KeyB) {
        mask |= M8_LEFT;
    }
    if keys.pressed(KeyCode::KeyN) {
        mask |= M8_DOWN;
    }
    if keys.pressed(KeyCode::KeyP) {
        mask |= M8_UP;
    }
    if keys.pressed(KeyCode::ControlLeft) {
        mask |= M8_SELECT;
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        mask |= M8_START;
    }

    if mask != *prev_mask {
        let _ = connection.tx.send(vec![b'C', mask]);
        *prev_mask = mask;
    }
}

pub struct M8DisplayPlugin;
impl Plugin for M8DisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::AutoVsync,
                //mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                resolution: WindowResolution::new(DISPLAY_WIDTH, DISPLAY_HEIGHT),
                title: TITLE.into(),
                ..default()
            }),
            ..default()
        }));

        app.add_systems(Startup, setup_display);
        app.add_systems(Update, render.run_if(in_state(M8LoadingState::Running)));
        app.add_systems(Update, input.run_if(in_state(M8LoadingState::Running)));
    }
}
