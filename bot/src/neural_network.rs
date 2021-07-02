use crate::{Algorithm, Encoder, HandMaker, HeuristicBot};
use log::debug;
use once_cell::sync::Lazy;
use rand::Rng;
use std::{collections::HashMap, error::Error, sync::Arc, time::Instant};
use tract_onnx::{
    prelude::{
        tract_ndarray::Array2, tvec, Datum, Framework, InferenceFact, InferenceModel,
        InferenceModelExt, TVec, Tensor, TypedModel, TypedRunnableModel,
    },
    tract_hir::tract_core::downcast_rs::__std::cmp::Ordering,
};
use turbo_hearts_api::{
    can_claim, BotState, Card, Cards, ChargeState, GameEvent, GameState, Rank, Seat, Suit,
    VoidState, WonState,
};

static LEAD_POLICY: Lazy<TypedRunnableModel<TypedModel>> =
    Lazy::new(|| load_model(true, true).unwrap());

static LEAD_VALUE: Lazy<TypedRunnableModel<TypedModel>> =
    Lazy::new(|| load_model(true, false).unwrap());

static FOLLOW_POLICY: Lazy<TypedRunnableModel<TypedModel>> =
    Lazy::new(|| load_model(false, true).unwrap());

static FOLLOW_VALUE: Lazy<TypedRunnableModel<TypedModel>> =
    Lazy::new(|| load_model(false, false).unwrap());

fn load_model(lead: bool, policy: bool) -> Result<TypedRunnableModel<TypedModel>, Box<dyn Error>> {
    let path = format!(
        "assets/{}-{}.onnx",
        if lead { "lead" } else { "follow" },
        if policy { "policy" } else { "value" }
    );
    let mut model: InferenceModel = tract_onnx::onnx().model_for_path(&path)?;
    model.set_input_fact(
        0,
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, Encoder::CARDS_LEN]),
    )?;
    model.set_input_fact(
        1,
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, Encoder::QUEEN_LEN]),
    )?;
    model.set_input_fact(
        2,
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, Encoder::JACK_LEN]),
    )?;
    model.set_input_fact(
        3,
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, Encoder::TEN_LEN]),
    )?;
    model.set_input_fact(
        4,
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, Encoder::HEARTS_LEN]),
    )?;
    model.set_input_fact(
        5,
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, Encoder::CHARGED_LEN]),
    )?;
    model.set_input_fact(
        6,
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, Encoder::LED_LEN]),
    )?;
    if !lead {
        model.set_input_fact(
            7,
            InferenceFact::dt_shape(f32::datum_type(), tvec![1, Encoder::TRICK_LEN]),
        )?;
    }
    Ok(model.into_optimized()?.into_runnable()?)
}

#[derive(Clone)]
pub struct NeuralNetworkBot {
    hand_maker: HandMaker,
    initial_state: GameState,
    plays: Vec<Card>,
}

impl NeuralNetworkBot {
    pub fn new() -> Self {
        Lazy::force(&LEAD_POLICY);
        Lazy::force(&LEAD_VALUE);
        Lazy::force(&FOLLOW_POLICY);
        Lazy::force(&FOLLOW_VALUE);
        Self {
            hand_maker: HandMaker::new(),
            initial_state: GameState::new(),
            plays: Vec::with_capacity(52),
        }
    }
}

impl Algorithm for NeuralNetworkBot {
    fn pass(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        HeuristicBot.pass(bot_state, game_state)
    }

    fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        HeuristicBot.charge(bot_state, game_state)
    }

    fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        let legal_plays = game_state.legal_plays(bot_state.post_pass_hand);
        let distinct_plays =
            legal_plays.distinct_plays(game_state.played, game_state.current_trick);
        if distinct_plays.len() == 1 {
            return choose(bot_state, distinct_plays.max(), legal_plays, distinct_plays);
        }
        let mut money_counts = HashMap::new();
        let mut iters = 0;
        let now = Instant::now();
        while now.elapsed().as_millis() < 4500 {
            iters += 1;
            let hands = self.hand_maker.make(&bot_state.void);
            let brute_force = ShallowBruteForce::new(hands);
            for card in distinct_plays {
                let mut game = game_state.clone();
                game.apply(&GameEvent::Play {
                    seat: bot_state.seat,
                    card,
                });
                let scores = brute_force.solve(&mut game);
                *money_counts.entry(card).or_default() += scores.money(bot_state.seat);
            }
        }
        if log::log_enabled!(log::Level::Debug) {
            debug!(
                "{} iterations, {:?}",
                iters,
                money_counts
                    .iter()
                    .map(|(k, v)| (k, *v / iters as f32))
                    .collect::<HashMap<_, _>>()
            );
        }
        let mut best_card = Card::TwoClubs;
        let mut best_money = f32::MIN;
        for (card, money) in money_counts.into_iter() {
            if money > best_money {
                best_money = money;
                best_card = card;
            }
        }
        choose(bot_state, best_card, legal_plays, distinct_plays)
    }

    fn on_event(&mut self, _: &BotState, game_state: &GameState, event: &GameEvent) {
        self.hand_maker.on_event(game_state, event);
        match event {
            GameEvent::StartTrick { leader } if self.plays.is_empty() => {
                self.initial_state = game_state.clone();
                self.initial_state.next_actor = Some(*leader);
            }
            GameEvent::Play { card, .. } => self.plays.push(*card),
            GameEvent::HandComplete { .. } => self.plays.clear(),
            _ => {}
        }
    }
}

struct ShallowBruteForce {
    hands: [Cards; 4],
}

impl ShallowBruteForce {
    fn new(hands: [Cards; 4]) -> Self {
        Self { hands }
    }

    fn solve(&self, state: &mut GameState) -> ApproximateScores {
        if state.played.len() >= 48 {
            while state.played != Cards::ALL {
                let seat = state.next_actor.unwrap();
                let card = (self.hands[seat.idx()] - state.played).max();
                state.apply(&GameEvent::Play { seat, card });
            }
            return ApproximateScores::from_won(state.charges, state.won);
        }
        let seat = state.next_actor.unwrap();
        if state.current_trick.is_empty()
            && state.won.can_run(seat)
            && can_claim(state, VoidState::new(), seat, self.hands[seat.idx()])
        {
            return ApproximateScores::from_won(state.charges, state.won.claim(seat));
        }
        if state.current_trick.is_empty() && state.played.len() < 36 {
            return self.generate_value(state);
        }
        let plays = self.generate_policy(state);
        if plays.len() == 1 {
            state.apply(&GameEvent::Play {
                seat,
                card: plays.max(),
            });
            return self.solve(state);
        }
        let mut best_scores = ApproximateScores::empty();
        let mut best_money = f32::MIN;
        for card in plays {
            let mut state = state.clone();
            state.apply(&GameEvent::Play { seat, card });
            let scores = self.solve(&mut state);
            let money = scores.money(seat);
            if money > best_money {
                best_scores = scores;
                best_money = money;
            }
        }
        best_scores
    }

    fn generate_policy(&self, game_state: &GameState) -> Cards {
        let seat = game_state.next_actor.unwrap();
        let legal = game_state.legal_plays(self.hands[seat.idx()]);
        let distinct = legal.distinct_plays(game_state.played, game_state.current_trick);
        if distinct.len() <= 3 {
            return distinct;
        }
        let input = self.model_input(game_state);
        let model = if game_state.current_trick.is_empty() {
            Lazy::force(&LEAD_POLICY)
        } else {
            Lazy::force(&FOLLOW_POLICY)
        };
        let output = model.run(input).unwrap();
        let mut output = output[0].as_slice::<f32>().unwrap().into_iter().cloned();

        let mut policies = Vec::new();
        for &suit in &Suit::VALUES {
            let chargeable = (suit.cards() & Cards::CHARGEABLE).max();
            let nine = suit.with_rank(Rank::Nine);
            let high = chargeable.above();
            let middle = nine.above() - high - chargeable;
            let low = nine.below();

            let mut cards = (high & legal).into_iter();
            for _ in 0..high.len() {
                let value = output.next().unwrap();
                if let Some(card) = cards.next() {
                    policies.push((card, value));
                }
            }
            let value = output.next().unwrap();
            if legal.contains(chargeable) {
                policies.push((chargeable, value));
            }

            let mut cards = (middle & legal).into_iter();
            for _ in 0..middle.len() {
                let value = output.next().unwrap();
                if let Some(card) = cards.next() {
                    policies.push((card, value));
                }
            }
            let value = output.next().unwrap();
            if legal.contains(nine) {
                policies.push((nine, value));
            }

            let mut cards = (low & legal).into_iter();
            for _ in 0..low.len() {
                let value = output.next().unwrap();
                if let Some(card) = cards.next() {
                    policies.push((card, value));
                }
            }
        }
        policies.sort_unstable_by(|p1, p2| {
            if p1.1 < p2.1 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
        let mut plays = Cards::NONE;
        while plays.len() < 3 {
            let card = policies.pop().unwrap().0;
            plays |= if distinct.contains(card) {
                card
            } else {
                (card.above() & distinct).min()
            };
        }
        plays
    }

    fn generate_value(&self, game_state: &mut GameState) -> ApproximateScores {
        let input = self.model_input(game_state);
        let model = if game_state.current_trick.is_empty() {
            Lazy::force(&LEAD_VALUE)
        } else {
            Lazy::force(&FOLLOW_VALUE)
        };
        let output = model.run(input).unwrap();
        ApproximateScores::from_model(&game_state, output)
    }

    fn model_input(&self, game_state: &GameState) -> TVec<Tensor> {
        let seat = game_state.next_actor.unwrap();
        let mut input = TVec::with_capacity(8);
        input.push(
            Array2::from_shape_vec(
                (1, Encoder::CARDS_LEN),
                Encoder::new(Encoder::CARDS_LEN)
                    .cards(seat, game_state.played, self.hands)
                    .into_inner(),
            )
            .unwrap()
            .into(),
        );
        input.push(
            Array2::from_shape_vec(
                (1, Encoder::QUEEN_LEN),
                Encoder::new(Encoder::QUEEN_LEN)
                    .queen(seat, game_state.won)
                    .into_inner(),
            )
            .unwrap()
            .into(),
        );
        input.push(
            Array2::from_shape_vec(
                (1, Encoder::JACK_LEN),
                Encoder::new(Encoder::JACK_LEN)
                    .jack(seat, game_state.won)
                    .into_inner(),
            )
            .unwrap()
            .into(),
        );
        input.push(
            Array2::from_shape_vec(
                (1, Encoder::TEN_LEN),
                Encoder::new(Encoder::TEN_LEN)
                    .ten(seat, game_state.won)
                    .into_inner(),
            )
            .unwrap()
            .into(),
        );
        input.push(
            Array2::from_shape_vec(
                (1, Encoder::HEARTS_LEN),
                Encoder::new(Encoder::HEARTS_LEN)
                    .hearts(seat, game_state.won)
                    .into_inner(),
            )
            .unwrap()
            .into(),
        );
        input.push(
            Array2::from_shape_vec(
                (1, Encoder::CHARGED_LEN),
                Encoder::new(Encoder::CHARGED_LEN)
                    .charged(game_state.charges)
                    .into_inner(),
            )
            .unwrap()
            .into(),
        );
        input.push(
            Array2::from_shape_vec(
                (1, Encoder::LED_LEN),
                Encoder::new(Encoder::LED_LEN)
                    .led(game_state.led_suits)
                    .into_inner(),
            )
            .unwrap()
            .into(),
        );
        if !game_state.current_trick.is_empty() {
            input.push(
                Array2::from_shape_vec(
                    (1, Encoder::TRICK_LEN),
                    Encoder::new(Encoder::TRICK_LEN)
                        .trick(seat, game_state.played, game_state.current_trick)
                        .into_inner(),
                )
                .unwrap()
                .into(),
            );
        }
        input
    }
}

#[derive(Debug)]
struct ApproximateScores {
    scores: [f32; 4],
}

impl ApproximateScores {
    fn empty() -> Self {
        Self { scores: [0.0; 4] }
    }

    fn from_won(charges: ChargeState, won: WonState) -> Self {
        let scores = won.scores(charges);
        Self {
            scores: [
                scores.score(Seat::North) as f32,
                scores.score(Seat::East) as f32,
                scores.score(Seat::South) as f32,
                scores.score(Seat::West) as f32,
            ],
        }
    }

    fn from_model(state: &GameState, output: TVec<Arc<Tensor>>) -> Self {
        let seat = state.next_actor.unwrap();
        let north = (4 - seat.idx()) % 4;
        let east = (5 - seat.idx()) % 4;
        let south = (6 - seat.idx()) % 4;
        let west = 3 - seat.idx();

        let mut hearts = {
            let hearts = output[3].as_slice::<f32>().unwrap();
            [hearts[north], hearts[east], hearts[south], hearts[west]]
        };
        let mut queen = if let Some(s) = state.won.queen_winner() {
            let mut queen = [0.0; 4];
            queen[s.idx()] = 1.0;
            queen
        } else {
            let queen = output[0].as_slice::<f32>().unwrap();
            [queen[north], queen[east], queen[south], queen[west]]
        };
        for i in 0..4 {
            if hearts[i] >= 0.97 && queen[i] >= 0.9 {
                hearts[i] *= -1.0;
                queen[i] *= -1.0;
            }
        }
        let qf = if state.charges.is_charged(Card::QueenSpades) {
            26.0
        } else {
            13.0
        };
        let hf = if state.charges.is_charged(Card::AceHearts) {
            26.0
        } else {
            13.0
        };
        let mut scores = [
            qf * queen[0] + hf * hearts[0],
            qf * queen[1] + hf * hearts[1],
            qf * queen[2] + hf * hearts[2],
            qf * queen[3] + hf * hearts[3],
        ];
        let jf = if state.charges.is_charged(Card::JackDiamonds) {
            -20.0
        } else {
            -10.0
        };
        if let Some(s) = state.won.jack_winner() {
            scores[s.idx()] += jf;
        } else {
            let jack = output[1].as_slice::<f32>().unwrap();
            scores[0] += jf * jack[north];
            scores[1] += jf * jack[east];
            scores[2] += jf * jack[south];
            scores[3] += jf * jack[west];
        };
        let tf = if state.charges.is_charged(Card::TenClubs) {
            3.0
        } else {
            1.0
        };
        if let Some(s) = state.won.ten_winner() {
            scores[s.idx()] *= 1.0 + tf;
        } else {
            let ten = output[2].as_slice::<f32>().unwrap();
            scores[0] *= 1.0 + tf * ten[north];
            scores[1] *= 1.0 + tf * ten[east];
            scores[2] *= 1.0 + tf * ten[south];
            scores[3] *= 1.0 + tf * ten[west];
        };
        Self { scores }
    }

    fn money(&self, seat: Seat) -> f32 {
        self.scores[0] + self.scores[1] + self.scores[2] + self.scores[3]
            - 4.0 * self.scores[seat.idx()]
    }
}

fn choose(bot_state: &BotState, card: Card, legal_plays: Cards, distinct_plays: Cards) -> Card {
    let other_plays = distinct_plays.below(card);
    let mut cards = if other_plays.is_empty() {
        legal_plays.below(card) | card
    } else {
        legal_plays.below(card).above(other_plays.max()) | card
    };
    let passed = cards & (bot_state.post_pass_hand - bot_state.pre_pass_hand);
    if !passed.is_empty() {
        cards = passed;
    }
    let index = rand::thread_rng().gen_range(0..cards.len());
    cards.into_iter().nth(index).unwrap()
}
