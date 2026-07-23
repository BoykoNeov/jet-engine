# Rung 45 — The transient two-spool surge line on the FUEL path: the plant's rho signal never reaches the surge object

**Scope.** `phi_excursion_fuel` / `transient_surge_margin_fuel` on `TwoSpoolFuelTransient` —
rung 44's transient-surge diagnostic, now marched on rung 43's fuel-controlled plant (`Tt4` an
OUTPUT that overshoots). Rung 44 measured the surge excursion with `Tt4` **commanded** (a clean
ramp, no overshoot); its own concession named this extension and predicted the excursion sign and
schedule-slaving would survive onto the fuel path. Rung 45 measures what actually happens.

> **The rung-43 TIT overshoot is strongly `rho`-MONOTONE (~12% over a 25× `rho` range), yet it
> does NOT reach the reference-free surge object: the raw transient min `phi` is `rho`-INVARIANT
> (<2%, an order weaker than the TIT channel). The `rho` signal is real in the PLANT but never
> reaches the SURGE MARGIN — it surfaces only in reference-dependent currencies (an
> output-`Tt4`-referenced excursion swings ~40% over `rho`, a moving-reference artifact). So rung
> 44's "`rho` powerless over surge" SURVIVES the control swap on the reference-free object. Fuel
> control ENLARGES the surge approach (rung 35 on two shafts) and COMPRESSES the LP-eats-more
> DOMINANCE in the excursion currency (the strong LP asymmetry moving to the raw margin), but
> neither the enlargement nor the schedule-slaving is the finding — rung 44 forecast both. The
> finding is that the plant's `rho`-monotone overshoot does not propagate to the surge object: a
> rung-43 currency-circularity echo on the surge axis.**

Like rungs 7–30, 36, 41 and 44 this is a **pure diagnostic beside the cycle**: the rung-45
methods only *read* rung 43's `integrate_fuel` trajectory and rung 41's imposed `phi_surge`; they
add **no state** and feed nothing back. Arming a surge line leaves rung 43's
`integrate_fuel`/`equilibrium_fuel` **bit-for-bit** unchanged, and the default
`build_turbojet(…).run(…)` design path is **bit-for-bit rung 6**.

---

## Sign-space only — inherited from rungs 41/44, doubled

`phi_surge` stays **imposed** (rung 41) and the complex mode's magnitude stays **disclaimed**
(rung 40). So — exactly as in rung 44 — rung 45 makes **no surge-survival claim**. It delivers
**signs**: which way the transient moves the point, which spool eats it, whether the plant's
`rho` signal reaches the surge object, and whether the fuel control enlarges the approach. The
**rung-36 discipline is enforced exactly: report the crossing, gate the flip.**

---

## The measured objects

Both methods march a **fuel ramp** whose steady endpoints are the fuel-equivalents of
`Tt4_lo → Tt4_hi` (`fuel_for_Tt4`, the rung-43 pin that makes fuel and `Tt4` control land on the
SAME steady endpoints — apples-to-apples), from the running-line start at `Tt4_lo`, via
`integrate_fuel`. `Tt4` FLOATS and OVERSHOOTS.

### The reference is the COMMANDED schedule, NOT the output `Tt4`

`phi_excursion_fuel` returns the signed extremum of `phi(s) − phi_steady(Tt4_cmd(s))` per spool,
where `Tt4_cmd(s)` is the **linear** `Tt4` ramp the fuel command corresponds to — **not** the
overshooting output. This is rung 44's discipline read literally ("strip the steady *schedule's*
drift"): the overshoot is the transient, not part of the schedule. Two consequences:

- **It reduces to rung 44 exactly on the `Tt4` path** (there command ≡ output, no overshoot).
- **Referencing to the OUTPUT instead folds rung 43's `rho`-monotone overshoot into the
  baseline** — a moving-reference currency trap (the surge-axis echo of rung 43's
  currency-circularity). An output-referenced excursion reads a ~40% `rho`-swing that is entirely
  in the moving reference, not the operating point; it is **NOT** used.

`transient_surge_margin_fuel` puts the surge claim on the **reference-free** object: the RAW
transient min `phi` (what actually crosses `phi_surge`), compared against the commanded steady
min — under the rung-36 discipline (`margin_min_*` may go negative; `crossed_*` flags it).

---

## THE FINDINGS

### (1) THE HEADLINE — the plant's `rho` signal does not reach the surge object

Over `rho` ∈ [0.2, 5.0] (a 25× range), accel `Tt4` 1000 → 1400, `r = 0.5`, flow/press:

| `rho` | raw `min_phi_lp` | `Tt4_peak` |
|---|---|---|
| 0.2 | 0.7393 | 1584.7 |
| 0.5 | 0.7366 | 1640.1 |
| 1.0 | 0.7355 | 1695.4 |
| 2.0 | 0.7349 | 1741.4 |
| 5.0 | 0.7346 | 1778.2 |

`Tt4_peak` swings **~12%** (the rung-43 overshoot, strongly `rho`-monotone — a heavier LP spool
lags harder, spiking `f`). The raw surge object `min_phi_lp` moves **0.63%** — inside rung 44's
own <2% invariance bar, an **order weaker** than the TIT channel and in the **same direction**
(consistent mechanism, not decoupling). The plant is `rho`-loud; the surge margin is `rho`-quiet.
Contrast: the **output**-`Tt4`-referenced excursion swings ~40% over the same `rho` — a
moving-reference artifact, the reason the reference-free object carries the claim.

**The cross-rung correction.** Rung 44's "`rho` powerless / schedule-slaved" was measured with
`Tt4` **commanded** (no overshoot). On the fuel path `rho` re-enters the **plant** loudly through
the overshoot (rung 43), and a naively-referenced excursion would report `rho` re-opening on the
surge axis. It does not: the reference-free surge object stays `rho`-invariant. Rung 45 is to the
fuel path what rung 43 was to the overshoot — **the currency you pick decides whether the
inter-spool clock appears to matter, and only the reference-free one is honest.**

### (2) FUEL ENLARGES the surge approach — rung 35 on two shafts (a confirmed prediction)

Raw `min_phi_lp`, matched endpoints and ramp rate, fuel vs `Tt4` control (rung 44):

| `r` | fuel `min_phi_lp` | `Tt4`-control `min_phi_lp` |
|---|---|---|
| 1.0 | 0.7512 | 0.7611 |
| 0.5 | 0.7355 | 0.7515 |
| 0.3 | 0.7189 | 0.7408 |

Fuel dips **deeper toward surge at every matched `r`** — the `Tt4` overshoot amplifies the surge
approach. This is rung 35's "the two accel limits are coupled" (the TIT excursion enlarges the
surge excursion), now on two shafts. Rung 44's concession **explicitly predicted** it, so it is
the **confirmation leg**, not the headline.

### (3) The split SURVIVES, the DOMINANCE COMPRESSES

`phi_excursion_fuel`, accel 1000 → 1400, `r = 0.5`, `rho = 1.0` (commanded reference):

| shape | `ext_lp` | `ext_hp` | ratio | decel `ext_lp` | decel `ext_hp` |
|---|---|---|---|---|---|
| flow/press | −0.1805 | −0.1225 | 1.47 | +0.1814 | +0.1171 |
| press/flow | −0.1749 | −0.1412 | 1.24 | +0.1734 | +0.1327 |
| tilted | −0.1806 | −0.1331 | 1.36 | +0.1796 | +0.1251 |
| hp-only | −0.2112 | −0.1212 | 1.74 | +0.2127 | +0.1161 |

Both spools swing toward surge on the accel (`ext < 0`), the decel is the exact mirror
(`ext > 0`), and the LP LEADS at every shape (`|ext_lp| > |ext_hp|`, incl. the mode-free
`hp-only`). But the **dominance COMPRESSES**: the ratio falls to **1.24–1.74** vs rung 44's
**1.6–2.2** — the `Tt4` overshoot loads the HP transient lag, closing the gap. So `phi_excursion_fuel`
gates only the **ordering**; the **strong** LP asymmetry moves to the raw margin (finding 5), where
the LP crosses while the HP clears wide.

### (4) Ramp-rate GOVERNS — the surviving governing variable (a confirmed prediction)

Raw `min_phi_lp` (reference-free), flow/press, `rho = 1.0`, accel 1000 → 1400:
`r` = 1.0 → 0.5 → 0.3 → 0.1 gives `min_phi_lp` = 0.7512 → 0.7355 → 0.7189 → deeper still —
monotone deeper as the ramp gets faster. The schedule against the shaft clock is the governing
variable, surviving the control swap. Rung 44 predicted this too.

### (5) Report the crossing, gate the flip — on the ACCEL

`transient_surge_margin_fuel`, flow/press, accel 1000 → 1400, `r = 0.3`, floor `phi_surge_lp = 0.746`
(in the gap between the transient min ~0.719 and the steady min ~0.773), `phi_surge_hp = 0.55`:

```
margin_min_lp = -0.0271   (raw transient min phi_lp - phi_surge)   crossed_lp = True
steady_min_lp = +0.0271   (commanded steady min - phi_surge)        -> steady CLEARS
margin_min_hp = +0.2836                                             crossed_hp = False
```

The LP transient dips **below** a floor every steady point clears — the flip — and it lands on
the **LP** spool (the HP never approaches: the strong LP-eats-more, the compressed excursion ratio
notwithstanding). The crossing DEPTH is disclaimed (imposed floor, ramp rate); the gated object is
`margin_min_lp < steady_min_lp` on the accel.

**Only the accel is gated.** A decel moves the point **away** from surge, so the raw min `phi`
relaxes onto the low-power steady point and the raw margin is **degenerate** there
(`tr ≈ st ≈ 0`). The decel MIRROR lives on the referenced excursion (finding 3, decel `ext > 0`),
exactly where rung 44's decel gate sat.

---

## Reduce-to-prior contract (the spine)

- **Read-only ⇒ rung 43 bit-for-bit.** The rung-45 methods only *read* `integrate_fuel`; they add
  no state and never write `phi_surge`. Arming a surge line leaves `integrate_fuel` and
  `equilibrium_fuel` **`==`** identical, and the referenced excursion is identical armed vs bare.
  The rung 38–44 suites pass **unchanged**.
- **The split methods are inherently two-shaft.** `lp_disabled` is **not** a reduce axis for a
  split *between* spools; both methods **assert** on the degenerate engine (rung 44's contract, one
  rung on; rung 40's `lp_disabled` dispatch is untouched and still reduces to rung 35 on its own
  gate).
- **Reduces to rung 44 on the `Tt4` path by construction** — the commanded reference makes
  `phi_excursion_fuel` the fuel-control image of `phi_excursion` (command ≡ output when `Tt4` is
  commanded). Not a runtime `==` (the plants differ — one overshoots), a construction identity.
- **Cycle untouched ⇒ rung 6 bit-for-bit.** The default design run is unchanged; rung 45 is
  read-only.

### Why rung 45 carries no independent bare-math gate (rung 43's precedent)

Rung 45 **reads** rung 43's `integrate_fuel` trajectory. Control-invariance (rung 43 gate 1) lands
every steady fuel point exactly on rung 40's steady manifold, which rung 40's own independent
bare-math cascade ties down — so the fuel-surge object is anchored **transitively**. The genuinely
new content is the **transient signs** (the currency trap, the enlargement, the compression, the
flip) — shape-robust directions, not magnitudes a bare-math replica would constrain (rung 42/43
set this precedent).

---

## Verification gates (`tests/test_rung45.py`)

1. **REDUCE.** Read-only ⇒ rung 43 `integrate_fuel`/`equilibrium_fuel` **bit-for-bit** (`==`);
   the referenced excursion identical armed vs bare; `lp_disabled` **asserts** on both methods
   (inherently two-shaft); default design run **bit-for-bit rung 6**.
2. **THE SPLIT SURVIVES, DOMINANCE COMPRESSES.** Accel `ext_lp < 0` and `ext_hp < 0`, decel both
   `> 0` (mirror), `|ext_lp| > |ext_hp|` every shape incl. `hp-only`; and at **every shape the
   fuel-path excursion ratio is below rung 44's `Tt4`-path ratio on the same maps** (a shape-matched
   RELATIVE comparison — 1.47<1.90, 1.24<1.61, 1.36<1.76, 1.74<2.23 — not a bare-magnitude
   threshold). Sign + ordering; magnitudes disclaimed.
3. **THE HEADLINE — the currency trap.** (a) **Same-currency trap** (all in `phi`): over `rho` ∈
   [0.2, 5.0] the REFERENCE-FREE surge object (raw `min_phi_lp`) is `rho`-invariant (**< 2%**),
   while the OUTPUT-`Tt4`-referenced excursion swings **> 20%** — a moving-reference artifact. The
   more the reference tracks the overshoot, the more `rho` leaks in: reference-free quietest
   (0.63%) < the shipped COMMANDED-ramp excursion intermediate (~8%, **NOT** claimed `rho`-flat —
   only its sign/ordering are gated) < output loudest (~33%). So the output reference reads a
   `rho`-dependence that is **not in the operating point** — the reason the surge claim rides on
   the reference-free margin. (b) **Plant-loud / object-quiet:** raw `min_phi_lp` spread **< 2%**
   WHILE `Tt4_peak` spread **> 5%** (and `> 5×` the `min_phi` spread) — the plant's `rho` signal
   does not reach the surge object.
4. **FUEL ENLARGES.** Raw `min_phi_lp` (fuel) `<` `Tt4`-control (rung 44) at matched `r` ∈
   {1.0, 0.5, 0.3}. Sign only.
5. **RAMP-RATE GOVERNS.** Raw `min_phi_lp` monotone-deeper as `r` falls over {1.0, 0.5, 0.3, 0.1}.
6. **REPORT THE CROSSING, GATE THE FLIP.** `transient_surge_margin_fuel`'s `margin_min_lp <
   steady_min_lp` on the accel; a floor in the gap ⇒ `crossed_lp = True`, `crossed_hp = False`
   while `steady_min_lp > 0`; unarmed maps ⇒ the method asserts. (Decel not gated on the raw
   object — degenerate; its mirror is gate 2's referenced excursion.)

---

## Concessions

- **`phi_surge` imposed (doubled, tripled across rungs 41/44/45) — no survival claim.** Every
  margin magnitude and every absolute crossing is disclaimed; only the currency trap, the
  enlargement sign, the split/compression signs, the ramp-rate monotonicity and the flip's sign
  are load-bearing. Rung 45 does **not** claim the engine surges or survives on the fuel path.
- **`rho` is a DISCLAIMED clock group, doubled** — inherited from rung 40. The finding is that the
  plant's `rho` signal (the overshoot) does not reach the reference-free surge object; the weak
  0.63% residual is same-signed and an order under the TIT channel — **weakly coupled, not
  decoupled**. No effective clock is claimed.
- **Reference choice is load-bearing and disclosed.** The commanded (moving-ramp) reference is
  used for the excursion — it reduces to rung 44 on the `Tt4` path; the output reference is named
  and rejected (it re-introduces rung 43's overshoot as a moving-reference artifact). The shipped
  commanded-ramp excursion is **not itself `rho`-flat** (it carries ~8% over the 25× `rho` range —
  intermediate between the reference-free 0.63% and the output ~33%); only its **sign and
  ordering** are gated, not its magnitude. The `rho`-invariance claim is put on the reference-free
  raw margin alone, so the headline does not ride on the reference debate.
- **Every magnitude rides on the representative maps, the fuel step, the `Tt4` band and the imposed
  floor** — the excursion depths, the ratio's value, the enlargement gap, the crossing depth.
  Rung-32 methodology: shapes disclosed, claims shape-robust, magnitudes disclaimed.
- **Reacting-gas fuel control deferred** (rung 35/43's concession, carried verbatim):
  `_tt4_from_f` asserts against an equilibrium gas, so the fuel-surge object runs on the
  non-equilibrium gases; the finding is gas-independent and the reacting reduce is the `Tt4`-control
  path (rung 44, bit-for-bit rung 6 at the cycle).
- **Fully-choked branch, both NGVs choked, no bypass/bleed, one `eta_m`, isentropic knobs** —
  inherited from rungs 38–44 unchanged. The subsonic/unchoked LP branch's transient remains open.
- **Diagnostic beside the cycle.** The fuel-path transient surge line reads the running point; it
  never feeds back.

---

## Anchor

`docs/plans/rung45-anchor-fuel-surge.md`. The **method** is again Cohen–Rogers–Saravanamuttoo
*Gas Turbine Theory* Ch. 9 (the acceleration working-line swing toward surge), now with the fuel
step outrunning the shaft response through a **floating** `Tt4` — so the TIT overshoot (rung 35/43)
and the surge approach (rung 44) are the **same** transient viewed in two currencies. Rung 45's own
contributions are the **currency trap on the surge axis** (the plant's `rho`-monotone overshoot
does not reach the `rho`-invariant reference-free surge object), the **quantified enlargement** vs
`Tt4` control, the **dominance compression** of the LP-eats-more excursion ratio, and the
**report-crossing/gate-flip on the accel** fuel-path surge object.
