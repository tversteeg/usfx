use criterion::{criterion_group, criterion_main, Criterion};
use usfx::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("short pulse", |b| {
        let mut buffer = [0.0; 2000];

        let mut sample = Sample::default()
            .env_attack(0.01)
            .env_decay(0.01)
            .env_release(0.01)
            .build::<SineWave>();

        b.iter(|| {
            sample.generate(&mut buffer);
        });
    });
    c.bench_function("medium pulse", |b| {
        let mut buffer = [0.0; 2000];

        let mut sample = Sample::default()
            .env_attack(0.1)
            .env_decay(0.1)
            .env_release(0.1)
            .build::<SineWave>();

        b.iter(|| {
            sample.generate(&mut buffer);
        });
    });
    c.bench_function("long attack", |b| {
        let mut buffer = [0.0; 2000];

        let mut sample = Sample::default()
            .env_attack(1.0)
            .env_decay(0.0)
            .env_release(0.0)
            .build::<SineWave>();

        b.iter(|| {
            sample.generate(&mut buffer);
        });
    });
    c.bench_function("long decay", |b| {
        let mut buffer = [0.0; 2000];

        let mut sample = Sample::default()
            .env_attack(0.0)
            .env_decay(1.0)
            .env_release(0.0)
            .build::<SineWave>();

        b.iter(|| {
            sample.generate(&mut buffer);
        });
    });
    c.bench_function("long release", |b| {
        let mut buffer = [0.0; 2000];

        let mut sample = Sample::default()
            .env_attack(0.0)
            .env_decay(0.0)
            .env_release(1.0)
            .build::<SineWave>();

        b.iter(|| {
            sample.generate(&mut buffer);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
