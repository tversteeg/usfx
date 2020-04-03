use cpal::traits::{EventLoopTrait, HostTrait};
use rand::prelude::*;
use rust_music_theory::{
    note::{Notes, PitchClass},
    scale::{Mode, Scale, ScaleType},
};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

// Audio quality
const SAMPLE_RATE: usize = 44_100;

// Beats per minute
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
            mixer: Arc::new(Mutex::new(usfx::Mixer::new(SAMPLE_RATE))),
        }
    }

    /// Play samples.
    pub fn play(&mut self, samples: Vec<usfx::Sample>) {
        let mut mixer = self.mixer.lock().unwrap();
        // Add all the samples to the mixer
        samples.into_iter().for_each(|sample| mixer.play(sample));
    }

    /// Start a thread which will emit the audio.
    pub fn run(&mut self) {
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

        let mixer = self.mixer.clone();

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

fn kick(rng: &mut ThreadRng) -> Vec<usfx::Sample> {
    // Combine a short high punch with a longer low bass
    vec![
        *usfx::Sample::default()
            .osc_frequency(rng.gen_range(155, 165))
            .osc_type(usfx::OscillatorType::Sine)
            .env_attack(0.05)
            .env_decay(0.05)
            .env_sustain(0.5)
            .env_release(0.05)
            .dis_crunch(0.1),
        *usfx::Sample::default()
            .osc_frequency(150)
            .osc_type(usfx::OscillatorType::Sine)
            .env_attack(0.1)
            .env_decay(0.1)
            .env_sustain(0.5)
            .env_release(0.2)
            .dis_crunch(0.2),
    ]
}

fn hat() -> Vec<usfx::Sample> {
    // An annoying high chirpy sound
    vec![*usfx::Sample::default()
        .osc_frequency(2000)
        .osc_type(usfx::OscillatorType::Square)
        .env_attack(0.01)
        .env_decay(0.01)
        .env_sustain(0.5)
        .env_release(0.01)
        .dis_crunch(1.0)]
}

fn lead(lead_frequencies: &[usize], index: &mut usize) -> Vec<usfx::Sample> {
    *index = (*index + 1) % lead_frequencies.len();

    // The lead synth, frequency is based on the generated scale
    vec![*usfx::Sample::default()
        .osc_frequency(lead_frequencies[*index])
        .osc_type(usfx::OscillatorType::Triangle)
        .env_attack(0.01)
        .env_decay(0.1)
        .env_sustain(0.4)
        .env_release(0.3)
        .dis_crunch(1.0)]
}

fn generate_lead_frequencies(mut rng: &mut ThreadRng) -> Vec<usize> {
    // Generate a scale for the lead
    let scale = Scale::new(
        ScaleType::HarmonicMinor,
        PitchClass::C,
        4,
        Some(Mode::Phrygian),
    )
    .unwrap();

    // Get the notes
    let scale_notes = scale.notes();

    // Choose 8 random notes
    (0..8)
        .map(
            |_| match scale_notes.iter().choose(&mut rng).unwrap().pitch_class {
                // Convert the pitch class of the note to a frequency
                PitchClass::C => 262,
                PitchClass::Cs => 277,
                PitchClass::D => 294,
                PitchClass::Ds => 311,
                PitchClass::E => 330,
                PitchClass::F => 349,
                PitchClass::Fs => 370,
                PitchClass::G => 392,
                PitchClass::Gs => 415,
                PitchClass::A => 440,
                PitchClass::As => 466,
                PitchClass::B => 494,
            },
        )
        .collect()
}

fn main() {
    // Spawn a background thread where an audio device is opened with cpal
    let mut audio = Audio::new();
    audio.run();

    // The delay needed to follow the BPM
    let beat_delay_milliseconds = (60.0 / BPM * 1000.0 / 4.0) as u64;

    // Initialize the random number generator
    let mut rng = thread_rng();

    // Procedurally generate frequencies for the lead
    let lead_frequencies = generate_lead_frequencies(&mut rng);

    let mut current_lead = 0;

    // Really ugly way to layout a track
    loop {
        // If we want the music to play at the exact same time it's better to chain the vectors,
        // but having a "random" delay creates a more organic feeling
        audio.play(kick(&mut rng));
        audio.play(hat());

        thread::sleep(Duration::from_millis(beat_delay_milliseconds));

        audio.play(hat());

        thread::sleep(Duration::from_millis(beat_delay_milliseconds));

        audio.play(lead(&lead_frequencies[..], &mut current_lead));
        audio.play(hat());

        thread::sleep(Duration::from_millis(beat_delay_milliseconds));

        audio.play(hat());

        thread::sleep(Duration::from_millis(beat_delay_milliseconds));
    }
}
