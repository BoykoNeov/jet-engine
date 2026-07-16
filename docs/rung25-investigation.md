# Rung 25 investigation — "locally-resolved SCALE" — INVESTIGATED, **NOT SHIPPED**

> **Status.** This is a **negative-result record**, not a rung spec. Rung 25 was built and tested as
> a prototype, returned a negative verdict (below), and was **deliberately not added to the ladder**.
> The shipped ladder ends at **rung 24**. There is no `rung25-spec.md`, no `gas.py`/`main.py`/test
> code, and no entry in the rung table — by design.
>
> **Why this file exists:** so the negative result is not re-investigated from scratch. If you are
> about to "attack the mixing ceiling with a locally-resolved SCALE," read this first — it was done.

## What rung 25 was
The next attack on the standing **mixing-ceiling seam** after rung 24 ("localizes the RATE, not the
SCALE"). Rung 24 resolved each cell's mixing *rate* but kept one global time *scale* `τ_mix`, whose
~20× swing held `⟨EI⟩(J)` monotone. Rung 25 tried to localize the **SCALE**:

- **A penetration-growing plume.** The jet centers penetrate as `δ = k_p·√(SH)·J^p` and the plume
  widths grow with them, `σ_y = k_y·H·f`, `σ_z = k_z·S·f`, `f = (J/J_opt)^p`. This replaces rung 24's
  *fixed* `σ_y=k_y·H`, `σ_z=k_z·S`. Baseline exponent `p = 1/4`.
- **A finite-`τ_res` dwell cap** (reusing rung-16's residence-time knob, **no new time constant**):
  `τ_cell = (1 − e^{−ω·τ_res})/ω`, bounded to `[0, τ_res]`. This is the load-bearing change — it
  **removes rung-24's monotone `τ_mix` backbone**, which is what let `⟨EI⟩(J)` finally turn.
- **A genuinely new O(1) constant `c_D`** in the absolute rate `D_t = c_D·U_j·σ`, `U_j = √J·U_c`.
  The hoped-for "`c_D = C_e·k_y`, no new constant" identity **FAILED** (pinned `c_D ≈ 0.0067` vs
  `C_e·k_y ≈ 0.056`, ~8× off). `c_D` is a real free parameter; every result below was checked at two
  `c_D` pivots (pin `⟨τ⟩=τ_mix` at `J=16` vs `J=36`, a ×4 amplitude spread).

Design point: `H=0.10, S=0.0625, k_p=0.316, k_y=k_z=0.28, C_e=0.20, U_c=75, τ_res=2.5 ms, φ_p=1.5`,
`far=0.0272`, `Tt4=1500 K`. Holdeman group `C=(S/H)√J`; `C_opt=2.5` at `J_opt=16`.

## The verdict — framing (B), a sharp NEGATIVE

**The positive (real, first in the project).** The finite-`τ_res` cap does what rung 24 could not:
`⟨EI⟩(J)` **turns off monotone**. At the baseline `p=1/4` it is a clean interior U, robust across
both `c_D` pivots (min at `J=25`, `C≈3.12`; depth 10–17%). The field even carries a genuine
**over-penetration penalty**: the segregation `g` is **U-shaped in `J`** (the jet centers `δ∝J^p`
overshoot at high `J` and leave the far region under-mixed), and it is the `g`-driven term (term2,
the larger one) that sets the `⟨EI⟩`-min location — not the dwell.

**The negative (why it was not shipped).** The whole result rides on the **unanchored penetration
exponent `p`**. `δ∝J^p` relocates the penetration/`g` optimum in `J` (hence in `C`), so the
`⟨EI⟩`-min location — and even the *existence* of the turn — track `p`:

| σ-law `δ∝J^p` | `⟨EI⟩` min (both `c_D` pivots) | shape |
|---|---|---|
| `J^{1/4}` (hand-picked baseline) | **interior U at `J=25` (`C≈3.12`)** | clean turn, robust to `c_D`×4 |
| `J^{1/3}` | drifts to lean edge `J=9` / near `C_opt` `J=16` | turn nearly gone |
| `J^{1/2}` (≈ standard jet-in-crossflow trajectory) | rich edge `J=36` | **monotone-down — no turn** |

`⟨EI⟩(J)` rows, pivot `J=16` (`c_D=0.0068`):
- `p=1/4`: 2.556 2.467 2.386 2.307 **2.291** 2.351 2.452  (interior min `J=25`)
- `p=1/3`: **2.313** 2.336 2.396 2.453 2.502 2.501 2.487  (min at lean edge — turn gone)
- `p=1/2`: 2.115 2.103 2.054 1.964 1.836 1.494 **0.626**  (monotone ↓ to rich edge)

(J grid `[9,13,16,20,25,30,36]`. The `p=1/3`, pivot-`J=36` cell **railed** during `c_D` pinning
— `c_D≈4.74`, ~100× every other cell — and is unreliable; the verdict does not rest on it.)

So there is **no defensible emissions-optimum location**: the "min offset-rich of `C_opt`" that the
baseline `p=1/4` showed is a **cartoon artifact of the chosen exponent**, not physics. And the more
physically-standard `p≈1/2` gives *no interior turn at all*, which strengthens the negative.

## What this means for the seam (the corrected takeaway)
Localizing the **SCALE** is *still not enough* to pin the emissions optimum — but **not** because a
penetration penalty is missing (it is present: `g` is U-shaped). It is because the penetration
**law** `δ∝J^p` has an **unanchored exponent**, a free cartoon parameter that slides the optimum
anywhere in `J`. The seam's next real requirement is therefore an **anchored `δ(J)`/`σ(J)`
penetration law** — a physically-derived exponent (and, ideally, the full cross-plane pattern with
it) — not merely "a pattern with a penalty."

**Do NOT** re-run: the growing-Gaussian-σ + finite-`τ_res`-cap construction at a hand-picked `p`.
That is exactly this investigation, and its answer is above. **A new attempt is only worthwhile** if
it brings an *anchored* penetration exponent (or a real transported/CFD cross-plane field), which is
the genuinely new physics this negative result isolates.

## Reproduction
Prototypes and full per-pivot data tables lived in `M:\claud_projects\temp\rung25\` (proto4–8,
`RESULT_proto*.md`; `proto8.py` is the σ-law discriminator). That folder is **outside git** (project
temp policy), so this tracked file is self-contained and is the durable record. The prototypes reused
the shipped rung-24 chemistry (`Gas.zoned_nox(..., spatial_local=SpatialLocalPDF(...))`) via a
module-level monkeypatch of `gas._spatial_local_field` with the growing-σ field and a `CappedMixing`
subclass overriding `tau_q` — i.e. **nothing in the shipped code was changed.**
