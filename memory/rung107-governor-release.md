---
name: rung107-governor-release
description: "Rung 107 (2026-11-10), the governor beside rung 52's realistic release — process lessons: a frozen mechanism can HIT for the wrong reason (H6 sagged 16/16, but the sag was the slow ATTACK's, not the release's — found only by varying the two edges SEPARATELY); a 'no-lag runs never sag' comparison held only on the hold it was run on (the side agent caught the scope); write a missing vacuity rule BEFORE the first run; check a 'τ-free by construction' premise numerically; a mechanism measured AT a numerical knob floor is the floor's (CORRECTS→BOUNDS 106, after shipping)"
metadata:
  node_type: memory
  type: project
  originSessionId: cd77f469-59bb-4cab-8601-250d168d18ba
  modified: 2026-11-10T20:20:55.205Z
---

Rung 107 shipped the governor beside rung 52's fast-attack / slow-release lag (spec `docs/rung107-spec.md`, anchor
`docs/plans/rung107-anchor-governor-release.md` committed `7b15c6b` before code). Decisions: [[rung107-decisions]].
Follows [[rung106-governor-lag]]. Headline: holding thrust, the governed thrust sags, mostly because the slow CUT lets
fuel through first; the release adds little. Scored 38 HIT / 10 MISS / 12 VACUOUS.

**Lessons (process):**
1. **A HIT can rest on the wrong mechanism — split a two-edge device's edges before naming the cause.** H6 ("the
   governed thrust sags") hit 16/16, and the anchor's reason was the RELEASE paying back the stored cut. The panel's own
   numbers hinted otherwise (the sag barely moved from release 0.1 to 1.0, and the engine was at 78 % of the step at the
   move's end against 33 % instant). A probe varying attack and release SEPARATELY (at a fine step: the fast pairs blow
   up at the coarse one) showed an instant release keeps ~90 % of the sag (the edges do not add, so not a share). That refuted the seam's own premise. Gate the
   decomposition when the headline rests on it.
2. **A comparison holds only on the configuration it was run on.** "The no-lag governor never sags" was measured on the
   direct thrust hold; my own "other laws" table showed the instant governor sagging 5–13 % under HP speed / read thrust.
   A side agent flagged it before the row was written. Scope every claim to its hold in the row, the panel, the README.
3. **Write the vacuity rule a claim LACKS before the first run.** H3 had no frozen rule; without one, a cell whose 95 %
   time came before the release would have HIT for free. The advisor caught it; I wrote the rule (inherit H2's) into
   `score.rs`'s header first. In the end every H3 cell was VACUOUS (the slow runs never reached 95 %).
4. **Check a "τ-free by construction" premise numerically.** The anchor said the deepest cut cannot depend on the
   release time (the pinned trigger). It does, by ≤ 4e-4: the deepest point sits AT the crossing, whose step already
   read τ_rel in its inner stages. Printing the four values bit for bit caught it.
6. **When a headline's mechanism sits AT a knob's floor, ask whether that floor is physical or numerical — before
   naming the cause.** The "slow CUT makes the sag" verdict was measured at the page's 0.2 τ attack, which is the step
   check's floor, not hardware. My own fine-step probe (sagcause) already showed a fast attack sags a third as much and
   rises with the release — i.e. rung 106's premise HOLDS there. A side agent flagged it after shipping; the row went
   from "CORRECTS 106's seam" to "BOUNDS", and the page's moved-knob note now says so (follow-up commit, same day).
5. **A gate with nothing to hold is dropped, and the spec says why.** The governed lean-hold test held 0 of 126 points;
   0 holds in 1 344 + 270 admitted runs — dropped, recorded in spec § 2.
