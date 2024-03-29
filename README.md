<h1 align="center">μsfx</h1>
	
<p align="center">
	<a href="https://github.com/tversteeg/usfx/actions"><img src="https://github.com/tversteeg/usfx/workflows/CI/badge.svg" alt="CI"/></a>
	<a href="https://crates.io/crates/usfx"><img src="https://img.shields.io/crates/v/usfx.svg" alt="Version"/></a>
	<a href="https://docs.rs/usfx"><img src="https://img.shields.io/badge/api-rustdoc-blue.svg" alt="Rust Documentation"/></a>
	<img src="https://img.shields.io/crates/l/usfx.svg" alt="License"/>
	<br/>
</p>

<!-- cargo-rdme start -->

Generate sound effects for your game in realtime.

## Example

```rust
// Create a simple blip sound
let mut sample = usfx::Sample::default();
sample.volume(0.5);

// Use a sine wave oscillator at 500 hz
sample.osc_type(usfx::OscillatorType::Sine);
sample.osc_frequency(500);

// Set the envelope
sample.env_attack(0.02);
sample.env_decay(0.05);
sample.env_sustain(0.2);
sample.env_release(0.5);

// Add some distortion
sample.dis_crunch(0.5);
sample.dis_drive(0.9);

// Create a mixer so we can play the sound
let mut mixer = usfx::Mixer::default();

// Play our sample
mixer.play(sample);

// Plug our mixer into the audio device loop
// ...
mixer.generate(&mut audio_device_buffer);
```

<!-- cargo-rdme end -->

The [`cpal`](examples/cpal.rs) & [`sdl`](examples/sdl2.rs) examples illustrate how to use it with different audio libraries. The [`music`](examples/music.rs) example shows how to create procedurally generated music with it (don't expect a masterpiece though, it's obvious I'm not a musician).

### CPAL Example

To build the [`cpal`](examples/cpal.rs) & [`music`](examples/music.rs) examples on Linux you will need to have the alsa development libraries:

```bash
sudo apt install libasound2-dev
```

### SDL Example

To build the [`sdl`](examples/sdl2.rs) you will need the SDL2 development libraries, on Linux:

```bash
sudo apt install libsdl2-dev
```

## Tools

- [usfx-test](https://github.com/emmabritton/uxfs-test) - pretty GUI program for playing with the parameters by @emmabritton

## Special Thanks

- [sfxr-rs](https://github.com/bzar/sfxr-rs) - inspiration
- [amsynth](https://github.com/amsynth/amsynth) - distortion algorithm
