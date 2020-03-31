use cpal::traits::{EventLoopTrait, HostTrait};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const SAMPLE_RATE: usize = 44_100;
const BPM: f32 = 132.0;

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

fn kick() -> Vec<usfx::Generator> {
    // Combine a short high punch with a longer low bass
    vec![
        usfx::Sample::default()
            .osc_frequency(160)
            .env_attack(0.05)
            .env_decay(0.05)
            .env_sustain(0.5)
            .env_release(0.05)
            .sample_rate(SAMPLE_RATE)
            .build::<usfx::SineWave>(),
        usfx::Sample::default()
            .osc_frequency(150)
            .env_attack(0.1)
            .env_decay(0.1)
            .env_sustain(0.5)
            .env_release(0.2)
            .sample_rate(SAMPLE_RATE)
            .build::<usfx::SineWave>(),
    ]
}

fn hat() -> Vec<usfx::Generator> {
    // An annoying high chirpy sound
    vec![usfx::Sample::default()
        .osc_frequency(2000)
        .env_attack(0.01)
        .env_decay(0.01)
        .env_sustain(0.5)
        .env_release(0.01)
        .sample_rate(SAMPLE_RATE)
        .build::<usfx::SquareWave>()]
}

fn main() {
    let mut audio = Audio::new();

    let beat_delay_milliseconds = (60.0 / BPM * 1000.0 / 4.0) as u64;

    // Spawn a background thread where an audio device is opened with cpal
    audio.run();

    // Really ugly way to layout a track
    loop {
        // If we want the music to play at the exact same time it's better to chain the vectors,
        // but having a "random" delay creates a more organic feeling
        audio.play(kick());
        audio.play(hat());

        thread::sleep(Duration::from_millis(beat_delay_milliseconds));

        audio.play(hat());

        thread::sleep(Duration::from_millis(beat_delay_milliseconds));

        audio.play(hat());

        thread::sleep(Duration::from_millis(beat_delay_milliseconds));

        audio.play(hat());

        thread::sleep(Duration::from_millis(beat_delay_milliseconds));
    }
}
