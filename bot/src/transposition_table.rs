use std::collections::HashMap;
use turbo_hearts_api::{Cards, GameState, Rank, Seat, Suit, Suits, WonState};

pub struct TranspositionTable<T> {
    suits: SuitTranspositions,
    table: HashMap<TranspositionKey, T>,
}

impl<T: Copy> TranspositionTable<T> {
    pub fn new(hands: [Cards; 4]) -> Self {
        Self {
            suits: SuitTranspositions::new(hands),
            table: HashMap::new(),
        }
    }

    pub fn lookup(&mut self, state: &GameState) -> Result<T, TranspositionKey> {
        let key = TranspositionKey {
            leader: state.next_actor.unwrap(),
            leads: state.led_suits,
            won: state.won,
            trick: trick_transposition(state),
            transpositions: [
                self.suits.transposition(Suit::Clubs, state.played),
                self.suits.transposition(Suit::Diamonds, state.played),
                self.suits.transposition(Suit::Hearts, state.played),
                self.suits.transposition(Suit::Spades, state.played),
            ],
        };
        match self.table.get(&key) {
            Some(won) => Ok(*won),
            None => Err(key),
        }
    }

    pub fn cache(&mut self, key: TranspositionKey, won: T) {
        self.table.insert(key, won);
        if self.table.len() == self.table.capacity() && self.table.capacity() > 1 << 29 {
            self.shrink();
        }
    }

    #[cold]
    #[inline(never)]
    fn shrink(&mut self) {
        let old_len = self.table.len();
        self.table.retain(|k, _| k.trick == 0);
        eprintln!("Shrinking from {} to {} keys", old_len, self.table.len());
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TranspositionKey {
    leader: Seat,             // 2  8
    leads: Suits,             // 3  8
    won: WonState,            // 19 32
    trick: u16,               // 9  16
    transpositions: [u16; 4], // 52 64
}

fn trick_transposition(state: &GameState) -> u16 {
    let trick = state.current_trick;
    if trick.is_empty() {
        return 0;
    }
    let cards = trick.cards();
    let len = trick.len();
    let nined = cards.contains(trick.suit().with_rank(Rank::Nine));
    let rank = ((Cards::ALL - state.played) & trick.suit().cards())
        .above((trick.suit().cards() & cards).max())
        .len();
    let suit = trick.suit();
    let seat = trick.winning_seat(state.next_actor.unwrap());
    (len as u16) << 9
        | (nined as u16) << 8
        | (rank as u16) << 4
        | (suit as u16) << 2
        | (seat as u16)
}

struct SuitTranspositions {
    transpositions: [Box<[u16]>; 4],
}

impl SuitTranspositions {
    fn new(hands: [Cards; 4]) -> Self {
        Self {
            transpositions: [
                suit_transposition(Suit::Clubs, hands),
                suit_transposition(Suit::Diamonds, hands),
                suit_transposition(Suit::Hearts, hands),
                suit_transposition(Suit::Spades, hands),
            ],
        }
    }

    fn transposition(&self, suit: Suit, played: Cards) -> u16 {
        let index = (suit.cards() & played).bits >> (16 * suit.idx());
        self.transpositions[suit.idx()][index as usize]
    }
}

fn suit_transposition(suit: Suit, hands: [Cards; 4]) -> Box<[u16]> {
    let mut transpositions: HashMap<SuitTransposition, u16> = HashMap::with_capacity(1 << 13);
    let mut table = Vec::with_capacity(1 << 13);
    for i in 0..1 << 13 {
        let played = Cards {
            bits: i << (16 * suit.idx()),
        };
        let partial_hands = [
            hands[0] - played,
            hands[1] - played,
            hands[2] - played,
            hands[3] - played,
        ];
        let transposition = SuitTransposition::new(suit, partial_hands);
        let class = *transpositions.entry(transposition).or_insert(i as u16);
        table.push(class);
    }
    table.into_boxed_slice()
}

#[derive(Eq, PartialEq, Hash, Debug)]
struct SuitTransposition(Vec<Entry>);

#[derive(Eq, PartialEq, Hash, Debug)]
enum Entry {
    Chargeable(Seat),
    Nine(Seat),
    Other(Seat),
}

impl SuitTransposition {
    fn new(suit: Suit, hands: [Cards; 4]) -> SuitTransposition {
        let mut entries = Vec::with_capacity(13);
        for card in suit.cards() {
            if let Some(seat) = hands
                .iter()
                .position(|hand| hand.contains(card))
                .map(|index| Seat::VALUES[index])
            {
                entries.push(if Cards::CHARGEABLE.contains(card) {
                    Entry::Chargeable(seat)
                } else if card.rank() == Rank::Nine {
                    Entry::Nine(seat)
                } else {
                    Entry::Other(seat)
                });
            }
        }
        Self(entries)
    }
}
