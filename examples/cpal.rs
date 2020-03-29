use cpal::traits::{EventLoopTrait, HostTrait};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

/// Manages the audio.
pub struct Audio {
    sample: Arc<Mutex<Option<usfx::Generator>>>,
}

impl Audio {
    /// Instantiate a new audio object without a generator.
    pub fn new() -> Self {
        Self {
            sample: Arc::new(Mutex::new(None)),
        }
    }

    /// Play a sample.
    pub fn play(&mut self, new: usfx::Generator) {
        let mut sample = self.sample.lock().unwrap();
        *sample = Some(new);
    }

    /// Start a thread which will emit the audio.
    pub fn run(&mut self) {
        let sample = self.sample.clone();

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
                sample_rate: cpal::SampleRate(44_100),
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
                    } => match *sample.lock().unwrap() {
                        Some(ref mut sample) => sample.generate(&mut buffer),
                        None => {
                            for elem in buffer.iter_mut() {
                                *elem = 0.0;
                            }
                        }
                    },
                    _ => panic!("output type buffer can not be used"),
                }
            });
        });
    }
}

fn main() {
    let sample = usfx::Sample::default()
        .osc_frequency(441)
        .sample_rate(44_100)
        .build::<usfx::TriangleWave>();

    let mut audio = Audio::new();

    // Spawn a background thread where an audio device is opened with cpal
    audio.run();

    // Play the sample
    audio.play(sample);

    thread::sleep(Duration::from_millis(5_000));
}
