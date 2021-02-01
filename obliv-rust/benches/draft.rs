use criterion::{criterion_group, criterion_main, Criterion};
use obliv_rust::draft::greet;

fn bench_greet() {
    greet();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("greet", |b| b.iter(|| bench_greet()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
