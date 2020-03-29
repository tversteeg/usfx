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
    fn new(sample_rate: f32, frequency: f32) -> Self
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
    #[inline(always)]
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
    fn build_lut(&self, sample_rate: f32, frequency: f32) -> Vec<f32> {
        // Create a table twice the size so we don't have to use modulo on every frame
        (0..sample_rate as usize * 2)
            .map(|i| self.wave_func(i as f32, frequency, sample_rate))
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
///
/// ```rust
/// // Generate a sine wave at 2khz
/// let sine_wave = usfx::Sample::default()
///     .osc_frequency(2000.0)
///     .build::<usfx::SineWave>();
///
/// // Plug it into a audio library, see the examples for a cpal & SDL2 implementation
/// ```
pub struct Sample {
    sample_rate: usize,
    osc_frequency: usize,
}

impl Default for Sample {
    /// The default is a 441hz wave with a sample rate of 441000.
    fn default() -> Self {
        Self {
            osc_frequency: 441,
            sample_rate: 44_100,
        }
    }
}

impl Sample {
    /// Set the frequency of the oscillator in hertz.
    pub fn osc_frequency<'a>(&'a mut self, frequency: usize) -> &'a mut Self {
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
        Generator {
            oscillator: Box::new(<O>::new(self.sample_rate as f32, self.osc_frequency as f32)),
        }
    }
}

/// Convert samples with PCM.
pub struct Generator {
    oscillator: Box<dyn Oscillator>,
}

impl Generator {
    /// Generate a frame for the sample.
    ///
    /// The output buffer can be smaller but not bigger than the sample size.
    pub fn generate(&mut self, mut output: &mut [f32]) {
        self.oscillator.generate(&mut output);
    }
}

// Oscillator generator, makes it so the implementation of a oscillator only needs to expose a wave
// func & a name for the struct.
macro_rules! oscillator {
    ( $name:ident($index:ident, $frequency:ident, $sample_rate:ident) $wave_func:tt ) => {
        /// A simple oscillator.
        pub struct $name {
            phase: usize,
            phase_lut: Vec<f32>,
        }

        impl Oscillator for $name {
            /// Create a new oscillator.
            fn new(sample_rate: f32, frequency: f32) -> Self {
                let mut osc = Self {
                    phase: 0,
                    phase_lut: vec![],
                };
                osc.phase_lut = osc.build_lut(sample_rate, frequency);

                osc
            }

            /// Generate the wave function.
            #[inline(always)]
            fn wave_func(&self, $index: f32, $frequency: f32, $sample_rate: f32) -> f32 {
                $wave_func
            }

            /// Return the current index of the phase.
            #[inline(always)]
            fn phase_index(&self) -> usize {
                self.phase as usize
            }

            #[inline(always)]
            fn set_phase_index(&mut self, index: usize) {
                self.phase = index;
            }

            /// Return the lookup table from the index until the end of the table.
            #[inline(always)]
            fn phase_lut(&self, index: usize) -> &[f32] {
                &self.phase_lut[index..]
            }

            #[inline(always)]
            fn phase_lut_size(&self) -> usize {
                // We divide the size by half because that's the real size of the lookup table
                self.phase_lut.len() / 2
            }
        }
    };
}

oscillator! { SineWave(index, frequency, sample_rate) {
    (index * frequency * PI2 / sample_rate).sin()
}}

oscillator! { SawWave(index, frequency, sample_rate) {
    let steps = sample_rate / frequency;

    1.0 - ((index / steps) % 1.0) * 2.0
}}

oscillator! { SquareWave(index, frequency, sample_rate) {
    let steps = sample_rate / frequency;

    if (index / steps) % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}}

oscillator! { TriangleWave(index, frequency, sample_rate) {
    let steps = sample_rate / frequency;

    let slope = (index / steps) % 1.0 * 2.0;
    if slope < 1.0 {
        -1.0 + slope * 2.0
    } else {
        3.0 - slope * 2.0
    }
}}
