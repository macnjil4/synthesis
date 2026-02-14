pub mod filter;
pub mod oscillator;
pub mod voice;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, StreamConfig, SupportedStreamConfig};
use fundsp::audiounit::AudioUnit;

use crate::engine::oscillator::{Waveform, build_oscillator};

/// Initialize the default audio output device and its preferred configuration.
pub fn init_audio_device() -> (Device, SupportedStreamConfig) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output audio device available");
    let supported_config = device
        .default_output_config()
        .expect("no default output config");
    (device, supported_config)
}

/// Build and start an audio output stream from the given graph.
/// The returned Stream is already playing.
pub fn start_stream(
    device: &Device,
    supported_config: &SupportedStreamConfig,
    mut graph: Box<dyn AudioUnit>,
) -> cpal::Stream {
    let sample_rate = supported_config.sample_rate();
    let channels = supported_config.channels() as usize;

    let config = StreamConfig {
        channels: channels as u16,
        sample_rate,
        buffer_size: cpal::BufferSize::Fixed(256),
    };

    graph.set_sample_rate(sample_rate as f64);
    graph.allocate();

    let sample_format = supported_config.sample_format();
    let stream = match sample_format {
        SampleFormat::F32 => build_stream::<f32>(device, &config, graph, channels),
        SampleFormat::I16 => build_stream::<i16>(device, &config, graph, channels),
        SampleFormat::U16 => build_stream::<u16>(device, &config, graph, channels),
        _ => panic!("unsupported sample format: {sample_format}"),
    };

    stream.play().expect("failed to play audio stream");
    stream
}

/// Initialize audio output and play the given waveform for the specified duration.
pub fn play(waveform: Waveform, frequency: f32, amplitude: f32, duration_secs: f32) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output audio device available");

    let supported_config = device
        .default_output_config()
        .expect("no default output config");

    let sample_rate = supported_config.sample_rate();
    let channels = supported_config.channels() as usize;

    let config = StreamConfig {
        channels: channels as u16,
        sample_rate,
        buffer_size: cpal::BufferSize::Fixed(256),
    };

    let mut graph = build_oscillator(waveform, frequency, amplitude);
    graph.set_sample_rate(sample_rate as f64);
    graph.allocate();

    println!(
        "Playing {} wave at {} Hz (amplitude {}) for {}s",
        waveform, frequency, amplitude, duration_secs
    );
    println!(
        "Audio: {} Hz sample rate, {} channels, 256 sample buffer",
        sample_rate, channels
    );

    let sample_format = supported_config.sample_format();

    let stream = match sample_format {
        SampleFormat::F32 => build_stream::<f32>(&device, &config, graph, channels),
        SampleFormat::I16 => build_stream::<i16>(&device, &config, graph, channels),
        SampleFormat::U16 => build_stream::<u16>(&device, &config, graph, channels),
        _ => panic!("unsupported sample format: {sample_format}"),
    };

    stream.play().expect("failed to play audio stream");

    std::thread::sleep(std::time::Duration::from_secs_f32(duration_secs));

    drop(stream);
    println!("Done.");
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    mut graph: Box<dyn AudioUnit>,
    channels: usize,
) -> cpal::Stream
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let frames = data.len() / channels;
                for frame_idx in 0..frames {
                    let (l, r) = graph.get_stereo();
                    let left: T = T::from_sample(l);
                    let right: T = T::from_sample(r);

                    let base = frame_idx * channels;
                    data[base] = left;
                    if channels > 1 {
                        data[base + 1] = right;
                    }
                    // Fill remaining channels with left signal
                    for ch in 2..channels {
                        data[base + ch] = left;
                    }
                }
            },
            |err| eprintln!("audio stream error: {err}"),
            None,
        )
        .expect("failed to build output stream")
}
