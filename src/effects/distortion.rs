use crate::effects::Effect;

/// A simple distortion effect.
#[derive(Debug)]
pub struct Distortion {
    /// Overdrive that adds hard clipping.
    crunch: f32,
    /// Overdrive with soft clipping.
    drive: f32,
}

impl Distortion {
    /// Setup the effect.
    pub fn new(crunch: f32, drive: f32) -> Self {
        let crunch = 1.0 - crunch.max(0.01);

        Self { crunch, drive }
    }
}

impl Effect for Distortion {
    /// Apply the effect on the buffer.
    ///
    /// Algorithm from: https://github.com/amsynth/amsynth
    fn apply(&mut self, buffer: &mut [f32], _offset: usize) {
        buffer.iter_mut().for_each(|tone| {
            let x = *tone * self.drive;
            let sign = x.signum();
            // Make negative numbers positive, apply the power function and make them negative
            // again
            *tone = (x * sign).powf(self.crunch) * sign;
        });
    }
}
