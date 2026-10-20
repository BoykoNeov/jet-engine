# Rung 105 scratch archive — what the anchor's givens were measured with

Archived 2026-10-20 before any rung-105 code entered the repo (the anchor: `docs/plans/rung105-anchor-hp-vane.md`).

* `probe/` — G1 (`g1.txt`), against the repo at `4ba69ee`: the reaction below which the HP vane chokes, at every
  running-line point of the one-shaft (Fly it) and two-shaft (rig) engines.
* `probe2/` — P1–P7, against a throwaway worktree of `4ba69ee` carrying the PROTOTYPE: `prototype.patch` (its diff)
  plus `hp_vane.rs.prototype` (the new, untracked module). `probe2/Cargo.toml` points at that worktree
  (`W:/temp/claude/rung105-wt`, deleted at the session's end — re-create it from the patch to re-run). In the prototype
  the law (reaction 0.3) was the DEFAULT; `--features pin` (`hp-pin-default`) made the old pin the default instead.
  * `p1.txt` — the law against the pin on the steady running lines (one shaft: rung 31/33's matcher; two shafts:
    rung 39's map matcher, below choke on). NB its first column is the first law that SOLVED, labelled `pin` in this
    run: at one-shaft hot Mach 0.3 the pin refuses, so those rows compare reactions 0.3/0.5 with 0.15, not with the
    pin (the label was fixed in `p1.rs` after this output was taken).
  * `p2.rs` — the pin's refusal words at one-shaft hot Mach 0.3 (read in session).
  * `p3.txt` — rung 104's headline on rung 104's own probe cells under the HP law.
  * `p4.txt` — rung 33's (★★) residual on a fine grid: TWO roots at hot Mach 0.3, 650–900 K, the pin too.
  * `p5_pin.txt` / `p5_law.txt` — COST on the pages' own entries (best of 5 / 3).
  * `p6.txt` — the law's call census on one Fly-it point and one slam.
  * `p7.txt` — the floors: the design point (each engine's distance from its own design run, pin and law) and the
    perfect gas choked with the shortcut off.
* `p8.rs` — where the CAPTURE's vane flips from choked to unchoked against G1's `R*` (the knob-definition check; its
  output was read in session: first 0.26–0.27 against 0.18–0.20, then, after the textbook-reaction fix, equal to the
  fourth digit).
* `p1b.txt`, `p3b.txt`, `p7b.txt` — P1, P3, P7 re-run on the CORRECTED knob (the anchor's G3/G7 numbers); `p1b`'s first
  column is labelled `(REF)` — the first law that solved — and the "choked" demonstration setting is 0.10.

**Added during the build, against the repo with rung 105's code (`probe3/Cargo.toml` points at it):**

* `preflight.txt` — the scored cells' classes only (`probe3/src/pf.rs`).
* `c1f.txt` — COST on the shipped code (`probe3/src/c1.rs`), after the three rounds of changes (spec § COST); `d1.rs` the
  one crash it found mid-way (the equilibrium search losing its bracket before the closure's end clamps).
* `r9.txt` — the red check's run (the effective A4 forced to A4 in a scratch worktree).
* `rep.txt` — the reported-only numbers (`probe3/src/rep.rs`): H1's margin, S3's thrust, the one shaft below choke, G4's
  extent per law, the slams, the pages' crash map.

**Added after the build (the census, 2026-10-21):**

* `r1b.txt` — R1b: every failure of the gate built with `hp-law-default`, classed by its panic message (rules in its
  header).
* `noflight.txt` — which of those failures are the law refusing for want of its flight's ambient pressure: a scratch
  worktree at `7f1936d` with one `eprintln!` in `hp_vane::no_flight`, twelve suspect test binaries run.
* `probe3/src/d2.rs` — rung 78's own rig, its first march, the law off and on: the march is identical to round-off, the
  stator's "riding" label flips, the set its reader indexed goes empty.
* `c0.txt` — C0 head to head (`probe3/src/c0.rs`, which names no rung-105 setting so it builds against `4ba69ee` too):
  the same pin requests on the old and the new code, interleaved twice.
