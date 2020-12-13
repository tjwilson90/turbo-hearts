use std::collections::HashMap;
use turbo_hearts_api::{Cards, Rank, Seat, Suit, Suits, WonState};

pub struct TranspositionTable {
    suits: SuitTranspositions,
    table: HashMap<TranspositionKey, WonState>,
}

impl TranspositionTable {
    pub fn new(hands: [Cards; 4]) -> Self {
        Self {
            suits: SuitTranspositions::new(hands),
            table: HashMap::new(),
        }
    }

    pub fn lookup(
        &self,
        leader: Seat,
        leads: Suits,
        won: WonState,
        played: Cards,
    ) -> Result<WonState, TranspositionKey> {
        let key = TranspositionKey {
            leader,
            leads,
            won,
            transpositions: [
                self.suits.transposition(Suit::Clubs, played),
                self.suits.transposition(Suit::Diamonds, played),
                self.suits.transposition(Suit::Hearts, played),
                self.suits.transposition(Suit::Spades, played),
            ],
        };
        match self.table.get(&key) {
            Some(won) => Ok(*won),
            None => Err(key),
        }
    }

    pub fn cache(&mut self, key: TranspositionKey, won: WonState) {
        self.table.insert(key, won);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TranspositionKey {
    leader: Seat,
    leads: Suits,
    won: WonState,
    transpositions: [u16; 4],
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
