use criterion::{criterion_group, criterion_main, Criterion};
use usfx::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("short pulse", |b| {
        let mut buffer = [0.0; 2000];

        let mut sample = Sample::default();
        sample.env_attack(0.01);
        sample.env_decay(0.01);
        sample.env_release(0.01);

        let mut mixer = Mixer::new(2000);
        mixer.play(sample);

        b.iter(|| {
            mixer.generate(&mut buffer);
        });
    });
    c.bench_function("medium pulse", |b| {
        let mut buffer = [0.0; 2000];

        let mut sample = Sample::default();
        sample.env_attack(0.1);
        sample.env_decay(0.1);
        sample.env_release(0.1);

        let mut mixer = Mixer::new(2000);
        mixer.play(sample);

        b.iter(|| {
            mixer.generate(&mut buffer);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
