//! Story 1.13. Exploratory report, verdict table, tuning, and the budget reducer.

use sxp_bench::{
    BudgetPlan, ClassScore, Label, Meta, Row, SidecarChoice, TuneRow, assert_same_evaluation,
    benchmark_manifest, classify, lower_cap, reduce_budget, report_rows, sizes_hold, tune,
};

fn meta(threads: usize, sidecar: SidecarChoice) -> Meta<'static> {
    Meta {
        threads,
        sidecar,
        host: "fixture",
        start: "1970-01-01T00:00:00Z",
        end: "1970-01-01T00:00:01Z",
        elapsed_seconds: 1,
    }
}

fn gate() -> &'static str {
    r#"{"corpus_gate_ready":false,"scope":"corpus","unresolved":[{"field":"k.status","reason":"proposed"}]}"#
}

fn row(id: &str, class: &str, role: &str, counts: bool, transform: &str, bytes: Vec<u8>) -> Row {
    Row {
        id: id.to_string(),
        class: class.to_string(),
        role: role.to_string(),
        counts_toward_sm1: counts,
        transform: transform.to_string(),
        bytes,
    }
}

#[test]
fn verdict_table_matches_the_policy() {
    let text = |wins, n, full, base| ClassScore {
        class: "text".to_string(),
        evaluation_inputs: n,
        wins,
        full_bytes: full,
        baselines_only_bytes: base,
    };
    let pcm = text;
    let exploratory = classify(false, &[text(3, 3, 90, 100), pcm(3, 3, 90, 100)]);
    assert!(exploratory.exploratory);
    assert!(exploratory.label.is_none());
    assert!(!exploratory.opens_pcm_experiments);

    let one = classify(
        true,
        &[
            ClassScore {
                class: "text".to_string(),
                evaluation_inputs: 3,
                wins: 2,
                full_bytes: 90,
                baselines_only_bytes: 100,
            },
            ClassScore {
                class: "pcm".to_string(),
                evaluation_inputs: 3,
                wins: 0,
                full_bytes: 100,
                baselines_only_bytes: 100,
            },
        ],
    );
    assert_eq!(one.label, Some(Label::Negative));
    assert!(one.opens_pcm_experiments);

    let marginal = classify(
        true,
        &[
            ClassScore {
                class: "text".to_string(),
                evaluation_inputs: 4,
                wins: 2,
                full_bytes: 100,
                baselines_only_bytes: 101,
            },
            ClassScore {
                class: "pcm".to_string(),
                evaluation_inputs: 3,
                wins: 2,
                full_bytes: 90,
                baselines_only_bytes: 100,
            },
        ],
    );
    assert_eq!(marginal.label, Some(Label::Marginal));

    let positive = classify(
        true,
        &[
            ClassScore {
                class: "text".to_string(),
                evaluation_inputs: 3,
                wins: 2,
                full_bytes: 90,
                baselines_only_bytes: 100,
            },
            ClassScore {
                class: "executable".to_string(),
                evaluation_inputs: 3,
                wins: 2,
                full_bytes: 90,
                baselines_only_bytes: 100,
            },
        ],
    );
    assert_eq!(positive.label, Some(Label::Positive));
    assert_eq!(positive.growth_percent, 1);
    assert_eq!(positive.gain_denominator, "baselines-only class sum");

    let few = classify(
        true,
        &[ClassScore {
            class: "text".to_string(),
            evaluation_inputs: 3,
            wins: 3,
            full_bytes: 1,
            baselines_only_bytes: 2,
        }],
    );
    assert_eq!(few.label, Some(Label::Inconclusive));
}

#[test]
fn selector_defect_is_not_a_verdict() {
    assert_eq!(
        sizes_hold(5, 4, 4, 4, 4, 4),
        Err(sxp_bench::Error::SelectorDefect)
    );
    assert!(sizes_hold(4, 4, 4, 5, 6, 7).is_ok());
}

#[test]
fn budget_shrinks_tuning_before_the_cap() {
    let plan = BudgetPlan {
        tuning_ids: vec!["t".to_string()],
        evaluation_ids: vec!["e".to_string()],
        cap: 8,
    };
    assert_eq!(reduce_budget(false, &plan), plan);
    let shrunk = reduce_budget(true, &plan);
    assert!(shrunk.tuning_ids.is_empty());
    assert_eq!(shrunk.evaluation_ids, plan.evaluation_ids);
    assert_eq!(shrunk.cap, 8);
    let lowered = lower_cap(&shrunk);
    assert_eq!(lowered.cap, 4);
    assert!(assert_same_evaluation(&plan.evaluation_ids, &lowered.evaluation_ids).is_ok());
    assert!(
        assert_same_evaluation(
            &["b".to_string(), "a".to_string()],
            &["a".to_string(), "b".to_string()]
        )
        .is_ok()
    );
    assert_eq!(
        assert_same_evaluation(&plan.evaluation_ids, &["other".to_string()]),
        Err(sxp_bench::Error::NewExperiment)
    );
}

#[test]
fn empty_and_one_byte_reports_are_exploratory_and_repeatable() {
    let rows = vec![
        row(
            "empty",
            "synthetic",
            "synthetic",
            false,
            "kernel-predict",
            vec![],
        ),
        row("one", "text", "evaluation", true, "identity", vec![0x11]),
    ];
    let first = report_rows(
        &rows,
        &["missing-row".to_string()],
        b"manifest",
        gate(),
        &meta(0, SidecarChoice::Auto),
    )
    .expect("report");
    let second = report_rows(
        &rows,
        &["missing-row".to_string()],
        b"manifest",
        gate(),
        &meta(0, SidecarChoice::Auto),
    )
    .expect("report");
    assert_eq!(first.report, second.report);
    assert_eq!(first.annex, second.annex);
    assert_eq!(first.sidecars, second.sidecars);
    assert!(
        first
            .report
            .contains("\"status\":\"exploratory/unreportable\"")
    );
    assert!(first.report.contains("\"verdict\":null"));
    assert!(
        first
            .report
            .contains("\"model_constants\":\"provisional-strawman\"")
    );
    assert!(first.report.contains("\"d07\":\"open\""));
    assert!(first.report.contains("\"d09\":\"open\""));
    assert!(first.report.contains("k.status"));
    assert!(first.report.contains("\"overlap\""));
    assert!(first.timing.starts_with("host=fixture\n"));
    assert!(first.timing.contains("within_budget=true\n"));
    assert!(!first.report.contains("fixture\n"));
    assert_eq!(first.sidecars.len(), 2);
    assert_eq!(&first.sidecars[0].1[..4], b"SXPC");
    assert!(
        first
            .sidecars
            .iter()
            .any(|(_, bytes)| bytes.windows(2).any(|pair| pair == [0xFF, 0xFF]))
    );

    let dense = report_rows(
        &rows,
        &[],
        b"manifest",
        gate(),
        &meta(0, SidecarChoice::Dense),
    )
    .expect("dense");
    assert!(dense.report.contains("\"search_mode\":\"reference\""));
    assert!(
        dense
            .sidecars
            .iter()
            .all(|(_, bytes)| !bytes.windows(2).any(|pair| pair == [0xFF, 0xFF]))
    );

    let threaded = report_rows(
        &rows,
        &["missing-row".to_string()],
        b"manifest",
        gate(),
        &meta(2, SidecarChoice::Auto),
    )
    .expect("threads");
    assert_eq!(threaded.sidecars, first.sidecars);
    assert_eq!(
        threaded.report.replace("\"threads\":2", "\"threads\":0"),
        first.report
    );
    accounts_match_file_len(&first.report);
    let empty = sidecar_named(&first.sidecars, "empty.sxpc");
    assert_eq!(span_count(&empty), 0);
    reconcile_sidecar(&sidecar_named(&first.sidecars, "one.sxpc"), &[0x11]);
}

#[test]
fn tune_rejects_evaluation_rows_and_sorts_ties() {
    let leaked = [TuneRow {
        role: "evaluation",
        bytes: &[1, 2, 3, 4],
    }];
    assert_eq!(tune(&leaked), Err(sxp_bench::Error::EvaluationLeak));
    let rows = [TuneRow {
        role: "tuning",
        bytes: &[0u8; 32],
    }];
    let tuning = tune(&rows).expect("tune");
    assert_eq!(tuning.points.len(), 11);
    assert_eq!(tuning.winner, tuning.points[0]);
    assert_eq!(tuning.width_grid, "not-run");
    assert_eq!(tuning.width_reason, "d09-open");
    let mut scores = tuning.points.clone();
    sxp_bench::sort_points(&mut scores);
    assert_eq!(scores, tuning.points);
    let json = sxp_bench::tuning_json(&tuning).expect("tuning.json");
    assert!(json.ends_with('\n'));
    let value: serde_json::Value = serde_json::from_str(json.trim_end()).expect("json");
    let points = value["points"].as_array().expect("points");
    assert_eq!(points.len(), 11);
    assert_eq!(value["winner"], points[0]);
    assert_eq!(value["width_grid"].as_str(), Some("not-run"));
    assert_eq!(value["width_reason"].as_str(), Some("d09-open"));
    assert!(json.contains("\"delta\":\"1\""));
    assert!(json.contains("\"delta\":\"0.5\""));
}

#[test]
fn synthetics_round_trip_and_stay_exploratory() {
    let manifest = "\
[[input]]
id = \"synthetic-polynomial\"
class = \"synthetic\"
role = \"synthetic\"
counts_toward_sm1 = false
transform = \"kernel-predict\"
derived_sha256 = \"7364c598b868aae6247ed6554b1fc9e2e251220f237c913f0ac2720a5b2b5e92\"

[[input]]
id = \"synthetic-trigonometric\"
class = \"synthetic\"
role = \"synthetic\"
counts_toward_sm1 = false
transform = \"kernel-predict\"
derived_sha256 = \"1a2fa720c8ec350f1519e98632eae12add852e2d73c4ce931fd1d91129e93fcb\"

[[input]]
id = \"synthetic-recurrence\"
class = \"synthetic\"
role = \"synthetic\"
counts_toward_sm1 = false
transform = \"kernel-predict\"
derived_sha256 = \"90b1408a5ea7b47f68e2ed160763d5f1d196a4533e74ad67a357005466a1c291\"
";
    let artifacts =
        benchmark_manifest(manifest, gate(), &meta(0, SidecarChoice::Auto)).expect("synthetics");
    assert!(
        artifacts
            .report
            .contains("\"status\":\"exploratory/unreportable\"")
    );
    assert!(artifacts.report.contains("\"verdict\":null"));
    assert_eq!(artifacts.sidecars.len(), 3);
    accounts_match_file_len(&artifacts.report);
    let value: serde_json::Value = serde_json::from_str(artifacts.report.trim_end()).expect("json");
    let inputs = value["inputs"].as_array().expect("inputs");
    assert_eq!(inputs.len(), 3);
    for input in inputs {
        assert_eq!(input["input_bytes"].as_u64(), Some(4096));
        assert_eq!(input["class"].as_str(), Some("synthetic"));
    }
    assert!(value["classes"].as_array().expect("classes").is_empty());
    assert_eq!(value["synthetic"].as_array().expect("synthetic").len(), 1);
}

#[test]
fn an_unclassified_row_is_listed_and_tuning_is_rejected() {
    let rows = vec![row("note", "notes", "probe", false, "identity", vec![1])];
    let artifacts = report_rows(
        &rows,
        &[],
        b"manifest",
        gate(),
        &meta(0, SidecarChoice::Off),
    )
    .expect("probe");
    let value: serde_json::Value = serde_json::from_str(artifacts.report.trim_end()).expect("json");
    assert_eq!(value["inputs"].as_array().expect("inputs").len(), 1);
    assert!(value["classes"].as_array().expect("classes").is_empty());
    assert!(value["synthetic"].as_array().expect("synthetic").is_empty());
    assert!(
        value["missing_inputs"]
            .as_array()
            .expect("missing")
            .is_empty()
    );
    let tuning = vec![row("t", "text", "tuning", false, "identity", vec![1])];
    assert_eq!(
        report_rows(
            &tuning,
            &[],
            b"manifest",
            gate(),
            &meta(0, SidecarChoice::Off)
        )
        .err(),
        Some(sxp_bench::Error::BadManifest)
    );
}

#[test]
fn duplicate_and_mislabeled_rows_are_rejected_before_search() {
    let dup = vec![
        row("a", "notes", "probe", false, "identity", vec![1]),
        row("a", "notes", "probe", false, "identity", vec![1]),
    ];
    assert_eq!(
        report_rows(&dup, &[], b"m", gate(), &meta(0, SidecarChoice::Off)).err(),
        Some(sxp_bench::Error::BadManifest)
    );
    let labeled = vec![row(
        "synthetic-polynomial",
        "text",
        "evaluation",
        true,
        "identity",
        vec![1],
    )];
    assert_eq!(
        report_rows(&labeled, &[], b"m", gate(), &meta(0, SidecarChoice::Off)).err(),
        Some(sxp_bench::Error::BadManifest)
    );
}

#[test]
fn manifest_without_synthetics_is_rejected() {
    let manifest = "[[input]]\nid = \"x\"\nclass = \"text\"\nrole = \"evaluation\"\ncounts_toward_sm1 = true\ntransform = \"identity\"\nderived_sha256 = \"\"\n";
    let err = benchmark_manifest(manifest, gate(), &meta(0, SidecarChoice::Off));
    assert_eq!(err.err(), Some(sxp_bench::Error::BadManifest));
}

fn accounts_match_file_len(report: &str) {
    let value: serde_json::Value = serde_json::from_str(report.trim_end()).expect("json");
    let inputs = value["inputs"].as_array().expect("inputs");
    for input in inputs {
        let arms = input["arms"].as_object().expect("arms");
        for arm in arms.values() {
            let file_len = arm["file_len"].as_u64().expect("file_len");
            let account = arm["account"].as_object().expect("account");
            let mut sum = 0u64;
            for key in [
                "file_header",
                "block_headers",
                "parameters",
                "alignment_padding",
                "raw_payload",
                "coder_payload",
                "coder_flush",
                "crc",
            ] {
                sum += account[key].as_u64().expect(key);
            }
            assert_eq!(sum, file_len);
        }
        if input["input_bytes"].as_u64() == Some(0) {
            for arm in arms.values() {
                assert_eq!(arm["file_len"].as_u64(), Some(7));
                let account = &arm["account"];
                assert_eq!(account["raw_payload"].as_u64(), Some(0));
                assert_eq!(account["coder_payload"].as_u64(), Some(0));
                assert_eq!(account["coder_flush"].as_u64(), Some(0));
                assert_eq!(account["crc"].as_u64(), Some(4));
            }
        }
    }
}

fn sidecar_named(sidecars: &[(String, Vec<u8>)], name: &str) -> Vec<u8> {
    sidecars
        .iter()
        .find(|(id, _)| id == name)
        .map(|(_, bytes)| bytes.clone())
        .expect(name)
}

fn span_count(bytes: &[u8]) -> u32 {
    let count = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    let at = 10 + count * 2;
    u32::from_le_bytes(bytes[at..at + 4].try_into().expect("span count"))
}

fn reconcile_sidecar(bytes: &[u8], input: &[u8]) {
    assert_eq!(&bytes[..4], b"SXPC");
    let count = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    let mut cursor = 10usize;
    let mut ids = Vec::with_capacity(count);
    for _ in 0..count {
        ids.push(u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]));
        cursor += 2;
    }
    let spans = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().expect("spans")) as usize;
    cursor += 4;
    let book = sxp_encoder::generate_codebook("fixture-v1").expect("codebook");
    let baselines = sxp_encoder::all_baselines();
    for _ in 0..spans {
        let offset = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().expect("offset"));
        cursor += 8;
        let code = bytes[cursor];
        cursor += 1;
        let nominal = match code {
            0 => sxp_kernel::NominalLength::N256,
            1 => sxp_kernel::NominalLength::N1024,
            2 => sxp_kernel::NominalLength::N2048,
            _ => panic!("length code"),
        };
        let start = usize::try_from(offset).expect("offset");
        let remaining = u64::try_from(input.len() - start).expect("remaining");
        let actual = usize::try_from(sxp_kernel::actual_len(nominal, remaining)).expect("actual");
        let block = &input[start..start + actual];
        for id in &ids {
            let cost = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
            cursor += 2;
            if cost == 0xFFFF {
                continue;
            }
            let kind = kind_for(*id, &baselines, &book.formulas);
            let encoded =
                sxp_kernel::encode_block(kind, nominal, remaining, block).expect("encode");
            assert_eq!(encoded.len(), usize::from(cost));
        }
    }
    assert_eq!(cursor, bytes.len());
}

fn kind_for(
    id: u16,
    baselines: &[sxp_kernel::BlockKind],
    formulas: &[sxp_kernel::BlockKind],
) -> sxp_kernel::BlockKind {
    if let Some(kind) = baselines
        .iter()
        .copied()
        .find(|kind| sxp_encoder::candidate_id(*kind) == Some(id))
    {
        return kind;
    }
    let index = usize::from(id - 259);
    formulas[index]
}
