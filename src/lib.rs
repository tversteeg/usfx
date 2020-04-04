//! # Title
//!
//! Short description.
//!
//! Long description.
//!
//! ## Example
//! ```rust
//! # use usfx::*;
//! ```

// Test the code in README.md
#[cfg(test)]
doc_comment::doctest!("../README.md");

mod effects;
mod envelope;
mod oscillator;

use effects::{distortion::Distortion, Effect};
use envelope::{Envelope, State};
use oscillator::Oscillator;
pub use oscillator::OscillatorType;
use std::{cell::RefCell, collections::HashMap};

/// Audio sample that procedurally generates it's sound.
///
/// Plug this into the [`Mixer`] object to play the sound.
///
/// ```rust
/// // Generate a sine wave at 2khz
/// let mut sine_wave = usfx::Sample::default();
/// sine_wave.osc_frequency(2000);
/// sine_wave.osc_type(usfx::OscillatorType::Sine);
///
/// // Add it to the mixer
/// let mut mixer = usfx::Mixer::default();
/// mixer.play(sine_wave);
///
/// // Plug it into a audio library, see the examples for a cpal & SDL2 implementation
/// // ...
/// // Call the generator to get a buffer for the audio library
/// # let mut buffer = [0.0];
/// mixer.generate(&mut buffer);
/// ```
///
/// [`Generator`]: struct.Generator.html
#[derive(Debug, Copy, Clone)]
pub struct Sample {
    volume: f32,
    osc_frequency: usize,
    osc_type: OscillatorType,
    env_attack: f32,
    env_decay: f32,
    env_release: f32,
    env_sustain: f32,
    dis_crunch: Option<f32>,
    dis_drive: Option<f32>,
}

impl Default for Sample {
    /// The default is a sinewave of 441 hz.
    fn default() -> Self {
        Self {
            volume: 1.0,
            osc_frequency: 441,
            osc_type: OscillatorType::Sine,
            env_attack: 0.01,
            env_decay: 0.1,
            env_sustain: 0.5,
            env_release: 0.5,
            dis_crunch: None,
            dis_drive: None,
        }
    }
}

impl Sample {
    /// Set the volume which is a multiplier of the result.
    ///
    /// A range from 0.0-1.0 will result in proper behavior, but you can experiment with other
    /// values.
    pub fn volume(&mut self, volume: f32) -> &mut Self {
        self.volume = volume;

        self
    }

    /// Set the frequency of the oscillator in hertz.
    ///
    /// A range from 1-20000 is allowed.
    pub fn osc_frequency(&mut self, frequency: usize) -> &mut Self {
        self.osc_frequency = frequency;

        self
    }

    /// Set the type of the oscillator.
    ///
    /// See the [`OscillatorType`] enum for supported wave types.
    ///
    /// [`OscillatorType`]: enum.OscillatorType.html
    pub fn osc_type(&mut self, oscillator: OscillatorType) -> &mut Self {
        self.osc_type = oscillator;

        self
    }

    /// Set the time until the first envelope slope reaches it's maximum height.
    ///
    /// A range from 0.0-1.0 will result in proper behavior, but you can experiment with other
    /// values.
    pub fn env_attack(&mut self, attack: f32) -> &mut Self {
        self.env_attack = attack;

        self
    }

    /// Set the time it takes from the maximum height to go into the main plateau.
    ///
    /// A range from 0.0-1.0 will result in proper behavior, but you can experiment with other
    /// values.
    pub fn env_decay(&mut self, decay: f32) -> &mut Self {
        self.env_decay = decay;

        self
    }

    /// Set the height of the main plateau.
    ///
    /// A range from 0.0-1.0 will result in proper behavior, but you can experiment with other
    /// values.
    pub fn env_sustain(&mut self, sustain: f32) -> &mut Self {
        self.env_sustain = sustain;

        self
    }

    /// Set the time it takes to go from the end of the plateau to zero.
    ///
    /// A range from 0.0-1.0 will result in proper behavior, but you can experiment with other
    /// values.
    pub fn env_release(&mut self, release: f32) -> &mut Self {
        self.env_release = release;

        self
    }

    /// Overdrive that adds hard clipping.
    ///
    /// A range from 0.0-1.0 will result in proper behavior, but you can experiment with other
    /// values.
    pub fn dis_crunch(&mut self, crunch: f32) -> &mut Self {
        self.dis_crunch = Some(crunch);

        self
    }

    /// Overdrive with soft clipping.
    ///
    /// A range from 0.0-1.0 will result in proper behavior, but you can experiment with other
    /// values.
    pub fn dis_drive(&mut self, drive: f32) -> &mut Self {
        self.dis_drive = Some(drive);

        self
    }
}

/// Convert samples with PCM.
///
/// This struct is created by [`Sample`].
/// You can use this generator directly or plug it into a [`Mixer`] object.
///
/// [`Sample`]: struct.Sample.html
/// [`Mixer`]: struct.Mixer.html
#[derive(Debug)]
struct Generator {
    /// Whether we are finished running the sample.
    pub(crate) finished: bool,
    /// The total offset.
    offset: usize,
    /// Multiplier of the result.
    volume: f32,

    /// The oscillator, because it's a trait it has to be boxed.
    oscillator: Oscillator,
    /// The ADSR envelope.
    envelope: Envelope,

    /// Distortion effect.
    distortion: Option<Distortion>,
}

impl Generator {
    /// Generate the sound for the sample.
    fn run(&mut self, mut output: &mut [f32]) {
        // Run the oscillator
        self.oscillator.generate(&mut output, self.offset);

        // Apply the ADSR and set the state if we're finished or not
        if self.envelope.apply(&mut output, self.offset) == State::Done {
            self.finished = true;
        }

        // Apply the distortion
        if let Some(distortion) = &mut self.distortion {
            distortion.apply(&mut output, self.offset);
        }

        // Apply the volume
        if self.volume != 1.0 {
            output.iter_mut().for_each(|tone| *tone *= self.volume);
        }

        self.offset += output.len();
    }
}

/// Manage samples and mix the volume output of each.
///
/// ```rust
/// // Instantiate a new mixer with a sample rate of 44100
/// let mut mixer = usfx::Mixer::new(44_100);
///
/// // Create a default sample as the sinewave
/// let sample = usfx::Sample::default();
/// // Create another sample with a trianglewave
/// let mut other_sample = usfx::Sample::default();
/// other_sample.osc_type(usfx::OscillatorType::Triangle);
///
/// // Play two oscillators at the same time
/// mixer.play(sample);
/// mixer.play(other_sample);
///
/// // This buffer should be passed by the audio library.
/// let mut buffer = [0.0; 44_100];
/// // Fill the buffer with procedurally generated sound.
/// mixer.generate(&mut buffer);
/// ```
#[derive(Debug)]
pub struct Mixer {
    /// List of generators.
    generators: Vec<Generator>,
    /// Store the sample rate so we can keep oscillator buffers.
    sample_rate: usize,
    /// A lookup table of oscillator buffers.
    oscillator_lookup: HashMap<(usize, OscillatorType), RefCell<Vec<f32>>>,
}

impl Mixer {
    /// Create a new mixer object.
    pub fn new(sample_rate: usize) -> Self {
        Self {
            sample_rate,
            ..Self::default()
        }
    }

    /// Play a sample.
    pub fn play(&mut self, sample: Sample) {
        // Create the ADSR envelope generator
        let envelope = Envelope::new(
            self.sample_rate as f32,
            sample.env_attack,
            sample.env_decay,
            sample.env_sustain,
            sample.env_release,
        );

        // Get the cached buffer (or automatically create a new one)
        let buffer = self.oscillator_buffer(sample.osc_frequency, sample.osc_type);

        // Create the oscillator
        let oscillator = Oscillator::new(buffer, self.sample_rate);

        // Create the distortion if applicable
        let distortion = match (sample.dis_crunch, sample.dis_drive) {
            (Some(crunch), Some(drive)) => Some(Distortion::new(crunch, drive)),
            (Some(crunch), None) => Some(Distortion::new(crunch, 1.0)),
            (None, Some(drive)) => Some(Distortion::new(0.0, drive)),
            (None, None) => None,
        };

        // Combine them in a generator
        let generator = Generator {
            finished: false,
            offset: 0,
            volume: sample.volume,

            oscillator,
            envelope,

            distortion,
        };

        // Use the generator
        self.generators.push(generator);
    }

    /// Generate a frame for the sample.
    ///
    /// The output buffer can be smaller but not bigger than the sample size.
    ///
    /// ```rust
    /// // Instantiate a new mixer
    /// let mut mixer = usfx::Mixer::default();
    ///
    /// // Create a default sample as the sinewave
    /// mixer.play(usfx::Sample::default());
    ///
    /// // This buffer should be passed by the audio library
    /// let mut buffer = [0.0; 44_100];
    /// // Fill the buffer with procedurally generated sound
    /// mixer.generate(&mut buffer);
    /// ```
    pub fn generate(&mut self, output: &mut [f32]) {
        // Set the buffer to zero
        output.iter_mut().for_each(|tone| *tone = 0.0);

        // If there are no generators just return the empty buffer
        let generators_len = self.generators.len();
        if generators_len == 0 {
            return;
        }

        // Run the generators
        self.generators
            .iter_mut()
            .for_each(|generator| generator.run(output));

        // Remove the ones that are finished
        self.generators.retain(|generator| !generator.finished);

        // Calculate the inverse so we can multiply instead of divide which is more efficient
        let buffer_len_inv = 1.0 / generators_len as f32;

        // Divide the generators by the current samples
        output.iter_mut().for_each(|tone| *tone *= buffer_len_inv);
    }

    /// Retrieve an oscillator buffer or create it when it doesn't exist yet.
    fn oscillator_buffer(
        &mut self,
        frequency: usize,
        oscillator_type: OscillatorType,
    ) -> RefCell<Vec<f32>> {
        match self.oscillator_lookup.get(&(frequency, oscillator_type)) {
            // A buffer was already cached, return it
            Some(buffer) => RefCell::clone(buffer),
            // Nothing is found, cache a new buffer of frequencies
            None => {
                // Build a lookup table and wrap it in a refcell so there can be multiple immutable
                // references to it
                let lut =
                    RefCell::new(oscillator_type.build_lut(frequency as f32, self.sample_rate));

                // Clone it so it can be returned after the original object is inserted
                let cloned_ref = RefCell::clone(&lut);

                // Add the new lookup table to the cache
                self.oscillator_lookup
                    .insert((frequency, oscillator_type), lut);

                cloned_ref
            }
        }
    }
}

impl Default for Mixer {
    /// The default sample rate is 44100.
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            generators: vec![],
            oscillator_lookup: HashMap::new(),
        }
    }
}
