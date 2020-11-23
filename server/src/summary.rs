use crate::{CardsReject, Database, Game};
use rusqlite::{Rows, ToSql};
use std::mem;
use turbo_hearts_api::{
    CardsError, ChargingRules, GameEvent, GameEventsRequest, GameId, GameState, GameSummaryEvent,
    GameSummaryResponse, LeaderboardGame, LeaderboardHand, LeaderboardRequest, LeaderboardResponse,
    Player, Seat, UserId,
};
use warp::{Filter, Rejection, Reply};

const SELECT_GAME_EVENTS: &'static str = r#"
SELECT   game_id,
         timestamp,
         event
FROM     event
WHERE    game_id = ?
ORDER BY event_id"#;

const SELECT_FIRST_PAGE_OF_GAME_EVENTS: &'static str = r#"
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

const SELECT_NEXT_PAGE_OF_GAME_EVENTS: &'static str = r#"
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
        .and(leaderboard(db.clone()).or(game(db.clone())).or(games(db)))
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
                        let mut stmt = tx.prepare_cached(SELECT_FIRST_PAGE_OF_GAME_EVENTS)?;
                        let rows = stmt.query(&[page_size])?;
                        read_leaderboard(rows)?
                    }
                    Some(game_id) => {
                        let mut stmt = tx.prepare_cached(SELECT_NEXT_PAGE_OF_GAME_EVENTS)?;
                        let rows = stmt.query::<&[&dyn ToSql]>(&[&game_id, &page_size])?;
                        read_leaderboard(rows)?
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

fn read_leaderboard(mut rows: Rows<'_>) -> Result<LeaderboardResponse, rusqlite::Error> {
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
            hands.push(LeaderboardHand {
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
                let mut complete_hands = Vec::with_capacity(4);
                mem::swap(&mut hands, &mut complete_hands);
                games.push(LeaderboardGame {
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

fn game<'a>(db: infallible!(&'a Database)) -> reply!() {
    async fn handle(game_id: GameId, db: &Database) -> Result<impl Reply, Rejection> {
        let reply = db
            .run_read_only(|tx| {
                let mut stmt = tx.prepare_cached(SELECT_GAME_EVENTS)?;
                let rows = stmt.query(&[game_id])?;
                let mut games = read_games(rows)?;
                if let Some(game) = games.pop() {
                    Ok(game)
                } else {
                    Err(CardsError::IncompleteGame(game_id))
                }
            })
            .map_err(CardsReject)?;
        Ok(warp::reply::json(&reply))
    }

    warp::path!("game" / GameId).and(db).and_then(handle)
}

fn games<'a>(db: infallible!(&'a Database)) -> reply!() {
    async fn handle(db: &Database, request: GameEventsRequest) -> Result<impl Reply, Rejection> {
        let GameEventsRequest { game_id, page_size } = request;
        let page_size = page_size.unwrap_or(100);
        let games = db
            .run_read_only(|tx| {
                Ok(match game_id {
                    None => {
                        let mut stmt = tx.prepare_cached(SELECT_FIRST_PAGE_OF_GAME_EVENTS)?;
                        let rows = stmt.query(&[page_size])?;
                        read_games(rows)?
                    }
                    Some(game_id) => {
                        let mut stmt = tx.prepare_cached(SELECT_NEXT_PAGE_OF_GAME_EVENTS)?;
                        let rows = stmt.query::<&[&dyn ToSql]>(&[&game_id, &page_size])?;
                        read_games(rows)?
                    }
                })
            })
            .map_err(CardsReject)?;
        Ok(warp::reply::json(&games))
    }

    warp::path!("games")
        .and(db)
        .and(warp::query())
        .and_then(handle)
}

fn read_games(mut rows: Rows<'_>) -> Result<Vec<GameSummaryResponse>, CardsError> {
    let mut games = Vec::new();
    let mut hands = Vec::new();
    let mut game = Game::new();
    let mut events = Vec::new();
    let mut players = [Player::Human {
        user_id: UserId::null(),
    }; 4];
    let mut rules = ChargingRules::Classic;
    while let Some(row) = rows.next()? {
        let game_id = row.get(0)?;
        let timestamp = row.get(1)?;
        let event = row.get(2)?;
        if let GameEvent::Sit {
            north,
            east,
            south,
            west,
            rules: charging_rules,
            ..
        } = &event
        {
            hands = Vec::with_capacity(4);
            game = Game::new();
            players = [*north, *east, *south, *west];
            rules = *charging_rules;
        }
        let mut synthetic_events = Vec::new();
        let was_playing = game.state.phase.is_playing();
        game.apply(&event, |_, e| {
            if e != &event {
                synthetic_events.push(e.clone());
            }
        });
        events.push(GameSummaryEvent {
            timestamp,
            event,
            synthetic_events,
        });
        let is_playing = game.state.phase.is_playing();
        if was_playing && !is_playing {
            hands.push(events);
            events = Vec::new();
            if hands.len() == 4 {
                games.push(GameSummaryResponse {
                    game_id,
                    players,
                    rules,
                    hands,
                });
                hands = Vec::new();
                game = Game::new();
            }
        }
    }
    Ok(games)
}
