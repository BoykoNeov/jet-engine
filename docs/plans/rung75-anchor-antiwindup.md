# Rung 75 anchor — THE DECLARED ANTI-WINDUP DEVICE (rung 74 § 10's own seam)

Scored in `docs/rung75-spec.md` § 9. The rule this project runs under: **a prediction that is
edited after the measurement is not a prediction.**

**AND THIS ANCHOR'S § 0 IS BIGGER THAN RUNG 74's, WHICH IS ITSELF THE FIRST DISCLOSURE.** Rung
74 disclosed two pre-anchor measurements; this one discloses a whole probe, because the seam it
attacks is stated by rung 74 as *the cell does not exist* and there is no honest way to write
predictions about a plant without first establishing that there is one. Everything in § 0 was
measured before this document existed (`M:\claud_projects\temp\rung75\probe0_cell_exists.py`,
output saved beside it) and **none of it is scored**. What § 2B scores is everything the probe
does not settle — and the probe settles nothing about a Jacobian, because at the time it ran
the only instrument that could read one was **blind** (§ 0.6).

§ 2 is split as rungs 72/73/74's were. **§ 2A is DERIVED** — worked out on paper from the
inherited laws — and is listed as derivation, **not scored as prediction**, except where § 9
finds a derivation measured false (rung 72's D5 precedent).

---

## 0. WHAT WAS MEASURED BEFORE THIS DOCUMENT EXISTED

### 0.1 The seam, and the device the accident was standing in for

Rung 74 § 10: *AN ANTI-WINDUP DEVICE, EXPLICITLY. § 4 says the clip coordinate has one by
accident. A declared tracking/reset law on the demand plant would make `demand × applied`
exist, and the comparison would isolate what the accident was buying.*

Rung 74 § 4 established the accident. In clip coordinates the masked applied-referenced leg
runs **into** rung 52's floor at `g = 0` and halts; in demand coordinates the identical motion
is `dw/ds = (cap − mf_app)/τ > 0` with **nothing in its path**, the joint IC sweep diverges
(residual `2.898e−3` after 60 iterations) and the march never starts.

The textbook device is **back-calculation**: pull the state toward the fuel actually applied,
on its own clock.

    dw/ds = ( target − w ) / τ  +  ( mf_app − w ) / τ_t              [`track`]

`τ_t` is **a new constant** — the first new clock since rung 65 — and it has no derivation from
anything shipped. Every finding here is therefore a property of the **sweep** (the two limits
and the trend between them), never of a chosen value: the treatment `φ_lim` has had since
rungs 36/49, and rung 54's *every verdict is a threshold ON it*.

### 0.2 THE CELL EXISTS — the feasibility check, and it picks the rung

At `φ_lim = 0.76` (rung 74's own both-legs-ride arm), `ds = 0.005`, all four inherited clocks
at `0.05`:

| `τ_t` | `τ_t/τ_f` | `applied` | `sched` |
|---|---|---|---|
| 0.4 | 8 | **no** (res 2.472e−6) | yes |
| 0.2 | 4 | **no** (res 4.442e−9) | yes |
| 0.1 | 2 | **yes**, 54 iters | yes |
| 0.05 | 1 | **yes**, 32 iters | yes |
| 0.025 | 0.5 | **yes**, 20 iters | yes |
| 0.0125 | 0.25 | **yes**, 14 iters | yes |
| 0.00625 | 0.125 | **yes**, 10 iters | yes |

**The cell rung 74 reports as having no plant is reached.** So this is a rung, and it is the
one this anchor is written for.

### 0.3 THE "EXISTENCE" BOUNDARY IS THE SOLVER, AND THE LAW IS EXACT — this corrects rung 74

**"The device makes the cell exist" is the wrong sentence and it is not the one this rung will
ship.** The park law (§ 2A.1) gives a **finite** equilibrium at every finite `τ_t`. What the
device buys is a **contraction**: the IC sweep's map has slope

    σ = τ_t / ( τ + τ_t )   < 1 for every finite τ_t ,   → 1 as τ_t → ∞

so the inherited sweep (60 iterations, tolerance `1e-12`, assert at `1e-9`) converges in
`ceil( ln(tol/res₀) / ln σ )` iterations. With **rung 74's own reported residual** as `res₀`
and no fitted constant at all:

| `τ_t` | `σ` | predicted iters | measured |
|---|---|---|---|
| 0.1 | 0.66667 | **54** | **54** |
| 0.05 | 0.50000 | **32** | **32** |
| 0.025 | 0.33333 | **20** | **20** |
| 0.0125 | 0.20000 | **14** | **14** |
| 0.00625 | 0.11111 | **10** | **10** |

**Five of five, exactly.** And it runs backwards too: `τ_t = ∞ ⇒ σ = 1 ⇒` no contraction at
all, the residual sits where it started — **rung 74's `2.898e−3` is this same sweep at `σ = 1`,
which is why that number never moved.** Rung 74's verdict stands untouched (`w* → ∞`, there is
no finite equilibrium); what is corrected is the reading of its residual: it was not a solver
failing to find a plant, it was a **contraction with ratio exactly one**.

The two `no` rows are therefore the **60-iteration cap** cutting a geometric sequence, not a
property of the plant — a solver boundary, and § 2B.7 turns it into a scored prediction.

### 0.4 THE RK4 FLOOR IS ARITHMETIC, AND IT GRID-LIMITS THE FAST END

The device adds `1/τ_t` to each of the two fuel-side diagonals, so `_rk4_floor_shared`'s
inherited `ds·Σ(1/τ_i) ≤ 2` admits

    τ_t  ≥  2·ds / ( 2 − ds·Σ(1/τ_i) )  =  0.00625     at ds = 0.005, four clocks of 0.05

Measured: `τ_t = 0.005` asserts by name (`ds·Σ = 2.400`). **Perfect tracking (`τ_t → 0`) is not
reachable on this grid and is not claimed** — the reachable range is `τ_t/τ_f ∈ [0.125, ∞)`.
The constant is not loosened to reach further (rung 65's lesson, and rung 74's `_rk4_floor_shared`
already carries its own re-justification).

### 0.5 WHAT ELSE THE PROBE ALREADY SHOWED — disclosed, not scored

* the bill's **direction** under `applied`: `max Tt4` 1197.4–1197.5 K for `τ_t ≤ 0.05`, against
  rung 74's `demand-latched × applied` at **1359.88 K** — the declared device beats the
  accident by ~160 K;
* a **redline crossing between `τ_t = 0.05` and `τ_t = 0.1`**: 1197.50 K against **1210.49 K**,
  i.e. a slow tracking clock breaches `Tt4_max = 1200`;
* under `sched` the device is nearly **inert on the bill** (1197.68 → 1197.41 across the whole
  sweep), while under `applied` it is decisive;
* the masked leg's park point scales roughly **linearly in `τ_t`** (`w/mf_sched` = 1.026, 1.062,
  1.122, 1.249, 1.468 at `τ_t/τ_f` = 0.125 … 2), which is § 2A.1's law to first order.

These are **not** scored. § 2B scores the pointwise park law, the whole Jacobian, the control
row, the hand-over and the reduce.

### 0.6 THE INSTRUMENT WAS BLIND, AND IT WAS FOUND BEFORE ANY JACOBIAN WAS READ

Every inherited gains reader differences the **target** and hands the result to `_jac4`, which
assembles `J[i][j] = (∂cmd_i/∂x_j − δ_ij)/τ_i`. **The tracking term is in neither**: it is not
part of any leg's target, and `τ_t` is not in `taus`. So `demand_gains` run on the `track` cell
would have reported the masked diagonal unchanged, `det J` still dead and the spectrum
invariant — **a perfect refutation of this rung's own headline, having measured nothing.** That
is rung 73's `_reference` no-op with the sign flipped, and it would have passed every gate one
would think to write.

`_rhs_laws` / `_rhs_gains_at` difference the **derivative** instead, so all sixteen entries
including every diagonal are measured end to end. **`τ_t` is deliberately NOT added to `taus`**:
letting `_jac4` write `−1/τ_t` onto the diagonal would be the seventh instance of the
shipped-instrument-agrees-with-itself pattern (rung 67 gate 9, rung 71 § 1.4, rung 72 §§ 4 and
8, rung 73's `_reference`, rung 74 § 1.1). Rung 73 *weakened* `_jac4` to measure two diagonals;
this finishes that move for all four. **No Jacobian had been read when this was written.**

---

## 1. THE PLANT, AND THE FOURTH DECLARED KNOB

`_windup_law` joins `_share_law` (72), `_ref_law` (73) and `_lag_coord` (74):

| `_windup_law` | the masked leg's stop | is |
|---|---|---|
| `none` | none at all | **RUNG 74**, by the branch not being taken |
| `track` | back-calculation onto `mf_app`, clock `τ_t` | **THIS RUNG** |

**REFUSED, both by assert and by name:**

* **`clip × track`** — rung 52's `max(0,·)` is still in that coordinate, so the cell would run
  **two** anti-windup devices at once and attribute the result to this one. Rung 63's *change
  one law at a time*, which rung 74 § 2 records itself breaking in a `for` loop.
* **`demand-latched × track`** — the latch is the accidental device; same refusal, one device on.
* raising the 60-iteration IC cap **in the plant** (§ 2B.7 lifts it in a *reader*, to measure a
  derived number, and the plant keeps the inherited cap).
* adding `τ_t` to `taus` (§ 0.6).

**THE ACCIDENT TRACKED A DIFFERENT SIGNAL, AND THAT IS THE COMPARISON THIS RUNG EXISTS FOR.**
The inherited stop clamps the state at the **SCHEDULE** (`w ≤ mf_sched`); the declared device
pulls it toward the **APPLIED FUEL** (`mf_app ≤ mf_sched`). They coincide exactly while no leg
is cutting and diverge only where one is — which is the only regime the limiter family lives in.

---

## 2A. DERIVED — worked out on paper. NOT SCORED.

* **D1 — THE PARK LAW.** Setting `dw/ds = 0` for a **masked** leg under the **applied**
  reference, where `target = w + cap − mf_app` so the leg's own `w` cancels from the first term:

      w* = mf_app + ( τ_t / τ ) · ( cap − mf_app )

  an offset **above** the applied fuel, linear in the clock ratio and in the leg's own slack.
  `τ_t → 0` parks it at `mf_app` exactly (textbook perfect tracking, a leg with nothing to
  unwind); `τ_t → ∞` recovers rung 74's divergence. Under **`sched`** the first term keeps its
  own `w` and the park law is the weighted mean `w* = (τ_t·cap + τ·mf_app)/(τ + τ_t)` —
  **two different park laws, which is what makes the windup × reference 2×2 non-degenerate.**
* **D2 — THE MASKED DIAGONAL.** `∂RHS_masked/∂w_masked` is `−1/τ_t` under `applied` (rung 73
  cancelled the `1/τ` term to **zero**, which is why its pole sat at the origin) and
  `−(1/τ + 1/τ_t)` under `sched` (where it was already `−1/τ`). **So the pole leaves the origin
  under `applied` and merely moves under `sched`: the `det J` revival is `applied`-only, and it
  is the same mechanism as § 0.5's split on the bill, not a second finding.**
* **D3 — THE DEVICE IS THE ZERO FUNCTION ON THE LEG THAT HOLDS.** `mf_app = min(mf_sched,wf,wr)`
  equals `w_auth` **in a neighbourhood**, so `(mf_app − w_auth)/τ_t` is identically zero — not
  small, zero — and so is its derivative. The authoritative diagonal is untouched and rung 72's
  *ONE plant IS rungs 68/69/70/71 by AUTHORITY* survives.
* **D4 — THE MASKED COLUMN IS UNTOUCHED, so `n_live ≤ 3` stands a FOURTH time.** The tracking
  term sits in the masked leg's **ROW** (it reads the authoritative leg's state through
  `mf_app`); nothing reads the masked leg, because `min()` is still flat in it. `M` stays
  block-triangular.
* **D5 — THE MASKED ROW'S COUPLING TO THE AUTHORITATIVE LEG.** `∂RHS_masked/∂w_auth` is
  `1/τ_t − 1/τ` under `applied` — so it **changes sign at `τ_t = τ`** and vanishes exactly
  there — and `+1/τ_t` under `sched`, where rung 74 had exactly `0`.
* **D6 — THE CONTRACTION.** § 0.3, and it is derivation *stated after* the measurement, which
  is why it is here and not in § 2B.

## 2B. PREDICTED — scored in § 9. No Jacobian has been read.

* **P1** Measured by RHS differencing on the `(demand, applied, track)` plant, the **masked
  diagonal is `−1/τ_t`** to ≤ 1e−6 relative, at **two** clocks (`τ_t = 0.05` and `0.0125`),
  and their **ratio is 4.000**.
* **P2** The **authoritative diagonal is unmoved** between `none` and `track` at the same state
  (≤ 1e−9 relative), and **`track_leak = 0.0` exactly** — the device is the zero function
  there, at the base point and at both perturbations of that leg's own state (D3).
* **P3** **`mask_leak = 0.0` exactly** under `track`, as it is under `none` — the masked column
  is untouched, `M` is block-triangular and **`n_live` is still ≤ 3** (D4). *The seam closes by
  refutation for the FOURTH running.*
* **P4** **`det J` REVIVES under `applied`**: `≈ 0` (below the `1e−4·rate` zero threshold) under
  `none`, non-zero under `track`, and **`|det J|` scales exactly as `1/τ_t`** — the ratio
  between `τ_t = 0.0125` and `0.05` is **4.000** to ≤ 1e−6 relative, because the live 3×3 block
  is rung 71's and unmoved. **`zeros` drops by exactly `n_masked = 1`.**
* **P5** Under **`sched`**, `det J` is **non-zero in BOTH** cells and `zeros` does **not** drop
  — the revival is `applied`-only (D2), one mechanism with two faces.
* **P6** The masked row's coupling `J[masked][auth]` **changes sign between `τ_t = 0.025` and
  `τ_t = 0.1`** under `applied` and is **zero to ≤ 1e−6 relative at `τ_t = τ_masked`** (D5);
  under `sched` it is `+1/τ_t` where rung 74 measured exactly `0`.
* **P7** With the IC cap lifted **in a reader only**, `τ_t = 0.2` and `τ_t = 0.4` converge at
  **exactly 98 and 185 iterations** — `ceil(ln(1e−12/2.898e−3)/ln σ)`, zero fitted constants
  (§ 0.3). This converts § 0.2's two `no` rows from an artifact into an explained one.
* **P8** **THE CONTROL ROW: dormant ⇒ `track` ≡ `latch`.** Where no leg is cutting,
  `mf_app = mf_sched` and the declared device pulls to exactly where the accident clamps, so
  the two agree to ≤ 1e−9 on every dormant point and diverge only where a leg is cutting.
* **P9** **THE HAND-OVER IS MONOTONE IN `τ_t`, AND ITS DIRECTION DISCRIMINATES THE MECHANISM.**
  Two are live and both are named here in advance: *(a) windup-dominant* — a less wound-up leg
  has less to unwind, so the fast-tracking end hands over **earliest** and the time is monotone
  **increasing** in `τ_t`; *(b) `Tt4`-path-dominant* — tracking holds `Tt4` down and a held-down
  `Tt4` **delays** the governor, which is exactly what refuted rung 74's P5, so the time would
  be flat or inverted. **Registered prediction: (a)**, on § 0.5's redline crossing being
  consistent with it. If (b), rung 74 § 2.1's mechanism generalises and this rung says so.
* **P10** **A THRESHOLD ON THE NEW CONSTANT.** There is a critical `τ_t*` strictly between 0.05
  and 0.1 at which `max Tt4` crosses `Tt4_max`, and it lies **within a factor of 2 of `τ_f`** —
  rung 54's shape (*every verdict is a threshold on the disclosed constant*), on the one
  constant this rung adds.
* **P11** **THE REDUCE:** `_windup_law = 'none'` reproduces rung 74 **bit-for-bit** on all
  fourteen of its gates (the hook's branch is not taken — not a tolerance).
* **P12** **AND IT IS NOT VACUOUS** (rung 73's `charpoly_selftest` discipline, rung 74's
  `test_the_clip_reduce_is_not_vacuous`): the same machine under `track` must **differ**.

**REFUSED IN ADVANCE:** deriving `τ_t` from anything shipped and presenting it as derived;
quoting any finding at a single `τ_t` without its sweep; loosening `_rk4_floor_shared` to reach
`τ_t < 0.00625`; running `clip × track` or `demand-latched × track`; and reporting the pole,
`det J` or `zeros` through any reader that differences a **target**.
