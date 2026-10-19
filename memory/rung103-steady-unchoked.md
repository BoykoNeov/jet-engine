---
name: rung103-steady-unchoked
description: Rung 103 (steady matchers below choke) process lessons — a floor on a slope that vanishes at a boundary is foreseeable; removing a refusal exposes the code behind it (a pre-existing panic at 19 % of low-throttle page requests); read a test that passes unchanged after a behaviour change
metadata:
  node_type: memory
  type: feedback
  originSessionId: cb6a1eda-ec03-4b47-9ef7-4103a61da000
  modified: 2026-10-19T12:21:16.327Z
---

Rung 103, 2026-10-19. Decisions in [[rung103-decisions]].

**1. Before freezing an absolute floor on a sensitivity, ask how it behaves at the edge of its domain.** H1's arrow
(LP compressor → HP pressure ratio) runs through the nozzle's flow function, which is FLAT at M = 1 — so the slope is
zero at the choke boundary. The 1e−6 floor missed exactly the cell at M9 0.9998 (37/38). Foreseeable from the
derivation; it was not looked for. Kept as a miss, not re-barred.
**How to apply:** for any "≥ X" bar on a derivative, write down where it must vanish and either exclude cells within
a margin of that edge in the anchor or set the bar relative.

**2. Removing a refusal exposes the code behind it.** The Cooling view's `booking.expect` panicked whenever the metal
needed no cooling (pre-existing, 900 K on the defaults); the unchoked refusal had hidden most of its reach. The seeded
crash map against the base commit found 226 panics in 1 200 low-throttle requests on the OLD code. Same family as
[[solver-audit-rust-era]] (removing a false failure exposes a true one).
**How to apply:** when a rung turns a refusal into an answer, crash-map the old side too — the comparison tells which
failures are yours.

**3. A test that still passes after a behaviour change can be vacuous.** `heavy_cooling_starts_the_knobs_search…`
passed: it accepted either outcome and compared the page against a switch-OFF engine. Read every test the census
predicted would change and did not fail.

**4. Census by refusal SITE misses failures earlier on the same path** (the advisor's catch): ten points failed in the
pinned rebuild ("back-pressure") before the guard — the pin's own artifact. Dispatch on any pinned failure.

**5. A guessed coverage count failed again** (R2 "≥ 100 choked cells", measured 87) — [[rust-port-guessed-census-bars]].

**6. Shell:** `cmd //c "start … cargo --manifest-path \"W:/…/jet engine/…\""` breaks on the space — use the
repo-relative `rust/Cargo.toml` (cmd's cwd is the repo). And `python3` is the Store stub that hangs
([[windows-tooling-file-hazards]]) — it hung again; use the Edit tool.

Related: [[rung97-unchoked-lp]], [[rung102-afterburner]] (print the scored quantity dry; count a fired trigger).
