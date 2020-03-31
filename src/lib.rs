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

mod envelope;
mod oscillator;

use envelope::{Envelope, State};
use oscillator::Oscillator;
pub use oscillator::{SawWave, SineWave, SquareWave, TriangleWave};

/// Audio sample that procedurally generates it's sound.
///
/// This is the builder that will construct the [`Generator`].
///
/// ```rust
/// // Generate a sine wave at 2khz
/// let mut sine_wave = usfx::Sample::default()
///     .osc_frequency(2000)
///     .build::<usfx::SineWave>();
///
/// // Plug it into a audio library, see the examples for a cpal & SDL2 implementation
/// // ...
/// // Call the generator to get a buffer for the audio library
/// # let mut buffer = [0.0];
/// sine_wave.generate(&mut buffer);
/// ```
///
/// [`Generator`]: struct.Generator.html
#[derive(Debug)]
pub struct Sample {
    sample_rate: f32,
    osc_frequency: usize,
    env_attack: f32,
    env_decay: f32,
    env_release: f32,
    env_sustain: f32,
}

impl Default for Sample {
    /// The default is a 441hz wave with a sample rate of 441000.
    fn default() -> Self {
        Self {
            sample_rate: 44_100.0,
            osc_frequency: 441,
            env_attack: 0.01,
            env_decay: 0.1,
            env_sustain: 0.5,
            env_release: 0.5,
        }
    }
}

impl Sample {
    /// Set the sample rate, this depends on the audio device.
    pub fn sample_rate(&'_ mut self, sample_rate: usize) -> &'_ mut Self {
        self.sample_rate = sample_rate as f32;

        self
    }

    /// Set the frequency of the oscillator in hertz.
    pub fn osc_frequency(&'_ mut self, frequency: usize) -> &'_ mut Self {
        self.osc_frequency = frequency;

        self
    }

    /// Set the time until the first envelope slope reaches it's maximum height.
    pub fn env_attack(&'_ mut self, attack: f32) -> &'_ mut Self {
        self.env_attack = attack;

        self
    }

    /// Set the time it takes from the maximum height to go into the main plateau.
    pub fn env_decay(&'_ mut self, decay: f32) -> &'_ mut Self {
        self.env_decay = decay;

        self
    }

    /// Set the height of the main plateau.
    pub fn env_sustain(&'_ mut self, sustain: f32) -> &'_ mut Self {
        self.env_sustain = sustain;

        self
    }

    /// Set the time it takes to go from the end of the plateau to zero.
    pub fn env_release(&'_ mut self, release: f32) -> &'_ mut Self {
        self.env_release = release;

        self
    }

    /// Create the sample object.
    ///
    /// A type implementing the Oscillator trait is required as the generic argument.
    ///
    /// ```rust
    /// // Create a default sample with a sinewave
    /// usfx::Sample::default()
    ///     .build::<usfx::SineWave>();
    /// ```
    pub fn build<O>(&self) -> Generator
    where
        O: Oscillator + 'static,
    {
        Generator {
            finished: false,
            offset: 0,
            oscillator: Box::new(<O>::new(self.sample_rate, self.osc_frequency as f32)),
            envelope: Envelope::new(
                self.sample_rate,
                self.env_attack,
                self.env_decay,
                self.env_sustain,
                self.env_release,
            ),
        }
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
pub struct Generator {
    /// Whether we are finished running the sample.
    pub(crate) finished: bool,
    /// The total offset.
    offset: usize,
    /// The oscillator, because it's a trait it has to be boxed.
    oscillator: Box<dyn Oscillator>,
    /// The ADSR envelope.
    envelope: Envelope,
}

impl Generator {
    /// Generate a frame for the sample.
    ///
    /// The output buffer can be smaller but not bigger than the sample size.
    ///
    /// ```rust
    /// // Create a default sample as the sinewave
    /// let mut generator = usfx::Sample::default()
    ///     .build::<usfx::SineWave>();
    ///
    /// // This buffer should be passed by the audio library.
    /// let mut buffer = [0.0; 44_100];
    /// // Fill the buffer with procedurally generated sound.
    /// generator.generate(&mut buffer);
    /// ```
    pub fn generate(&mut self, mut output: &mut [f32]) {
        // Nothing to generate anymore
        if self.finished {
            // Fill the buffer with zeros
            output.iter_mut().for_each(|tone| *tone = 0.0);
        } else {
            self.run(&mut output);
        }
    }

    /// Internal generator, used by the mixer and this generate function.
    pub(crate) fn run(&mut self, mut output: &mut [f32]) {
        // Run the oscillator
        self.oscillator.generate(&mut output, self.offset);

        // Apply the ADSR and set the state if we're finished or not
        if self.envelope.apply(&mut output, self.offset) == State::Done {
            self.finished = true;
        } else {
            self.offset += output.len();
        }
    }
}

/// Manage samples and mix the volume output of each.
///
/// ```rust
/// // Instantiate a new mixer
/// let mut mixer = usfx::Mixer::default();
///
/// // Create a default sample as the sinewave
/// let sample = usfx::Sample::default();
///
/// // Play two oscillators at the same time
/// mixer.play(sample.build::<usfx::SineWave>());
/// mixer.play(sample.build::<usfx::TriangleWave>());
///
/// // This buffer should be passed by the audio library.
/// let mut buffer = [0.0; 44_100];
/// // Fill the buffer with procedurally generated sound.
/// mixer.generate(&mut buffer);
/// ```
#[derive(Debug, Default)]
pub struct Mixer {
    /// List of generators.
    generators: Vec<Generator>,
}

impl Mixer {
    /// Play a sample.
    pub fn play(&mut self, generator: Generator) {
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
    /// mixer.play(usfx::Sample::default().build::<usfx::SineWave>());
    ///
    /// // This buffer should be passed by the audio library
    /// let mut buffer = [0.0; 44_100];
    /// // Fill the buffer with procedurally generated sound
    /// mixer.generate(&mut buffer);
    /// ```
    pub fn generate(&mut self, output: &mut [f32]) {
        let generators_len = self.generators.len();
        if generators_len == 0 {
            // No generators are running, set the result to zero
            output.iter_mut().for_each(|tone| *tone = 0.0);
            return;
        }

        // Run the generators
        self.generators
            .iter_mut()
            .for_each(|generator| generator.run(output));

        // Remove the ones that are finished
        self.generators.retain(|generator| generator.finished);

        // Calculate the inverse so we can multiply instead of divide which is more efficient
        let buffer_len_inv = 1.0 / generators_len as f32;

        // Divide the generators by the current samples
        output.iter_mut().for_each(|tone| *tone *= buffer_len_inv);
    }
}
