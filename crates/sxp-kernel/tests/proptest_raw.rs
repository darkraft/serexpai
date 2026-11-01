// fuzz-profile: proptest =1.12.0, default-features = false, features = ["std"],
// cases = 256, rng_seed = RngSeed::Fixed(0x5111_0001),
// failure_persistence = FileFailurePersistence::Off
// Finite cases are not a proof over every byte string.
// A failing generated case is reduced and added under spec/vectors/reject/.

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngSeed};
use sxp_kernel::{Error, encode_whole_file_raw_header, read_whole_file_raw, write_whole_file_raw};

fn config() -> Config {
    Config {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        rng_seed: RngSeed::Fixed(0x5111_0001),
        ..Config::default()
    }
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn round_trip_payloads(payload in proptest::collection::vec(any::<u8>(), 0..=4096usize)) {
        let mut encoded = Vec::new();
        write_whole_file_raw(payload.len() as u64, &mut payload.as_slice(), &mut encoded).unwrap();
        let header = encode_whole_file_raw_header(payload.len() as u64).unwrap();
        let leb = u64::from(header.len) - 2;
        prop_assert_eq!(encoded.len() as u64, payload.len() as u64 + 6 + leb);
        let mut sink = Vec::new();
        read_whole_file_raw(&mut encoded.as_slice(), &mut sink).unwrap();
        prop_assert_eq!(sink, payload);
    }

    #[test]
    fn suffixes_after_magic_do_not_panic(extra in proptest::collection::vec(any::<u8>(), 0..=64usize)) {
        let mut input = vec![0x8D, 0x11];
        input.extend(extra);
        let mut sink = Vec::new();
        let _ = read_whole_file_raw(&mut input.as_slice(), &mut sink);
    }
}

#[test]
fn truncation_prefixes_of_known_streams() {
    let streams: [&[u8]; 2] = [
        include_bytes!("../../../spec/vectors/whole-file-raw-empty.bin"),
        &[0x8D, 0x11, 0x01, 0xA5, 0xEA, 0xB8, 0xBE, 0x74],
    ];
    for stream in streams {
        for len in 0..stream.len() {
            let mut sink = Vec::new();
            let err = read_whole_file_raw(&stream[..len], &mut sink).expect_err("prefix");
            assert_eq!(err, prefix_error(stream, len), "len {len}");
        }
    }
}

fn prefix_error(stream: &[u8], len: usize) -> Error {
    if len < 2 {
        return Error::TruncatedHeader;
    }
    if len == 2 {
        return Error::TruncatedLeb128;
    }
    if stream.get(2).copied() == Some(0x01) && len == 3 {
        return Error::TruncatedPayload;
    }
    Error::TruncatedTrailer
}
