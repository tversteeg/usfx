use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, Stream, SupportedStreamConfig};
use rand::prelude::*;
use rust_music_theory::scale::Direction;
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
const SAMPLE_RATE: u32 = 44_100;

// Beats per minute
const BPM: f32 = 132.0;

/// Manages the audio.
pub struct Audio {
    mixer: Arc<Mutex<usfx::Mixer>>,
    stream: Stream,
}

impl Audio {
    /// Instantiate a new audio object without a generator.
    #[allow(clippy::new_without_default)] //stream doesn't support default
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
                None,
            )
            .expect("could not build output stream");

        let struct_mixer = mixer;
        Self {
            mixer: struct_mixer,
            stream,
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
        self.stream.play().expect("unable to start stream");
    }
}

fn kick(rng: &mut ThreadRng) -> Vec<usfx::Sample> {
    // Combine a short high punch with a longer low bass
    vec![*usfx::Sample::default()
        .volume(0.5)
        .osc_frequency(150)
        .osc_type(usfx::OscillatorType::Triangle)
        .env_attack(0.07)
        .env_decay(0.05)
        .env_sustain(0.9)
        .env_release(rng.gen_range(0.1..0.2))]
}

fn hat() -> Vec<usfx::Sample> {
    // An annoying high chirpy sound
    vec![*usfx::Sample::default()
        .volume(0.2)
        .osc_type(usfx::OscillatorType::Noise)
        .env_attack(0.02)
        .env_decay(0.02)
        .env_sustain(0.7)
        .env_release(0.0)]
}

fn lead(lead_frequencies: &[usize], index: &mut usize) -> Vec<usfx::Sample> {
    *index = (*index + 1) % lead_frequencies.len();

    // The lead synth, frequency is based on the generated scale
    vec![*usfx::Sample::default()
        .volume(0.5)
        .osc_frequency(lead_frequencies[*index])
        .osc_type(usfx::OscillatorType::Square)
        .osc_duty_cycle(usfx::DutyCycle::Eight)
        .env_attack(0.02)
        .env_decay(0.3)
        .env_sustain(0.4)
        .env_release(0.5)
        .dis_crunch(0.3)
        .dis_drive(0.2)]
}

fn generate_lead_frequencies(mut rng: &mut ThreadRng) -> Vec<usize> {
    // Generate a scale for the lead
    let scale = Scale::new(
        ScaleType::HarmonicMinor,
        PitchClass::C,
        4,
        Some(Mode::Phrygian),
        Direction::Ascending,
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
