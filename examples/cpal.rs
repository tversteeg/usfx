use cpal::traits::{EventLoopTrait, HostTrait};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const SAMPLE_RATE: usize = 44_100;

/// Manages the audio.
#[derive(Default)]
pub struct Audio {
    mixer: Arc<Mutex<usfx::Mixer>>,
}

impl Audio {
    /// Instantiate a new audio object without a generator.
    pub fn new() -> Self {
        Self {
            mixer: Arc::new(Mutex::new(usfx::Mixer::new(SAMPLE_RATE))),
        }
    }

    /// Play samples.
    pub fn play(&mut self, sample: &usfx::Sample) {
        // Add the sample to the mixer
        self.mixer.lock().unwrap().play(sample);
    }

    /// Start a thread which will emit the audio.
    pub fn run(&mut self) {
        let mixer = self.mixer.clone();

        // Setup the audio system
        let host = cpal::default_host();
        let event_loop = host.event_loop();

        let device = host
            .default_output_device()
            .expect("no output device available");

        let format = cpal::Format {
            channels: 1,
            sample_rate: cpal::SampleRate(SAMPLE_RATE as u32),
            data_type: cpal::SampleFormat::F32,
        };

        let stream_id = event_loop
            .build_output_stream(&device, &format)
            .expect("could not build output stream");

        event_loop
            .play_stream(stream_id)
            .expect("could not play stream");

        thread::spawn(move || {
            event_loop.run(move |stream_id, stream_result| {
                let stream_data = match stream_result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                        return;
                    }
                };

                match stream_data {
                    cpal::StreamData::Output {
                        buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                    } => mixer.lock().unwrap().generate(&mut buffer),
                    _ => panic!("output type buffer can not be used"),
                }
            });
        });
    }
}

fn main() {
    let mut audio = Audio::new();

    // Play a low sample with a square wave
    audio.play(
        usfx::Sample::default()
            .osc_frequency(1000.0)
            .osc_type(usfx::Oscillator::Sine)
            .env_attack(0.1)
            .env_decay(0.1)
            .env_sustain(0.5)
            .env_release(0.1)
            .sample_rate(SAMPLE_RATE),
    );

    // Spawn a background thread where an audio device is opened with cpal
    audio.run();

    thread::sleep(Duration::from_millis(3_000));
}
