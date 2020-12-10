use rand::{seq::SliceRandom, Rng};
use std::{
    error::Error,
    fs,
    fs::File,
    io::BufWriter,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
        mpsc::Sender,
        Arc,
    },
    time::SystemTime,
};
use tfrecord::{Example, ExampleWriter, Feature, RecordWriterInit};
use threadpool::ThreadPool;
use turbo_hearts_api::{
    BotState, Card, Cards, GameEvent, GamePhase, GameState, PassDirection, Seat,
};
use turbo_hearts_bot::{
    encoder, Algorithm, Bot, BruteForce, DuckBot, GottaTryBot, HeuristicBot, RandomBot, VoidState,
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = Arc::clone(&stop);
        ctrlc::set_handler(move || {
            stop.store(true, Ordering::SeqCst);
        })?;
    }
    let mut deck = Cards::ALL.into_iter().collect::<Vec<_>>();
    let mut writer = Writer::new()?;
    let pool = ThreadPool::default();
    let (tx, rx) = mpsc::channel();
    while !stop.load(Ordering::SeqCst) {
        if pool.active_count() + pool.queued_count() < pool.max_count() {
            deck.shuffle(&mut rand::thread_rng());
            let mut trainer = Trainer::new(&deck);
            let tx = tx.clone();
            pool.execute(move || trainer.run(tx));
        } else if let Ok((lead, record)) = rx.recv() {
            writer.write(lead, record)?;
        }
    }
    drop(tx);
    while let Ok((lead, record)) = rx.recv() {
        writer.write(lead, record)?;
    }
    Ok(())
}

struct Writer {
    train_lead: ExampleWriter<BufWriter<File>>,
    train_follow: ExampleWriter<BufWriter<File>>,
    validate_lead: ExampleWriter<BufWriter<File>>,
    validate_follow: ExampleWriter<BufWriter<File>>,
}

impl Writer {
    fn new() -> Result<Self, Box<dyn Error>> {
        fs::create_dir_all("data/train/lead").unwrap();
        fs::create_dir_all("data/train/follow").unwrap();
        fs::create_dir_all("data/validate/lead").unwrap();
        fs::create_dir_all("data/validate/follow").unwrap();
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        Ok(Self {
            train_lead: RecordWriterInit::create(format!("data/train/lead/{}.tfrec", timestamp))?,
            train_follow: RecordWriterInit::create(format!(
                "data/train/follow/{}.tfrec",
                timestamp
            ))?,
            validate_lead: RecordWriterInit::create(format!(
                "data/validate/lead/{}.tfrec",
                timestamp
            ))?,
            validate_follow: RecordWriterInit::create(format!(
                "data/validate/follow/{}.tfrec",
                timestamp
            ))?,
        })
    }

    fn write(&mut self, lead: bool, example: Example) -> Result<(), Box<dyn Error>> {
        Ok(match (lead, rand::thread_rng().gen_range(0.0, 1.0)) {
            (true, x) if x < 0.8 => self.train_lead.send(example)?,
            (true, _) => self.validate_lead.send(example)?,
            (_, x) if x < 0.8 => self.train_follow.send(example)?,
            _ => self.validate_follow.send(example)?,
        })
    }
}

fn bot() -> Bot {
    match rand::thread_rng().gen_range(0, 4) {
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
    void_state: VoidState,
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
            void_state: VoidState::new(),
        }
    }

    fn run(&mut self, tx: Sender<(bool, Example)>) {
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
        for _ in 0..rand::thread_rng().gen_range(24, 36) {
            self.play_one();
        }
        let mut brute_force = BruteForce::new([
            self.bot_states[0].post_pass_hand,
            self.bot_states[1].post_pass_hand,
            self.bot_states[2].post_pass_hand,
            self.bot_states[3].post_pass_hand,
        ]);
        self.report(&tx, &mut brute_force);
        if self.game_state.current_trick.is_empty() {
            for _ in 0..rand::thread_rng().gen_range(1, 3) {
                self.play_one();
            }
        } else {
            while !self.game_state.current_trick.is_empty() {
                self.play_one();
            }
        }
        self.report(&tx, &mut brute_force);
    }

    fn deal(&mut self) {
        let pass_direction = PassDirection::from(rand::thread_rng().gen_range(0, 4));
        self.game_state.phase = match pass_direction {
            PassDirection::Left => GamePhase::PassLeft,
            PassDirection::Right => GamePhase::PassRight,
            PassDirection::Across => GamePhase::PassAcross,
            PassDirection::Keeper => GamePhase::PassKeeper,
        };
        self.game_state.apply(&GameEvent::Deal {
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
            self.game_state.apply(&GameEvent::SendPass {
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
            self.game_state.apply(&GameEvent::RecvPass {
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
                    self.game_state.apply(&event);
                    self.bots[i].on_event(&self.bot_states[i], &self.game_state, &event);
                }
            }
        }
    }

    fn play_one(&mut self) {
        let seat = self.game_state.next_actor.unwrap();
        let card = self.bots[seat.idx()].play(&self.bot_states[seat.idx()], &self.game_state);
        let event = GameEvent::Play { seat, card };
        self.void_state.on_event(&self.game_state, &event);
        self.game_state.apply(&event);
    }

    fn report(&self, tx: &Sender<(bool, Example)>, brute_force: &mut BruteForce) {
        let won = brute_force.solve(&mut self.game_state.clone()).1;
        let seat = self.game_state.next_actor.unwrap();
        let mut record = Example::with_capacity(12);
        record.insert(
            "cards".to_string(),
            Feature::FloatList(encoder::cards(
                seat,
                self.game_state.played,
                [
                    self.bot_states[0].post_pass_hand,
                    self.bot_states[1].post_pass_hand,
                    self.bot_states[2].post_pass_hand,
                    self.bot_states[3].post_pass_hand,
                ],
            )),
        );
        record.insert(
            "won_queen".to_string(),
            Feature::FloatList(encoder::queen(seat, self.game_state.won)),
        );
        record.insert(
            "won_jack".to_string(),
            Feature::FloatList(encoder::jack(seat, self.game_state.won)),
        );
        record.insert(
            "won_ten".to_string(),
            Feature::FloatList(encoder::ten(seat, self.game_state.won)),
        );
        record.insert(
            "won_hearts".to_string(),
            Feature::FloatList(encoder::hearts(seat, self.game_state.won)),
        );
        record.insert(
            "charged".to_string(),
            Feature::FloatList(encoder::charged(self.game_state.charges)),
        );
        record.insert(
            "led".to_string(),
            Feature::FloatList(encoder::led(self.game_state.led_suits)),
        );
        if !self.game_state.current_trick.is_empty() {
            record.insert(
                "trick".to_string(),
                Feature::FloatList(encoder::trick(seat, self.game_state.current_trick)),
            );
        }
        record.insert(
            "win_queen".to_string(),
            Feature::FloatList(encoder::queen(seat, won)),
        );
        record.insert(
            "win_jack".to_string(),
            Feature::FloatList(encoder::jack(seat, won)),
        );
        record.insert(
            "win_ten".to_string(),
            Feature::FloatList(encoder::ten(seat, won)),
        );
        record.insert(
            "win_hearts".to_string(),
            Feature::FloatList(encoder::hearts(seat, won)),
        );
        tx.send((self.game_state.current_trick.is_empty(), record))
            .unwrap();
    }
}
