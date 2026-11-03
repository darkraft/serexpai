//! Regenerable fixture Codebook. The production recipe is unresolved.
//!
//! Fixture `cw` literals are not a D-07 choice. This module takes no input bytes.

use sha2::{Digest, Sha256};
use sxp_kernel::{BlockKind, Polynomial, Recurrence, Trigonometric, formula_section};

use crate::Error;

/// One generated Codebook. `formulas` is in id order, starting at 259.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Codebook {
    pub name: String,
    pub label: String,
    pub version: u16,
    pub formulas: Vec<BlockKind>,
    pub content_sha256: [u8; 32],
}

/// The only recipe `generate_codebook` accepts.
///
/// # Errors
///
/// [`Error::AbsentRecipe`] for an empty or unknown name.
/// [`Error::UnresolvedProduction`] for `production` and `benchmark`.
pub fn generate_codebook(name: &str) -> Result<Codebook, Error> {
    match name {
        "production" | "benchmark" => Err(Error::UnresolvedProduction),
        "fixture-v1" => assemble_codebook("fixture-v1", "fixture", 1, fixture_formulas()?),
        "" => Err(Error::AbsentRecipe),
        _ => Err(Error::AbsentRecipe),
    }
}

/// Dedup, check caps, then hash. Ids are `259 + index` after the drop.
///
/// # Errors
///
/// [`Error::UnresolvedProduction`] when the name or the label is `production`
/// or `benchmark`. [`Error::AbsentRecipe`] when the name is empty.
/// [`Error::CapExceeded`] and [`Error::MissingFamily`] come from
/// [`check_family_caps`]. A baseline kind, or a formula the kernel rejects,
/// is [`Error::Kernel`].
pub fn assemble_codebook(
    name: &str,
    label: &str,
    version: u16,
    formulas: Vec<BlockKind>,
) -> Result<Codebook, Error> {
    if name == "production" || name == "benchmark" || label == "production" || label == "benchmark"
    {
        return Err(Error::UnresolvedProduction);
    }
    if name.is_empty() {
        return Err(Error::AbsentRecipe);
    }
    let mut kept = Vec::new();
    let mut seen = Vec::new();
    for kind in formulas {
        if candidate_is_baseline(kind) {
            return Err(Error::Kernel(sxp_kernel::Error::InvalidParameter));
        }
        let section = formula_section(kind).map_err(Error::Kernel)?;
        if seen.iter().any(|got: &Vec<u8>| got == &section) {
            continue;
        }
        seen.push(section);
        kept.push(kind);
    }
    let (polynomial, trigonometric, recurrence) = family_counts(&kept)?;
    check_family_caps(polynomial, trigonometric, recurrence)?;
    let codebook = Codebook {
        name: name.to_owned(),
        label: label.to_owned(),
        version,
        formulas: kept,
        content_sha256: [0; 32],
    };
    let bytes = canonical_bytes(&codebook)?;
    Ok(Codebook {
        content_sha256: sha256(&bytes)?,
        ..codebook
    })
}

/// A family above 2048, or a total above 4096, is [`Error::CapExceeded`].
/// A zero family after that is [`Error::MissingFamily`].
///
/// # Errors
///
/// [`Error::CapExceeded`] or [`Error::MissingFamily`].
pub fn check_family_caps(
    polynomial: usize,
    trigonometric: usize,
    recurrence: usize,
) -> Result<(), Error> {
    if polynomial > 2048 || trigonometric > 2048 || recurrence > 2048 {
        return Err(Error::CapExceeded);
    }
    let total = polynomial
        .saturating_add(trigonometric)
        .saturating_add(recurrence);
    if total > 4096 {
        return Err(Error::CapExceeded);
    }
    if polynomial == 0 || trigonometric == 0 || recurrence == 0 {
        return Err(Error::MissingFamily);
    }
    Ok(())
}

/// SHA-256 preimage. Magic, schema version, name, label, recipe version, count, entries.
///
/// # Errors
///
/// [`Error::Kernel`] when a formula section cannot be written.
pub fn canonical_bytes(codebook: &Codebook) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    out.extend_from_slice(b"sxp-codebook-v1");
    out.extend_from_slice(&1u16.to_le_bytes());
    push_str(&mut out, &codebook.name)?;
    push_str(&mut out, &codebook.label)?;
    out.extend_from_slice(&codebook.version.to_le_bytes());
    let count = u32::try_from(codebook.formulas.len()).map_err(|_| Error::CapExceeded)?;
    out.extend_from_slice(&count.to_le_bytes());
    for kind in &codebook.formulas {
        let family = family_tag(*kind)?;
        let section = formula_section(*kind).map_err(Error::Kernel)?;
        let len = u16::try_from(section.len())
            .map_err(|_| Error::Kernel(sxp_kernel::Error::BlockTooLarge))?;
        out.push(family);
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&section);
    }
    Ok(out)
}

fn fixture_formulas() -> Result<Vec<BlockKind>, Error> {
    let polynomial = BlockKind::polynomial(
        Polynomial::new(2, 8, &[0, 0, 2_147_483_647]).map_err(Error::Kernel)?,
        32,
    )
    .map_err(Error::Kernel)?;
    let trigonometric = BlockKind::trigonometric(
        Trigonometric::new(1, 8, 1 << 24, &[0, 1 << 20, 0]).map_err(Error::Kernel)?,
        22,
    )
    .map_err(Error::Kernel)?;
    let recurrence = BlockKind::recurrence(
        Recurrence::new(2, 251, &[3, 5, 7], &[1, 2]).map_err(Error::Kernel)?,
        5,
    )
    .map_err(Error::Kernel)?;
    Ok(vec![polynomial, trigonometric, recurrence])
}

fn candidate_is_baseline(kind: BlockKind) -> bool {
    matches!(
        kind,
        BlockKind::Raw | BlockKind::Direct | BlockKind::PreviousByte | BlockKind::Constant(_)
    )
}

fn family_counts(formulas: &[BlockKind]) -> Result<(usize, usize, usize), Error> {
    let mut polynomial = 0usize;
    let mut trigonometric = 0usize;
    let mut recurrence = 0usize;
    for kind in formulas {
        match kind {
            BlockKind::Polynomial { .. } => polynomial = polynomial.saturating_add(1),
            BlockKind::Trigonometric { .. } => trigonometric = trigonometric.saturating_add(1),
            BlockKind::Recurrence { .. } => recurrence = recurrence.saturating_add(1),
            BlockKind::Raw
            | BlockKind::Direct
            | BlockKind::PreviousByte
            | BlockKind::Constant(_) => {
                return Err(Error::Kernel(sxp_kernel::Error::InvalidParameter));
            }
        }
    }
    Ok((polynomial, trigonometric, recurrence))
}

fn family_tag(kind: BlockKind) -> Result<u8, Error> {
    match kind {
        BlockKind::Polynomial { .. } => Ok(0),
        BlockKind::Trigonometric { .. } => Ok(1),
        BlockKind::Recurrence { .. } => Ok(2),
        BlockKind::Raw | BlockKind::Direct | BlockKind::PreviousByte | BlockKind::Constant(_) => {
            Err(Error::Kernel(sxp_kernel::Error::InvalidParameter))
        }
    }
}

fn push_str(out: &mut Vec<u8>, text: &str) -> Result<(), Error> {
    let len = u16::try_from(text.len()).map_err(|_| Error::CapExceeded)?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(text.as_bytes());
    Ok(())
}

fn sha256(bytes: &[u8]) -> Result<[u8; 32], Error> {
    let digest = Sha256::digest(bytes);
    let mut out = [0u8; 32];
    if digest.len() != out.len() {
        return Err(Error::Kernel(sxp_kernel::Error::InvalidParameter));
    }
    for (index, byte) in digest.iter().enumerate() {
        let slot = out
            .get_mut(index)
            .ok_or(Error::Kernel(sxp_kernel::Error::InvalidParameter))?;
        *slot = *byte;
    }
    Ok(out)
}
