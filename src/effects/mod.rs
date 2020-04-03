pub mod distortion;

use std::fmt::Debug;

/// Generic interface for effects.
pub trait Effect: Debug + Send {
    /// Apply the effect on the buffer.
    fn apply(&mut self, buffer: &mut [f32], offset: usize);
}
