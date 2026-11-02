---
baseline_commit: d06ecd694521da163610b852050edde371c4bfb0
---

# Story 1.7: Adapt an order-0 probability model

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want one causal model reconstructed from already decoded decisions,
so that a Block can code each byte as eight binary decisions and rebuild the same frequencies from the bits already decided, without transmitting a probability table.

## Acceptance Criteria

Stable IDs from the epic breakdown. A test or the committed hash must be traceable to each ID. This story is not done until x86_64 and aarch64 evidence both exist. Native evidence is enough to start the next story. It does not close AC06.

1. **S1.7-AC01.** G0 already specifies the model in `bitstream-spec-seed.md` §5. Do not edit the seed, and do not add a second normative document. The tree has 255 nodes, ids `1..=255`. Each symbol is eight decisions, most significant bit first. A node holds `p` (`u16`, P(bit 0) scaled by 65536) and `n` (`u16`, saturating count). A new model and `reset` set every node to `n = 0` and to the configured initial `p`. The rate used for a decision is `RECIP[n]` before `n` increases. The strawman is a scalar `p = 32768`, `CAP = 24`, and `RECIP[n] = floor(65536 / (n + 2))` for `n = 0..=24`. Those three are provisional until D-07 closes. They are `pub` constants, labelled provisional in rustdoc, and the 25 little-endian `u16` RECIP bytes are hashed.
2. **S1.7-AC02.** `reset` restores every node, including nodes that were updated. Each symbol walks from node 1; after a decision `b` at node `m` the next id is `2m + b`. The id is a function of the depth and the bits already decided, never of a byte index and never of a bit that has not been decided yet. `encode_symbol` may read the symbol only as the source of the current bit. No probability table, model id, or initial-`p` field is written to the coder stream.
3. **S1.7-AC03.** Bit 0 sets `p = p + (((65536 - p) * R) >> 16)`. Bit 1 sets `p = p - ((p * R) >> 16)`. The multiply is `u32`. `R` is in `1..=65535`, so the product is at most `65535 * 65535 = 4_294_836_225`, which is below `2^32`. Do not use `i32`, a signed shift, `wrapping_mul`, or a clamp. Legal `p` stays in `1..=65535`. `freq0 = max(p >> 4, 1)` and `freq1 = 4096 - freq0`, both in `1..=4095`. `try_new` rejects an initial `p` outside `16..=65520`, a `cap` outside `1..=255`, a RECIP length other than `cap + 1`, and a RECIP entry of `0`.
4. **S1.7-AC04.** The fixtures below are the committed schedules. Fresh node, the CAP boundary (updates 23, 24, and 25), and the zero and one runs of 132, 133, 140, and 141 decisions are exact `(p, n, freq0, freq1)` rows. The RECIP hash is `spec/vectors/model-recip-strawman.sha256`. The zero run ends at `freq0 = 4094`, not 4095. The one run ends at `freq1 = 4095` and `p = 26`, not `p = 1`.
5. **S1.7-AC05.** For symbol `0xA5` the encoder and decoder traces agree on node, `n` before, `p` before, `freq0`, `freq1`, bit, `n` after, `p` after, and coder `range`. The order inside each decision is `freq`, then `RangeEncoder::encode` or `RangeDecoder::decode`, then one `update`. After `encode_symbol` returns, `freq` already observes the updated node. There is no pending-symbol field, no deferred-update flag, and no test that skips a decode.
6. **S1.7-AC06.** Scalar strawman and one per-depth fixture are both tested. Symbols `0..=255` visit every id `1..=255`. `reset` makes a following symbol match a fresh model. The 256-symbol stream hash matches on the tested target. Correctness tests do not score a tuning corpus and do not pick a new `CAP` or a new RECIP schedule. Both platforms match the committed hashes.

## Tasks / Subtasks

- [x] **Configuration** (AC: 1, 3)
  - [x] Add `crates/sxp-kernel/src/model.rs`. Do not edit `range_coder.rs`. The coder stays a caller-supplied `(bit, freq0)` machine.
  - [x] `pub enum InitialProbability { Scalar(u16), ByDepth([u16; 8]) }`. Derive `Clone, Copy, Debug, PartialEq, Eq`.
  - [x] `pub struct ModelConfig` with private initial probability, `cap: u16`, and an owned `Vec<u16>` copy of RECIP. Derive `Clone, Debug, PartialEq, Eq`.
  - [x] `pub fn try_new(initial: InitialProbability, cap: u16, recip: &[u16]) -> Result<Self, Error>`. Reject, with `Error::InvalidParameter` and no partial value, when: a scalar `p` is outside `16..=65520`; any depth entry is outside `16..=65520`; `cap` is `0` or greater than `255`; `recip.len() != usize::from(cap) + 1`; any RECIP entry is `0`. `u16` cannot hold `65536`, so that bound is not a separate input. `65535` is a legal entry. `16` and `65520` are legal initial probabilities.
  - [x] `pub const STRAWMAN_P: u16 = 32768`, `pub const STRAWMAN_CAP: u16 = 24`, `pub const STRAWMAN_RECIP: [u16; 25]` equal to the table in Dev Notes. Rustdoc on all three: provisional until D-07, not the frozen contract, schedule `floor(65536 / (n + 2))` (the simpler DELTA = 1 schedule).
  - [x] `ModelConfig::strawman() -> Result<Self, Error>` is `try_new(InitialProbability::Scalar(STRAWMAN_P), STRAWMAN_CAP, &STRAWMAN_RECIP)`.
  - [x] No `unsafe`. No new dependency. No float. No `i32`. No `build.rs`. No `wrapping_*` on `p` or `R`.
- [x] **Nodes** (AC: 1, 2, 3)
  - [x] `pub struct NodeState { pub p: u16, pub n: u16 }`. Derive `Clone, Copy, Debug, PartialEq, Eq`. Rustdoc: trace observation, not a wire field.
  - [x] `pub struct Order0Model`. Private `nodes: [Node; 256]` and the owned config. Index 0 is unused storage. Derive `Debug` only.
  - [x] `pub fn new(config: ModelConfig) -> Self` and `pub fn reset(&mut self)`. Both write all 255 nodes. Scalar: every node gets that `p` and `n = 0`. ByDepth: node `id` gets `table[id.ilog2() as usize]` and `n = 0`. `1u16.ilog2()` is 0 and `255u16.ilog2()` is 7. Read the table with `get`, not `[]`. Index 0 is left `p = 0`, `n = 0` and is never a successful `node()` result.
  - [x] `pub fn node(&self, id: u16) -> Result<NodeState, Error>`, `pub fn freq(&self, id: u16) -> Result<(u16, u16), Error>`, `pub fn update(&mut self, id: u16, bit: u8) -> Result<(), Error>`. `id == 0` or `id > 255` is `Error::InvalidParameter` before any write. `bit > 1` is `Error::InvalidParameter` and leaves that node unchanged. `freq` returns `(freq0, freq1)` and does not change state.
  - [x] `update` reads `R` from `recip[n]` while `n` is still the stored count, then writes `p`, then writes `n = min(n + 1, cap)`. Use `get` on the RECIP vec. The `u32` product fits; convert the next `p` with `u16::try_from`. `Ok` only when the value is in `1..=65535`. Do not clamp to `1` or `65535`. Do not clamp to `[16, 4080]`.
  - [x] `freq0` is `let shifted = u16::from(p >> 4); if shifted == 0 { 1 } else { shifted }`. `freq1 = 4096 - freq0`. This is the only use of `max` with 1. A stored `p` of `1..=15` yields `freq0 = 1` without changing `p`.
- [x] **Symbol walk** (AC: 2, 5)
  - [x] `pub struct DecisionTrace` with `node`, `n_before`, `p_before`, `freq0`, `freq1`, `bit`, `n_after`, `p_after`, `coder_range: u32`. Derive `Clone, Copy, Debug, PartialEq, Eq`. Not a wire field.
  - [x] `pub fn encode_symbol(&mut self, encoder: &mut RangeEncoder, symbol: u8) -> Result<[DecisionTrace; 8], Error>`. For depth `0..8`: the bit is `(symbol >> (7 - depth)) & 1`; the node starts at 1; call `freq` then `node` (both unchanged by those calls); call `encoder.encode(bit, freq0)`; read `encoder.state().range`; call `update` once; record the after state. Only then set the next id to `2 * node + bit` for the following depth. Do not read that next id after depth 7. Do not index `nodes` by `symbol`.
  - [x] `pub fn decode_symbol(&mut self, decoder: &mut RangeDecoder) -> Result<(u8, [DecisionTrace; 8]), Error>`. Same order with `decoder.decode(freq0)`. The returned byte is the eight bits, most significant bit first. There is no symbol argument. Always eight `decode` calls. No branch that skips a decision.
  - [x] Build the `[DecisionTrace; 8]` with `get_mut`. `src/` denies `indexing_slicing`.
  - [x] Rustdoc on `encode_symbol` and on `decode_symbol`, not only on the struct: the order is frequency, then coder, then one model update; `p` uses `u32`; the product is below `2^32`; there is no clamp and no pending symbol.
  - [x] Do not add a field other than `p` and `n` on a node. Do not snapshot the model to roll back a later coder error. An `encode` error returns before `update` for that decision; earlier decisions in the same symbol stay updated.
- [x] **Exports** (AC: 1)
  - [x] In `lib.rs` add `mod model;` and re-export `InitialProbability`, `ModelConfig`, `Order0Model`, `NodeState`, `DecisionTrace`, `STRAWMAN_P`, `STRAWMAN_CAP`, and `STRAWMAN_RECIP`.
  - [x] Extend the module-doc sentence so it also names the order-0 probability model. The G0 hash stays `f4bea1e5b729db1df957563fd3e8e5bec70c0ab0a382f0f0afc7f9e82196e12e`. Do not edit the lint block. Do not add an `Error` variant. Do not edit `error.rs`.
  - [x] Do not edit `tests/support/mod.rs` or `spec/vectors/index.toml`. The support loader asserts 24 rows.
- [x] **Update fixtures** (AC: 3, 4)
  - [x] Add `crates/sxp-kernel/tests/model.rs`. Do not `mod support`. Use `Order0Model::new(ModelConfig::strawman()?)` or the test's `Result` via a helper that returns `Error`. Tests may use `assert`.
  - [x] Fresh node 1, before any update: `p = 32768`, `n = 0`, `freq = (2048, 2048)`.
  - [x] One `update(1, 0)`: `p = 49152`, `n = 1`, `freq = (3072, 1024)`. `R` was `32768`, not `21845`. `p = 43690` is the increment-first result and must not appear.
  - [x] A second fresh model, one `update(1, 1)`: `p = 16384`, `n = 1`, `freq = (1024, 3072)`.
  - [x] Drive node 1 with bit 0 this many times, then assert `(p, n, freq0, freq1)`:

    | updates | p | n | freq0 | freq1 |
    |---:|---:|---:|---:|---:|
    | 23 | 64165 | 23 | 4010 | 86 |
    | 24 | 64219 | 24 | 4013 | 83 |
    | 25 | 64269 | 24 | 4016 | 80 |
    | 132 | 65503 | 24 | 4093 | 3 |
    | 133 | 65504 | 24 | 4094 | 2 |
    | 140 | 65510 | 24 | 4094 | 2 |
    | 141 | 65510 | 24 | 4094 | 2 |

  - [x] The same counts with bit 1:

    | updates | p | n | freq0 | freq1 |
    |---:|---:|---:|---:|---:|
    | 23 | 1371 | 23 | 85 | 4011 |
    | 24 | 1317 | 24 | 82 | 4014 |
    | 25 | 1267 | 24 | 79 | 4017 |
    | 133 | 32 | 24 | 2 | 4094 |
    | 134 | 31 | 24 | 1 | 4095 |
    | 140 | 26 | 24 | 1 | 4095 |
    | 141 | 26 | 24 | 1 | 4095 |

  - [x] Update 24 is the one that stores `n = 24` and used `RECIP[23] = 2621`. Update 25 keeps `n = 24` and used `RECIP[24] = 2520`. Assert the `p` column. Do not accept `p = 64271` at update 25 (that is `RECIP[23]` used again).
  - [x] After 141 zeros, `freq0` is 4094. After 141 ones, `freq1` is 4095 and `p` is 26. Do not assert `freq0 == 4095` on the zero run or `p == 1` on the one run.
- [x] **RECIP hash** (AC: 1, 4)
  - [x] Commit `spec/vectors/model-recip-strawman.sha256` containing `a83498752cd008facb7ae8b1e689101b49fb3a36d98ab65a63a6e37a0e1dfd20` and one trailing newline. Load it with `include_str!(...).trim()`. Do not add a `[[vector]]` row.
  - [x] Hash the 50 bytes formed by `STRAWMAN_RECIP` in order, each `to_le_bytes()`. `STRAWMAN_RECIP[0] = 32768`, `[23] = 2621`, `[24] = 2520`, length 25. SHA-256 matches the file. Hex with `write!(s, "{b:02x}")` and `sha2 = "=0.11.1"`.
- [x] **Symbol 0xA5 trace** (AC: 2, 5)
  - [x] One fresh strawman model and one `RangeEncoder`. `encode_symbol(0xA5)` visits, in order, nodes `1, 3, 6, 13, 26, 52, 105, 210` with bits `1, 0, 1, 0, 0, 1, 0, 1`. Every row has `n_before = 0`, `p_before = 32768`, `freq0 = 2048`, `freq1 = 2048`, `n_after = 1`. Bit 0 rows have `p_after = 49152`. Bit 1 rows have `p_after = 16384`.
  - [x] `coder_range` after each decision, before `finish`: `0x8000_07FF`, `0x4000_0000`, `0x2000_0000`, `0x1000_0000`, `0x0800_0000`, `0x0400_0000`, `0x0200_0000`, `0x0100_0000`.
  - [x] `finish` is `A4 FF F8 00`. A fresh `RangeDecoder` on those bytes and a fresh model: `decode_symbol` returns `0xA5` and the same eight traces, including `coder_range`. `finish` is `Ok` and `bytes_consumed()` is 4.
  - [x] After the encoder symbol, node 2 and node 255 are still `p = 32768`, `n = 0`. Nodes 1 and 210 are `p = 16384`, `n = 1`. Node 3 is `p = 49152`, `n = 1`.
  - [x] `freq(1)` after `encode_symbol` returns `(1024, 3072)` without a further update. The update is not waiting for the next symbol.
- [x] **Paths, depth, reset** (AC: 2, 6)
  - [x] Encode symbols `0..=255` on one fresh model and record every trace `node`. The set is exactly `1..=255`.
  - [x] Symbol `0x00` visits `1, 2, 4, 8, 16, 32, 64, 128`, all bit 0. Symbol `0xFF` visits `1, 3, 7, 15, 31, 63, 127, 255`, all bit 1. After `0xFF` alone, node 255 is `p = 16384`, `n = 1`, and node 2 is still `p = 32768`, `n = 0`.
  - [x] `ByDepth([16, 32, 48, 64, 80, 96, 112, 128])` with `STRAWMAN_CAP` and `STRAWMAN_RECIP`. Before any update, `node` and `freq` for ids `1, 2, 4, 8, 16, 32, 64, 128` are `p = 16, 32, 48, 64, 80, 96, 112, 128` and `freq0 = 1, 2, 3, 4, 5, 6, 7, 8`, each with `n = 0`. `encode_symbol(0x00)` on that model has first-row `p_before = 16`, `freq0 = 1`, bit 0, `p_after = 32776`, `n_after = 1`, `coder_range = 0x0FFF_FF00`. A scalar strawman would have reported `freq0 = 2048` here. This config is not `ModelConfig::strawman`.
  - [x] Encode `0xFF` on a strawman model, call `reset`, then encode `0xA5`. The bytes match a second model that only encoded `0xA5`. Node 255 is back to `p = 32768`, `n = 0` after `reset` and stays there, because `0xA5` does not visit it.
  - [x] Two `encode_symbol(0x00)` calls on one model. After the first, node 1 is `p = 49152`, `n = 1`, and `freq(1)` is `(3072, 1024)`. The second symbol's first trace row is node 1, `n_before = 1`, `p_before = 49152`, `freq0 = 3072`, bit 0, `n_after = 2`, `p_after = 54613`, `coder_range = 0xBFFA_0000`. Both symbols together finish as `00 00 00 00 00`, the same bytes as one `0x00`. Do not use the length to prove the second update. The node state is the proof. `decode_symbol` twice on that buffer returns `0x00` then `0x00`, and the second first-row matches.
- [x] **Rejection** (AC: 3)
  - [x] `try_new` rejects scalar `0`, `15`, `65521`, and `65535`. It accepts `16` and `65520`.
  - [x] `try_new` rejects `ByDepth` when any one of the eight entries is `15` or `65521`, including when the other seven are `32768`.
  - [x] `cap = 0` is rejected. `cap = 256` is rejected. `cap = 24` with 24 entries is rejected. `cap = 24` with 26 entries is rejected. A RECIP entry of `0` is rejected. `cap = 1` with `[1, 65535]` is accepted.
  - [x] `node(0)`, `freq(0)`, `update(0, 0)`, `node(256)`, and `update(1, 2)` are `Error::InvalidParameter`. After `update(1, 2)` on a fresh model, node 1 is still `p = 32768`, `n = 0`. `update(1, 0)` after that rejected call yields `p = 49152`, `n = 1`.
- [x] **256-symbol hash** (AC: 6)
  - [x] Commit `spec/vectors/model-symbols-256.sha256` containing `36c5d0b28ec06cee114b8b9f328929cd9abcdcd5bdf450e56504f42fb5d71e9a` and one trailing newline.
  - [x] One strawman model, one `RangeEncoder`, symbols `0..=255` in order, no `reset` between them. `finish` has length 257. The first eight bytes are `00 1C 1C 0B 1F 9A 8A 7D`. The last eight bytes are `1F 9B 58 5D 2B 41 53 00`. SHA-256 of the whole stream matches the file. A fresh model and `RangeDecoder` return those 256 bytes and `bytes_consumed()` is 257. `finish` is `Ok`.
  - [x] Do not regenerate the file from a broken model to make the test pass. The script in Dev Notes is the authority if the file and the test disagree.
- [x] **Property tests** (AC: 3, 6)
  - [x] `proptest = "=1.12.0"`, `default-features = false`. Config: `cases: 256`, `rng_seed: RngSeed::Fixed(0x5111_0007)`, `failure_persistence: Some(Box::new(FileFailurePersistence::Off))`, remaining fields from `Config::default()`. Do not change seeds `0x5111_0001` through `0x5111_0006`. No regex strategies.
  - [x] Draw `0..=48` bytes. Encode them with one strawman model and one `RangeEncoder`. Decode with a fresh model. The bytes match, `bytes_consumed()` equals the buffer length, and `finish` is `Ok`. After every `update` in a parallel single-node walk of 256 random bits, `p` is in `1..=65535`, `n` is `<= STRAWMAN_CAP`, and `freq0` is in `1..=4095`.
- [x] **Both platforms** (AC: 6)
  - [x] Run `./ci.sh x86_64-unknown-linux-gnu` and `./ci.sh aarch64-unknown-linux-gnu`. Record both commands, `git rev-parse HEAD`, and that `model-recip-strawman` and `model-symbols-256` matched. Existing range-coder, recurrence, trigonometric, polynomial, residual, and whole-file raw tests must still pass.
  - [x] `aarch64-unknown-linux-gnu` links with `aarch64-linux-gnu-gcc` and runs under `qemu-aarch64` (`.cargo/config.toml`). If those are not on `PATH`, put `/tmp/serexpai-cross/bin` ahead of `PATH`. Do not edit `.cargo/config.toml`.
  - [x] Leave this subtask unchecked if either target did not run. Do not set the story Status to `done`. `dev-story` sets `review`. Code review sets `done` only when both runs are recorded.
  - [x] Do not edit `bitstream-spec-seed.md`, `CHANGELOG.md`, the workspace version, `range_coder.rs`, `polynomial.rs`, `trigonometric.rs`, `recurrence.rs`, `residual.rs`, `raw.rs`, or story 1.1–1.6 tests. Do not commit unless the user asks.

### Review Findings

- [x] [Review][Patch] Strawman `p` and `CAP` docs name the RECIP schedule as if it were those constants [crates/sxp-kernel/src/model.rs:10]
- [x] [Review][Patch] Decode rustdoc says eight calls even when an error returns early [crates/sxp-kernel/src/model.rs:233]
- [x] [Review][Patch] The `freq0` floor for `p` below 16 is untested [crates/sxp-kernel/src/model.rs:162]
- [x] [Review][Patch] Per-depth test only reads the power-of-two spine [crates/sxp-kernel/tests/model.rs:213]
- [x] [Review][Patch] Reset test does not read nodes that `0xFF` dirties and `0xA5` skips [crates/sxp-kernel/tests/model.rs:240]
- [x] [Review][Patch] `cap = 256` is rejected by the length check as well as the cap check [crates/sxp-kernel/tests/model.rs:342]

## Dev Notes

### What this story builds

One adaptive order-0 bit model in `sxp-kernel`. It owns `p` and `n` for 255 nodes, turns a byte into eight `freq0` values, and updates each node only after the range coder has consumed that decision. The range coder's signatures stay as story 1.6 left them.

Out of scope: `encode_block` and Block headers (story 1.8); choosing `CAP`, the RECIP schedule, or a per-depth table by measuring tuning inputs (D-07, still open); rANS (seed Appendix A); a Python reference decoder (amendment 6, G4); `spec/bitstream.md` (story 2.1). Do not freeze the strawman. Do not score Baselines-only. Do not add a context model, a 256-slot byte model, or SIMD.

The withdrawn 12-bit rule is not this update. Do not implement `p += ((target - p) * R) >> 16` and do not clamp to `[16, 4080]`. That rule stalled P(0) at 4070/4096.

### Procedure

Governing text is seed §5, which elaborates AD-7. Nothing in this section is written to the stream.

```
node ids 1..=255
decision 0 uses node 1
after bit b at node m, the next node is 2*m + b
eight decisions, MSB first

R = RECIP[n]          # n before the increment
if bit == 0: p = p + (((65536 - p) * R) >> 16)
else:        p = p - ((p * R) >> 16)
n = min(n + 1, CAP)

freq0 = max(p >> 4, 1)
freq1 = 4096 - freq0
```

Per decision, with the existing coder: take `freq0`, encode or decode the bit (that call updates coder state and refills), then apply the update once. A second Block is `reset`, not coder state carried over. This story does not frame the Block. The caller calls `reset`.

`ModelConfig::strawman` RECIP, index = `n`:

```
32768 21845 16384 13107 10922
 9362  8192  7281  6553  5957
 5461  5041  4681  4369  4096
 3855  3640  3449  3276  3120
 2978  2849  2730  2621  2520
```

`RECIP[0] = 32768` is the fresh-node rate. `RECIP[24] = 2520` is the saturated rate. The other candidate schedule, `floor(131072 / (2n + 3))`, is not implemented.

Initial `p` outside `16..=65520` is a rejected configuration. After a legal update, `p` may lie in `1..=15` or `65521..=65535`. Those values stay stored. `freq0` is then 1 or 4095. The zero-run stall is `p = 65510` (`freq0 = 4094`). The one-run stall is `p = 26` (`freq0 = 1`, `freq1 = 4095`).

### 256-symbol oracle

Run from the repo root only if the Rust digest disagrees with the pin. Stdout is the digest. Do not copy this script into the kernel.

```python
import hashlib

CAP = 24
RECIP = [65536 // (n + 2) for n in range(CAP + 1)]

def update(p, n, bit):
    R = RECIP[n]
    if bit == 0:
        p = p + (((65536 - p) * R) >> 16)
    else:
        p = p - ((p * R) >> 16)
    return p, min(n + 1, CAP)

def encode(symbols):
    nodes = {i: [32768, 0] for i in range(1, 256)}
    low, rng, cache, cache_size, out = 0, 0xFFFFFFFF, 0, 1, []
    def shift_low():
        nonlocal low, cache, cache_size
        if low < 0xFF000000 or low >= 2**32:
            carry = low >> 32
            out.append((cache + carry) & 0xFF)
            for _ in range(cache_size - 1):
                out.append((0xFF + carry) & 0xFF)
            cache = (low >> 24) & 0xFF
            cache_size = 0
        cache_size += 1
        low = (low & 0x00FFFFFF) << 8
    for symbol in symbols:
        node = 1
        for k in range(8):
            bit = (symbol >> (7 - k)) & 1
            p, n = nodes[node]
            freq0 = max(p >> 4, 1)
            bound = (rng >> 12) * freq0
            if bit == 0:
                rng = bound
            else:
                low += bound
                rng -= bound
            while rng < 2**24:
                rng = (rng << 8) & 0xFFFFFFFF
                shift_low()
            nodes[node] = list(update(p, n, bit))
            node = 2 * node + bit
    for _ in range(5):
        shift_low()
    assert out[0] == 0
    return bytes(out[1:])

raw = encode(list(range(256)))
assert len(raw) == 257
assert raw[:8].hex() == "001c1c0b1f9a8a7d"
assert raw[-8:].hex() == "1f9b585d2b415300"
print(hashlib.sha256(raw).hexdigest())
```

The same `shift_low` as story 1.6 produces `A4 FF F8 00` for the single symbol `0xA5`.

### Kernel shape to preserve

`lib.rs` forbids unsafe code and denies `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`, and `unreachable` on `src/`. Tests may use `assert`. Read tables with `get`.

Public surface that must keep its signatures: `read_whole_file_raw`, `write_whole_file_raw`, `encode_whole_file_raw_header`, `decode_header`, `Header`, `HeaderBytes`, `encode_leb128`, `decode_leb128`, `Leb128`, `LENGTH_LIMIT`, `Crc32`, `crc32_iso_hdlc`, `signed_residual`, `residual_symbol`, `reconstruct`, `direct_symbol`, `Polynomial`, `Trigonometric`, `check_sine_lut`, `Recurrence`, `RangeEncoder`, `RangeDecoder`, `EncoderState`, `DecoderState`, `encode_schedule`, `decode_schedule`, `exact_size`, and every existing `Error` variant plus `Error::name`.

`sxp-kernel` has an empty `[dependencies]` table. `ci.sh` fails unless `cargo tree -e normal` and `-e build` for `sxp-kernel` are each one line, and unless `crates/sxp-kernel/build.rs` is absent. `sha2` and `proptest` are already dev-dependencies.

`Error::Display` is the variant name only. Domain rejection stays `Error::InvalidParameter`.

### Project Structure Notes

- Model: `crates/sxp-kernel/src/model.rs`. It may call `range_coder`. `range_coder` must not call the model.
- Export: `crates/sxp-kernel/src/lib.rs`, one `mod` and the `pub use` list above.
- Tests: `crates/sxp-kernel/tests/model.rs`.
- Hashes: `spec/vectors/model-recip-strawman.sha256`, `spec/vectors/model-symbols-256.sha256`.
- Do not put the model in `sxp-encoder`. Do not fold it into `range_coder.rs`.

### Previous story intelligence

Story 1.6 put the range coder in its own module, exported it with `pub use`, and tested it from `tests/` without `mod support`. The hash artifact is a one-line SHA-256 file, not an `index.toml` row. `Error::InvalidParameter` is the domain rejection. Integration tests own `proptest` and `sha2`. Story 1.6 used proptest seed `0x5111_0006`. The next free seed is `0x5111_0007`.

Story 1.6's review patches were a truncated refill left usable, and required rustdoc sitting on the struct instead of on `encode`. Put the order, the `u32` product, and the no-clamp rule on `encode_symbol` and `decode_symbol`. Reject `cap` 0 and RECIP entry 0, the values that fit in the parameter types. Prefer the pinned `p` rows over a round-trip that would also pass for the withdrawn 12-bit update.

`encode` and `decode` still take a caller-supplied `freq0`. Do not add a model argument to them. Do not call `residual_symbol`, `Polynomial`, `Trigonometric`, or `Recurrence` from the model.

### Git intelligence

`HEAD` is `d06ecd694521da163610b852050edde371c4bfb0`, `feat: code decisions with the binary range coder (v0.0.12)`. The coder module exists and is the callee. Recent titles:

- `d06ecd6` feat: code decisions with the binary range coder (v0.0.12)
- `aa2fb8f` feat: evaluate a recurrence formula (v0.0.11)
- `277f5a1` feat: evaluate a trigonometric formula (v0.0.10)
- `09f0f51` feat: evaluate a polynomial formula (v0.0.9)
- `fb5a3fd` feat: round-trip whole-file raw and reconstruct predictions (v0.0.8)

### Latest technical information

Stay on the pinned toolchain `1.99.1` (`rust-toolchain.toml`). Do not `rustup update`. `u16::ilog2` is the depth. The multiply `(65536 - p) * R` is exact in `u32` for every legal pair: the maximum product is `65535 * 65535 = 4_294_836_225`, and `u32::MAX` is `4_294_967_295`. A wider integer is unnecessary. `i32` overflows that product. Do not add a crate. `proptest 1.12.0` and `sha2 0.11.1` are already pinned.

### Project context reference

No `project-context.md` exists in this tree. The kernel lint block in `lib.rs` and AD-7 in the architecture spine are the coding standard.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md` — Story 1.7, FR-8, D-07, acceptance evidence]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md` — §5, §6 reference decode order]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md` — AD-7]
- [Source: `_bmad-output/implementation-artifacts/1-6-code-decisions-with-the-binary-range-coder.md` — coder API, hash artifact, proptest profile, both-platform gate]
- [Source: `docs/decision_logs/decision_log_2026_W44.md` — D-07 left unfrozen, 2026-11-01]
- [Source: `crates/sxp-kernel/src/lib.rs` — lint block and G0 hash]
- [Source: `crates/sxp-kernel/src/range_coder.rs` — `encode`, `decode`, `state`]
- [Source: `crates/sxp-kernel/src/error.rs` — `Error::InvalidParameter`]

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

- Clippy `needless_range_loop` on the node-coverage assertion. The test now walks `seen.iter().enumerate().skip(1)`.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Added `Order0Model` in `sxp-kernel`. 255 nodes, MSB-first walk, `u32` update, `freq0 = max(p >> 4, 1)`, strawman `p = 32768`, `CAP = 24`, `RECIP[n] = floor(65536 / (n + 2))`. The strawman stays provisional under D-07.
- `encode_symbol` and `decode_symbol` take `freq`, call the existing range coder, then update the node once. `range_coder.rs` was not edited.
- Pinned fresh-node, CAP-boundary, 141-step zero and one schedules, symbol `0xA5` traces, per-depth init, reset, and the two SHA-256 files.
- `./ci.sh x86_64-unknown-linux-gnu` passed on 2026-11-02. `git rev-parse HEAD` was `d06ecd694521da163610b852050edde371c4bfb0`. `model-recip-strawman` and `model-symbols-256` matched. Existing range-coder, recurrence, trigonometric, polynomial, residual, and whole-file raw tests passed.
- `./ci.sh aarch64-unknown-linux-gnu` passed on the same date and the same `HEAD`, under `qemu-aarch64` with `/tmp/serexpai-cross/bin` on `PATH`. The same two hashes matched.
- Code review applied 6 patches. Seven findings were dismissed. Nothing was deferred. After those patches, `./ci.sh x86_64-unknown-linux-gnu` and `./ci.sh aarch64-unknown-linux-gnu` passed again at the same uncommitted `HEAD` `d06ecd694521da163610b852050edde371c4bfb0`. Both model hashes matched.

### File List

- crates/sxp-kernel/src/model.rs
- crates/sxp-kernel/src/lib.rs
- crates/sxp-kernel/tests/model.rs
- spec/vectors/model-recip-strawman.sha256
- spec/vectors/model-symbols-256.sha256
- _bmad-output/implementation-artifacts/1-7-adapt-an-order-0-probability-model.md
- _bmad-output/implementation-artifacts/sprint-status.yaml

### Change Log

- 2026-11-02: Story context created. Strawman `p`, `CAP`, and RECIP stay provisional under D-07.
- 2026-11-02: Implemented the order-0 model, pinned fixtures, and both-platform CI.
- 2026-11-02: Addressed code review findings — 6 patches applied, 7 dismissed, none deferred. Both CI targets were rerun and passed.

## Story completion status

done. Both target CI runs passed after the review patches, including the RECIP hash and the 256-symbol hash. The Block framer stays with story 1.8. D-07 remains open.
