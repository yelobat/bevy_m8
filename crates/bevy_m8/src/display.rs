use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::{
    M8UpdateSystems,
    decoder::{M8Command, M8Decoder},
    serial::M8Connection,
};

/// The title used for the Display window.
const TITLE: &'static str = "Bevy M8";

/// The Display Width.
const DISPLAY_WIDTH: f32 = 320.0;

/// The Display Height.
const DISPLAY_HEIGHT: f32 = 340.0;

/// The total number of pixels in the Display.
const DISPLAY_PIXEL_COUNT: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize;

#[derive(Debug, Resource)]
pub struct M8Display(Handle<Image>);

fn setup_display(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(ClearColor(Color::Srgba(Srgba {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    })));

    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: DISPLAY_WIDTH,
                height: DISPLAY_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    let mut image = Image::new_fill(
        Extent3d {
            width: DISPLAY_WIDTH as u32,
            height: DISPLAY_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[16, 16, 24, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::nearest();

    let handle = images.add(image);
    commands.spawn(Sprite {
        image: handle.clone(),
        ..default()
    });

    commands.insert_resource(M8Display(handle.clone()));
}

fn input(mut connection: ResMut<M8Connection>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyR) {
        match connection.send_reset_command() {
            Ok(_) => (),
            Err(err) => warn!("Failed to send Reset Command: {err:?}"),
        };
    } else if input.just_pressed(KeyCode::KeyF) {
        match connection.send(&[b'C', b'\x04']) {
            Ok(_) => (),
            Err(err) => warn!("Failed to send Right Key Press: {err:?}"),
        }
    } else if input.just_released(KeyCode::KeyF) {
        match connection.send(&[b'C', b'\x00']) {
            Ok(_) => (),
            Err(err) => warn!("Failed to send Right Key Release: {err:?}"),
        }
    }
}

fn render(decoder: Res<M8Decoder>, display: Res<M8Display>, mut images: ResMut<Assets<Image>>) {
    if let Some(image) = images.get_mut(&display.0) {
        for command in decoder.command_decoder.commands.iter() {
            match command {
                M8Command::DrawRectangle { pos, size, colour } => {
                    for x in pos.x..pos.x + size.width {
                        for y in pos.y..pos.y + size.height {
                            image.set_color_at(x as u32, y as u32, *colour).unwrap();
                        }
                    }
                }
                M8Command::DrawCharacter {
                    c,
                    pos,
                    foreground,
                    background,
                } => {
                    ()
                },
                M8Command::DrawOscilloscopeWaveform { colour, waveform } => {
                    ()
                }
                M8Command::SystemInfo {
                    hardware_type,
                    major,
                    minor,
                    patch,
                    font_mode,
                } => info!(
                    "Show System Info: HW: {}, Version: {}.{}.{}, {}",
                    hardware_type, major, minor, patch, font_mode
                ),
            }
        }
    }
}

pub struct M8DisplayPlugin;
impl Plugin for M8DisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::AutoVsync,
                mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                title: TITLE.into(),
                ..default()
            }),
            ..default()
        }));

        app.add_systems(Startup, setup_display);
        app.add_systems(Update, input.in_set(M8UpdateSystems::Input));
        app.add_systems(Update, render.in_set(M8UpdateSystems::DisplayRender));
    }
}
