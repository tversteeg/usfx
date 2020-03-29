use criterion::{black_box, criterion_group, criterion_main, Criterion};
use usfx::{Sample, SineWave};

fn criterion_benchmark(c: &mut Criterion) {
    // Defaults with simple wave types

    c.bench_function("sine wave generate", |b| {
        b.iter(|| {
            black_box(Sample::default().build::<SineWave>());
        });
    });

    c.bench_function("sine wave generate", |b| {
        let mut buffer = [0.0; 2000];

        let mut sample = Sample::default().build::<SineWave>();

        b.iter(|| {
            sample.generate(&mut buffer);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
