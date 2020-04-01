use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::time::Duration;

struct Sample {
    sample: usfx::Generator,
}

impl AudioCallback for Sample {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.sample.generate(out)
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1),
        samples: None,
    };

    // Create a default sample, which is a sine-wave of 441 hz
    let sample = usfx::Sample::default().build();

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |_spec| Sample { sample })
        .unwrap();

    device.resume();
    std::thread::sleep(Duration::from_millis(5_000));
}
