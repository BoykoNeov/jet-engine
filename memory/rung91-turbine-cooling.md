---
name: rung91-turbine-cooling
description: Rung 91 (turbine cooling air, vane + blade) process lessons — derived bars and a Richardson reading made every bar land; perl line inserts merged statements; a numeric knob never refreshed its note
metadata:
  type: project
---

Rung 91 (2026-10-15): turbine cooling air, two streams (vane before the HP rotor, blade after), steady + transient + rung 64's valve + sandbox knobs in ONE rung (user's choice, as rung 90). Anchor `e46ab3d` before any cooling code; all 12 claims HIT (model within 0.0062 of every reading).

**Lesson 1 — rung 90's lesson applied paid off: derive every bar, and read what the model predicts.** Bars came from rung 90's measured residual scaled by the new slopes; crossing bars = magnitude bar / local slope. The model is first-order, so readings were Richardson-extrapolated (`2D(0.01) − D(0.02)`), not one-sided at 0.02 — with slopes up to 2× rung 90's, a one-sided reading would have scored curvature the model never claimed.
**How to apply:** when the instrument is a linear model, extrapolate the measurement to zero perturbation before comparing.

**Lesson 2 — never insert lines with a perl `s/…$/$&\n…/` one-liner.** `$` matches before the newline, so `$&` holds no newline and the "new line" lands on the SAME line (it merged statements twice here: refusals in `customer_bleed.rs`, a panel entry in `panels/mod.rs`). Also a stray `python -` in a command hung on the Store stub (see [[windows-tooling-file-hazards]]).
**How to apply:** use the Edit tool for every insertion; reserve perl for same-line substitutions, and grep the result.

**Lesson 3 — the sandbox's numeric knobs only schedule a run; they never refresh the view.** A note shown/hidden by a numeric knob stayed hidden; `browser.mjs` caught it. The cooling knobs now call `showSettings()`.

**Lesson 4 — an inactive dispatch returns no booking.** The scorer compared the customer bleed through rung 91's matcher (cooling off ⇒ rung 90's body ⇒ `booking: None`) and crashed; compare a lower rung through its OWN matcher.

Related: [[rung90-customer-bleed]], [[sandbox-direction]], [[rung64-phi-bleed-limiter]].
