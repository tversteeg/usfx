use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::time::Duration;

const SAMPLE_RATE: usize = 44_100;

struct Sample {
    mixer: usfx::Mixer,
}

impl AudioCallback for Sample {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.mixer.generate(out)
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE as i32),
        channels: Some(1),
        samples: None,
    };

    // Create a default sample, which is a sine-wave of 441 hz
    let sample = usfx::Sample::default();

    // Create a mixer that will play our sample
    let mut mixer = usfx::Mixer::new(SAMPLE_RATE);

    // Play the sample
    mixer.play(sample);

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |_spec| Sample { mixer })
        .unwrap();

    device.resume();

    std::thread::sleep(Duration::from_millis(5_000));
}
