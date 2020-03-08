use crate::{
    card::Card,
    cards::Cards,
    db::Database,
    game::{id::GameId, state::GameState},
    player::Player,
    user::UserId,
};
use rusqlite::{Rows, NO_PARAMS};
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

#[derive(Debug, Serialize)]
struct CompleteGames {
    games: Vec<CompleteGame>,
}

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

pub fn scores(db: infallible!(Database)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: Option<GameId>,
    }

    async fn handle(db: Database, request: Request) -> Result<impl Reply, Rejection> {
        let Request { game_id } = request;
        let games = db.run_read_only(|tx| {
            let game_ids = match game_id {
                None => {
                    let mut stmt = tx.prepare(
                        r#"
                        SELECT   e.game_id
                        FROM     event e,
                                 game g,
                                 game_player gp
                        WHERE    e.game_id = g.game_id
                        AND      e.game_id = gp.game_id
                        AND      e.event_id = 0
                        AND      g.completed_time IS NOT NULL
                        AND      NOT e.event LIKE '%"type":"bot"'
                        ORDER BY g.completed_time DESC limit 100"#,
                    )?;
                    let rows = stmt.query(NO_PARAMS)?;
                    read_game_ids(rows)?
                }
                Some(game_id) => {
                    let mut stmt = tx.prepare(
                        r#"
                        SELECT e.game_id
                        FROM   event e,
                               game g,
                               game_player gp
                        WHERE  e.game_id = g.game_id
                               AND e.game_id = gp.game_id
                               AND e.event_id = 0
                               AND g.completed_time IS NOT NULL
                               AND g.completed_time < (SELECT completed_time
                                                       FROM   game
                                                       WHERE  game_id = ?)
                               AND NOT e.event LIKE '%"type":"bot"'
                        ORDER  BY g.completed_time DESC
                        LIMIT  100"#,
                    )?;
                    let rows = stmt.query(&[game_id])?;
                    read_game_ids(rows)?
                }
            };
            let mut games = Vec::with_capacity(game_ids.len());
            for game_id in game_ids {
                let mut stmt = tx.prepare_cached(
                    "SELECT timestamp, event FROM event WHERE game_id = ? ORDER BY event_id",
                )?;
                let mut rows = stmt.query(&[game_id])?;
                let mut hands = Vec::with_capacity(4);
                let mut state = GameState::new();
                let mut timestamp = 0;
                while let Some(row) = rows.next()? {
                    timestamp = row.get(0)?;
                    let event = row.get(1)?;
                    let was_playing = state.phase.is_playing();
                    state.apply(&event);
                    let is_playing = state.phase.is_playing();
                    if was_playing && !is_playing {
                        hands.push(CompleteHand {
                            charges: state.charged,
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
                    }
                }
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
                    hands,
                });
            }
            Ok(games)
        })?;
        Ok(warp::reply::json(&games))
    }

    warp::path!("scores")
        .and(db)
        .and(warp::query())
        .and_then(handle)
}

fn read_game_ids(mut rows: Rows<'_>) -> Result<Vec<GameId>, rusqlite::Error> {
    let mut game_ids = Vec::new();
    while let Some(row) = rows.next()? {
        game_ids.push(row.get(0)?);
    }
    Ok(game_ids)
}

fn winner_of(state: &GameState, card: Card) -> UserId {
    for i in 0..4 {
        if state.won[i].contains(card) {
            return state.players[i];
        }
    }
    unreachable!()
}
