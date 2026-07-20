# The β margin, hardened — rung 28's own weak point, re-checked on the axis it named

**Status: a CONFIRMATION of rung 28, NOT a new rung.** No new physics, no new constant, no code-path
change. It re-measures a margin rung 28 shipped as disclosed-but-uncomfortable, and finds it
structurally safer than the raw number suggested. Rung 28's *numbers are unaffected*; only its
statement of its own weak point is sharpened.

## What rung 28 left open

Rung 28's β repair (`docs/rung28-spec.md` § Correction 2) replaced rung 27's false
"NO arrives super-equilibrium" premise with the condition that actually carries the bound:

```
  τ_exact / τ_surrogate = (1+u)² / [ (1+u)² − (1−β²) ]  >  1   for all a ≥ 0,  whenever β < 1
  β = R1/(R2+R3),   u = βa
```

`β<1` ⇒ rung 27's `a≫1` surrogate is a uniform **lower bound on τ** (upper bound on the rate) in
**both** regimes — formation and destruction — which is what the entry-false premise never supplied.

Rung 28 disclosed the margin as its least comfortable: max β over the nozzle path runs
**0.103 → 0.261 → 0.512 → 0.513** for `Tt4 = 1500 → 1800 → 2200 → 2400 K`. It **rises with `Tt4`** and
reaches **half** the β=1 threshold — *a factor 2, not orders*. The seam it filed:

> **β at higher `π_c` / hotter cycles** — the one margin here that is a factor rather than orders.

## The result: β is pressure-inert, and π_c pushes it DOWN

**β has no direct pressure channel at all.** Every term is a product of *two* concentrations —
`R1 = k1f[O][N2]`, `R2 = k2r[NO]_e[O]`, `R3 = k3r[NO]_e[H]` — so `c_tot²` cancels exactly, top and
bottom:

```
  β = k1f·x_O·x_N2 / ( x_NOe·(k2r·x_O + k3r·x_H) )
```

Mole fractions and T-only rate constants. Nothing else. Measured flat to **8 significant digits**
(`0.51234156`) across a **160× pressure span** at fixed composition and T — this is an identity, not a
numerical near-miss.

So π_c can only reach β **indirectly**, and both of its indirect channels were measured separately by
counterfactual (pin one, vary the other) at `Tt4 = 2200 K`:

| π_c | `far` | `Tt9` [K] | β (composition channel alone) | β (`Tt9` channel alone) | β (**actual, in-cycle**) |
|---|---|---|---|---|---|
| 10 | 0.05426 | 1991.8 | 0.5123 | 0.5123 | **0.5123** |
| 20 | 0.05022 | 1893.9 | 0.5095 | 0.4605 | **0.4561** |
| 40 | 0.04546 | 1772.3 | 0.4951 | 0.3954 | **0.3755** |
| 80 | 0.03984 | 1620.8 | 0.4728 | 0.3146 | **0.2776** |

Raising π_c raises `Tt3`, which (a) cuts `far` — the **composition** channel, weak — and (b) cuts `Tt9`
by extracting more turbine work — the **temperature** channel, dominant. **Both push β down.** Net
in-cycle β falls monotonically, `0.512 → 0.278` over π_c 10→80.

**Higher π_c is protective, not threatening — the opposite of what the seam feared.** And it hardens
rung 27 on the same axis: entry `Da_NO` falls with π_c too (`2.0e-3 → 1.8e-4` at `Tt4=2200`), so the
freeze-from-entry verdict gets *deeper*, not thinner, exactly where β was suspected of weakening.

## β does not saturate — but its crossing is a temperature, and it is off the map

The shipped `0.512 → 0.513` at 2200→2400 K reads like a plateau. **It is not.** On a fixed mixture β
climbs monotonically and without limit in T:

| T [K] | 1200 | 1600 | 2000 | 2400 | 2800 | **3200** | 3600 |
|---|---|---|---|---|---|---|---|
| β | 0.114 | 0.304 | 0.517 | 0.716 | 0.891 | **1.039** | 1.163 |

(frozen station-4 mixture, `far=0.054`.) The apparent flatness is a coincidence of the *runnable
boundary curving*, not saturation — so the comforting reading is the wrong one, and this doc does not
take it.

What replaces it is better. Because β is pressure-inert and composition is second-order, **the β=1
crossing is characterised by a single temperature** — and that temperature is far above anything
reachable:

| π_c (at `Tt4=2400`) | actual nozzle-entry `Tt9` | β=1 at | headroom |
|---|---|---|---|
| 10 | 2200.1 K | 4069.2 K | **1869 K (1.85×)** |
| 40 | 1987.6 K | 3198.5 K | **1211 K (1.61×)** |
| 80 | 1841.6 K | 3185.0 K | **1343 K (1.73×)** |

The cycle stops long before: `Tt4 ≥ 2450–2500 K` has no solution at all (the rung-6 burner-balance
assert fires). **A 0.5-vs-1.0 margin in β is a 1.6–1.9× margin in the variable that actually drives
it** — which is the honest way to state it.

## The whole-plane maximum: 0.5444, and it is INTERIOR

β is non-monotone in *both* axes (the runnable boundary curves), so the hot corner is not the max.
Scanned over the runnable `(Tt4, π_c)` plane — `-` marks cells with no cycle solution:

| `Tt4` \ π_c | 8 | 10 | 12 | 15 | 20 | 25 | 30 | 40 |
|---|---|---|---|---|---|---|---|---|
| 2200 | 0.5243 | 0.5123 | 0.5003 | 0.4828 | 0.4561 | 0.4325 | 0.4115 | 0.3755 |
| 2300 | **0.5444** | 0.5429 | 0.5382 | 0.5283 | 0.5095 | 0.4905 | 0.4724 | 0.4396 |
| 2400 | – | 0.5131 | 0.5311 | 0.5405 | 0.5394 | 0.5307 | 0.5194 | 0.4946 |
| 2450 | – | – | – | 0.5106 | 0.5354 | 0.5376 | 0.5327 | 0.5154 |
| 2480 | – | – | – | – | 0.5172 | 0.5334 | 0.5350 | 0.5244 |

**That max landed on the grid EDGE** (`π_c = 8`, the lowest scanned) with the row still climbing —
which would mean the edge had been found, not the maximum. β rises as π_c *falls*, and lower π_c means
lower `Tt3` hence *more* burner headroom, so π_c = 2…7 are runnable and had to be checked:

| `Tt4` \ π_c | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 10 |
|---|---|---|---|---|---|---|---|---|
| 2100 | 0.5288 | 0.5233 | 0.5144 | 0.5046 | 0.4948 | 0.4853 | 0.4761 | 0.4589 |
| 2200 | 0.5236 | 0.5376 | **0.5405** | 0.5388 | 0.5349 | 0.5299 | 0.5243 | 0.5123 |
| 2300 | – | 0.4691 | 0.5116 | 0.5295 | 0.5384 | 0.5428 | **0.5444** | 0.5429 |

**β TURNS OVER below π_c ≈ 8 — it does not keep climbing.** The mechanism is the same two channels,
now competing with opposite signs: as π_c falls, `far` **rises** (0.0576 → 0.0674 over π_c 13→3 at
`Tt4=2300`), which pushes β **down**, while `Tt9` **rises** (2062 → 2221 K), which pushes β **up**. At
high π_c the `Tt9` channel dominates; at low π_c the composition channel takes over. The crossing is
what makes the maximum interior.

Fine scan confirms it — `Tt4` ∈ [2250, 2375] × π_c ∈ [6, 13]:

| `Tt4` \ π_c | 6 | 7 | 8 | 9 | 10 | 11 | 13 |
|---|---|---|---|---|---|---|---|
| 2250 | 0.5434 | 0.5420 | 0.5393 | 0.5357 | 0.5316 | 0.5272 | 0.5178 |
| 2275 | 0.5431 | 0.5442 | 0.5433 | 0.5413 | 0.5384 | 0.5351 | 0.5274 |
| 2300 | 0.5384 | 0.5428 | **0.5444** | 0.5442 | 0.5429 | 0.5408 | 0.5351 |
| 2325 | 0.5274 | 0.5364 | 0.5413 | 0.5437 | **0.5444** | 0.5439 | 0.5407 |
| 2350 | 0.5042 | 0.5222 | 0.5323 | 0.5383 | 0.5417 | 0.5434 | 0.5434 |
| 2375 | – | 0.4899 | 0.5126 | 0.5253 | 0.5330 | 0.5379 | 0.5423 |

**Plane max β = 0.5444 — an INTERIOR maximum** (all four neighbours lower), sitting on a **flat
diagonal ridge** that plateaus at ≈0.544 and is reached at several `(Tt4, π_c)` pairs — (2300, 8),
(2325, 10), and within 0.0005 at (2275, 7) and (2325, 11). Nothing on the plane exceeds it.

This is slightly **above** the 0.513 rung 28 shipped as its worst case, so the correction is not purely
favourable: **0.544 is the number to quote** — but it is now an established maximum rather than a
sampled one.

## The honest statement, revised

Rung 28 said: *β is ~0.51, rising with `Tt4`, a factor not orders, and higher π_c / hotter cycles is
where it wants re-checking.*

It now reads: **β ≤ 0.545 over the entire runnable `(Tt4, π_c)` plane (an established interior max, not
a sampled corner); it is exactly pressure-inert by
construction; and the one knob the seam suspected (π_c) pushes it DOWN through both of its channels.
The β=1 crossing is a temperature ~1.6–1.9× above the hottest reachable nozzle entry, and the cycle
ceases to solve long before it.**

**What is still not claimed.** β<1 remains **empirical, not a theorem** — the identity above shows β is
a ratio of mole-fraction-weighted rate constants with no bound forcing it below 1, and it demonstrably
exceeds 1 above ~3200 K. A cycle that delivered a genuinely hotter *nozzle entry* (not a hotter `Tt4`
at fixed component efficiencies — that route is closed by the burner balance) would still need this
re-checked. What is now excluded is the specific worry rung 28 filed: **pressure/π_c is not that route.**

## Gates

In `tests/test_rung28.py`:

- `test_beta_is_exactly_pressure_invariant` — the `c_tot²` cancellation, to 12 digits over 160× in p.
- `test_beta_falls_with_pressure_ratio` — the protective direction, in-cycle.
- `test_beta_plane_maximum_is_interior` — the plane bound `< 0.6`, the max sitting **on the ridge**,
  and interiority: the ridge strictly beats **both** flanks (low-π_c *and* high-π_c/hot), so neither
  the bound nor its interiority can drift into a scan-edge artifact.
- `test_beta_margin_is_disclosed_not_comfortable` — the original band walk, extended to forbid the
  **false comfort** that β plateaus (it climbs monotonically in T and crosses 1 off-cycle).
