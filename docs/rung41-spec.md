# Rung 41 — The two-spool surge line: the exposure SPLITS between the spools

Rung 36 drew a surge line on **one** compressor and found the margin **thin at low power**.
Rungs 39 and 40 both closed by naming the same open seam in almost the same words: *"a
two-spool surge line — rung 36's machinery is single-spool, and now there are **two**
compressors."* Rung 41 draws it on both.

> **The two-spool running line does not halve the low-power surge problem — it CONCENTRATES
> it on the LP compressor.** Over a 2:1 throttle the LP flow coefficient falls **~29 %** while
> the HP's falls **~7 %** and is *bounded* (it turns back up): the HP face is **shielded** by rung 39's `(†)`
> cancellation — it sees only its **own** pressure ratio, while the LP face carries the
> **product** `π_LPC·π_HPC`. That asymmetry is not a sign but a **closed form**: each face's
> flow-coefficient sensitivity is an explicit function of the pressure ratios it sees, and the
> HP's contains **no LP quantity at all**. Its corollary is a **zero-new-constant** critical
> pressure ratio — the anchor rung 36's **dead** one never got:
>
> **(★)  `1 + η_c(τ_c − 1) = γ_c`  ⟺  `π_c* = γ_c^(γ_c/(γ_c−1))` = 3.2467 at `γ_c = 1.4`**
>
> — the point where a choked-NGV compressor's running line is **stationary in flow
> coefficient**. `η_c`, the shaft constant, `cp_t/cp_c`, `τ_HPT` and the design
> pressure-ratio split **all drop out**; only `γ_c` survives.

Like rungs 7–30 and 36 this is a **pure diagnostic beside the cycle**: `phi_surge` is read
**only** by the rung-41 surge methods, so a surge-line-carrying map leaves rung 39's `match`
and rung 40's transient **bit-for-bit** unchanged, and the default `build_turbojet(…).run(…)`
design path is untouched (bit-for-bit rung 6).

---

## The machinery: rung 36's construction, doubled

At a two-spool running-line point each compressor has its own `(n, φ)` map coordinate, so
rung 36's constant-speed margin applies to each:

```
SM_L = π_LPC(n_L, φ_surge,L)/π_LPC,op − 1        SM_H = π_HPC(n_H, φ_surge,H)/π_HPC,op − 1
```

`_pi_c_spool` is rung 36's `_pi_c_map` parameterized by spool — the **same** forward
speed-line + efficiency-island arithmetic `_hp_eta_loop`/`_lp_eta_loop` use — so at the
operating `(n, φ)` it reproduces the shipped `π` on each spool (gate 2: two code paths, one
`π`, per spool). The cost is rung 36's cost **doubled**: **two** imposed stall flow
coefficients. Every margin **magnitude** is therefore disclaimed, exactly as in rung 36.

---

## The split, and its structural cause

Rung 39's `(†)` is what does it. Referred to the HP compressor face,
`pt4/pt25 = π_b·π_HPC` — **`π_LPC` cancels** — so with the HPT NGV choked the HP's corrected
flow, its speed line and hence its flow coefficient close on the single **internal** ratio
`x_H = Tt4/Tt25`:

```
τ_HPC − 1 = K·x_H            (HP shaft balance + the geometric τ_HPT)
n_H²·ψ(φ_H) = x_H/x_H,d      (the speed line)
m_H  ∝  π_HPC/√x_H           (the choke; MFP* is Tt-independent on CPG)
```

Three equations, one parameter — and **no LP quantity anywhere**. The LP face instead
carries the **product** (rung 39's `(‡)`: `m_L ∝ π_LPC·π_HPC·MFP*·√(Tt2/Tt4)/(1+f)`) and
rides `x_L = Tt4/Tt2`.

Writing `φ ∝ Π_face/x_face` with `π = [1+η_c(τ_c−1)]^k`, `k = γ_c/(γ_c−1)`, the
log-sensitivities are **closed form**:

```
s_H = dlnφ_H/dlnx_H = k(1 − π_HPC^(−1/k)) − 1                                  ← π_HPC ALONE
s_L = dlnφ_L/dlnx_L = k(1 − π_LPC^(−1/k)) + k(1 − π_HPC^(−1/k))/τ_LPC − 1      ← the PRODUCT
```

Measured against the shipped matcher (CPG, flat maps, gate 4): both hold to **< 0.05**
absolute across design splits, flight conditions and `γ_c`. **Dropping the `π_HPC` term from
`s_L` fails by 0.81–1.00** — 60–100× worse, and with the **wrong sign**. So the
asymmetry is certified quantitatively, not merely as an observed ordering: *the HP's
sensitivity needs no LP pressure ratio; the LP's cannot be written without the HP's.*

Because `π_LPC·π_HPC ≫ π_HPC`, `s_L ≫ s_H` at every throttle point — the LP compressor takes
essentially the whole excursion. Measured over `Tt4` 1500 → 800 on the shaped maps: `φ_L` falls
**~29 %**, `φ_H` **~7 %**, a **3.8×–4.3×** ratio across four shape pairs (on CPG + flat maps the
LP figure is ~34 %). **Gate 3** asserts the sign: `φ_L` falls more than **3×** as far as `φ_H`,
and `φ_L < φ_H` at every part-power point, across four shape pairs.

### A framing PROBED AND WITHDRAWN

The obvious headline — *"the HP running line collapses onto one flight-independent curve, the
LP's does not"* — is **wrong, and was written and then removed**. On the choked branch
`τ_LPC − 1 = K_L·x_L` and `x_H = x_L/τ_LPC`, so `x_L` and `x_H` are in **bijection**: the
whole matched state is a **one-parameter family** and *both* running lines collapse, on
*either* ratio. (The flight condition enters only through `Tt2`; `p0` is pure scale — gate 4b,
the rung-33 pressure-homogeneity result on two spools.) The collapse is real but **shared**,
so it separates nothing. What separates the spools is *which pressure ratios enter the
sensitivity* — hence the gate above.

---

## The closed form (★), and what it is NOT

Setting `s_H = 0` in the sensitivity above: `k(1 − π^(−1/k)) = 1` ⇒ `π^(1/k) = k/(k−1) = γ_c`,
i.e.

```
(★)   1 + η_c(τ_c − 1) = γ_c        ⟺        π_c* = γ_c^(γ_c/(γ_c−1))
```

**`γ_c` alone.** The measured turn moves only **1.5 %** in pressure ratio (3.286 → 3.337) across
every case below — and all of that is the fuel fraction (§ kill test). Verified (gate 5) invariant to `η_HPC` (0.80 → 0.95), `η_HPT`, `η_LPC`,
`γ_t`, `cp_t`, the design split (3×6, 4.5×4, 2.25×8) and the flight condition — while the
turn's location **in `Tt4` moves 666 → 1171 K (1.76×)** across those same cases. *The closest
approach is at a pressure ratio, not at a throttle setting.* Sweeping `γ_c` over 1.30–1.45,
the measured turn tracks the closed form with a **constant** residual.

That residual is **+0.44 %** in the `(★)` form at realistic `hPR`, and it is entirely the
**fuel fraction**: `f` enters both `K` (through `(1+f)`) and the choked corrected flow
(through `1/(1+f)`), so `(★)` is exact only with `f` frozen. **Kill test** (gate 5b): raise
`hPR` by ×1000 so `f` → 1e-5, and the residual falls **monotonically to zero**, linearly in
`f` (0.443 % → 0.143 % → 0.042 % → 0.004 % → 0.000 %). Nothing else is hiding in it.

**Three regimes.** A face whose design pressure ratio is **above** `π*` walks *toward* surge
in flow coefficient as it throttles, bottoms at `π*`, and walks back out; one **below** `π*`
walks *away* from surge from the design point on (verified at a 6×3 split, where `φ_H` rises
monotonically 1.000 → 1.157). Whether the turn is *reachable* depends on the choked envelope:
at a 1.5×12 split, or at low `M0`, the nozzle unchokes before `π_HPC` falls to `π*` and
`flow_coefficient_turn` returns `kind="RAIL"` rather than inventing an interior minimum.

**What (★) is NOT.** It is the stationary point of the running-line **flow coefficient** — an
**incidence / running-line-geometry** fact. It is **not** a minimum of the surge margin. The
worst pressure-ratio margin is still at idle, on **both** spools. That divergence is the
payoff, not a caveat — see below. And the turnaround *phenomenon* (φ rising again at very low
power) rides on the analytic speed-line + choke pin: a real front-stage-stalling map may not
reproduce it. The **location** `π*` is what is gated, and only within the model's flat-map,
CPG idealization; on shaped maps it shifts ~3 % and on a variable-`cp` gas ~2.5 % — disclaimed,
rung-32 methodology.

---

## The margin ordering: the LP is the exposed spool

With the **same map shape** on both spools and a **common** imposed floor, `SM_L < SM_H` at
every point and the ratio
`SM_L/SM_H` **falls monotonically** with throttle (gate 6, three shapes × three floors). Both
schedules inherit rung 36's sign (thin at low power); the LP's simply collapses several times
faster, and at a floor of 0.70 the LP running line **crosses** a surge line the HP never
approaches.

The measure is deliberately the **ratio**, not the absolute gap: both margins tend to zero at
deep throttle, so the gap must eventually shrink too (it peaks near `Tt4 ≈ 1300` on these
maps). **A partial artifact, named — do not over-attribute.** Matching the map *shape* does **not**
make this a clean controlled comparison: the two compressors still carry different **design
pressure ratios** (`π_LPC` = 3 vs `π_HPC` = 6), and a smaller design pressure ratio gives a
smaller pressure-ratio margin at the same flow-coefficient gap. So `SM_L < SM_H` **already holds
at the design point** — `tilted`, `Tt4` = 1500: 0.3165 vs 0.5186, where `φ_L = φ_H = 1` and there
is no exposure difference at all. That level offset is `π_LPC` alone, **not** the running line.

**The load-bearing content is therefore the RATIO `SM_L/SM_H` COLLAPSING with throttle** (0.61 →
0.15 at `φ_surge` = 0.55) — that *is* the running-line divergence, and it is what gate 6 asserts.
The flow-coefficient headline (`φ_L` −29 % vs `φ_H` −7 %) is unaffected by this caveat: both are
normalized to 1 at design, so that excursion is a pure running-line statement.

**Not claimed:** which spool binds at *unmatched* floors or *unmatched* shapes — that
is floor- and shape-dependent (with `press/flow` at design, `SM_L > SM_H`). The rung-36
discipline, enforced: sign under a disclosed comparison, never a magnitude, never a crossing.

**Also not claimed: "the slip protects the LP spool."** That is the textbook twin-spool
rationale and it is a **counterfactual this model does not run** (it would need the same LP
compressor rigidly tied to the HP shaft). What rung 41 shows is the complementary truth —
the LP is the **exposed** spool, which is where booster bleed valves live. Rung 39's
`slip = N_L/N_H` is a **speed ratio**, not a surge-proximity measure; the surge-proximity
measure is `φ_L`, and the two are related through the matching, not the same object.

---

## The cross-rung CORRECTION of rung 36 (the rung-28 shape)

`(★)` is **surfaced by** the two-spool work, not **created by** it: it holds for a single
spool too. In fact — measured, not assumed — for rung 36's own `π_c = 10` engine the turn sits
at `Tt4 ≈ 620`, **inside its choked envelope**. Rung 36 simply never looked below the bottom
of its plotted range. So:

- **Rung 36's gated verdict SURVIVES.** `SM_N` is still monotone-thin at low power, past the
  turn, on all three of its surge-realistic shapes (gate 7b). No rung-36 test changes.
- **Its stated MECHANISM is corrected.** Rung 36 wrote: *"the trend is set by `φ_op(Tt4)` —
  the running-line flow coefficient."* But `φ_op` is **not monotone**; below `π*` it turns
  around and walks *away* from the floor, while the margin keeps thinning. Freezing one
  coordinate at a time (`surge_margin_channels`) separates two comparable channels:

  | channel | frozen | decay over `Tt4` 1500→650 (`surge_flow`) | share of `ln` decay |
  |---|---|---|---|
  | **φ-walk** (rung 36's stated cause) | `n` at design | 0.4838 → 0.2060 | ~53 % |
  | **speed-line flattening** (`τ_c−1 ∝ n²`) | `φ` at design | 0.4838 → 0.2318 | ~47 % |
  | full `SM_N` | — | 0.4838 → 0.1045 | 100 % |

  The two are **comparable**, and below `π*` the φ channel **reverses** while the speed-line
  channel keeps consuming margin — so at deep throttle the flattening speed line is the *only*
  channel still thinning the margin.
- **Rung 36's conclusion is untouched.** Both channels are choked-hardware-determined and
  hence **floor-independent**, so its sign-robustness argument survives intact. Only the
  single-channel attribution was wrong.

### The reconciliation (why (★) matters at all)

Rung 36's currency equivalence `E0 ≥ SM_N ⟺ φ_step ≤ φ_surge` is a **constant-speed**
statement — one speed line, so pressure-ratio margin and flow-coefficient proximity are the
same statement. Along a **varying-speed running line** they are **different schedules**, and
the HP spool is the clean exhibit where they visibly **diverge**: `φ_H` turns up past `π*`
while `SM_H` keeps falling (gate 7a, asserted as a deliberate divergence so the tempting
"(★) is the worst-margin point" claim cannot creep in).

---

## Reduce-to-prior contract (the spine)

- **Surge line on ⇒ rung 39/40 bit-for-bit.** `phi_surge` (the rung-36 field, reused — no new
  knob) is read **only** by the rung-41 surge methods. A `phi_surge`-carrying map leaves
  `TwoSpoolMapMatcher.match` and `TwoSpoolTransient.equilibrium` **`==`** identical (gates 1a,
  1b); `is_flat()` still ignores `phi_surge`. The rung 31–40 suites pass **unchanged** (72/72)
  — the standing witness.
- **`π` reproduction, per spool (non-tautological).** `_pi_c_spool` at the operating `(n, φ)`
  equals the shipped `π_LPC`/`π_HPC` to ≤1e-9 — each margin is measured on the very forward
  map that sets that spool's running line.
- **Cycle untouched ⇒ rung 6 bit-for-bit.** The default design run is unchanged; rung 41 is
  read-only.

---

## Verification gates (`tests/test_rung41.py`)

1. **REDUCE — pure diagnostic.** A `phi_surge`-carrying map ⇒ rung 39 `match` and rung 40
   `equilibrium` **bit-for-bit** (`==`), four shape pairs; `is_flat` ignores the floor;
   default design run bit-for-bit rung 6.
2. **`π` REPRODUCTION (non-tautological), BOTH spools** — `_pi_c_spool(n_op, φ_op)` == the
   shipped `π` on each spool, ≤1e-9, four shapes × five throttles.
3. **THE SPLIT** — `φ_L` falls >3× as far as `φ_H`; `φ_L < φ_H` at every part-power point;
   the HP's own ratio spans a narrower range. Shape-robust; magnitudes disclaimed.
4. **THE SHIELDING, quantitative (the two-spool non-tautological gate)** — the closed-form
   `s_H` (containing **no LP quantity**) and `s_L` (containing the **product**) each match the
   measured sensitivity to <0.05, while **dropping `π_HPC` from `s_L`** misses by >0.5 — a
   >10× separation, wrong sign. Plus 4b: the flight condition enters only through `Tt2`
   (`p0` pure scale, ≤1e-12) — the *withdrawn* collapse framing recorded as its true, weaker
   statement.
5. **THE CLOSED FORM (★)** — `1 + η_c(τ_c−1) = γ_c` within 1 % across `η_HPC`/`η_HPT`/`η_LPC`/
   `γ_t`/`cp_t`/three design splits/two flight conditions, while `Tt4*` moves >1.4×; tracks
   `γ_c` over 1.30–1.45. **5b, the KILL TEST:** raising `hPR` drives `f` → 0 and the residual
   falls monotonically below 1e-4 — the whole residual is the fuel fraction.
6. **THE MARGIN ORDERING** — matched shapes + a common floor ⇒ `SM_L < SM_H` everywhere and
   `SM_L/SM_H` monotone-falling (halving at least), three shapes × three floors. **The gated
   content is the falling RATIO** (the running-line divergence); the `SM_L < SM_H` *level* is
   partly the `π_LPC` = 3 vs `π_HPC` = 6 design split and is named as such, not attributed to
   exposure. Sign only.
7. **THE DIVERGENCE + THE RUNG-36 CORRECTION** — (a) `φ_H` turns **up** while `SM_H` keeps
   **falling** (asserted as a deliberate divergence); (b) on rung 36's own single spool the
   turn is inside its choked envelope near `π*`, its `SM_N` is still monotone (**verdict
   survives**), and both channels are comparable and neither negligible (**mechanism
   corrected**).
8. **CYCLE UNTOUCHED** — default design run bit-for-bit rung 6.

---

## Concessions

- **Two imposed `φ_surge`** — rung 36's one disclosed constant, doubled. Every margin
  magnitude, and every **crossing** into surge, is disclaimed; only signs under controlled
  comparisons are load-bearing.
- **`(★)` is CPG + flat-map.** It uses `γ_c` as a constant and `ψ ≡ 1`, `η_c` constant. On
  shaped maps it shifts ~3 %, on a variable-`cp` gas ~2.5 %. The *turnaround phenomenon*
  itself rides on the analytic speed line + choke pin and may not survive a real
  front-stage-stalling map.
- **`(★)` is not a margin extremum** and is not offered as one (gate 7a exists to prevent
  that reading).
- **No slip-protection claim** — the rigid-shaft counterfactual is not run.
- **Which spool binds is claimed only at matched shapes + a common floor**; at unmatched
  floors/shapes the ordering can reverse at high power.
- **Steady, choked branch, both NGVs choked, no bypass, one `η_m`** — inherited from rungs
  38/39. The **transient** surge line (measuring rung 40's complex inter-spool mode against a
  stability boundary) and the **subsonic/unchoked LP branch** are deliberately deferred.
- **No bleed valve / variable stator** — rung 41 exhibits the margin these devices protect on
  the spool where they actually live; it does not model them (rung 36's standing concession).
- **Diagnostic beside the cycle.** The surge line reads the running line; it never feeds back.

---

## Anchor

`docs/plans/rung41-anchor-two-spool-surge.md`. The **method** is again Cohen–Rogers–
Saravanamuttoo *Gas Turbine Theory* Ch. 9 (superimpose the equilibrium running line on the
compressor characteristic; the margin is the pressure-ratio gap to the low-flow stability
boundary), now applied to **two** characteristics. CRS's twin-spool statement that *the HP
compressor of a choked-NGV two-spool engine operates over a very small region of its
characteristic* is exactly what the `(†)` shielding produces here, with the sensitivity
formula as its quantitative form. Rung 41's own contributions are the **quantified
asymmetry** (the HP's sensitivity contains no LP pressure ratio; the LP's cannot be written
without the HP's), the **closed form `(★)`** with its fuel-fraction kill test, and the
**cross-rung correction** of rung 36's single-channel mechanism.
