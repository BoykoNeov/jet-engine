---
name: rung104-decisions
description: "User scope decisions 2026-10-20 for rung 104 (A45, the LP turbine's NGV, never choked — replace rung 38's pin with a flow law): 3-setting knob (pin / ellipse default / two rows / one row), ON EVERYWHERE, all solvers in ONE rung, Python-held oracles keep their UNMOVED keys (moved ones retired by name), SORT every failing older test"
metadata:
  node_type: memory
  type: project
  originSessionId: 608a4dc7-68b6-4be6-be9a-6ef698d34bc2
  modified: 2026-10-20T13:42:21.857Z
---

Picked 2026-10-20 from my post-103 list ("the turbine-throat check"). NB I first mis-described it as the HP turbine's
vanes; it is `A45`, the LP turbine's NGV — corrected to the user.

**Probe facts (`docs/plans/scratch-archive/rung104/g1.txt`, `g2.txt`):** LP turbine total ratio 1.451–1.463 on the whole choked
running line; even if the vane took the whole drop to exit static at M5=0.6 it reaches only 0.990–0.996 of the
critical ratio — A45 is NEVER choked, design point included. HP NGV bound 1.58–1.60× critical (borderline with
reaction — a concession, not built). With the nozzle choked, CPG: pin ≡ any design-captured law to 1e-15 (the choked
A8 pins π_LPT whatever the vane does). Real gas (G3, the gas's own nozzle routine): π_HPT shift at Tt4 700 — one row +0.26 %, two rows +0.39 %, ellipse +0.72 % (pin < 1 row < 2 rows < ellipse: the stiffest law moves least; my first G2 one-row law had no γ(T) and was WRONG; row laws 3 min/point in the probe — need a fast form). n rows → ellipse to ~0.5 % at n=8–16 (G3a). Superseded G2 note: constant-γ one-row nozzle law
+2.3 % at Tt4 700 (to be re-measured with the gas's own subsonic nozzle routine). Below choke: not yet measured.

**User answers:**
- Law: a KNOB with three settings — Stodola's ellipse (default), one stage with vane+rotor sharing equally (middle;
  the guessed split), single vane row; plus the old pin as its own setting (the literal old code path).
- Default: **ON EVERYWHERE** (not a page-only switch).
- Default LAW confirmed 2026-10-20 after learning the ellipse moves MOST (side-agent note): **ellipse stays default**;
  sort old tests under it (the strict test — a finding surviving it survives the milder laws, IF the ordering holds);
  **every finding that FLIPS is re-checked under TWO ROWS** and the spec says whether it flips under the realistic law
  or only the extreme one.
- Scope: **everything in one rung** — steady six, transient family, afterburner, cooling, rung 97/103 branches.
- Python-held oracles (two-shaft): **RETIRE** them — but only AT THE END of rung 104 (user, 2026-10-20, on a side-agent note): during the build they run on the old-law (pin) setting as the safety check that the pin setting is still the old model bit for bit; retire once rung 104 passes.
- Older rungs' failing tests: **SORT EVERY ONE** — (1) number moved, finding holds → update bar; (2) finding
  weakens/flips → correction in that rung's spec; (3) untouched. (User chose this over a one-table survey.)
  "Recheck" means each rung's FINDING IN WORDS, not its old numbers.

**How to apply:** anchor first (advisor list: compiler-census every A45 read by making the field private — four
direct reads at afterburner/turbine_cooling/unchoked_lp outside the solver; time a Controls slam with the nested
solve; reach census = whole gate in a throwaway worktree with the law ON; check many equal rows → ellipse before
claiming they are one family). MEASURED order (G3, P9): pin < one row < two rows < ellipse — the stiffest law moves
most. Anchor: `docs/plans/rung104-anchor-turbine-law.md`. Related: [[rung103-decisions]],
[[rung97-unchoked-lp]].
- **Oracles, refined 2026-10-20 (AskUserQuestion, after learning the cost):** keep the UNMOVED parts. Only the values
  the flow law changes (real-gas, equilibrium-gas, below-choke) are retired, by name per file; everything else stays
  checked against Python (the rungs 46–48 oracle is the only guard on three bugs). Supersedes "retire whole".
  DONE: 87 458 of 263 647 keys retired (lists in `rust/oracle/rung104_retired/`); besides the real/eq gases, 264 keys
  at eight perfect-gas cells of rung 39's `turb` map (η_LPT off design ⇒ the law differs even choked). Counts the law
  moves are held as Rust-owned numbers (Python's on `pin-default`) — spec § 5.
