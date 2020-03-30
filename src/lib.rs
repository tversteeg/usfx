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

use envelope::Envelope;
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
pub struct Sample {
    sample_rate: usize,
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
            sample_rate: 44_100,
            osc_frequency: 441,
            env_attack: 0.01,
            env_decay: 0.1,
            env_sustain: 0.2,
            env_release: 0.1,
        }
    }
}

impl Sample {
    /// Set the sample rate, this depends on the audio device.
    pub fn sample_rate(&'_ mut self, sample_rate: usize) -> &'_ mut Self {
        self.sample_rate = sample_rate;

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
            offset: 0,
            oscillator: Box::new(<O>::new(self.sample_rate as f32, self.osc_frequency as f32)),
            envelope: Envelope::new(
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
///
/// [`Sample`]: struct.Sample.html
pub struct Generator {
    /// The total offset.
    pub offset: usize,
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
        let buffer_len = output.len();

        self.oscillator.generate(&mut output, self.offset);

        self.offset += buffer_len;
    }
}
