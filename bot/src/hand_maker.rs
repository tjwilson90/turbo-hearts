use rand::{seq::SliceRandom, RngExt};
use std::collections::HashMap;
use turbo_hearts_api::{Card, Cards, GameEvent, GameState, Seat, Suit, VoidState};

#[derive(Clone, Debug)]
pub struct HandMaker {
    hands: [Cards; 4],
}

impl HandMaker {
    pub fn new() -> Self {
        Self {
            hands: [Cards::NONE; 4],
        }
    }

    pub fn on_event(&mut self, state: &GameState, event: &GameEvent) {
        match event {
            GameEvent::Deal {
                north,
                east,
                south,
                west,
                ..
            } => {
                self.hands[0] = *north;
                self.hands[1] = *east;
                self.hands[2] = *south;
                self.hands[3] = *west;
            }
            GameEvent::SendPass { from, cards } => {
                self.hands[from.idx()] -= *cards;
                let recv = state.phase.pass_receiver(*from);
                if *from != recv {
                    self.hands[recv.idx()] |= *cards;
                }
            }
            GameEvent::RecvPass { to, cards } => self.hands[to.idx()] |= *cards,
            GameEvent::Charge { seat, cards } => self.hands[seat.idx()] |= *cards,
            GameEvent::RevealCharges {
                north,
                east,
                south,
                west,
            } => {
                self.hands[0] |= *north;
                self.hands[1] |= *east;
                self.hands[2] |= *south;
                self.hands[3] |= *west;
            }
            GameEvent::Play { seat, card } => self.hands[seat.idx()] |= *card,
            GameEvent::Claim { seat, hand } => self.hands[seat.idx()] |= *hand,
            GameEvent::HandComplete { .. } => self.hands = [Cards::NONE; 4],
            _ => {}
        }
    }

    pub fn known_cards(&self, seat: Seat) -> Cards {
        self.hands[seat.idx()]
    }

    pub fn make(&self, void: VoidState) -> [Cards; 4] {
        self.sampler(void).sample(&mut rand::rng())
    }

    /// Build once per decision, then reuse for all determinizations.
    pub fn sampler(&self, void: VoidState) -> HandSampler {
        HandSampler::new(self.hands, void)
    }
}

/// Uniform over complete deals consistent with known ownership and public voids.
/// Counts suit allocations rather than backtracking to the first feasible deal.
/// Played cards remain in their original owners' hands, so every hand has size 13.
pub struct HandSampler {
    known: [Cards; 4],
    unknown: [Vec<Card>; 4],
    capacities: [u8; 4],
    allowed: [[bool; 4]; 4],
    counts: HashMap<(usize, [u8; 4]), AllocationNode>,
    unconstrained: bool,
}

struct AllocationNode {
    total: f64,
    choices: Vec<([u8; 4], [u8; 4], f64)>,
}

impl HandSampler {
    fn new(known: [Cards; 4], void: VoidState) -> Self {
        let mut all = Cards::NONE;
        let mut capacities = [0; 4];
        for i in 0..4 {
            assert!(!all.contains_any(known[i]), "conflicting known ownership");
            assert!(known[i].len() <= 13, "too many known cards");
            all |= known[i];
            capacities[i] = (13 - known[i].len()) as u8;
        }
        let unknown = Suit::VALUES.map(|s| (s.cards() - all).into_iter().collect::<Vec<_>>());
        let allowed = Suit::VALUES.map(|s| Seat::VALUES.map(|p| !void.is_void(p, s)));
        let unconstrained = (0..4)
            .all(|s| unknown[s].is_empty() || (0..4).all(|p| capacities[p] == 0 || allowed[s][p]));
        let mut sampler = Self {
            known,
            unknown,
            capacities,
            allowed,
            counts: HashMap::new(),
            unconstrained,
        };
        if !unconstrained {
            assert!(
                sampler.count(0, capacities) > 0.0,
                "inconsistent hand constraints"
            );
        }
        sampler
    }

    fn allocations(&self, suit: usize, capacities: [u8; 4]) -> Vec<([u8; 4], f64)> {
        let n = self.unknown[suit].len() as u8;
        let cap = std::array::from_fn::<_, 4, _>(|i| {
            if self.allowed[suit][i] {
                capacities[i]
            } else {
                0
            }
        });
        let mut result = Vec::new();
        for a in 0..=n.min(cap[0]) {
            for b in 0..=(n - a).min(cap[1]) {
                for c in 0..=(n - a - b).min(cap[2]) {
                    let d = n - a - b - c;
                    if d <= cap[3] {
                        let weight = binomial(n, a) * binomial(n - a, b) * binomial(n - a - b, c);
                        result.push(([a, b, c, d], weight));
                    }
                }
            }
        }
        result
    }

    fn count(&mut self, suit: usize, capacities: [u8; 4]) -> f64 {
        if suit == 4 {
            return if capacities == [0; 4] { 1.0 } else { 0.0 };
        }
        if let Some(node) = self.counts.get(&(suit, capacities)) {
            return node.total;
        }
        let mut total = 0.0;
        let mut choices = Vec::new();
        for (allocation, weight) in self.allocations(suit, capacities) {
            let remaining = std::array::from_fn(|i| capacities[i] - allocation[i]);
            let mass = weight * self.count(suit + 1, remaining);
            total += mass;
            if mass > 0.0 {
                choices.push((allocation, remaining, mass));
            }
        }
        self.counts
            .insert((suit, capacities), AllocationNode { total, choices });
        total
    }

    pub fn sample<R: RngExt + ?Sized>(&mut self, rng: &mut R) -> [Cards; 4] {
        let mut hands = self.known;
        if self.unconstrained {
            let mut deck = [Card::TwoClubs; 52];
            let mut len = 0;
            for card in self.unknown.iter().flatten() {
                deck[len] = *card;
                len += 1;
            }
            let cards = &mut deck[..len];
            cards.shuffle(rng);
            let mut cards = cards.iter().copied();
            for i in 0..4 {
                hands[i] |= cards
                    .by_ref()
                    .take(self.capacities[i] as usize)
                    .collect::<Cards>();
            }
            return hands;
        }
        let mut capacities = self.capacities;
        for suit in 0..4 {
            // The conditional allocation distribution is constant throughout
            // this decision. Reuse its masses instead of allocating a vector
            // and looking up every descendant for every sampled hand.
            let node = self.counts.get(&(suit, capacities)).unwrap();
            let mut target = rng.random::<f64>() * node.total;
            let mut chosen = None;
            for &(allocation, remaining, mass) in &node.choices {
                chosen = Some((allocation, remaining));
                target -= mass;
                if target <= 0.0 {
                    break;
                }
            }
            let (allocation, remaining) = chosen.expect("feasible suit allocation");
            let mut deck = [Card::TwoClubs; 13];
            let cards = &mut deck[..self.unknown[suit].len()];
            cards.copy_from_slice(&self.unknown[suit]);
            cards.shuffle(rng);
            let mut cards = cards.iter().copied();
            for i in 0..4 {
                hands[i] |= cards
                    .by_ref()
                    .take(allocation[i] as usize)
                    .collect::<Cards>();
            }
            capacities = remaining;
        }
        hands
    }
}

fn binomial(n: u8, k: u8) -> f64 {
    (0..k).fold(1.0, |v, i| v * (n - i) as f64 / (i + 1) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha12Rng;

    #[test]
    fn uniform_constrained_deals() {
        let unknown: Cards = "23C 23D".parse().unwrap();
        let mut cards = (Cards::ALL - unknown).into_iter();
        let known = std::array::from_fn(|_| cards.by_ref().take(12).collect());
        let void = VoidState::new()
            .mark_void(Seat::North, Suit::Clubs)
            .mark_void(Seat::West, Suit::Diamonds);
        let mut sampler = HandSampler::new(known, void);
        assert_eq!(sampler.count(0, [1; 4]), 8.0);
        let mut counts = HashMap::new();
        let mut rng = ChaCha12Rng::seed_from_u64(1);
        for _ in 0..8000 {
            let hands = sampler.sample(&mut rng);
            let mut all = Cards::NONE;
            for i in 0..4 {
                assert_eq!(hands[i].len(), 13);
                assert!(hands[i].contains_all(known[i]));
                assert!(!all.contains_any(hands[i]));
                all |= hands[i];
                for card in hands[i] - known[i] {
                    assert!(!void.is_void(Seat::VALUES[i], card.suit()));
                }
            }
            assert_eq!(all, Cards::ALL);
            *counts
                .entry(hands.map(|hand| (hand & unknown).bits))
                .or_insert(0) += 1;
        }
        assert_eq!(counts.len(), 8);
        for count in counts.values() {
            assert!((850..1150).contains(count));
        }
    }

    #[test]
    fn tracks_passes_reveals_claims_and_resets() {
        let mut maker = HandMaker::new();
        let state = GameState::new();
        let hand: Cards = "AKQJT98765432S".parse().unwrap();
        maker.on_event(
            &state,
            &GameEvent::Deal {
                north: hand,
                east: Cards::NONE,
                south: Cards::NONE,
                west: Cards::NONE,
                pass: turbo_hearts_api::PassDirection::Left,
            },
        );
        let pass: Cards = "AKQS".parse().unwrap();
        maker.on_event(
            &state,
            &GameEvent::SendPass {
                from: Seat::North,
                cards: pass,
            },
        );
        assert_eq!(maker.known_cards(Seat::North), hand - pass);
        assert_eq!(maker.known_cards(Seat::East), pass);
        maker.on_event(
            &state,
            &GameEvent::Claim {
                seat: Seat::South,
                hand: Cards::HEARTS,
            },
        );
        assert_eq!(maker.known_cards(Seat::South), Cards::HEARTS);
        maker.on_event(
            &state,
            &GameEvent::HandComplete {
                north_score: 0,
                east_score: 0,
                south_score: 0,
                west_score: 0,
            },
        );
        for seat in Seat::VALUES {
            assert_eq!(maker.known_cards(seat), Cards::NONE);
        }
    }
}
