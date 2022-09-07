use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, Stream, SupportedStreamConfig};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const SAMPLE_RATE: u32 = 44_100;

/// Manages the audio.
pub struct Audio {
    mixer: Arc<Mutex<usfx::Mixer>>,
    stream: Stream,
}

impl Audio {
    /// Instantiate a new audio object without a generator.
    pub fn new() -> Self {
        let mixer = Arc::new(Mutex::new(usfx::Mixer::new(SAMPLE_RATE as usize)));
        // Setup the audio system
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("no output device available");

        let config = device
            .supported_output_configs()
            .expect("no output configs available")
            .find(|config| config.sample_format() == SampleFormat::F32);

        if config.is_none() {
            panic!("no F32 config available");
        }

        let config = config.unwrap();

        if config.min_sample_rate() > SampleRate(SAMPLE_RATE)
            || config.max_sample_rate() < SampleRate(SAMPLE_RATE)
        {
            panic!("44100 Hz not supported");
        }

        let format = SupportedStreamConfig::new(
            config.channels(),
            SampleRate(SAMPLE_RATE),
            config.buffer_size().clone(),
            SampleFormat::F32,
        );

        let stream_mixer = mixer.clone();

        let stream = device
            .build_output_stream::<f32, _, _>(
                &format.config(),
                move |data, _| stream_mixer.lock().unwrap().generate(data),
                |err| eprintln!("cpal error: {:?}", err),
            )
            .expect("could not build output stream");

        let struct_mixer = mixer.clone();
        Self {
            mixer: struct_mixer,
            stream,
        }
    }

    /// Play samples.
    pub fn play(&mut self, sample: usfx::Sample) {
        // Add the sample to the mixer
        self.mixer.lock().unwrap().play(sample);
    }

    /// Start a thread which will emit the audio.
    pub fn run(&mut self) {
        self.stream.play().expect("unable to start stream");
    }
}

fn main() {
    let mut audio = Audio::new();

    let mut sample = usfx::Sample::default();
    sample.osc_frequency(1000);
    sample.osc_type(usfx::OscillatorType::Sine);
    sample.env_attack(0.1);
    sample.env_decay(0.1);
    sample.env_sustain(0.5);
    sample.env_release(0.5);
    sample.dis_crunch(0.2);

    // Play a low sample with a square wave
    audio.play(sample);

    // Spawn a background thread where an audio device is opened with cpal
    audio.run();

    thread::sleep(Duration::from_millis(3_000));
}
