/// A default ADSR envelope.
pub(crate) struct Envelope {
    /// Time until the first slope reaches it's maximum height.
    attack: f32,
    /// Time it takes from the maximum height to go into the main plateau.
    decay: f32,
    /// Height of the main plateau.
    sustain: f32,
    /// Time it takes to go from the end of the plateau to zero.
    release: f32,
}

impl Envelope {
    /// Instantiate a new envelope generater following the ADSR principle.
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
        }
    }
}
