use criterion::{criterion_group, criterion_main, Criterion};
use usfx::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("mixer generate 1", |b| {
        let mut buffer = [0.0; 2000];

        let mut mixer = Mixer::default();
        // Give it a long attack so it keeps running
        mixer.play(Sample::default().env_attack(100.0).build::<SineWave>());

        b.iter(|| {
            mixer.generate(&mut buffer);
        });
    });
    c.bench_function("mixer generate 100", |b| {
        let mut buffer = [0.0; 2000];

        let mut mixer = Mixer::default();
        for i in 1..100 {
            mixer.play(
                Sample::default()
                    // Give it a long attack so it keeps running
                    .env_attack(i as f32 * 100.0)
                    .build::<SineWave>(),
            );
        }

        b.iter(|| {
            mixer.generate(&mut buffer);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
