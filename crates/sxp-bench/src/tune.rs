//! Model-constant grid. Scores stay on the Baselines-only side path.
//!
//! The winner is recorded and is not installed into `encode_block`.

use sxp_encoder::{
    LengthPolicy, SearchCandidate, SearchMode, all_baselines, candidate_id, search_arm,
};
use sxp_kernel::{InitialProbability, ModelConfig};

use crate::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Point {
    pub cap: u16,
    pub delta: String,
    pub initial: String,
    pub p: u16,
    pub score: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tuning {
    pub points: Vec<Point>,
    pub winner: Point,
    pub width_grid: String,
    pub width_reason: String,
}

pub struct TuneRow<'a> {
    pub role: &'a str,
    pub bytes: &'a [u8],
}

/// Score the fixed grid. An evaluation row writes nothing.
pub fn tune(rows: &[TuneRow<'_>]) -> Result<Tuning, Error> {
    if rows.is_empty() {
        return Err(Error::BadManifest);
    }
    for row in rows {
        if row.role != "tuning" {
            return Err(Error::EvaluationLeak);
        }
    }
    let candidates = baseline_candidates()?;
    let mut scalars = Vec::new();
    for cap in [24u16, 32, 60, 128, 255] {
        for (delta, half) in [("1", false), ("0.5", true)] {
            let model = model_of(cap, half, false)?;
            let score = score_rows(rows, &candidates, &model)?;
            scalars.push(Point {
                cap,
                delta: delta.to_string(),
                initial: "scalar".to_string(),
                p: 32_768,
                score,
            });
        }
    }
    sort_points(&mut scalars);
    let best = scalars.first().cloned().ok_or(Error::BadManifest)?;
    let half = best.delta == "0.5";
    let model = model_of(best.cap, half, true)?;
    let depth_score = score_rows(rows, &candidates, &model)?;
    let mut points = scalars;
    points.push(Point {
        cap: best.cap,
        delta: best.delta.clone(),
        initial: "by_depth".to_string(),
        p: 32_768,
        score: depth_score,
    });
    sort_points(&mut points);
    let winner = points.first().cloned().ok_or(Error::BadManifest)?;
    Ok(Tuning {
        points,
        winner,
        width_grid: "not-run".to_string(),
        width_reason: "d09-open".to_string(),
    })
}

/// Canonical `tuning.json` bytes. The default benchmark does not call this.
pub fn tuning_json(tuning: &Tuning) -> Result<String, Error> {
    let mut out = String::from("{\"points\":[");
    for (index, point) in tuning.points.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&point_json(point)?);
    }
    out.push_str("],\"winner\":");
    out.push_str(&point_json(&tuning.winner)?);
    let width = json_string(&tuning.width_grid)?;
    let reason = json_string(&tuning.width_reason)?;
    out.push_str(&format!(
        ",\"width_grid\":{width},\"width_reason\":{reason}}}\n"
    ));
    Ok(out)
}

fn point_json(point: &Point) -> Result<String, Error> {
    let delta = json_string(&point.delta)?;
    let initial = json_string(&point.initial)?;
    Ok(format!(
        "{{\"cap\":{},\"delta\":{delta},\"initial\":{initial},\"p\":{},\"score\":{}}}",
        point.cap, point.p, point.score
    ))
}

fn json_string(value: &str) -> Result<String, Error> {
    serde_json::to_string(value).map_err(|_| Error::BadManifest)
}

pub fn sort_points(points: &mut [Point]) {
    points.sort_by(|left, right| {
        left.score
            .cmp(&right.score)
            .then(left.cap.cmp(&right.cap))
            .then(delta_rank(&left.delta).cmp(&delta_rank(&right.delta)))
            .then(initial_rank(&left.initial).cmp(&initial_rank(&right.initial)))
    });
}

fn delta_rank(delta: &str) -> u8 {
    if delta == "1" { 0 } else { 1 }
}

fn initial_rank(initial: &str) -> u8 {
    if initial == "scalar" { 0 } else { 1 }
}

fn score_rows(
    rows: &[TuneRow<'_>],
    candidates: &[SearchCandidate],
    model: &ModelConfig,
) -> Result<u64, Error> {
    let mut total = 0u64;
    for row in rows {
        let arm = search_arm(
            row.bytes,
            candidates,
            LengthPolicy::All,
            SearchMode::HeaderPrune,
            0,
            model,
        )
        .map_err(Error::kernel)?;
        total = total
            .checked_add(u64::try_from(arm.file.len()).map_err(|_| Error::CostOverflow)?)
            .ok_or(Error::CostOverflow)?;
    }
    Ok(total)
}

fn baseline_candidates() -> Result<Vec<SearchCandidate>, Error> {
    let mut out = Vec::new();
    for kind in all_baselines() {
        let id = candidate_id(kind).ok_or(Error::Kernel("NoCandidate".to_string()))?;
        out.push(SearchCandidate { id, kind });
    }
    Ok(out)
}

fn model_of(cap: u16, delta_half: bool, by_depth: bool) -> Result<ModelConfig, Error> {
    let recip = recip_table(cap, delta_half)?;
    let initial = if by_depth {
        InitialProbability::ByDepth([32_768; 8])
    } else {
        InitialProbability::Scalar(32_768)
    };
    ModelConfig::try_new(initial, cap, &recip).map_err(Error::kernel)
}

fn recip_table(cap: u16, delta_half: bool) -> Result<Vec<u16>, Error> {
    let mut out = Vec::new();
    let mut n = 0u16;
    loop {
        let denom = if delta_half {
            u32::from(n).saturating_mul(2).saturating_add(3)
        } else {
            u32::from(n).saturating_add(2)
        };
        let numer = if delta_half { 131_072u32 } else { 65_536u32 };
        if denom == 0 {
            return Err(Error::Kernel("InvalidParameter".to_string()));
        }
        let rate = numer / denom;
        let rate =
            u16::try_from(rate).map_err(|_| Error::Kernel("InvalidParameter".to_string()))?;
        if rate == 0 {
            return Err(Error::Kernel("InvalidParameter".to_string()));
        }
        out.push(rate);
        if n == cap {
            break;
        }
        n = n.saturating_add(1);
    }
    Ok(out)
}
