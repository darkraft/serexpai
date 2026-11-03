//! Exact-size baseline selection. Costs come from [`sxp_kernel::encode_block`].
//! The file bytes come from the kernel writers.

use std::collections::{BTreeMap, BTreeSet};

use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use sxp_kernel::{
    BlockKind, BlockPlan, LENGTH_LIMIT, NominalLength, actual_len, encode_block,
    encode_whole_file_raw_header, write_blocked_file, write_whole_file_raw,
};

use crate::Error;

/// Which nominals may start a full Block. The final Block can still use the
/// canonical nominal from {256, 1024, 2048}.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LengthPolicy {
    All,
    Fixed(NominalLength),
}

/// `Reference` encodes every candidate. `HeaderPrune` skips a later candidate
/// whose section is already at least the incumbent size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchMode {
    Reference,
    HeaderPrune,
}

/// Sum of encoded Block bytes, how many Blocks, and the winning steps in
/// file order. The file header and the CRC are not included.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartitionKey {
    pub block_bytes: u64,
    pub block_count: u32,
    pub steps: Vec<(u8, u16)>,
}

/// One chosen Block. `block_len` is the kernel encoding, not the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    pub nominal: NominalLength,
    pub kind: BlockKind,
    pub id: u16,
    pub block_len: u64,
}

/// The blocked partition, and whether Whole-file raw replaced it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub key: PartitionKey,
    pub choices: Vec<Choice>,
    pub whole_file_raw: bool,
    pub file_len: u64,
}

/// `(offset, length code) → (candidate id, encoded Block bytes)`.
pub type SpanTable = BTreeMap<(u64, u8), (u16, u64)>;

#[derive(Clone, Copy)]
struct Node {
    bytes: u64,
    count: u32,
    code: u8,
    id: u16,
    next: u64,
}

/// Raw = 0, Direct = 1, Previous-byte = 2, Constant `c` = `3 + c`.
pub fn candidate_id(kind: BlockKind) -> u16 {
    match kind {
        BlockKind::Raw => 0,
        BlockKind::Direct => 1,
        BlockKind::PreviousByte => 2,
        BlockKind::Constant(byte) => 3u16.saturating_add(u16::from(byte)),
    }
}

/// Raw, Direct, Previous-byte, then Constant 0 through 255.
pub fn all_baselines() -> Vec<BlockKind> {
    let mut kinds = Vec::with_capacity(259);
    kinds.push(BlockKind::Raw);
    kinds.push(BlockKind::Direct);
    kinds.push(BlockKind::PreviousByte);
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

/// `N >= 2^35` is rejected before a search.
pub fn check_length(n: u64) -> Result<(), Error> {
    if n >= LENGTH_LIMIT {
        Err(Error::Kernel(sxp_kernel::Error::LengthOutOfRange))
    } else {
        Ok(())
    }
}

/// Full nominals in the arm, plus the canonical final when `remaining` is
/// small enough. The result is sorted and deduplicated.
pub fn allowed_nominals(remaining: u64, policy: LengthPolicy) -> Vec<NominalLength> {
    if remaining == 0 {
        return Vec::new();
    }
    let set: &[NominalLength] = match policy {
        LengthPolicy::All => &[
            NominalLength::N256,
            NominalLength::N1024,
            NominalLength::N2048,
        ],
        LengthPolicy::Fixed(NominalLength::N256) => &[NominalLength::N256],
        LengthPolicy::Fixed(NominalLength::N1024) => &[NominalLength::N1024],
        LengthPolicy::Fixed(NominalLength::N2048) => &[NominalLength::N2048],
    };
    let mut max_bytes = 0u64;
    for nominal in set {
        max_bytes = max_bytes.max(nominal.bytes());
    }
    let mut out = Vec::new();
    for nominal in set {
        if nominal.bytes() <= remaining {
            out.push(*nominal);
        }
    }
    if remaining <= max_bytes {
        let canonical = canonical_nominal(remaining);
        if !out.contains(&canonical) {
            out.push(canonical);
        }
    }
    out.sort_by_key(|nominal| nominal.bytes());
    out.dedup();
    out
}

fn canonical_nominal(remaining: u64) -> NominalLength {
    if remaining <= 256 {
        NominalLength::N256
    } else if remaining <= 1024 {
        NominalLength::N1024
    } else {
        NominalLength::N2048
    }
}

fn section_len(kind: BlockKind) -> u64 {
    match kind {
        BlockKind::Constant(_) => 2,
        BlockKind::Raw | BlockKind::Direct | BlockKind::PreviousByte => 1,
    }
}

fn nominal_from_code(code: u8) -> Result<NominalLength, Error> {
    match code {
        0 => Ok(NominalLength::N256),
        1 => Ok(NominalLength::N1024),
        2 => Ok(NominalLength::N2048),
        _ => Err(Error::MissingSpan),
    }
}

fn kind_from_id(id: u16) -> Result<BlockKind, Error> {
    match id {
        0 => Ok(BlockKind::Raw),
        1 => Ok(BlockKind::Direct),
        2 => Ok(BlockKind::PreviousByte),
        3..=258 => {
            let byte = u8::try_from(id.saturating_sub(3)).map_err(|_| Error::NoCandidate)?;
            Ok(BlockKind::Constant(byte))
        }
        _ => Err(Error::NoCandidate),
    }
}

fn slice_of(input: &[u8], offset: u64, len: u64) -> Result<&[u8], Error> {
    let start = usize::try_from(offset)
        .map_err(|_| Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
    let width =
        usize::try_from(len).map_err(|_| Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
    let end = start
        .checked_add(width)
        .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
    input
        .get(start..end)
        .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))
}

fn sorted_candidates(candidates: &[BlockKind]) -> Vec<BlockKind> {
    let mut kinds = candidates.to_vec();
    kinds.sort_by_key(|kind| candidate_id(*kind));
    kinds.dedup_by_key(|kind| candidate_id(*kind));
    kinds
}

/// Pick the candidate with the smallest `(exact size, id)`.
///
/// `HeaderPrune` walks ids in ascending order and skips a kind once its
/// section length is greater than or equal to the best exact size so far.
/// The first kind is always costed. A cost error is returned as-is.
#[doc(hidden)]
pub fn choose_candidate(
    candidates: &[BlockKind],
    mode: SearchMode,
    cost: impl Fn(BlockKind) -> Result<u64, sxp_kernel::Error>,
) -> Result<(BlockKind, u16, u64), Error> {
    let kinds = sorted_candidates(candidates);
    let mut best: Option<(BlockKind, u16, u64)> = None;
    for kind in kinds {
        let id = candidate_id(kind);
        if mode == SearchMode::HeaderPrune
            && let Some((_, _, incumbent)) = best
            && section_len(kind) >= incumbent
        {
            continue;
        }
        let bytes = cost(kind).map_err(Error::Kernel)?;
        let replace = match best {
            None => true,
            Some((_, best_id, best_bytes)) => (bytes, id) < (best_bytes, best_id),
        };
        if replace {
            best = Some((kind, id, bytes));
        }
    }
    best.ok_or(Error::NoCandidate)
}

fn spans_from(n: u64, policy: LengthPolicy) -> Result<Vec<(u64, NominalLength)>, Error> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![0u64];
    let mut spans = Vec::new();
    while let Some(offset) = pending.pop() {
        if offset == n || !seen.insert(offset) {
            continue;
        }
        let remaining = n
            .checked_sub(offset)
            .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
        for nominal in allowed_nominals(remaining, policy) {
            spans.push((offset, nominal));
            let next = offset
                .checked_add(actual_len(nominal, remaining))
                .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
            if next < n {
                pending.push(next);
            }
        }
    }
    Ok(spans)
}

fn reachable_desc(n: u64, policy: LengthPolicy) -> Result<Vec<u64>, Error> {
    let spans = spans_from(n, policy)?;
    let mut offsets = BTreeSet::new();
    for (offset, _) in spans {
        offsets.insert(offset);
    }
    let mut desc: Vec<u64> = offsets.into_iter().collect();
    desc.sort_by(|left, right| right.cmp(left));
    Ok(desc)
}

fn node_better(candidate: Node, incumbent: Node) -> bool {
    (
        candidate.bytes,
        candidate.count,
        candidate.code,
        candidate.id,
    ) < (
        incumbent.bytes,
        incumbent.count,
        incumbent.code,
        incumbent.id,
    )
}

/// Backward DP over a finished span table. This function does not encode.
///
/// At one offset each nominal has a distinct length code, so the lexicographic
/// key is that code and then the id. Suffix bytes and suffix block count are
/// still added. A missing allowed span is [`Error::MissingSpan`].
#[doc(hidden)]
pub fn reduce(n: u64, policy: LengthPolicy, spans: &SpanTable) -> Result<PartitionKey, Error> {
    if n == 0 {
        return Ok(PartitionKey {
            block_bytes: 0,
            block_count: 0,
            steps: Vec::new(),
        });
    }
    let mut nodes: BTreeMap<u64, Node> = BTreeMap::new();
    for offset in reachable_desc(n, policy)? {
        let remaining = n
            .checked_sub(offset)
            .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
        let mut winner: Option<Node> = None;
        for nominal in allowed_nominals(remaining, policy) {
            let code = nominal.code();
            let (id, edge_bytes) = spans
                .get(&(offset, code))
                .copied()
                .ok_or(Error::MissingSpan)?;
            let next = offset
                .checked_add(actual_len(nominal, remaining))
                .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
            let (suffix_bytes, suffix_count) = if next == n {
                (0u64, 0u32)
            } else {
                let suffix = nodes.get(&next).ok_or(Error::MissingSpan)?;
                (suffix.bytes, suffix.count)
            };
            let bytes = edge_bytes
                .checked_add(suffix_bytes)
                .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
            let count = suffix_count
                .checked_add(1)
                .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
            let candidate = Node {
                bytes,
                count,
                code,
                id,
                next,
            };
            let replace = match winner {
                None => true,
                Some(incumbent) => node_better(candidate, incumbent),
            };
            if replace {
                winner = Some(candidate);
            }
        }
        nodes.insert(offset, winner.ok_or(Error::MissingSpan)?);
    }

    let mut steps = Vec::new();
    let mut offset = 0u64;
    loop {
        if offset == n {
            break;
        }
        let node = nodes.get(&offset).ok_or(Error::MissingSpan)?;
        steps.push((node.code, node.id));
        let next = node.next;
        if next <= offset {
            return Err(Error::MissingSpan);
        }
        offset = next;
    }
    let root = nodes.get(&0).ok_or(Error::MissingSpan)?;
    Ok(PartitionKey {
        block_bytes: root.bytes,
        block_count: root.count,
        steps,
    })
}

fn choices_from(key: &PartitionKey, spans: &SpanTable, n: u64) -> Result<Vec<Choice>, Error> {
    let mut choices = Vec::new();
    let mut offset = 0u64;
    for (code, id) in &key.steps {
        let (table_id, block_len) = spans
            .get(&(offset, *code))
            .copied()
            .ok_or(Error::MissingSpan)?;
        if table_id != *id {
            return Err(Error::MissingSpan);
        }
        let nominal = nominal_from_code(*code)?;
        let remaining = n
            .checked_sub(offset)
            .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
        let actual = actual_len(nominal, remaining);
        choices.push(Choice {
            nominal,
            kind: kind_from_id(*id)?,
            id: *id,
            block_len,
        });
        offset = offset
            .checked_add(actual)
            .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
    }
    if offset != n {
        return Err(Error::MissingSpan);
    }
    Ok(choices)
}

fn predicted_file_len(n: u64, block_bytes: u64, whole_file_raw: bool) -> Result<u64, Error> {
    let header = encode_whole_file_raw_header(n).map_err(Error::Kernel)?;
    let payload = if whole_file_raw { n } else { block_bytes };
    u64::from(header.len)
        .checked_add(payload)
        .and_then(|sum| sum.checked_add(4))
        .ok_or(Error::Kernel(sxp_kernel::Error::LengthOutOfRange))
}

fn cost_span(
    input: &[u8],
    offset: u64,
    nominal: NominalLength,
    n: u64,
    candidates: &[BlockKind],
    mode: SearchMode,
) -> Result<(u64, u8, u16, u64), Error> {
    let remaining = n
        .checked_sub(offset)
        .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
    let actual = actual_len(nominal, remaining);
    let bytes = slice_of(input, offset, actual)?;
    let (_kind, id, len) = choose_candidate(candidates, mode, |kind| {
        encode_block(kind, nominal, remaining, bytes).and_then(|encoded| {
            u64::try_from(encoded.len()).map_err(|_| sxp_kernel::Error::BlockTooLarge)
        })
    })?;
    Ok((offset, nominal.code(), id, len))
}

fn fill_table(
    input: &[u8],
    candidates: &[BlockKind],
    policy: LengthPolicy,
    mode: SearchMode,
    threads: usize,
    n: u64,
) -> Result<SpanTable, Error> {
    let spans = spans_from(n, policy)?;
    let rows = if threads <= 1 {
        let mut rows = Vec::with_capacity(spans.len());
        for (offset, nominal) in spans {
            rows.push(cost_span(input, offset, nominal, n, candidates, mode)?);
        }
        rows
    } else {
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .map_err(|_| Error::ThreadPool)?;
        pool.install(|| {
            spans
                .par_iter()
                .map(|(offset, nominal)| cost_span(input, *offset, *nominal, n, candidates, mode))
                .collect::<Result<Vec<_>, Error>>()
        })?
    };
    let mut table = SpanTable::new();
    for (offset, code, id, len) in rows {
        table.insert((offset, code), (id, len));
    }
    Ok(table)
}

/// Choose a baseline partition. `threads == 0` is one calling thread.
///
/// The span table is filled first. [`reduce`] then runs on the calling thread.
/// Whole-file raw wins when there are no input bytes, or when the Blocks are
/// not strictly smaller than `N`.
pub fn select_baselines(
    input: &[u8],
    candidates: &[BlockKind],
    policy: LengthPolicy,
    mode: SearchMode,
    threads: usize,
) -> Result<Selection, Error> {
    let n = u64::try_from(input.len())
        .map_err(|_| Error::Kernel(sxp_kernel::Error::LengthOutOfRange))?;
    check_length(n)?;
    if n == 0 {
        return Ok(Selection {
            key: PartitionKey {
                block_bytes: 0,
                block_count: 0,
                steps: Vec::new(),
            },
            choices: Vec::new(),
            whole_file_raw: true,
            file_len: 7,
        });
    }
    let candidates = sorted_candidates(candidates);
    if candidates.is_empty() {
        return Err(Error::NoCandidate);
    }
    let table = fill_table(input, &candidates, policy, mode, threads, n)?;
    let key = reduce(n, policy, &table)?;
    let choices = choices_from(&key, &table, n)?;
    let whole_file_raw = key.block_bytes >= n;
    let file_len = predicted_file_len(n, key.block_bytes, whole_file_raw)?;
    Ok(Selection {
        key,
        choices,
        whole_file_raw,
        file_len,
    })
}

/// Write the selection. Whole-file raw and blocked bytes both come from the kernel.
pub fn encode_baselines(
    input: &[u8],
    candidates: &[BlockKind],
    policy: LengthPolicy,
    mode: SearchMode,
    threads: usize,
) -> Result<Vec<u8>, Error> {
    let selection = select_baselines(input, candidates, policy, mode, threads)?;
    let n = u64::try_from(input.len())
        .map_err(|_| Error::Kernel(sxp_kernel::Error::LengthOutOfRange))?;
    let mut output = Vec::new();
    if selection.whole_file_raw {
        write_whole_file_raw(n, input, &mut output).map_err(Error::Kernel)?;
        return Ok(output);
    }
    let mut ranges = Vec::new();
    let mut offset = 0u64;
    for choice in &selection.choices {
        let remaining = n
            .checked_sub(offset)
            .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
        let actual = actual_len(choice.nominal, remaining);
        ranges.push((choice.kind, choice.nominal, offset, actual));
        offset = offset
            .checked_add(actual)
            .ok_or(Error::Kernel(sxp_kernel::Error::BlockLengthMismatch))?;
    }
    let mut plans = Vec::with_capacity(ranges.len());
    for (kind, nominal, start, len) in &ranges {
        plans.push(BlockPlan {
            kind: *kind,
            nominal: *nominal,
            bytes: slice_of(input, *start, *len)?,
        });
    }
    write_blocked_file(n, &plans, &mut output).map_err(Error::Kernel)?;
    Ok(output)
}
