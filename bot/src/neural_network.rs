use crate::{encoder, Algorithm, HandMaker, HeuristicBot};
use log::debug;
use once_cell::sync::Lazy;
use rand::Rng;
use std::{collections::HashMap, error::Error, sync::Arc, time::Instant};
use tract_onnx::prelude::{
    tract_ndarray::Array2, tvec, Datum, Framework, InferenceFact, InferenceModel,
    InferenceModelExt, TVec, Tensor, TypedModel, TypedRunnableModel,
};
use turbo_hearts_api::{BotState, Card, Cards, ChargeState, GameEvent, GameState, Seat, WonState};

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
            InferenceFact::dt_shape(f32::datum_type(), tvec![1, 62]),
        )?;
    }
    Ok(model.into_optimized()?.into_runnable()?)
}

pub struct NeuralNetworkBot {
    hand_maker: HandMaker,
    initial_state: GameState,
    plays: Vec<Card>,
}

impl NeuralNetworkBot {
    pub fn new() -> Self {
        Lazy::force(&LEAD_MODEL);
        Lazy::force(&FOLLOW_MODEL);
        Self {
            hand_maker: HandMaker::new(),
            initial_state: GameState::new(),
            plays: Vec::with_capacity(52),
        }
    }

    fn total_divergence(&self, hands: [Cards; 4]) -> f32 {
        let brute_force = ShallowBruteForce::new(hands);
        let mut state = self.initial_state.clone();
        let mut divergence = 1.0;
        for &play in &self.plays {
            let seat = state.next_actor.unwrap();
            let plays = state
                .legal_plays(hands[seat.idx()])
                .distinct_plays(state.played, state.current_trick);
            if plays.len() > 1 {
                divergence += local_divergence(&brute_force, &state, seat, play, plays);
            }
            state.apply(&GameEvent::Play { seat, card: play });
        }
        divergence
    }

    fn play_blocking(&self, bot_state: &BotState, game_state: &GameState) -> Card {
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
            let hands = self.hand_maker.make();
            let divergence = self.total_divergence(hands);
            for card in distinct_plays {
                let mut game = game_state.clone();
                game.apply(&GameEvent::Play {
                    seat: bot_state.seat,
                    card,
                });
                let mut brute_force = ShallowBruteForce::new(hands);
                let scores = brute_force.solve(distinct_plays.len(), &mut game);
                *money_counts.entry(card).or_default() += scores.money(bot_state.seat) / divergence;
            }
        }
        debug!("{} iterations, {:?}", iters, money_counts);
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

    fn solve(&mut self, depth: usize, state: &mut GameState) -> ApproximateScores {
        if state.played.len() >= 48
            || ((depth >= 32 || state.current_trick.is_empty()) && state.played.len() < 36)
        {
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
            return ApproximateScores::from_won(game_state.charges, game_state.won);
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
                Array2::from_shape_vec((1, 62), encoder::trick(seat, game_state.current_trick))
                    .unwrap()
                    .into(),
            );
            FOLLOW_MODEL.run(input).unwrap()
        };
        ApproximateScores::from_model(&game_state, output)
    }
}

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
            scores[s.idx()] *= tf * 2.0;
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
    let index = rand::thread_rng().gen_range(0, cards.len());
    cards.into_iter().nth(index).unwrap()
}

fn local_divergence(
    brute_force: &ShallowBruteForce,
    state: &GameState,
    seat: Seat,
    play: Card,
    plays: Cards,
) -> f32 {
    let equivalent = if plays.contains(play) {
        play
    } else {
        plays.above(play).min()
    };
    let mut best_money = f32::MIN;
    for card in plays - equivalent {
        let scores = brute_force.evaluate(&mut {
            let mut state = state.clone();
            state.apply(&GameEvent::Play { seat, card });
            state
        });
        let money = scores.money(seat);
        if money > best_money {
            best_money = money;
        }
    }
    let scores = brute_force.evaluate(&mut {
        let mut state = state.clone();
        state.apply(&GameEvent::Play { seat, card: play });
        state
    });
    f32::max(best_money - scores.money(seat), 0.0)
}
