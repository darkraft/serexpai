use sxp_kernel::{
    BlockKind, BlockPlan, Error, NominalLength, read_file, write_blocked_file, write_whole_file_raw,
};

#[test]
fn read_file_round_trips_both_modes_and_rejects_before_a_body() {
    let mut raw = Vec::new();
    write_whole_file_raw(0, &mut std::io::empty(), &mut raw).unwrap();
    let mut sink = Vec::new();
    read_file(raw.as_slice(), &mut sink).unwrap();
    assert!(sink.is_empty());

    let payload = [0x11u8];
    let plan = BlockPlan {
        kind: BlockKind::Raw,
        nominal: NominalLength::N256,
        bytes: &payload,
    };
    let mut blocked = Vec::new();
    write_blocked_file(1, &[plan], &mut blocked).unwrap();
    sink.clear();
    read_file(blocked.as_slice(), &mut sink).unwrap();
    assert_eq!(sink, payload);

    let overflow = include_bytes!("../../../spec/vectors/reject/leb-overflow.bin");
    sink.clear();
    let err = read_file(overflow.as_slice(), &mut sink).unwrap_err();
    assert_eq!(err, Error::Leb128Overflow);
    assert!(sink.is_empty());

    sink.clear();
    let err = read_file([0x00u8].as_slice(), &mut sink).unwrap_err();
    assert_eq!(err, Error::BadMagic);
    assert!(sink.is_empty());
}

#[test]
fn read_file_reports_a_crc_mismatch_after_a_short_body() {
    let mut raw = Vec::new();
    write_whole_file_raw(0, &mut std::io::empty(), &mut raw).unwrap();
    let last = raw.len() - 1;
    raw[last] = raw[last].wrapping_add(1);
    let mut sink = Vec::new();
    let err = read_file(raw.as_slice(), &mut sink).unwrap_err();
    assert_eq!(err, Error::CrcMismatch);
}
