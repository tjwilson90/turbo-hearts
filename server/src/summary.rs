use crate::{CardsReject, Database, Game};
use rusqlite::{Rows, ToSql};
use std::mem;
use turbo_hearts_api::{
    CardsError, CompleteGame, CompleteHand, GameEvent, GameId, GameState, HandEvent, HandResponse,
    LeaderboardRequest, LeaderboardResponse, PassDirection, Player, Seat, UserId,
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

pub fn router<'a>(db: infallible!(&'a Database)) -> reply!() {
    warp::path("summary")
        .and(leaderboard(db.clone()).or(hand(db)))
        .boxed()
}

fn leaderboard<'a>(db: infallible!(&'a Database)) -> reply!() {
    async fn handle(db: &Database, request: LeaderboardRequest) -> Result<impl Reply, Rejection> {
        let LeaderboardRequest { game_id, page_size } = request;
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

fn read_games(mut rows: Rows<'_>) -> Result<LeaderboardResponse, rusqlite::Error> {
    let mut games = Vec::new();
    let mut hands = Vec::with_capacity(4);
    let mut players = [Player::Human {
        user_id: UserId::null(),
    }; 4];
    let mut state = GameState::new();
    while let Some(row) = rows.next()? {
        let game_id = row.get(0)?;
        let timestamp = row.get(1)?;
        let event = row.get(2)?;
        let was_playing = state.phase.is_playing();
        state.apply(&event);
        if let GameEvent::Sit {
            north,
            east,
            south,
            west,
            ..
        } = event
        {
            players = [north, east, south, west];
        }
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
                    state.won.hearts(Seat::North),
                    state.won.hearts(Seat::East),
                    state.won.hearts(Seat::South),
                    state.won.hearts(Seat::West),
                ],
                queen_winner: players[state.won.queen_winner().unwrap().idx()].user_id(),
                ten_winner: players[state.won.ten_winner().unwrap().idx()].user_id(),
                jack_winner: players[state.won.jack_winner().unwrap().idx()].user_id(),
            });
            if hands.len() == 4 {
                let mut complete_hands = Vec::new();
                mem::swap(&mut hands, &mut complete_hands);
                games.push(CompleteGame {
                    game_id,
                    completed_time: timestamp,
                    players,
                    hands: complete_hands,
                });
                state = GameState::new();
            }
        }
    }
    games.sort_by_key(|game| -game.completed_time);
    Ok(games)
}

fn hand<'a>(db: infallible!(&'a Database)) -> reply!() {
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
                        events.push(HandEvent {
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
                        Ok(HandResponse {
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
