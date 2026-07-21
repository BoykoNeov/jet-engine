---
name: rung33-subsonic-matching
description: "SHIPPED rung 33 = the subsonic-nozzle matching branch below unchoke; the INVERSION of rung 31 (subsonic tau_t varies even on CPG); closes rung 31's deferred dual-mode seam"
metadata: 
  node_type: memory
  type: project
  originSessionId: 58610487-9be4-499a-bab0-3ad35df1eac6
  modified: 2026-07-21T10:29:06.133Z
---

SHIPPED rung 33 = **the subsonic-nozzle matching branch**. Closes the seam rung 31 named
and deferred (it flagged the nozzle-unchoke boundary `Tt4≈600` at design and said
"Mattingly's dual mode, deferred"). See `docs/rung33-spec.md`,
`docs/plans/rung33-anchor-subsonic.md`, `tests/test_rung33.py`.

**What it is:** below unchoke the nozzle is SUBSONIC (`p9=p0`, `M9<1`), so only the **NGV
stays choked** — rung 31's two-choke pin `(★)` is void. The nozzle passes the
compressible-flow `MFP(M9)` with `M9=M9(pt9/p0)`, and `pt9/p0` moves with `π_c`, so `π_t`
becomes the **equilibrating unknown** matching NGV-choked supply to subsonic demand
`(★★): resid(π_t)=ṁ_NGV−ṁ_noz=0`. A 1-D root-find with rung 31's `(f,pt4)` fixed point
nested inside. `OffDesignMatcher._match_subsonic`, auto-dispatched from `.match` (checks the
rebuilt nozzle; only fires subsonic). `OffDesignResult.branch` = "choked"/"subsonic".

**THE RUNG = the INVERSION of [[rung31-offdesign-matching]].** Choked branch: `τ_t` coupled
through the `γ_t(T)` curve (var-cp, 2nd order) → drifted on reacting gas but **machine-constant
on CPG** (rung 31 gate 2). Subsonic branch: coupled through **`π_c`** (structural, 1st order) →
`τ_t` **VARIES even on a CPG gas** (~1.2% across window, rises toward 1). The effect that DIED
on CPG for the choked branch is FIRST-ORDER and ALIVE on the subsonic branch. Rung 31's "the
turbine does not know the operating condition changed" holds **only while both throats choke**.
On the reacting gas the composition drift muddies it (partly cancels) → the CPG isolation is the
clean statement.

**Advisor framing correction (load-bearing):** the coupling is to the RATIO `π_c` via `pt9/p0`,
**NOT** ambient `p0`. Cycle is pressure-homogeneous deg-1 → ratios `p0`-invariant to machine
zero (gate 6). I had it wrong initially ("re-couples to ambient p0"); advisor caught it.

**Envelope — TWO boundaries:** ABOVE = nozzle-unchoke (`Tt4≈600` at M0=0.85; **widens at low
ram** — CPG unchokes at `Tt4≈820` at M0≈0.10, the idle-descent regime); BELOW = **thrust-neutral
idle** (`Tt4≈440`; `π_c→1`, `(1+f)V9→V0`, net thrust→0; below it the engine windmills → SUB-IDLE,
reported not force-fit). The lower bound surfaced as a `_score` cascade crash (negative eta_o at
net-drag) — guarded it in `_match_subsonic` (compute sp_thrust, assert >0) rather than touching
the shared `_score`.

**Reduce:** choked path left **LITERALLY unchanged** ⇒ choked points bit-for-bit rung 31 (31/32
suites pass unchanged, 14/14 — the bit-for-bit witness). **Gate 4 (advisor's load-bearing catch):**
my first gate 4 (checking the shipped point satisfies textbook `MFP(M9)`/isentropic identities) was
**TAUTOLOGICAL** — those are algebraic identities the Nozzle satisfies by construction, and gate 1
(reduce-to-design) is a CHOKED point that returns before dispatch, so the subsonic solve had NO
independent anchor. Fixed to an **independent CPG closed-form solve of (★★)** (pure algebra, no
`_sonic_throat`/`Nozzle`) reproducing the shipped `(π_t,π_c,τ_t,M9)` to machine zero — verified it
catches a 1% `π_c` corruption gates 1/2 miss. Precedent: mirrors rung 31 gate 2 / rung 29's "without
which the reduce gate is a tautology". Advisor caught this post-commit; fixed in a follow-up commit.

**Out of scope:** subsonic + component map — `MapMatcher` overrides `match`, stays choked-only.
NGV assumed choked. Spool-down/windmilling **transient dynamics** below idle = next seam (now that
both `N` from [[rung32-component-maps]] and the subsonic branch exist). Afterburner still open.
