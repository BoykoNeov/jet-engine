# Rung 48 anchor — the Wf/pt3 acceleration schedule

**Method anchor.** Cohen–Rogers–Saravanamuttoo, *Gas Turbine Theory*, Ch. 9 — the acceleration
fuel schedule drawn as a **`Wf/pt3` vs speed** law sitting BETWEEN the steady running line and the
surge line, min-selected with the maximum-turbine-temperature limiter of rungs 46/47. `Wf/pt3` is
the classic choice because it is the fuel/air-loading proxy the control can measure directly at
the compressor delivery, and because it is **feedforward on the cause** (fuel in, delivery
pressure) rather than feedback on a consequence (turbine temperature) — which is exactly the
property rung 48 turns into a measurement.

Here: `Wf ≤ (1 + m)·κ_ss(n_H)·pt3` with `pt3 = pt4/π_b = π_HPC·π_LPC·pt2` (already carried by
`_close_fuel` — **zero new plant**), `n_H` the corrected HP speed the HP map already runs on, and
`κ_ss(n_H) = (Wf/pt3)` **read off the plant's own steady equilibria** over the accel band. The
schedule SHAPE is therefore derived; the entire imposition is the single scalar `m`. The cap is
implicit in `Wf` (both `pt3` and `n_H` move with the fuel through the closure), so it is a
bracketed Illinois set-point solve — the same structure as rung 46's `_topping_fuel`.

**Config for every table below** (`tests/test_rung48.py` reproduces them):
`FLIGHT = FlightCondition(T0=250, p0=50_000, M0=0.85)`, design `π_LPC=3, π_HPC=6, Tt4=1500`,
`REAL` losses (as rungs 45–47), `nozzle_convergent=True`, **CPG** gas
(`γc=1.4, cp_c=1004, γt=1.3, cp_t=1239, hPR=42.8e6`; the finding is gas-independent — rungs
35/43/45/46/47's carried concession). Shapes (rung 47's set): `flow/press`, `tilted`, `hp-only`
(LP flat). Accel 1000→1400 K, `ρ=1`, `s_settle=4.0`, `ds=0.02`.

`relief_* = min_phi_*(limited) − min_phi_*(bare)`, `> 0` safer (rung 45's reference-free surge
object). `s_eng` = the first `s` at which the leg clips. `fuel_rm = ∫(schedule − applied) ds`.

---

## Table A — THE WINDOW (the enabling measurement): the bare ratio vs the steady line, r = 0.5

`(Wf/pt3)/κ_ss` on the BARE march (`m = 0` cap ≡ κ_ss·pt3, so this ratio IS the margin needed to
engage at that instant):

| `s` | 0.00 | 0.06 | 0.10 | 0.18 | **0.24** | 0.32 | 0.40 | 0.46 |
|---|---|---|---|---|---|---|---|---|
| ratio | 1.0000 | 1.1335 | 1.2105 | 1.3340 | **1.4004** | 1.4574 | 1.4841 | 1.4885 |
| `φ_LP` | 0.7731 | 0.7558 | 0.7473 | 0.7373 | **0.7355 ← min** | 0.7399 | 0.7510 | 0.7629 |
| `φ_HP` | 0.9433 | 0.9179 | 0.9038 | 0.8822 | 0.8714 | 0.8633 | **0.8612 ← min** | 0.8629 |
| `Tt4` | 1000 | 1111 | 1183 | 1319 | 1411 | 1518 | 1608 | 1663 |

The ratio starts at exactly 1 (the march starts on the running line), rises **monotonically**
through BOTH minima, and peaks at 1.4885. So `m` maps continuously to an engagement start time
`s_eng(m)` sweeping the whole ramp. **This is what makes the rung possible.** Note `s_lp* = 0.240`
is at 48 % of the ramp and `s_hp* = 0.400` at 80 % — the two minima are well separated at this `r`.

## Table B — THE HEADLINE: the per-spool engagement crossing (flow/press, r = 0.5)

bare `min φ_LP = 0.73547` @ `s_lp* = 0.240`, `min φ_HP = 0.86120` @ `s_hp* = 0.400`,
`ν_H` end = 0.95906, `Tt4` peak 1695.4 K.

| `m` | `s_eng` | vs `s_lp*` | relief_lp | relief_hp | fuel_rm | `Tt4` pk | `ν_H` end |
|---|---|---|---|---|---|---|---|
| 0.05 | 0.040 | upstream | +0.031124 | +0.072520 | 0.04445 | 1243.7 | **0.87246** ← degenerate |
| 0.10 | 0.060 | upstream | +0.024898 | +0.063047 | 0.02264 | 1460.1 | 0.95714 |
| 0.15 | 0.080 | upstream | +0.018965 | +0.053719 | 0.01202 | 1489.5 | 0.95894 |
| 0.20 | 0.100 | upstream | +0.013285 | +0.044526 | 0.00696 | 1518.8 | 0.95903 |
| 0.25 | 0.140 | upstream | +0.008498 | +0.035737 | 0.00411 | 1546.1 | 0.95905 |
| 0.30 | 0.160 | upstream | +0.004415 | +0.026839 | 0.00237 | 1577.6 | 0.95906 |
| 0.35 | 0.200 | upstream | +0.001818 | +0.018433 | 0.00127 | 1607.1 | 0.95906 |
| 0.36 | 0.220 | upstream | +0.000691 | +0.016875 | 0.00110 | 1612.8 | 0.95906 |
| 0.38 | 0.220 | upstream | +0.000179 | +0.013399 | 0.00081 | 1629.4 | 0.95906 |
| 0.40 | 0.240 | **ON the min** | +0.000053 | +0.010228 | 0.00057 | 1642.1 | 0.95906 |
| 0.42 | 0.280 | downstream | **0.000000** | +0.007493 | 0.00037 | 1651.0 | 0.95906 |
| 0.44 | 0.300 | downstream | **0.000000** | +0.004594 | 0.00022 | 1662.2 | 0.95906 |
| 0.45 | 0.320 | downstream | **0.000000** | +0.003385 | 0.00015 | 1672.9 | 0.95906 |
| 0.46 | 0.340 | downstream | **0.000000** | +0.002051 | 0.00010 | 1672.9 | 0.95906 |
| 0.48 | 0.400 | downstream (**= `s_hp*`**) | **0.000000** | **+0.000016** | 0.00002 | 1686.7 | 0.95906 |
| 0.50 | — dormant | — | 0.000000 | 0.000000 | 0.00000 | 1695.4 | 0.95906 |

Two crossings, ONE instrument: `relief_lp` → exactly 0 as `s_eng` passes `s_lp* = 0.24`;
`relief_hp` still +0.0075 there and dying only as `s_eng` reaches `s_hp* = 0.40`.

**The non-tautology (why this is not rung 44 restated):** `fuel_rm` falls smoothly and stays
POSITIVE across both crossings (0.00081 → 0.00057 → 0.00037 → 0.00022 → 0.00010 → 0.00002); the
settled `ν_H` is bare's 0.95906 for every `m ≥ 0.25` (and within 0.012 % at `m = 0.15`); and at
`m = 0.45` ONE clip removing ONE quantity of fuel rebates the HP (+0.0034) and gives the LP
**exactly nothing**. A ramp-rate effect cannot split two spools from the same removed fuel.

**The honest boundary:** at `m = 0.05` the leg binds from `s = 0.04` and never releases — `ν_H`
end 0.87246 vs 0.95906 (the accel has NOT completed) and `Tt4` peak collapses to 1243.7 K. There
the leg HAS become rung 44's ramp-rate lever, and its apparent relief is largely the accel being
de-fanged. The finding is stated only in the endpoint-preserving window `m ∈ [0.10, 0.45]`.

## Table C — THE DEGENERATE CASE that could have broken it: fast ramp, r = 0.15

There `s_lp* = s_hp* = 0.140` (both minima coincide; the LP min sits at 93 % of the ramp). The
timing rule then predicts ONE crossing, not a split:

| `m` | `s_eng` | relief_lp | relief_hp | fuel_rm |
|---|---|---|---|---|
| 0.20 | 0.040 | +0.057795 | +0.099017 | 0.00893 |
| 0.40 | 0.060 | +0.033225 | +0.062796 | 0.00194 |
| 0.60 | 0.100 | +0.013490 | +0.028705 | 0.00033 |
| 0.70 | 0.120 | +0.004928 | +0.012305 | 0.00007 |
| 0.75 | 0.140 | +0.003519 | +0.004621 | 0.00002 |
| 0.78 | 0.160 | **0.000000** | **0.000000** | 0.00000 |

Both reliefs die TOGETHER. A "the LP spool is special" reading would have predicted the split
persists; the timing reading predicts exactly this. The crossing must therefore be stated **per
`r`** — `s_lp*` moves with the ramp rate (0.240 at `r = 0.5`, 0.140 at `r = 0.15`) — never as a
universal `s` threshold.

## Table C2 — the HP crossing, demonstrated on a SLOW ramp (r = 2.0)

At `r = 0.5` the ratio peak (1.4885) runs out of dial just as `s_eng` reaches `s_hp* = 0.40`, so
the HP side shows a collapse (+0.000016) rather than a clean exact zero — weaker evidence than the
LP side. A slower ramp separates the minima further and leaves dial to spare:

| `r` | `s_lp*` | `s_hp*` | `m` | `s_eng` | past `s_hp*` | relief_lp | relief_hp | fuel_rm |
|---|---|---|---|---|---|---|---|---|
| 2.0 | 0.32 | 0.64 | 0.20 | 0.700 | **yes** | 0.000000 | **0.000000** | 0.00002 |
| 1.0 | 0.28 | 0.50 | 0.30 | 0.440 | no | 0.000000 | +0.001217 | 0.00020 |

The `r = 2.0` row is the HP-side analogue of the LP result: engagement strictly PAST the HP
minimum, fuel still being removed, relief EXACTLY zero — and the march is bit-identical to bare
through BOTH minima. Both crossings are therefore demonstrated at the mechanism level, not just
the consequence.

**Mechanism check (r = 0.5, gate 8b):** the first divergence between the limited and bare marches
lands exactly at `s_eng` — 0.280 at `m = 0.42`, 0.320 at `m = 0.45`, 0.400 at `m = 0.48` — all
downstream of `s_lp* = 0.240`. Nothing upstream of the LP minimum moves, which is why the
switch-off is exact and not merely small.

## Table D — SHAPE ROBUSTNESS (r = 0.5)

The crossing rule (`relief_lp > 0` iff `s_eng < s_lp*`, exactly 0 otherwise) holds on `tilted`
and on **`hp-only`** (LP map FLAT ⇒ no rung-40 complex inter-spool mode), so the rule does not
ride on that mode — gate 13. Magnitudes vary with shape and are disclaimed.

---

## What is NOT claimed

- No claim about the margin a real accel schedule carries, nor about any crossing's DEPTH
  (`m` imposed, `φ_surge` imposed — rung 41's discipline, maps representative — rung 32's).
- No claim that this leg protects the redline: it is a compressor-protection leg (`Tt4` peak is
  still 1642 K at `m = 0.40`). The min-select composite with rungs 46/47's leg is what a real
  accel schedule actually is.
- No claim in the `m → 0` corner, which is reported as the degeneracy boundary and gated
  separately so it cannot be quietly folded into the finding.
