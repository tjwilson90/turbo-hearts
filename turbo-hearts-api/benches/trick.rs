use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use turbo_hearts_api::{Card, Seat, Trick};

const CARDS: [Card; 8] = [
    Card::SixSpades,
    Card::QueenHearts,
    Card::NineSpades,
    Card::FiveSpades,
    Card::TwoSpades,
    Card::TenClubs,
    Card::FourSpades,
    Card::QueenSpades,
];

pub fn new(c: &mut Criterion) {
    c.bench_function("new", |b| b.iter(|| Trick::new()));
}

pub fn is_empty(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("is_empty", ""),
        &Trick::new(),
        |b, trick| {
            b.iter(|| trick.is_empty());
        },
    );
}

pub fn suit(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("suit", ""),
        &{
            let mut trick = Trick::new();
            trick.push(Card::FiveDiamonds);
            trick
        },
        |b, trick| {
            b.iter(|| trick.suit());
        },
    );
}

pub fn is_complete(c: &mut Criterion) {
    let mut g = c.benchmark_group("is_complete");
    g.bench_with_input(
        "length 3",
        &{
            let mut trick = Trick::new();
            trick.push(Card::FiveClubs);
            trick.push(Card::JackClubs);
            trick.push(Card::EightClubs);
            trick
        },
        |b, trick| {
            b.iter(|| trick.is_complete());
        },
    );
    g.bench_with_input(
        "length 4, no nine",
        &{
            let mut trick = Trick::new();
            trick.push(Card::FiveClubs);
            trick.push(Card::JackClubs);
            trick.push(Card::EightClubs);
            trick.push(Card::SevenSpades);
            trick
        },
        |b, trick| {
            b.iter(|| trick.is_complete());
        },
    );
    g.bench_with_input(
        "length 4, nine",
        &{
            let mut trick = Trick::new();
            trick.push(Card::FiveClubs);
            trick.push(Card::JackClubs);
            trick.push(Card::NineClubs);
            trick.push(Card::SevenSpades);
            trick
        },
        |b, trick| {
            b.iter(|| trick.is_complete());
        },
    );
    g.bench_with_input(
        "length 8",
        &{
            let mut trick = Trick::new();
            trick.push(Card::FiveClubs);
            trick.push(Card::JackClubs);
            trick.push(Card::NineClubs);
            trick.push(Card::SevenSpades);
            trick.push(Card::AceClubs);
            trick.push(Card::QueenSpades);
            trick.push(Card::ThreeClubs);
            trick.push(Card::EightSpades);
            trick
        },
        |b, trick| {
            b.iter(|| trick.is_complete());
        },
    );
    g.finish();
}

pub fn cards(c: &mut Criterion) {
    let mut g = c.benchmark_group("cards");
    let mut trick = Trick::new();
    g.bench_with_input("0", &trick, |b, trick| {
        b.iter(|| trick.cards());
    });
    for i in 0..8 {
        trick.push(CARDS[i]);
        g.bench_with_input(format!("{}", i + 1), &trick, |b, trick| {
            b.iter(|| trick.cards());
        });
    }
    g.finish();
}

pub fn winning_seat(c: &mut Criterion) {
    let mut g = c.benchmark_group("winning_seat");
    let mut trick = Trick::new();
    for i in 0..8 {
        trick.push(Card::from(i));
        g.bench_with_input(format!("{}", i + 1), &trick, |b, trick| {
            b.iter(|| trick.winning_seat(Seat::North));
        });
    }
    g.finish();
}

pub fn push(c: &mut Criterion) {
    c.bench_function("push", |b| {
        b.iter_batched(
            || {
                let mut trick = Trick::new();
                trick.push(Card::FiveDiamonds);
                trick
            },
            |mut trick| trick.push(Card::ThreeSpades),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    new,
    is_empty,
    suit,
    is_complete,
    cards,
    winning_seat,
    push,
);
criterion_main!(benches);
