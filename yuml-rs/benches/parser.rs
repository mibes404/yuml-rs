use criterion::{criterion_group, criterion_main, Criterion};
use yuml_rs::parse_yuml;

pub fn criterion_benchmark(c: &mut Criterion) {
    let yuml = include_str!("../test/activity.yuml");
    c.bench_function("activity.yuml", |b| b.iter(|| parse_yuml(yuml)));

    let yuml = include_str!("../test/class.yuml");
    c.bench_function("class.yuml", |b| b.iter(|| parse_yuml(yuml)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
