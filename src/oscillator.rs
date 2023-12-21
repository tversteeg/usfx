use randomize::{formulas, PCG32};
use std::{cell::RefCell, f32::consts::PI};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const PI2: f32 = PI * 2.0;

/// Possible values for the duty cycle of the square wave.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum DutyCycle {
    /// A duty cycle of 12.5%.
    Eight,
    /// A duty cycle of 25%.
    Quarter,
    /// A duty cycle of 33%.
    Third,
    /// A duty cycle of 50%.
    Half,
}

impl DutyCycle {
    /// Convert it to a number we can compare with.
    pub fn to_frac(self) -> f32 {
        match self {
            DutyCycle::Eight => 0.125,
            DutyCycle::Quarter => 0.25,
            DutyCycle::Third => 0.33,
            DutyCycle::Half => 0.5,
        }
    }
}

impl Default for DutyCycle {
    /// The default cycle is half.
    fn default() -> Self {
        DutyCycle::Half
    }
}

/// Wave form generation type.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum OscillatorType {
    /// A continuus tone.
    Sine,
    /// Strong, clear, buzzing sound.
    Saw,
    /// Smooth sound, between sine & square.
    Triangle,
    /// Rich sound, between sine & saw.
    ///
    /// This wave type uses `osc_duty_cycle` from `Sample`.
    Square,
    /// White noise, very noisy.
    ///
    /// `osc_frequency` is the seed for the RNG.
    Noise,
}

impl OscillatorType {
    /// Build a lookup table from this type.
    ///
    /// The table will be twice the size of the sample rate so we can use the whole size with an
    /// offset in it.
    pub(crate) fn build_lut(
        self,
        frequency: usize,
        duty_cycle: DutyCycle,
        sample_rate: usize,
    ) -> Vec<f32> {
        // Create a table twice the size so we don't have to use modulo on every frame
        let buffer_size = sample_rate * 2;

        match self {
            OscillatorType::Sine => {
                // Move this calculation out of the loop for performance reasons
                let mult = frequency as f32 * PI2 / sample_rate as f32;

                (0..buffer_size)
                    .map(|index| (index as f32 * mult).sin())
                    .collect()
            }
            OscillatorType::Saw => (0..buffer_size)
                .map(|index| {
                    1.0 - ((index as f32 / sample_rate as f32 * frequency as f32) % 1.0) * 2.0
                })
                .collect(),
            OscillatorType::Triangle => (0..buffer_size)
                .map(|index| {
                    let slope = (index as f32 / sample_rate as f32 * frequency as f32) % 1.0 * 2.0;
                    if slope < 1.0 {
                        -1.0 + slope * 2.0
                    } else {
                        3.0 - slope * 2.0
                    }
                })
                .collect(),
            OscillatorType::Square => (0..buffer_size)
                .map(|index| {
                    if (index as f32 / sample_rate as f32 * frequency as f32) % 1.0
                        < duty_cycle.to_frac()
                    {
                        1.0
                    } else {
                        -1.0
                    }
                })
                .collect(),
            OscillatorType::Noise => {
                let mut pcg = PCG32::seed(frequency as u64, 5);

                (0..buffer_size)
                    .map(|_| formulas::f32_closed_neg_pos(pcg.next_u32()))
                    .collect()
            }
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
