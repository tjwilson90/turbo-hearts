use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use turbo_hearts_api::{Card, GameEvent, GameState, Seat};

pub fn apply(c: &mut Criterion) {
    let mut g = c.benchmark_group("apply");
    let state = GameState::new();
    let event = GameEvent::Play {
        seat: Seat::North,
        card: Card::FiveClubs,
    };
    g.bench_with_input(
        BenchmarkId::new("play", ""),
        &(state, event),
        |b, (state, event)| {
            b.iter_batched(
                || state.clone(),
                |mut state| state.apply(event),
                BatchSize::SmallInput,
            );
        },
    );
}

criterion_group!(benches, apply);
criterion_main!(benches);
