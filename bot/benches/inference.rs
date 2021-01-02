use criterion::Criterion;
use std::error::Error;
use tract_onnx::{
    prelude::{
        tract_ndarray::Array2, tvec, Datum, Framework, InferenceFact, InferenceModel,
        InferenceModelExt, TVec,
    },
    tract_hir::tract_core::downcast_rs::__std::time::Duration,
};
use turbo_hearts_api::{
    ChargeState, ChargingRules, ClaimState, DoneState, GamePhase, GameState, Seat, Suit, Suits,
    Trick, WonState,
};
use turbo_hearts_bot::Encoder;

pub fn inference(c: &mut Criterion) -> Result<(), Box<dyn Error>> {
    let mut g = c.benchmark_group("inference");
    for i in 0..8 {
        let batch_size = 1 << i;
        let mut model: InferenceModel = tract_onnx::onnx()
            .model_for_path("/Users/twilson/code/turbo-hearts/assets/lead-model.onnx")?;
        model.set_input_fact(
            0,
            InferenceFact::dt_shape(f32::datum_type(), tvec![batch_size, Encoder::CARDS_LEN]),
        )?;
        model.set_input_fact(
            1,
            InferenceFact::dt_shape(f32::datum_type(), tvec![batch_size, Encoder::QUEEN_LEN]),
        )?;
        model.set_input_fact(
            2,
            InferenceFact::dt_shape(f32::datum_type(), tvec![batch_size, Encoder::JACK_LEN]),
        )?;
        model.set_input_fact(
            3,
            InferenceFact::dt_shape(f32::datum_type(), tvec![batch_size, Encoder::TEN_LEN]),
        )?;
        model.set_input_fact(
            4,
            InferenceFact::dt_shape(f32::datum_type(), tvec![batch_size, Encoder::HEARTS_LEN]),
        )?;
        model.set_input_fact(
            5,
            InferenceFact::dt_shape(f32::datum_type(), tvec![batch_size, Encoder::CHARGED_LEN]),
        )?;
        model.set_input_fact(
            6,
            InferenceFact::dt_shape(f32::datum_type(), tvec![batch_size, Encoder::LED_LEN]),
        )?;
        let model = model.into_optimized()?.into_runnable()?;
        let state = GameState {
            rules: ChargingRules::Classic,
            phase: GamePhase::PlayLeft,
            done: DoneState::new(),
            charge_count: 0,
            charges: ChargeState::new(),
            next_actor: Some(Seat::East),
            played: "2C QC JC TC".parse().unwrap(),
            claims: ClaimState::new(),
            won: WonState::new(),
            led_suits: Suits::NONE | Suit::Clubs,
            current_trick: Trick::new(),
        };
        let hands = [
            "92S QH AKT93D AJ654C".parse().unwrap(),
            "AS 74H QJ842D KT873C".parse().unwrap(),
            "QJ7543S KJH 765D 92C".parse().unwrap(),
            "KT86S AT986532H QC".parse().unwrap(),
        ];
        let mut input = TVec::with_capacity(8);
        let mut encoder = Encoder::new(batch_size * Encoder::CARDS_LEN);
        for _ in 0..batch_size {
            encoder = encoder.cards(Seat::East, state.played, hands);
        }
        input.push(
            Array2::from_shape_vec((batch_size, Encoder::CARDS_LEN), encoder.into_inner())
                .unwrap()
                .into(),
        );
        let mut encoder = Encoder::new(batch_size * Encoder::QUEEN_LEN);
        for _ in 0..batch_size {
            encoder = encoder.queen(Seat::East, state.won);
        }
        input.push(
            Array2::from_shape_vec((batch_size, Encoder::QUEEN_LEN), encoder.into_inner())
                .unwrap()
                .into(),
        );
        let mut encoder = Encoder::new(batch_size * Encoder::JACK_LEN);
        for _ in 0..batch_size {
            encoder = encoder.jack(Seat::East, state.won);
        }
        input.push(
            Array2::from_shape_vec((batch_size, Encoder::JACK_LEN), encoder.into_inner())
                .unwrap()
                .into(),
        );
        let mut encoder = Encoder::new(batch_size * Encoder::TEN_LEN);
        for _ in 0..batch_size {
            encoder = encoder.ten(Seat::East, state.won);
        }
        input.push(
            Array2::from_shape_vec((batch_size, Encoder::TEN_LEN), encoder.into_inner())
                .unwrap()
                .into(),
        );
        let mut encoder = Encoder::new(batch_size * Encoder::HEARTS_LEN);
        for _ in 0..batch_size {
            encoder = encoder.hearts(Seat::East, state.won);
        }
        input.push(
            Array2::from_shape_vec((batch_size, Encoder::HEARTS_LEN), encoder.into_inner())
                .unwrap()
                .into(),
        );
        let mut encoder = Encoder::new(batch_size * Encoder::CHARGED_LEN);
        for _ in 0..batch_size {
            encoder = encoder.charged(state.charges);
        }
        input.push(
            Array2::from_shape_vec((batch_size, Encoder::CHARGED_LEN), encoder.into_inner())
                .unwrap()
                .into(),
        );
        let mut encoder = Encoder::new(batch_size * Encoder::LED_LEN);
        for _ in 0..batch_size {
            encoder = encoder.led(state.led_suits);
        }
        input.push(
            Array2::from_shape_vec((batch_size, Encoder::LED_LEN), encoder.into_inner())
                .unwrap()
                .into(),
        );
        g.bench_with_input(
            format!("inference{}", batch_size),
            &(model, input),
            |b, (model, input)| {
                b.iter(|| model.run(input.clone()).unwrap());
            },
        );
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut criterion = Criterion::default().measurement_time(Duration::from_secs(10));
    inference(&mut criterion)?;
    criterion.final_summary();
    Ok(())
}
