use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::decoder::M8Command;

/// The title used for the Display window.
const TITLE: &'static str = "Bevy M8";

/// The Display Width.
const DISPLAY_WIDTH: f32 = 320.0;

/// The Display Height.
const DISPLAY_HEIGHT: f32 = 240.0;

/// The total number of pixels in the Display.
const DISPLAY_PIXEL_COUNT: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize;

#[derive(Debug, Resource)]
pub struct M8Display(Handle<Image>);

impl M8Display {
    fn process_command(image: &mut Image, command: &M8Command) {
        match command {
            M8Command::DrawRectangle { pos, size, colour } => {
                for x in pos.x..pos.x + size.width {
                    for y in pos.y..pos.y + size.height {
                        image.set_color_at(x.into(), y.into(), *colour).unwrap();
                    }
                }
            }
            M8Command::DrawCharacter {
                c,
                pos,
                foreground,
                background,
            } => (),
            M8Command::DrawOscilloscopeWaveform { colour, waveform } => (),
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
        &[0, 0, 0, 255],
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
    }
}
