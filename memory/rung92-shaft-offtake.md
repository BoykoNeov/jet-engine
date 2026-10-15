---
name: rung92-shaft-offtake
description: "Rung 92 (shaft-power offtake, HP/LP, share/power) process lessons — an exact identity has a domain; ask the proven-identical model why a search failed; perl `\\|` inside s|…| is alternation"
metadata:
  node_type: memory
  type: project
  originSessionId: b0abb806-9b52-4fd4-86e5-1f905db55823
  modified: 2026-10-15T19:14:40.580Z
---

Rung 92 (2026-10-15): a generator load on either shaft, share or fixed power, steady + slam + valve + sandbox knob in ONE rung (user's choice, as 90/91). Anchor `1773b7d` before any offtake code; 11 HIT, 1 MISS (B2: the LP load's steady HP relief reverses on a fast slam), A6's 650 K not scorable (unchoked). Headline found while deriving: a bleed IS a power offtake plus dumped air — exact on flat maps, gated to 1e-9.

**Lesson 1 — an exact identity has a domain; scope its predictions to it.** The identity (offtake = measured bleed response − the +1 referral) gave model-free predictions that matched the hand model to 0.003 on flat maps. On shaped maps the same rebuild said −0.28 where the truth was −0.025 (the hand model said −0.035). The anchor had limited it to flat maps and kept shaped claims to signs, which is why nothing broke.
**How to apply:** when a derivation is "exact", write down the assumptions that make it so (here: constant map efficiency and loading) and score it only inside them.

**Lesson 2 — when a search fails, ask the model you proved identical, don't write a cause.** A load the engine cannot carry made the slam model's equilibrium search wander off the compressor map, and the page blamed the maps. With no lever the steady matcher is the same plant (the plant gate proves it), so the page now re-asks it and uses its refusal ("no steady point" past the fold). See [[sandbox-slice4a]] (a cause written before its message was seen).
**How to apply:** route an ambiguous failure to a solver whose equivalence is gated, and only claim the cause it states.

**Lesson 3 — `s|…\|…|…|` in perl: an escaped delimiter is a bare `|`, i.e. ALTERNATION.** An empty branch matched at position 0 and an insertion landed at the top of `sandbox_controls.rs`. Same family as rung 91's `$&` merge: use the Edit tool for every multi-line insertion. Also: a bash heredoc holding a long Rust block failed with "unexpected EOF"; Write the block to a temp file and `cat >>` it. See [[windows-tooling-file-hazards]].

**Lesson 4 — measure which refusal fires before wording it.** A fixed power past the fold runs away (less work → less air → larger share), so the trial check fires before any root exists; "would take all the work" was technically true but the wrong story. A probe over loads × throttles showed it; the message became "no steady point".

Related: [[rung91-turbine-cooling]], [[rung90-customer-bleed]], [[sandbox-direction]].
