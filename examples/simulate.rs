use turbo_hearts_api::{
    BotStrategy, Cards, CardsError, ChargingRules, Database, GameEvent, GameId, Games, Player,
    PlayerWithOptions, Seed, UserId,
};

fn player() -> PlayerWithOptions {
    PlayerWithOptions {
        player: Player::Bot {
            user_id: UserId::new(),
            strategy: BotStrategy::Simulate,
        },
        rules: ChargingRules::Classic,
        seat: None,
    }
}

#[tokio::main]
async fn main() -> Result<(), CardsError> {
    env_logger::init();
    let temp_dir = tempfile::tempdir().unwrap();
    let mut path = temp_dir.path().to_owned();
    path.push("test.db");
    let db = &*Box::leak(Box::new(Database::new(path).unwrap()));
    let games = &*Box::leak(Box::new(Games::new(db, false)));
    let game_id = GameId::new();
    games.start_game(
        game_id,
        [player(), player(), player(), player()],
        Seed::random(),
    )?;
    let mut rx = games.subscribe(game_id, UserId::new(), None).await?;
    let mut hands = [Cards::NONE; 4];
    while let Some((event, _)) = rx.recv().await {
        use GameEvent::*;
        match event {
            Deal {
                north,
                east,
                south,
                west,
                ..
            } => {
                hands = [north, east, south, west];
                println!("North {}", north);
                println!("East  {}", east);
                println!("South {}", south);
                println!("West  {}", west);
            }
            SendPass { from, cards } => {
                hands[from.idx()] -= cards;
                println!("{} passes {}", from, cards);
            }
            RecvPass { to, cards } => {
                hands[to.idx()] |= cards;
                println!("{} receives {}", to, cards);
            }
            StartCharging => {
                println!("North {}", hands[0]);
                println!("East  {}", hands[1]);
                println!("South {}", hands[2]);
                println!("West  {}", hands[3]);
            }
            Charge { seat, cards } => {
                if cards.is_empty() {
                    println!("{} charges nothing", seat);
                } else {
                    println!("{} charges {}", seat, cards);
                }
            }
            Play { seat, card } => {
                println!("{} plays {} from {}", seat, card, hands[seat.idx()]);
                hands[seat.idx()] -= card;
            }
            EndTrick { winner } => {
                println!("{} wins the trick", winner);
            }
            Claim { seat, .. } => {
                println!("{} claims the remaining tricks", seat);
            }
            AcceptClaim { claimer, acceptor } => {
                println!("{} accepts {}'s claim", acceptor, claimer);
            }
            RejectClaim { claimer, rejector } => {
                println!("{} rejects {}'s claim", rejector, claimer);
            }
            HandComplete {
                north_score,
                east_score,
                south_score,
                west_score,
            } => {
                let total = north_score + east_score + south_score + west_score;
                println!(
                    "North score = {}, money = {}",
                    north_score,
                    total - 4 * north_score
                );
                println!(
                    "East  score = {}, money = {}",
                    east_score,
                    total - 4 * east_score
                );
                println!(
                    "South score = {}, money = {}",
                    south_score,
                    total - 4 * south_score
                );
                println!(
                    "West  score = {}, money = {}",
                    west_score,
                    total - 4 * west_score
                );
            }
            GameComplete { .. } => {
                break;
            }
            _ => {}
        }
    }
    Ok(())
}
