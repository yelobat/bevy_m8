//! This file provides audio capabilities.

use bevy::prelude::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::bounded;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Stores the audio input and output streams.
#[derive(Resource)]
struct M8StreamResource {
    _input: cpal::Stream,
    _output: cpal::Stream,
}

/// Error that can occur during audio processing.
#[derive(Resource, Clone)]
struct M8AudioError(Arc<AtomicBool>);

fn setup_m8_audio(world: &mut World) {
    let host = cpal::default_host();
    let error = world.resource::<M8AudioError>().0.clone();

    let input_device = host.input_devices().unwrap().find(|x| {
        x.description()
            .map(|description| description.name().contains("M8"))
            .unwrap_or(false)
    });
    let output_device = host.default_output_device().expect("No output device!");

    if let Some(input_device) = input_device {
        let input_config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();
        let output_config: cpal::StreamConfig =
            output_device.default_output_config().unwrap().into();

        let (tx, rx) = bounded::<f32>(8820);

        let error_input = error.clone();
        let input_stream = input_device
            .build_input_stream(
                &input_config,
                move |data: &[f32], _| {
                    for &sample in data {
                        let _ = tx.try_send(sample);
                    }
                },
                move |err| {
                    error!("M8 Audio Input Error: {:?}", err);
                    error_input.store(true, Ordering::SeqCst);
                },
                None,
            )
            .unwrap();

        let error_output = error.clone();
        let output_stream = output_device
            .build_output_stream(
                &output_config,
                move |data: &mut [f32], _| {
                    for sample in data.iter_mut() {
                        *sample = rx.try_recv().unwrap_or(0.0);
                    }
                },
                move |err| {
                    error!("Audio Output Error: {:?}", err);
                    error_output.store(true, Ordering::SeqCst);
                },
                None,
            )
            .unwrap();

        input_stream.play().unwrap();
        output_stream.play().unwrap();

        world.insert_non_send_resource(M8StreamResource {
            _input: input_stream,
            _output: output_stream,
        });

        error.store(false, Ordering::SeqCst);
        info!("M8 Audio Stream Started.");
    }
}

fn recover_m8_audio(world: &mut World) {
    let error = world.resource::<M8AudioError>().0.clone();

    if error.load(Ordering::SeqCst) {
        warn!("Attempting to recover M8 audio stream...");
        world.remove_non_send_resource::<M8StreamResource>();
        setup_m8_audio(world);
    }
}

/// Dirtywave M8 Audio plugin.
pub struct M8AudioPlugin;
impl Plugin for M8AudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(M8AudioError(Arc::new(AtomicBool::new(false))));
        setup_m8_audio(app.world_mut());
        app.add_systems(Update, recover_m8_audio);
    }
}
