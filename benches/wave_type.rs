use criterion::{black_box, criterion_group, criterion_main, Criterion};
use usfx::*;

fn criterion_benchmark(c: &mut Criterion) {
    // All wavetypes have the same generate function
    c.bench_function("generate", |b| {
        let mut buffer = [0.0; 2000];

        let mut sample = Sample::default().build::<SineWave>();

        b.iter(|| {
            sample.generate(&mut buffer);
        });
    });

    c.bench_function("sine wave setup", |b| {
        b.iter(|| {
            black_box(Sample::default().build::<SineWave>());
        });
    });
    c.bench_function("saw wave setup", |b| {
        b.iter(|| {
            black_box(Sample::default().build::<SawWave>());
        });
    });
    c.bench_function("square wave setup", |b| {
        b.iter(|| {
            black_box(Sample::default().build::<SquareWave>());
        });
    });
    c.bench_function("triangle wave setup", |b| {
        b.iter(|| {
            black_box(Sample::default().build::<TriangleWave>());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
