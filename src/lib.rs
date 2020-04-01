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
use oscillator::{Oscillator as OscillatorTrait, SawWave, SineWave, SquareWave, TriangleWave};

/// Wave form generation type.
#[derive(Debug, Copy, Clone)]
pub enum Oscillator {
    Sine,
    Saw,
    Triangle,
    Square,
}

impl Oscillator {
    /// Instantiate the oscillator struct.
    fn create_struct(self, sample_rate: f32, frequency: f32) -> Box<dyn OscillatorTrait> {
        match self {
            Oscillator::Sine => Box::new(SineWave::new(sample_rate, frequency)),
            Oscillator::Saw => Box::new(SawWave::new(sample_rate, frequency)),
            Oscillator::Triangle => Box::new(TriangleWave::new(sample_rate, frequency)),
            Oscillator::Square => Box::new(SquareWave::new(sample_rate, frequency)),
        }
    }
}

/// Audio sample that procedurally generates it's sound.
///
/// This is the builder that will construct the [`Generator`].
///
/// ```rust
/// // Generate a sine wave at 2khz
/// let mut sine_wave = usfx::Sample::default()
///     .osc_frequency(2000.0)
///     .osc_type(usfx::Oscillator::Sine)
///     .build();
///
/// // Plug it into a audio library, see the examples for a cpal & SDL2 implementation
/// // ...
/// // Call the generator to get a buffer for the audio library
/// # let mut buffer = [0.0];
/// sine_wave.generate(&mut buffer);
/// ```
///
/// [`Generator`]: struct.Generator.html
#[derive(Debug, Copy, Clone)]
pub struct Sample {
    sample_rate: f32,
    osc_frequency: f32,
    osc_type: Oscillator,
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
            osc_frequency: 441.0,
            osc_type: Oscillator::Sine,
            env_attack: 0.01,
            env_decay: 0.1,
            env_sustain: 0.5,
            env_release: 0.5,
        }
    }
}

impl Sample {
    /// Set the sample rate, this depends on the audio device.
    ///
    /// When the [`Mixer`] struct is used this field is ignored.
    ///
    /// [`Mixer`]: struct.Mixer.html
    pub fn sample_rate<'a>(&'a mut self, sample_rate: usize) -> &'a mut Self {
        self.sample_rate = sample_rate as f32;

        self
    }

    /// Set the frequency of the oscillator in hertz.
    pub fn osc_frequency<'a>(&'a mut self, frequency: f32) -> &'a mut Self {
        self.osc_frequency = frequency;

        self
    }

    /// Set the type of the oscillator.
    ///
    /// See the [`Oscillator`] enum for supported wave types.
    ///
    /// [`Oscillator`]: enum.Oscillator.html
    pub fn osc_type<'a>(&'a mut self, oscillator: Oscillator) -> &'a mut Self {
        self.osc_type = oscillator;

        self
    }

    /// Set the time until the first envelope slope reaches it's maximum height.
    pub fn env_attack<'a>(&'a mut self, attack: f32) -> &'a mut Self {
        self.env_attack = attack;

        self
    }

    /// Set the time it takes from the maximum height to go into the main plateau.
    pub fn env_decay<'a>(&'a mut self, decay: f32) -> &'a mut Self {
        self.env_decay = decay;

        self
    }

    /// Set the height of the main plateau.
    pub fn env_sustain<'a>(&'a mut self, sustain: f32) -> &'a mut Self {
        self.env_sustain = sustain;

        self
    }

    /// Set the time it takes to go from the end of the plateau to zero.
    pub fn env_release<'a>(&'a mut self, release: f32) -> &'a mut Self {
        self.env_release = release;

        self
    }

    /// Create the sample object.
    ///
    /// A type implementing the Oscillator trait is required as the generic argument.
    ///
    /// ```rust
    /// // Create a default sample with a sinewave
    /// usfx::Sample::default().build();
    /// ```
    pub fn build(&self) -> Generator {
        let envelope = Envelope::new(
            self.sample_rate,
            self.env_attack,
            self.env_decay,
            self.env_sustain,
            self.env_release,
        );

        let oscillator = self
            .osc_type
            .create_struct(self.sample_rate, self.osc_frequency);

        Generator {
            finished: false,
            offset: 0,
            oscillator,
            envelope,
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
    oscillator: Box<dyn OscillatorTrait>,
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
    /// let mut generator = usfx::Sample::default().build();
    ///
    /// // This buffer should be passed by the audio library.
    /// let mut buffer = [0.0; 44_100];
    /// // Fill the buffer with procedurally generated sound.
    /// generator.generate(&mut buffer);
    /// ```
    pub fn generate(&mut self, mut output: &mut [f32]) {
        // Set the buffer to zero
        output.iter_mut().for_each(|tone| *tone = 0.0);

        if !self.finished {
            self.run(&mut output);
        }
    }

    /// Internal generator, used by the mixer and the generate function.
    fn run(&mut self, mut output: &mut [f32]) {
        // Run the oscillator
        self.oscillator.generate(&mut output, self.offset);

        // Apply the ADSR and set the state if we're finished or not
        if self.envelope.apply(&mut output, self.offset) == State::Done {
            self.finished = true;
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
/// other_sample.osc_type(usfx::Oscillator::Triangle);
///
/// // Play two oscillators at the same time
/// mixer.play(&sample);
/// mixer.play(&other_sample);
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
}

impl Mixer {
    /// Create a new mixer object.
    pub fn new(sample_rate: usize) -> Self {
        Self {
            sample_rate,
            generators: vec![],
        }
    }

    /// Play a sample.
    pub fn play(&mut self, sample: &Sample) {
        // Create the ADSR envelope generator
        let envelope = Envelope::new(
            sample.sample_rate,
            sample.env_attack,
            sample.env_decay,
            sample.env_sustain,
            sample.env_release,
        );

        // Create the oscillator
        let oscillator = sample
            .osc_type
            .create_struct(self.sample_rate as f32, sample.osc_frequency);

        // Combine them in a generator
        let generator = Generator {
            finished: false,
            offset: 0,
            oscillator,
            envelope,
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
    /// mixer.play(&usfx::Sample::default());
    ///
    /// // This buffer should be passed by the audio library
    /// let mut buffer = [0.0; 44_100];
    /// // Fill the buffer with procedurally generated sound
    /// mixer.generate(&mut buffer);
    /// ```
    pub fn generate(&mut self, output: &mut [f32]) {
        // Set the buffer to zero
        output.iter_mut().for_each(|tone| *tone = 0.0);

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
}

impl Default for Mixer {
    /// The default sample rate is 44100.
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            generators: vec![],
        }
    }
}
