---
name: 'MBSC v1: claim chart of US 11,234,023 B2 and family against the MBSC coder design'
type: research-report
created: '2026-10-31'
purpose: technical briefing for a patent attorney (freedom-to-operate review that gates the kernel coder, AD-7)
subject: 'US 11,234,023 B2 (Microsoft), its continuation US 2022/0109891 A1, and EP/CN/KR/WO family members'
inputs:
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md
  - _bmad-output/specs/spec-serexpai/SPEC.md
  - _bmad-output/planning-artifacts/research/open-questions-research-2026-10-31.md
status: draft for counsel
---

# Claim chart: US 11,234,023 B2 and family vs. the MBSC rANS coder

> **NOT LEGAL ADVICE.** This document is technical research, prepared to brief a patent attorney. It is not a legal opinion and does not state whether any product infringes, or whether any claim is valid or enforceable. Every finding is phrased as "element appears present / absent / unclear on the current design". Each one is a technical reading of claim text against a design document, and claim construction is for counsel. Judgement calls are marked **⚖ COUNSEL** where they occur.

**Evidence legend.**

- **[F]** Fetched on 2026-10-31 and read directly. For Google Patents pages, that means the full page text.
- **[S]** Read only from a web-search result excerpt or a search-result page dump, not from a direct fetch of the source.
- **[UNVERIFIED]** Not confirmed from a primary source in this research.
- **[ESTIMATE]** Author's arithmetic from the spec, not a measurement.

Google Patents states on every page that its legal status, dates and assignee lists "are an assumption and not a legal conclusion". Counsel should confirm status in USPTO Patent Center and Assignment, the EPO Register, CNIPA and KIPRIS.

---

## Executive summary (technical)

- **What the granted US claims cover.** All 28 claims of US 11,234,023 B2 concern the **decoder**, and all three independent claims (1, 19, 20) recite a "two-phase structure". Claim 1 has two conditions that were added during prosecution:
  - a state update that is applied *depending on a determination of whether an output symbol from a previous iteration was generated*; and
  - symbol generation that happens *depending on a determination of whether the state "includes sufficient information"* (state "greater than a threshold").
- **How MBSC differs.** The MBSC decoder (spec seed §6) follows ryg_rans `RansDecGet`/`RansDecAdvance` (2014). Each step extracts the slot, decides the bit, updates the state for **that same** decision, and renormalizes with a `while x < L` loop. It has no deferred update, no "valid symbol" test, and no gating test before generation. On the current design, those two determination elements **appear absent**. The "two-phase structure" and "previous iteration" framing is **unclear** (⚖ COUNSEL: claim construction, plus possible equivalents limited by prosecution history).
- **Dependent claims.** The dependent claims on byte-wise merge, a 32-bit state and the update formula `x = f·x[upper] + x[lower] − cf` describe MBSC's arithmetic closely. They matter only if claim 1 is met.
- **The continuation.** US 17/552,295 (US 2022/0109891 A1) is **pending and on appeal** (notice of appeal 2026-05-07, per Google Patents). Its published independent claims (21, 30, 40) require a header **syntax element indicating whether the RANS decoder state is to be flushed/re-initialized**. MBSC has no such syntax element (flush is unconditional at every Block, and no state is carried across Blocks), so the element **appears absent**. The claims may be amended on appeal.
- **The EP member.** EP 3 991 303 B1 was granted 2025-09-10 and is **under opposition by Jarosław Duda** (filed 2026-06-08). Its claim 1 folds both determination steps into the independent claim. An EP divisional (EP 4 607 803) has had intention to grant announced (2026-06-25). KR 10-2922298 B1 was registered 2026-01-30 and a KR divisional was filed. CN 114073009 B is granted and a CN divisional exists.
- **PTAB, litigation, licensing.** No IPR, PGR, reexamination or litigation was found on the US patent. No licensing pledge was found.
- **Design choices to preserve.** The per-decision decode step with immediate update and full renormalization; no flush/retain or other per-Block coder syntax; no state carried across Blocks; binary decisions only (no symbol-width syntax); an adaptive model with nothing transmitted (no static-model selection); single stream, no interleaving or SIMD, and no hardware pipelining.
- **Fallback.** Range coding (Martin 1979) and the LZMA range coder are old, published techniques, and the LZMA SDK is public domain. A binary range coder under the same §5 model gives the same rate to within about 1 byte per Block, depending on the flush variant chosen. The live patents found that touch adaptive binary coding (Qualcomm, Tencent) claim **video** coding. They concern the probability-update rule, so they apply to MBSC's §5 model whichever coder is used.

---

## 1. Patent facts

### 1.1 US 11,234,023 B2

| Item | Finding | Source |
|---|---|---|
| Number / kind | US 11,234,023 B2 | [F] [Google Patents](https://patents.google.com/patent/US11234023B2/en) |
| Title | "Features of range asymmetric number system encoding and decoding" | [F] same |
| Application | US 16/456,602, filed 2019-06-28; pre-grant publication US 2020/0413106 A1 (2020-12-31) | [F] same; [F] [US20200413106A1](https://patents.google.com/patent/US20200413106A1/en) |
| Priority date | 2019-06-28 (no earlier priority claimed) | [F] same |
| Grant date | 2022-01-25 (legal event "PATENTED CASE" dated 2022-01-05) | [F] same; [S] PDF face "Date of Patent: Jan. 25, 2022" ([patentimages PDF](https://patentimages.storage.googleapis.com/43/8e/b4/55f763a089ddc4/US11234023.pdf)) |
| Patent term adjustment | "extended or adjusted under 35 U.S.C. 154(b) by 0 days" | [S] PDF face (search excerpt) |
| Expiry estimate | 2039-06-28 (20 years from filing, PTA 0), subject to maintenance fees and any disclaimer | [F] Google "Anticipated expiration 2039-06-28" |
| Status | Active (Google Patents) | [F] |
| Assignee | Microsoft Technology Licensing, LLC (Redmond, WA). Assignment from the inventors recorded 2019-10-08, reel/frame 050657/0473 | [F] |
| Inventors | Derek E. Gladding, Sudharsan Gopalakrishnan, Shaileshkumar D. Kumbhani, Hsu-Kuei Lin | [F] |
| Examiner / firm | Primary Examiner Susan E. Hodges; Klarquist Sparkman, LLP | [S] PDF face (search excerpt) |
| Claims | 28: independent 1 (system), 19 (method), 20 (computer-readable media) | [F] |
| Maintenance fees | 3.5-year fee ("4th year", code M1551, large entity) paid 2025-06-20. By statute the next (7.5-year) fee falls due 2029-07-25, with the window opening 2029-01-25 and a surcharge period to 2030-01-25 [ESTIMATE from 35 U.S.C. 41(b); counsel to confirm] | [F] legal events |
| Terminal disclaimer | None appears in the Google Patents legal events for this patent. **[UNVERIFIED]**: check the file wrapper | [F] (absence only) |
| Cited references (examiner-cited, *) | US7095343B2*, US20140105282A1*, US20150103902A1*, US20150349807A1*, US20160248440A1* (Greenfield, ANS), US9941900B1*, US10681388B2* | [F] "Citations (8)" table |
| Other cited art | US20170164007A1 (Google, "Mixed boolean-token ANS coefficient coding"), Duda 2014 (arXiv), Duda et al. 2015 (IEEE PCS), **Giesen "rANS in practice" (Dec 2015)**, Wikipedia "Asymmetric Numeral Systems" (2019-05-29), ISR/WO for PCT/US2020/032397 | [F] |

### 1.2 Family members

| Member | Status (as shown by the source) | Key dates | Source |
|---|---|---|---|
| **US 2022/0109891 A1** (US 17/552,295), continuation | **Pending; on appeal.** Events: notice of allowance mailed 2022-12-05; then a response to a non-final action (2023-05-21), final rejections 2023-07-06 and 2024-06-06, new cases docketed 2023-11-16 and 2024-11-19 (consistent with RCEs [UNVERIFIED]), notice of appeal 2025-06-19, appeal brief 2025-11-05, non-final action 2026-01-05/06, **notice of appeal 2026-05-07** | Filed 2021-12-15; published 2022-04-07 | [F] [Google Patents](https://patents.google.com/patent/US20220109891A1/en) |
| **WO 2020/263438 A1** (PCT/US2020/032397) | "Ceased" (normal once the national phase is entered) | Filed 2020-05-11; published 2020-12-30 | [S] [Google Patents WO](https://patents.google.com/patent/WO2020263438A1/en); [S] [PATENTSCOPE](https://patentscope.wipo.int/search/en/WO2020263438) |
| **EP 3 991 303 B1** (EP20728649.3) | **Granted 2025-09-10; under opposition.** Opposition by **DUDA, Jaroslaw** (representative Korbela), effective 2026-06-08; notice of opposition sent to the proprietor 2026-06-21/22. UPC opt-out registered (effective 2025-05-13). Lapsed in many contracting states for failure to file a translation or pay the fee (Google lists HR, GR, SE, LV and others; full list **[UNVERIFIED]**). National annual fee for year 7 paid 2026-04-21 in at least one state | A1 2022-05-04; decision to grant 2025-08-14 | [F] [Google Patents EP B1](https://patents.google.com/patent/EP3991303B1/en); [S] [greyb ipverse EP3991303](https://ipverse.greyb.com/patents/EP3991303) ("Granted And Under Opposition"; direct fetch timed out) |
| **EP 4 607 803** (EP25179939.1), divisional of EP 3 991 303 | **Intention to grant announced** (communication 2026-06-25, published 2026-07-29); UPC opt-out registered 2026-08-12. Claims **[UNVERIFIED]** (the A3 page shows no claims) | A2 2025-08-27; A3 (search report) 2025-10-29 | [F] [Google Patents EP4607803A3](https://patents.google.com/patent/EP4607803A3/en) |
| **CN 114073009 B** (CN202080047288.1) | Granted (B publication); 28 claims mirroring the US claims. Grant date **[UNVERIFIED]** | A published 2022-02-18 | [F] [Google Patents CN](https://patents.google.com/patent/CN114073009B/en) |
| **CN 202610281368.2** (divisional) | Listed as a family application; status and claims **[UNVERIFIED]** | — | [F] (listed in the US/CN pages) |
| **KR 10-2922298 B1** (KR1020227002786) | **Registered 2026-01-30** (registration published 2026-02-03) | Laid open 2022-03-11 (KR20220031636A) | [F] [Google Patents KR](https://patents.google.com/patent/KR102922298B1/en) |
| **KR 1020267003085** (KR20260019670A), divisional | Filed 2026-01-29; status and claims **[UNVERIFIED]** | — | [F] (listed) |

### 1.3 PTAB, litigation and licensing

| Item | Finding | Source |
|---|---|---|
| IPR / PGR / ex parte reexamination on US 11,234,023 | **None found.** The Google Patents legal events list no PTAB or reexamination entries. Targeted searches returned only unrelated Microsoft IPRs (ParTec '442, InterDigital, Edge Networking). The PGR window (9 months from grant) closed on or about 2022-10-25 [ESTIMATE] | [F] legal events; [S] searches |
| Litigation | **None found** (no Google Patents litigation entries; no search hits) | [F]/[S] |
| EP opposition | Duda, 2026-06-08; the citation list includes "Citation in opposition procedure 001–006" and a "Consolidated list of cited opposition documents" (not retrieved) | [S] greyb; [F] EP legal events |
| Licensing commitments | No royalty-free pledge or standards declaration found for this family. Microsoft's general IP policy: licenses on "commercially reasonable" terms and "reserves the right not to license" | [S] [Microsoft IP licensing policy](https://www.microsoft.com/en-us/legal/intellectualproperty/tech-licensing/policy) |
| JPEG XL | "Microsoft has not declared to ISO that it does [affect JPEG XL]" (Sneyers, 2022) | [F] [The Register, 2022-02-17](https://www.theregister.com/software/2022/02/17/alarm-raised-after-microsoft-wins-data-encoding-patent/394460) |
| OIN | SFC's "initial analysis shows that OIN protection will not meaningfully extend to FOSS uses that could infringe this patent" (Kuhn, quoted) | [F] same (secondary) |
| Microsoft's own rANS code | `microsoft/silicon-rans`: a Verilog decoder whose fragment header has "1 bit to reset rANS state between fragments", "4 bits to select between 16 pre-baked probability distributions", "2 bits to temporarily narrow the width of the output stream", and "IO and compute are handled in alternating cycles". Licensed MIT, which contains no express patent clause | [F] [README](https://raw.githubusercontent.com/microsoft/silicon-rans/main/README.md), [LICENSE](https://raw.githubusercontent.com/microsoft/silicon-rans/main/LICENSE) |

### 1.4 Prosecution context (US 16/456,602)

- **Rejections.** A non-final rejection issued on 2020-05-21 and a **final rejection for obviousness on 2020-10-27**. Microsoft responded under AFCP 2.0 on 2021-03-02 ("The Applicant respectfully disagrees with the rejections") ([F] [The Register, 2021-03-13](https://www.theregister.com/software/2021/03/13/third-times-a-harm-microsoft-tries-to-get-twice-rejected-encoding-patent-past-skeptical-examiners/408210)).
- **Later events** ([F] Google Patents legal events): response after final forwarded 2021-03-06; advisory action 2021-03-12; "docketed new case" 2021-04-06 (consistent with an RCE **[UNVERIFIED]**); non-final action 2021-04-21; response 2021-08-27; notice of allowance 2021-09-13; issue fee 2021-12-16.
- **Claim 1 amendments that preceded allowance** (comparison of the published application with the grant, [F]). Published claim 1 recited only: "selectively updating state … using probability information for an output symbol from a previous iteration"; "selectively merging a portion of the encoded data from an input buffer into the state"; and "selectively generating an output symbol for a current iteration using the state". The granted claim 1 **adds**:
  1. "depending on a determination of whether or not an output symbol from a previous iteration was generated" (on the update);
  2. "depending on a determination of whether or not the state of the RANS decoder includes sufficient information to generate an output symbol for a current iteration … if the state of the RANS decoder is greater than a threshold" (on the generation);
  3. "encoded data from a bitstream" (was "for at least part of a bitstream").

  The examiner's rejection grounds and the references applied are **[UNVERIFIED]**; the file wrapper was not retrieved. ⚖ COUNSEL: whether amendment-based estoppel limits equivalents for limitations 1 and 2.
- **What the description discloses but the claims do not recite.** The description states, for the flush feature: "In alternative example implementations, the RANS encoder (520) always flushes RANS decoder state between fragments" ([F] US B2 description §VI.A). ⚖ COUNSEL: relevance of disclosed-but-unclaimed alternatives (dedication-disclosure) to the continuation claims.

---

## 2. What the independent claims cover (plain language)

**US 11,234,023 B2, claims 1, 19 and 20** (system, method and computer-readable media; the same three operations). A RANS decoder works in repeated iterations, each split into two phases.

- **Phase 1.** If the previous iteration produced a symbol, the decoder now applies that symbol's state update (`x = f·(x>>n) + (x & mask) − c`). If it did not, nothing happens.
- **Phase 2.** If the state is below a threshold, the decoder merges a piece of encoded data (for example one byte) into it. Then, *only if* the state is now large enough ("greater than a threshold"), it produces the next symbol.

The claimed structure therefore **defers** each symbol's state update to the next iteration. It merges input at a bounded rate and **gates** symbol output on a sufficiency test, so some iterations produce no symbol. The description presents this as a hardware pipeline ([F] §VI.C: "at most one portion of encoded data is added to the RANS decoder state per iteration of phase 1 processing … in some iterations, no output symbols are generated"). The claims themselves are not limited to hardware, except dependent claim 17, and claim 5 says the phases are "logical phases" that may run "in the same clock cycle".

**EP 3 991 303 B1, claims 1, 12 and 13.** These cover the same structure. The two determinations are written into claim 1 as explicit if/otherwise steps:

- "determining (912) whether the output symbol from the previous iteration was generated; if so … adjusting … otherwise, skipping the adjusting";
- "determining (932) whether the state … includes sufficient information … if so, determining (934) inverse probability information and finding the output symbol … otherwise, skipping".

The merge operates on "the selectively updated state … to generate a second state". The EP claim 1 has no "greater than a threshold" wording.

**CN 114073009 B.** 28 claims that mirror the US claims one for one ([F] machine translation).

**US 2022/0109891 A1, as published (claims 21–40).** These cover **signalled flush/retain**:

- **Claim 21 (encoder method).** The encoder determines whether the decoder state "is to be flushed/re-initialized" for a part of the bitstream, and a header carries "the syntax element that indicates" that decision.
- **Claim 30 (decoder system) and claim 40 (media storing the encoded data).** The decoder reads that syntax element and decides on that basis whether to flush or retain.
- **Dependent claims** add: initial state information (claims 22, 31, and 32 "a 32-bit value"); retention when the flag says not to flush (claims 23, 34); per-fragment headers (claims 24, 35); evaluating compression with the state retained versus flushed (claim 25); symbol-width selection with lookup tables (26, 27, 37, 38); static-model selection (28, 39); and the two-phase decode (36).

Whether these are the claims currently pending on appeal is **[UNVERIFIED]**.

---

## 3. The MBSC design being compared

| MBSC feature | Where specified | Content |
|---|---|---|
| State, interval, scale | spec seed §6 | Unsigned 32-bit `x`, interval `[L, 256·L)`, `L = 2²³`, total 4096 (12-bit) |
| Decode step (normative) | spec seed §6 | `slot = x & 4095`; `bit = 0 if slot < freq0 else 1`; `(freq, start)` from the bit; `x = freq·(x >> 12) + slot − start`; `while x < L: x = (x << 8) \| next_byte()` |
| Initialization / termination | spec seed §6, §7; AD-11 | Decoder reads LE32 `x` from the first 4 Block payload bytes and rejects `x ∉ [L, 2³¹)`; after the last decision `x` must equal `L`. Payload length is derived, not stored (AD-8) |
| Encoder | spec seed §6; AD-7 | Buffers `(bit, probability)` decisions in forward order, encodes them backward from `x = L`, then lays out `LE32(x_final)` followed by the reversed renormalization bytes so the decoder reads forward |
| Flush / state across Blocks | spec seed §6 ("The format has no signalled or selective flush"); AD-8; SPEC Constraints ("adds no signalled selective-flush feature") | Unconditional 4-byte state write at the end of every Block. No syntax element chooses flush or retain. No coder state crosses a Block boundary |
| Probability model | spec seed §5; AD-7 | Adaptive, binary. 8 decisions per byte through a 255-node MSB-first bit tree; 16-bit `p`, count-indexed `RECIP[n]` rate; `freq0 = max(p >> 4, 1)`; reset per Block; nothing transmitted |
| Symbol width | spec seed §5 | Always binary decisions (1 bit); no width field |
| Block header | spec seed §3.1 | 2-bit length code and 3-bit kind (Raw, Direct, Previous-byte, Constant, Polynomial, Trigonometric, Recurrence). Kind selects the **predictor**, not the coder's model |
| Interleaving / SIMD | AD-7; SPEC Constraints and Non-goals | None; single stream |
| Deferred items touching the coder | Spine Deferred table: "Coder choice", "Global rANS stream, minimal flush"; seed §9 items 3–4; open-questions Q6.7 | A shorter flush or a global stream may be revisited by a version bump. A binary range coder is the planned fallback |

This step structure matches ryg_rans `rans_byte.h` (2014, [F]): `RansDecGet` returns `x & mask`, and `RansDecAdvance` computes `x = freq * (x >> scale_bits) + (x & mask) - start` followed by `if (x < RANS_BYTE_L) { do x = (x << 8) | *ptr++; while (x < RANS_BYTE_L); }`.

---

## 4. Claim chart

Claim text is quoted from the Google Patents rendering ([F]). On the US page, the rendering drops the referenced claim number for some dependent claims (5, 6, 8–13, 15–17, 28). Those dependencies are inferred from the claim-for-claim identical CN 114073009 B ("根据权利要求1所述…") and marked *(dep. inferred)*. Confirm against the official B2 text **[UNVERIFIED]**.

Status key: **Present** = element appears present on the current design; **Absent** = element appears absent on the current design; **Unclear** = depends on claim construction or on implementation detail not fixed by the spec.

### 4.1 US 11,234,023 B2, claim 1 (system)

| # | Claim element (verbatim) | MBSC mapping | Status | Technical reason | ⚖ |
|---|---|---|---|---|---|
| 1-pre | "A computer system comprising:" | MBSC CLI on a general-purpose computer (Spine, Environments & Operations) | Present | A general-purpose computer running the decoder | |
| 1a | "an encoded data buffer configured to store encoded data from a bitstream; and" | Block payload bytes held in memory (AD-8, AD-12: one Block at a time, ≤ 64 KiB heap) | Present | The decoder reads from an in-memory byte buffer | |
| 1b | "a range asymmetric number system ("RANS") decoder configured to perform operations using a two-phase structure for RANS decoding operations, the operations comprising:" | spec seed §6 decode step | **Unclear** | MBSC is a byte-wise rANS decoder. Its step is one sequential sequence (extract, decide, update, renormalize) with no phase division. The description calls its phases "logical phases" that may run "in the same clock cycle" (claim 5), so whether a sequential software step can be read as a "two-phase structure" is a construction question | ⚖ meaning of "two-phase structure" |
| 1c-i | "during a first phase of the two-phase structure, selectively updating, depending on a determination of whether or not an output symbol from a previous iteration was generated, state of the RANS decoder…" | §6: `x = freq·(x >> 12) + slot − start`, applied **unconditionally** and immediately after the bit is decided | **Absent** | MBSC makes no determination about whether a symbol was generated. Every step produces exactly one decision and always updates `x` for that decision in the same step. There is no "valid symbol" flag and no pending-symbol variable (contrast the patent's `sym_buf_full`, description §VI.M) | ⚖ loop-rotation reading; equivalents limited by the prosecution amendment (§1.4) |
| 1c-ii | "…using probability information for the output symbol from the previous iteration," | §6 `(freq, start)` of the decided bit | **Unclear** | MBSC uses the probability information of the decision just produced. Whether that is "from a previous iteration" depends on where an "iteration" is taken to start (see 1c-i) | ⚖ "iteration" boundaries |
| 1c-iii | "the state of the RANS decoder being tracked using a value;" | §6: 32-bit `x` | Present | A single integer state | |
| 1d | "during a second phase of the two-phase structure, selectively merging a portion of the encoded data from an input buffer into the state of the RANS decoder; and" | §6: `while x < L: x = (x << 8) \| next_byte()` | **Unclear** (merge itself: present) | The merge is conditional on `x < L`, which is "selective" in substance, byte-wise and from a buffer. Differences: MBSC merges **all** bytes needed (0, 1 or 2 here, since `x ≥ 2¹¹` after an update) in one inner loop before the next decision, whereas the patent's embodiment merges "at most one portion … per iteration". Also no separate "second phase" | ⚖ "a portion"; "during a second phase" |
| 1e | "during the second phase of the two-phase structure, selectively generating, depending on a determination of whether or not the state of the RANS decoder includes sufficient information to generate an output symbol for a current iteration, the output symbol for the current iteration using the state of the RANS decoder, the state of the RANS decoder including sufficient information to generate the output symbol for the current iteration if the state of the RANS decoder is greater than a threshold." | §6: the next step runs `slot = x & 4095` unconditionally; `x ≥ L` is an invariant guaranteed by the renormalization loop and the initial-state check (§7) | **Absent** (as specified) | MBSC never tests whether it can generate a decision. It always generates exactly 8 × (Block length) decisions, and `x ≥ L` holds by construction. The only comparison is the renormalization loop exit (`x < L` false). Whether that exit test is "a determination of … sufficient information" is the main construction risk. Note also MBSC's invariant is `x ≥ L`, while the claim says "greater than a threshold" | ⚖ whether the loop-exit test counts as the determination; equivalents and estoppel (this limitation was added to obtain allowance) |

**Claim 1 overall (technical reading):** 1-pre, 1a and 1c-iii appear present. 1c-i and 1e appear absent on the current design. 1b, 1c-ii and 1d are unclear.

### 4.2 Claims 19 (method) and 20 (media)

| # | Claim element (verbatim, distinctive parts) | MBSC mapping | Status | Reason | ⚖ |
|---|---|---|---|---|---|
| 19-pre | "In a computer system, a method comprising: receiving encoded data from a bitstream; decoding the encoded data using a range asymmetric number system ("RANS") decoder, including:" | `decompress`; benchmark round-trip verification (AD-14) | Present | MBSC decodes rANS data | |
| 19a–c | same three operations as 1c–1e ("during a first phase … selectively updating, depending on a determination of whether or not an output symbol from a previous iteration was generated…"; "…selectively merging…"; "…selectively generating, depending on a determination … greater than a threshold") | §6 | As 1c, 1d, 1e | As in 4.1 | As 4.1 |
| 20-pre | "One or more computer-readable media storing computer-executable instructions for causing one or more processors, when programmed thereby, to cause a range asymmetric number system ("RANS") decoder to perform operations, the operations comprising:" | Distributed source and binaries of `mbsc-kernel` (Spine: "distributed as source in v1") | Present (as a form of claim) | Software on media | ⚖ source distribution vs binaries |
| 20a–c | same three operations ("…selectively merging a portion of encoded data from an input buffer…") | §6 | As 1c, 1d, 1e | As in 4.1 | As 4.1 |

### 4.3 US dependent claims relevant to MBSC

These matter only if the claim they depend on is met.

| Claim | Added feature (verbatim, abridged with "…") | MBSC | Status | Reason |
|---|---|---|---|---|
| 2 (→1), 21 (→20), 25 (→19) | "…encoded data is in a fragment of the bitstream … initializing the RANS decoder … reading one or more syntax elements from a header in the fragment of the bitstream; and configuring the RANS decoder using the one or more syntax elements." | Block header §3.1 (length code, kind) | **Unclear** ⚖ | A Block is a fragment-like unit with a header. The length code fixes how many decisions are decoded, and kind selects the predictor. Whether either "configures the RANS decoder" is a construction question |
| 3 (→2) | "retrieving initial state information; and loading an initial state, as the state of the RANS decoder, using the initial state information." | §6: LE32 `x` read at Block start | Present (if 2 met) | Initial state loaded from 4 bytes |
| 4 (→2) | "a syntax element that indicates whether or not the state of the RANS decoder is to be flushed and re-initialized …; a syntax element that indicates an adjustment to symbol width …; and/or a syntax element that indicates a selection of a static probability model … from among multiple available static probability models." | §3.1, §5, §6 | **Absent** | None of the three exists: flush is unconditional, decisions are always binary, and the model is adaptive and not selected. Kind selects the predictor and every kind shares one adaptive model (AD-2, AD-7). ⚖ whether kind could be read as a "model" selector |
| 5 (→1, dep. inferred) | "…output symbols are for residual data for media … first phase and the second phase are logical phases … performed in different clock cycles or in the same clock cycle … single data stream or multiple different data streams." | Residuals δ (AD-3); 8-bit PCM is an input class | **Unclear** ⚖ | Residual coding is present. "Media" covers at least the PCM class. Single stream is present |
| 6 (→1), 22 (→20), 26 (→19) | "…determining whether the output symbol from the previous iteration was generated; if so: determining the probability information … adjusting the state … otherwise, skipping the adjusting…" | §6 | **Absent** | No such determination and no skip path (as 1c-i) |
| 7 (→6) | "…performing one or more lookup operations in one or more lookup tables." | `freq0` computed from adaptive `p`; `RECIP` is used only for the model update | **Absent** (if read as probability-information lookup) | No lookup table supplies the coder's probability information. ⚖ whether the `RECIP` lookup counts |
| 8 (→6, dep. inferred) | "…sub-range size fwd_f … cumulative sub-range threshold fwd_cf … x=fwd_f×x[upper]+x[lower]−fwd_cf" | §6 `x = freq·(x >> 12) + slot − start` | Present (formula) | The same arithmetic, which is also Duda 2013 and ryg_rans 2014 |
| 9 (→1), 23 (→20), 27 (→19) | "…determining whether the state of the RANS decoder satisfies the threshold; if so, combining the portion … otherwise, skipping the combining…" | `while x < L` | Present (in substance) | Conditional merge |
| 10 (→9) | "shifting the state … by a given number of bits; and adding the portion of the encoded data, which has the given number of bits." | `(x << 8) \| byte` | Present | |
| 11 (→9) | "…satisfies the threshold if the state of the RANS decoder is less than the threshold." | `x < L` | Present | |
| 12 (→1, dep. inferred) | "…input buffer is configured to store one byte of the encoded data at a time … portion … is one byte, and … a 32-bit value." | Byte-wise, 32-bit `x` | Present (the "one byte at a time" input buffer is **Unclear**: MBSC reads from a slice) | |
| 13 (→1), 24 (→20), 28 (→19, dep. inferred) | "…forward probability information … determining whether the state … includes sufficient information … if so, determining inverse probability information and finding the output symbol … otherwise, skipping…" | §6 `slot < freq0` | **Absent** (gating); Unclear (the "inverse probability" step itself) | No sufficiency test. The symbol is found by one comparison against `freq0` |
| 14 (→13) | "…inverse probability information includes … lookup tables … generating … further depends on a count of output symbols remaining to be generated being greater than zero." | Fixed decision count per Block; no table | Absent (lookup); count loop Unclear | |
| 15 (→13) | "…determining a sub-range of the state … associated with the output symbol…" | `slot < freq0` picks the sub-range | Present (if 13 met) | |
| 16 (→1) | "…repeating the selectively updating, the selectively merging, and the selectively generating in successive iterations, until there are no more output symbols to decode…" | Loop over 8 × length decisions | Unclear (depends on 1c–1e) | |
| 17 (→1) | "…implemented with special-purpose hardware including: the input buffer; an output buffer …; a state register …; logic … configured to perform the selectively updating; … selectively merging; … selectively generating." | Software only (Rust, Python) | **Absent** | No special-purpose hardware |
| 18 (→1) | "…during the first phase: selectively re-filling the input buffer from the encoded data buffer; and/or selectively writing the output symbol from the previous iteration to a symbol vector buffer." | — | Unclear | Depends on the phase mapping |

### 4.4 EP 3 991 303 B1, claim 1 (differences from US claim 1)

| Element (verbatim, EP) | MBSC | Status | Reason |
|---|---|---|---|
| "a range asymmetric number system, RANS, decoder (630) comprising an input buffer (632; 850)" | Byte slice and read pointer | Unclear ⚖ | Whether a slice is "an input buffer" |
| "as part of a first phase, selectively updating (910) state … using probability information (830) for an output symbol from a previous iteration" | §6 | Unclear | As 1c-ii |
| "as part of a second phase, selectively merging (920) a portion … into the selectively updated state … to generate a second state" | §6 renormalization | Unclear | As 1d |
| "determining (912) whether the output symbol from the previous iteration was generated; if so: … adjusting (916) … otherwise, skipping the adjusting" | §6 | **Absent** | As 1c-i |
| "determining (932) whether the state … includes sufficient information … if so, determining (934) inverse probability information and finding the output symbol … otherwise, skipping the finding" | §6 | **Absent** | As 1e. The EP claim states no threshold, so construction of "sufficient information" is open ⚖ |

EP dependent claims 2–11 track US claims 2, 4, 5, 8, 9, 10, 15, 16, 17 and 18. EP claim 12 (method) and claim 13 (media) repeat claim 1. EP status: under opposition (§1.2). ⚖ COUNSEL: which EP states remain in force (many lapsed), and whether the opposition stays any assertion.

### 4.5 US 2022/0109891 A1, published claims (pending, on appeal)

| Claim | Element (verbatim) | MBSC | Status | Reason | ⚖ |
|---|---|---|---|---|---|
| 21 | "encoding input symbols using a range asymmetric number system ("RANS") encoder, thereby generating encoded data for at least part of a bitstream, including:" | Kernel `encode_block` (AD-2) | Present | MBSC encodes with rANS | |
| 21 | "determining whether or not state of a RANS decoder is to be flushed/re-initialized for decoding of the encoded data for the at least part of the bitstream;" | §6; AD-8 | **Absent** | The encoder makes no such determination. Flush and re-initialization are fixed by the format at every Block | ⚖ whether a format-fixed rule is "determining" |
| 21 | "performing RANS encoding; and" | §6 | Present | | |
| 21 | "outputting the encoded data …, wherein a header in the at least part of the bitstream includes the syntax element that indicates whether or not the state of the RANS decoder is to be flushed/re-initialized …" | §3.1 Block header | **Absent** | No header field carries a flush/retain value. ⚖ whether the Block header's mere presence (each Block implies a fresh state) could "indicate" a flush. Note the antecedent problem in the published text ("the syntax element") | ⚖ |
| 22 / 31 / 32 | initial state information included when flushed; "the initial state information is a 32-bit value" | §6 LE32 `x` | Present (if 21/30 met) | | |
| 23 / 34 | not flushed → "the bitstream lacks initial state information" / "retaining the state" | — | Absent | MBSC never retains state | |
| 24 / 35 | per-fragment headers each carrying the flush/retain syntax element | — | Absent | | |
| 25 | "evaluating compression efficiency with the state … retained versus … flushed/re-initialized" | AD-10 Rate selector | **Absent** | The selector compares Candidates and lengths by exact size, never retain vs flush | |
| 26 / 27 / 37 / 38 | symbol-width selection or adjustment, with "pre-defined lookup tables having probability information for the selected symbol width" | §5 | Absent | Binary only; no tables | |
| 28 / 39 | "selecting a static probability model … from among multiple available static probability models; setting a syntax element …" | §5 | Absent | Adaptive model, nothing transmitted | |
| 30 | decoder system: "a header … includes a syntax element that indicates whether or not state of the RANS decoder is to be flushed/re-initialized …; … based at least in part on the syntax element, determining whether or not the state … is to be flushed/re-initialized … performing RANS decoding" | §6, §7 | **Absent** | No such syntax element and no decoder decision based on one | ⚖ as 21 |
| 36 | two-phase decode under claim 30 (without the "determination" limitations of the '023 claim 1) | §6 | Unclear | As 1b–1d. Depends on claim 30 | ⚖ |
| 40 | media storing encoded data whose header includes the flush/re-initialize syntax element | MBSC files | **Absent** | No such element in the MBSC Bitstream | |

### 4.6 Feature checklist (the points requested)

| Topic | Patent family | MBSC (current design) | Reading |
|---|---|---|---|
| Two-phase structure | Required by every '023 and EP independent claim; continuation dep. 36 | Single sequential per-decision step (§6) | Unclear ⚖ |
| "Selective" state update | Update deferred to the next iteration, conditional on a symbol having been generated | Immediate, unconditional update of the same decision | Appears absent |
| "Selective" merging of input | Conditional on `state < threshold`; at most one portion per phase in the embodiment | Conditional `while x < L` loop, multi-byte | Merge present in substance; "a portion … during a second phase" unclear ⚖ |
| Gated generation ("sufficient information") | Required ('023 cl. 1/19/20; EP cl. 1/12/13) | Never gated; `x ≥ L` invariant | Appears absent; loop-exit construction ⚖ |
| Adaptive vs static probabilities | Independent claims agnostic ("probability information"); static models and lookup tables only in dependents (4, 7, 14; cont. 26–28, 37–39) | Adaptive bit-tree model | Adaptivity does **not** by itself keep the independent claims away; it keeps the static-model dependents absent |
| Renormalization style | Byte-wise, 32-bit, shift-and-add, `x < threshold` (dep. 9–12) | Identical style (ryg_rans) | Present in substance for those dependents; also the 2014 prior-art style |
| Flush / state retention across fragments | Signalled flush/retain is the continuation's core (21, 30, 40) and '023 dep. 4 | Unconditional flush per Block, no syntax element, no state carried | Appears absent |
| Interleaving | Background and description only; claim 5 "single data stream or multiple" | None | Not required by any claim read; MBSC absent anyway |
| Encoder-side claims | None in '023. Continuation 21–29: flush decision and syntax element only | Forward buffering then backward encoding (§6, AD-7) | No family claim found that recites forward buffering with backward encoding. That technique is in fpaqa (2007) and Giesen (2015) (§6) |

---

## 5. Design-around analysis

### 5.1 MBSC choices that keep elements absent (must be preserved)

| # | Design choice | Keeps absent | Spec anchor | Notes |
|---|---|---|---|---|
| P1 | Each decode step outputs one decision, applies **that** decision's state update in the same step, and renormalizes **fully** (`while x < L`) before the next decision | '023 1c-i, 1e, 6, 13; EP cl. 1 determinations | seed §6 | The central point. The normative text already specifies results this way; the kernel and every decoder (Python, clean-room) should follow it structurally |
| P2 | No "valid/pending symbol" state and no test that gates generation | 1e, 13, EP 932–934 | seed §6 | The decision count is fixed by the Block length (8 × actual length) |
| P3 | Flush unconditional at every Block; no flush/retain syntax element; no coder state crosses a Block | Continuation 21, 30, 40 and dependents; '023 dep. 4 | seed §6 ("no signalled or selective flush"); AD-8; SPEC Constraints | Matches the patent's own disclosed "always flushes" alternative ⚖ |
| P4 | Binary decisions only; no symbol-width field | '023 dep. 4; continuation 26, 27, 37, 38 | seed §5 | |
| P5 | One adaptive model shared by all Candidates; nothing transmitted; no model-selection field | '023 dep. 4, 7, 14; continuation 28, 39 | seed §5; AD-2; AD-7 | Kind codes must keep selecting predictors only |
| P6 | Software only; single stream; no interleaving, SIMD or hardware pipelining | '023 dep. 5 (multi-stream wording), 17; description's two-phase hardware | AD-7; SPEC Constraints and Non-goals | |
| P7 | Rate selector never evaluates "retain vs flush" | Continuation 25 | AD-10 | |

### 5.2 Changes that would bring elements in (avoid without a renewed FTO review)

1. **Restructuring the decoder loop** so that each iteration starts by applying the *previous* decision's update. This covers loop rotation, software pipelining, "advance at top of loop" or a `pending` variable, especially with a first-iteration guard such as `if i > 0` or `if have_prev`. The guard could be read as "a determination of whether or not an output symbol from a previous iteration was generated" (1c-i).
2. **Merging at most one byte per iteration** and emitting a decision only when `x` exceeds a threshold, with "no-symbol" iterations. This brings 1d and 1e closer. It is exactly the patent's embodiment and the silicon-rans design ("IO and compute are handled in alternating cycles").
3. **Any per-Block or per-file flag that chooses whether rANS state is re-initialized or carried over.** This would affect the continuation's claims 21, 30 and 40. The Spine's Deferred row "Global rANS stream, minimal flush" is the live risk:
   - A global stream with **no** syntax element (format-version-wide rule) is technically different from a signalled choice.
   - A *signalled* choice, even a single file-level bit, would map onto "a header … includes a syntax element that indicates whether or not the state … is to be flushed/re-initialized". ⚖
4. **A per-Block field that selects among static or pre-trained models, or narrows symbol width.** For example, "model kind", an initial-probability table index, or coding δ with fewer bits when all residuals are small. These would affect the static-model and width dependents and continuation claims.
5. **Hardware, FPGA or GPU decoders** built in the two-phase style, and interleaved multi-stream decoding.

### 5.3 Low-cost design-around options for the unclear elements

| Option | What changes | Rate impact | Spec impact | Recommendation |
|---|---|---|---|---|
| D1. Make the step structure normative | Add to seed §6 a sentence that decoding is a sequence of complete decision steps (extract → decide → update for that decision → renormalize until `x ≥ L`), and that the state update for a decision is never deferred. Add the same rule to AD-7 as a kernel coding rule: no pending-symbol state, no first-iteration guard, no generation gating | **0 bytes** (the bitstream is unchanged) | seed §6, AD-7 text only; no version bump (format unchanged) | **Recommend** (cheap, and it fixes P1/P2 for every implementation including the clean-room decoder of AD-15) |
| D2. Keep the full `while` renormalization | Forbid "one byte per iteration" renormalization in the kernel and the reference decoders | 0 | seed §6 note | Recommend (part of D1) |
| D3. Pin the flush policy | Amend the Deferred row "Global rANS stream, minimal flush": any change that lets coder state cross a Block requires a renewed FTO review, and no flush/retain syntax element may be added. Bloom's minimal flush (Q6.7 option b) keeps per-Block re-initialization and adds no syntax, so it does not trigger this concern on its own. It loses the `x = L` end check (Q6.7) | Global stream would save up to the 24–32-bit per-Block excess (≈ 0.094–0.125 bits per input byte at 256-byte Blocks, ≈ 0.012–0.016 at 2048) [ESTIMATE from Q6.7 measurement]; forgone by D3 | Spine Deferred table; seed §9 item 3 | Recommend |
| D4. Switch to the binary range coder fallback | Replace §6 with an LZMA-style binary range coder driven by the same §5 frequencies | See §7: −2 to +1 byte per Block depending on the flush variant | seed §6, §7 (rANS reject rules), §8 rANS vectors, §9 item 4; AD-7, AD-11, AD-14 report label "rANS flush" | Keep ready. It removes the "RANS decoder" limitation from all family claims seen. Choose it if counsel considers 1b/1d/1e too uncertain |
| D5. Avoid uABS/rABS as a "middle" option | — | — | — | Not recommended as a design-around. It is still in the ANS family, and whether "RANS" covers binary range-ANS variants is ⚖ |

**Recommended change:** adopt D1 + D2 + D3 now (documentation and coding-rule changes only, no rate cost, no format change). Keep D4 as the fallback the FTO review can select.

---

## 6. Prior-art timeline, by element

All dates are publication or commit dates as shown by the cited source. Priority date of the family: **2019-06-28**.

| Date | Item | Elements it bears on | Source |
|---|---|---|---|
| 1976 / 1979-07 | Pasco (1976) FIFO arithmetic code; G. N. N. Martin, "Range encoding", Video & Data Recording Conf., Southampton, 1979 | Fallback base technique; byte-wise renormalization in range coders | [F] [Wikipedia "Range coding"](https://en.wikipedia.org/wiki/Range_coding) (secondary; originals not fetched **[UNVERIFIED]**) |
| 2007-10 | Duda, arXiv 0710.3861 (asymmetric binary systems), cited in fpaqa source | Binary ANS | [S] via fpaqa source; paper not fetched **[UNVERIFIED]** |
| **2007-12-15** | Mahoney, **fpaqa**: adaptive order-0 binary model with Duda's ABS. "saving predictions in a stack and then coding them in reverse order … in blocks"; "x is reset to state x = Q"; decoder "expects to find x = Q" at block end | **Encoder forward buffering / backward encoding; per-block reset; end-state check**; adaptive binary decisions with ANS | [S] [fpaqa.cpp (paq_history mirror)](https://github.com/andrew-epstein/paq_history/blob/master/fpaq/fpaqa.cpp); [S] [mattmahoney.net/dc](https://mattmahoney.net/dc/) ("fpaqa … Dec. 15, 2007"; fpaqb "bytewise I/O", Dec. 18–20, 2007) |
| 2010-02-26 | Mahoney, *Data Compression Explained*: MSB-first bitwise coding of a byte through contexts "1 followed by up to 7 earlier bits" (a 255-node tree); count-based rate `1/(count + DELTA)` with `LIMIT`; ABC coding with a stack | MBSC §5 model structure (also relevant to the Tencent/Qualcomm model patents in §7) | [S] [DCE 2010-02-26 PDF](https://nishi.dreamhosters.com/u/dce2010-02-26.pdf) |
| **2013-11-11** (v2 2014-01-06) | Duda, arXiv 1311.2540: rANS coding/decoding functions, streaming renormalization | rANS core; update formula (dep. 8) | [F] [arXiv API record](http://export.arxiv.org/api/query?id_list=1311.2540,1402.3392); [F] [abs page](https://arxiv.org/abs/1311.2540) |
| **2014-02-02** | Giesen, ryg_rans initial commit; `rans_byte.h` ("public domain … 2014"): 32-bit state, `L = 1<<23`, byte renormalization (`do x = (x << 8) \| *ptr++; while (x < RANS_BYTE_L)`), 4-byte LE flush, forward-reading decoder, separate `RansDecAdvanceStep` and `RansDecRenorm` | Decode step (1b–1e); conditional merge (dep. 9–12); 32-bit and byte (dep. 12); initial state load (dep. 3) | [F] [rans_byte.h](https://raw.githubusercontent.com/rygorous/ryg_rans/master/rans_byte.h); [F] [GitHub commits API](https://api.github.com/repos/rygorous/ryg_rans/commits?per_page=100) |
| 2014-02-02 | Giesen, "rANS notes" (blog): streaming rANS, LIFO; adaptive models require the encoder to run "a forward pass … and then a backwards pass" | Encoder buffering / backward encoding | [F] [fgiesen.wordpress.com 2014-02-02](https://fgiesen.wordpress.com/2014/02/02/rans-notes/) |
| 2014-02-03 … 02-25 | Bloom, "Understanding ANS" series (e.g. parts 5, 6, 11) and "ANS Applications" (adaptive modelling requires "buffering up the models to make the encode work backwards") | rANS mechanics; adaptive use | [S] [cbloomrants 2014-02-03](http://cbloomrants.blogspot.com/2014/02/02-03-14-understanding-ans-5.html), [2014-02-25](http://cbloomrants.blogspot.com/2014/02/02-25-14-ans-applications.html) |
| **2014-02-14** | Giesen, "Interleaved entropy coders", arXiv 1402.3392 | Interleaving / multiple streams (claim 5; description) | [F] [arXiv API record](http://export.arxiv.org/api/query?id_list=1311.2540,1402.3392) |
| 2015-05-09 | Bloom, "Oodle LZNA": rANS with adaptive models (nibble-wise) | Adaptive rANS | [F] [cbloomrants](http://cbloomrants.blogspot.com/2015/05/05-09-15-oodle-lzna.html) |
| 2015-05-11 | Bloom, "ANS Minimal Flush": standard rANS flush and decoder assertion `x == 1<<31` at end; minimal-flush variant | Flush / termination; end-state check | [F] [cbloomrants](http://cbloomrants.blogspot.com/2015/05/05-11-15-ans-minimal-flush.html) |
| 2015-05-26 | Giesen, "Models for adaptive arithmetic coding" (adaptive binary and medium-alphabet models with rANS) | Adaptive models with rANS | [S] [fgiesen](https://fgiesen.wordpress.com/2015/05/26/models-for-adaptive-arithmetic-coding/) |
| 2015 | Najmabadi, Wang, Baroud, Simon, "High throughput hardware architectures for asymmetric numeral systems entropy coding", ISPA 2015 | Hardware decoder structure (relevant to "two-phase") | Listed in [F] [Wikipedia ANS](https://en.wikipedia.org/wiki/Asymmetric_numeral_systems); paper **[UNVERIFIED]** |
| **2015-12-21** | Giesen, "rANS in practice" (cited on the face of '023): adaptive rANS; `BufferedRansEncoder` replaying "buffered symbols in reverse order" (credits fpaqa); "process the input data in chunks and flush the coder periodically, resetting the rANS state every time"; interleaving; bypass bits | **Encoder buffering; per-chunk flush and reset (cf. P3)**; interleaving | [F] [fgiesen 2015-12-21](https://fgiesen.wordpress.com/2015/12/21/rans-in-practice/) |
| 2016 | Zstandard (FSE/tANS) public release; Apple LZFSE open-sourced (July 2016) | Static per-block tables (contrast with static-model dependents) | [F] [Wikipedia ANS](https://en.wikipedia.org/wiki/Asymmetric_numeral_systems) refs; [F] [FSE README](https://raw.githubusercontent.com/Cyan4973/FiniteStateEntropy/dev/README.md) (undated); LZFSE primary **[UNVERIFIED]** |
| 2016-12-06 → 2018 | Google US 15/370,840 "mixed boolean-token ANS coefficient coding" (published as US 2017/0164007 A1, cited on '023). Non-final rejection 2018-08-28 of claims 1–20 under §103 over Nguyen (US 6,823,016) + Duda (2014) + Giesen (2014); EPO preliminary opinion of no inventive step (Feb 2018); later abandoned | Climate; examiner reliance on Duda/Giesen | [S] [EFF copy of the rejection](https://www.eff.org/files/2018/08/30/non-final_rejection_of_google_ans_patent.pdf); [S] [EFF 2018-08](https://www.eff.org/deeplinks/2018/08/after-patent-office-rejection-it-time-google-abandon-its-attempt-patent-use-public); [S] [Ars Technica 2018-06-10](https://arstechnica.com/tech-policy/2018/06/inventor-says-google-is-patenting-work-he-put-in-the-public-domain/); [F] The Register 2021/2022 (abandonment) |
| 2019-05-29 | Wikipedia "Asymmetric Numeral Systems" (cited on '023) | General | [F] cited on the patent face |
| 2019-08 | JPEG XL committee draft, arXiv 1908.03565 | Probably **after** the 2019-06-28 priority date ⚖ | Ref. in [F] Wikipedia; not fetched **[UNVERIFIED]** |
| undated | Microsoft `silicon-rans` (Verilog; fragment header with reset bit, 16 model selector, width narrowing; alternating IO and compute cycles) | Shows the patentee's own embodiment; publication date **[UNVERIFIED]** | [F] [README](https://raw.githubusercontent.com/microsoft/silicon-rans/main/README.md) |
| 2026-06-08 | Duda's EP opposition, with six cited documents (D1–D6) not retrieved | All EP claim elements | [S] greyb (register links) |

**Gaps for counsel.** No fetched source above was found to disclose, before 2019-06-28, a rANS decoder that (a) **defers** a symbol's state update to the next iteration conditional on a "symbol generated" flag, or (b) **gates** symbol generation on a state-sufficiency test with "no-symbol" iterations. Those are the limitations added to obtain allowance. The EP opposition documents (D1–D6) and the 2015 hardware paper are the most likely places to look.

---

## 7. Fallback: binary range coder under the same model

### 7.1 Patent status of the base technique

| Technique | Finding | Source |
|---|---|---|
| Range coding (Martin 1979) | Published 1979. "After the expiration of the first (1978) arithmetic coding patent [US 4,122,440, IBM], range coding appeared to clearly be free of patent encumbrances … Since that time, patents on various well-known arithmetic coding techniques have also expired." Any patent on the 1979 method itself would be long expired [ESTIMATE: US term ≤ 20 years from filing] | [F] [Wikipedia "Range coding"](https://en.wikipedia.org/wiki/Range_coding) (secondary) |
| LZMA range coder (Pavlov) | "LZMA SDK is placed in the public domain" (since SDK 4.62). "in the final version of LZMA, arithmetic coding was replaced by range coding." xz's liblzma range coder: 0BSD; adaptive bit model with an `rc_bittree` identical in structure to MBSC's tree (`model_index = 1`, then `(model_index << 1) + bit`) | [F] [7-zip.org/sdk](https://www.7-zip.org/sdk.html); [F] [xz range_encoder.h](https://raw.githubusercontent.com/tukaani-project/xz/master/src/liblzma/rangecoder/range_encoder.h) |
| CABAC (H.264, 2003) | Standardized 2003. Individual CABAC patents and their expiry **[UNVERIFIED]** (not researched patent by patent). MBSC's model is not CABAC (no context selection, no FSM state table) | — |

### 7.2 Live patents on adaptive binary arithmetic or range coding found

Status of each is **[UNVERIFIED]** unless stated.

| Patent | Assignee | Claim focus (as read) | Relevance to MBSC | Source |
|---|---|---|---|---|
| US 12,348,780 B2 "Block-level window size update for arithmetic coding" (priority US 63/306,849, 2022-02-04; filed 2022-11-30) | Tencent America LLC | Claim 1: decoding a **video stream**; "calculating a probability update rate as a function of the determined count … restarting the value of the count with the pre-defined value at … PU …, TU …, CTU level" | Closest to the **§5 model**: rate = `RECIP[n]` with count `n`, reset per Block. Claim is video-specific (PU/TU/CTU). Applies to MBSC **whichever coder** is used. Prior art for count-indexed rates: Mahoney DCE 2010 (§6) ⚖ | [S] [patents.us US12348780](https://patents.us/US12348780); [S] [PubChem](https://pubchem.ncbi.nlm.nih.gov/patent/US-12348780-B2) |
| US 10,554,988 B2 "Binary arithmetic coding with parameterized probability estimation finite state machines" (priority 2017-03-22; cited on '023) | Qualcomm | Claim 1: "decoding video data … determining one or more Finite State Machine (FSM) parameters for a next bin … parameterized state updating function" | Video-specific. MBSC's `n`-indexed rate could be argued to be a "parameter" ⚖ | [F] [Google Patents](https://patents.google.com/patent/US10554988B2/en) |
| US 10,148,961 B2 "Arithmetic coder with multiple window sizes" | Qualcomm | Claim 1: CABAC; per-context window sizes for probability update | Video/CABAC-specific. MBSC uses one schedule for all nodes, not per-context windows | [S] [exa.ai record](https://exa.ai/library/legal/patent/28bmn7r1211); [S] [patentsdb](https://patentsdb.com/patent/US10148961B2) |
| US 10,869,062 B2 "Probability initialization and signaling for adaptive arithmetic coding in video coding" (cited on '023) | Qualcomm | Title only read | MBSC signals no initialization; title only **[UNVERIFIED]** claim scope | [F] citation list |

### 7.3 Rate and overhead vs. the current rANS [ESTIMATE]

The same §5 frequencies (total 4096) drive both coders, so the per-decision ideal code length is identical. The differences are termination and integer truncation.

| Variant | Per-Block termination bytes | Excess over ideal (bits per Block) | bits per input byte at 256 / 1024 / 2048-byte Blocks (flush bytes only) | End-of-stream check | Notes |
|---|---|---|---|---|---|
| Current rANS (seed §6) | 4 | 24–32 measured (Q6.7) | 0.125 / 0.031 / 0.016 | `x == L` (strong; 0/6000 mutations accepted, Q3) | Baseline |
| RC-A: LZMA exactly (5 flush bytes; first byte always `0x00`) | 5 | ≈ 32–40 | 0.156 / 0.039 / 0.020 | `code == 0` at end (xz `rc_is_finished`) and first-byte-zero check | ≈ +1 byte per Block vs baseline. xz comments that the first byte "could have been omitted" ([F] [range_decoder.h](https://raw.githubusercontent.com/tukaani-project/xz/master/src/liblzma/rangecoder/range_decoder.h)) |
| RC-B: LZMA with the always-zero byte omitted | 4 | ≈ 24–32 (slack = log₂ range ∈ [24, 32]) | 0.125 / 0.031 / 0.016 | `code == 0` (equivalent strength to `x == L`) | Rate-neutral vs baseline |
| RC-C: minimal flush (emit only enough bytes to identify the final interval) | ≈ 1–2 | ≈ 8–16 | ≈ 0.03–0.06 / 0.008–0.016 / 0.004–0.008 | Weak; relies on the mandatory CRC-32 trailer (§2) | Saves ≈ 2–3 bytes per Block vs baseline (cf. Bloom 2015: arithmetic-coder minimal flush excess "at most 1 byte") |

- **Precision.** With a 32-bit range kept ≥ 2²⁴ and `bound = (range >> 12) · freq0`, truncation costs at most log₂(1 + 2⁻¹²) ≈ 3.5·10⁻⁴ bits per decision (≤ 0.003 bits per byte). That is negligible, and of the same order as the rANS approximation at `L = 2²³`, `M = 2¹²`.
- **Encoder.** Forward encoding removes the need to buffer decisions and encode backward (AD-7). It adds carry propagation (LZMA `cache`/`cache_size`). Exact sizes for AD-10 remain available, and bytes are emitted monotonically, which may make the lower-bound prune easier to prove.
- **Spec impact.** Rewrite seed §6. Replace the §7 rANS reject rules (initial `x ∈ [L, 2³¹)` and final `x = L`) with range-coder rules (for example `code == 0` at end). Regenerate the §8 rANS vectors. Update AD-7 and AD-11 wording, and rename "rANS flush" in the AD-14 report. §2–§5 are unchanged (seed §9 item 4).

**Fallback in two sentences.** A binary range coder (Martin 1979 / LZMA, whose SDK is public domain) under the unchanged §5 model removes every "RANS decoder" limitation in the claims read here, at a rate cost between −2 and +1 bytes per Block depending on the flush variant (RC-B is rate-neutral). The live adaptive-binary-coding patents found (Tencent US 12,348,780; Qualcomm US 10,554,988 and US 10,148,961) claim video coding and concern the probability-update rule, so they bear on §5 equally under either coder and need their own review ⚖.

---

## 8. Questions for counsel

1. **"Two-phase structure" (all '023 and EP independents).** Can a sequential software step (extract, decide, update, renormalize) be construed as a "two-phase structure", given the description's "logical phases … in the same clock cycle" (claim 5) and the hardware-centred embodiment (§VI.C, §VI.M, FIG. 8)?
2. **"Depending on a determination of whether or not an output symbol from a previous iteration was generated" (1c-i).** This was added to obtain allowance (§1.4). Does MBSC's unconditional, same-step update fall outside it literally? Does amendment-based estoppel bar equivalents? Would a first-iteration guard in some future implementation change that analysis?
3. **"Sufficient information … greater than a threshold" (1e; EP 932).** Can the renormalization loop's exit condition (`x ≥ L`) be read as "a determination of whether the state includes sufficient information to generate an output symbol", when MBSC never skips generating a decision? Does "greater than" versus MBSC's `≥ L` matter?
4. **"A portion of the encoded data" (1d).** Does merging a variable number of bytes in one inner loop read on "selectively merging a portion … during a second phase"?
5. **"From a previous iteration" (1c-ii).** How are "iterations" delimited for a software loop that produces exactly one decision per step?
6. **Continuation (US 17/552,295).** What are the claims currently pending on appeal (Patent Center), and could any amended claim cover an *unconditional* per-Block re-initialization with no syntax element, given the "always flushes" alternative disclosed in the description? Is that alternative dedicated to the public for doctrine-of-equivalents purposes?
7. **Block header as an "indication".** Could MBSC's mandatory Block header (§3.1), which always implies a fresh coder state, be argued to be "a syntax element that indicates whether or not the state … is to be flushed/re-initialized"?
8. **Global rANS stream (Deferred).** If MBSC later adopted a single rANS state across Blocks with no syntax element, as a format-version-wide rule, would that raise continuation-claim issues? Would a single file-level bit choosing global versus per-Block streams do so?
9. **EP.** In which EP states is EP 3 991 303 B1 still in force after the PG25 lapses? What are the scope and stage of the Duda opposition, and what are documents D1–D6? What claims does the divisional EP 4 607 803 have in its intended grant?
10. **KR / CN.** What claims were granted in KR 10-2922298 B1 and are pending in the KR and CN divisionals? Do any recite flush signalling or encoder features without the two determinations?
11. **Acts covered.** For claim 20 (media) and claim 1 (system), does distributing **source code** under MIT OR Apache-2.0, or binaries, or running `benchmark` round-trip decoding (AD-14), differ in exposure? Does the Apache-2.0 patent grant in MBSC's own licence matter at all to a third-party patent (Q2 notes it does not)?
12. **Implied licence.** Does Microsoft's MIT-licensed `silicon-rans` (which implements the patent's own fragment-header features) give rise to any implied licence, and to what scope?
13. **Model patents.** Do Tencent US 12,348,780 B2 (count-based rate with block-level reset, video-limited) or Qualcomm US 10,554,988 B2 / US 10,148,961 B2 bear on MBSC's §5 model for a general byte compressor? Is Mahoney's DCE (2010) relevant prior art for those?
14. **Fallback timing.** Given the continuation is pending, would counsel prefer the binary range coder (D4) at spec freeze on risk-management grounds, irrespective of the claim-1 reading?
15. **Spec citations.** Should the published spec (CC BY 4.0) cite fpaqa (2007), ryg_rans (2014), Giesen (2014, 2015) and Bloom (2015) as prior art for the coder and termination, in addition to Duda (2013) and ryg_rans already planned (SPEC Constraints)? Are there disadvantages?
16. **Maintenance and term.** Please confirm PTA (0 days on the face), the absence of terminal disclaimers in both US cases, and the 7.5-year maintenance deadline.

---

## 9. Items marked [UNVERIFIED]

- The exact dependency numbers of US claims 5, 6, 8–13, 15–17 and 28 (the Google rendering drops them; inferred from CN 114073009 B).
- US examiner rejection grounds and the references applied in the 2020-05-21 and 2020-10-27 actions; whether the 2021-04-06 "docketed new case" was an RCE.
- Absence of terminal disclaimers in US 11,234,023 and US 17/552,295.
- The claims currently pending in US 17/552,295 after the 2026 appeal steps (only the 2022 published claims were read); whether the "new case" dockets in 2023/2024 were RCEs; the meaning of the 2022-12-05 notice of allowance followed by further rejections.
- EP 3 991 303: the full list of states where it lapsed or remains in force; the content of the opposition documents D1–D6; the opposition's procedural stage (greyb page read only via search excerpt).
- EP 4 607 803: claims as intended for grant.
- CN 114073009 B grant date; CN 202610281368.2 status and claims.
- KR 10-2922298 B1 claim text (page fetched, claims not reviewed); KR 1020267003085 claims and status.
- WO 2020/263438 page read only via search excerpt.
- PTA of 0 days and examiner/firm names (read from a search excerpt of the PDF face, not the PDF itself).
- 7.5-year maintenance deadline (statutory arithmetic, not a USPTO record).
- Martin 1979 and Pasco 1976 originals (only Wikipedia's account was read); Duda arXiv 0710.3861.
- Najmabadi et al., ISPA 2015 (hardware ANS architectures): not fetched.
- LZFSE primary source and date; FSE/zstd primary release dates (Wikipedia references only).
- JPEG XL committee draft (arXiv 1908.03565): content and whether it is prior art.
- `microsoft/silicon-rans` publication date.
- CABAC patent expiry; the claim scope of US 10,869,062 B2; the legal status of US 10,554,988 B2, US 10,148,961 B2 and US 12,348,780 B2.
- Bloom's 2014 "Understanding ANS" posts and Giesen's 2015-05-26 post (search excerpts only).
- All rate figures marked [ESTIMATE] in §5.3 and §7.3 (structural arithmetic, not measured; RC-A/B/C are not implemented).

---

## 10. References

**Subject patent family**
- US 11,234,023 B2: <https://patents.google.com/patent/US11234023B2/en> (also <https://patents.google.com/patent/US11234023B2>); PDF: <https://patentimages.storage.googleapis.com/43/8e/b4/55f763a089ddc4/US11234023.pdf>
- US 2020/0413106 A1 (published application): <https://patents.google.com/patent/US20200413106A1/en>
- US 2022/0109891 A1 (continuation, US 17/552,295): <https://patents.google.com/patent/US20220109891A1/en>
- WO 2020/263438 A1: <https://patents.google.com/patent/WO2020263438A1/en>; <https://patentscope.wipo.int/search/en/WO2020263438>
- EP 3 991 303 B1: <https://patents.google.com/patent/EP3991303B1/en>; opposition summary: <https://ipverse.greyb.com/patents/EP3991303>; EPO Register: <https://register.epo.org/application?number=EP20728649>
- EP 4 607 803 A3: <https://patents.google.com/patent/EP4607803A3/en>
- CN 114073009 B: <https://patents.google.com/patent/CN114073009B/en>
- KR 10-2922298 B1: <https://patents.google.com/patent/KR102922298B1/en>
- Microsoft silicon-rans: <https://github.com/microsoft/silicon-rans> (README, LICENSE under raw.githubusercontent.com)
- Microsoft IP licensing policy: <https://www.microsoft.com/en-us/legal/intellectualproperty/tech-licensing/policy>

**Press and context**
- The Register, 2021-03-13: <https://www.theregister.com/software/2021/03/13/third-times-a-harm-microsoft-tries-to-get-twice-rejected-encoding-patent-past-skeptical-examiners/408210>
- The Register, 2022-02-17: <https://www.theregister.com/software/2022/02/17/alarm-raised-after-microsoft-wins-data-encoding-patent/394460>
- Ars Technica, 2018-06-10: <https://arstechnica.com/tech-policy/2018/06/inventor-says-google-is-patenting-work-he-put-in-the-public-domain/>
- EFF, 2018-08: <https://www.eff.org/deeplinks/2018/08/after-patent-office-rejection-it-time-google-abandon-its-attempt-patent-use-public>; rejection PDF: <https://www.eff.org/files/2018/08/30/non-final_rejection_of_google_ans_patent.pdf>
- iMagazine (PL), 2026-03-16 (opposition deadline 2026-06-10): <https://imagazine.pl/2026/03/16/microsoft-patent-polski-algorytm-ans-jaroslaw-duda-interwencja/>
- Wikipedia, Asymmetric numeral systems: <https://en.wikipedia.org/wiki/Asymmetric_numeral_systems>

**ANS prior art**
- Duda, arXiv 1311.2540: <https://arxiv.org/abs/1311.2540>; metadata: <http://export.arxiv.org/api/query?id_list=1311.2540,1402.3392>
- Giesen, arXiv 1402.3392 "Interleaved entropy coders": <https://arxiv.org/abs/1402.3392>
- Giesen, ryg_rans `rans_byte.h`: <https://raw.githubusercontent.com/rygorous/ryg_rans/master/rans_byte.h>; repository: <https://github.com/rygorous/ryg_rans>; commits: <https://api.github.com/repos/rygorous/ryg_rans/commits?per_page=100>
- Giesen, "rANS notes" (2014-02-02): <https://fgiesen.wordpress.com/2014/02/02/rans-notes/>
- Giesen, "rANS in practice" (2015-12-21): <https://fgiesen.wordpress.com/2015/12/21/rans-in-practice/>
- Giesen, "Models for adaptive arithmetic coding" (2015-05-26): <https://fgiesen.wordpress.com/2015/05/26/models-for-adaptive-arithmetic-coding/>
- Bloom, "Understanding ANS – 5" (2014-02-03): <http://cbloomrants.blogspot.com/2014/02/02-03-14-understanding-ans-5.html>; "– 6": <http://cbloomrants.blogspot.com/2014/02/02-04-14-understanding-ans-6.html>; "– 11": <http://cbloomrants.blogspot.com/2014/02/02-14-14-understanding-ans-11.html>; "ANS Applications": <http://cbloomrants.blogspot.com/2014/02/02-25-14-ans-applications.html>
- Bloom, "Oodle LZNA" (2015-05-09): <http://cbloomrants.blogspot.com/2015/05/05-09-15-oodle-lzna.html>
- Bloom, "ANS Minimal Flush" (2015-05-11): <http://cbloomrants.blogspot.com/2015/05/05-11-15-ans-minimal-flush.html>
- Mahoney, fpaqa (2007): <https://github.com/andrew-epstein/paq_history/blob/master/fpaq/fpaqa.cpp>; program list: <https://mattmahoney.net/dc/>
- Mahoney, *Data Compression Explained* (2010-02-26): <https://nishi.dreamhosters.com/u/dce2010-02-26.pdf>
- Collet, FSE: <https://raw.githubusercontent.com/Cyan4973/FiniteStateEntropy/dev/README.md>
- LZFSE: <https://github.com/lzfse/lzfse>

**Range coding and fallback**
- Wikipedia, Range coding: <https://en.wikipedia.org/wiki/Range_coding>
- LZMA SDK: <https://www.7-zip.org/sdk.html>
- xz `range_encoder.h`: <https://raw.githubusercontent.com/tukaani-project/xz/master/src/liblzma/rangecoder/range_encoder.h>; `range_decoder.h`: <https://raw.githubusercontent.com/tukaani-project/xz/master/src/liblzma/rangecoder/range_decoder.h>
- US 10,554,988 B2 (Qualcomm): <https://patents.google.com/patent/US10554988B2/en>
- US 10,148,961 B2 (Qualcomm): <https://exa.ai/library/legal/patent/28bmn7r1211>; <https://patentsdb.com/patent/US10148961B2>
- US 12,348,780 B2 (Tencent America): <https://patents.us/US12348780>; <https://pubchem.ncbi.nlm.nih.gov/patent/US-12348780-B2>

**MBSC documents compared**
- `bitstream-spec-seed.md` §3, §5, §6, §7, §9; `ARCHITECTURE-SPINE.md` AD-2, AD-7, AD-8, AD-10, AD-11, AD-14, Deferred table; `SPEC.md` Constraints, Non-goals, Open Questions; `open-questions-research-2026-10-31.md` Q2 (patent landscape), Q3 (corruption detection), Q6.7 (flush options).
