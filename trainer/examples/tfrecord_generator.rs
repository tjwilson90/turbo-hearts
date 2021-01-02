use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rand::Rng;
use std::{error::Error, fs, fs::File};
use tfrecord::{Example, ExampleWriter, Feature, RecordWriterInit};
use turbo_hearts_api::{Cards, GameState};
use turbo_hearts_bot::Encoder;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let mut writer = Writer::new()?;
    for entry in fs::read_dir("data")? {
        let entry = entry?;
        match entry.file_name().to_str() {
            Some(name) if name.starts_with("complete") => {}
            _ => continue,
        };
        let mut decoder = GzDecoder::new(File::open(entry.path())?);
        loop {
            let game_state: GameState = match bincode::deserialize_from(&mut decoder) {
                Ok(game_state) => game_state,
                _ => {
                    break;
                }
            };
            let north = Cards {
                bits: bincode::deserialize_from(&mut decoder)?,
            };
            let east = Cards {
                bits: bincode::deserialize_from(&mut decoder)?,
            };
            let south = Cards {
                bits: bincode::deserialize_from(&mut decoder)?,
            };
            let west = Cards {
                bits: bincode::deserialize_from(&mut decoder)?,
            };
            let won = bincode::deserialize_from(&mut decoder)?;
            let plays: Vec<i16> = bincode::deserialize_from(&mut decoder)?;

            let seat = game_state.next_actor.unwrap();
            let hands = [north, east, south, west];

            let mut record = Example::with_capacity(13);
            record.insert(
                "cards".to_string(),
                Feature::FloatList(
                    Encoder::new(Encoder::CARDS_LEN)
                        .cards(seat, game_state.played, hands)
                        .into_inner(),
                ),
            );
            record.insert(
                "won_queen".to_string(),
                Feature::FloatList(
                    Encoder::new(Encoder::QUEEN_LEN)
                        .queen(seat, game_state.won)
                        .into_inner(),
                ),
            );
            record.insert(
                "won_jack".to_string(),
                Feature::FloatList(
                    Encoder::new(Encoder::JACK_LEN)
                        .jack(seat, game_state.won)
                        .into_inner(),
                ),
            );
            record.insert(
                "won_ten".to_string(),
                Feature::FloatList(
                    Encoder::new(Encoder::TEN_LEN)
                        .ten(seat, game_state.won)
                        .into_inner(),
                ),
            );
            record.insert(
                "won_hearts".to_string(),
                Feature::FloatList(
                    Encoder::new(Encoder::HEARTS_LEN)
                        .hearts(seat, game_state.won)
                        .into_inner(),
                ),
            );
            record.insert(
                "charged".to_string(),
                Feature::FloatList(
                    Encoder::new(Encoder::CHARGED_LEN)
                        .charged(game_state.charges)
                        .into_inner(),
                ),
            );
            record.insert(
                "led".to_string(),
                Feature::FloatList(
                    Encoder::new(Encoder::LED_LEN)
                        .led(game_state.led_suits)
                        .into_inner(),
                ),
            );
            if !game_state.current_trick.is_empty() {
                record.insert(
                    "trick".to_string(),
                    Feature::FloatList(
                        Encoder::new(Encoder::TRICK_LEN)
                            .trick(seat, game_state.played, game_state.current_trick)
                            .into_inner(),
                    ),
                );
            }
            let encoded_plays = Encoder::new(Encoder::PLAYS_LEN)
                .plays(hands[seat.idx()], &game_state, &plays)
                .into_inner();
            if !encoded_plays.is_empty() {
                let mut record = record.clone();
                record.insert("plays".to_string(), Feature::FloatList(encoded_plays));
                writer.write(game_state.current_trick.is_empty(), true, record)?;
            }
            record.insert(
                "win_queen".to_string(),
                Feature::FloatList(
                    Encoder::new(Encoder::QUEEN_LEN)
                        .queen(seat, won)
                        .into_inner(),
                ),
            );
            record.insert(
                "win_jack".to_string(),
                Feature::FloatList(Encoder::new(Encoder::JACK_LEN).jack(seat, won).into_inner()),
            );
            record.insert(
                "win_ten".to_string(),
                Feature::FloatList(Encoder::new(Encoder::TEN_LEN).ten(seat, won).into_inner()),
            );
            record.insert(
                "win_hearts".to_string(),
                Feature::FloatList(
                    Encoder::new(Encoder::HEARTS_LEN)
                        .hearts(seat, won)
                        .into_inner(),
                ),
            );
            writer.write(game_state.current_trick.is_empty(), false, record)?;
        }
    }
    Ok(())
}

struct Writer {
    train_lead_policy: ExampleWriter<GzEncoder<File>>,
    train_lead_value: ExampleWriter<GzEncoder<File>>,
    train_follow_policy: ExampleWriter<GzEncoder<File>>,
    train_follow_value: ExampleWriter<GzEncoder<File>>,
    validate_lead_policy: ExampleWriter<GzEncoder<File>>,
    validate_lead_value: ExampleWriter<GzEncoder<File>>,
    validate_follow_policy: ExampleWriter<GzEncoder<File>>,
    validate_follow_value: ExampleWriter<GzEncoder<File>>,
}

fn writer(path: &str) -> Result<ExampleWriter<GzEncoder<File>>, Box<dyn Error>> {
    let writer = File::create(path)?;
    let encoder = GzEncoder::new(writer, Compression::default());
    Ok(RecordWriterInit::from_writer(encoder)?)
}

impl Writer {
    fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            train_lead_policy: writer("data/train-lead-policy.tfrec.gz")?,
            train_lead_value: writer("data/train-lead-value.tfrec.gz")?,
            train_follow_policy: writer("data/train-follow-policy.tfrec.gz")?,
            train_follow_value: writer("data/train-follow-value.tfrec.gz")?,
            validate_lead_policy: writer("data/validate-lead-policy.tfrec.gz")?,
            validate_lead_value: writer("data/validate-lead-value.tfrec.gz")?,
            validate_follow_policy: writer("data/validate-follow-policy.tfrec.gz")?,
            validate_follow_value: writer("data/validate-follow-value.tfrec.gz")?,
        })
    }

    fn write(&mut self, lead: bool, policy: bool, example: Example) -> Result<(), Box<dyn Error>> {
        Ok(match (rand::thread_rng().gen_bool(0.9), lead, policy) {
            (true, true, true) => self.train_lead_policy.send(example)?,
            (true, true, false) => self.train_lead_value.send(example)?,
            (true, false, true) => self.train_follow_policy.send(example)?,
            (true, false, false) => self.train_follow_value.send(example)?,
            (false, true, true) => self.validate_lead_policy.send(example)?,
            (false, true, false) => self.validate_lead_value.send(example)?,
            (false, false, true) => self.validate_follow_policy.send(example)?,
            (false, false, false) => self.validate_follow_value.send(example)?,
        })
    }
}
