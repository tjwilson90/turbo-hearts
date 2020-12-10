use crate::{encoder, Algorithm, HandMaker, HeuristicBot, VoidState};
use log::{debug, Level};
use once_cell::sync::Lazy;
use rand::Rng;
use std::{collections::HashMap, error::Error, sync::Arc, time::Instant};
use tract_onnx::prelude::{
    tract_ndarray::Array2, tvec, Datum, Framework, InferenceFact, InferenceModel,
    InferenceModelExt, TVec, Tensor, TypedModel, TypedRunnableModel,
};
use turbo_hearts_api::{BotState, Card, Cards, GameEvent, GameState, Seat, WonState};

static LEAD_MODEL: Lazy<TypedRunnableModel<TypedModel>> =
    Lazy::new(|| load_model(true, "assets/lead-model.onnx").unwrap());

static FOLLOW_MODEL: Lazy<TypedRunnableModel<TypedModel>> =
    Lazy::new(|| load_model(false, "assets/follow-model.onnx").unwrap());

fn load_model(lead: bool, model: &str) -> Result<TypedRunnableModel<TypedModel>, Box<dyn Error>> {
    let mut model: InferenceModel = tract_onnx::onnx().model_for_path(model)?;
    model.set_input_fact(
        0, // cards
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, 260]),
    )?;
    model.set_input_fact(
        1, // won_queen
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, 4]),
    )?;
    model.set_input_fact(
        2, // won_jack
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, 4]),
    )?;
    model.set_input_fact(
        3, // won_ten
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, 4]),
    )?;
    model.set_input_fact(
        4, // won_hearts
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, 4]),
    )?;
    model.set_input_fact(
        5, // charged
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, 4]),
    )?;
    model.set_input_fact(
        6, // led
        InferenceFact::dt_shape(f32::datum_type(), tvec![1, 3]),
    )?;
    if !lead {
        model.set_input_fact(
            7, // trick
            InferenceFact::dt_shape(f32::datum_type(), tvec![1, 59]),
        )?;
    }
    Ok(model.into_optimized()?.into_runnable()?)
}

pub struct NeuralNetworkBot {
    void: VoidState,
}

impl NeuralNetworkBot {
    pub fn new() -> Self {
        Lazy::force(&LEAD_MODEL);
        Lazy::force(&FOLLOW_MODEL);
        Self {
            void: VoidState::new(),
        }
    }

    fn play_blocking(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        let legal_plays = game_state.legal_plays(bot_state.post_pass_hand);
        let distinct_plays =
            legal_plays.distinct_plays(game_state.played, game_state.current_trick);
        if distinct_plays.len() == 1 {
            return choose(bot_state, distinct_plays.max(), legal_plays, distinct_plays);
        }
        let hand_maker = HandMaker::new(&bot_state, &game_state, self.void.clone());
        let mut money_counts = HashMap::new();
        let mut iters = 0;
        let deadline = match (bot_state.post_pass_hand - game_state.played).len() {
            x if x < 3 => 2500,
            x if x < 9 => 3500,
            _ => 4500,
        };
        let now = Instant::now();
        while now.elapsed().as_millis() < deadline {
            iters += 1;
            let hands = hand_maker.make();
            for card in distinct_plays {
                let mut game = game_state.clone();
                game.apply(&GameEvent::Play {
                    seat: bot_state.seat,
                    card,
                });
                let mut brute_force = ShallowBruteForce::new(hands);
                let scores = brute_force.solve(distinct_plays.len(), &mut game);
                *money_counts.entry(card).or_default() += scores.money(bot_state.seat);
            }
        }
        if log::log_enabled!(Level::Debug) {
            debug!(
                "{} iterations, {:?}",
                iters,
                money_counts
                    .iter()
                    .map(|(c, m)| (*c, *m / iters as f32))
                    .collect::<Vec<_>>()
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
}

impl Algorithm for NeuralNetworkBot {
    fn pass(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        HeuristicBot::new().pass(bot_state, game_state)
    }

    fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        HeuristicBot::new().charge(bot_state, game_state)
    }

    fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        tokio::task::block_in_place(move || self.play_blocking(&bot_state, &game_state))
    }

    fn on_event(&mut self, _: &BotState, game_state: &GameState, event: &GameEvent) {
        self.void.on_event(game_state, event)
    }
}

struct ShallowBruteForce {
    hands: [Cards; 4],
}

impl ShallowBruteForce {
    fn new(hands: [Cards; 4]) -> Self {
        Self { hands }
    }

    fn solve(&mut self, depth: usize, state: &mut GameState) -> ApproximateScores {
        if (depth >= 32 && state.played.len() < 36) || state.played.len() >= 48 {
            return self.evaluate(state);
        }
        let seat = state.next_actor.unwrap();
        let plays = state
            .legal_plays(self.hands[seat.idx()])
            .distinct_plays(state.played, state.current_trick);
        if plays.len() == 1 {
            state.apply(&GameEvent::Play {
                seat,
                card: plays.max(),
            });
            return self.solve(depth, state);
        }
        let mut best_scores = ApproximateScores::empty();
        let mut best_money = f32::MIN;
        for card in plays {
            let mut state = state.clone();
            state.apply(&GameEvent::Play { seat, card });
            let scores = self.solve(depth * plays.len(), &mut state);
            let money = scores.money(seat);
            if money > best_money {
                best_scores = scores;
                best_money = money;
            }
        }
        best_scores
    }

    fn evaluate(&self, game_state: &mut GameState) -> ApproximateScores {
        if game_state.played.len() >= 48 {
            while game_state.played != Cards::ALL {
                let seat = game_state.next_actor.unwrap();
                let card = (self.hands[seat.idx()] - game_state.played).max();
                game_state.apply(&GameEvent::Play { seat, card });
            }
            return ApproximateScores::from_won(game_state.charges.all_charges(), game_state.won);
        }
        let seat = game_state.next_actor.unwrap();
        let mut input = TVec::with_capacity(8);
        input.push(
            Array2::from_shape_vec(
                (1, 260),
                encoder::cards(seat, game_state.played, self.hands),
            )
            .unwrap()
            .into(),
        );
        input.push(
            Array2::from_shape_vec((1, 4), encoder::queen(seat, game_state.won))
                .unwrap()
                .into(),
        );
        input.push(
            Array2::from_shape_vec((1, 4), encoder::jack(seat, game_state.won))
                .unwrap()
                .into(),
        );
        input.push(
            Array2::from_shape_vec((1, 4), encoder::ten(seat, game_state.won))
                .unwrap()
                .into(),
        );
        input.push(
            Array2::from_shape_vec((1, 4), encoder::hearts(seat, game_state.won))
                .unwrap()
                .into(),
        );
        input.push(
            Array2::from_shape_vec((1, 4), encoder::charged(game_state.charges))
                .unwrap()
                .into(),
        );
        input.push(
            Array2::from_shape_vec((1, 3), encoder::led(game_state.led_suits))
                .unwrap()
                .into(),
        );
        let output = if game_state.current_trick.is_empty() {
            LEAD_MODEL.run(input).unwrap()
        } else {
            input.push(
                Array2::from_shape_vec((1, 59), encoder::trick(seat, game_state.current_trick))
                    .unwrap()
                    .into(),
            );
            FOLLOW_MODEL.run(input).unwrap()
        };
        ApproximateScores::from_model(game_state.charges.all_charges(), seat, output)
    }
}

struct ApproximateScores {
    scores: [f32; 4],
}

impl ApproximateScores {
    fn empty() -> Self {
        Self { scores: [0.0; 4] }
    }

    fn from_won(charged: Cards, won: WonState) -> Self {
        let scores = won.scores(charged);
        Self {
            scores: [
                scores.score(Seat::North) as f32,
                scores.score(Seat::East) as f32,
                scores.score(Seat::South) as f32,
                scores.score(Seat::West) as f32,
            ],
        }
    }

    fn from_model(charged: Cards, seat: Seat, output: TVec<Arc<Tensor>>) -> Self {
        let queen = max_seat(seat, output[0].as_slice::<f32>().unwrap());
        let jack = max_seat(seat, output[1].as_slice::<f32>().unwrap());
        let ten = max_seat(seat, output[2].as_slice::<f32>().unwrap());
        let hearts = output[3].as_slice::<f32>().unwrap();
        let hearts_max = max_seat(seat, hearts);
        let heart_multiplier = if charged.contains(Card::AceHearts) {
            26.0
        } else {
            13.0
        };
        let mut scores = [
            heart_multiplier * hearts[(4 - seat.idx()) % 4],
            heart_multiplier * hearts[(5 - seat.idx()) % 4],
            heart_multiplier * hearts[(6 - seat.idx()) % 4],
            heart_multiplier * hearts[3 - seat.idx()],
        ];
        scores[queen.idx()] += if charged.contains(Card::QueenSpades) {
            26.0
        } else {
            13.0
        };
        if queen == hearts_max && hearts[hearts_max.idx()] > 0.94 {
            scores[queen.idx()] *= -1.0;
        }
        scores[jack.idx()] += if charged.contains(Card::JackDiamonds) {
            -20.0
        } else {
            -10.0
        };
        scores[ten.idx()] *= if charged.contains(Card::TenClubs) {
            4.0
        } else {
            2.0
        };
        Self { scores }
    }

    fn money(&self, seat: Seat) -> f32 {
        self.scores[0] + self.scores[1] + self.scores[2] + self.scores[3]
            - 4.0 * self.scores[seat.idx()]
    }
}

fn max_seat(seat: Seat, values: &[f32]) -> Seat {
    let mut idx = 0;
    let mut max = f32::MIN;
    for (i, value) in values.into_iter().cloned().enumerate() {
        if value > max {
            max = value;
            idx = i;
        }
    }
    Seat::VALUES[(idx + seat.idx()) % 4]
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
    let index = rand::thread_rng().gen_range(0, cards.len());
    cards.into_iter().nth(index).unwrap()
}
