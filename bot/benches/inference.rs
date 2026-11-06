use criterion::{black_box, criterion_group, criterion_main, Criterion};
use turbo_hearts_bot::{PolicyNetwork, POLICY_FEATURES};

fn inference(c: &mut Criterion) {
    let model =
        PolicyNetwork::from_bytes(include_bytes!("../../assets/models/policy-avoid.thp")).unwrap();
    let input = std::array::from_fn::<_, POLICY_FEATURES, _>(|i| (i % 7) as f32 / 7.0);
    c.bench_function("policy inference", |b| {
        b.iter(|| model.predict(black_box(&input)))
    });
}

criterion_group!(benches, inference);
criterion_main!(benches);
