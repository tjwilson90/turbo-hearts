use crate::{Algorithm, VoidState};
use rand::Rng;
use turbo_hearts_api::{BotState, Card, Cards, GameEvent, GameState, Rank, Suit};

macro_rules! check {
    ($hand:ident, $cards:expr, $len:literal) => {
        let cards = $cards.into();
        if $hand.contains_any(cards) && ($hand & cards.max().suit().cards()).len() <= $len {
            $hand -= ($hand & cards).max();
            continue;
        }
    };
}

macro_rules! play {
    ($cards:expr) => {
        let cards = $cards;
        if !cards.is_empty() {
            return cards;
        }
    };
}

macro_rules! dont_play {
    ($hand:ident, $cards:expr) => {
        if !($hand - $cards).is_empty() {
            $hand -= $cards;
            if $hand.len() == 1 {
                return $hand;
            }
        }
    };
}

pub struct HeuristicBot {
    void: VoidState,
}

impl HeuristicBot {
    pub fn new() -> Self {
        Self {
            void: VoidState::new(),
        }
    }

    pub fn from(void: VoidState) -> Self {
        Self { void }
    }

    fn lead(&self, mut ours: Cards, theirs: Cards, game_state: &GameState) -> Cards {
        let spades = ours & Cards::SPADES;
        if theirs.contains(Card::QueenSpades) && !spades.is_empty() {
            let low_spades = spades.below(Card::QueenSpades);
            let high_spades = spades.above(Card::QueenSpades);
            let other_spades = theirs & Cards::SPADES;

            // If it's safe to drill spades, do so.
            if high_spades.is_empty()
                || low_spades.len() >= 4
                || (low_spades.len() >= 3 && other_spades.len() <= 8)
                || (low_spades.len() >= 2 && other_spades.len() <= 4)
            {
                play!(low_spades);
            }

            // We don't believe in fake charges (and we have few enough low
            // spades that the test above didn't trigger).
            if game_state.charges.is_charged(Card::QueenSpades)
                && !game_state.led_suits.contains(Suit::Spades)
            {
                play!(high_spades);
            } else {
                dont_play!(ours, high_spades | Card::NineSpades);
            }
        }

        // If someone else would be forced to take the queen should we lead it, do so.
        if ours.contains(Card::QueenSpades)
            && theirs.below(Card::QueenSpades).is_empty()
            && !theirs.above(Card::QueenSpades).is_empty()
        {
            return Card::QueenSpades.into();
        }

        // Otherwise don't lead the queen
        dont_play!(ours, Card::QueenSpades);

        let clubs = ours & Cards::CLUBS;
        if theirs.contains(Card::TenClubs) && !clubs.is_empty() {
            let low_clubs = clubs.below(Card::TenClubs);
            let high_clubs = clubs.above(Card::TenClubs);
            let other_clubs = theirs & Cards::CLUBS;

            // If it's safe to drill clubs, do so.
            if high_clubs.is_empty()
                || low_clubs.len() >= 4
                || (low_clubs.len() >= 3 && other_clubs.len() <= 9)
                || (low_clubs.len() >= 2 && other_clubs.len() <= 5)
            {
                play!(low_clubs);
            } else {
                dont_play!(ours, high_clubs | Card::NineClubs);
            }
        }

        // If we can lead the jack and win it, do so.
        if ours.contains(Card::JackDiamonds) && theirs.above(Card::JackDiamonds).is_empty() {
            return Card::JackDiamonds.into();
        }

        // If someone else would be forced to take the ten should we lead it, do so.
        if ours.contains(Card::TenClubs)
            && theirs.below(Card::TenClubs).is_empty()
            && !theirs.above(Card::TenClubs).is_empty()
        {
            return Card::TenClubs.into();
        }

        // Otherwise don't lead the ten.
        dont_play!(ours, Card::TenClubs);

        // Don't lead suits everyone else is void in.
        for suit in &Suit::VALUES {
            if !theirs.contains_any(suit.cards()) {
                dont_play!(ours, suit.cards());
            }
        }

        ours
    }

    fn follow(
        &self,
        mut ours: Cards,
        theirs: Cards,
        bot_state: &BotState,
        game_state: &GameState,
    ) -> Cards {
        let trick: Cards = game_state.current_trick.cards();
        let suit = game_state.current_trick.suit();
        let winning = game_state.current_trick.winning_seat(bot_state.seat) == bot_state.seat;
        let winning_card = (trick & suit.cards()).max();

        // If the ten or queen is on the trick try to duck.
        if !winning && (trick.contains(Card::QueenSpades) || trick.contains(Card::TenClubs)) {
            dont_play!(ours, ours.above(winning_card));
        }

        // If we're going to get to play twice, don't play our highest card immediately.
        if trick.len() < 4 && trick.contains(suit.with_rank(Rank::Nine)) {
            dont_play!(ours, ours.max());
        }

        if suit == Suit::Spades {
            // If we can play the queen and not win the trick, do so.
            if ours.contains(Card::QueenSpades)
                && !winning
                && !trick.above(Card::QueenSpades).is_empty()
            {
                return Card::QueenSpades.into();
            }

            // Otherwise don't play the queen
            dont_play!(ours, Card::QueenSpades);

            // If we can play a high spade and not take the queen, do so.
            if !trick.contains(Card::QueenSpades)
                && self.is_last_play_in_suit(trick, bot_state, game_state)
            {
                play!(ours.above(Card::QueenSpades));
            }

            // If we can play the king of spades under the ace while the queen is out, do so.
            if trick.contains(Card::AceSpades)
                && ours.contains(Card::KingSpades)
                && theirs.contains(Card::QueenSpades)
            {
                return Card::KingSpades.into();
            }

            // If the queen's charged and it doesn't look fake, play a high spade.
            if game_state.charges.is_charged(Card::QueenSpades)
                && !game_state.led_suits.contains(Suit::Spades)
                && theirs.contains(Card::QueenSpades)
                && ours.below(Card::QueenSpades).len() < 4
            {
                play!(ours.above(Card::QueenSpades));
            }

            dont_play!(ours, ours.above(Card::QueenSpades));
        }

        // If the jack's on the trick and we might win it, attempt to do so.
        if trick.contains(Card::JackDiamonds) && !ours.above(winning_card).is_empty() {
            return ours.above(winning_card).max().into();
        }

        if ours.contains(Card::JackDiamonds) {
            let high_diamond = Ord::max(winning_card, Card::JackDiamonds);

            // If we can play the jack and win the trick do so.
            if (winning || winning_card < Card::JackDiamonds)
                && (theirs.above(high_diamond).is_empty()
                    || self.is_last_play_in_suit(trick, bot_state, game_state))
            {
                return Card::JackDiamonds.into();
            }

            // Otherwise don't play the jack.
            dont_play!(ours, Card::JackDiamonds);
        }

        if suit == Suit::Clubs {
            // If we can play the ten and not win the trick, do so.
            if ours.contains(Card::TenClubs) && !winning && !trick.above(Card::TenClubs).is_empty()
            {
                return Card::TenClubs.into();
            }

            // Otherwise don't play the ten.
            dont_play!(ours, Card::TenClubs);

            // If we can play a high club and not take the ten, do so.
            if !trick.contains(Card::TenClubs)
                && self.is_last_play_in_suit(trick, bot_state, game_state)
            {
                play!(ours.above(Card::TenClubs));
            }

            // If we can play a high club under another while the ten is out, do so.
            if theirs.contains(Card::TenClubs) {
                play!(ours.below(winning_card).above(Card::TenClubs));
            }

            // If the ten's charged and it doesn't look fake, play a high club.
            if game_state.charges.is_charged(Card::TenClubs)
                && trick.contains(Card::TwoClubs)
                && theirs.contains(Card::TenClubs)
                && ours.below(Card::TenClubs).len() < 3
            {
                play!(ours.above(Card::TenClubs));
            }

            // Otherwise don't play a high club that might win the trick.
            dont_play!(ours, ours.above(Card::TenClubs));
        }

        ours
    }

    fn slough(
        &self,
        mut ours: Cards,
        theirs: Cards,
        bot_state: &BotState,
        game_state: &GameState,
    ) -> Cards {
        let trick: Cards = game_state.current_trick.cards();
        let suit = game_state.current_trick.suit();
        let winning = game_state.current_trick.winning_seat(bot_state.seat) == bot_state.seat;
        let winning_card = (trick & suit.cards()).max();

        // If we have the queen
        if ours.contains(Card::QueenSpades) {
            if !winning {
                // Slough it if we're not winning the trick.
                return Card::QueenSpades.into();
            }

            // Otherwise don't slough any spades.
            dont_play!(ours, Cards::SPADES);
        }

        // If we can slough the jack on ourselves, do so.
        if ours.contains(Card::JackDiamonds)
            && winning
            && (theirs.above(winning_card).is_empty()
                || self.is_last_play_in_suit(trick, bot_state, game_state))
        {
            return Card::JackDiamonds.into();
        }

        // Otherwise don't slough the jack.
        dont_play!(ours, Card::JackDiamonds);

        // Slough high spades if the queen is out and we don't have sufficient protection.
        if theirs.contains(Card::QueenSpades)
            && theirs.below(Card::QueenSpades).len() >= ours.below(Card::QueenSpades).len()
        {
            play!(ours.above(Card::QueenSpades));
        }

        // If we have the ten
        if ours.contains(Card::TenClubs) {
            if !winning {
                // Slough it if we're not winning the trick.
                return Card::TenClubs.into();
            }

            // Otherwise don't slough any clubs.
            dont_play!(ours, Cards::CLUBS);
        }

        // Slough high clubs if the ten is out and we don't have sufficient protection.
        if theirs.contains(Card::TenClubs)
            && theirs.below(Card::TenClubs).len() >= ours.below(Card::TenClubs).len()
        {
            play!(ours.above(Card::TenClubs));
        }

        // Don't throw away high diamonds if the jack looks profitable.
        if theirs.contains(Card::JackDiamonds)
            && (game_state.charges.is_charged(Card::JackDiamonds)
                || !theirs.contains(Card::QueenSpades)
                || ours.below(Card::JackDiamonds).len() >= 3)
        {
            dont_play!(ours, Cards::DIAMONDS.above(Card::JackDiamonds));
        }

        let worst_suit = Suit::VALUES
            .iter()
            .cloned()
            .max_by_key(|&suit| danger(suit, bot_state, game_state))
            .unwrap();
        let worst_cards = ours & worst_suit.cards();
        if worst_cards.is_empty() {
            ours
        } else if worst_cards.len() == 1 {
            worst_cards
        } else if worst_cards.len() <= 3 && theirs.contains(Card::QueenSpades) {
            worst_cards.max().into()
        } else {
            (worst_cards - worst_cards.max()).max().into()
        }
    }

    fn is_last_play_in_suit(
        &self,
        trick: Cards,
        bot_state: &BotState,
        game_state: &GameState,
    ) -> bool {
        let suit = game_state.current_trick.suit();
        if (game_state.played | bot_state.post_pass_hand).contains_all(suit.cards()) {
            return true;
        }
        let seat = bot_state.seat;
        match trick.len() {
            7 => true,
            6 => self.void.is_void(seat.left(), suit),
            5 => self.void.is_void(seat.left(), suit) && self.void.is_void(seat.across(), suit),
            3 => !trick.contains(suit.with_rank(Rank::Nine)),
            2 => {
                !trick.contains(suit.with_rank(Rank::Nine)) && self.void.is_void(seat.left(), suit)
            }
            1 => {
                !trick.contains(suit.with_rank(Rank::Nine))
                    && self.void.is_void(seat.left(), suit)
                    && self.void.is_void(seat.across(), suit)
            }
            _ => false,
        }
    }
}

impl Algorithm for HeuristicBot {
    fn pass(&mut self, bot_state: &BotState, _: &GameState) -> Cards {
        let mut hand = bot_state.pre_pass_hand;
        if hand.contains_any(Cards::HEARTS) {
            if (hand & Cards::HEARTS).len() == 1 {
                hand -= Cards::HEARTS;
            } else {
                hand -= (hand & Cards::HEARTS).into_iter().nth(1).unwrap();
            }
        }
        while hand.len() > 10 {
            check!(hand, Card::QueenSpades, 2);
            check!(hand, Card::AceSpades, 2);
            check!(hand, Card::KingSpades, 2);
            check!(hand, Card::TenClubs, 2);
            check!(hand, Card::AceClubs, 2);
            check!(hand, Card::KingClubs, 2);
            check!(hand, Card::QueenClubs, 2);
            check!(hand, Card::JackClubs, 2);
            check!(hand, Cards::HEARTS, 1);
            check!(hand, Cards::CLUBS, 1);
            check!(hand, Cards::DIAMONDS, 1);
            check!(hand, Card::QueenSpades, 4);
            check!(hand, Card::AceSpades, 4);
            check!(hand, Card::KingSpades, 4);
            check!(hand, Card::TenClubs, 4);
            check!(hand, Card::AceClubs, 4);
            check!(hand, Card::KingClubs, 4);
            check!(hand, Card::QueenClubs, 4);
            check!(hand, Card::JackClubs, 4);
            check!(hand, Cards::HEARTS, 2);
            check!(hand, Cards::CLUBS, 2);
            check!(hand, Cards::DIAMONDS, 2);
            check!(hand, Cards::SPADES, 1);
            check!(hand, Cards::HEARTS, 13);
            check!(hand, Cards::CLUBS, 13);
            check!(hand, Cards::DIAMONDS, 13);
            check!(hand, Cards::SPADES, 13);
        }
        bot_state.pre_pass_hand - hand
    }

    fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        let hand = bot_state.post_pass_hand;
        let chargeable = hand - game_state.charges.all_charges();
        let mut charge = Cards::NONE;
        if chargeable.contains(Card::QueenSpades) {
            let spades = hand & Cards::SPADES;
            if spades.len() >= 6 || (spades.len() >= 5 && hand.contains(Card::NineSpades)) {
                if (hand - Cards::SPADES - Card::TwoClubs)
                    .into_iter()
                    .any(|c| c.rank() < Rank::Five)
                {
                    charge |= Card::QueenSpades;
                }
            }
        }
        if chargeable.contains(Card::TenClubs) {
            let clubs = hand & Cards::CLUBS;
            if clubs.len() >= 6 || (clubs.len() >= 5 && hand.contains(Card::NineClubs)) {
                if (hand - Cards::CLUBS)
                    .into_iter()
                    .any(|c| c.rank() < Rank::Five)
                {
                    charge |= Card::TenClubs;
                }
            }
        }
        if chargeable.contains(Card::AceHearts) {
            let hearts = hand & Cards::HEARTS;
            if hearts.below(Card::EightHearts).len() >= 3 {
                charge |= Card::AceHearts;
            }
        }
        if chargeable.contains(Card::JackDiamonds) {
            let diamonds = hand & Cards::DIAMONDS;
            let high_diamonds = diamonds.above(Card::JackDiamonds).len();
            let high_cards = (hand - Cards::DIAMONDS)
                .into_iter()
                .filter(|c| *c == Card::QueenSpades || c.rank() > Rank::Queen)
                .count();
            if high_diamonds == 3
                || (diamonds.len() >= 5 && high_diamonds == 2 && high_cards > 1)
                || (diamonds.len() >= 6 && high_diamonds == 1 && high_cards > 2)
            {
                charge |= Card::JackDiamonds;
            }
        }
        charge
    }

    fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        let ours = game_state.legal_plays(bot_state.post_pass_hand);

        // If we only have one legal play, play it
        if ours.len() == 1 {
            return ours.max();
        }
        let theirs = Cards::ALL - bot_state.post_pass_hand - game_state.played;
        let good_plays = if game_state.current_trick.is_empty() {
            self.lead(ours, theirs, game_state)
        } else if game_state.current_trick.suit().cards().contains_any(ours) {
            self.follow(ours, theirs, bot_state, game_state)
        } else {
            self.slough(ours, theirs, bot_state, game_state)
        };
        let index = rand::thread_rng().gen_range(0..good_plays.len());
        good_plays.into_iter().nth(index).unwrap()
    }

    fn on_event(&mut self, _: &BotState, state: &GameState, event: &GameEvent) {
        self.void.on_event(state, event);
    }
}

fn danger(suit: Suit, bot_state: &BotState, game_state: &GameState) -> i8 {
    let all = suit.cards() & (Cards::ALL - game_state.played);
    let mut ours = all & bot_state.post_pass_hand;
    let mut theirs = all - ours;
    if ours.is_empty() || theirs.is_empty() {
        // We can't win any tricks in this suit.
        return i8::MIN;
    }
    if suit != Suit::Hearts && theirs.len() >= 7 {
        // Assume first trick will contain high cards and not be dangerous
        ours -= ours.max();
        theirs -= theirs.max();
        theirs -= theirs.max();
        theirs -= theirs.max();
    }
    if ours.is_empty() || ours.max() < theirs.min() {
        // We can duck everything.
        return i8::MIN;
    }
    let mut danger = 0;

    // High cards that we're never forced to play because of our long suit are not dangerous.
    let skip = if theirs.contains(suit.with_rank(Rank::Nine)) {
        ours.len().saturating_sub(theirs.len() + 1)
    } else {
        ours.len().saturating_sub(theirs.len())
    };
    for card in ours.into_iter().skip(skip) {
        // Cards are more dangerous if they can be ducked, and less dangerous if they can duck.
        danger += theirs.below(card).len() as i8 - theirs.above(card).len() as i8;
    }
    if suit == Suit::Hearts {
        // Hearts are points; dangerous if they're winners, good to have as losers.
        danger *= 2;
    }
    danger
}
