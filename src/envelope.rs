/// The current state of the ADSR.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum State {
    Attack,
    Decay(usize),
    Release(usize),
    Done,
}

/// A default ADSR envelope.
#[derive(Debug)]
pub(crate) struct Envelope {
    /// Time until the first slope reaches it's maximum height.
    attack_slope: f32,
    /// Time it takes from the maximum height to go into the main plateau.
    decay_slope: f32,
    /// Height of the main plateau.
    sustain_height: f32,
    /// Time it takes to go from the end of the plateau to zero.
    release_slope: f32,

    /// The current state of the ADSR.
    state: State,
}

impl Envelope {
    /// Instantiate a new envelope generater following the ADSR principle.
    pub fn new(sample_rate: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack_slope: 1.0 / attack / sample_rate,
            decay_slope: 1.0 / decay / sustain / sample_rate,
            sustain_height: sustain,
            release_slope: 1.0 / release / sustain / sample_rate,
            state: State::Attack,
        }
    }

    /// Apply the envelope on a buffer.
    pub fn apply(&mut self, buffer: &mut [f32], offset: usize) -> State {
        buffer.iter_mut().enumerate().for_each(|(index, tone)| {
            let index_with_offset = index + offset;
            match self.state {
                // Going up
                State::Attack => {
                    let multiplier = index_with_offset as f32 * self.attack_slope;
                    *tone *= multiplier;

                    if multiplier >= 1.0 {
                        // Move to the new state when we are at the top
                        self.state = State::Decay(index_with_offset);
                    }
                }
                // Going down to the middle
                State::Decay(last_offset) => {
                    let multiplier =
                        1.0 - ((index_with_offset - last_offset) as f32 * self.decay_slope);
                    *tone *= multiplier;

                    if multiplier <= self.sustain_height {
                        // Move to the new state when we are at the sustain height
                        self.state = State::Release(index_with_offset);
                    }
                }
                // Going from the middle to the bottom
                State::Release(last_offset) => {
                    let multiplier = self.sustain_height
                        - ((index_with_offset - last_offset) as f32 * self.release_slope);
                    *tone *= multiplier;

                    if multiplier <= 0.0 {
                        // We are finished when the multiplier is zero
                        self.state = State::Done;
                    }
                }
                // Nothing left
                State::Done => {
                    *tone = 0.0;
                }
            }
        });

        self.state
    }
}
