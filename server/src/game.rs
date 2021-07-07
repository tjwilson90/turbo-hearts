use crate::{util, BotRunner, CardsError, Database, GetJson, Subscriber, ToSqlJson, ToSqlStr};
use log::info;
use rand_distr::Gamma;
use rusqlite::{ToSql, Transaction};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
};
use tokio::{
    sync::{mpsc, mpsc::UnboundedReceiver, Mutex},
    task,
};
use turbo_hearts_api::{
    Card, Cards, GameEvent, GameId, GamePhase, HashedSeed, PassDirection, Player,
    PlayerWithOptions, Seat, Seed, UserId,
};

type Game = turbo_hearts_api::Game<Subscriber>;

#[derive(Clone)]
pub struct Games {
    db: &'static Database,
    bot_delay: Option<Gamma<f32>>,
    inner: Arc<Mutex<HashMap<GameId, Arc<Mutex<Game>>>>>,
}

impl Games {
    pub fn new(db: &'static Database, delay: bool) -> Self {
        Self {
            db,
            bot_delay: if delay {
                Some(Gamma::new(2.5, 0.8).unwrap())
            } else {
                None
            },
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn ping(&self) {
        let mut inner = self.inner.lock().await;
        let mut unwatched = Vec::new();
        for (game_id, game) in inner.iter() {
            let mut game = game.lock().await;
            broadcast(&mut game, &GameEvent::Ping);
            if game.subscribers.is_empty() {
                unwatched.push(*game_id);
            }
        }
        for game_id in unwatched {
            inner.remove(&game_id);
        }
    }

    async fn with_game<F, T>(&self, game_id: GameId, f: F) -> Result<T, CardsError>
    where
        F: FnOnce(&mut Game) -> Result<T, CardsError>,
    {
        let mut launch_bots = false;
        let game = {
            let mut inner = self.inner.lock().await;
            match inner.entry(game_id) {
                Entry::Occupied(entry) => Arc::clone(entry.get()),
                Entry::Vacant(entry) => {
                    let game = Arc::new(Mutex::new(Game::new()));
                    entry.insert(Arc::clone(&game));
                    launch_bots = true;
                    game
                }
            }
        };
        let mut game = game.lock().await;
        if game.events.is_empty() {
            self.db
                .run_read_only(|tx| hydrate_events(&tx, game_id, &mut game))?;
        }
        if game.events.is_empty() {
            Err(CardsError::UnknownGame(game_id))
        } else {
            if launch_bots {
                if let GameEvent::Sit {
                    north,
                    east,
                    south,
                    west,
                    ..
                } = game.events[0].clone()
                {
                    self.run_bot(game_id, Seat::North, north, &mut game);
                    self.run_bot(game_id, Seat::East, east, &mut game);
                    self.run_bot(game_id, Seat::South, south, &mut game);
                    self.run_bot(game_id, Seat::West, west, &mut game);
                }
            }
            f(&mut game)
        }
    }

    fn run_bot(&self, game_id: GameId, seat: Seat, player: Player, game: &mut Game) {
        if let Player::Bot { user_id, strategy } = player {
            let (tx, rx) = mpsc::unbounded_channel();
            let sub = Subscriber::new(tx, None);
            task::spawn(BotRunner::new(user_id, strategy).run(
                game_id,
                self.clone(),
                rx,
                self.bot_delay,
            ));
            self.replay_events(&game, &sub, Some(seat));
            game.bots.push((seat, sub));
            info!(
                "run_bot: game_id={}, seat={}, player={:?}",
                game_id, seat, player
            );
        }
    }

    pub fn start_game(
        &self,
        game_id: GameId,
        players: [PlayerWithOptions; 4],
        seed: Seed,
    ) -> Result<(), CardsError> {
        let hashed_seed = HashedSeed::from(&seed);
        let result = self.db.run_with_retry(|tx| {
            persist_events(
                &tx,
                game_id,
                0,
                &[
                    GameEvent::Sit {
                        north: players[0].player,
                        east: players[1].player,
                        south: players[2].player,
                        west: players[3].player,
                        rules: players[0].rules,
                        seed: seed.clone(),
                    },
                    hashed_seed.deal(PassDirection::Left),
                ],
            )
        });
        info!(
            "start_game: game_id={}, error={:?}",
            game_id,
            result.as_ref().err()
        );
        result
    }

    pub async fn subscribe(
        &self,
        game_id: GameId,
        user_id: UserId,
        last_event_id: Option<usize>,
    ) -> Result<UnboundedReceiver<(GameEvent, usize)>, CardsError> {
        let (tx, rx) = mpsc::unbounded_channel();
        let sub = Subscriber::new(tx, last_event_id);
        self.with_game(game_id, |game| {
            let seat = game.seat(user_id);
            let mut subscribers = game
                .subscribers
                .iter()
                .map(|(user_id, _)| *user_id)
                .collect::<HashSet<_>>();
            if subscribers.insert(user_id) {
                broadcast(game, &GameEvent::JoinGame { user_id });
            }
            self.replay_events(&game, &sub, seat);
            sub.send(GameEvent::EndReplay { subscribers });
            game.subscribers.push((user_id, sub));
            Ok(())
        })
        .await?;
        info!("subscribe: game_id={}, user_id={}", game_id, user_id);
        Ok(rx)
    }

    fn replay_events(&self, game: &Game, sub: &Subscriber, seat: Option<Seat>) {
        let mut copy = Game::new();
        for event in &game.events {
            copy.apply(event, |g, e| {
                sub.send(e.redact(seat, g.state.rules));
            });
        }
    }

    pub async fn pass_cards(
        &self,
        game_id: GameId,
        user_id: UserId,
        cards: Cards,
    ) -> Result<(), CardsError> {
        let result = self
            .with_game(game_id, |game| match game.seat(user_id) {
                None => Err(CardsError::InvalidPlayer(user_id, game_id)),
                Some(seat) => {
                    game.verify_pass(game_id, seat, cards)?;
                    let mut events = vec![GameEvent::SendPass { from: seat, cards }];
                    if game.state.phase != GamePhase::PassKeeper {
                        let sender = game.state.phase.pass_sender(seat);
                        if game.state.done.sent_pass(sender) {
                            events.push(GameEvent::RecvPass {
                                to: seat,
                                cards: game.pre_pass_hand[sender.idx()]
                                    - game.post_pass_hand[sender.idx()],
                            });
                        }
                        let receiver = game.state.phase.pass_receiver(seat);
                        if game.state.done.sent_pass(receiver) {
                            events.push(GameEvent::RecvPass {
                                to: receiver,
                                cards,
                            });
                        }
                    }
                    if game.state.phase == GamePhase::PassKeeper
                        && Seat::all(|s| s == seat || game.state.done.sent_pass(s))
                    {
                        let passes = Cards::ALL
                            - game.post_pass_hand[0]
                            - game.post_pass_hand[1]
                            - game.post_pass_hand[2]
                            - game.post_pass_hand[3]
                            | cards;
                        events.extend_from_slice(&game.seed.keeper_pass(passes));
                    }
                    self.db.run_with_retry(|tx| {
                        persist_events(&tx, game_id, game.events.len(), &events)
                    })?;
                    for event in events {
                        game.apply(&event, |g, e| broadcast(g, e));
                    }
                    Ok(())
                }
            })
            .await;
        info!(
            "pass: game_id={}, user_id={}, cards={}, error={:?}",
            game_id,
            user_id,
            cards,
            result.as_ref().err()
        );
        result
    }

    pub async fn charge_cards(
        &self,
        game_id: GameId,
        user_id: UserId,
        cards: Cards,
    ) -> Result<(), CardsError> {
        let result = self
            .with_game(game_id, |game| match game.seat(user_id) {
                None => Err(CardsError::InvalidPlayer(user_id, game_id)),
                Some(seat) => {
                    game.verify_charge(game_id, seat, cards)?;
                    let event = GameEvent::Charge { seat, cards };
                    self.db.run_with_retry(|tx| {
                        persist_events(&tx, game_id, game.events.len(), &[event.clone()])
                    })?;
                    game.apply(&event, |g, e| broadcast(g, e));
                    Ok(())
                }
            })
            .await;
        info!(
            "charge: game_id={}, user_id={}, cards={}, error={:?}",
            game_id,
            user_id,
            cards,
            result.as_ref().err()
        );
        result
    }

    pub async fn play_card(
        &self,
        game_id: GameId,
        user_id: UserId,
        card: Card,
    ) -> Result<bool, CardsError> {
        let result = self
            .with_game(game_id, |game| match game.seat(user_id) {
                None => Err(CardsError::InvalidPlayer(user_id, game_id)),
                Some(seat) => {
                    game.verify_play(game_id, seat, card)?;
                    let mut events = vec![GameEvent::Play { seat, card }];
                    let ends_hand = game.state.played | card == Cards::ALL;
                    if ends_hand {
                        if let Some(event) = game.deal_event() {
                            events.push(event);
                        }
                    }
                    self.db.run_with_retry(|tx| {
                        persist_events(&tx, game_id, game.events.len(), &events)?;
                        if ends_hand && game.state.phase == GamePhase::PlayKeeper {
                            finish_game(&tx, game_id)?;
                        }
                        Ok(())
                    })?;
                    for event in events {
                        game.apply(&event, |g, e| broadcast(g, e));
                    }
                    Ok(game.state.phase == GamePhase::Complete)
                }
            })
            .await;
        info!(
            "play: game_id={}, user_id={}, card={}, error={:?}",
            game_id,
            user_id,
            card,
            result.as_ref().err()
        );
        result
    }

    pub async fn claim(&self, game_id: GameId, user_id: UserId) -> Result<(), CardsError> {
        let result = self
            .with_game(game_id, |game| match game.seat(user_id) {
                None => Err(CardsError::InvalidPlayer(user_id, game_id)),
                Some(seat) => {
                    game.verify_claim(game_id, seat)?;
                    let event = GameEvent::Claim {
                        seat,
                        hand: game.post_pass_hand[seat.idx()] - game.state.played,
                    };
                    self.db.run_with_retry(|tx| {
                        persist_events(&tx, game_id, game.events.len(), &[event.clone()])
                    })?;
                    game.apply(&event, |g, e| broadcast(g, e));
                    Ok(())
                }
            })
            .await;
        info!(
            "claim: game_id={}, user_id={}, error={:?}",
            game_id,
            user_id,
            result.as_ref().err()
        );
        result
    }

    pub async fn accept_claim(
        &self,
        game_id: GameId,
        user_id: UserId,
        claimer: Seat,
    ) -> Result<bool, CardsError> {
        let result = self
            .with_game(game_id, |game| match game.seat(user_id) {
                None => Err(CardsError::InvalidPlayer(user_id, game_id)),
                Some(seat) => {
                    game.verify_accept_claim(game_id, claimer, seat)?;
                    let mut events = vec![GameEvent::AcceptClaim {
                        claimer,
                        acceptor: seat,
                    }];
                    let ends_hand = game
                        .state
                        .claims
                        .accept(claimer, seat)
                        .successfully_claimed(claimer);
                    if ends_hand {
                        if let Some(event) = game.deal_event() {
                            events.push(event);
                        }
                    }
                    self.db.run_with_retry(|tx| {
                        persist_events(&tx, game_id, game.events.len(), &events)?;
                        if ends_hand && game.state.phase == GamePhase::PlayKeeper {
                            finish_game(&tx, game_id)?;
                        }
                        Ok(())
                    })?;
                    for event in events {
                        game.apply(&event, |g, e| broadcast(g, e));
                    }
                    Ok(game.state.phase == GamePhase::Complete)
                }
            })
            .await;
        info!(
            "accept_claim: game_id={}, user_id={}, claimer={}, error={:?}",
            game_id,
            user_id,
            claimer,
            result.as_ref().err()
        );
        result
    }

    pub async fn reject_claim(
        &self,
        game_id: GameId,
        user_id: UserId,
        claimer: Seat,
    ) -> Result<(), CardsError> {
        let result = self
            .with_game(game_id, |game| match game.seat(user_id) {
                None => Err(CardsError::InvalidPlayer(user_id, game_id)),
                Some(seat) => {
                    game.verify_reject_claim(game_id, claimer)?;
                    let event = GameEvent::RejectClaim {
                        claimer,
                        rejector: seat,
                    };
                    self.db.run_with_retry(|tx| {
                        persist_events(&tx, game_id, game.events.len(), &[event.clone()])
                    })?;
                    game.apply(&event, |g, e| broadcast(g, e));
                    Ok(())
                }
            })
            .await;
        info!(
            "reject_claim: game_id={}, user_id={}, claimer={}, error={:?}",
            game_id,
            user_id,
            claimer,
            result.as_ref().err()
        );
        result
    }

    pub async fn chat(
        &self,
        game_id: GameId,
        user_id: UserId,
        message: String,
    ) -> Result<(), CardsError> {
        let result = self
            .with_game(game_id, |game| {
                let event = GameEvent::Chat { user_id, message };
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, game_id, game.events.len(), &[event.clone()])
                })?;
                game.apply(&event, |g, e| broadcast(g, e));
                Ok(())
            })
            .await;
        info!(
            "chat: game_id={}, user_id={}, error={:?}",
            game_id,
            user_id,
            result.as_ref().err()
        );
        result
    }
}

fn broadcast(game: &mut Game, event: &GameEvent) {
    let rules = game.state.rules;
    let players = game.players;
    let mut disconnects = HashSet::new();
    game.subscribers.retain(|(user_id, tx)| {
        let seat = seat(players, *user_id);
        if tx.send(event.redact(seat, rules)) {
            true
        } else {
            disconnects.insert(*user_id);
            false
        }
    });
    if !disconnects.is_empty() {
        for (user_id, _) in &game.subscribers {
            disconnects.remove(user_id);
        }
        for user_id in disconnects {
            broadcast(game, &GameEvent::LeaveGame { user_id });
        }
    }
    for (seat, bot) in &game.bots {
        bot.send(event.redact(Some(*seat), rules));
    }
}

fn seat(players: [UserId; 4], user_id: UserId) -> Option<Seat> {
    players
        .iter()
        .position(|&id| id == user_id)
        .map(|idx| Seat::VALUES[idx])
}

fn finish_game(tx: &Transaction, game_id: GameId) -> Result<(), CardsError> {
    tx.execute::<&[&dyn ToSql]>(
        "UPDATE game SET completed_time = ? WHERE game_id = ?",
        &[&util::timestamp(), &game_id.sql()],
    )?;
    Ok(())
}

pub fn persist_events(
    tx: &Transaction,
    game_id: GameId,
    event_id: usize,
    events: &[GameEvent],
) -> Result<(), CardsError> {
    let mut stmt = tx.prepare_cached(
        "INSERT INTO event (game_id, event_id, timestamp, event) VALUES (?, ?, ?, ?)",
    )?;
    let timestamp = util::timestamp();
    let mut event_id = event_id as isize;
    for event in events {
        stmt.execute::<&[&dyn ToSql]>(&[&game_id.sql(), &event_id, &timestamp, &event.sql()])?;
        event_id += 1;
    }
    Ok(())
}

fn hydrate_events(tx: &Transaction, game_id: GameId, game: &mut Game) -> Result<(), CardsError> {
    let mut stmt = tx.prepare_cached(
        "SELECT event FROM event WHERE game_id = ? AND event_id >= ? ORDER BY event_id",
    )?;
    let mut rows = stmt.query::<&[&dyn ToSql]>(&[&game_id.sql(), &(game.events.len() as i64)])?;
    while let Some(row) = rows.next()? {
        let event = row.get_json(0)?;
        game.apply(&event, |_, _| {});
    }
    Ok(())
}
