//! A public-information continuation for opponents who try to stop a moon.
use turbo_hearts_api::{BotState, Card, Cards, GameState, Seat, Suit};

/// Take a point to break an established moon threat. This is a rollout model,
/// not an unconditional override of the actual search decision. It sees only
/// the acting player's hand, captured points, and the public trick.
pub(crate) fn defensive_play(bot: &BotState, state: &GameState) -> Option<Card> {
    let threat = Seat::VALUES.iter().copied().find(|&s| {
        s != bot.seat && state.won.can_run(s) && (state.won.hearts(s) >= 3 || state.won.queen(s))
    })?;
    let legal = state.legal_plays(bot.post_pass_hand);
    let unseen = Cards::ALL - bot.post_pass_hand - state.played;
    if state.current_trick.is_empty() {
        let hearts = legal & Cards::HEARTS;
        if !hearts.is_empty() && (unseen & Cards::HEARTS).above(hearts.max()).is_empty() {
            return Some(hearts.max());
        }
        return None;
    }
    let trick = state.current_trick;
    let winner = trick.winning_seat(bot.seat);
    let suit = trick.suit();
    let winning_card = (trick.cards() & suit.cards()).max();
    if winner != bot.seat
        && trick
            .cards()
            .contains_any(Cards::HEARTS | Card::QueenSpades)
    {
        let winners = (legal & suit.cards()).above(winning_card);
        if !winners.is_empty() {
            return Some(winners.max());
        }
    }
    // The queen itself can supply the point that stops a run.
    if suit == Suit::Spades && legal.contains(Card::QueenSpades) && winning_card < Card::QueenSpades
    {
        let mut child = state.clone();
        child.apply(&turbo_hearts_api::GameEvent::Play {
            seat: bot.seat,
            card: Card::QueenSpades,
        });
        if child.current_trick.is_empty()
            || (unseen & Cards::SPADES).above(Card::QueenSpades).is_empty()
        {
            return Some(Card::QueenSpades);
        }
    }
    // When another defender has already won the completed trick, supply a
    // heart instead of discarding a harmless card. Account for extended nines.
    if winner != threat && winner != bot.seat {
        let hearts = legal & Cards::HEARTS;
        if !hearts.is_empty() {
            let card = hearts.min();
            let mut child = state.clone();
            child.apply(&turbo_hearts_api::GameEvent::Play {
                seat: bot.seat,
                card,
            });
            if child.current_trick.is_empty() && !child.won.can_run(threat) {
                return Some(card);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_hearts_api::{GamePhase, Trick, WonState};

    #[test]
    fn takes_a_heart_to_break_a_live_moon_and_stops_after_it_is_broken() {
        let bot = BotState::new(Seat::West, "AH 2S".parse().unwrap());
        let mut state = GameState::new();
        state.phase = GamePhase::PlayLeft;
        state.next_actor = Some(bot.seat);
        state.current_trick = Trick::new()
            .push(Card::KingHearts)
            .push(Card::FiveHearts)
            .push(Card::SixHearts);
        state.played = "234H K56H 2C".parse().unwrap();
        state.won = WonState::new().win(Seat::North, "234H".parse().unwrap());
        assert_eq!(defensive_play(&bot, &state), Some(Card::AceHearts));
        state.won = state.won.win(Seat::East, Card::SevenHearts.into());
        assert_eq!(defensive_play(&bot, &state), None);
    }

    #[test]
    fn supplies_a_heart_to_another_defender_on_the_last_play() {
        let bot = BotState::new(Seat::West, "2H 3D".parse().unwrap());
        let mut state = GameState::new();
        state.phase = GamePhase::PlayLeft;
        state.next_actor = Some(bot.seat);
        state.current_trick = Trick::new()
            .push(Card::TwoSpades)
            .push(Card::AceSpades)
            .push(Card::ThreeSpades);
        state.played = "345H 2A3S 2C".parse().unwrap();
        state.won = WonState::new().win(Seat::North, "345H".parse().unwrap());
        assert_eq!(defensive_play(&bot, &state), Some(Card::TwoHearts));
    }
}
