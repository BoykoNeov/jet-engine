# Rung 19 — Super-equilibrium O & prompt NO: lifting the equilibrium-O lower bound

Rungs 7–18 read the rung-6 **equilibrium** `[O]` into the Zeldovich rate and form thermal NO on it.
That single assumption — `[O]=[O]_eq` — makes **every** NO number since rung 7 (and every rung-17
super-equilibrium margin `a`) a **lower bound**. Rung 19 lifts it two ways and shows — the load-bearing
result — that **both lifts contradict the "the rich primary explodes with NO" intuition**, from
opposite directions. It adds **no new species** and touches **no cycle path**: a decoupled diagnostic,
so the cycle stays **bit-for-bit rung 6**.

> **Read `docs/rung7-spec.md` (the Zeldovich integrator + the equilibrium-O concession) and
> `docs/rung9-spec.md` (the rich zoned primary) first**, and `docs/plans/rung19-anchor-
> superequilibrium-prompt.md` (numbers-before-code: the sourced Westenberg/De Soete forms, the
> `m(T)` multiplier, the corrected prompt φ-shape, the T-sensitivity discriminator, the two
> concessions). This file states only what *changes*.

---

## What rung 19 adds (and what it deliberately does not)

**Adds — two lower-bound-lifting channels, both on the existing rung-7 substrate:**

- **A super-equilibrium-O multiplier `m(T)`** on the thermal rate. Fluent's partial-equilibrium O
  (Westenberg, adds the 3-body `O+O+M⇌O₂+M`) sits **above** equilibrium O; the two share `[O₂]^0.5`,
  so their ratio is **dimensionless and T-only**: `m(T)=(C2/C1)·T·exp((θ1−θ2)/T)∈[1.16,1.50]`. We lift
  **our own** rung-6 `comp["O"]` by `m(T)` inside the rung-7 integrator. `m≡1` ⇒ **bit-for-bit rung 7**.
  This is the one thing rung 19 **computes** cleanly (only `C2/C1, θ1−θ2` — no absolute-magnitude risk).
- **An imposed prompt-NO φ-bump** — De Soete's (1975) global rate reduced to its **fitted rich-peaking
  φ-shape**: `EI_prompt(φ,T)=scale·max(f(φ,n),0)·exp(−Ea/RuT)`, `f(φ,n)=4.75+0.0819n−23.2φ+32φ²−12.2φ³`.
  An **additive** trace channel beside thermal. `prompt=None` ⇒ code-path-identical to the prior rung.
- **A summed-trace guard** — the decoupling assertion now spans **all three** contributions
  (`x_NO,thermal·m + x_NO,prompt < 0.02`), like rung 12's guard spanned both streams.

**Deliberately deferred / imposed (seams kept — see the anchor §5):**

- **Prompt magnitude is imposed, not derived** (a 0-D burnt pool has no flame structure) — the
  rung-18-style imposed closure. Only the **φ-shape** and the **directional** prompt/thermal ratio are
  certified, not the g/kg number.
- **Super-eq O magnitude is semi-empirical** (Westenberg's fitted ratio) — a full-equilibrium pool
  **cannot self-yield** super-equilibrium O (the rung-18 impossibility again). Certified: the reduce,
  the units cross-validation, and "T-driven & modest".
- **De Soete φ-validity 0.6–1.6** — φ>1.6 is extrapolation (f<0), clamped and flagged; the deep-rich
  flank up to the rung-9 soot bound (φ=2) is outside the prompt model.
- **Detailed Fenimore (`CH+N₂→HCN→…`) and radical-decay history** — both would need new species / a
  relaxing pocket; kept as explicit next seams.

---

## The load-bearing result: the "rich explosion" intuition fails twice, and prompt survives where thermal dies

The naive story is "NOx blows up in the hot rich primary." Rung 19 shows the two lifts behave oppositely:

1. **Super-equilibrium O is T-driven, not rich-driven.** The lift `m(T)−1` depends on **temperature
   only** (measured identical across φ=0.8→2.0) — and it is *weakest* in the O₂-depleted rich primary,
   where thermal NO is already near zero. At the rung-17 design point φ_p=1.5 super-eq O lifts ~nothing.
2. **Prompt NO is the rich-specific lift — but it *survives*, it doesn't explode.** Its φ-shape peaks
   slightly rich (φ≈1.1 in EI, pulled lean from the f-peak 1.24 by the falling `exp(−Ea/RT)`), and it
   dies on the deep-rich flank (f<0 past φ≈1.65). What it *does* is fill the flank where **thermal
   collapses**: the prompt/thermal ratio grows **monotonically rich, 0.12→168** across φ=0.8→1.5. So
   just rich of stoich, prompt is what keeps NO from going to zero.
3. **Both stay trace** (Σ NO ≤ 0.40% < 2%) ⇒ **cycle bit-for-bit rung 6**.

**The T-sensitivity discriminator:** prompt carries a *single* Arrhenius exp, thermal a *double*
(`k1f·[O]_eq`). Measured 2000→2400 K at stoich: thermal **×566**, prompt **×21** — prompt is ~27×
milder, the quantitative face of "survives where thermal dies," and a clean gate.

And the rung-17 corollary discharged: since every rung-17 margin `a` was built on equilibrium O, rung 19
makes explicit that they are **lower bounds** — the super-eq lift `m(T)` and the prompt channel both
push `a` up (documented, not re-gated: rung 17 stays a pure composition of rung-8/11/16 + rung-14).

---

## The equations — a diagnostic layer, no station changes

Every cycle station is **bit-for-bit rung 6**. Evaluated on the frozen pool `comp` at `(T, p)` and, for
prompt, the local φ:

```
super-eq O:  m(T) = (C2/C1)·T·exp((θ1−θ2)/T)        (C2/C1, θ1−θ2 from Westenberg; m≡1 => rung 7)
             thermal rate uses [O] = m(T)·comp["O"]  (lift OUR equilibrium O; everything else rung 7)
prompt:      EI_prompt(φ,T) = scale · max(f(φ,n),0) · exp(−Ea/RuT)      (φ>1.6 => extrapolation, f→0)
             f(φ,n) = 4.75 + 0.0819n − 23.2φ + 32φ² − 12.2φ³
total:       EI_total = EI_thermal(with m·[O]) + EI_prompt
guard:       x_NO,thermal·m + x_NO,prompt < 0.02      (summed-trace, spans both channels)
```

- **Standing asserts:** (1) the rung-7 `K`-check unchanged; (2) `1 ≤ m(T) ≤ 2` over the flame band
  (the multiplier is a lift, bounded); (3) the **summed** trace guard `< 0.02`; (4) `f(φ)` clamped at 0
  for φ>1.6 (never a negative prompt EI).

## Design (additive, mirrors rungs 7–18)
- `thermal_nox(far, T, p, tau, super_eq_o=False, prompt=None)` and
  `zoned_nox(…, super_eq_o=False, prompt=None)` gain the two knobs. `super_eq_o=False` ⇒ `m≡1`;
  `prompt=None` ⇒ no additive term. Both off ⇒ the exact prior code path.
- A `PromptNO` config (imposed `scale`, carbon number `n`, the De Soete coefficients) rides beside the
  existing mixing configs — a *diagnostic* config, never entering `_equil_solve`.
- `_thermal_no` takes an optional O-multiplier (defaults 1.0 ⇒ byte-identical). `NOxState`/`ZonedNOxState`
  record `m`, `ei_no_super_o`, `ei_no_prompt`, `ei_no_total` (all `None`/1.0 for the off path).

## Verification gates (priority order)
1. **Reduce-to-lower-rung (load-bearing).** `super_eq_o=False` + `prompt=None` ⇒ **bit-for-bit rung
   7/8** (identical `_thermal_no` call, `m=1.0`); every rung 1–18 suite green untouched; cycle
   bit-for-bit rung 6.
2. **super-eq units cross-validation.** Westenberg equilibrium O / `comp["O"]` ∈ [0.94, 0.99] across
   the (φ,T) grid (certifies the O pool *and* the SI units).
3. **super-eq is T-driven not rich.** `m(T)` is **φ-independent** to machine precision; `m∈[1.15,1.55]`
   over 1800–2400 K, decreasing in T, `→1` as T→∞.
4. **prompt f(φ) shape.** Peak at φ≈1.24, negative past φ≈1.65 (clamped); `a(X_O₂)` continuous at the
   three breakpoints (transcription checks).
5. **prompt survives where thermal dies.** prompt/thermal ratio strictly increasing in φ over [0.8,1.5]
   (0.12→168).
6. **T-sensitivity discriminator.** thermal rise-factor / prompt rise-factor 2000→2400 K > 10× (≈27×).
7. **summed trace guard.** Σ x_NO < 0.02 across both channels (measured max 0.0040).

## Conservation asserts (rung-19 deltas)
Carry over rung 7's `K`-check + rung 6's atom balance, plus: `1 ≤ m(T) ≤ 2`; the **summed** trace guard
`x_NO,thermal·m + x_NO,prompt < 0.02`; `f(φ)` clamped ≥ 0.

## Done when
Reduce-to-lower-rung reproduces rungs 1–18 to the digit (existing suites untouched, green) and the
cycle is bit-for-bit rung 6; gates 2–7 hold. `tests/test_rung19.py` covers them. `main.py` gains a
rung-19 panel (the φ-sweep: thermal vs super-eq-lifted vs prompt EI, the monotone prompt/thermal ratio,
the T-sensitivity discriminator, and the two concessions stated loudly). `CLAUDE.md` scope + rung table
+ deferred seams updated (super-eq O / prompt NO done as **lifts of the equilibrium-O lower bound**;
detailed Fenimore + radical-decay history = the next seams). The anchor doc's honest-scope §5 is the
canonical statement of the two concessions (imposed prompt magnitude; semi-empirical super-eq ratio).
```
