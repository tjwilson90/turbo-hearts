//! A public-information continuation for opponents who try to stop a moon.
use turbo_hearts_api::{BotState, Card, Cards, GameState, Seat, Suit};

/// A current master heart with enough cover to duck future heart tricks. Only
/// protect it on a discard, while another player's moon is still possible.
pub(crate) fn supported_stopper(bot: &BotState, state: &GameState) -> Option<Card> {
    if state.current_trick.is_empty()
        || state.current_trick.winning_seat(bot.seat) == bot.seat
        || !Seat::VALUES
            .iter()
            .any(|&s| s != bot.seat && state.won.can_run(s))
    {
        return None;
    }
    let hand = bot.post_pass_hand - state.played;
    let hearts = hand & Cards::HEARTS;
    let legal = state.legal_plays(hand);
    if hand.contains_any(state.current_trick.suit().cards())
        || hearts.len() < 3
        || (legal - hearts.max()).is_empty()
    {
        return None;
    }
    let unseen = Cards::HEARTS - hand - state.played;
    if unseen.above(hearts.max()).is_empty() {
        Some(hearts.max())
    } else {
        None
    }
}

/// Preventive own continuation. Preserve a supported master before points are
/// captured; after an opposing player captures a point, use control to break
/// the run. Does not assume any other defender will cooperate.
pub(crate) fn planned_play(bot: &BotState, state: &GameState) -> Option<Card> {
    if let Some(stopper) = supported_stopper(bot, state) {
        let choices = crate::HeuristicBot.choices(bot, state) - stopper;
        if !choices.is_empty() {
            return Some(choices.min());
        }
        let legal = state.legal_plays(bot.post_pass_hand) - stopper;
        return Some(legal.min());
    }
    // Reuse the tactical defender after the first point, without changing the
    // frozen experimental model's three-heart threshold.
    defensive_play_after(bot, state, 1)
}

/// Alternative preventive continuation, evaluated across sampled deals before
/// selecting it. A heart on an early completed trick removes other players'
/// option to duck that trick and still moon. An off-suit nine cannot extend the
/// current trick, and cannot later be forced into a runner's winning suit.
pub(crate) fn disruptive_play(bot: &BotState, state: &GameState) -> Option<Card> {
    if let Some(card) = defensive_play_after(bot, state, 1) {
        return Some(card);
    }
    let threats: Vec<_> = Seat::VALUES
        .iter()
        .copied()
        .filter(|&s| s != bot.seat && state.won.can_run(s))
        .collect();
    if threats.is_empty() || state.current_trick.is_empty() {
        return None;
    }
    let hand = bot.post_pass_hand - state.played;
    let legal = state.legal_plays(hand);
    let suit = state.current_trick.suit();
    if hand.contains_any(suit.cards()) || state.current_trick.winning_seat(bot.seat) == bot.seat {
        return planned_play(bot, state);
    }
    let stopper = supported_stopper(bot, state);
    let hearts = legal & Cards::HEARTS;
    if !hearts.is_empty() {
        let card = hearts.min();
        if Some(card) != stopper {
            let mut after = state.clone();
            after.apply(&turbo_hearts_api::GameEvent::Play {
                seat: bot.seat,
                card,
            });
            // Require an actually completed trick, including extensions by a
            // nine. Do not assume the current high card will hold with seats left.
            if after.current_trick.is_empty() && threats.iter().any(|&s| !after.won.can_run(s)) {
                return Some(card);
            }
        }
    }
    let unseen = Cards::ALL - bot.post_pass_hand - state.played;
    let nine = legal
        .into_iter()
        .filter(|c| {
            c.rank() == turbo_hearts_api::Rank::Nine
                && c.suit() != Suit::Hearts
                && !(unseen & c.suit().cards())
                    .above((hand & c.suit().cards()).max())
                    .is_empty()
        })
        .max_by_key(|c| (unseen & c.suit().cards()).above(*c).len());
    nine.or_else(|| planned_play(bot, state))
}

/// Take a point to break an established moon threat. This is a rollout model,
/// not an unconditional override of the actual search decision. It sees only
/// the acting player's hand, captured points, and the public trick.
pub(crate) fn defensive_play(bot: &BotState, state: &GameState) -> Option<Card> {
    defensive_play_after(bot, state, 3)
}

fn defensive_play_after(bot: &BotState, state: &GameState, hearts: u8) -> Option<Card> {
    let threat = Seat::VALUES.iter().copied().find(|&s| {
        s != bot.seat
            && state.won.can_run(s)
            && (state.won.hearts(s) >= hearts || state.won.queen(s))
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
    fn supplies_an_early_heart_but_not_before_an_extended_trick_has_finished() {
        let bot = BotState::new(Seat::West, "A62H 9D".parse().unwrap());
        let mut state = GameState::new();
        state.phase = GamePhase::PlayLeft;
        state.played = "234C AKQJTD".parse().unwrap();
        state.current_trick = Trick::new()
            .push(Card::ThreeClubs)
            .push(Card::AceClubs)
            .push(Card::FourClubs);
        state.played |= state.current_trick.cards();
        assert_eq!(disruptive_play(&bot, &state), Some(Card::TwoHearts));
        assert!(state.won.can_run(Seat::North));
        state.current_trick = Trick::new()
            .push(Card::NineClubs)
            .push(Card::AceClubs)
            .push(Card::FourClubs);
        state.played |= Card::NineClubs;
        // The heart-supply tactic cannot treat the first four cards of a nine
        // trick as final. Check its actual successor rather than a length rule.
        let mut after = state.clone();
        after.apply(&turbo_hearts_api::GameEvent::Play {
            seat: bot.seat,
            card: Card::TwoHearts,
        });
        assert!(!after.current_trick.is_empty());
        // Without a weak nine or supported stopper, no preventive action applies.
        let bot = BotState::new(Seat::West, "62H 9D".parse().unwrap());
        assert_eq!(disruptive_play(&bot, &state), None);
    }

    #[test]
    fn discards_an_off_suit_nine_but_keeps_one_supported_by_own_control() {
        let mut bot = BotState::new(Seat::West, "9D 2S".parse().unwrap());
        let mut state = GameState::new();
        state.phase = GamePhase::PlayLeft;
        state.current_trick = Trick::new().push(Card::KingClubs);
        state.played = "2KC".parse().unwrap();
        assert_eq!(disruptive_play(&bot, &state), Some(Card::NineDiamonds));
        bot.post_pass_hand |= Card::AceDiamonds;
        assert_eq!(disruptive_play(&bot, &state), None);
        state.won = state
            .won
            .win(Seat::North, Card::TwoHearts.into())
            .win(Seat::East, Card::ThreeHearts.into());
        bot.post_pass_hand -= Card::AceDiamonds;
        assert_eq!(disruptive_play(&bot, &state), None);
    }

    #[test]
    fn preserves_control_only_with_cover_and_a_live_opposing_moon() {
        let mut bot = BotState::new(Seat::West, "AJ64H 2D".parse().unwrap());
        let mut state = GameState::new();
        state.phase = GamePhase::PlayLeft;
        state.next_actor = Some(bot.seat);
        state.current_trick = Trick::new().push(Card::KingClubs);
        state.played = "2KC".parse().unwrap();
        assert_eq!(supported_stopper(&bot, &state), Some(Card::AceHearts));
        assert_ne!(planned_play(&bot, &state), Some(Card::AceHearts));
        bot.post_pass_hand = "A4H 2D".parse().unwrap();
        assert_eq!(supported_stopper(&bot, &state), None);
        bot.post_pass_hand = "KJ64H 2D".parse().unwrap();
        assert_eq!(supported_stopper(&bot, &state), None);
        state.played |= Card::AceHearts;
        assert_eq!(supported_stopper(&bot, &state), Some(Card::KingHearts));
        state.won = state
            .won
            .win(Seat::North, Card::AceHearts.into())
            .win(Seat::East, Card::TwoHearts.into());
        assert_eq!(supported_stopper(&bot, &state), None);
    }

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
