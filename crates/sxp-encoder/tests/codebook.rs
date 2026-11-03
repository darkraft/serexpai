use std::cell::Cell;

use sha2::{Digest, Sha256};
use sxp_encoder::{
    CostCell, Error, LengthPolicy, SearchCandidate, SearchMode, all_baselines, assemble_codebook,
    canonical_bytes, check_family_caps, choose_candidate, encode_codebook, generate_codebook,
    select_baselines, select_codebook, span_cells,
};
use sxp_kernel::{
    BlockKind, BlockPlan, NominalLength, Polynomial, encode_block, read_blocked_file,
    write_blocked_file,
};

const FIXTURE_HASH: &str = include_str!("../../../spec/vectors/codebook-fixture-v1.sha256");

fn fixture() -> sxp_encoder::Codebook {
    generate_codebook("fixture-v1").expect("fixture")
}

fn trace(kind: BlockKind, len: u16) -> Vec<u8> {
    match kind {
        BlockKind::Polynomial { formula, .. } => {
            (0..len).map(|n| formula.predict(n).unwrap()).collect()
        }
        BlockKind::Trigonometric { formula, .. } => {
            (0..len).map(|n| formula.predict(n).unwrap()).collect()
        }
        BlockKind::Recurrence { formula, .. } => formula.predict_span(len).unwrap(),
        _ => panic!("baseline"),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

#[test]
fn fixture_is_three_formulas_and_hashes() {
    let book = fixture();
    assert_eq!(book.name, "fixture-v1");
    assert_eq!(book.label, "fixture");
    assert_eq!(book.version, 1);
    assert_eq!(book.formulas.len(), 3);
    assert_ne!(book.label, "benchmark");
    assert_ne!(book.name, "production");
    let again = fixture();
    assert_eq!(book.content_sha256, again.content_sha256);
    let bytes = canonical_bytes(&book).unwrap();
    assert!(bytes.starts_with(b"sxp-codebook-v1"));
    assert_eq!(sha256_hex(&bytes), hex_of(&book.content_sha256));
    assert_eq!(sha256_hex(&bytes), FIXTURE_HASH.trim());
    let alt = assemble_codebook("fixture-alt", "fixture", 1, book.formulas.clone()).unwrap();
    assert_ne!(alt.content_sha256, book.content_sha256);
}

#[test]
fn rejected_recipes_emit_nothing() {
    assert_eq!(generate_codebook("").unwrap_err(), Error::AbsentRecipe);
    assert_eq!(
        generate_codebook("no-such").unwrap_err(),
        Error::AbsentRecipe
    );
    assert_eq!(
        generate_codebook("fixture-alt").unwrap_err(),
        Error::AbsentRecipe
    );
    assert_eq!(
        generate_codebook("production").unwrap_err(),
        Error::UnresolvedProduction
    );
    assert_eq!(
        generate_codebook("benchmark").unwrap_err(),
        Error::UnresolvedProduction
    );
    let formulas = fixture().formulas;
    assert_eq!(
        assemble_codebook("production", "fixture", 1, formulas.clone()).unwrap_err(),
        Error::UnresolvedProduction
    );
    assert_eq!(
        assemble_codebook("fixture-v1", "benchmark", 1, formulas.clone()).unwrap_err(),
        Error::UnresolvedProduction
    );
    assert_eq!(
        assemble_codebook("", "fixture", 1, formulas.clone()).unwrap_err(),
        Error::AbsentRecipe
    );
    assert_eq!(
        assemble_codebook("fixture-v1", "fixture", 1, vec![BlockKind::Raw]).unwrap_err(),
        Error::Kernel(sxp_kernel::Error::InvalidParameter)
    );
}

#[test]
fn caps_and_duplicates() {
    assert_eq!(
        check_family_caps(2049, 0, 0).unwrap_err(),
        Error::CapExceeded
    );
    assert_eq!(
        check_family_caps(1, 1, 0).unwrap_err(),
        Error::MissingFamily
    );
    assert_eq!(
        check_family_caps(2048, 2048, 0).unwrap_err(),
        Error::MissingFamily
    );
    assert_eq!(
        check_family_caps(2048, 2048, 2048).unwrap_err(),
        Error::CapExceeded
    );
    assert_eq!(
        check_family_caps(2048, 2048, 1).unwrap_err(),
        Error::CapExceeded
    );
    assert!(check_family_caps(2048, 2047, 1).is_ok());

    let book = fixture();
    let mut duplicated = book.formulas.clone();
    duplicated.insert(1, book.formulas[0]);
    let assembled = assemble_codebook("fixture-v1", "fixture", 1, duplicated).unwrap();
    assert_eq!(assembled.formulas.len(), 3);
    assert_eq!(assembled.formulas, book.formulas);

    let narrow = BlockKind::polynomial(Polynomial::new(0, 8, &[0]).unwrap(), 8).unwrap();
    let wider = BlockKind::polynomial(Polynomial::new(0, 8, &[0]).unwrap(), 9).unwrap();
    let mut both = vec![narrow, wider];
    both.extend(book.formulas[1..].iter().copied());
    let kept = assemble_codebook("widths", "fixture", 1, both).unwrap();
    assert_eq!(kept.formulas.len(), 4);
}

#[test]
fn each_formula_wins_its_trace() {
    let book = fixture();
    let cases = [(0u16, 48u16), (1, 64), (2, 32)];
    for (index, len) in cases {
        let kind = book.formulas[usize::from(index)];
        let input = trace(kind, len);
        for threads in [0usize, 1, 2] {
            for mode in [SearchMode::Reference, SearchMode::HeaderPrune] {
                let selection = select_codebook(
                    &input,
                    &all_baselines(),
                    &book,
                    LengthPolicy::All,
                    mode,
                    threads,
                )
                .unwrap();
                assert!(!selection.whole_file_raw, "id {index} threads {threads}");
                assert_eq!(selection.choices.len(), 1);
                assert_eq!(selection.choices[0].id, 259 + index);
                let encoded = encode_codebook(
                    &input,
                    &all_baselines(),
                    &book,
                    LengthPolicy::All,
                    mode,
                    threads,
                )
                .unwrap();
                assert_eq!(&encoded[..2], &[0x8d, 0x01]);
                assert_eq!(encoded.len() as u64, selection.file_len);
                let mut decoded = Vec::new();
                read_blocked_file(encoded.as_slice(), &mut decoded).unwrap();
                assert_eq!(decoded, input);
            }
        }
    }
}

#[test]
fn ties_prefer_the_smaller_id_and_zeros_skip_formulas() {
    let book = fixture();
    let raw = SearchCandidate {
        id: 0,
        kind: BlockKind::Raw,
    };
    let narrow = BlockKind::polynomial(Polynomial::new(0, 8, &[0]).unwrap(), 1).unwrap();
    let formula = SearchCandidate {
        id: 259,
        kind: narrow,
    };
    let later = SearchCandidate {
        id: 260,
        kind: book.formulas[1],
    };
    let tie_calls = Cell::new(0u32);
    let tied = choose_candidate(&[raw, formula], SearchMode::HeaderPrune, |_candidate| {
        tie_calls.set(tie_calls.get() + 1);
        Ok(5)
    })
    .unwrap();
    assert_eq!(tied.1, 0);
    assert_eq!(tie_calls.get(), 2);
    let formula_wins = choose_candidate(&[raw, formula], SearchMode::Reference, |candidate| {
        Ok(if candidate.id == 0 { 5 } else { 4 })
    })
    .unwrap();
    assert_eq!(formula_wins.1, 259);
    let ordered = choose_candidate(&[later, formula], SearchMode::Reference, |_| Ok(4)).unwrap();
    assert_eq!(ordered.1, 259);

    let zeros = vec![0u8; 32];
    let mut candidates = Vec::new();
    for kind in all_baselines() {
        candidates.push(SearchCandidate {
            id: sxp_encoder::candidate_id(kind).unwrap(),
            kind,
        });
    }
    for (index, kind) in book.formulas.iter().copied().enumerate() {
        candidates.push(SearchCandidate {
            id: 259 + index as u16,
            kind,
        });
    }
    let pruned = span_cells(
        &zeros,
        NominalLength::N256,
        32,
        &candidates,
        SearchMode::HeaderPrune,
    )
    .unwrap();
    let reference = span_cells(
        &zeros,
        NominalLength::N256,
        32,
        &candidates,
        SearchMode::Reference,
    )
    .unwrap();
    assert!(
        reference
            .iter()
            .all(|(_, cell)| matches!(cell, CostCell::Evaluated(_)))
    );
    let mut incumbent = u64::MAX;
    for ((id, cell), (_, full)) in pruned.iter().zip(reference.iter()) {
        match cell {
            CostCell::Evaluated(cost) => incumbent = incumbent.min(*cost),
            CostCell::Skipped => {
                let CostCell::Evaluated(cost) = full else {
                    panic!("reference skipped {id}");
                };
                assert!(
                    *cost >= incumbent,
                    "id {id} cost {cost} incumbent {incumbent}"
                );
                assert!(*id >= 259);
            }
        }
    }
    assert!(
        pruned
            .iter()
            .any(|(_, cell)| matches!(cell, CostCell::Skipped))
    );
    let blocked = select_codebook(
        &zeros,
        &all_baselines(),
        &book,
        LengthPolicy::All,
        SearchMode::HeaderPrune,
        1,
    )
    .unwrap();
    let dense = select_codebook(
        &zeros,
        &all_baselines(),
        &book,
        LengthPolicy::All,
        SearchMode::Reference,
        1,
    )
    .unwrap();
    assert_eq!(blocked.key.steps, dense.key.steps);
    assert!(
        select_baselines(
            &zeros,
            &[book.formulas[0]],
            LengthPolicy::All,
            SearchMode::Reference,
            1
        )
        .is_err()
    );
    let mut tainted = book.clone();
    tainted.formulas[0] = BlockKind::Raw;
    assert_eq!(
        select_codebook(
            &zeros,
            &all_baselines(),
            &tainted,
            LengthPolicy::All,
            SearchMode::Reference,
            1
        )
        .unwrap_err(),
        Error::Kernel(sxp_kernel::Error::InvalidParameter)
    );
    let clash = choose_candidate(
        &[
            SearchCandidate {
                id: 259,
                kind: book.formulas[0],
            },
            SearchCandidate {
                id: 259,
                kind: book.formulas[1],
            },
        ],
        SearchMode::Reference,
        |_| Ok(4),
    );
    assert_eq!(clash.unwrap_err(), Error::NoCandidate);
}

#[test]
fn oracle_matches_the_three_traces() {
    let book = fixture();
    for (index, len) in [(0u16, 48u16), (1, 64), (2, 32)] {
        let input = trace(book.formulas[usize::from(index)], len);
        let (id, block_bytes) = oracle_span(&input, &book);
        let selection = select_codebook(
            &input,
            &all_baselines(),
            &book,
            LengthPolicy::All,
            SearchMode::Reference,
            1,
        )
        .unwrap();
        assert_eq!(selection.choices[0].id, id);
        assert_eq!(selection.key.block_bytes, block_bytes);
        assert_eq!(selection.key.block_count, 1);
        assert_eq!(selection.key.steps, vec![(0, id)]);
        let encoded = encode_codebook(
            &input,
            &all_baselines(),
            &book,
            LengthPolicy::All,
            SearchMode::Reference,
            1,
        )
        .unwrap();
        let kind = book.formulas[usize::from(id - 259)];
        let mut file = Vec::new();
        write_blocked_file(
            input.len() as u64,
            &[BlockPlan {
                kind,
                nominal: NominalLength::N256,
                bytes: &input,
            }],
            &mut file,
        )
        .unwrap();
        assert_eq!(encoded, file);
        assert_eq!(encoded.len() as u64, selection.file_len);
    }
}

fn oracle_span(input: &[u8], book: &sxp_encoder::Codebook) -> (u16, u64) {
    let remaining = input.len() as u64;
    let mut best: Option<(u64, u16)> = None;
    for kind in all_baselines() {
        consider(
            &mut best,
            kind,
            sxp_encoder::candidate_id(kind).unwrap(),
            input,
            remaining,
        );
    }
    for (index, kind) in book.formulas.iter().copied().enumerate() {
        consider(&mut best, kind, 259 + index as u16, input, remaining);
    }
    let (bytes, id) = best.unwrap();
    (id, bytes)
}

fn consider(best: &mut Option<(u64, u16)>, kind: BlockKind, id: u16, input: &[u8], remaining: u64) {
    let encoded = encode_block(kind, NominalLength::N256, remaining, input).unwrap();
    let bytes = encoded.len() as u64;
    let replace = match best {
        None => true,
        Some((best_bytes, best_id)) => (bytes, id) < (*best_bytes, *best_id),
    };
    if replace {
        *best = Some((bytes, id));
    }
}

fn hex_of(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
