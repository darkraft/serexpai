//! Little-endian `SXPC` cost matrix.

use sxp_encoder::{CostCell, SpanRecord};

use crate::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    Default,
    OptIn,
    Dense,
}

impl Coverage {
    pub const fn byte(self) -> u8 {
        match self {
            Self::Default => 1,
            Self::OptIn => 2,
            Self::Dense => 3,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::OptIn => "opt-in",
            Self::Dense => "dense",
        }
    }
}

pub fn sidecar_bytes(
    coverage: Coverage,
    reference: bool,
    candidate_ids: &[u16],
    spans: &[SpanRecord],
) -> Result<Vec<u8>, Error> {
    let count = u16::try_from(candidate_ids.len()).map_err(|_| Error::CostOverflow)?;
    let span_count = u32::try_from(spans.len()).map_err(|_| Error::CostOverflow)?;
    let mut out = Vec::new();
    out.extend_from_slice(b"SXPC");
    out.push(1);
    out.push(coverage.byte());
    out.push(if reference { 1 } else { 0 });
    out.push(0);
    out.extend_from_slice(&count.to_le_bytes());
    for id in candidate_ids {
        out.extend_from_slice(&id.to_le_bytes());
    }
    out.extend_from_slice(&span_count.to_le_bytes());
    for span in spans {
        if span.cells.len() != candidate_ids.len() {
            return Err(Error::CostOverflow);
        }
        if span.length_code > 2 {
            return Err(Error::BadManifest);
        }
        out.extend_from_slice(&span.offset.to_le_bytes());
        out.push(span.length_code);
        for (index, (id, cell)) in span.cells.iter().enumerate() {
            let expected = candidate_ids.get(index).ok_or(Error::CostOverflow)?;
            if id != expected {
                return Err(Error::CostOverflow);
            }
            let stored = match cell {
                CostCell::Skipped => {
                    if reference {
                        return Err(Error::CostOverflow);
                    }
                    0xFFFF
                }
                CostCell::Evaluated(cost) => {
                    if *cost > 65_534 {
                        return Err(Error::CostOverflow);
                    }
                    u16::try_from(*cost).map_err(|_| Error::CostOverflow)?
                }
            };
            out.extend_from_slice(&stored.to_le_bytes());
        }
    }
    Ok(out)
}
