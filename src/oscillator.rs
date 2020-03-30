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

    /// Get the lookup table.
    fn phase_lut(&self, index: usize) -> &[f32];

    /// Generate the output buffer.
    fn generate(&mut self, output: &mut [f32], offset: usize) {
        output
            .iter_mut()
            .zip(self.phase_iter(offset))
            .for_each(|(old, new)| *old = *new);
    }

    /// Create a lookup table so we don't have to calculate everything every frame.
    fn build_lut(&self, sample_rate: f32, frequency: f32) -> Vec<f32> {
        // Create a table twice the size so we don't have to use modulo on every frame
        (0..sample_rate as usize * 2)
            .map(|i| self.wave_func(i as f32, frequency, sample_rate))
            .collect()
    }

    /// Create an iterator that starts at the phase offset.
    fn phase_iter(&self, offset: usize) -> SliceIter<f32> {
        self.phase_lut(offset).iter()
    }
}

// Oscillator generator, makes it so the implementation of a oscillator only needs to expose a wave
// func & a name for the struct.
macro_rules! oscillator {
    ( $(#[$outer:meta])* $name:ident($index:ident, $frequency:ident, $sample_rate:ident) $wave_func:tt ) => {
        $(#[$outer])*
        pub struct $name {
            phase_lut: Vec<f32>,
            sample_rate: usize,
        }

        impl Oscillator for $name {
            /// Create a new oscillator.
            fn new(sample_rate: f32, frequency: f32) -> Self {
                let mut osc = Self {
                    phase_lut: vec![],
                    sample_rate: sample_rate as usize,
                };
                osc.phase_lut = osc.build_lut(sample_rate, frequency);

                osc
            }

            /// Generate the wave function.
            #[inline(always)]
            fn wave_func(&self, $index: f32, $frequency: f32, $sample_rate: f32) -> f32 {
                $wave_func
            }

            /// Return the lookup table from the index until the end of the table.
            #[inline(always)]
            fn phase_lut(&self, index: usize) -> &[f32] {
                let rotating_index = index % self.sample_rate;
                &self.phase_lut[rotating_index..]
            }
        }
    };
}

oscillator! {
    /// A simple sine wave oscillator.
    SineWave(index, frequency, sample_rate) {
    (index * frequency * PI2 / sample_rate).sin()
}}

oscillator! {
    /// A simple saw wave oscillator.
    SawWave(index, frequency, sample_rate) {
    let steps = sample_rate / frequency;

    1.0 - ((index / steps) % 1.0) * 2.0
}}

oscillator! {
    /// A simple square wave oscillator.
    SquareWave(index, frequency, sample_rate) {
    let steps = sample_rate / frequency;

    if (index / steps) % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}}

oscillator! {
    /// A simple triangle wave oscillator.
    TriangleWave(index, frequency, sample_rate) {
    let steps = sample_rate / frequency;

    let slope = (index / steps) % 1.0 * 2.0;
    if slope < 1.0 {
        -1.0 + slope * 2.0
    } else {
        3.0 - slope * 2.0
    }
}}
