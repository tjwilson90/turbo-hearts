use crate::CardsReject;
use rusqlite::{Rows, ToSql};
use serde::{Deserialize, Serialize};
use std::mem;
use turbo_hearts_api::{
    Card, Cards, CardsError, ChargingRules, Database, Game, GameEvent, GameId, GameState,
    PassDirection, Player, Seat, UserId,
};
use warp::{Filter, Rejection, Reply};

const INITIAL_SCORES: &'static str = r#"
WITH ids AS
(
         SELECT   e.game_id
         FROM     event e,
                  game g
         WHERE    e.game_id = g.game_id
         AND      e.event_id = 0
         AND      g.completed_time IS NOT NULL
         AND      NOT e.event LIKE '%"type":"bot"%'
         ORDER BY g.completed_time DESC limit ?)
SELECT   game_id,
         timestamp,
         event
FROM     event
WHERE    game_id IN ids
ORDER BY game_id,
         event_id"#;

const PAGED_SCORES: &'static str = r#"
WITH ids AS
(
         SELECT   e.game_id
         FROM     event e,
                  game g
         WHERE    e.game_id = g.game_id
         AND      e.event_id = 0
         AND      g.completed_time IS NOT NULL
         AND      g.completed_time <
                  (
                         SELECT completed_time
                         FROM   game
                         WHERE  game_id = ?)
         AND      NOT e.event LIKE '%"type":"bot"%'
         ORDER BY g.completed_time DESC limit ?)
SELECT   game_id,
         timestamp,
         event
FROM     event
WHERE    game_id IN ids
ORDER BY game_id,
         event_id"#;

#[derive(Debug, Serialize)]
struct CompleteGame {
    game_id: GameId,
    completed_time: i64,
    players: [Player; 4],
    hands: Vec<CompleteHand>,
}

#[derive(Debug, Serialize)]
struct CompleteHand {
    charges: [Cards; 4],
    hearts_won: [u8; 4],
    queen_winner: UserId,
    ten_winner: UserId,
    jack_winner: UserId,
}

pub fn router<'a>(db: infallible!(&'a Database)) -> reply!() {
    warp::path("summary")
        .and(leaderboard(db.clone()).or(hand(db)))
        .boxed()
}

fn leaderboard<'a>(db: infallible!(&'a Database)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: Option<GameId>,
        page_size: Option<u32>,
    }

    async fn handle(db: &Database, request: Request) -> Result<impl Reply, Rejection> {
        let Request { game_id, page_size } = request;
        let page_size = page_size.unwrap_or(100);
        let games = db
            .run_read_only(|tx| {
                Ok(match game_id {
                    None => {
                        let mut stmt = tx.prepare_cached(INITIAL_SCORES)?;
                        let rows = stmt.query(&[page_size])?;
                        read_games(rows)?
                    }
                    Some(game_id) => {
                        let mut stmt = tx.prepare_cached(PAGED_SCORES)?;
                        let rows = stmt.query::<&[&dyn ToSql]>(&[&game_id, &page_size])?;
                        read_games(rows)?
                    }
                })
            })
            .map_err(CardsReject)?;
        Ok(warp::reply::json(&games))
    }

    warp::path!("leaderboard")
        .and(db)
        .and(warp::query())
        .and_then(handle)
}

fn read_games(mut rows: Rows<'_>) -> Result<Vec<CompleteGame>, rusqlite::Error> {
    let mut games = Vec::new();
    let mut hands = Vec::with_capacity(4);
    let mut state = GameState::new();
    while let Some(row) = rows.next()? {
        let game_id = row.get(0)?;
        let timestamp = row.get(1)?;
        let event = row.get(2)?;
        let was_playing = state.phase.is_playing();
        state.apply(&event);
        let is_playing = state.phase.is_playing();
        if was_playing && !is_playing {
            hands.push(CompleteHand {
                charges: [
                    state.charges.charges(Seat::North),
                    state.charges.charges(Seat::East),
                    state.charges.charges(Seat::South),
                    state.charges.charges(Seat::West),
                ],
                hearts_won: [
                    (state.won[0] & Cards::HEARTS).len() as u8,
                    (state.won[1] & Cards::HEARTS).len() as u8,
                    (state.won[2] & Cards::HEARTS).len() as u8,
                    (state.won[3] & Cards::HEARTS).len() as u8,
                ],
                queen_winner: winner_of(&state, Card::QueenSpades),
                ten_winner: winner_of(&state, Card::TenClubs),
                jack_winner: winner_of(&state, Card::JackDiamonds),
            });
            if hands.len() == 4 {
                let mut complete_hands = Vec::new();
                mem::swap(&mut hands, &mut complete_hands);
                games.push(CompleteGame {
                    game_id,
                    completed_time: timestamp,
                    players: [
                        Player::Human {
                            user_id: state.players[0],
                        },
                        Player::Human {
                            user_id: state.players[1],
                        },
                        Player::Human {
                            user_id: state.players[2],
                        },
                        Player::Human {
                            user_id: state.players[3],
                        },
                    ],
                    hands: complete_hands,
                });
                state = GameState::new();
            }
        }
    }
    games.sort_by_key(|game| -game.completed_time);
    Ok(games)
}

fn winner_of(state: &GameState, card: Card) -> UserId {
    for i in 0..4 {
        if state.won[i].contains(card) {
            return state.players[i];
        }
    }
    unreachable!()
}

fn hand<'a>(db: infallible!(&'a Database)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        hand: PassDirection,
    }

    #[derive(Debug, Serialize)]
    struct Response {
        north: Player,
        east: Player,
        south: Player,
        west: Player,
        rules: ChargingRules,
        events: Vec<Event>,
    }

    #[derive(Debug, Serialize)]
    struct Event {
        timestamp: i64,
        event: GameEvent,
        synthetic_events: Vec<GameEvent>,
    }

    async fn handle(
        game_id: GameId,
        hand: PassDirection,
        db: &Database,
    ) -> Result<impl Reply, Rejection> {
        let reply = db
            .run_read_only(|tx| {
                let mut stmt = tx.prepare_cached(
                    "SELECT timestamp, event FROM event WHERE game_id = ? ORDER BY event_id",
                )?;
                let mut rows = stmt.query(&[game_id])?;
                let mut game = Game::new();
                let mut start = false;
                let mut end = false;
                let mut events = Vec::new();
                while let Some(row) = rows.next()? {
                    let timestamp = row.get(0)?;
                    let event = row.get(1)?;
                    if !start {
                        start = match event {
                            GameEvent::Deal { pass, .. } if pass == hand => true,
                            _ => false,
                        }
                    }
                    let mut synthetic_events = Vec::new();
                    game.apply(&event, |_, e| {
                        if start && !end {
                            if e != &event {
                                synthetic_events.push(e.clone());
                            }
                            if let GameEvent::HandComplete { .. } = e {
                                end = true;
                            }
                        }
                    });
                    if start {
                        events.push(Event {
                            timestamp,
                            event,
                            synthetic_events,
                        });
                    }
                    if end {
                        break;
                    }
                }
                if !start {
                    Err(CardsError::UnknownHand(game_id, hand))
                } else if !end {
                    Err(CardsError::IncompleteHand(game_id, hand))
                } else {
                    if let GameEvent::Sit {
                        north,
                        east,
                        south,
                        west,
                        rules,
                        ..
                    } = &game.events[0]
                    {
                        Ok(Response {
                            north: *north,
                            east: *east,
                            south: *south,
                            west: *west,
                            rules: *rules,
                            events,
                        })
                    } else {
                        panic!("first event must be a sit event");
                    }
                }
            })
            .map_err(CardsReject)?;
        Ok(warp::reply::json(&reply))
    }

    warp::path!("hand" / GameId / PassDirection)
        .and(db)
        .and_then(handle)
}
