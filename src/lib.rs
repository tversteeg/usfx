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

use std::{f32::consts::PI, slice::Iter as SliceIter};

const PI2: f32 = PI * 2.0;

/// A trait defining a struct as an oscillator.
pub trait Oscillator: Send {
    /// Instantiate a new oscillator.
    fn new(setup: OscillatorSetup) -> Self
    where
        Self: Sized;

    /// The function used to generate the wave table.
    fn wave_func(&self, index: f32, frequency: f32, sample_rate: f32) -> f32;

    /// Set the new phase position.
    fn set_phase_index(&mut self, index: usize);

    /// Get the index of the phase.
    fn phase_index(&self) -> usize;

    /// Get the lookup table.
    fn phase_lut(&self, index: usize) -> &[f32];

    /// Get the length of the lookup table.
    fn phase_lut_size(&self) -> usize;

    /// Move the phase to a certain offset.
    fn move_phase(&mut self, offset: usize) {
        self.set_phase_index((self.phase_index() + offset) % self.phase_lut_size());
    }

    /// Generate the output buffer.
    fn generate(&mut self, output: &mut [f32]) {
        output
            .iter_mut()
            .zip(self.phase_iter())
            .for_each(|(old, new)| *old = *new);

        self.move_phase(output.len());
    }

    /// Create a lookup table so we don't have to calculate everything every frame.
    fn build_lut(&self, setup: OscillatorSetup) -> Vec<f32> {
        // Create a table twice the size so we don't have to use modulo on every frame
        (0..setup.sample_rate * 2)
            .map(|i| self.wave_func(i as f32, setup.frequency, setup.sample_rate as f32))
            .collect()
    }

    /// Create an iterator that starts at the phase offset.
    fn phase_iter(&self) -> SliceIter<f32> {
        self.phase_lut(self.phase_index()).iter()
    }
}

/// Audio sample that procedurally generates it's sound.
///
/// This is the builder that will construct the `Generator`.
pub struct Sample {
    sample_rate: usize,
    osc_frequency: f32,
}

impl Default for Sample {
    fn default() -> Self {
        Self {
            osc_frequency: 1000.0,
            sample_rate: 44_100,
        }
    }
}

impl Sample {
    /// Set the frequency of the oscillator in hertz.
    pub fn osc_frequency<'a>(&'a mut self, frequency: f32) -> &'a mut Self {
        self.osc_frequency = frequency;

        self
    }

    /// Set the sample rate, this depends on the audio device.
    pub fn sample_rate<'a>(&'a mut self, sample_rate: usize) -> &'a mut Self {
        self.sample_rate = sample_rate;

        self
    }

    /// Create the sample object.
    ///
    /// A oscillator wave type is required as the generic argument.
    pub fn build<O>(&self) -> Generator
    where
        O: Oscillator + 'static,
    {
        let oscilattor_setup = OscillatorSetup {
            // 1 khz
            frequency: 1000.0,
            sample_rate: 44_100,
        };

        Generator {
            oscillator: Box::new(<O>::new(oscilattor_setup)),
        }
    }
}

pub struct Generator {
    oscillator: Box<dyn Oscillator>,
}

impl Generator {
    /// Generate a frame for the sample.
    pub fn generate(&mut self, mut output: &mut [f32]) {
        self.oscillator.generate(&mut output);
    }
}

/// Setup information for an oscillator.
pub struct OscillatorSetup {
    pub frequency: f32,
    pub sample_rate: usize,
}

/// A sine as waveform of the oscilattor.
pub struct SineWave {
    phase: usize,
    phase_lut: Vec<f32>,
}

impl Oscillator for SineWave {
    /// Create a new simple sine oscillator.
    fn new(setup: OscillatorSetup) -> Self {
        let mut sine = Self {
            phase: 0,
            phase_lut: vec![],
        };

        sine.phase_lut = sine.build_lut(setup);

        sine
    }

    /// A simple sine wave.
    fn wave_func(&self, index: f32, frequency: f32, sample_rate: f32) -> f32 {
        (index * frequency * PI2 / sample_rate).sin()
    }

    fn phase_index(&self) -> usize {
        self.phase as usize
    }

    fn set_phase_index(&mut self, index: usize) {
        self.phase = index;
    }

    fn phase_lut(&self, index: usize) -> &[f32] {
        &self.phase_lut[index..]
    }

    fn phase_lut_size(&self) -> usize {
        self.phase_lut.len() / 2
    }
}
