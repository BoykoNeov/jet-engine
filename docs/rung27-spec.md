# Rung 27 — NO freeze-out: is the frozen-NO ASSUMPTION earned?

Every NO number since **rung 7** has assumed the station-4 exhaust NO freezes through the nozzle. The
**rung-14/17 dropped-clamp corollary** reads `max_a = x_no_frozen/x_no_e(T9) ≫ 1` **off that
assumption** — the numerator is frozen *by fiat*. Rung 26 then anchored a recombination clock and
showed the **major** pool does **not** freeze at entry (`s_freeze` walks 0.12→0.38 hot). So the
assumption is now visibly unearned for the majors. **Rung 27 asks the same of NO** — and, applying
rung-26's exact machinery (an anchored clock → a local `Da(T,p)` → the same relaxation march) to a NO
clock built from **rung 7's own Zeldovich reverse rates**, finds the assumption is **EARNED**.

**THE HEADLINE IS A DERIVED ASSUMPTION + AN INVERTED KILL TEST — NOT a moving freeze point.** Rung 26's
signature finding (a shut-off point that moves with `Tt4`) has **no analogue** here: `Da_NO ≪ 1` from
the nozzle entry at *every* `Tt4`, so `s_freeze_NO ≡ 0` and nothing moves. We do **not** manufacture
one. What rung 27 delivers instead:

1. The frozen-NO assumption carried since rung 7 is **derived**, on an **upper bound**, robust to the
   clock's regime and to the NO level.
2. The **kill test INVERTS rung 26's**: two anchored clocks in the same nozzle, freezing for
   structurally **opposite** reasons.
3. The Da_NO-vs-Da_recomb **separation NARROWS with `Tt4`** — the honest quantified trend, with **no
   crossing** claimed.

> **Read `docs/rung26-spec.md` first** (the anchored-clock / local-`Da` pattern, `_freeze_out_expand`,
> the kill-test discipline) and `docs/plans/rung27-anchor-no-freeze-out.md` (the go/no-go probe, the
> `Da_NO` table, the kill legs). This file states only what *changes*. **No new chemistry, no new
> species, no new bound** — NO is trace and never perturbs the flow. A **pure diagnostic beside the
> cycle**: the cycle stays bit-for-bit rung 6.

---

## The anchor — rung 7's OWN Zeldovich reverse rates, zero new constants

The NO clock uses the extended-Zeldovich rate constants already at `gas.py:750` (Hanson & Salimian
1984, as tabulated in Turns), **already K-checked in-code** against the project's `a6/a7` thermo by
`_kcheck_ratio` (detailed balance ties the transcribed rates to the thermo; measured 1.035–1.044).
**Zero new constants**, exactly as rung 26 reused GRI-Mech 3.0.

| Reaction (reverse) | A | n | θ (K) |
|---|---|---|---|
| `2r: NO + O → N + O2` | 3.8e9 | 1.0 | **20820** |
| `3r: NO + H → N + OH` | 1.7e14 | 0.0 | **24560** |

**The contrast with rung 26's clock is the rung.** Rung 26's recombination sinks all have `Ea = 0`;
these have a **large thermal barrier**.

## Derive before you code — the governing relations

**1. The clock (and why it is `[NO]`-independent).** From the rung-7/10 reverse-rate form
`d[NO]/dt = 2R1(1−a²)/(1+βa)` with `a=[NO]/[NO]_e`, `β=R1/(R2+R3)`, the exact local linearised
relaxation time is `τ_NO = −1/(df/dc) = [NO]_e(1+βa)²/(2R1(2a+βa²+β))`. In the **super-equilibrium**
limit the clamp cares about (`a ≫ 1`) this collapses to

```
  τ_NO → 1 / ( 2 ( k2r[O] + k3r[H] ) ) = 1 / ( 2 c_tot ( k2r·x_O + k3r·x_H ) )
```

**independent of `[NO]_e` AND of `a`** — the `[NO]_e` inside `R2/R3` cancels the `a` in the numerator.

*Physical justification:* super-equilibrium NO is destroyed by its reverse reactions with the radical
pool `[O],[H]`; the destruction time is one over those pseudo-first-order rates. That the clock does
**not** carry `[NO]_e` means **the freeze answer does not depend on which frozen NO level is fed in** —
a robustness property rung 26's clock (whose `x_OH` *is* the whole radical pool) does not have. The
clock depends on `[O],[H]` — **the very radicals rung 26's clock destroys** — so the two rungs
interlock (§ deferred).

**2. The local Damköhler number (the switch).** Identical to rung 26, on the same `τ_res`:

```
  Da_NO(s) = rate_scale · τ_res / τ_NO(T(s), p(s); comp),   τ_res = L/(0.6·V9_frozen)
```

**3. The freeze criterion — and why nothing moves.** `Da_NO < 1` ⇒ `relax → 0` ⇒ NO holds at whatever
it had. The finding (§ gate) is `Da_NO ≪ 1` from the entry at *every* `Tt4`, so the crossing
`Da_NO = 1` never happens — **frozen from entry, always**. That is the derivation of the rung-7
assumption; there is no `s_freeze` to report.

**Bound.** Evaluated on the **frozen station-4 pool**: radical-rich (`[O],[H]` not yet recombined) and
`τ_NO ∝ 1/([O],[H])` ⇒ the **fastest possible** NO relaxation ⇒ `Da_NO` **upper** bound. If NO freezes
here it freezes *a fortiori* on the real radical-depleting path. Same bounding logic rung 26 used for
`x_OH`.

## The march — a trace scalar on rung-14's frozen path

Unlike rung-26's `_freeze_out_expand` (which marches the whole reacting vector on the `dh=v·dp` energy
spine), `_no_freeze_out_expand` marches a **single scalar** — the NO mole fraction — along rung-14's
**frozen isentropic** nozzle path (composition fixed at `comp_entry`; the frozen temperature at each
step solved by the *same entropy bisection as `_expand_nozzle`*, so the exit `T9` and hence the clamp
denominator match `nozzle_flow` bit-for-bit). At each step NO relaxes toward the local equilibrium:
`x_no ← x_no + (1 − exp(−Da_NO·ds))·(x_NO_e(T) − x_no)`.

## The kill test — the two terms AGREE; this INVERTS rung 26

`τ_NO = 1/(2 c_tot(k2r(T) x_O + k3r(T) x_H))` — two clean factors, no `[NO]_e`. On the frozen pool
pinned, `Tt4=2200` (`T` 1992→1214 K, `p` 427→50 kPa):

| leg | over the expansion | effect on `τ_NO` |
|---|---|---|
| kill T (`k` pinned ⇒ density alone) | `c_tot` craters | τ ×5.2 — **DRIVES** freezing |
| kill p (`c_tot` pinned ⇒ `T` alone) | `k` craters (Arrhenius) | τ ×1.6e3 — **DRIVES** freezing |

**Both legs drive; Arrhenius dominates density ~300–2000×.** The inversion, precisely:

|  | rung 26 (recombination) | **rung 27 (NO)** |
|---|---|---|
| barrier | `Ea = 0` ⇒ `k` **accelerates** on cooling | `θ ≈ 20820/24560 K` ⇒ `k` **craters** |
| molecularity | **ter**molecular ⇒ `c_tot²` | **bi**molecular ⇒ `c_tot¹` |
| the two terms | **OPPOSE** — density wins *despite* `T` | **AGREE** — both drive |
| kill p | Da **rises** (no freeze) | Da **falls** (freezes) |
| ⇒ | freezes mid-nozzle, point **moves** | freezes **at entry**, ~10³–10⁹ margin |

Same nozzle, two anchored clocks, opposite mechanism structure. The gate re-runs rung 26's clock on
the same path to show its kill-p makes `τ` *shrink* — the sign rung 27 inverts.

## The reduce contract — to rung 14/17's clamp, stated loudly

- **`no_freeze_out_nozzle` is a NEW method** beside `nozzle_flow` (rung 14) and `freeze_out_nozzle`
  (rung 26); the whole rung-1..26 suite is bit-for-bit unchanged (gate 1).
- **`Da_NO ≡ 0` ⇒ marched NO == frozen NO == rung 14/17's `max_a`, BIT-FOR-BIT** (LOAD-BEARING). Driven
  with a literal-zero `da_no_fn`, `relax = 1−exp(0) = 0`, so `x_no` never moves; the clamp `max_a` lands
  on `nozzle_flow`'s fully-frozen number to the ULP (the exit `T9` matches `_expand_nozzle` because the
  bisection is identical). Also the drift tripwire for the duplicated frozen-path solve.
- **`rate_scale → ∞` ⇒ NO tracks equilibrium ⇒ `max_a → 1`** (the clamp goes DORMANT). The counterpoint:
  the clamp fires *only* because the real `Da_NO` is tiny.

## Verification gates (priority order)

1. **RUNG 14/26 UNTOUCHED** — a new method; `nozzle_flow` and `freeze_out_nozzle` are bit-for-bit
   across the call.
2. **REDUCE — `Da_NO` off ⇒ rung 14/17 clamp BIT-FOR-BIT** (LOAD-BEARING): both the `rate_scale→0`
   limit and a literal-zero `da_no_fn` reproduce `nozzle_flow`'s `max_a`; the march exit `T9` ==
   `_expand_nozzle`'s.
3. **LIMIT — `rate_scale→∞` ⇒ clamp DORMANT** (`max_a→1`, `relaxed_fraction→1`).
4. **THE FINDING — FROZEN FROM ENTRY at EVERY `Tt4`**: `Da_NO < 1` at entry for all `Tt4` (unlike rung
   26's major pool, frozen-from-entry only lean); `|relaxed_fraction| ≪ 1`. The gate contrasts against
   a `freeze_out_nozzle` call at the same hot point (majors relax; NO does not).
5. **THE KILL TEST — both terms AGREE (drive), INVERTING rung 26**: kill-T and kill-p both grow `τ_NO`;
   rung 26's kill-M on the same path shrinks `τ` (opposite sign).
6. **MARGIN TREND — separation NARROWS with `Tt4`, no crossing**: `τ_NO/τ_recomb` at entry strictly
   decreases (collapses ≥ 10³ across the band); `Da_NO` entry monotone-up but `< 1` everywhere.
7. **CLAMP EARNED** — the marched NO fires the clamp (`max_a > 1`) and equals `max_a_frozen` to the
   anchored margin (`< 1%`).
8. **ROBUST to the NO LEVEL** — the clock is `[NO]`-independent, so (NO frozen) `max_a ∝ x_no_frozen`
   exactly (2× NO ⇒ 2× clamp).
9. **CYCLE UNTOUCHED** — a call does not perturb station 4 or the cycle `V9`.
10. **GUARDS** — requires the equilibrium gas; rejects `p9 > pt9`; config asserts (`L>0`, `nstep≥100`,
    `rate_scale>0`).

## NOT claimed

- **A freeze LOCATION or a moving freeze point.** `Da_NO ≪ 1` everywhere ⇒ `s_freeze_NO ≡ 0`; rung
  26's headline has no analogue and is not manufactured. (Because the margin is 3–9 orders, no O(1) `L`
  touches the answer — *stronger* than rung 26's location disclaimer, not weaker.)
- **A crossing within any physical `Tt4`.** The separation narrows steeply but `Da_NO` stays `< 1` on
  the whole swept band; extrapolating a crossing would be an overclaim.
- **The single-representative-reaction network.** Only `2r/3r` are summed (the dominant super-eq
  destruction channels); the full Zeldovich network is not.

## The honest character of this rung — a CONFIRMATION that closes a seam

Rungs 24/26 delivered an inversion and a negative; rung 27 delivers a **confirmation**: the assumption
was right all along. That is quieter, but it is not free — it **retires the last premise the rung-14/17
clamp corollary stood on** (that the exhaust NO it reads is genuinely frozen), and it does so with a
kill test that inverts rung 26's, tying the two nozzle-chemistry rungs into one picture (recombination
freezes *despite* temperature; NO freezes *because* of it). The clock's coupling to `[O],[H]` — the
radicals rung 26 destroys — is the load-bearing subtlety and the seam to the coupled refinement below.

## Deferred (kept additive)

- **The rung-26-coupled march** — NO riding the *relaxing* pool, so `[O],[H]` deplete under it. This
  can only *slow* NO further (radical-poorer ⇒ larger `τ_NO`), moving the answer **deeper into frozen**;
  it is a secondary refinement, not the load-bearing claim (which stands on the radical-rich upper
  bound). Needs rung 26's marched composition threaded into the NO clock.
- **A resolved `τ_res`** (inherited from rung 26) — retire `L`; here the margin is so large it is moot.
- **Detailed Fenimore / prompt-NO freeze** — a different (formation) channel; out of scope.

## Done when

`Gas.no_freeze_out_nozzle` returns the marched exhaust-NO clamp (`Da_NO` entry/exit, `frozen_from_entry`
True everywhere, `max_a` vs `max_a_frozen`, `relaxed_fraction ≈ 0`) beside rung 14's `nozzle_flow`;
the assumption is **derived on an upper bound**, the kill test **inverts rung 26's**, the separation
**narrows with `Tt4`** with no crossing; `main.py` prints the rung-27 panel; `tests/test_rung27.py` is
green on gates 1–10; the whole prior suite is untouched.
