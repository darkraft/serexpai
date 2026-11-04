//! Byte-account parts. Empty, one byte, and two Blocks.

use sxp_kernel::{
    BlockKind, BlockPlan, ModelConfig, NominalLength, write_blocked_accounted, write_blocked_file,
    write_raw_accounted,
};

fn parts_cover(bytes: &[u8], header: u64, middle: u64, crc: u64) {
    let header = usize::try_from(header).unwrap();
    let middle = usize::try_from(middle).unwrap();
    let crc = usize::try_from(crc).unwrap();
    assert_eq!(header + middle + crc, bytes.len());
    let mut joined = bytes[..header].to_vec();
    joined.extend_from_slice(&bytes[header..header + middle]);
    joined.extend_from_slice(&bytes[header + middle..]);
    assert_eq!(joined, bytes);
}

#[test]
fn raw_accounts_split_into_header_payload_and_crc() {
    let empty = write_raw_accounted(0, &[]).unwrap();
    assert_eq!(empty.account.total() as usize, empty.bytes.len());
    assert_eq!(empty.bytes.len(), 7);
    assert_eq!(empty.account.raw_payload, 0);
    assert_eq!(empty.account.crc, 4);
    assert_eq!(empty.account.block_headers, 0);
    assert_eq!(empty.account.coder_payload, 0);
    assert_eq!(empty.account.coder_flush, 0);
    parts_cover(
        &empty.bytes,
        empty.account.file_header,
        empty.account.raw_payload,
        empty.account.crc,
    );

    let one = write_raw_accounted(1, &[0x11]).unwrap();
    assert_eq!(one.account.total() as usize, one.bytes.len());
    assert_eq!(one.account.raw_payload, 1);
    assert_eq!(one.account.crc, 4);
    parts_cover(
        &one.bytes,
        one.account.file_header,
        one.account.raw_payload,
        one.account.crc,
    );
}

#[test]
fn two_raw_blocks_and_one_direct_block_match_their_accounts() {
    let left = vec![0u8; 256];
    let right = vec![1u8; 256];
    let plans = [
        BlockPlan {
            kind: BlockKind::Raw,
            nominal: NominalLength::N256,
            bytes: &left,
        },
        BlockPlan {
            kind: BlockKind::Raw,
            nominal: NominalLength::N256,
            bytes: &right,
        },
    ];
    let model = ModelConfig::strawman().unwrap();
    let file = write_blocked_accounted(512, &plans, &model).unwrap();
    let mut expected = Vec::new();
    write_blocked_file(512, &plans, &mut expected).unwrap();
    assert_eq!(file.bytes, expected);
    assert_eq!(file.account.total() as usize, file.bytes.len());
    assert_eq!(file.account.block_headers, 2);
    assert_eq!(file.account.parameters, 0);
    assert_eq!(file.account.alignment_padding, 0);
    assert_eq!(file.account.raw_payload, 512);
    assert_eq!(file.account.coder_payload, 0);
    assert_eq!(file.account.coder_flush, 0);
    assert_eq!(file.account.crc, 4);
    let middle = file.account.block_headers
        + file.account.parameters
        + file.account.alignment_padding
        + file.account.raw_payload
        + file.account.coder_payload
        + file.account.coder_flush;
    parts_cover(
        &file.bytes,
        file.account.file_header,
        middle,
        file.account.crc,
    );

    let coded = [0u8];
    let direct = [BlockPlan {
        kind: BlockKind::Direct,
        nominal: NominalLength::N256,
        bytes: &coded,
    }];
    let coded_file = write_blocked_accounted(1, &direct, &model).unwrap();
    assert_eq!(coded_file.account.total() as usize, coded_file.bytes.len());
    assert_eq!(coded_file.account.raw_payload, 0);
    assert_eq!(coded_file.account.block_headers, 1);
    assert_eq!(coded_file.account.parameters, 0);
    assert_eq!(coded_file.account.coder_flush, 5);
    assert_eq!(coded_file.account.coder_payload, 0);
    let coded_middle = coded_file.account.block_headers
        + coded_file.account.parameters
        + coded_file.account.alignment_padding
        + coded_file.account.raw_payload
        + coded_file.account.coder_payload
        + coded_file.account.coder_flush;
    parts_cover(
        &coded_file.bytes,
        coded_file.account.file_header,
        coded_middle,
        coded_file.account.crc,
    );
}
