use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use sxp_encoder::{
    Choice, Error, LengthPolicy, SearchCandidate, SearchMode, Selection, SpanCost, SpanTable,
    all_baselines, allowed_nominals, candidate_id, check_length, choose_candidate,
    encode_baselines, reduce, select_baselines,
};
use sxp_kernel::{
    BlockKind, LENGTH_LIMIT, NominalLength, actual_len, encode_block, read_blocked_file,
    read_whole_file_raw, write_whole_file_raw,
};

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn zeros(n: usize) -> Vec<u8> {
    vec![0u8; n]
}

fn as_search(kinds: &[BlockKind]) -> Vec<SearchCandidate> {
    kinds
        .iter()
        .copied()
        .map(|kind| SearchCandidate {
            id: candidate_id(kind).expect("baseline"),
            kind,
        })
        .collect()
}

fn raw_span(bytes: u64) -> SpanCost {
    SpanCost {
        id: 0,
        bytes,
        kind: BlockKind::Raw,
    }
}

fn constants() -> Vec<BlockKind> {
    (0..=255).map(BlockKind::Constant).collect()
}

fn select(
    input: &[u8],
    candidates: &[BlockKind],
    policy: LengthPolicy,
    mode: SearchMode,
    threads: usize,
) -> Selection {
    select_baselines(input, candidates, policy, mode, threads).expect("select")
}

fn encode(
    input: &[u8],
    candidates: &[BlockKind],
    policy: LengthPolicy,
    mode: SearchMode,
    threads: usize,
) -> Vec<u8> {
    let selection = select(input, candidates, policy, mode, threads);
    let bytes = encode_baselines(input, candidates, policy, mode, threads).expect("encode");
    assert_eq!(bytes.len() as u64, selection.file_len);
    assert!(bytes.len() as u64 <= input.len() as u64 + 11);
    round_trip(input, &selection, &bytes);
    bytes
}

fn round_trip(input: &[u8], selection: &Selection, bytes: &[u8]) {
    let mut decoded = Vec::new();
    if selection.whole_file_raw {
        read_whole_file_raw(bytes, &mut decoded).expect("raw decode");
    } else {
        read_blocked_file(bytes, &mut decoded).expect("blocked decode");
    }
    assert_eq!(decoded, input);
}

#[test]
fn length_limit_is_rejected_without_a_buffer() {
    let err = check_length(LENGTH_LIMIT).expect_err("limit");
    assert_eq!(err.to_string(), "LengthOutOfRange");
    assert_eq!(Error::NoCandidate.to_string(), "NoCandidate");
    assert_eq!(Error::ThreadPool.to_string(), "ThreadPool");
    assert_eq!(Error::MissingSpan.to_string(), "MissingSpan");
}

#[test]
fn allowed_nominals_match_the_tables() {
    let all = [
        (1u64, vec![NominalLength::N256]),
        (255, vec![NominalLength::N256]),
        (256, vec![NominalLength::N256]),
        (257, vec![NominalLength::N256, NominalLength::N1024]),
        (1023, vec![NominalLength::N256, NominalLength::N1024]),
        (1024, vec![NominalLength::N256, NominalLength::N1024]),
        (
            1025,
            vec![
                NominalLength::N256,
                NominalLength::N1024,
                NominalLength::N2048,
            ],
        ),
        (
            2047,
            vec![
                NominalLength::N256,
                NominalLength::N1024,
                NominalLength::N2048,
            ],
        ),
        (
            2048,
            vec![
                NominalLength::N256,
                NominalLength::N1024,
                NominalLength::N2048,
            ],
        ),
        (
            2049,
            vec![
                NominalLength::N256,
                NominalLength::N1024,
                NominalLength::N2048,
            ],
        ),
    ];
    for (remaining, expected) in all {
        assert_eq!(
            allowed_nominals(remaining, LengthPolicy::All),
            expected,
            "all {remaining}"
        );
    }
    assert!(allowed_nominals(0, LengthPolicy::All).is_empty());

    let fixed = [
        (1u64, NominalLength::N256, vec![NominalLength::N256]),
        (256, NominalLength::N256, vec![NominalLength::N256]),
        (257, NominalLength::N256, vec![NominalLength::N256]),
        (2049, NominalLength::N256, vec![NominalLength::N256]),
        (4096, NominalLength::N256, vec![NominalLength::N256]),
        (1, NominalLength::N1024, vec![NominalLength::N256]),
        (256, NominalLength::N1024, vec![NominalLength::N256]),
        (257, NominalLength::N1024, vec![NominalLength::N1024]),
        (1024, NominalLength::N1024, vec![NominalLength::N1024]),
        (1025, NominalLength::N1024, vec![NominalLength::N1024]),
        (2048, NominalLength::N1024, vec![NominalLength::N1024]),
        (2049, NominalLength::N1024, vec![NominalLength::N1024]),
        (4096, NominalLength::N1024, vec![NominalLength::N1024]),
        (1, NominalLength::N2048, vec![NominalLength::N256]),
        (256, NominalLength::N2048, vec![NominalLength::N256]),
        (257, NominalLength::N2048, vec![NominalLength::N1024]),
        (1024, NominalLength::N2048, vec![NominalLength::N1024]),
        (1025, NominalLength::N2048, vec![NominalLength::N2048]),
        (2048, NominalLength::N2048, vec![NominalLength::N2048]),
        (2049, NominalLength::N2048, vec![NominalLength::N2048]),
        (4096, NominalLength::N2048, vec![NominalLength::N2048]),
    ];
    for (remaining, fixed_nominal, expected) in fixed {
        assert_eq!(
            allowed_nominals(remaining, LengthPolicy::Fixed(fixed_nominal)),
            expected,
            "fixed {fixed_nominal:?} at {remaining}"
        );
    }
}

#[test]
fn candidate_ids_are_raw_then_baselines_then_constants() {
    assert_eq!(candidate_id(BlockKind::Raw), Some(0));
    assert_eq!(candidate_id(BlockKind::Direct), Some(1));
    assert_eq!(candidate_id(BlockKind::PreviousByte), Some(2));
    assert_eq!(candidate_id(BlockKind::Constant(0)), Some(3));
    assert_eq!(candidate_id(BlockKind::Constant(1)), Some(4));
    assert_eq!(candidate_id(BlockKind::Constant(255)), Some(258));
    assert_eq!(all_baselines().len(), 259);
}

#[test]
fn header_prune_skips_constants_only_when_the_incumbent_is_at_most_two() {
    let kinds = all_baselines();
    let reference_calls = Cell::new(0u32);
    let prune_calls = Cell::new(0u32);
    let reference = choose_candidate(&as_search(&kinds), SearchMode::Reference, |candidate| {
        reference_calls.set(reference_calls.get() + 1);
        Ok(if candidate.kind == BlockKind::Raw {
            2
        } else {
            50
        })
    })
    .expect("reference");
    let pruned = choose_candidate(&as_search(&kinds), SearchMode::HeaderPrune, |candidate| {
        prune_calls.set(prune_calls.get() + 1);
        Ok(if candidate.kind == BlockKind::Raw {
            2
        } else {
            50
        })
    })
    .expect("prune");
    assert_eq!(reference_calls.get(), 259);
    assert_eq!(prune_calls.get(), 3);
    assert_eq!(reference.1, 0);
    assert_eq!(reference.2, 2);
    assert_eq!(pruned.1, 0);
    assert_eq!(pruned.2, 2);

    let mut reversed = all_baselines();
    reversed.reverse();
    let reversed_calls = Cell::new(0u32);
    let reversed_winner = choose_candidate(
        &as_search(&reversed),
        SearchMode::HeaderPrune,
        |candidate| {
            reversed_calls.set(reversed_calls.get() + 1);
            Ok(if candidate.kind == BlockKind::Raw {
                2
            } else {
                50
            })
        },
    )
    .expect("reversed prune");
    assert_eq!(reversed_calls.get(), 3);
    assert_eq!(reversed_winner.1, 0);
    assert_eq!(reversed_winner.2, 2);

    let wide_calls = Cell::new(0u32);
    let wide = choose_candidate(&as_search(&kinds), SearchMode::HeaderPrune, |candidate| {
        wide_calls.set(wide_calls.get() + 1);
        Ok(match candidate.kind {
            BlockKind::Raw => 5,
            BlockKind::Direct => 4,
            _ => 9,
        })
    })
    .expect("wide");
    assert_eq!(wide_calls.get(), 259);
    assert_eq!(wide.1, 1);
    assert_eq!(wide.2, 4);
}

#[test]
fn empty_input_is_the_seven_byte_raw_stream() {
    let selection = select(&[], &[], LengthPolicy::All, SearchMode::Reference, 0);
    assert!(selection.whole_file_raw);
    assert!(selection.choices.is_empty());
    assert_eq!(selection.key.block_bytes, 0);
    assert_eq!(selection.key.block_count, 0);
    assert_eq!(selection.file_len, 7);
    let bytes = encode(
        &[],
        &all_baselines(),
        LengthPolicy::All,
        SearchMode::Reference,
        1,
    );
    assert_eq!(hex(&bytes), "8d 11 00 00 00 00 00");
}

#[test]
fn empty_candidate_list_is_an_error_when_there_are_bytes() {
    let err = select_baselines(&[0], &[], LengthPolicy::All, SearchMode::Reference, 1)
        .expect_err("empty");
    assert_eq!(err.to_string(), "NoCandidate");
}

#[test]
fn pinned_zero_fixtures() {
    let baselines = all_baselines();

    let six = select(
        &zeros(6),
        &baselines,
        LengthPolicy::All,
        SearchMode::Reference,
        1,
    );
    assert_eq!(six.key.steps, vec![(0, 0)]);
    assert_eq!(six.key.block_bytes, 7);
    assert_eq!(six.key.block_count, 1);
    assert!(six.whole_file_raw);
    let mut raw6 = Vec::new();
    write_whole_file_raw(6, &zeros(6)[..], &mut raw6).expect("raw6");
    assert_eq!(
        encode(
            &zeros(6),
            &baselines,
            LengthPolicy::All,
            SearchMode::Reference,
            1
        ),
        raw6
    );

    let seven = encode(
        &zeros(7),
        &baselines,
        LengthPolicy::All,
        SearchMode::Reference,
        1,
    );
    assert_eq!(hex(&seven), "8d 11 07 00 00 00 00 00 00 00 7e df 6c 9d");
    let seven_key = select(
        &zeros(7),
        &baselines,
        LengthPolicy::All,
        SearchMode::Reference,
        1,
    );
    assert_eq!(seven_key.key.steps, vec![(0, 1)]);
    assert_eq!(seven_key.key.block_bytes, 7);
    assert!(seven_key.whole_file_raw);
    assert_eq!(seven_key.choices[0].kind, BlockKind::Direct);

    let eight = encode(
        &zeros(8),
        &baselines,
        LengthPolicy::All,
        SearchMode::Reference,
        1,
    );
    assert_eq!(hex(&eight), "8d 01 08 04 00 00 00 00 00 00 69 df 22 65");
    let eight_key = select(
        &zeros(8),
        &baselines,
        LengthPolicy::All,
        SearchMode::Reference,
        1,
    );
    assert_eq!(eight_key.key.steps, vec![(0, 1)]);
    assert_eq!(eight_key.key.block_bytes, 7);
    assert!(!eight_key.whole_file_raw);

    let one = select(
        &zeros(1),
        &constants(),
        LengthPolicy::All,
        SearchMode::Reference,
        1,
    );
    assert_eq!(one.key.steps, vec![(0, 4)]);
    assert_eq!(one.key.block_bytes, 6);
    assert!(one.whole_file_raw);
    assert_eq!(one.choices[0].kind, BlockKind::Constant(1));

    let nine = encode(
        &zeros(9),
        &constants(),
        LengthPolicy::All,
        SearchMode::Reference,
        1,
    );
    assert_eq!(hex(&nine), "8d 01 09 0c 00 00 00 00 00 00 00 ae 14 09 e6");
    let nine_key = select(
        &zeros(9),
        &constants(),
        LengthPolicy::All,
        SearchMode::Reference,
        1,
    );
    assert_eq!(nine_key.key.steps, vec![(0, 3)]);
    assert_eq!(nine_key.key.block_bytes, 8);
    assert!(!nine_key.whole_file_raw);
    assert_eq!(nine_key.choices[0].kind, BlockKind::Constant(0));
}

#[test]
fn call_order_does_not_change_the_eight_zero_file() {
    let mut reversed = all_baselines();
    reversed.reverse();
    let forward = encode(
        &zeros(8),
        &all_baselines(),
        LengthPolicy::All,
        SearchMode::Reference,
        1,
    );
    let backward = encode(
        &zeros(8),
        &reversed,
        LengthPolicy::All,
        SearchMode::Reference,
        1,
    );
    assert_eq!(forward, backward);
}

#[test]
fn injected_ties_use_the_production_reducer() {
    let mut count_tie = SpanTable::new();
    count_tie.insert((0, 1), raw_span(10));
    count_tie.insert((0, 0), raw_span(5));
    count_tie.insert((256, 0), raw_span(5));
    let key = reduce(512, LengthPolicy::All, &count_tie).expect("count");
    assert_eq!(key.block_bytes, 10);
    assert_eq!(key.block_count, 1);
    assert_eq!(key.steps, vec![(1, 0)]);

    let mut lex_tie = SpanTable::new();
    for (offset, nominal) in every_span(1280, LengthPolicy::All) {
        lex_tie.insert((offset, nominal.code()), raw_span(100));
    }
    lex_tie.insert((0, 0), raw_span(6));
    lex_tie.insert((256, 1), raw_span(6));
    lex_tie.insert((0, 1), raw_span(6));
    lex_tie.insert((1024, 0), raw_span(6));
    let key = reduce(1280, LengthPolicy::All, &lex_tie).expect("lex");
    assert_eq!(key.block_bytes, 12);
    assert_eq!(key.block_count, 2);
    assert_eq!(key.steps, vec![(0, 0), (1, 0)]);

    let missing = reduce(256, LengthPolicy::All, &SpanTable::new()).expect_err("missing");
    assert_eq!(missing.to_string(), "MissingSpan");
}

#[test]
fn fixed_arms_emit_the_canonical_nominal_and_round_trip() {
    let baselines = all_baselines();
    let cases = [
        (256usize, NominalLength::N1024, NominalLength::N256),
        (257, NominalLength::N1024, NominalLength::N1024),
        (300, NominalLength::N2048, NominalLength::N1024),
    ];
    for (len, policy, nominal) in cases {
        let input = zeros(len);
        let selection = select(
            &input,
            &baselines,
            LengthPolicy::Fixed(policy),
            SearchMode::Reference,
            1,
        );
        assert_eq!(selection.choices.len(), 1, "len {len}");
        assert_eq!(selection.choices[0].nominal, nominal, "len {len}");
        assert!(!selection.whole_file_raw, "len {len}");
        encode(
            &input,
            &baselines,
            LengthPolicy::Fixed(policy),
            SearchMode::Reference,
            1,
        );
    }

    let chains = [
        (
            257usize,
            NominalLength::N256,
            vec![NominalLength::N256, NominalLength::N256],
        ),
        (
            1025,
            NominalLength::N1024,
            vec![NominalLength::N1024, NominalLength::N256],
        ),
        (
            600,
            NominalLength::N256,
            vec![
                NominalLength::N256,
                NominalLength::N256,
                NominalLength::N256,
            ],
        ),
    ];
    for (len, policy, nominals) in chains {
        let input = zeros(len);
        let selection = select(
            &input,
            &baselines,
            LengthPolicy::Fixed(policy),
            SearchMode::Reference,
            1,
        );
        assert!(!selection.whole_file_raw, "chain {len}");
        assert_eq!(selection.choices.len(), nominals.len(), "chain {len}");
        for (choice, nominal) in selection.choices.iter().zip(nominals.iter()) {
            assert_eq!(choice.nominal, *nominal, "chain {len}");
        }
        encode(
            &input,
            &baselines,
            LengthPolicy::Fixed(policy),
            SearchMode::Reference,
            1,
        );
    }
}

#[test]
fn thread_counts_and_header_prune_match_on_the_search_set() {
    let baselines = all_baselines();
    for len in [1usize, 7, 8, 255, 256, 257, 512, 600] {
        let input = fill(len);
        let reference = select(
            &input,
            &baselines,
            LengthPolicy::All,
            SearchMode::Reference,
            1,
        );
        let quiet = select(
            &input,
            &baselines,
            LengthPolicy::All,
            SearchMode::Reference,
            0,
        );
        let parallel = select(
            &input,
            &baselines,
            LengthPolicy::All,
            SearchMode::Reference,
            2,
        );
        let pruned = select(
            &input,
            &baselines,
            LengthPolicy::All,
            SearchMode::HeaderPrune,
            2,
        );
        assert_eq!(quiet.key, reference.key, "threads 0 len {len}");
        assert_eq!(parallel.key, reference.key, "threads 2 len {len}");
        assert_eq!(pruned.key, reference.key, "prune len {len}");
        let reference_bytes = encode(
            &input,
            &baselines,
            LengthPolicy::All,
            SearchMode::Reference,
            1,
        );
        let parallel_bytes = encode(
            &input,
            &baselines,
            LengthPolicy::All,
            SearchMode::Reference,
            2,
        );
        let pruned_bytes = encode(
            &input,
            &baselines,
            LengthPolicy::All,
            SearchMode::HeaderPrune,
            1,
        );
        assert_eq!(parallel_bytes, reference_bytes, "bytes threads {len}");
        assert_eq!(pruned_bytes, reference_bytes, "bytes prune {len}");
        assert_eq!(brute(&input), reference.key.steps, "brute steps {len}");
        assert_eq!(
            brute_bytes(&input),
            reference.key.block_bytes,
            "brute bytes {len}"
        );
        assert_eq!(
            brute_count(&input),
            reference.key.block_count,
            "brute count {len}"
        );
        same_span(&input, &reference.choices);
    }
}

fn fill(len: usize) -> Vec<u8> {
    let mut state = 0x5111_0009u32;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        out.push(u8::try_from(state & 0xff).unwrap_or(0));
    }
    out
}

fn every_span(n: u64, policy: LengthPolicy) -> Vec<(u64, NominalLength)> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![0u64];
    let mut spans = Vec::new();
    while let Some(offset) = pending.pop() {
        if offset == n || !seen.insert(offset) {
            continue;
        }
        let remaining = n - offset;
        for nominal in allowed_nominals(remaining, policy) {
            spans.push((offset, nominal));
            let next = offset + actual_len(nominal, remaining);
            if next < n {
                pending.push(next);
            }
        }
    }
    spans
}

#[derive(Clone)]
struct Brute {
    bytes: u64,
    count: u32,
    steps: Vec<(u8, u16)>,
}

fn brute_at(input: &[u8], offset: u64, memo: &mut BTreeMap<u64, Brute>) -> Brute {
    let n = input.len() as u64;
    if offset == n {
        return Brute {
            bytes: 0,
            count: 0,
            steps: Vec::new(),
        };
    }
    if let Some(found) = memo.get(&offset) {
        return found.clone();
    }
    let remaining = n - offset;
    let mut best: Option<Brute> = None;
    for nominal in allowed_nominals(remaining, LengthPolicy::All) {
        let actual = actual_len(nominal, remaining);
        let start = offset as usize;
        let end = (offset + actual) as usize;
        let slice = &input[start..end];
        let mut span: Option<(u64, u16)> = None;
        for kind in all_baselines() {
            let encoded = encode_block(kind, nominal, remaining, slice).expect("block");
            let bytes = encoded.len() as u64;
            let id = candidate_id(kind).expect("baseline");
            let replace = match span {
                None => true,
                Some((best_bytes, best_id)) => (bytes, id) < (best_bytes, best_id),
            };
            if replace {
                span = Some((bytes, id));
            }
        }
        let (edge_bytes, id) = span.expect("span");
        let suffix = brute_at(input, offset + actual, memo);
        let mut steps = vec![(nominal.code(), id)];
        steps.extend(suffix.steps.iter().copied());
        let candidate = Brute {
            bytes: edge_bytes + suffix.bytes,
            count: suffix.count + 1,
            steps,
        };
        let replace = match &best {
            None => true,
            Some(incumbent) => {
                (candidate.bytes, candidate.count, &candidate.steps)
                    < (incumbent.bytes, incumbent.count, &incumbent.steps)
            }
        };
        if replace {
            best = Some(candidate);
        }
    }
    let winner = best.expect("partition");
    memo.insert(offset, winner.clone());
    winner
}

fn brute(input: &[u8]) -> Vec<(u8, u16)> {
    brute_at(input, 0, &mut BTreeMap::new()).steps
}

fn brute_bytes(input: &[u8]) -> u64 {
    brute_at(input, 0, &mut BTreeMap::new()).bytes
}

fn brute_count(input: &[u8]) -> u32 {
    brute_at(input, 0, &mut BTreeMap::new()).count
}

fn same_span(input: &[u8], choices: &[Choice]) {
    let n = input.len() as u64;
    let mut offset = 0u64;
    for choice in choices {
        let remaining = n - offset;
        let actual = actual_len(choice.nominal, remaining);
        let start = offset as usize;
        let end = (offset + actual) as usize;
        let slice = &input[start..end];
        for kind in all_baselines() {
            let encoded = encode_block(kind, choice.nominal, remaining, slice).expect("span");
            let len = encoded.len() as u64;
            let id = candidate_id(kind).expect("baseline");
            assert!(
                len > choice.block_len || (len == choice.block_len && id >= choice.id),
                "smaller candidate {id} at {offset}"
            );
        }
        offset += actual;
    }
    assert_eq!(offset, n);
}
