use cpal::traits::{EventLoopTrait, HostTrait};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

/// Manages the audio.
#[derive(Default)]
pub struct Audio {
    mixer: Arc<Mutex<usfx::Mixer>>,
}

impl Audio {
    /// Instantiate a new audio object without a generator.
    pub fn new() -> Self {
        Self {
            mixer: Arc::new(Mutex::new(usfx::Mixer::default())),
        }
    }

    /// Play samples.
    pub fn play(&mut self, samples: Vec<usfx::Generator>) {
        let mut mixer = self.mixer.lock().unwrap();
        // Add all the samples to the mixer
        samples.into_iter().for_each(|sample| mixer.play(sample));
    }

    /// Start a thread which will emit the audio.
    pub fn run(&mut self) {
        let mixer = self.mixer.clone();

        thread::spawn(|| {
            // Setup the audio system
            let host = cpal::default_host();
            let event_loop = host.event_loop();

            let device = host
                .default_output_device()
                .expect("no output device available");

            // This is the only format usfx supports
            let format = cpal::Format {
                channels: 1,
                sample_rate: cpal::SampleRate(22_050),
                data_type: cpal::SampleFormat::F32,
            };

            let stream_id = event_loop
                .build_output_stream(&device, &format)
                .expect("could not build output stream");

            event_loop
                .play_stream(stream_id)
                .expect("could not play stream");

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
    // Create a low sample with a square wave
    let sample1 = usfx::Sample::default()
        .osc_frequency(441)
        .sample_rate(22_050)
        .build::<usfx::SquareWave>();
    // Create a higher sample with a sine wave
    let sample2 = usfx::Sample::default()
        .osc_frequency(882)
        .sample_rate(22_050)
        .build::<usfx::SineWave>();

    let mut audio = Audio::new();

    // Play the samples
    audio.play(vec![sample1, sample2]);

    // Spawn a background thread where an audio device is opened with cpal
    audio.run();

    thread::sleep(Duration::from_millis(1_000));
}
