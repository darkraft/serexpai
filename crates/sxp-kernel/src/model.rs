//! Order-0 probability model from bitstream-spec-seed.md §5.
//!
//! The strawman constants are provisional until D-07. This module calls the
//! range coder. The coder does not know about nodes, and nothing here is
//! written into the coder stream.

use crate::Error;
use crate::range_coder::{RangeDecoder, RangeEncoder};

/// Provisional until D-07, not the frozen contract.
/// Part of the provisional strawman whose RECIP schedule is
/// `floor(65536 / (n + 2))` (the simpler DELTA = 1 schedule).
pub const STRAWMAN_P: u16 = 32_768;

/// Provisional until D-07, not the frozen contract.
/// Part of the provisional strawman whose RECIP schedule is
/// `floor(65536 / (n + 2))` (the simpler DELTA = 1 schedule).
pub const STRAWMAN_CAP: u16 = 24;

/// Provisional until D-07, not the frozen contract.
/// Schedule `floor(65536 / (n + 2))` (the simpler DELTA = 1 schedule).
/// Index is the count `n` before the update. `RECIP[24]` is the saturated rate.
pub const STRAWMAN_RECIP: [u16; 25] = [
    32_768, 21_845, 16_384, 13_107, 10_922, 9_362, 8_192, 7_281, 6_553, 5_957, 5_461, 5_041, 4_681,
    4_369, 4_096, 3_855, 3_640, 3_449, 3_276, 3_120, 2_978, 2_849, 2_730, 2_621, 2_520,
];

/// How a new Block fills `p`. Not a wire field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitialProbability {
    Scalar(u16),
    ByDepth([u16; 8]),
}

/// Owned model configuration. The RECIP table is copied at construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelConfig {
    initial: InitialProbability,
    cap: u16,
    recip: Vec<u16>,
}

impl ModelConfig {
    /// Rejects an initial `p` outside `16..=65520`, a `cap` outside `1..=255`,
    /// a RECIP length other than `cap + 1`, and a RECIP entry of `0`.
    pub fn try_new(initial: InitialProbability, cap: u16, recip: &[u16]) -> Result<Self, Error> {
        match initial {
            InitialProbability::Scalar(p) => {
                if !(16..=65_520).contains(&p) {
                    return Err(Error::InvalidParameter);
                }
            }
            InitialProbability::ByDepth(table) => {
                for p in table {
                    if !(16..=65_520).contains(&p) {
                        return Err(Error::InvalidParameter);
                    }
                }
            }
        }
        if !(1..=255).contains(&cap) {
            return Err(Error::InvalidParameter);
        }
        if recip.len() != usize::from(cap) + 1 {
            return Err(Error::InvalidParameter);
        }
        for rate in recip {
            if *rate == 0 {
                return Err(Error::InvalidParameter);
            }
        }
        Ok(Self {
            initial,
            cap,
            recip: recip.to_vec(),
        })
    }

    /// Scalar `STRAWMAN_P`, `STRAWMAN_CAP`, and `STRAWMAN_RECIP`.
    ///
    /// Provisional until D-07. Not the frozen contract.
    pub fn strawman() -> Result<Self, Error> {
        Self::try_new(
            InitialProbability::Scalar(STRAWMAN_P),
            STRAWMAN_CAP,
            &STRAWMAN_RECIP,
        )
    }
}

/// Trace observation of one node. Not a wire field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeState {
    pub p: u16,
    pub n: u16,
}

/// One coded decision. Not a wire field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecisionTrace {
    pub node: u16,
    pub n_before: u16,
    pub p_before: u16,
    pub freq0: u16,
    pub freq1: u16,
    pub bit: u8,
    pub n_after: u16,
    pub p_after: u16,
    pub coder_range: u32,
}

const EMPTY_TRACE: DecisionTrace = DecisionTrace {
    node: 0,
    n_before: 0,
    p_before: 0,
    freq0: 0,
    freq1: 0,
    bit: 0,
    n_after: 0,
    p_after: 0,
    coder_range: 0,
};

#[derive(Clone, Copy, Debug)]
struct Node {
    p: u16,
    n: u16,
}

/// 255-node adaptive order-0 bit tree. Index 0 is unused storage.
#[derive(Debug)]
pub struct Order0Model {
    nodes: [Node; 256],
    config: ModelConfig,
}

impl Order0Model {
    pub fn new(config: ModelConfig) -> Self {
        let mut model = Self {
            nodes: [Node { p: 0, n: 0 }; 256],
            config,
        };
        model.reset();
        model
    }

    /// Restore every node to its configured initial `p` and `n = 0`.
    pub fn reset(&mut self) {
        write_nodes(&mut self.nodes, self.config.initial);
    }

    pub fn node(&self, id: u16) -> Result<NodeState, Error> {
        let node = self.slot(id)?;
        Ok(NodeState {
            p: node.p,
            n: node.n,
        })
    }

    /// `(freq0, freq1)` for the stored `p`. Does not change state.
    pub fn freq(&self, id: u16) -> Result<(u16, u16), Error> {
        let state = self.node(id)?;
        let shifted = state.p >> 4;
        let freq0 = if shifted == 0 { 1 } else { shifted };
        Ok((freq0, 4096 - freq0))
    }

    /// Apply one bit using `RECIP[n]` from before `n` increases.
    ///
    /// The multiply is `u32`. The product is below `2^32`. There is no clamp.
    pub fn update(&mut self, id: u16, bit: u8) -> Result<(), Error> {
        if bit > 1 {
            return Err(Error::InvalidParameter);
        }
        let before = self.node(id)?;
        let rate = self
            .config
            .recip
            .get(usize::from(before.n))
            .copied()
            .ok_or(Error::InvalidParameter)?;
        let p = next_p(before.p, bit, rate)?;
        let n = match before.n.checked_add(1) {
            Some(next) => next.min(self.config.cap),
            None => self.config.cap,
        };
        let slot = self.slot_mut(id)?;
        *slot = Node { p, n };
        Ok(())
    }

    /// Code one byte, most significant bit first.
    ///
    /// The order is frequency, then the coder, then one model update. `p` uses
    /// `u32`. The product `(65536 - p) * R` is below `2^32`. There is no clamp
    /// and no pending symbol. An error returns immediately. Decisions already
    /// taken in this symbol stay updated. There is no rollback.
    pub fn encode_symbol(
        &mut self,
        encoder: &mut RangeEncoder,
        symbol: u8,
    ) -> Result<[DecisionTrace; 8], Error> {
        let mut traces = [EMPTY_TRACE; 8];
        let mut node = 1u16;
        for depth in 0..8 {
            let bit = (symbol >> (7 - depth)) & 1;
            let (freq0, freq1) = self.freq(node)?;
            let before = self.node(node)?;
            encoder.encode(bit, freq0)?;
            let coder_range = encoder.state().range;
            self.update(node, bit)?;
            let after = self.node(node)?;
            let trace = DecisionTrace {
                node,
                n_before: before.n,
                p_before: before.p,
                freq0,
                freq1,
                bit,
                n_after: after.n,
                p_after: after.p,
                coder_range,
            };
            *traces.get_mut(depth).ok_or(Error::InvalidParameter)? = trace;
            if depth + 1 < 8 {
                node = next_id(node, bit)?;
            }
        }
        Ok(traces)
    }

    /// Decode one byte, most significant bit first.
    ///
    /// The order is frequency, then the coder, then one model update. `p` uses
    /// `u32`. The product `(65536 - p) * R` is below `2^32`. There is no clamp
    /// and no pending symbol. There is no branch that skips a decision. An
    /// error returns immediately, and decisions already taken in this symbol
    /// stay updated. There is no rollback.
    pub fn decode_symbol(
        &mut self,
        decoder: &mut RangeDecoder<'_>,
    ) -> Result<(u8, [DecisionTrace; 8]), Error> {
        let mut traces = [EMPTY_TRACE; 8];
        let mut node = 1u16;
        let mut symbol = 0u8;
        for depth in 0..8 {
            let (freq0, freq1) = self.freq(node)?;
            let before = self.node(node)?;
            let bit = decoder.decode(freq0)?;
            let coder_range = decoder.state().range;
            self.update(node, bit)?;
            let after = self.node(node)?;
            symbol = (symbol << 1) | bit;
            let trace = DecisionTrace {
                node,
                n_before: before.n,
                p_before: before.p,
                freq0,
                freq1,
                bit,
                n_after: after.n,
                p_after: after.p,
                coder_range,
            };
            *traces.get_mut(depth).ok_or(Error::InvalidParameter)? = trace;
            if depth + 1 < 8 {
                node = next_id(node, bit)?;
            }
        }
        Ok((symbol, traces))
    }

    fn slot(&self, id: u16) -> Result<&Node, Error> {
        if id == 0 || id > 255 {
            return Err(Error::InvalidParameter);
        }
        self.nodes
            .get(usize::from(id))
            .ok_or(Error::InvalidParameter)
    }

    fn slot_mut(&mut self, id: u16) -> Result<&mut Node, Error> {
        if id == 0 || id > 255 {
            return Err(Error::InvalidParameter);
        }
        self.nodes
            .get_mut(usize::from(id))
            .ok_or(Error::InvalidParameter)
    }
}

fn write_nodes(nodes: &mut [Node; 256], initial: InitialProbability) {
    match initial {
        InitialProbability::Scalar(p) => {
            for id in 1usize..=255 {
                if let Some(slot) = nodes.get_mut(id) {
                    *slot = Node { p, n: 0 };
                }
            }
        }
        InitialProbability::ByDepth(table) => {
            for depth in 0usize..8 {
                let Some(p) = table.get(depth).copied() else {
                    continue;
                };
                let start = 1usize << depth;
                let end = (start << 1).min(256);
                for id in start..end {
                    if let Some(slot) = nodes.get_mut(id) {
                        *slot = Node { p, n: 0 };
                    }
                }
            }
        }
    }
}

/// `u32` update. Legal `R` keeps the product below `2^32` and `p` in `1..=65535`.
fn next_p(p: u16, bit: u8, rate: u16) -> Result<u16, Error> {
    let p32 = u32::from(p);
    let r = u32::from(rate);
    let updated = if bit == 0 {
        let delta = ((65_536 - p32) * r) >> 16;
        p32 + delta
    } else {
        let delta = (p32 * r) >> 16;
        p32 - delta
    };
    let stored = u16::try_from(updated).map_err(|_| Error::InvalidParameter)?;
    if !(1..=65_535).contains(&stored) {
        return Err(Error::InvalidParameter);
    }
    Ok(stored)
}

fn next_id(node: u16, bit: u8) -> Result<u16, Error> {
    node.checked_mul(2)
        .and_then(|doubled| doubled.checked_add(u16::from(bit)))
        .ok_or(Error::InvalidParameter)
}
