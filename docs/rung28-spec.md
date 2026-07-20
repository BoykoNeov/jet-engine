# Rung 28 — the rung-26-coupled NO march: confirming rung 27, correcting both its reasons

Rung 27 read its NO clock on the **frozen** station-4 pool along rung-14's frozen isentropic path, and
deferred the coupled march with a one-line prediction:

> "NO riding the *relaxing* pool, so `[O],[H]` deplete under it. This **can only** *slow* NO further
> (radical-poorer ⇒ larger `τ_NO`), moving the answer **deeper into frozen**; it is a secondary
> refinement, not the load-bearing claim."

Rung 28 builds it. **Rung 27's verdict is CONFIRMED — and both of the reasons it gave are WRONG.**

> **Read `docs/rung27-spec.md` and `docs/rung26-spec.md` first**, plus
> `docs/plans/rung28-anchor-coupled-no-march.md` (the probe, the two-channel table, the limit sweep).
> **No new chemistry, no new species, no new constants** — every rate is rung 26's (GRI-Mech 3.0) or
> rung 7's (Zeldovich reverse). NO is trace and never perturbs the flow; the coupling is **one-way**
> (pool → NO, never NO → pool). A **pure diagnostic beside the cycle**: the cycle stays bit-for-bit
> rung 6, and rungs 26/27 are bit-for-bit untouched.

---

## THE HEADLINE IS STRUCTURALLY UNREACHABLE — and that is the first thing to establish

The nozzle-**entry** state (`comp_entry`, `Tt9`, `pt9`) is an **input**, not a marched quantity. So
`Da_NO` at entry is **bit-for-bit rung 27's at every `Tt4`** (gate 3, asserted as `==`). Since rung 27's
finding *is* "frozen from entry", **no coupling can touch it.** Everything below refines the *mechanism*
inside a verdict that cannot move. Build boldly, claim narrowly.

## Correction 1 — "can ONLY slow NO further" is one-sided: there are TWO opposing channels

Coupling to rung 26 couples to **all** of rung 26 — including that recombination is **exothermic**.

| channel | mechanism | on `τ_NO` | on the verdict |
|---|---|---|---|
| **(1) radical depletion** | `[O],[H]` recombine on the relaxing path | `τ_NO ∝ 1/([O],[H])` **rises** | **DEEPER** frozen |
| **(2) heat release** | recombination heat ⇒ `T(s)` **above** the frozen isentrope | this clock is **Arrhenius** (`θ≈20820/24560 K`) ⇒ `k` **rises** | **LESS** frozen |

Rung 27 saw only channel 1. **A composition-only coupling would structurally exclude channel 2 and
thereby manufacture rung 27's prediction** — which is why `_coupled_no_march` takes a full
`[(s,p,T,comp)]` **trajectory**, and why the two **hybrids** (frozen-T + coupled-comp; coupled-T +
frozen-comp) are first-class outputs rather than probe artifacts.

Measured at the nozzle exit, anchored rate (`Da_NO(exit)` relative to rung 27's):

| `Tt4` [K] | `ch1` (depletion) | `ch2` (heat release) | **net** | `\|ln ch2/ln ch1\|` |
|---|---|---|---|---|
| 1500 | 0.880 | 1.000 | **0.880** | 0.003 |
| 1800 | 0.634 | 1.009 | **0.640** | 0.020 |
| 2200 | 0.382 | 1.130 | **0.432** | 0.127 |
| 2400 | 0.379 | 1.594 | **0.604** | **0.481** |

**Rung 27's CONCLUSION holds** (`net < 1` everywhere — deeper into frozen). **Its MECHANISM does not:**
the opposing channel is real, grows **monotonically**, and cancels **~half** the depletion effect at the
hot edge. It even makes the **net trend non-monotone** in `Tt4` (deepest ~2200–2300 K, reversing by
2400). *That* turnaround is **NOT claimed** — it rides on how far the pool relaxes, i.e. on `L/τ_res`,
exactly as rungs 26/27 disclaim `s_freeze`. The **monotone rise of `|ln ch2/ln ch1|`** is the certified
trend (a ratio of two channels on the *same* path — far more robust than either alone).

## Why depletion wins anyway — UNBOUNDED vs SATURATING (the load-bearing argument)

Both channels are switched on by the *same* thing (how far the rung-26 pool relaxes), so the honest test
is the **limit**: drive `pool_rate_scale` up — *maximum* heat release **and** *maximum* depletion — and
see whether the net can flip. At `Tt4=2200 K`:

| `pool_rate_scale` | `ch1` | `ch2` | **net** |
|---|---|---|---|
| 1e0 | 0.3821 | 1.1302 | 0.4318 |
| 1e2 | 0.0053 | 1.2106 | 0.0064 |
| 1e4 | 0.0002 | 1.2136 | 0.0003 |
| 1e6 | 0.0001 | 1.2138 | **0.0001** |

- **Channel 1 is UNBOUNDED.** `τ_NO ∝ 1/([O],[H])`, and as the pool equilibrates `[O],[H]` fall to their
  *local equilibrium* values, which **crater** on cooling. `ch1 → 0`, no floor.
- **Channel 2 SATURATES.** The heat release is capped by the **finite chemical enthalpy** frozen into the
  dissociation — its ceiling is the rung-14 equilibrium-vs-frozen exit-`T` gap. `ch2` stops (×1.02 /
  ×1.21 / ×2.14 at 1800/2200/2400 K).

So **at any chemistry faster than anchored, depletion wins decisively.** Rung 27's conclusion is robust
for an **asymptotic** reason, not a numerical coincidence. **This is the rung's load-bearing claim.**

## Correction 2 — the β repair: rung 27's bound is right, its justification was false

`_tau_no_destroy` is the `a≫1` limit of rung 7's rate. Rung 27 justified that with:

> "That is the clamp-relevant regime: exhaust NO arrives **SUPER-equilibrium** (rung 14/17), so its fate
> is destruction, not formation."

**That premise is FALSE where it is needed.** At the nozzle **entry**, `a = x_no_frozen/x_no_e(Tt9)`:

| `Tt4` [K] | `a` **entry** | `a` **exit** |
|---|---|---|
| 1500 | 2.09 | 250 |
| 1800 | **0.61** | 33.0 |
| 2200 | **0.31** | 7.47 |
| 2400 | **0.45** | 7.98 |

NO arrives **SUB-equilibrium** for `Tt4 ≥ 1800 K` — below the local ceiling at the hot entry, so its
initial drift is **formation**, not destruction. It becomes super-equilibrium only as the gas **cools**
(the exit, where rungs 14/17 read the clamp — *there* the premise holds). But **freeze-from-entry is
decided exactly where the premise fails.**

What actually holds is **β < 1**. With `β = R1/(R2+R3)` and `u = βa`, the exact linearisation gives

```
  τ_exact / τ_surrogate = (1+u)² / [ (1+u)² − (1−β²) ]   >  1   for all a ≥ 0,  whenever β < 1
```

→ 1 as `a→∞`, → `1/β²` as `a→0`. So the surrogate is the **fast asymptote approached from above**: a
**uniform LOWER bound on τ** — an **UPPER bound on the rate** — in **both** regimes. "The surrogate says
frozen" ⇒ "truly frozen", whether NO is forming or destroying. **Rung 27's numbers are unaffected**
(the surrogate is conservative, which is exactly what its bound claim needed); only its stated *reason*
was wrong. `_tau_no_destroy` carries an ERRATUM to this effect; rung 27 is otherwise not rewritten.

**The honest weak point.** β<1 is **empirical on this band, not a theorem**, and it is the least
comfortable margin in the rung: max β over the path runs **0.103 → 0.261 → 0.512 → 0.513** for
`Tt4 = 1500 → 1800 → 2200 → 2400 K`. It **rises with `Tt4`** (the same direction as rung 27's narrowing
separation) and reaches **half** the β=1 threshold at the hot end — a factor 2, not orders. It stays
`< 1` everywhere the cycle runs, and `Tt4 ≥ 2500 K` **has no cycle solution at all** (the rung-6 burner
balance assert fires), so no crossing is reachable — but a hotter or higher-`π_c` cycle is where this
would want re-checking. Gated (`test_beta_margin_is_disclosed_not_comfortable`), not buried.

## A third, smaller correction — the entry margin is band-specific

Rung 27 quotes a **"3–9 order"** entry margin. That is measured on **its** band (topping at 2200 K).
Extended to 2400 K the entry `Da_NO` climbs to **2.1e-2** — still frozen, but only **~1.7 orders** clear.
The verdict is unchanged; its *comfort* is not uniform. Gated as a monotone thinning trend.

## Read the ratios correctly — the CLOCK's depth, not NO's motion

`net = ×0.43` does **not** mean NO moved by 43%. **NO does not move at all**: `Da_NO` stays orders below
1 on every path here, so `relaxed_fraction ≈ 0` throughout. The ratios say **how much further below the
freeze threshold the coupling pushes the clock**. Fate: *frozen*. Mechanism: *these ratios*.

`relaxed_fraction` goes slightly **negative** (≈ −1e-3) at `Tt4 ≥ 1800 K`. Physical, not a bug: the
sub-equilibrium entry means the first (tiny) drift is **upward** (formation). The β<1 bound covers that
direction too.

## The interlock with rung 26

Rung 26's pool freeze point `s_freeze` **gates this entire rung**: it sets how far the pool relaxes,
hence how strongly *both* channels switch on. Lean (`s_freeze=0`, frozen from entry) the coupling is
nearly inert (`net=0.880`); hot (`s_freeze=0.38`) it bites (`net=0.432`). **The coupling matters exactly
where rung 26 is most alive** — which is the concrete form of the "the two rungs interlock" note rung 27
gestured at.

## The reduce contract

- **`coupled_no_freeze_out_nozzle` is a NEW method**; rungs 1–27 are bit-for-bit unchanged (gate 1). The
  trajectory recorder added to `_freeze_out_expand` is a **pure observer** (gate 1b asserts every
  returned float is identical with and without it).
- **`couple=False` ⇒ rung 27's march BIT-FOR-BIT** (LOAD-BEARING). Structural, not numerical: feeding
  `_frozen_no_trajectory` makes `_coupled_no_march` execute the *identical expression sequence* as
  `_no_freeze_out_expand`. This is why the relaxer takes a trajectory instead of hard-wiring a path —
  one energy-spine march could not be bit-for-bit to *both* rung 26 and rung 27.
- **`rate_scale → 0` ⇒ the rung-14/17 clamp `max_a` bit-for-bit** (as rung 27).
- The clamp **denominator is held on rung-14's FROZEN `T9`** deliberately: the coupled exit is warmer, so
  an equilibrium NO read there would move `max_a` for a purely **thermodynamic** reason and entangle it
  with the kinetic finding.

## Verification gates (priority order)

1. **RUNGS 26/27 UNTOUCHED** — bit-for-bit across a rung-28 call; the recorder is a pure observer.
2. **REDUCE (LOAD-BEARING)** — `couple=False` ⇒ rung 27 bit-for-bit; `rate_scale→0` ⇒ the clamp.
3. **HEADLINE SAFE** — entry `Da_NO` `==` rung 27's at every `Tt4`; `frozen_from_entry` everywhere; the
   margin thins monotonically (recorded, not hidden).
4. **BOTH CHANNELS REAL** — `ch1<1`, `ch2>1`, pool warmer, radicals depleted; `|ln ch2/ln ch1|` rises
   monotonically to >0.3.
5. **CONCLUSION SURVIVES** — `net < 1` at every in-band `Tt4`.
6. **STRUCTURAL WIN** — over 6 orders of `pool_rate_scale`: `ch1` monotone → <1e-3 (unbounded), `ch2`
   monotone but **saturating** (last decade moves it <1e-3); depletion dominates at every `Tt4`.
7. **THE β REPAIR** — sub-equilibrium entry for `Tt4≥1800 K`, super-equilibrium exit, `β<1`,
   `τ_exact ≥ τ_surrogate` pointwise, and the closed form matches to 1e-9.
8. **NO DOES NOT MOVE** — `|relaxed_fraction| < 1e-2`; `max_a ≈ max_a_frozen`; the clamp still fires.
9. **CYCLE UNTOUCHED + GUARDS.**

## NOT claimed

- **The net turnaround's location** (~2200–2300 K). It rides on `L/τ_res`, as rungs 26/27 disclaim
  `s_freeze`. Only the **monotone channel ratio** is claimed.
- **β < 1 as a theorem.** Empirical on the runnable band, with the margin disclosed above.
- **Any change to rung 27's numbers.** The surrogate is conservative; only its justification is repaired.
- **A coupled *pool* answer.** The coupling is one-way; NO never feeds back (it is trace).

## The honest character of this rung

Rung 27 was a confirmation. Rung 28 is a **confirmation of a confirmation whose reasoning did not
survive contact** — the verdict was right twice over, and both of its stated justifications were wrong:
one incomplete (the missing heat-release channel) and one false-where-it-matters (super-equilibrium at
entry). That is the project's thesis in miniature: **the deliverable is understanding, and a right answer
held for wrong reasons is not yet understanding.** The rung's positive content is the replacement
reasoning — *unbounded-vs-saturating* for the coupling, *β<1* for the clock — both of which are stronger
than what they replace, because neither depends on the regime being what rung 27 assumed.

## Deferred (kept additive)

- **A coupled *formation* clock.** Where NO arrives sub-equilibrium the physical drift is formation; the
  `a≫1` surrogate bounds it but does not *describe* it. An exact-`a` march would, at the cost of the
  `[NO]_e`-independence that makes the present clock robust. Moot while `Da_NO ≪ 1`.
- **A resolved `τ_res`** (inherited from rungs 26/27) — retire `L`. Would also pin the net turnaround
  this rung disclaims.
- **β at higher `π_c` / hotter cycles** — the one margin here that is a factor rather than orders.
