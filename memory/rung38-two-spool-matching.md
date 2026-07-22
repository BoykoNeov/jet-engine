---
name: rung38-two-spool-matching
description: "SHIPPED rung 38 = two-spool matching (LPC+LPT/HPC+HPT, no bypass); rung-31 (star) trick chained twice via a THIRD choked throat (A45); self-corrected mid-rung finding (compressor-efficiency-leaf, NOT spool independence); advisor set the reduce contract before code"
metadata: 
  node_type: memory
  type: project
  modified: 2026-07-22T09:02:23.697Z
  originSessionId: 9daab701-a7a6-48a6-91bf-793f765918be
---

SHIPPED rung 38 (`build_two_spool_turbojet` / `TwoSpoolEngine` / `TwoSpoolMatcher`, separate
classes, NOT subclasses of `Engine`/`OffDesignMatcher`) is the FIRST TWO-SHAFT rung. Closes a seam
named since rung 31 (two-spool/multi-shaft dynamics). Station layout `0→2→25→3→4→45→5→9` (LPC/HPC
on the same series flow path, no bypass — a fan/bypass split is a different engine, explicitly out
of scope).

**Advisor set the shape before any code.** Consulted twice: (1) before deriving anything — advisor
said the reduce contract is make-or-break and is almost certainly EXACT DISPATCH not a knob-to-zero
(a degenerate `π_LPC=1` wouldn't reduce exactly because the LPT-NGV throat `A45` would still be
present and its geometric pin wouldn't land on `τ_LPT=1` automatically); told me to scope to STEADY
matching only (defer the two-shaft transient, since its reduce gate needs this matcher first) and to
build the two-spool DESIGN reference myself (nothing upstream gives one for free). (2) after I worked
the derivation on paper — confirmed the triangular-cascade math but flagged BEFORE I wrote code that
my "π_LPC independent of the HP spool" framing was too strong and would need the `f`-residual
caveat; told me to scope to the FULLY-CHOKED branch only (nozzle-unchoke on the LP spool is a
genuinely different solve, NOT a free reuse of rung 33's `_match_subsonic`, since that's built for
one turbine); told me to frame the finding as a NO-COMPRESSOR-MAP model artifact, not a physical law.

**The mechanism**: a two-spool turbojet adds a THIRD choked throat no single-spool rung has — the
LP-turbine NGV / inter-turbine duct (station 45, area `A45`) between the HP turbine's exit and the
LP turbine's inlet. With all three throats (`A4, A45, A8`) choked, rung 31's `(★)` mass-flow trick
chains TWICE: `τ_HPT` pinned by `(A4,A45)` alone, `τ_LPT` by `(A45,A8)` alone — Tt4→Tt45→Tt5 purely
geometric.

**THE FINDING — self-corrected mid-implementation (not just disclosed, actually caught and fixed
via smoke-testing before locking the spec/test).** My FIRST framing ("the LP spool solves
independent of the HP spool") was WRONG — verified numerically that `η_HPT` demonstrably moves
`π_LPC` too, since it shapes the shared `Tt45` that feeds BOTH shaft balances. Re-derived precisely:
each compressor's OWN isentropic efficiency is a TERMINAL LEAF (enters only the last algebraic step
converting an already energy-fixed `ΔT` into a pressure ratio) — `η_LPC` cannot reach `π_HPC`,
`η_HPC` cannot reach `π_LPC` — but every turbine/geometry parameter (both turbine `η`'s, all three
areas) legitimately reaches BOTH compressor ratios. So the real claim is narrower: no 2×2
SIMULTANEOUS solve between the two compressor pressure ratios (Step 3/π_LPC fully resolves before
Step 4/π_HPC begins), NOT "the spools don't talk." This is the SAME kind of correction rung 32 made
to rung 31's own over-claim ("choked hardware IS the map" → actually the map matters) — here made
BY ME before shipping, not by the advisor catching it after. Framed explicitly as a NO-COMPRESSOR-MAP
model artifact: "two-spool + maps" (rung-32-style) would likely reintroduce real 2×2 coupling and is
filed as the deferred seam.

**Reduce = exact dispatch** (rung-37 pattern): `TwoSpoolMatcher(design, ..., lp_disabled=True)`
never builds an LPC/LPT/`A45` — `__init__` holds a plain `OffDesignMatcher` and `.match()` forwards
to it verbatim (bit-for-bit `==`).

**Non-tautological gate**: since the `lp_disabled` reduce never enters the two-spool cascade code
path at all (same structural issue as rung 33's gate 1), the ONLY thing tying the cascade's actual
NUMBERS down is an independent bare-math CPG solve (no `Gas`/`Component`/`TwoSpoolMatcher` calls) —
reproduces `(π_LPC, π_HPC, τ_HPT, τ_LPT)` to machine zero across a throttle sweep, plus the
structural CPG fact that both `τ`'s are `Tt4`-independent (since `choked_mfp` is `Tt`-independent
for CPG — the standard fact that a calorically-perfect choked throat's corrected mass flow is a pure
`(γ,R)` constant).

**A refactoring choice that paid off**: exposed the inner Steps-1–4 solve as its own `_cascade()`
method (not inlined in `match()`'s loop) specifically so the finding is directly testable by
mutating an instance attribute (`matcher.eta_hpc = X`) and re-calling `_cascade` at a FIXED
`(Tt2, Tt4, f)` — isolating the claim from the outer `(f, pt4)` fixed-point loop's own (separately
disclosed, weak, equilibrium-gas-only) cross-talk between the spools.

Deferred seams this rung names: nozzle-unchoke on the LP spool (rung-33-shaped follow-on); two-spool
+ component maps (likely correction, rung-31→32 pattern repeated); the two-shaft transient (needs
this steady matcher first, the natural rung-34 analogue); fan/bypass (different engine).

See [[rung31-offdesign-matching]] [[rung32-component-maps]] [[rung33-subsonic-matching]]
[[rung34-spool-transient]] [[rung37-combustor-dynamics]].
