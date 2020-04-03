use criterion::{criterion_group, criterion_main, Criterion};
use usfx::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("distortion", |b| {
        let mut buffer = [0.0; 2000];

        let mut sample = Sample::default();
        sample.env_attack(10.0);
        sample.env_decay(10.0);
        sample.env_release(10.0);
        sample.dis_crunch(1.0);
        sample.dis_drive(0.8);

        let mut mixer = Mixer::new(2000);
        mixer.play(sample);

        b.iter(|| {
            mixer.generate(&mut buffer);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
