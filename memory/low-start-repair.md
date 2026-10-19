---
name: low-start-repair
description: "2026-10-19 solver repair (sandbox plan § 37): rung 40's equilibrium re-starts lower where full speed refuses, root kept only if it runs — lessons: a fallback with a filter the main path lacks makes a rule depend on the solver's start (user chose keep-as-is + honest words); a refusal's reason is ADDED after its old words so word-sorting readers don't move; a moved floor stales the page's quoted numbers"
metadata:
  node_type: memory
  type: project
  originSessionId: a079546d-3557-4fda-a49e-a422441b0488
  modified: 2026-10-19T18:00:29.618Z
---

Repair (not a rung), 2026-10-19, closing [[hp-wall-repair]]'s "still open": just above idle the full-speed start
(1, 1) has no closure root (compressor exit hotter than the burner target). Fix in
`TwoSpoolTransientCore::try_equilibrium`: only where `start = None` refused, re-run the Newton from `LOW_STARTS`
(0.9 … 0.4); keep the first root only if thrust per inlet air > 0, else refuse with the full-speed words FIRST and
"; and below idle … more drag than thrust" after. Gate `rust/tests/low_start_repair.rs`. No Python-held key moved.

**User decision (2026-10-19): keep the fix, honest words — NOT one below-idle rule everywhere.** The full-speed start
still returns choked below-idle points (41 on the probe grid; the Controls page starts at −64 to −80 N/(kg/s) at
Mach 2.6), and the steady matchers refuse below idle only unchoked. Recorded as open, not fixed.

**Lesson 1 — a filter on a fallback only is a rule that depends on the solver's START.** A side agent caught it; I had
measured it (41 points) but not framed it as a user decision. Frame it before shipping, with plain consequences.

**Lesson 2 — when a refusal gains a reason, APPEND it after the old words.** `classify` / `abort_kind` and the page's
`endpoint_words` sort refusals by substring; appending keeps every existing sort, and the new branch goes FIRST.

**Lesson 3 — a moved floor stales every number the product quotes about it.** The page's refusal text quoted
510–520 K / 610 K; no test read those numbers. Re-measure (lowest accepted start per map and Mach) and grep.

Hazard: launch cargo with a spaced manifest path as `Start-Process cmd -ArgumentList '/c','start','/belownormal',
'/b','/wait','cargo',…` (priority from the start); setting `PriorityClass` after `Start-Process` misses children.
Related: [[hp-wall-repair]], [[rung103-steady-unchoked]], [[windows-tooling-file-hazards]].
