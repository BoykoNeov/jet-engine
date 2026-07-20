# Rung 30 — The choked convergent nozzle: is FULL EXPANSION earned?

Every thrust number since rung 2 has expanded the nozzle **fully** to ambient `p9 = p0`.
At the design point that means `M9 = 1.86` — **supersonic**. A plain **convergent** nozzle
physically cannot reach a supersonic exit: it can accelerate the flow only to `M = 1` at
its throat, and there the throat *is* the exit. So the "fully expanded" nozzle the cycle
has quoted all along is silently a **converging–diverging** (or ideal) nozzle. Nobody ever
asked whether a real fixed convergent nozzle — the standard choice for a subsonic engine,
and the one whose *fixed throat area* rung 31 (off-design) needs — could do it.

Rung 30 asks it the way rung 29 asked whether **freezing the turbine** was earned: **bracket
the idealization.** Ideal full expansion (the shipped nozzle) vs a fixed convergent nozzle
that must choke. Zero new knobs, no rate.

**The verdict: NOT earned at this pressure ratio — the convergent nozzle chokes, and the
"full expansion" idealization is worth 6.6% of specific thrust** — of which the
underexpanded pressure-thrust term recovers more than half of the raw momentum deficit. The
reason the loss is only 6.6% and not the ~40% the halved exit velocity would suggest is the
rung.

---

## What is actually new: the flow decides `p9`, geometry does not expand it

The shipped `Nozzle` expands to a **specified** exit static pressure `p_exit` (default
`p0` ⇒ fully expanded — the rung-1/2 case). That is a *told* back-pressure: it models a
nozzle whose area schedule is free to reach it (C-D, or ideal). A **convergent** nozzle has
no such freedom — its exit is its minimum area, so the exit state is **decided by the
physics**, not specified:

- If the available pressure ratio `pt9/p0` is **below** the critical ratio, the flow stays
  subsonic and the convergent nozzle *does* reach `p9 = p0` — identical to the shipped
  nozzle (**this is the reduce-to-prior branch**).
- If `pt9/p0` **exceeds** the critical ratio, the flow **chokes** at `M9 = 1`; the exit
  cannot fall below the sonic pressure `p*`, so `p9 = p* > p0` — the jet leaves
  **underexpanded**, and the residual `(p9 − p0)` appears as **pressure thrust**, not as
  extra velocity.

So the one structural novelty is a **choke test + branch** in the nozzle. No new solver
elsewhere: the engine's specific-thrust formula already carries the pressure term (it was
validated underexpanded by Mattingly Example 7.1, `p9 = 2·p0`), so a choked exit just feeds
it `p9 = p*`.

### The sonic throat (the TPG velocity↔enthalpy trap, once more)

At `M9 = 1` the exit velocity equals the local speed of sound. In **totals→static**
isentropic form (hot-section properties at the burned composition `f`):

```
V9² = 2·( h_t(Tt9, f) − h_t(T*, f) )          # energy: KE is the enthalpy drop
V9  = a(T*) = sqrt( γ_t(T*, f) · R_t · T* )    # sonic condition, M9 = 1
```

Eliminating `V9` gives one equation in the throat static temperature `T*`:

```
h_t(Tt9, f) − h_t(T*, f)  =  ½ · γ_t(T*, f) · R_t · T*
```

As `T*` falls the left side (enthalpy drop) **rises** and the right side (sonic KE)
**falls**, so the residual is monotone and the root is unique — an inner **bisection** on
`T* ∈ (0, Tt9)`. The critical pressure then follows from the isentropic `pr` ratio, and the
choke test compares it to ambient:

```
p*      = pt9 · pr_t(T*, f) / pr_t(Tt9, f)
CHOKED  ⇔  p* > p0        (equivalently pt9/p0 > pt9/p*, the critical ratio)
```

**CPG closed form** (constant `γ_t`, the reduce target and gate 2):

```
T*/Tt9 = 2/(γ_t+1)
p*/pt9 = ( 2/(γ_t+1) ) ^ ( γ_t/(γ_t−1) )       # e.g. 0.5283 at γ=1.4, 0.5457 at γ=1.3
```

The TPG bisection reproduces this **bit-for-bit** on a self-consistent CPG gas (see gate 2).
On the reacting gas `γ_t` varies through the expansion, so the realized critical ratio
(1.8395 at the design point) legitimately differs from any single-`γ` closed form
(1.8389 at `γ_t(T*)`) by ~0.03% — that difference *is* the variable-`cp` physics, not error.

---

## Result

Design point `π_c = 10`, `M0 = 0.85`, `Tt4 = 1500 K`, real losses. Nozzle entry (station 5,
post-`π_n`): `Tt9 = 1262.69 K`, `pt9 = 314.27 kPa`, ambient `p0 = 50 kPa`, `f = 0.02718`.

| | ideal / full-expansion (shipped) | choked convergent (rung 30) |
|---|---|---|
| exit pressure `p9` | 50.00 kPa (= `p0`) | **170.85 kPa** (= `p*`, underexpanded 3.42×) |
| exit Mach `M9` | **1.864** (supersonic — needs a C-D nozzle) | **1.000** (sonic throat) |
| exit static `T9` | 811.7 K | 1094.7 K |
| exit velocity `V9` | 1039.9 m/s | 642.1 m/s |
| momentum thrust `(1+f)V9 − V0` | 798.4 | 389.8 |
| pressure thrust `(1+f)R_tT9(1−p0/p9)/V9` | 0 | **+356.0** |
| **specific thrust** | **798.4 N·s/kg** | **745.7 N·s/kg** |
| TSFC | 3.404e-5 | 3.645e-5 |

**Full expansion is NOT earned here.** A fixed convergent nozzle chokes hard (`p*` is 3.4×
above ambient), the exit velocity drops 38%, and specific thrust falls **6.6%** with a
matching 7.1% TSFC penalty. This is exactly why a high-pressure-ratio engine either accepts
the loss (simple convergent nozzle, subsonic engines) or fits a **converging–diverging** or
variable nozzle (supersonic engines) — the model has been quoting the C-D answer.

### The finding: the pressure term rescues 87% of the momentum loss

The exit velocity drops 38% (1039.9 → 642.1 m/s), which *alone* cuts the momentum thrust by
**51%** (798.4 → 389.8 N·s/kg). Read only that and choking looks catastrophic. But net
specific thrust falls only **6.6%**, because the energy that did *not* go into velocity did
not vanish — it left as **static-pressure excess**, and a nozzle exhausting into `p0 < p*`
converts that excess into direct **pressure thrust** (`+356.0`), which **recovers 87% of the
408.6 N·s/kg momentum deficit**. The pressure term is not a rounding correction — it is
**48% of the choked nozzle's total thrust**. Full expansion is still the *better* nozzle, but
the convergent nozzle is nowhere near the disaster the raw velocity drop implies, and the gap
between "51% loss" and "6.6% loss" *is* the pressure-thrust term the cycle has carried
honestly since rung 2.

---

## Reduce-to-prior contract

The shipped `Nozzle` behaviour is the **default** and is **untouched** — `convergent=False`
expands to the specified `p_exit` exactly as before. Two reduce statements, both gated:

1. **`convergent=False` (specified `p_exit`) ≡ rung 6 / rung 3 nozzle, bit-for-bit.**
   Structural: the choke branch is only entered when `convergent=True`. The production
   cycle, `main.py`, and every rung-7–29 diagnostic keep the ideal nozzle, so the cycle
   remains **bit-for-bit rung 6** and the rungs-7–29 invariant holds.
2. **`convergent=True` at a *subcritical* pressure ratio ≡ full expansion, bit-for-bit.**
   When `p* ≤ p0` the convergent nozzle reaches `p9 = p0` with `M9 < 1` — the same
   subsonic expansion the shipped nozzle computes. Verified at a low-PR condition
   (`π_c = 2`, ground-idle-like), where the nozzle is genuinely unchoked.

---

## Verification gates (`tests/test_rung30.py`)

1. **REDUCE TO PRIOR (the spine).** (a) `Nozzle(convergent=False)` reproduces the shipped
   nozzle's `M9, T9, V9, p9` bit-for-bit at the design point (structural — same code path).
   (b) `Nozzle(convergent=True)` at a subcritical PR (`π_c = 2`) reproduces full expansion
   `p9 = p0`, `M9 < 1` to 1e-12.
2. **THE SOLVER IS RIGHT (non-tautological).** On a **self-consistent CPG** gas
   (`R_t = (γ_t−1)/γ_t·cp_t`) the TPG sonic bisection reproduces the closed-form
   `T*/Tt = 2/(γ+1)` and `p*/pt = (2/(γ+1))^(γ/(γ−1))` to **1e-9** — two genuinely
   different code paths (enthalpy-integral bisection vs a closed form) onto the same `M=1`
   condition. The `M9 = 1` sonic identity `V9 == a(T9)` holds to 1e-9 on the reacting gas
   too. Without this gate, gate 1 is a tautology (it only exercises the *unchoked* path).
3. **THE VERDICT.** At the design point the convergent nozzle is **choked** (`p* > p0`,
   `M9 == 1` to 1e-9), `p*/p0 > 3`, and specific thrust is strictly **below** the
   full-expansion value by 5–8% (the pressure term keeps it from being ~40%). TSFC is
   strictly higher.
4. **CYCLE UNTOUCHED.** Building the engine with the default nozzle gives bit-for-bit the
   rung-6 stations, `V9`, and thrust (the invariant every rung 7+ depends on).
5. **DIRECTION / PHYSICS.** Choked ⇒ underexpanded (`p9 > p0`), momentum thrust falls but
   the pressure term is strictly positive; the two partially cancel. Asserted inside the
   nozzle on every call (contract #4).

---

## Concessions

- **Fixed throat, no area schedule.** Rung 30 detects the choke and sets the sonic exit; it
  does not carry a throat **area** `A8`/`A9` (only the per-unit-mass thrust, which needs no
  area — the pressure term is `A9(p9−p0)/ṁ` rewritten via the ideal-gas law). The **fixed
  geometry** — a specified `A9` that, with the choked flow function `ṁ√Tt/(A·pt) = const`,
  *pins* the nozzle in off-design matching — is **rung 31's** to add. Rung 30 supplies the
  choke physics that rung 31 stands on.
- **Convergent only.** No C-D / variable nozzle is modelled; the shipped specified-`p_exit`
  path *represents* the ideal-expansion (C-D) answer, and the finding is precisely the gap
  between the two. Adding a real C-D nozzle (a second throat, a design-Mach area ratio) is a
  later option, not this rung.
- **Diagnostic beside the cycle.** Like rungs 14/29, the choked nozzle is offered as an
  alternative and demonstrated in a panel; the production run stays on the ideal nozzle so
  the cycle numbers do not move. Switching the production nozzle to convergent is a
  **re-foundation** (it re-anchors every downstream number), deferred with the off-design
  work.
- **Perfectly/ideally expanded reference.** The 6.6% is measured against the shipped nozzle,
  which is itself the *ideal* (loss-free-expansion, `π_n` aside) C-D limit. A real C-D
  nozzle at this `π_c` would land between the two.

---

## Anchor

`docs/plans/rung30-anchor-choked-nozzle.md`. Two-part, both verified:
- **Textbook critical pressure ratio** (isentropic `M=1`): `p*/pt = 0.5283` at `γ=1.4`,
  `0.5457` at `γ=1.3` — first-rank gas-dynamics table values (Anderson, *Modern Compressible
  Flow*; every propulsion text). The solver reproduces them bit-for-bit on a self-consistent
  CPG gas.
- **Mattingly *Elements of Propulsion* Example 7.1 family** — the same dual gas + losses the
  repo already anchors (`docs/plans/rung2-anchor-mattingly.md`), whose underexpanded
  `p9 = 2·p0` case validated the pressure-thrust term the choked branch reuses.
