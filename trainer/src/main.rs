use flate2::{write::GzEncoder, Compression};
use rand::{seq::SliceRandom, Rng};
use std::{
    error::Error,
    fs,
    fs::File,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::SystemTime,
};
use turbo_hearts_api::{
    BotState, Card, Cards, ChargingRules, GameEvent, GamePhase, GameState, PassDirection, Seat,
    WonState,
};
use turbo_hearts_bot::{Algorithm, Bot, BruteForce, DuckBot, GottaTryBot, HeuristicBot, RandomBot};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = Arc::clone(&stop);
        ctrlc::set_handler(move || {
            stop.store(true, Ordering::SeqCst);
        })?;
    }
    let mut cnt = 0;
    let mut deck = Cards::ALL.into_iter().collect::<Vec<_>>();
    fs::create_dir_all("data")?;
    while !stop.load(Ordering::SeqCst) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let path = format!("data/partial-{}.gz", timestamp);
        let mut writer = Writer::new(&path)?;
        for _ in 0..3000 {
            deck.shuffle(&mut rand::thread_rng());
            let mut trainer = Trainer::new(&deck);
            trainer.run(&mut writer)?;
            cnt += 1;
            println!("{}", cnt);
            if stop.load(Ordering::SeqCst) {
                break;
            }
        }
        let output = writer.encoder.finish()?;
        output.sync_all()?;
        drop(output);
        fs::rename(path, format!("data/complete-{}.gz", timestamp))?
    }
    Ok(())
}

struct Writer {
    encoder: GzEncoder<File>,
}

impl Writer {
    fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let encoder = GzEncoder::new(File::create(path)?, Compression::default());
        Ok(Self { encoder })
    }

    fn write(
        &mut self,
        game_state: &GameState,
        hands: &[Cards; 4],
        won: &WonState,
        plays: &[i16],
    ) -> Result<(), Box<dyn Error>> {
        bincode::serialize_into(&mut self.encoder, game_state)?;
        bincode::serialize_into(&mut self.encoder, &hands[0].bits)?;
        bincode::serialize_into(&mut self.encoder, &hands[1].bits)?;
        bincode::serialize_into(&mut self.encoder, &hands[2].bits)?;
        bincode::serialize_into(&mut self.encoder, &hands[3].bits)?;
        bincode::serialize_into(&mut self.encoder, won)?;
        bincode::serialize_into(&mut self.encoder, plays)?;
        Ok(())
    }
}

fn bot() -> Bot {
    match rand::thread_rng().gen_range(0..4) {
        0 => Bot::Random(RandomBot::new()),
        1 => Bot::Duck(DuckBot::new()),
        2 => Bot::GottaTry(GottaTryBot::new()),
        _ => Bot::Heuristic(HeuristicBot::new()),
    }
}

fn bot_state(seat: Seat, deck: &[Card]) -> BotState {
    let hand = deck[13 * seat.idx()..13 * (seat.idx() + 1)]
        .iter()
        .cloned()
        .collect();
    BotState {
        seat,
        pre_pass_hand: hand,
        post_pass_hand: hand,
    }
}

struct Trainer {
    bot_states: [BotState; 4],
    bots: [Bot; 4],
    game_state: GameState,
}

impl Trainer {
    fn new(deck: &[Card]) -> Self {
        Self {
            bot_states: [
                bot_state(Seat::North, &deck),
                bot_state(Seat::East, &deck),
                bot_state(Seat::South, &deck),
                bot_state(Seat::West, &deck),
            ],
            bots: [bot(), bot(), bot(), bot()],
            game_state: GameState::new(),
        }
    }

    fn run(&mut self, writer: &mut Writer) -> Result<(), Box<dyn Error>> {
        self.deal();
        self.pass();
        self.charge();
        self.game_state.next_actor = self.bot_states.iter().find_map(|bot| {
            if bot.post_pass_hand.contains(Card::TwoClubs) {
                Some(bot.seat)
            } else {
                None
            }
        });
        for _ in 0..20 {
            self.play_one();
        }
        let mut brute_force = BruteForce::new([
            self.bot_states[0].post_pass_hand,
            self.bot_states[1].post_pass_hand,
            self.bot_states[2].post_pass_hand,
            self.bot_states[3].post_pass_hand,
        ]);
        for _ in 0..24 {
            self.play_one();
            let seat = self.game_state.next_actor.unwrap();
            let hand = self.bot_states[seat.idx()].post_pass_hand;
            let mut best_money = i16::MIN;
            let mut best_won = WonState::new();
            let mut plays = Vec::new();
            for card in self
                .game_state
                .legal_plays(hand)
                .distinct_plays(self.game_state.played, self.game_state.current_trick)
            {
                let mut state = self.game_state.clone();
                state.apply(&GameEvent::Play { seat, card });
                let won = brute_force.solve(&mut state);
                let money = won.scores(state.charges).money(seat);
                plays.push(money);
                if money > best_money {
                    best_money = money;
                    best_won = won;
                }
            }
            writer.write(&self.game_state, brute_force.hands(), &best_won, &plays)?;
        }
        Ok(())
    }

    fn deal(&mut self) {
        let pass_direction = PassDirection::from(rand::thread_rng().gen_range(0..4));
        self.game_state.phase = match pass_direction {
            PassDirection::Left => GamePhase::PassLeft,
            PassDirection::Right => GamePhase::PassRight,
            PassDirection::Across => GamePhase::PassAcross,
            PassDirection::Keeper => GamePhase::PassKeeper,
        };
        self.apply(&GameEvent::Deal {
            north: self.bot_states[0].pre_pass_hand,
            east: self.bot_states[1].pre_pass_hand,
            south: self.bot_states[2].pre_pass_hand,
            west: self.bot_states[3].pre_pass_hand,
            pass: pass_direction,
        });
    }

    fn pass(&mut self) {
        for i in 0..4 {
            let pass = self.bots[i].pass(&self.bot_states[i], &self.game_state);
            self.apply(&GameEvent::SendPass {
                from: self.bot_states[i].seat,
                cards: pass,
            });
            self.bot_states[i].post_pass_hand -= pass;
        }
        for i in 0..4 {
            let passer = match self.game_state.phase.direction() {
                PassDirection::Left => &self.bot_states[self.bot_states[i].seat.right().idx()],
                PassDirection::Right => &self.bot_states[self.bot_states[i].seat.left().idx()],
                PassDirection::Across => &self.bot_states[self.bot_states[i].seat.across().idx()],
                PassDirection::Keeper => &self.bot_states[i],
            };
            let pass = passer.pre_pass_hand - passer.post_pass_hand;
            self.apply(&GameEvent::RecvPass {
                to: self.bot_states[i].seat,
                cards: pass,
            });
            self.bot_states[i].post_pass_hand |= pass;
        }
    }

    fn charge(&mut self) {
        while self.game_state.phase.is_charging() {
            for i in 0..4 {
                if self.game_state.can_charge(self.bot_states[i].seat) {
                    let cards = self.bots[i].charge(&self.bot_states[i], &self.game_state);
                    let event = GameEvent::Charge {
                        seat: self.bot_states[i].seat,
                        cards,
                    };
                    self.apply(&event);
                }
            }
        }
    }

    fn play_one(&mut self) {
        let seat = self.game_state.next_actor.unwrap();
        let card = self.bots[seat.idx()].play(&self.bot_states[seat.idx()], &self.game_state);
        self.apply(&GameEvent::Play { seat, card });
    }

    fn apply(&mut self, event: &GameEvent) {
        for i in 0..4 {
            self.bots[i].on_event(
                &self.bot_states[i],
                &self.game_state,
                &event.redact(Some(self.bot_states[i].seat), ChargingRules::Classic),
            );
        }
        self.game_state.apply(event);
    }
}
