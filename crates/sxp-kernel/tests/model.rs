// fuzz-profile: proptest =1.12.0, default-features = false, features = ["std"],
// cases = 256, rng_seed = RngSeed::Fixed(0x5111_0007),
// failure_persistence = FileFailurePersistence::Off
// Finite cases are not a proof over every symbol sequence.
// Correctness evidence is separate from tuning-score evidence.

use std::fmt::Write as _;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngSeed};
use sha2::{Digest, Sha256};
use sxp_kernel::{
    DecisionTrace, Error, InitialProbability, ModelConfig, NodeState, Order0Model, RangeDecoder,
    RangeEncoder, STRAWMAN_CAP, STRAWMAN_P, STRAWMAN_RECIP,
};

const RECIP_SHA256: &str = include_str!("../../../spec/vectors/model-recip-strawman.sha256");
const SYMBOLS_SHA256: &str = include_str!("../../../spec/vectors/model-symbols-256.sha256");

fn config() -> Config {
    Config {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        rng_seed: RngSeed::Fixed(0x5111_0007),
        ..Config::default()
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        write!(hex, "{byte:02x}").unwrap();
    }
    hex
}

fn strawman() -> Order0Model {
    Order0Model::new(ModelConfig::strawman().unwrap())
}

fn drive(bit: u8, times: u32) -> (NodeState, (u16, u16)) {
    let mut model = strawman();
    for _ in 0..times {
        model.update(1, bit).unwrap();
    }
    let state = model.node(1).unwrap();
    let freq = model.freq(1).unwrap();
    (state, freq)
}

fn assert_row(bit: u8, times: u32, p: u16, n: u16, freq0: u16, freq1: u16) {
    let (state, freq) = drive(bit, times);
    assert_eq!(state, NodeState { p, n }, "bit {bit} after {times}");
    assert_eq!(freq, (freq0, freq1), "bit {bit} after {times}");
}

#[test]
fn fresh_node_uses_recip_before_the_increment() {
    let model = strawman();
    assert_eq!(model.node(1).unwrap(), NodeState { p: 32_768, n: 0 });
    assert_eq!(model.freq(1).unwrap(), (2048, 2048));

    let mut zeros = strawman();
    zeros.update(1, 0).unwrap();
    assert_eq!(zeros.node(1).unwrap(), NodeState { p: 49_152, n: 1 });
    assert_eq!(zeros.freq(1).unwrap(), (3072, 1024));
    assert_ne!(zeros.node(1).unwrap().p, 43_690);

    let mut ones = strawman();
    ones.update(1, 1).unwrap();
    assert_eq!(ones.node(1).unwrap(), NodeState { p: 16_384, n: 1 });
    assert_eq!(ones.freq(1).unwrap(), (1024, 3072));
}

#[test]
fn cap_boundary_and_saturation_schedules() {
    let zeros = [
        (23, 64_165, 23, 4010, 86),
        (24, 64_219, 24, 4013, 83),
        (25, 64_269, 24, 4016, 80),
        (132, 65_503, 24, 4093, 3),
        (133, 65_504, 24, 4094, 2),
        (140, 65_510, 24, 4094, 2),
        (141, 65_510, 24, 4094, 2),
    ];
    for (times, p, n, freq0, freq1) in zeros {
        assert_row(0, times, p, n, freq0, freq1);
    }
    assert_ne!(drive(0, 25).0.p, 64_271);

    let ones = [
        (23, 1371, 23, 85, 4011),
        (24, 1317, 24, 82, 4014),
        (25, 1267, 24, 79, 4017),
        (133, 32, 24, 2, 4094),
        (134, 31, 24, 1, 4095),
        (140, 26, 24, 1, 4095),
        (141, 26, 24, 1, 4095),
    ];
    for (times, p, n, freq0, freq1) in ones {
        assert_row(1, times, p, n, freq0, freq1);
    }
}

#[test]
fn recip_table_matches_the_committed_hash() {
    assert_eq!(STRAWMAN_P, 32_768);
    assert_eq!(STRAWMAN_CAP, 24);
    assert_eq!(STRAWMAN_RECIP.len(), 25);
    assert_eq!(STRAWMAN_RECIP[0], 32_768);
    assert_eq!(STRAWMAN_RECIP[23], 2621);
    assert_eq!(STRAWMAN_RECIP[24], 2520);
    let mut bytes = Vec::with_capacity(50);
    for rate in STRAWMAN_RECIP {
        bytes.extend_from_slice(&rate.to_le_bytes());
    }
    assert_eq!(bytes.len(), 50);
    assert_eq!(sha256_hex(&bytes), RECIP_SHA256.trim());
}

fn assert_a5(traces: &[DecisionTrace; 8]) {
    let nodes = [1u16, 3, 6, 13, 26, 52, 105, 210];
    let bits = [1u8, 0, 1, 0, 0, 1, 0, 1];
    let ranges = [
        0x8000_07FFu32,
        0x4000_0000,
        0x2000_0000,
        0x1000_0000,
        0x0800_0000,
        0x0400_0000,
        0x0200_0000,
        0x0100_0000,
    ];
    for (index, trace) in traces.iter().enumerate() {
        assert_eq!(trace.node, nodes[index]);
        assert_eq!(trace.bit, bits[index]);
        assert_eq!(trace.n_before, 0);
        assert_eq!(trace.p_before, 32_768);
        assert_eq!(trace.freq0, 2048);
        assert_eq!(trace.freq1, 2048);
        assert_eq!(trace.n_after, 1);
        let p_after = if trace.bit == 0 { 49_152 } else { 16_384 };
        assert_eq!(trace.p_after, p_after);
        assert_eq!(trace.coder_range, ranges[index]);
    }
}

#[test]
fn symbol_a5_encoder_and_decoder_traces_agree() {
    let mut model = strawman();
    let mut encoder = RangeEncoder::new();
    let encoded_traces = model.encode_symbol(&mut encoder, 0xA5).unwrap();
    assert_a5(&encoded_traces);
    assert_eq!(model.node(2).unwrap(), NodeState { p: 32_768, n: 0 });
    assert_eq!(model.node(255).unwrap(), NodeState { p: 32_768, n: 0 });
    assert_eq!(model.node(1).unwrap(), NodeState { p: 16_384, n: 1 });
    assert_eq!(model.node(210).unwrap(), NodeState { p: 16_384, n: 1 });
    assert_eq!(model.node(3).unwrap(), NodeState { p: 49_152, n: 1 });
    assert_eq!(model.freq(1).unwrap(), (1024, 3072));

    let bytes = encoder.finish();
    assert_eq!(bytes, [0xA4, 0xFF, 0xF8, 0x00]);

    let mut decoder_model = strawman();
    let mut decoder = RangeDecoder::new(&bytes).unwrap();
    let (symbol, decoded_traces) = decoder_model.decode_symbol(&mut decoder).unwrap();
    assert_eq!(symbol, 0xA5);
    assert_eq!(decoded_traces, encoded_traces);
    decoder.finish().unwrap();
    assert_eq!(decoder.bytes_consumed(), 4);
}

#[test]
fn every_node_is_visited_and_paths_are_prefix_bits() {
    let mut seen = [false; 256];
    let mut model = strawman();
    let mut encoder = RangeEncoder::new();
    for symbol in 0u16..=255 {
        let traces = model.encode_symbol(&mut encoder, symbol as u8).unwrap();
        for trace in traces {
            seen[usize::from(trace.node)] = true;
        }
    }
    assert!(!seen[0]);
    for (id, visited) in seen.iter().enumerate().skip(1) {
        assert!(*visited, "missing node {id}");
    }
    let _ = encoder.finish();

    let mut zeros = strawman();
    let mut encoder = RangeEncoder::new();
    let traces = zeros.encode_symbol(&mut encoder, 0x00).unwrap();
    let zero_nodes = [1u16, 2, 4, 8, 16, 32, 64, 128];
    for (trace, node) in traces.iter().zip(zero_nodes) {
        assert_eq!(trace.node, node);
        assert_eq!(trace.bit, 0);
    }

    let mut ones = strawman();
    let mut encoder = RangeEncoder::new();
    let traces = ones.encode_symbol(&mut encoder, 0xFF).unwrap();
    let one_nodes = [1u16, 3, 7, 15, 31, 63, 127, 255];
    for (trace, node) in traces.iter().zip(one_nodes) {
        assert_eq!(trace.node, node);
        assert_eq!(trace.bit, 1);
    }
    assert_eq!(ones.node(255).unwrap(), NodeState { p: 16_384, n: 1 });
    assert_eq!(ones.node(2).unwrap(), NodeState { p: 32_768, n: 0 });
}

#[test]
fn frequency_floor_keeps_a_probability_below_sixteen() {
    let config = ModelConfig::try_new(
        InitialProbability::Scalar(16),
        STRAWMAN_CAP,
        &STRAWMAN_RECIP,
    )
    .unwrap();
    let mut model = Order0Model::new(config);
    model.update(1, 1).unwrap();
    assert_eq!(model.node(1).unwrap(), NodeState { p: 8, n: 1 });
    assert_eq!(model.freq(1).unwrap(), (1, 4095));
    assert_eq!(model.freq(1).unwrap(), (1, 4095));
    assert_eq!(model.node(1).unwrap(), NodeState { p: 8, n: 1 });
}

#[test]
fn per_depth_initialization_changes_the_first_frequency() {
    let config = ModelConfig::try_new(
        InitialProbability::ByDepth([16, 32, 48, 64, 80, 96, 112, 128]),
        STRAWMAN_CAP,
        &STRAWMAN_RECIP,
    )
    .unwrap();
    let mut model = Order0Model::new(config);
    let table = [16u16, 32, 48, 64, 80, 96, 112, 128];
    for id in 1u16..=255 {
        let p = table[id.ilog2() as usize];
        assert_eq!(model.node(id).unwrap(), NodeState { p, n: 0 }, "node {id}");
        assert_eq!(model.freq(id).unwrap(), (p >> 4, 4096 - (p >> 4)));
    }
    let mut encoder = RangeEncoder::new();
    let traces = model.encode_symbol(&mut encoder, 0x00).unwrap();
    let first = traces[0];
    assert_eq!(first.p_before, 16);
    assert_eq!(first.freq0, 1);
    assert_eq!(first.bit, 0);
    assert_eq!(first.p_after, 32_776);
    assert_eq!(first.n_after, 1);
    assert_eq!(first.coder_range, 0x0FFF_FF00);
    assert_ne!(first.freq0, 2048);
}

#[test]
fn reset_restores_a_fresh_model() {
    let mut model = strawman();
    let mut encoder = RangeEncoder::new();
    model.encode_symbol(&mut encoder, 0xFF).unwrap();
    let _ = encoder.finish();
    model.reset();
    for id in [7u16, 15, 31, 63, 127, 255] {
        assert_eq!(
            model.node(id).unwrap(),
            NodeState { p: 32_768, n: 0 },
            "node {id}"
        );
    }

    let mut encoder = RangeEncoder::new();
    model.encode_symbol(&mut encoder, 0xA5).unwrap();
    let bytes = encoder.finish();
    assert_eq!(model.node(255).unwrap(), NodeState { p: 32_768, n: 0 });

    let mut fresh = strawman();
    let mut fresh_encoder = RangeEncoder::new();
    fresh.encode_symbol(&mut fresh_encoder, 0xA5).unwrap();
    assert_eq!(bytes, fresh_encoder.finish());
}

#[test]
fn second_zero_symbol_sees_the_updated_node() {
    let mut once = strawman();
    let mut encoder = RangeEncoder::new();
    once.encode_symbol(&mut encoder, 0x00).unwrap();
    assert_eq!(encoder.finish(), [0x00, 0x00, 0x00, 0x00, 0x00]);

    let mut model = strawman();
    let mut encoder = RangeEncoder::new();
    model.encode_symbol(&mut encoder, 0x00).unwrap();
    assert_eq!(model.node(1).unwrap(), NodeState { p: 49_152, n: 1 });
    assert_eq!(model.freq(1).unwrap(), (3072, 1024));
    let second = model.encode_symbol(&mut encoder, 0x00).unwrap();
    let first = second[0];
    assert_eq!(first.node, 1);
    assert_eq!(first.n_before, 1);
    assert_eq!(first.p_before, 49_152);
    assert_eq!(first.freq0, 3072);
    assert_eq!(first.bit, 0);
    assert_eq!(first.n_after, 2);
    assert_eq!(first.p_after, 54_613);
    assert_eq!(first.coder_range, 0xBFFA_0000);
    let bytes = encoder.finish();
    assert_eq!(bytes, [0x00, 0x00, 0x00, 0x00, 0x00]);

    let mut decoder_model = strawman();
    let mut decoder = RangeDecoder::new(&bytes).unwrap();
    let (first_symbol, _) = decoder_model.decode_symbol(&mut decoder).unwrap();
    let (second_symbol, second_traces) = decoder_model.decode_symbol(&mut decoder).unwrap();
    assert_eq!(first_symbol, 0x00);
    assert_eq!(second_symbol, 0x00);
    assert_eq!(second_traces[0].n_before, 1);
    assert_eq!(second_traces[0].p_before, 49_152);
    assert_eq!(second_traces[0].freq0, 3072);
    assert_eq!(second_traces[0].n_after, 2);
    assert_eq!(second_traces[0].p_after, 54_613);
    assert_eq!(second_traces[0].coder_range, 0xBFFA_0000);
    decoder.finish().unwrap();
}

#[test]
fn rejected_configurations_and_ids_do_not_change_state() {
    for p in [0u16, 15, 65_521, 65_535] {
        assert_eq!(
            ModelConfig::try_new(InitialProbability::Scalar(p), STRAWMAN_CAP, &STRAWMAN_RECIP),
            Err(Error::InvalidParameter)
        );
    }
    assert!(
        ModelConfig::try_new(
            InitialProbability::Scalar(16),
            STRAWMAN_CAP,
            &STRAWMAN_RECIP
        )
        .is_ok()
    );
    assert!(
        ModelConfig::try_new(
            InitialProbability::Scalar(65_520),
            STRAWMAN_CAP,
            &STRAWMAN_RECIP
        )
        .is_ok()
    );

    for bad in [15u16, 65_521] {
        let mut table = [32_768u16; 8];
        table[3] = bad;
        assert_eq!(
            ModelConfig::try_new(
                InitialProbability::ByDepth(table),
                STRAWMAN_CAP,
                &STRAWMAN_RECIP
            ),
            Err(Error::InvalidParameter)
        );
    }

    assert_eq!(
        ModelConfig::try_new(InitialProbability::Scalar(STRAWMAN_P), 0, &[1]),
        Err(Error::InvalidParameter)
    );
    let wide_recip = vec![1u16; 257];
    assert_eq!(
        ModelConfig::try_new(InitialProbability::Scalar(STRAWMAN_P), 256, &wide_recip),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        ModelConfig::try_new(
            InitialProbability::Scalar(STRAWMAN_P),
            24,
            &STRAWMAN_RECIP[..24]
        ),
        Err(Error::InvalidParameter)
    );
    let mut too_long = STRAWMAN_RECIP.to_vec();
    too_long.push(1);
    assert_eq!(
        ModelConfig::try_new(InitialProbability::Scalar(STRAWMAN_P), 24, &too_long),
        Err(Error::InvalidParameter)
    );
    let mut zero_rate = STRAWMAN_RECIP;
    zero_rate[4] = 0;
    assert_eq!(
        ModelConfig::try_new(
            InitialProbability::Scalar(STRAWMAN_P),
            STRAWMAN_CAP,
            &zero_rate
        ),
        Err(Error::InvalidParameter)
    );
    assert!(ModelConfig::try_new(InitialProbability::Scalar(32_768), 1, &[1, 65_535]).is_ok());

    let mut model = strawman();
    assert_eq!(model.node(0), Err(Error::InvalidParameter));
    assert_eq!(model.freq(0), Err(Error::InvalidParameter));
    assert_eq!(model.update(0, 0), Err(Error::InvalidParameter));
    assert_eq!(model.node(256), Err(Error::InvalidParameter));
    assert_eq!(model.update(1, 2), Err(Error::InvalidParameter));
    assert_eq!(model.node(1).unwrap(), NodeState { p: 32_768, n: 0 });
    model.update(1, 0).unwrap();
    assert_eq!(model.node(1).unwrap(), NodeState { p: 49_152, n: 1 });
}

#[test]
fn symbols_256_match_the_committed_hash() {
    let mut model = strawman();
    let mut encoder = RangeEncoder::new();
    for symbol in 0u16..=255 {
        model.encode_symbol(&mut encoder, symbol as u8).unwrap();
    }
    let encoded = encoder.finish();
    assert_eq!(encoded.len(), 257);
    assert_eq!(
        &encoded[..8],
        &[0x00, 0x1C, 0x1C, 0x0B, 0x1F, 0x9A, 0x8A, 0x7D]
    );
    assert_eq!(
        &encoded[encoded.len() - 8..],
        &[0x1F, 0x9B, 0x58, 0x5D, 0x2B, 0x41, 0x53, 0x00]
    );
    assert_eq!(sha256_hex(&encoded), SYMBOLS_SHA256.trim());

    let mut decoder_model = strawman();
    let mut decoder = RangeDecoder::new(&encoded).unwrap();
    for symbol in 0u16..=255 {
        let (decoded, _) = decoder_model.decode_symbol(&mut decoder).unwrap();
        assert_eq!(decoded, symbol as u8);
    }
    decoder.finish().unwrap();
    assert_eq!(decoder.bytes_consumed(), 257);
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn random_symbols_round_trip(symbols in prop::collection::vec(any::<u8>(), 0..=48)) {
        let mut model = strawman();
        let mut encoder = RangeEncoder::new();
        for &symbol in &symbols {
            model.encode_symbol(&mut encoder, symbol).unwrap();
        }
        let encoded = encoder.finish();
        let mut decoder_model = strawman();
        let mut decoder = RangeDecoder::new(&encoded).unwrap();
        for &symbol in &symbols {
            let (decoded, _) = decoder_model.decode_symbol(&mut decoder).unwrap();
            prop_assert_eq!(decoded, symbol);
        }
        decoder.finish().unwrap();
        prop_assert_eq!(decoder.bytes_consumed(), encoded.len());
    }

    #[test]
    fn random_bits_stay_inside_the_legal_interval(bits in prop::collection::vec(0u8..=1, 256)) {
        let mut model = strawman();
        for bit in bits {
            model.update(1, bit).unwrap();
            let state = model.node(1).unwrap();
            let (freq0, freq1) = model.freq(1).unwrap();
            prop_assert!((1..=65_535).contains(&state.p));
            prop_assert!(state.n <= STRAWMAN_CAP);
            prop_assert!((1..=4095).contains(&freq0));
            prop_assert_eq!(freq1, 4096 - freq0);
        }
    }
}
