//! Exploratory benchmark report. The published verdict stays null while gates are open.

use std::io::Cursor;

use sxp_encoder::{
    Codebook, LengthPolicy, SearchCandidate, SearchMode, all_baselines, candidate_id,
    generate_codebook, search_arm,
};
use sxp_kernel::{
    BlockKind, BlockTrace, ByteAccount, DecisionTrace, ModelConfig, NominalLength, read_file,
    trace_block, write_raw_accounted,
};

use crate::error::Error;
use crate::sidecar::{Coverage, sidecar_bytes};
use crate::synthetic::{polynomial_signal, recurrence_signal, sha256_hex, trigonometric_signal};

const SPEC_SEED: &str = "f4bea1e5b729db1df957563fd3e8e5bec70c0ab0a382f0f0afc7f9e82196e12e";
const CODEBOOK_HASH: &str = "a8f53ca8a7799035822e1fe6bf27962136b6af9ccf9e75b8785562d638cfef9b";
const FOUR_MIB: u64 = 4_194_304;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidecarChoice {
    Auto,
    Off,
    On,
    Dense,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub class: String,
    pub role: String,
    pub counts_toward_sm1: bool,
    pub transform: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Meta<'a> {
    pub threads: usize,
    pub sidecar: SidecarChoice,
    pub host: &'a str,
    pub start: &'a str,
    pub end: &'a str,
    pub elapsed_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Artifacts {
    pub report: String,
    pub annex: String,
    pub timing: String,
    pub sidecars: Vec<(String, Vec<u8>)>,
}

pub fn benchmark_manifest(manifest: &str, gate: &str, meta: &Meta<'_>) -> Result<Artifacts, Error> {
    let listed = parse_manifest(manifest)?;
    let mut rows = Vec::new();
    let mut missing = Vec::new();
    let mut seen_poly = false;
    let mut seen_trig = false;
    let mut seen_rec = false;
    for row in listed {
        legal_id(&row.id)?;
        match row.id.as_str() {
            "synthetic-polynomial" => {
                seen_poly = true;
                rows.push(generated(row, polynomial_signal().map_err(Error::kernel)?)?);
            }
            "synthetic-trigonometric" => {
                seen_trig = true;
                rows.push(generated(
                    row,
                    trigonometric_signal().map_err(Error::kernel)?,
                )?);
            }
            "synthetic-recurrence" => {
                seen_rec = true;
                rows.push(generated(row, recurrence_signal().map_err(Error::kernel)?)?);
            }
            _ => missing.push(row.id),
        }
    }
    if !seen_poly || !seen_trig || !seen_rec {
        return Err(Error::BadManifest);
    }
    missing.sort();
    report_rows(&rows, &missing, manifest.as_bytes(), gate, meta)
}

pub fn report_rows(
    rows: &[Row],
    missing: &[String],
    manifest: &[u8],
    gate: &str,
    meta: &Meta<'_>,
) -> Result<Artifacts, Error> {
    reject_bad_rows(rows, missing)?;
    let gate_view = parse_gate(gate)?;
    let book = generate_codebook("fixture-v1").map_err(Error::kernel)?;
    let hash = hex_bytes(&book.content_sha256)?;
    if hash != CODEBOOK_HASH {
        return Err(Error::HashMismatch);
    }
    let mode = if meta.sidecar == SidecarChoice::Dense {
        SearchMode::Reference
    } else {
        SearchMode::HeaderPrune
    };
    let model = ModelConfig::strawman().map_err(Error::kernel)?;
    let forced_constant = candidates_of(&constant_kinds())?;
    let forced_previous = candidates_of(&[BlockKind::Raw, BlockKind::PreviousByte])?;
    let forced_direct = candidates_of(&[BlockKind::Raw, BlockKind::Direct])?;
    let baselines = candidates_of(&all_baselines())?;
    let full = full_candidates(&book)?;
    let full_ids: Vec<u16> = full.iter().map(|candidate| candidate.id).collect();

    let mut inputs = Vec::new();
    let mut annex_rows = Vec::new();
    let mut sidecars = Vec::new();
    let mut grouped: Vec<Held> = Vec::new();

    for row in rows {
        let n = u64::try_from(row.bytes.len()).map_err(|_| Error::CostOverflow)?;
        let raw = raw_arm(&row.bytes).map_err(Error::kernel)?;
        let constant = one_arm(&row.bytes, &forced_constant, mode, meta.threads, &model)?;
        let previous = one_arm(&row.bytes, &forced_previous, mode, meta.threads, &model)?;
        let direct = one_arm(&row.bytes, &forced_direct, mode, meta.threads, &model)?;
        let base = one_arm(&row.bytes, &baselines, mode, meta.threads, &model)?;
        let full_arm = one_arm(&row.bytes, &full, mode, meta.threads, &model)?;
        for file in [
            &raw.file,
            &constant.file,
            &previous.file,
            &direct.file,
            &base.file,
            &full_arm.file,
        ] {
            round_trip(file, &row.bytes)?;
        }
        sizes_hold(
            full_arm.file_len,
            base.file_len,
            constant.file_len,
            previous.file_len,
            direct.file_len,
            raw.file_len,
        )?;
        let best = best_forced(constant.file_len, previous.file_len, direct.file_len);
        let gain = i64::try_from(base.file_len).map_err(|_| Error::CostOverflow)?
            - i64::try_from(full_arm.file_len).map_err(|_| Error::CostOverflow)?;
        let coverage = coverage_for(n, meta.sidecar);
        let sidecar = match coverage {
            None => SidecarJson {
                present: false,
                coverage: "off".to_string(),
                name: String::new(),
            },
            Some(kind) => {
                let name = format!("{}.sxpc", row.id);
                let bytes = sidecar_bytes(
                    kind,
                    mode == SearchMode::Reference,
                    &full_ids,
                    &full_arm.spans,
                )?;
                sidecars.push((name.clone(), bytes));
                SidecarJson {
                    present: true,
                    coverage: kind.name().to_string(),
                    name,
                }
            }
        };
        let annex = annex_of(&row.id, &row.bytes, &full_arm)?;
        annex_rows.push(annex);
        inputs.push(InputJson {
            id: row.id.clone(),
            class: row.class.clone(),
            role: row.role.clone(),
            counts_toward_sm1: row.counts_toward_sm1,
            input_bytes: n,
            input_sha256: sha256_hex(&row.bytes).map_err(Error::kernel)?,
            arms: ArmsJson {
                whole_file_raw: arm_json(&raw),
                forced_constant: arm_json(&constant),
                forced_previous: arm_json(&previous),
                forced_direct: arm_json(&direct),
                baselines_only: arm_json(&base),
                full: arm_json(&full_arm),
            },
            best_forced: BestForced {
                arm: best.0.to_string(),
                file_len: best.1,
            },
            gain_bytes: gain,
            win: full_arm.file_len < base.file_len,
            margin_bytes: gain,
            sidecar,
        });
        grouped.push(Held {
            class: row.class.clone(),
            role: row.role.clone(),
            counts: row.counts_toward_sm1,
            transform: row.transform.clone(),
            full_len: full_arm.file_len,
            base_len: base.file_len,
            win: full_arm.file_len < base.file_len,
            families: full_arm.families,
            zero_num: full_arm.zero_residual,
            zero_den: full_arm.residual_bytes,
        });
    }

    let mut missing_gates = gate_view.unresolved.clone();
    for extra in ["d07", "d09", "overlap"] {
        if !missing_gates.iter().any(|gate| gate == extra) {
            missing_gates.push(extra.to_string());
        }
    }
    let reportable = gate_view.corpus_gate_ready && missing_gates.is_empty();
    let status = if reportable {
        "reportable".to_string()
    } else {
        "exploratory/unreportable".to_string()
    };
    let report = Report {
        report_version: 1,
        status,
        opens_pcm_experiments: false,
        experiment_id: "exploratory-fixture-v1".to_string(),
        tool: "sxp-cli".to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        format_version: 1,
        layout: "d02-lsb-first".to_string(),
        spec_seed_sha256: SPEC_SEED.to_string(),
        model_constants: "provisional-strawman".to_string(),
        decisions: Decisions::pinned(),
        thresholds: Thresholds {
            growth_percent: 1,
            majority: "wins * 2 > evaluation_inputs".to_string(),
            gain_denominator: "baselines-only class sum".to_string(),
        },
        block_lengths: vec![256, 1024, 2048],
        codebook: CodebookJson {
            name: book.name,
            version: book.version,
            label: book.label,
            sha256: hash,
        },
        manifest_sha256: sha256_hex(manifest).map_err(Error::kernel)?,
        gate: gate_view,
        overlap: Overlap {
            k: 1024,
            k_status: "proposed".to_string(),
            result: "not-checked".to_string(),
        },
        missing_gates,
        search_mode: if mode == SearchMode::Reference {
            "reference".to_string()
        } else {
            "header-prune".to_string()
        },
        threads: u64::try_from(meta.threads).map_err(|_| Error::CostOverflow)?,
        tuning: TuningJson {
            ran: false,
            reason: Some("no-tuning-bytes".to_string()),
            width_grid: "not-run".to_string(),
            width_reason: "d09-open".to_string(),
        },
        missing_inputs: missing.to_vec(),
        inputs,
        classes: summaries(&grouped, Bucket::Class)?,
        synthetic: summaries(&grouped, Bucket::Synthetic)?,
        compressed_control: summaries(&grouped, Bucket::Compressed)?,
        non_counted_pcm: summaries(&grouped, Bucket::RawPcm)?,
    };
    let report_text = render_report(&report)?;
    let annex = render_annex(&annex_rows)?;
    Ok(Artifacts {
        report: report_text,
        annex,
        timing: timing_log(meta),
        sidecars,
    })
}

fn generated(row: ManifestRow, bytes: Vec<u8>) -> Result<Row, Error> {
    if !row.derived.is_empty() {
        let hash = sha256_hex(&bytes).map_err(Error::kernel)?;
        if hash != row.derived {
            return Err(Error::HashMismatch);
        }
    }
    Ok(Row {
        id: row.id,
        class: row.class,
        role: row.role,
        counts_toward_sm1: row.counts,
        transform: row.transform,
        bytes,
    })
}

fn raw_arm(bytes: &[u8]) -> Result<Done, sxp_kernel::Error> {
    let n = u64::try_from(bytes.len()).map_err(|_| sxp_kernel::Error::LengthOutOfRange)?;
    let accounted = write_raw_accounted(n, bytes)?;
    let file_len =
        u64::try_from(accounted.bytes.len()).map_err(|_| sxp_kernel::Error::BlockTooLarge)?;
    Ok(Done {
        file: accounted.bytes,
        file_len,
        account: accounted.account,
        work: Work {
            input_bytes: n,
            selected_model_decisions: 0,
            selected_coder_steps: 0,
            search_encode_block_calls: 0,
            search_skipped: 0,
        },
        spans: Vec::new(),
        families: Families::default(),
        zero_residual: 0,
        residual_bytes: 0,
        whole_file_raw: true,
        choices_note: Vec::new(),
    })
}

struct Done {
    file: Vec<u8>,
    file_len: u64,
    account: ByteAccount,
    work: Work,
    spans: Vec<sxp_encoder::SpanRecord>,
    families: Families,
    zero_residual: u64,
    residual_bytes: u64,
    whole_file_raw: bool,
    choices_note: Vec<ChoiceNote>,
}

struct ChoiceNote {
    kind: BlockKind,
    input_bytes: u64,
    nominal: NominalLength,
}

fn one_arm(
    bytes: &[u8],
    candidates: &[SearchCandidate],
    mode: SearchMode,
    threads: usize,
    model: &ModelConfig,
) -> Result<Done, Error> {
    let arm = search_arm(bytes, candidates, LengthPolicy::All, mode, threads, model)
        .map_err(Error::kernel)?;
    let n = u64::try_from(bytes.len()).map_err(|_| Error::CostOverflow)?;
    let mut families = Families::default();
    let mut zero_residual = 0u64;
    let mut residual_bytes = 0u64;
    let mut notes = Vec::new();
    if !arm.selection.whole_file_raw {
        let mut offset = 0u64;
        for choice in &arm.selection.choices {
            let remaining = n.checked_sub(offset).ok_or(Error::CostOverflow)?;
            let actual = sxp_kernel::actual_len(choice.nominal, remaining);
            families.add(choice.kind, choice.block_len);
            if counts_residual(choice.kind) {
                let slice = bytes
                    .get(usize_from(offset)?..usize_from(offset.saturating_add(actual))?)
                    .ok_or(Error::CostOverflow)?;
                let traced = trace_block(choice.kind, choice.nominal, remaining, slice, model)
                    .map_err(Error::kernel)?;
                residual_bytes = residual_bytes.saturating_add(actual);
                zero_residual = zero_residual.saturating_add(zero_symbols(&traced));
            }
            notes.push(ChoiceNote {
                kind: choice.kind,
                input_bytes: actual,
                nominal: choice.nominal,
            });
            offset = offset.saturating_add(actual);
        }
    }
    let file_len = u64::try_from(arm.file.len()).map_err(|_| Error::CostOverflow)?;
    Ok(Done {
        file: arm.file,
        file_len,
        account: arm.account,
        work: Work {
            input_bytes: n,
            selected_model_decisions: arm.selected_model_decisions,
            selected_coder_steps: arm.selected_coder_steps,
            search_encode_block_calls: arm.search_evaluated,
            search_skipped: arm.search_skipped,
        },
        spans: arm.spans,
        families,
        zero_residual,
        residual_bytes,
        whole_file_raw: arm.selection.whole_file_raw,
        choices_note: notes,
    })
}

fn annex_of(id: &str, bytes: &[u8], full: &Done) -> Result<AnnexRow, Error> {
    if full.whole_file_raw {
        return Ok(AnnexRow {
            id: id.to_string(),
            ideal_microbits: 0,
            entropy_microbits: 0,
            overhead_microbits: 0,
            flush_bytes: full.account.coder_flush,
        });
    }
    let model = ModelConfig::strawman().map_err(Error::kernel)?;
    let mut ideal = 0.0f64;
    let mut entropy = 0.0f64;
    let n = u64::try_from(bytes.len()).map_err(|_| Error::CostOverflow)?;
    let mut offset = 0u64;
    for note in &full.choices_note {
        let remaining = n.checked_sub(offset).ok_or(Error::CostOverflow)?;
        if note.kind == BlockKind::Raw {
            offset = offset.saturating_add(note.input_bytes);
            continue;
        }
        let slice = bytes
            .get(usize_from(offset)?..usize_from(offset.saturating_add(note.input_bytes))?)
            .ok_or(Error::CostOverflow)?;
        let traced = trace_block(note.kind, note.nominal, remaining, slice, &model)
            .map_err(Error::kernel)?;
        ideal += ideal_bits(&traced.traces)?;
        entropy += entropy_bits(&traced.symbols)?;
        offset = offset.saturating_add(note.input_bytes);
    }
    let ideal_micro = round_microbits(ideal)?;
    let entropy_micro = round_microbits(entropy)?;
    Ok(AnnexRow {
        id: id.to_string(),
        ideal_microbits: ideal_micro,
        entropy_microbits: entropy_micro,
        overhead_microbits: ideal_micro
            .checked_sub(entropy_micro)
            .ok_or(Error::CostOverflow)?,
        flush_bytes: full.account.coder_flush,
    })
}

fn ideal_bits(traces: &[DecisionTrace]) -> Result<f64, Error> {
    let mut sum = 0.0f64;
    for trace in traces {
        let freq = if trace.bit == 0 {
            trace.freq0
        } else {
            trace.freq1
        };
        if freq == 0 {
            return Err(Error::Kernel("InvalidParameter".to_string()));
        }
        sum += -(f64::from(freq) / 4096.0).log2();
    }
    Ok(sum)
}

fn entropy_bits(symbols: &[u8]) -> Result<f64, Error> {
    if symbols.is_empty() {
        return Ok(0.0);
    }
    let mut counts = [0u64; 256];
    for symbol in symbols {
        let slot = counts
            .get_mut(usize::from(*symbol))
            .ok_or(Error::Kernel("InvalidParameter".to_string()))?;
        *slot = slot.saturating_add(1);
    }
    let n = symbols.len() as f64;
    let mut bits = 0.0f64;
    for count in counts {
        if count == 0 {
            continue;
        }
        let c = count as f64;
        bits -= c * (c / n).log2();
    }
    Ok(bits)
}

fn round_microbits(bits: f64) -> Result<i64, Error> {
    if !bits.is_finite() {
        return Err(Error::Kernel("InvalidParameter".to_string()));
    }
    let scaled = bits * 1_000_000.0;
    let rounded = if scaled >= 0.0 {
        (scaled + 0.5).floor()
    } else {
        (scaled - 0.5).ceil()
    };
    if !rounded.is_finite() {
        return Err(Error::Kernel("InvalidParameter".to_string()));
    }
    Ok(rounded as i64)
}

fn zero_symbols(traced: &BlockTrace) -> u64 {
    let mut zeros = 0u64;
    for symbol in &traced.symbols {
        if *symbol == 0 {
            zeros = zeros.saturating_add(1);
        }
    }
    zeros
}

fn counts_residual(kind: BlockKind) -> bool {
    matches!(
        kind,
        BlockKind::PreviousByte
            | BlockKind::Constant(_)
            | BlockKind::Polynomial { .. }
            | BlockKind::Trigonometric { .. }
            | BlockKind::Recurrence { .. }
    )
}

fn usize_from(value: u64) -> Result<usize, Error> {
    usize::try_from(value).map_err(|_| Error::CostOverflow)
}

fn round_trip(file: &[u8], source: &[u8]) -> Result<(), Error> {
    let mut out = Vec::new();
    if read_file(Cursor::new(file), &mut out).is_err() || out != source {
        return Err(Error::RoundTripMismatch);
    }
    Ok(())
}

pub fn sizes_hold(
    full: u64,
    baselines: u64,
    constant: u64,
    previous: u64,
    direct: u64,
    raw: u64,
) -> Result<(), Error> {
    let forced = constant.min(previous).min(direct);
    if full <= baselines && baselines <= forced && forced <= raw {
        Ok(())
    } else {
        Err(Error::SelectorDefect)
    }
}

fn best_forced(constant: u64, previous: u64, direct: u64) -> (&'static str, u64) {
    let min = constant.min(previous).min(direct);
    if constant == min {
        ("forced_constant", constant)
    } else if direct == min {
        ("forced_direct", direct)
    } else {
        ("forced_previous", previous)
    }
}

fn coverage_for(n: u64, choice: SidecarChoice) -> Option<Coverage> {
    match choice {
        SidecarChoice::Off => None,
        SidecarChoice::Dense => Some(Coverage::Dense),
        SidecarChoice::On if n > FOUR_MIB => Some(Coverage::OptIn),
        SidecarChoice::On => Some(Coverage::Default),
        SidecarChoice::Auto if n <= FOUR_MIB => Some(Coverage::Default),
        SidecarChoice::Auto => None,
    }
}

fn arm_json(done: &Done) -> ArmJson {
    ArmJson {
        file_len: done.file_len,
        account: AccountJson::from(done.account),
        work: done.work.clone(),
    }
}

fn timing_log(meta: &Meta<'_>) -> String {
    let within = if meta.elapsed_seconds <= 86_400 {
        "true"
    } else {
        "false"
    };
    format!(
        "host={}\nscope=benchmark\nstart={}\nend={}\ncompleted=true\nbudget_seconds=86400\nwithin_budget={within}\n",
        meta.host, meta.start, meta.end
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Bucket {
    Class,
    Synthetic,
    Compressed,
    RawPcm,
}

struct Held {
    class: String,
    role: String,
    counts: bool,
    transform: String,
    full_len: u64,
    base_len: u64,
    win: bool,
    families: Families,
    zero_num: u64,
    zero_den: u64,
}

fn bucket_of(held: &Held) -> Result<Option<Bucket>, Error> {
    if held.role == "tuning" {
        return Err(Error::BadManifest);
    }
    if held.class == "synthetic" {
        Ok(Some(Bucket::Synthetic))
    } else if held.class == "compressed" {
        Ok(Some(Bucket::Compressed))
    } else if held.transform == "pcm16le-raw" {
        Ok(Some(Bucket::RawPcm))
    } else if held.counts
        && held.role == "evaluation"
        && matches!(held.class.as_str(), "text" | "pcm" | "executable")
    {
        Ok(Some(Bucket::Class))
    } else {
        Ok(None)
    }
}

fn summaries(rows: &[Held], bucket: Bucket) -> Result<Vec<ClassJson>, Error> {
    let names: Vec<&str> = if bucket == Bucket::Class {
        vec!["text", "pcm", "executable"]
    } else if bucket == Bucket::Synthetic {
        vec!["synthetic"]
    } else if bucket == Bucket::Compressed {
        vec!["compressed"]
    } else {
        vec!["pcm"]
    };
    let mut out = Vec::new();
    for name in names {
        let mut matched = Vec::new();
        for row in rows {
            if bucket_of(row)? == Some(bucket) && (bucket != Bucket::Class || row.class == name) {
                if bucket != Bucket::Class && bucket != Bucket::RawPcm && row.class != name {
                    continue;
                }
                if bucket == Bucket::RawPcm || row.class == name || bucket == Bucket::Class {
                    matched.push(row);
                }
            }
        }
        if bucket == Bucket::Class {
            matched.retain(|row| row.class == name);
        }
        if matched.is_empty() {
            continue;
        }
        out.push(fold(name, bucket == Bucket::Class, &matched)?);
    }
    Ok(out)
}

fn fold(name: &str, eligible_kind: bool, rows: &[&Held]) -> Result<ClassJson, Error> {
    let mut evaluation_inputs = 0u64;
    let mut wins = 0u64;
    let mut full_bytes = 0u64;
    let mut baselines_only_bytes = 0u64;
    let mut families = Families::default();
    let mut zero_num = 0u64;
    let mut zero_den = 0u64;
    let mut block_den = 0u64;
    for row in rows {
        evaluation_inputs = evaluation_inputs.saturating_add(1);
        if row.win {
            wins = wins.saturating_add(1);
        }
        full_bytes = full_bytes.saturating_add(row.full_len);
        baselines_only_bytes = baselines_only_bytes.saturating_add(row.base_len);
        families.add_families(&row.families);
        zero_num = zero_num.saturating_add(row.zero_num);
        zero_den = zero_den.saturating_add(row.zero_den);
        block_den = block_den.saturating_add(row.families.blocks());
    }
    let gain_bytes = baselines_only_bytes.saturating_sub(full_bytes.min(baselines_only_bytes));
    let clears = eligible_kind
        && baselines_only_bytes > full_bytes
        && gain_bytes.saturating_mul(100) >= baselines_only_bytes
        && wins.saturating_mul(2) > evaluation_inputs;
    Ok(ClassJson {
        class: name.to_string(),
        eligible: eligible_kind && evaluation_inputs > 0,
        evaluation_inputs,
        wins,
        full_bytes,
        baselines_only_bytes,
        gain_bytes: i64::try_from(baselines_only_bytes).map_err(|_| Error::CostOverflow)?
            - i64::try_from(full_bytes).map_err(|_| Error::CostOverflow)?,
        clears_growth_bar: clears,
        shares: families.shares(block_den, full_bytes, zero_num, zero_den),
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Families {
    raw_blocks: u64,
    direct_blocks: u64,
    previous_blocks: u64,
    constant_blocks: u64,
    polynomial_blocks: u64,
    trigonometric_blocks: u64,
    recurrence_blocks: u64,
    raw_bytes: u64,
    direct_bytes: u64,
    previous_bytes: u64,
    constant_bytes: u64,
    polynomial_bytes: u64,
    trigonometric_bytes: u64,
    recurrence_bytes: u64,
}

impl Families {
    fn add(&mut self, kind: BlockKind, bytes: u64) {
        match kind {
            BlockKind::Raw => {
                self.raw_blocks = self.raw_blocks.saturating_add(1);
                self.raw_bytes = self.raw_bytes.saturating_add(bytes);
            }
            BlockKind::Direct => {
                self.direct_blocks = self.direct_blocks.saturating_add(1);
                self.direct_bytes = self.direct_bytes.saturating_add(bytes);
            }
            BlockKind::PreviousByte => {
                self.previous_blocks = self.previous_blocks.saturating_add(1);
                self.previous_bytes = self.previous_bytes.saturating_add(bytes);
            }
            BlockKind::Constant(_) => {
                self.constant_blocks = self.constant_blocks.saturating_add(1);
                self.constant_bytes = self.constant_bytes.saturating_add(bytes);
            }
            BlockKind::Polynomial { .. } => {
                self.polynomial_blocks = self.polynomial_blocks.saturating_add(1);
                self.polynomial_bytes = self.polynomial_bytes.saturating_add(bytes);
            }
            BlockKind::Trigonometric { .. } => {
                self.trigonometric_blocks = self.trigonometric_blocks.saturating_add(1);
                self.trigonometric_bytes = self.trigonometric_bytes.saturating_add(bytes);
            }
            BlockKind::Recurrence { .. } => {
                self.recurrence_blocks = self.recurrence_blocks.saturating_add(1);
                self.recurrence_bytes = self.recurrence_bytes.saturating_add(bytes);
            }
        }
    }

    fn add_families(&mut self, other: &Self) {
        self.raw_blocks = self.raw_blocks.saturating_add(other.raw_blocks);
        self.direct_blocks = self.direct_blocks.saturating_add(other.direct_blocks);
        self.previous_blocks = self.previous_blocks.saturating_add(other.previous_blocks);
        self.constant_blocks = self.constant_blocks.saturating_add(other.constant_blocks);
        self.polynomial_blocks = self
            .polynomial_blocks
            .saturating_add(other.polynomial_blocks);
        self.trigonometric_blocks = self
            .trigonometric_blocks
            .saturating_add(other.trigonometric_blocks);
        self.recurrence_blocks = self
            .recurrence_blocks
            .saturating_add(other.recurrence_blocks);
        self.raw_bytes = self.raw_bytes.saturating_add(other.raw_bytes);
        self.direct_bytes = self.direct_bytes.saturating_add(other.direct_bytes);
        self.previous_bytes = self.previous_bytes.saturating_add(other.previous_bytes);
        self.constant_bytes = self.constant_bytes.saturating_add(other.constant_bytes);
        self.polynomial_bytes = self.polynomial_bytes.saturating_add(other.polynomial_bytes);
        self.trigonometric_bytes = self
            .trigonometric_bytes
            .saturating_add(other.trigonometric_bytes);
        self.recurrence_bytes = self.recurrence_bytes.saturating_add(other.recurrence_bytes);
    }

    fn blocks(self) -> u64 {
        self.raw_blocks
            .saturating_add(self.direct_blocks)
            .saturating_add(self.previous_blocks)
            .saturating_add(self.constant_blocks)
            .saturating_add(self.polynomial_blocks)
            .saturating_add(self.trigonometric_blocks)
            .saturating_add(self.recurrence_blocks)
    }

    fn shares(self, block_den: u64, byte_den: u64, zero_num: u64, zero_den: u64) -> Shares {
        Shares {
            blocks: FamilyShares {
                raw: Rational {
                    num: self.raw_blocks,
                    den: block_den,
                },
                direct: Rational {
                    num: self.direct_blocks,
                    den: block_den,
                },
                previous_byte: Rational {
                    num: self.previous_blocks,
                    den: block_den,
                },
                constant: Rational {
                    num: self.constant_blocks,
                    den: block_den,
                },
                polynomial: Rational {
                    num: self.polynomial_blocks,
                    den: block_den,
                },
                trigonometric: Rational {
                    num: self.trigonometric_blocks,
                    den: block_den,
                },
                recurrence: Rational {
                    num: self.recurrence_blocks,
                    den: block_den,
                },
            },
            bytes: FamilyShares {
                raw: Rational {
                    num: self.raw_bytes,
                    den: byte_den,
                },
                direct: Rational {
                    num: self.direct_bytes,
                    den: byte_den,
                },
                previous_byte: Rational {
                    num: self.previous_bytes,
                    den: byte_den,
                },
                constant: Rational {
                    num: self.constant_bytes,
                    den: byte_den,
                },
                polynomial: Rational {
                    num: self.polynomial_bytes,
                    den: byte_den,
                },
                trigonometric: Rational {
                    num: self.trigonometric_bytes,
                    den: byte_den,
                },
                recurrence: Rational {
                    num: self.recurrence_bytes,
                    den: byte_den,
                },
            },
            zero_residual: Rational {
                num: zero_num,
                den: zero_den,
            },
        }
    }
}

fn candidates_of(kinds: &[BlockKind]) -> Result<Vec<SearchCandidate>, Error> {
    let mut out = Vec::new();
    for kind in kinds {
        let id = candidate_id(*kind).ok_or(Error::Kernel("NoCandidate".to_string()))?;
        out.push(SearchCandidate { id, kind: *kind });
    }
    Ok(out)
}

fn full_candidates(book: &Codebook) -> Result<Vec<SearchCandidate>, Error> {
    let mut out = candidates_of(&all_baselines())?;
    for (index, kind) in book.formulas.iter().copied().enumerate() {
        let offset = u16::try_from(index).map_err(|_| Error::CostOverflow)?;
        let id = 259u16.checked_add(offset).ok_or(Error::CostOverflow)?;
        out.push(SearchCandidate { id, kind });
    }
    Ok(out)
}

fn constant_kinds() -> Vec<BlockKind> {
    let mut kinds = Vec::with_capacity(257);
    kinds.push(BlockKind::Raw);
    let mut byte = 0u8;
    loop {
        kinds.push(BlockKind::Constant(byte));
        if byte == u8::MAX {
            break;
        }
        byte = byte.saturating_add(1);
    }
    kinds
}

fn reject_bad_rows(rows: &[Row], missing: &[String]) -> Result<(), Error> {
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        legal_id(&row.id)?;
        if !seen.insert(row.id.clone()) {
            return Err(Error::BadManifest);
        }
        if row.role == "tuning" {
            return Err(Error::BadManifest);
        }
        if is_synthetic_id(&row.id)
            && (row.class != "synthetic" || row.role != "synthetic" || row.counts_toward_sm1)
        {
            return Err(Error::BadManifest);
        }
    }
    for id in missing {
        legal_id(id)?;
        if !seen.insert(id.clone()) {
            return Err(Error::BadManifest);
        }
    }
    Ok(())
}

fn is_synthetic_id(id: &str) -> bool {
    matches!(
        id,
        "synthetic-polynomial" | "synthetic-trigonometric" | "synthetic-recurrence"
    )
}

fn legal_id(id: &str) -> Result<(), Error> {
    if id.is_empty()
        || id == "."
        || id == ".."
        || id.contains('/')
        || id.contains('\\')
        || id.contains("..")
        || id.chars().any(char::is_control)
    {
        return Err(Error::BadManifest);
    }
    Ok(())
}

struct ManifestRow {
    id: String,
    class: String,
    role: String,
    counts: bool,
    transform: String,
    derived: String,
}

fn parse_manifest(text: &str) -> Result<Vec<ManifestRow>, Error> {
    let value: toml::Value = toml::from_str(text).map_err(|_| Error::BadManifest)?;
    let table = value.as_table().ok_or(Error::BadManifest)?;
    let inputs = table
        .get("input")
        .and_then(toml::Value::as_array)
        .ok_or(Error::BadManifest)?;
    let mut rows = Vec::new();
    for input in inputs {
        let row = input.as_table().ok_or(Error::BadManifest)?;
        rows.push(ManifestRow {
            id: string_field(row, "id")?,
            class: string_field(row, "class")?,
            role: string_field(row, "role")?,
            counts: row
                .get("counts_toward_sm1")
                .and_then(toml::Value::as_bool)
                .ok_or(Error::BadManifest)?,
            transform: string_field(row, "transform")?,
            derived: string_field(row, "derived_sha256")?,
        });
    }
    Ok(rows)
}

fn string_field(row: &toml::map::Map<String, toml::Value>, key: &str) -> Result<String, Error> {
    row.get(key)
        .and_then(toml::Value::as_str)
        .map(str::to_string)
        .ok_or(Error::BadManifest)
}

fn parse_gate(text: &str) -> Result<GateJson, Error> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|_| Error::BadManifest)?;
    let object = value.as_object().ok_or(Error::BadManifest)?;
    let ready = object
        .get("corpus_gate_ready")
        .and_then(serde_json::Value::as_bool)
        .ok_or(Error::BadManifest)?;
    let scope = object
        .get("scope")
        .and_then(serde_json::Value::as_str)
        .ok_or(Error::BadManifest)?
        .to_string();
    let unresolved_value = object
        .get("unresolved")
        .and_then(serde_json::Value::as_array)
        .ok_or(Error::BadManifest)?;
    let mut unresolved = Vec::new();
    for item in unresolved_value {
        if let Some(text) = item.as_str() {
            unresolved.push(text.to_string());
        } else if let Some(object) = item.as_object() {
            let field = object
                .get("field")
                .and_then(serde_json::Value::as_str)
                .ok_or(Error::BadManifest)?;
            unresolved.push(field.to_string());
        } else {
            return Err(Error::BadManifest);
        }
    }
    Ok(GateJson {
        corpus_gate_ready: ready,
        scope,
        unresolved,
    })
}

fn hex_bytes(bytes: &[u8]) -> Result<String, Error> {
    const HEX: [u8; 16] = *b"0123456789abcdef";
    let mut out = String::new();
    for byte in bytes {
        let hi = HEX
            .get(usize::from(byte >> 4))
            .copied()
            .ok_or(Error::HashMismatch)?;
        let lo = HEX
            .get(usize::from(byte & 0x0f))
            .copied()
            .ok_or(Error::HashMismatch)?;
        out.push(char::from(hi));
        out.push(char::from(lo));
    }
    Ok(out)
}

#[derive(Clone, Debug)]
struct Report {
    report_version: u64,
    status: String,
    opens_pcm_experiments: bool,
    experiment_id: String,
    tool: String,
    tool_version: String,
    format_version: u64,
    layout: String,
    spec_seed_sha256: String,
    model_constants: String,
    decisions: Decisions,
    thresholds: Thresholds,
    block_lengths: Vec<u16>,
    codebook: CodebookJson,
    manifest_sha256: String,
    gate: GateJson,
    overlap: Overlap,
    missing_gates: Vec<String>,
    search_mode: String,
    threads: u64,
    tuning: TuningJson,
    missing_inputs: Vec<String>,
    inputs: Vec<InputJson>,
    classes: Vec<ClassJson>,
    synthetic: Vec<ClassJson>,
    compressed_control: Vec<ClassJson>,
    non_counted_pcm: Vec<ClassJson>,
}

#[derive(Clone, Debug)]
struct Decisions {
    d01: String,
    d02: String,
    d03: String,
    d04: String,
    d05: String,
    d06: String,
    d07: String,
    d08: String,
    d09: String,
}

impl Decisions {
    fn pinned() -> Self {
        Self {
            d01: "approved".to_string(),
            d02: "approved".to_string(),
            d03: "approved".to_string(),
            d04: "open".to_string(),
            d05: "approved".to_string(),
            d06: "approved".to_string(),
            d07: "open".to_string(),
            d08: "open".to_string(),
            d09: "open".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
struct Thresholds {
    growth_percent: u64,
    majority: String,
    gain_denominator: String,
}

#[derive(Clone, Debug)]
struct CodebookJson {
    name: String,
    version: u16,
    label: String,
    sha256: String,
}

#[derive(Clone, Debug)]
struct GateJson {
    corpus_gate_ready: bool,
    scope: String,
    unresolved: Vec<String>,
}

#[derive(Clone, Debug)]
struct Overlap {
    k: u64,
    k_status: String,
    result: String,
}

#[derive(Clone, Debug)]
struct TuningJson {
    ran: bool,
    reason: Option<String>,
    width_grid: String,
    width_reason: String,
}

#[derive(Clone, Debug)]
struct InputJson {
    id: String,
    class: String,
    role: String,
    counts_toward_sm1: bool,
    input_bytes: u64,
    input_sha256: String,
    arms: ArmsJson,
    best_forced: BestForced,
    gain_bytes: i64,
    win: bool,
    margin_bytes: i64,
    sidecar: SidecarJson,
}

#[derive(Clone, Debug)]
struct ArmsJson {
    whole_file_raw: ArmJson,
    forced_constant: ArmJson,
    forced_previous: ArmJson,
    forced_direct: ArmJson,
    baselines_only: ArmJson,
    full: ArmJson,
}

#[derive(Clone, Debug)]
struct ArmJson {
    file_len: u64,
    account: AccountJson,
    work: Work,
}

#[derive(Clone, Debug)]
struct AccountJson {
    file_header: u64,
    block_headers: u64,
    parameters: u64,
    alignment_padding: u64,
    raw_payload: u64,
    coder_payload: u64,
    coder_flush: u64,
    crc: u64,
}

impl From<ByteAccount> for AccountJson {
    fn from(account: ByteAccount) -> Self {
        Self {
            file_header: account.file_header,
            block_headers: account.block_headers,
            parameters: account.parameters,
            alignment_padding: account.alignment_padding,
            raw_payload: account.raw_payload,
            coder_payload: account.coder_payload,
            coder_flush: account.coder_flush,
            crc: account.crc,
        }
    }
}

#[derive(Clone, Debug)]
struct Work {
    input_bytes: u64,
    selected_model_decisions: u64,
    selected_coder_steps: u64,
    search_encode_block_calls: u64,
    search_skipped: u64,
}

#[derive(Clone, Debug)]
struct BestForced {
    arm: String,
    file_len: u64,
}

#[derive(Clone, Debug)]
struct SidecarJson {
    present: bool,
    coverage: String,
    name: String,
}

#[derive(Clone, Debug)]
struct ClassJson {
    class: String,
    eligible: bool,
    evaluation_inputs: u64,
    wins: u64,
    full_bytes: u64,
    baselines_only_bytes: u64,
    gain_bytes: i64,
    clears_growth_bar: bool,
    shares: Shares,
}

#[derive(Clone, Debug)]
struct Shares {
    blocks: FamilyShares,
    bytes: FamilyShares,
    zero_residual: Rational,
}

#[derive(Clone, Debug)]
struct FamilyShares {
    raw: Rational,
    direct: Rational,
    previous_byte: Rational,
    constant: Rational,
    polynomial: Rational,
    trigonometric: Rational,
    recurrence: Rational,
}

#[derive(Clone, Copy, Debug)]
struct Rational {
    num: u64,
    den: u64,
}

#[derive(Clone, Debug)]
struct AnnexRow {
    id: String,
    ideal_microbits: i64,
    entropy_microbits: i64,
    overhead_microbits: i64,
    flush_bytes: u64,
}

struct Obj {
    body: String,
    empty: bool,
}

impl Obj {
    fn new() -> Self {
        Self {
            body: String::from("{"),
            empty: true,
        }
    }

    fn raw(&mut self, key: &str, value: &str) {
        if !self.empty {
            self.body.push(',');
        }
        self.empty = false;
        self.body.push('"');
        self.body.push_str(key);
        self.body.push_str("\":");
        self.body.push_str(value);
    }

    fn text(&mut self, key: &str, value: &str) -> Result<(), Error> {
        self.raw(key, &json_string(value)?);
        Ok(())
    }

    fn num(&mut self, key: &str, value: u64) {
        self.raw(key, &value.to_string());
    }

    fn signed(&mut self, key: &str, value: i64) {
        self.raw(key, &value.to_string());
    }

    fn flag(&mut self, key: &str, value: bool) {
        self.raw(key, if value { "true" } else { "false" });
    }

    fn end(mut self) -> String {
        self.body.push('}');
        self.body
    }
}

fn json_string(value: &str) -> Result<String, Error> {
    serde_json::to_string(value).map_err(|_| Error::BadManifest)
}

fn render_report(report: &Report) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.num("report_version", report.report_version);
    obj.text("status", &report.status)?;
    obj.raw("verdict", "null");
    obj.flag("opens_pcm_experiments", report.opens_pcm_experiments);
    obj.text("experiment_id", &report.experiment_id)?;
    obj.text("tool", &report.tool)?;
    obj.text("tool_version", &report.tool_version)?;
    obj.num("format_version", report.format_version);
    obj.text("layout", &report.layout)?;
    obj.text("spec_seed_sha256", &report.spec_seed_sha256)?;
    obj.text("model_constants", &report.model_constants)?;
    obj.raw("decisions", &render_decisions(&report.decisions)?);
    obj.raw("thresholds", &render_thresholds(&report.thresholds)?);
    obj.raw(
        "block_lengths",
        &format!(
            "[{},{},{}]",
            report.block_lengths.first().copied().unwrap_or(0),
            report.block_lengths.get(1).copied().unwrap_or(0),
            report.block_lengths.get(2).copied().unwrap_or(0)
        ),
    );
    obj.raw("codebook", &render_codebook(&report.codebook)?);
    obj.text("manifest_sha256", &report.manifest_sha256)?;
    obj.raw("gate", &render_gate(&report.gate)?);
    obj.raw("overlap", &render_overlap(&report.overlap)?);
    obj.raw("missing_gates", &render_strings(&report.missing_gates)?);
    obj.text("search_mode", &report.search_mode)?;
    obj.num("threads", report.threads);
    obj.raw("tuning", &render_tuning(&report.tuning)?);
    obj.raw("missing_inputs", &render_strings(&report.missing_inputs)?);
    obj.raw("inputs", &render_inputs(&report.inputs)?);
    obj.raw("classes", &render_classes(&report.classes)?);
    obj.raw("synthetic", &render_classes(&report.synthetic)?);
    obj.raw(
        "compressed_control",
        &render_classes(&report.compressed_control)?,
    );
    obj.raw("non_counted_pcm", &render_classes(&report.non_counted_pcm)?);
    Ok(format!("{}\n", obj.end()))
}

fn render_decisions(decisions: &Decisions) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.text("d01", &decisions.d01)?;
    obj.text("d02", &decisions.d02)?;
    obj.text("d03", &decisions.d03)?;
    obj.text("d04", &decisions.d04)?;
    obj.text("d05", &decisions.d05)?;
    obj.text("d06", &decisions.d06)?;
    obj.text("d07", &decisions.d07)?;
    obj.text("d08", &decisions.d08)?;
    obj.text("d09", &decisions.d09)?;
    Ok(obj.end())
}

fn render_thresholds(thresholds: &Thresholds) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.num("growth_percent", thresholds.growth_percent);
    obj.text("majority", &thresholds.majority)?;
    obj.text("gain_denominator", &thresholds.gain_denominator)?;
    Ok(obj.end())
}

fn render_codebook(codebook: &CodebookJson) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.text("name", &codebook.name)?;
    obj.num("version", u64::from(codebook.version));
    obj.text("label", &codebook.label)?;
    obj.text("sha256", &codebook.sha256)?;
    Ok(obj.end())
}

fn render_gate(gate: &GateJson) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.flag("corpus_gate_ready", gate.corpus_gate_ready);
    obj.text("scope", &gate.scope)?;
    obj.raw("unresolved", &render_strings(&gate.unresolved)?);
    Ok(obj.end())
}

fn render_overlap(overlap: &Overlap) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.num("k", overlap.k);
    obj.text("k_status", &overlap.k_status)?;
    obj.text("result", &overlap.result)?;
    Ok(obj.end())
}

fn render_tuning(tuning: &TuningJson) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.flag("ran", tuning.ran);
    match &tuning.reason {
        Some(reason) => obj.text("reason", reason)?,
        None => obj.raw("reason", "null"),
    }
    obj.text("width_grid", &tuning.width_grid)?;
    obj.text("width_reason", &tuning.width_reason)?;
    Ok(obj.end())
}

fn render_strings(values: &[String]) -> Result<String, Error> {
    let mut out = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&json_string(value)?);
    }
    out.push(']');
    Ok(out)
}

fn render_inputs(inputs: &[InputJson]) -> Result<String, Error> {
    let mut out = String::from("[");
    for (index, input) in inputs.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&render_input(input)?);
    }
    out.push(']');
    Ok(out)
}

fn render_input(input: &InputJson) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.text("id", &input.id)?;
    obj.text("class", &input.class)?;
    obj.text("role", &input.role)?;
    obj.flag("counts_toward_sm1", input.counts_toward_sm1);
    obj.num("input_bytes", input.input_bytes);
    obj.text("input_sha256", &input.input_sha256)?;
    obj.raw("arms", &render_arms(&input.arms)?);
    obj.raw("best_forced", &render_best(&input.best_forced)?);
    obj.signed("gain_bytes", input.gain_bytes);
    obj.flag("win", input.win);
    obj.signed("margin_bytes", input.margin_bytes);
    obj.raw("sidecar", &render_sidecar(&input.sidecar)?);
    Ok(obj.end())
}

fn render_arms(arms: &ArmsJson) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.raw("whole_file_raw", &render_arm(&arms.whole_file_raw)?);
    obj.raw("forced_constant", &render_arm(&arms.forced_constant)?);
    obj.raw("forced_previous", &render_arm(&arms.forced_previous)?);
    obj.raw("forced_direct", &render_arm(&arms.forced_direct)?);
    obj.raw("baselines_only", &render_arm(&arms.baselines_only)?);
    obj.raw("full", &render_arm(&arms.full)?);
    Ok(obj.end())
}

fn render_arm(arm: &ArmJson) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.num("file_len", arm.file_len);
    obj.raw("account", &render_account(&arm.account));
    obj.raw("work", &render_work(&arm.work));
    Ok(obj.end())
}

fn render_account(account: &AccountJson) -> String {
    let mut obj = Obj::new();
    obj.num("file_header", account.file_header);
    obj.num("block_headers", account.block_headers);
    obj.num("parameters", account.parameters);
    obj.num("alignment_padding", account.alignment_padding);
    obj.num("raw_payload", account.raw_payload);
    obj.num("coder_payload", account.coder_payload);
    obj.num("coder_flush", account.coder_flush);
    obj.num("crc", account.crc);
    obj.end()
}

fn render_work(work: &Work) -> String {
    let mut obj = Obj::new();
    obj.num("input_bytes", work.input_bytes);
    obj.num("selected_model_decisions", work.selected_model_decisions);
    obj.num("selected_coder_steps", work.selected_coder_steps);
    obj.num("search_encode_block_calls", work.search_encode_block_calls);
    obj.num("search_skipped", work.search_skipped);
    obj.end()
}

fn render_best(best: &BestForced) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.text("arm", &best.arm)?;
    obj.num("file_len", best.file_len);
    Ok(obj.end())
}

fn render_sidecar(sidecar: &SidecarJson) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.flag("present", sidecar.present);
    obj.text("coverage", &sidecar.coverage)?;
    obj.text("name", &sidecar.name)?;
    Ok(obj.end())
}

fn render_classes(classes: &[ClassJson]) -> Result<String, Error> {
    let mut out = String::from("[");
    for (index, class) in classes.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&render_class(class)?);
    }
    out.push(']');
    Ok(out)
}

fn render_class(class: &ClassJson) -> Result<String, Error> {
    let mut obj = Obj::new();
    obj.text("class", &class.class)?;
    obj.flag("eligible", class.eligible);
    obj.num("evaluation_inputs", class.evaluation_inputs);
    obj.num("wins", class.wins);
    obj.num("full_bytes", class.full_bytes);
    obj.num("baselines_only_bytes", class.baselines_only_bytes);
    obj.signed("gain_bytes", class.gain_bytes);
    obj.flag("clears_growth_bar", class.clears_growth_bar);
    obj.raw("shares", &render_shares(&class.shares));
    Ok(obj.end())
}

fn render_shares(shares: &Shares) -> String {
    let mut obj = Obj::new();
    obj.raw("blocks", &render_families(&shares.blocks));
    obj.raw("bytes", &render_families(&shares.bytes));
    obj.raw("zero_residual", &render_rational(shares.zero_residual));
    obj.end()
}

fn render_families(families: &FamilyShares) -> String {
    let mut obj = Obj::new();
    obj.raw("raw", &render_rational(families.raw));
    obj.raw("direct", &render_rational(families.direct));
    obj.raw("previous_byte", &render_rational(families.previous_byte));
    obj.raw("constant", &render_rational(families.constant));
    obj.raw("polynomial", &render_rational(families.polynomial));
    obj.raw("trigonometric", &render_rational(families.trigonometric));
    obj.raw("recurrence", &render_rational(families.recurrence));
    obj.end()
}

fn render_rational(rational: Rational) -> String {
    let mut obj = Obj::new();
    obj.num("num", rational.num);
    obj.num("den", rational.den);
    obj.end()
}

fn render_annex(rows: &[AnnexRow]) -> Result<String, Error> {
    let mut out = String::from("[");
    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        let mut obj = Obj::new();
        obj.text("id", &row.id)?;
        obj.signed("ideal_microbits", row.ideal_microbits);
        obj.signed("entropy_microbits", row.entropy_microbits);
        obj.signed("overhead_microbits", row.overhead_microbits);
        obj.num("flush_bytes", row.flush_bytes);
        out.push_str(&obj.end());
    }
    out.push(']');
    out.push('\n');
    Ok(out)
}
