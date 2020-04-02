use std::{cell::RefCell, f32::consts::PI};

const PI2: f32 = PI * 2.0;

/// Wave form generation type.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum OscillatorType {
    /// A continuus tone.
    Sine,
    /// Strong, clear, buzzing sound.
    Saw,
    /// Smooth sound, between sine & square.
    Triangle,
    /// Rich sound, between sine & saw.
    Square,
}

impl OscillatorType {
    /// Build a lookup table from this type.
    ///
    /// The table will be twice the size of the sample rate so we can use the whole size with an
    /// offset in it.
    pub(crate) fn build_lut(self, frequency: f32, sample_rate: usize) -> Vec<f32> {
        let wave_func = self.wave_function();

        // Create a table twice the size so we don't have to use modulo on every frame
        (0..sample_rate * 2)
            .map(|i| wave_func(i as f32, frequency, sample_rate as f32))
            .collect()
    }

    /// The way the oscillator calculates the output wave.
    ///
    /// Used to build the lookup table.
    fn wave_function(self) -> Box<dyn Fn(f32, f32, f32) -> f32> {
        match self {
            OscillatorType::Sine => Box::new(|index: f32, frequency: f32, sample_rate: f32| {
                (index * frequency * PI2 / sample_rate).sin()
            }),
            OscillatorType::Saw => Box::new(|index: f32, frequency: f32, sample_rate: f32| {
                let steps = sample_rate / frequency;

                1.0 - ((index / steps) % 1.0) * 2.0
            }),
            OscillatorType::Triangle => Box::new(|index: f32, frequency: f32, sample_rate: f32| {
                let steps = sample_rate / frequency;

                let slope = (index / steps) % 1.0 * 2.0;
                if slope < 1.0 {
                    -1.0 + slope * 2.0
                } else {
                    3.0 - slope * 2.0
                }
            }),
            OscillatorType::Square => Box::new(|index: f32, frequency: f32, sample_rate: f32| {
                let steps = sample_rate / frequency;

                if (index / steps) % 1.0 < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }),
        }
    }
}

/// The oscillator just loops through the already populated lookup table.
#[derive(Debug)]
pub(crate) struct Oscillator {
    /// The lookup table is a reference owned by the Mixer struct.
    lut: RefCell<Vec<f32>>,
    /// The sample rate, also half the size of the lookup table.
    sample_rate: usize,
}

impl Oscillator {
    /// Instantiate a new oscillator that uses the passed lookup table.
    pub(crate) fn new(lut: RefCell<Vec<f32>>, sample_rate: usize) -> Self {
        Self { lut, sample_rate }
    }

    /// Fill the output buffer with generated sound.
    pub(crate) fn generate(&mut self, output: &mut [f32], offset: usize) {
        let rotating_index = offset % self.sample_rate;

        // Combine the size of the output buffer with the size of the cached frequencies buffer and
        // add the frequencies to the output
        output
            .iter_mut()
            .zip(self.lut.borrow()[rotating_index..].iter())
            .for_each(|(old, new)| *old += *new);
    }
}
