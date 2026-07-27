# A limiter with BOTH edges inside the ramp — INVESTIGATED, **NOT SHIPPED**

> **Status.** A **negative-result record**, not a rung spec. It is the attack on the live remainder
> that `docs/pt3-sensor-lag-negative.md` § *What WOULD reopen this* named: *"a limiter whose
> engagement AND release are BOTH inside the ramp — e.g. a **rate-limited** or **washout/lead-lag
> filtered** `pt3` (not a pure lag) … **That is the live version of this seam.**"* It was derived,
> scanned and measured, returned a negative verdict, and was **deliberately not added to the ladder**
> — no `*-spec.md`, no `engine.py`/`main.py`/test code, no rung-table entry, by design. It sits in the
> `docs/tau-res-negative.md` / `docs/mixing-scale-negative.md` /
> `docs/mixing-jicf-anchor-negative.md` / `docs/pt3-sensor-lag-negative.md` family.
>
> **Why this file exists:** the negative is **structural — it closes the seam for the whole family of
> `pt3` filters, not one attack on it** — and it carries a positive by-product that is a strict
> upgrade of rung 48's law, plus two corrections to rung 48's spec text. **If you are about to build
> a rate-limited or washout-filtered `Wf/pt3` leg, read this first — it cannot work, and the question
> it was meant to answer is already answered.**

## What this investigation was FOR

Rung 48 shipped the feedforward `Wf/pt3` accel schedule and the unifying rule: **a fuel-side limiter
rebates a spool IFF it engages UPSTREAM of THAT spool's own surge minimum.** With a *single-edged*
limiter, "engages upstream of `s*`" and "the engaged window intersects `[0, s*)`" are the **same
statement**, so rung 48 could not distinguish

- **(E) an EDGE condition** — only the engagement instant matters, from
- **(W) a WINDOW condition** — what matters is the engaged window's overlap with `[0, s*)`.

A limiter with **two edges inside the ramp** would separate them. That was the target.

Config throughout is rung 48's: CPG gas, accel 1000→1400 K, `ρ = 1`, shapes `flow/press`; at
`r = 0.5`, `s_lp* = 0.24`, `s_hp* = 0.40`, `ν_H_end = 0.95906`.

**The verdict has two halves, and they turn out to be ONE structural fact.**

---

# HALF 1 — the two-edged limiter is structurally UNREACHABLE, for every `pt3` filter

## The common frame (this is the whole algebra)

Every candidate is a multiplicative modification `ρ ≡ p̃/pt3` of the pressure feeding rung 48's cap —
the deflation `docs/pt3-sensor-lag-negative.md` already established. The leg is ENGAGED iff

```
E(s) = R(s) − (1+m)·ρ(s) > 0,        R(s) ≡ mf_sched / (κ_ss(n_H)·pt3)
```

`R` is rung 48's finding-1 ratio. **Both edges inside the ramp ⟺ `E` changes sign twice on `(0, r)`.**

### First, a CORRECTION to the naive reading — `R` is not the bare `R` after engagement

`R` rides the *actual* trajectory. Once the leg clips, `pt3` is lower (`R` ↑) **and** `n_H` is lower so
`κ_ss` is lower (`R` ↑ again). Both measured, not assumed:

- `κ_ss(n_H)` is **strictly rising** on the running line: 1.4448e-8 → 1.9170e-8 over
  `n_H` = 0.83211 → 0.96881. So the second term's sign is confirmed.
- At `m = 0.25, r = 0.5`: `R_lim(sched)` reaches **1.9887** at `s = 0.5` against
  `R_bare(sched) = 1.5722` — a **26 % divergence**, opening from engagement onward.

**Engagement is SELF-REINFORCING, so RELEASE is harder than rung 48's bare finding-1 table (which
decelerates to 1.4885) suggests.** Every scan below is run on the *bare* `R`, i.e. it is an
**optimistic** bound on the release edge — which only strengthens the negative.

## Candidate 1a — a rate limit on the FUEL COMMAND: dead BY ALGEBRA, no code

On the linear ramp `mf_sched(s) = mf_lo + Δmf·s/r`, a command rate limit `Ẇ` binds iff `Ẇ < Δmf/r`,
and the applied command is then *exactly* the linear ramp of slope `Ẇ`, reaching `mf_hi` at
`s_catch = Δmf/Ẇ > r`. That is **rung 45's own ramp with `r′ = Δmf/Ẇ` — rung 44's ramp-rate lever BY
IDENTITY, not by resemblance.** Both edges miss the target regardless: engagement at `s = 0⁺` (the
boundary, not the interior), release strictly **post-ramp**. *The user's own prior suspicion that "the
rate limit likely degenerates into rung 44's lever" is exactly right, and it can be stated as an
identity rather than a likelihood.*

## Candidate 1b — a rate limit on the SENSED `pt3` (a slew limiter): dead STRUCTURALLY

A slew limiter has `ρ ≤ 1` (reads LOW ⇒ engages EARLY), so it is in the **same family as the pure lag
already found negative**. It has exactly one novelty, and it is a genuine candidate second edge: a
**HARD catch-up** where `ρ = 1` *exactly* (the lag only approaches asymptotically). At that instant
the cap jumps to rung 48's cap and the leg can RELEASE.

**Catch-up requires `d(pt3)/ds` to FALL below the rate limit inside the ramp.** Measured, with
**forward** differences whose windows never straddle `s = r`:

| `r` | `ds` | `d(pt3)/ds` first → last in-ramp | monotone RISING through the interior? |
|---|---|---|---|
| 0.15 | 0.005 | +1.2183e6 → +1.5981e6 | **True** |
| 0.15 | 0.0025 | +1.2183e6 → +1.6034e6 | **True** |
| 0.5 | 0.01 | +3.7023e5 → +8.4226e5 | **True** |
| 2.0 | 0.02 | +9.4484e4 → +2.7846e5 | True but for −21 out of 281 089 (**0.008 %**) at `s` = 1.64 |

The single exception is 0.008 % in size and sits at `s = 1.64`, far **downstream** of both minima
(0.32 / 0.64). So there is no recovery interval anywhere upstream of a surge minimum: the slew
limiter's catch-up is **structurally post-ramp** and it dies the pure lag's death, inherited verbatim.

**And the reason is structural, not config-specific:** `d(pt3)/ds` is driven by the spool
acceleration, which is driven by the power imbalance, which **grows monotonically through a
monotone-rising fuel ramp**. A rising fuel ramp cannot produce a falling pressure rate inside itself.

> **A METHOD WARNING, recorded so it is not re-discovered.** A first pass using **central**
> differences reported `d(pt3)/ds` *falling* at `r = 0.15, s = 0.140` — and that fall is an artifact:
> the window `[0.12, 0.16]` **straddles the ramp end at 0.15**, i.e. the kink in the fuel command.
> Under forward differencing at 2× and 4× finer `ds` it disappears entirely. **Difference one-sidedly
> inside the ramp.** (Same shape as the negative doc's sub-grid `s_eng` artifact — this seam punishes
> naive discretization twice.)

## Candidate 2 — the WASHOUT / LEAD-LAG: dead structurally, and by a 1104-config scan

Derived from the ladder's own history rather than an imported filter shape: the negative doc gave
`pt3` a first-order lag `q`; **lead compensation is the standard answer to a laggy transducer**, and
*exact* inversion (`pt3 = q + τ·dq/ds`) recovers rung 48 identically. **Over**-compensation with lead
constant `τ_L > τ`, writing `K ≡ τ_L/τ − 1`:

```
dq/ds = (pt3 − q)/τ,   q(0) = pt3(0)      p̃ = pt3 + K·(pt3 − q)     ⇒  ρ = 1 + K·(1 − q/pt3)
```

One scalar spans the family and **contains both existing points**: `K = −1` ⇒ `p̃ = q`, the pure lag
(the negative doc); `K = 0` ⇒ `p̃ = pt3`, rung 48 **bit-for-bit**; `K > 0` ⇒ the new lead territory.

**Why it still fails.** A release needs `(1+m)·ρ` to **overtake** `R` — which requires `R` to turn
over. But `R = mf_sched/(κ_ss·pt3)` can only turn over when **`mf_sched` SATURATES**, i.e. at the ramp
end (numerator stops growing, denominator keeps growing). Measured: `R_bare` peaks at 1.48859 @
`s = 0.4500` for `r = 0.5` (ramp end 0.5), and at 1.80675 @ `s = 0.1500` for `r = 0.15` — **AT the
ramp end, so no interior turnover at all.** Correspondingly the washout deficit `1 − q/pt3` peaks at
`s = r` for every `τ ≤ 0.2` and *post*-ramp for larger `τ`.

Scan over `τ ∈ {0.05, 0.1, 0.2, 0.4}` × `K ∈ {0.5, 1, 2, 4, 8, 16}` × 23 margins × `r ∈ {0.15, 0.5}`
= **1104 configurations**:

| `r` | configs with two edges inside `(0, r)` | of those, both edges upstream of `s_lp*` |
|---|---|---|
| 0.15 | **0** | 0 |
| 0.5 | **2** — `(τ=0.4, K=1, m=0.25)` eng@0.2270 rel@**0.4749**; `(τ=0.4, K=2, m=0.10)` eng@0.1072 rel@**0.4170** | **0** |

Both surviving releases land at 0.417 / 0.475 against `s_lp* = 0.23` **and `s_hp* = 0.39` — downstream
of BOTH minima.** *And `K` is not the independent release dial the design hoped for:* `ρ = 1 + K·d(s)`
with `d` monotone-rising means raising `K` moves **both** edges inward — the engaged window can only
shrink toward the crossing region, it cannot slide.

## THE UNIFYING REASON — the ramp is the only clock in the system

> Every candidate second edge — the pure lag's `ρ→1`, the slew limiter's hard catch-up, the washout's
> turnover, and the `R` turnover a lead must exploit — is manufactured by the **same event: the fuel
> ramp flattening.** And both surge minima are **ramp-driven**, hence strictly INSIDE the ramp. **So
> no filter on `pt3` can place a release edge upstream of a surge minimum.**

That is why this closes the **seam**, not one attack on it. It is also the same argument the pure-lag
negative used for its release edge — now shown to govern the entire `pt3`-filter family, including
the two candidates that doc explicitly held open as the live version.

---

# HALF 2 — the question the instrument was FOR is answered anyway, by a MECHANISM

The two-edged limiter was a *means*; edge-vs-window is the *end*. It is settled directly, and
decisively, on rung 48's own leg.

## The mechanism: a clip ARRESTS the φ descent, immediately and permanently

At `r = 0.5, m = 0.15`, **`ds = 0.01`** (so `s_lp*` reads 0.230 here, against 0.240 on rung 48's
`ds = 0.02` grid quoted in the preamble — both correct at their own step), φ_LP descends
**bit-identically with bare** through `s = 0.060`; the clip fires at `s = 0.070`; and φ_LP **turns
around and climbs monotonically from that instant**, through and past the bare minimum at 0.230:

| `s` (`ds` = 0.01) | 0.040 | 0.050 | 0.060 | **0.070** | 0.080 | 0.090 | … | 0.230 |
|---|---|---|---|---|---|---|---|---|
| φ_LP bare | 0.760992 | 0.758340 | 0.755840 | 0.753489 | 0.751288 | 0.749236 | … | **0.735448 ← min** |
| φ_LP limited | 0.760992 | 0.758340 | 0.755840 | **0.753996 ← min, CLIP fires** | 0.754427 | 0.754860 | … | 0.761086 |

Not a clip-created dip, and not a flattened monotone trace — **a genuine interior minimum AT
engagement.**

**The arrest is verified at the scope the predictor actually needs.** The formula below is a claim
about the **global** min of the limited march, so the load-bearing window is `s_eng → the END of the
trajectory` (settle included) — a dip anywhere after `s*`, post-ramp or after release, would move the
global min and break it. Both scopes were checked: φ is monotone non-decreasing from `s_eng` to `s*`
**and** from `s_eng` to the end of the march, in **all 16 armed rows × both spools = 32/32 cells**,
across `r` = 0.15 / 0.5 / 2.0. The limited march's global min sits at `min(s_eng, s*)` to within one
grid cell — which is exactly what `min_{s ≤ s_eng} φ_bare` encodes.

Physically: clipping fuel lowers `Tt4`, which raises the choked-NGV corrected-flow capacity, so the
compressor must pass more flow and `φ = m/n` rises at once.

## Therefore the law is an EDGE condition NECESSARILY — the window is EMPTY, not merely untestable

**The minimum is fixed at the OPENING of the engaged window.** Its length, the fuel it removes, and
its release edge are all causally *downstream* of a minimum already determined. So (W) cannot hold —
not because we failed to build the instrument, but because there is nothing for the window to do.

**And this is the SAME fact as Half 1:** the arrest is never undone because release is structurally
post-ramp. Measured on rung 48's own leg at `r = 0.5`: `s_rel` = 1.120 / 0.720 / 0.580 at
`m` = 0.25 / 0.35 / 0.42 — every one **after** the ramp ends at 0.5. The negative and the positive are
two faces of one structural claim.

## The sign becomes a closed-form MAGNITUDE, from ONE bare march

Bit-identical to `s_eng` (rung 48's gate 8b) **+** arrested after it ⇒

```
        relief  =  min_{s ≤ s_eng} φ_bare  −  min_s φ_bare
```

- **It recovers rung 48's exact-zero EXACTLY, at any `ds`.** For `s_eng > s*` the restricted window
  *contains* `s*`, so the two minima coincide and the prediction is identically 0 — verified at every
  such row (`m` = 0.40/0.42/0.45/0.48 at `r`=0.5; `m` = 0.15/0.20 at `r`=2.0). **Rung 48's crossing
  rule is DERIVED, not fitted.** *(The naive form `φ_bare(s_eng) − min φ_bare` is WRONG here — it
  predicts +0.0008/+0.0035/+0.0138 where the measured relief is exactly 0. The restriction to
  `[0, s_eng]` is load-bearing.)*
- **It is exact in the `ds → 0` limit — first-order convergent.** At `r = 0.5, m = 0.35` the relative
  error **halves as `ds` halves**:

  | `ds` | 0.02 | 0.01 | 0.005 | 0.0025 | 0.00125 |
  |---|---|---|---|---|---|
  | err % | 62.0 | 41.2 | 21.9 | 11.9 | **6.1** |
  | `err/ds` | 0.056 | 0.050 | 0.053 | 0.051 | 0.052 |

  `err/ds` constant ⇒ error is `O(ds)`, one grid step of the residual descent (`min@` sits 0–1 cells
  before the recorded `s_eng`). **A theorem about the mechanism, not a curve fit.**
- At production `ds` it is already accurate far upstream: **−1.3 % to −7.3 %** (`r`=0.5 `m`=0.15 at
  `ds`=0.01; `r`=0.15 `m`=0.20/0.40/0.60 at `ds`=0.005), both spools. The relative error blows up
  **near** the crossing only because the relief itself → 0 there while the absolute error stays ~5e-4.

**This is a strict upgrade of rung 48's law:** from *"rebates iff it engages upstream"* (a sign) to
*"the relief IS the portion of that spool's bare descent the limiter truncates"* (a magnitude,
predicted with **no limited march at all**).

---

## By-product — two corrections to rung 48's SPEC TEXT (the code is fine)

1. **Gate 10's description is wrong in the spec.** `docs/rung48-spec.md` gate 10 read *"`ν_H` at
   settle is unchanged to **1e-4** for `m ≥ 0.10`"*. The shipped test asserts **5e-4** (with an
   explicit comment saying 5e-4 and why) and sweeps `MARGINS = (0.15, 0.25, 0.35, 0.42, 0.45, 0.48)`
   — **`m = 0.10` is not in the swept set.** Verified green:
   `pytest tests/test_rung48.py -k "not_ramp_rate_lever or degeneracy" --runslow` ⇒ **2 passed**.
2. **The stated admissible window `m ∈ [0.10, 0.45]` is too wide at its low end.** Measured at
   `m = 0.10`, `r = 0.5`: `ν_H_end = 0.95714` against bare `0.95906` — a shift of **1.9e-3, ~4× the
   gate's own 5e-4 tolerance.** The honest window is **`m ∈ [0.15, 0.45]`**, which is what the gate
   actually tests. Both are corrected in `docs/rung48-spec.md`.

Also recorded: rung 48's relief is differenced off the **GLOBAL** raw min φ. At strong clip the
limited march's global min relocates to `s_eng`, so a global and a locally-windowed relief differ —
at `m` = 0.10/0.15 (the two lowest margins of the *stated* window, i.e. the same over-wide low end)
by 7–9 %, while for LP at **`m ≥ 0.25` they agree EXACTLY**, which is where rung 48's headline
crossing lives. **Under the mechanism above this is not a defect but a consequence** — the minimum is
*supposed* to relocate to `s_eng` — so no change to the shipped object is warranted; only the window
correction (2).

## What WOULD reopen this

Not a different `pt3` filter — Half 1 covers the family. The negative rests on one fact: **the only
signals with a turnover UPSTREAM of a surge minimum are the surge variables themselves.** `pt3`,
`Wf`, `n`, and every filter of them rise monotonically through the ramp; **φ has its minimum there by
definition.** So the live door is a **φ / surge-margin FEEDBACK limiter** — a limiter that watches the
thing being protected rather than a proxy for it. That is the one instrument that could place an edge
where a minimum lives, and hence the only one for which (W) could still be live. It is also a
genuinely different control object (feedback on the *protected variable*, where rung 46/47's is
feedback on TIT and rung 48's is feedforward on pressure), so it is a rung candidate, not a filter
variation.

## FOLLOW-UP — the named successor was BUILT, and it changed the answer to (W)

`docs/rung49-spec.md` walks through the door named above. Both halves of this doc survive
intact: Half 1's argument was always a statement about *proxy* signals, and it correctly
identified the one class that escapes it — a φ floor **does** close its window inside the ramp
(measured: `[0.120, 0.440]` and `[0.150, 0.350]` at `r`=0.5).

But **Half 2's conclusion does not extend to that instrument.** The truncated-descent law was
derived on rung 48's leg, whose release is structurally post-ramp — a **one-shot** arrest. Give
a limiter an interior release edge and a **second, opposite-signed term appears**: the withheld
fuel is handed back to a still-ramping plant and the descent **RE-OPENS**. So an LP φ-floor
**debits the HP** even though it engages upstream of `s_hp*`, and the unwatched minimum
relocates to one step *after* `s_rel`.

**(W) is therefore answered — and the answer is YES, the closing edge does something.** The
window is not empty; it was only empty for the family this doc closed. What survives verbatim
is the *mechanism*: the credit term is exactly `min_{s ≤ s_eng} φ_bare − min φ_bare`, and rung
49 reproduces its **exact zero** on a fresh instrument. What is corrected is the claim that the
release edge is *causally downstream of a minimum already determined* — true only when the
release lands in the settled region.

One more thing this doc got right and is worth keeping: *"the ramp is the only clock."* Rung 49
finds it governs the closing edge **instead of** the per-spool structure that governs the
opening one — at `r`=2.0 the debit is 8× larger at `s_rel≈r` than at `s_rel≈s_hp*`. That is a
**within-family** result: this doc's own 32/32 monotone-cell measurement shows rung 48's leg is
immune to the release debit at ratios where the φ floor is not (`s_rel/r` = 1.16 vs 1.20), so
*why that leg escapes* is left open by rung 49, not answered by it.

## Method note

Probes: `M:\claud_projects\temp\rung49-probe\` — `probe_rates.py` (`κ_ss` monotonicity, `d(pt3)/ds`,
the `R` self-reinforcement), `probe_rate_fine.py` (the one-sided-difference resolution),
`probe_lead_size.py` (the washout deficit), `probe_edges.py` (the 1104-config two-edge scan),
`probe_local_min.py` + `probe_shape.py` (global-vs-local, then the deciding shape check),
`probe_predictor.py` / `probe_converge.py` (the predictor and its `ds → 0` convergence);
`ALGEBRA.md` / `FINDINGS-RUNG49.md` are the working notes. No project file was modified by the
investigation except this doc, the rung-48 spec corrections, and the CLAUDE.md status map.
