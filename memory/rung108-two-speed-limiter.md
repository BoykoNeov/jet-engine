---
name: rung108-two-speed-limiter
description: "Rung 108 (2026-11-11), the two-speed temperature limiter — process lessons: re-read the EXACT words of an option the user picked before framing the next question on it (I told the user the box was off by default when their pick said the realistic case IS the default; a side agent caught it); a loop's assertion must name its cell before the first diagnosis (my first probe diagnosed the wrong cell); a 'pinned by construction' gate failed in arithmetic (RK4 stages read the second constant early, also at the ENGAGEMENT step) — restate the gate to the pinned split, keep the law"
metadata:
  node_type: memory
  type: project
  originSessionId: 7bd67372-25f9-448c-8397-ca17317f3231
  modified: 2026-11-11T17:51:03.916Z
---

Rung 108 shipped rung 52's fast-cut / slow-give-back lag on the redline clip, governed (0.005 step) and open (spec
`docs/rung108-spec.md`, anchor `docs/plans/rung108-anchor-two-speed-limiter.md`, § 3 frozen at `e14a723`). Decisions:
[[rung108-decisions]]. Follows [[rung107-governor-release]]. Scored 144 HIT / 4 MISS / 12 VACUOUS.

**Lessons (process):**
1. **Re-read the exact words of the option the user picked before building on it or asking the next question about
   it.** The page question's chosen option said "the realistic case is what you get by default"; I built the box
   unticked, then asked the cost question as "the box is off by default, so nothing that runs today gets slower" — the
   user's "ship as is" rested on my wrong description. A side agent flagged it; re-asked, the user chose ticked (on the
   page only; the model's default stays off so no earlier gate moves). **Why:** a default is a decision the user has
   already made; a later question that restates it wrongly silently overturns it. **How to apply:** grep the
   transcript for the picked label's description before writing the build or the next question that depends on it.
2. **Make a loop's assertion name its cell (route, shape, constant) before diagnosing.** The pinned-trigger test
   failed with only "at s=0.3"; I guessed the first iteration (governed) and spent a long probe compile proving that
   cell clean. The failing cell was the OPEN route at the coarse step — found the moment the message carried it.
3. **A "pinned by construction" claim can fail in arithmetic: log every RK4 stage, not only the recorded points.**
   "τ_rel is never read before the first crossing" is true of the recorded points but not of the stages: at the 0.02
   step a stage crossed one step early (rung 107's lesson) AND — new — at the ENGAGEMENT step a mid-step stage put g 4×
   above a target just leaving zero, twelve steps early. The advisor's call: keep the law (per evaluation, as 52/107),
   restate the gate up to the pinned SPLIT with the gap after it bounded, add a dated § 2 amendment (§ 3 untouched), and
   first measure whether the scored quantity moves (2.8e−5 K of 24 K — it did not).
4. **A new settings field or route variant breaks OTHER rungs' test binaries** (`tests/sandbox_controls.rs`'s full
   struct literal and exhaustive `match`, `tests/sandbox_page.rs`'s knob census read at defaults) — iterating on
   `--test rung108` alone hid both until the wrap-up gate. **How to apply:** after adding a field/variant, grep
   `rust/tests/` for the previous rung's newest field (e.g. `hp_vane:`) and `Route::GovernorRelease`, and update each.
5. (Repeat hazard) `python3 - <<EOF` from the Bash tool launched an interactive REPL that hung until stopped — twice in
   one session. Use the Edit tool for file edits; see [[windows-tooling-file-hazards]].
