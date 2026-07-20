# The resolved-`τ_res` investigation — INVESTIGATED, **NOT SHIPPED**

> **Status.** This is a **negative-result record**, not a rung spec. It is the attack on rung 26's
> open item (a) — *"a resolved `τ_res` from the nozzle area-schedule (retire the last geometric knob
> `L`, **pin** the location)"*. It was derived, prototyped and tested, returned a negative verdict on
> **both** halves of that seam, and was **deliberately not added to the ladder**. It is NOT a rung —
> no `*-spec.md`, no `gas.py`/`main.py`/test code, no entry in the rung table, by design.
>
> **Why this file exists:** so the negative is not re-investigated from scratch, and so the two
> *positive* by-products (rung 26 confirmed; its disclaimed magnitude refined ~3×) are not lost. If
> you are about to "derive `τ_res` from the area schedule," read this first — it was done.

## What this investigation was

Rung 26 (and 27, 28 after it) distributes a single total residence `τ_res = L/(0.6·V9_frozen)`
**uniformly in `s`**, the progress coordinate of the geometric pressure schedule
`p(s) = pt9·(p9/pt9)^s` (so `s` is linear in `ln p`). Rung 26 named both halves of this as a
simplification: the *uniform-in-`s`* distribution, and the unanchored geometric knob `L`.

This prototype derived the distribution from the nozzle **area schedule**.

**The derivation.** `dt = dx/V`; for a conical nozzle `A = πr²`, `dx = dr/tanθ`,
`dr = dA/(2√(πA))`. Mass conservation gives `A = ṁ·a` with `a(s) ≡ v̂(s)/V(s)` (specific volume over
velocity — **both already marched** in `_freeze_out_expand`, `V` off the `dh = v·dp` energy spine, so
this needs no new state). Then

```
  dt = ( √ṁ / (2·tanθ·√π) ) · [ |da/ds| / (√a · V) ] ds
```

`√ṁ` and `tanθ` enter only as constant prefactors, so they **cancel in the normalized shape**

```
  ĝ(s) ∝ |da/ds| / ( √a(s) · V(s) ) ,      ∫ ĝ ds = 1
```

That cancellation is real, and it was the whole appeal: a residence *distribution* that is a pure
output of the `(ρ,V)` expansion profile, with no knob and no `ṁ`. **It is also moot** — see below.

Scope tested (the minimal one): reshape the **distribution** only, holding rung-26's **scale**
`τ_res` fixed, so `Da_local(s) = τ_res·ĝ(s)/τ_chem(s)`. Design point as rung 26
(`π_c=10`, `M0=0.85`, `T0=250 K`, `p0=50 kPa`, standard losses), `Tt4 ∈ {1500, 1800, 2200} K`.

## The two coordinate pathologies — why `L` cannot simply be retired

The `s` frame inherited from rung 26 is a *pressure* schedule. It cannot carry a physical `dt`:

- **Entry divergence (fatal).** The march begins at **stagnation** (`Tt9, pt9`, `V=0`). Near `s→0`:
  `V ∝ √s` and `a ∝ s^{-1/2}`, so `ĝ ∝ s^{-7/4}` — **non-integrable**. The normalization
  `∫ĝ ds` **does not exist** without an **entry cutoff**, i.e. an entry Mach `M_e` and hence an entry
  plane. The cycle carries no such quantity. *(Confirmed numerically: the near-entry residence
  fraction grows without limit under grid refinement.)*
- **Throat zero.** `a = v̂/V` is minimal at the throat, so `|da/ds| → 0` and `ĝ` has a spurious zero
  there. The throat is real and resolved: `s_throat ≈ 0.333 / 0.300 / 0.278` at
  `Tt4 = 1500/1800/2200`, with `a_exit/a_min ≈ 1.56 / 1.74 / 1.92`.

**This is the negative on "retire `L`."** Deriving the distribution does not remove a geometric
input — it **adds** one. And the trade is strictly for the worse: because the integrand piles up at
the entry, the answer is **more** sensitive to `M_e` than rung 26 ever was to `L`.

## The deciding computation

Reshaped march, rung-26 scale preserved (`ĝ` normalized over `[s_e, 1]` so total residence is still
`τ_res` — **only the distribution is under test**), at two entry cutoffs. `s_e` from `M_e` via the
isentropic `p_e/pt9` at `γ=1.3` (used **only** for the cutoff, in no marched quantity).

| `Tt4` | `s_throat` | rung-26 uniform `s_freeze` | `M_e=0.3` (`s_e`) | `M_e=0.5` (`s_e`) |
|---|---|---|---|---|
| 1500 | 0.333 | 0.000 | 0.035 (0.032) | 0.090 (0.087) |
| 1800 | 0.300 | 0.118 | 0.090 (0.029) | 0.115 (0.080) |
| 2200 | 0.278 | 0.378 | 0.155 (0.027) | 0.190 (0.074) |

**Motion span:** uniform **0.378** → reshaped **0.120** (`M_e=0.3`) / **0.100** (`M_e=0.5`).

**The location is slaved to the cutoff.** At `Tt4=1500`, `s_freeze ≈ s_e` at *both* cutoffs
(0.035 vs 0.032; 0.090 vs 0.087). The freeze point is not pinned by the derived shape — it is pinned
by the knob the derived shape forced us to introduce. **This is the negative on "pin the location."**

### The convergence gate (blocking, and it PASSES)

`ĝ` is steep just above `s_e` and the normalization integral is dominated by that region, so the
entry-relative motion `s_freeze − s_e` had to be shown grid-independent before it could be claimed.
Refining the march and shape grids together:

| nstep/ngrid | `M_e=0.3` span | `M_e=0.5` span |
|---|---|---|
| 200/400   | 0.1245 | 0.1124 |
| 400/800   | 0.1270 | 0.1149 |
| 800/1600  | 0.1270 | 0.1137 |

**Converged**, to ≈0.127 and ≈0.114 — a **~11% cross-cutoff spread**. So the entry-relative motion
is real, not a normalization artifact.

## The positive by-products — rung 26 CONFIRMED, its disclaimed magnitude refined

Rung 26 certified only that the freeze motion **exists and rises with `Tt4`**, and **explicitly
disclaimed** the location/magnitude ("rides on `L`"). Both survive the reshape, so this is a
**confirmation**, not a correction of anything rung 26 claimed:

- **Existence and direction hold** — `s_freeze` still rises monotonically with `Tt4`, under both
  metrics and both cutoffs.
- **Lean dormancy holds** — at `Tt4=1500`, `s_freeze − s_e ≈ 0.001–0.003`: still *frozen from the
  entry plane*, reproducing rung 26's dormant-lean / earns-its-keep-hot arc.
- **The disclaimed magnitude refines ~3× smaller** (span 0.378 → ≈0.12). Refining a quantity the rung
  never asserted is not a break — it is exactly what the disclaimer was for.
- **A bonus correction.** Under uniform residence the hot case froze *downstream* of its throat
  (0.378 vs `s_throat`=0.278). With residence treated correctly, **every** freeze point lies
  upstream of its throat, so the freeze region does **not** cross the throat as `Tt4` moves — and
  the `ĝ`-zero-at-the-throat pathology never bites in practice.

**Disclaimed, explicitly** (the rung-28 `β` precedent — state the weak point, don't bury it): the
`Tt4=1800` **midpoint** wobbles ~1.7× with the cutoff (`s_freeze − s_e` = 0.057 at `M_e=0.3` vs
0.034 at 0.5). Only the **endpoints and the span** are cutoff-robust.

## What this means for the seam (the corrected takeaway)

Rung 26's open item (a) asked for two things and **gets neither**:

- **Retire `L`** — no. A physical residence distribution **cannot be derived from the pressure
  schedule alone**. It needs a resolved `A(x)` **and** an entry Mach: *more* geometric input, not
  less. The `ṁ`/`tanθ` cancellation is genuine but **moot**, because the normalization it feeds does
  not converge.
- **Pin the location** — no. The location merely changes which knob it rides on, from `L` to a `M_e`
  it is *more* sensitive to.

(Precedent: rung 18's "a 0-D transport **cannot derive** `C_opt`" — a shipped negative of the same
shape. The seam over-promised; this is the honest ceiling.)

**Do NOT** re-run: the conical-`ĝ(s)` reshape on the rung-26 `s` frame with a hand-picked entry
cutoff. That is exactly this investigation, and its answer is above. **A new attempt is only
worthwhile** if it brings a **genuine nozzle geometry** — a real `A(x)` schedule with a physical
entry plane and throat area (which in turn wants `ṁ` threaded into the diagnostic and a *choked*
nozzle, itself still a deferred seam) — i.e. the thing that would let residence be integrated in
`x` rather than reweighted in `ln p`.

One further concession, independent of all the above: `ĝ(s)` assumes the **conical** geometry
family. A bell/contoured nozzle gives a different shape.

## Reproduction

Probes lived in `M:\claud_projects\temp\rung29-tau-res\` (`probe_shape.py` — the throat and the
`s^{-7/4}` divergence; `probe_decide.py` — the reshaped march, `--conv` for the convergence gate;
`FINDINGS.md`). That folder is **outside git** (project temp policy), so this tracked file is the
durable record. The probes read the shipped rung-26 trajectory through the **rung-28 pure-observer
`record=` hook** on `_freeze_out_expand` and injected the reshaped clock through its existing
`da_local_fn` callable seam — i.e. **nothing in the shipped code was changed.**
