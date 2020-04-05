use criterion::{criterion_group, criterion_main, Criterion};
use usfx::*;

fn criterion_benchmark(c: &mut Criterion) {
    // All wavetypes have the same generate function
    c.bench_function("generate", |b| {
        let mut buffer = [0.0; 2000];

        let mut sample = Sample::default();
        sample.osc_type(OscillatorType::Sine);

        let mut mixer = Mixer::new(2000);
        mixer.play(sample);

        b.iter(|| {
            mixer.generate(&mut buffer);
        });
    });

    c.bench_function("sine wave setup", |b| {
        let mut mixer = Mixer::new(2000);

        let mut sample = Sample::default();
        sample.osc_type(OscillatorType::Sine);

        let mut freq = 1;
        b.iter(|| {
            sample.osc_frequency(freq);

            mixer.play(sample);

            freq += 1;
        });
    });
    c.bench_function("saw wave setup", |b| {
        let mut mixer = Mixer::new(2000);

        let mut sample = Sample::default();
        sample.osc_type(OscillatorType::Saw);

        let mut freq = 1;
        b.iter(|| {
            sample.osc_frequency(freq);

            mixer.play(sample);

            freq += 1;
        });
    });
    c.bench_function("square wave setup", |b| {
        let mut mixer = Mixer::new(2000);

        let mut sample = Sample::default();
        sample.osc_type(OscillatorType::Square);

        let mut freq = 1;
        b.iter(|| {
            sample.osc_frequency(freq);

            mixer.play(sample);

            freq += 1;
        });
    });
    c.bench_function("triangle wave setup", |b| {
        let mut mixer = Mixer::new(2000);

        let mut sample = Sample::default();
        sample.osc_type(OscillatorType::Triangle);

        let mut freq = 1;
        b.iter(|| {
            sample.osc_frequency(freq);

            mixer.play(sample);

            freq += 1;
        });
    });
    c.bench_function("noise wave setup", |b| {
        let mut mixer = Mixer::new(2000);

        let mut sample = Sample::default();
        sample.osc_type(OscillatorType::Noise);

        let mut freq = 1;
        b.iter(|| {
            sample.osc_frequency(freq);

            mixer.play(sample);

            freq += 1;
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
