# Rung 79 § 9's CONSTANT GAP — checked, and it CORRECTS rung 79 § 5

**Not a rung.** No new effect, no new plant code, no new constant — one swept knob (rung 48's
`margin`) and two shipped readers. This is a **CORRECTION to a shipped rung**, in the bucket
`docs/rung29-pi-c-margin.md` and `docs/rung28-beta-margin.md` occupy. Rung 28 is the repo's own
precedent for editing a shipped rung's spec from outside it.

**HONESTY, FIRST.** This was found by **exploration, not by a pre-registered prediction.** The
project's rule — *a prediction edited after the measurement is not a prediction* — cuts both
ways, so nothing below is scored as a prediction met. The one thing that was decided in advance
is the *discriminator* (§ 1), and it is named as such.

---

## 0. THE SEAM, IN RUNG 79 § 9's OWN WORDS

> **THE CONSTANT GAP.** 12.6136% to 13 digits across the accel (§ 6). **The obvious candidate is
> rung 48's `(1 + margin)` factor** — `margin = 0.10` here, and the accel cap carries it directly
> — with the residual ~2.6% from `κ(n_H)`'s own drift; that is a guess, and checking it means
> sweeping `margin` and seeing whether the gap tracks it. If it does, this is a **structural**
> relation between rung 48's schedule and rung 49's floor that no rung has named.

And rung 79 § 6's matching concession:

> **The gap is essentially constant and unexplained.** `gap ∈ [0.126136034007005,
> 0.126136034007016]` over 129 distinct values … That is measured, not derived; the schedule is
> built from the same plant through `accel_for`, which would explain it, and that explanation is
> **not checked**.

**Both are now checked. The premise is REFUTED and the residual has an OWNER.**

---

## 1. THE DISCRIMINATOR, CHOSEN BEFORE ANY SWEEP

Sweeping `margin` first would have confirmed `AccelSchedule.cap`'s own source line
(`(1.0 + self.margin) * k * pt3`) and left the seam's real question untouched. The gap's spread
is `gap_max − gap_min ≈ 1.1e−14` **relative over 1366 calls** — float noise. § 9's guessed
mechanism requires `κ(n_H)` to **drift**, and a drift shows up as **spread**. There is none.

So the discriminator is: **print `κ_ss(n_H)` over the band the accel traverses.**

    kappa_ss spread over the WHOLE table : 3.214196e-01   (32%, 13 entries)
    traversed n_H                        : [0.83574801, 0.83574801]
    kappa_ss spread THERE                : 0.000000e+00   (exactly)
    CLAMPED at the table's FIRST abscissa : 341 of 341 steps

`n_H` never leaves the schedule's low-end clamp. **`κ_ss` cannot drift because the plant never
moves.** Which raised the question this document is actually about.

---

## 2. THE FINDING — RUNG 79 § 5's MARCH NEVER LEAVES ITS INITIAL STATE

`coord_march`'s trajectory at rung 79's shipped settings, 341 steps, relative spread per key:

| key | range | rel. spread |
|---|---|---|
| `nu_lp` | [0.747544109, 0.747544109] | **exactly 0.0** |
| `nu_hp` | [0.790154086, 0.790154086] | **exactly 0.0** |
| `Tt4` | [1000.0, 1000.0] | 1.59e−15 |
| `mf` | [9.44449798e−03, …] | 1.29e−15 |
| `phi_lp` | [0.800000, 0.800000] | 1.67e−15 |
| `pt3` | — | 1.43e−15 |
| `s` | [0.0, 1.7] | — |
| `mf_sched` | [9.4445e−03, 2.3405e−02] | **1.478** |

**The commanded fuel ramps 1.48×; nothing else moves at all.** The two spool speeds are
constant to the last bit over the whole "accel". Rung 79 § 5's 1366 `_cap_fuel` calls are
**1366 calls at ONE operating point**.

**Rung 78's march is the same** (`ResidualGaugeTransient`, same rig, same settings):
`nu_lp` and `nu_hp` relative spread **exactly 0.0**, `phi_lp` = 0.800000. That is a sufficient
explanation for rung 78 § 5's `binds = 0` on the accel leg, which it recorded as a masked leg.

### 2.1 THE MECHANISM — A FLOOR ARMED EXACTLY AT THE INITIAL CONDITION

The rig arms **three** φ floors at the same wall `φ_lim = 0.80` (rung 49's fuel leg, rung 64's
bleed valve, rung 53/68's stator), all from the same `from_margin(cmap, ., sm)`. Disarming them
one at a time:

| armed | `φ_lp(start)` | `nu_lp` spread | `Tt4` range |
|---|---|---|---|
| **all four (shipped)** | 0.800000 | **0.0** | [1000.0, 1000.0] |
| no valve | 0.800000 | 4.75e−02 | [970.5, 986.0] |
| no fuel leg | 0.800000 | 1.15e−01 | [1000.0, **1283.1**] |
| no valve, no fuel | 0.800000 | 1.03e−01 | [985.6, 1282.7] |
| **stator only** | **0.8000000000** | — | — |
| **nothing armed** | **0.7731162133** | — | — |

Read bottom-up, that is the whole mechanism:

1. The **free** plant's initial operating point is `φ_lp = 0.7731`, **below** the wall.
2. The **stator** floor lifts it to **exactly 0.80** (`v(start) = −0.0675`, so the lever is
   demonstrably doing the lifting — it is not a coincidence of the design point).
3. Rung 49's **fuel** leg then reads a state that is already **ON** its own floor, so
   *"the fuel that holds `φ ≥ φ_lim`"* evaluates to **the fuel already flowing**.
4. The cap therefore equals the current fuel at every step, and the plant **cannot accelerate**.

**A limiter armed with zero initial margin has no transient.** The fuel leg is not late, not
lagged, and not masked — it is binding at `s = 0` with no authority left to give.

### 2.2 IT IS THE FLOOR PLACEMENT, NOT A BROKEN MARCH — POSITIVE CONTROL

The same rig, same code path, wall lowered:

| `φ_lim` | `nu_lp` spread | `nu_hp` spread | `Tt4` range |
|---|---|---|---|
| 0.80 | **0.0** | **0.0** | [1000.0, 1000.0] |
| 0.75 | 6.999e−02 | 5.889e−02 | [1000.0, 1180.0] |
| 0.70 | 6.999e−02 | 5.889e−02 | [1000.0, 1180.0] |

At `φ_lim ≤ 0.75` the march accelerates by 180 K and the two lower walls give **identical**
trajectories — i.e. below 0.7731 the leg is simply dormant. The standstill is a **threshold**
property of where the wall sits relative to the initial condition, and 0.80 is on the wrong side
of it by construction. (`φ_lim = 0.60` is not reachable: rung 74's `_cap_free` refuses the
operating point, correctly.)

### 2.3 WHY THE RIG IS LIKE THAT, AND WHY IT MUST NOT BE "FIXED"

`PHI_JAC = 0.80` sitting exactly on the wall is **what rung 79 §§ 1–4 require**: a constrained
linearisation has to be taken with every loop sitting **on** its constraint, or the Jacobian is
of the wrong system. The rig is correct for §§ 1–4 and **silently voids § 5** — the same
placement buys one section and destroys the other.

**So this document does NOT re-rig rung 79.** Changing `PHI_JAC` would rewrite every scored
number in rungs 72–79 and break `test_numeric_fingerprint.py`'s bit-exact rung-78/79 arms. The
standstill is **recorded and gated** instead (§ 4), so no future reader can quote § 5 as a
trajectory and no future edit can unpin it without forcing a rescore.

---

## 3. THE GAP'S RESIDUAL HAS AN OWNER — AND IT IS RUNG 77's STIFFNESS

With the plant pinned, `p_phi` (the φ cap) **is** the flowing fuel `mf`, exactly. So

    gap + 1  =  a_cap / mf

and `a_cap` is rung 76's `solve` cap law: the **fixed point** of `w = (1+m)·κ·pt3(w)`. Sweeping
`margin`:

| `margin` | `gap_min` | `binds/hits` |
|---|---|---|
| 0.00 | **0.000000000000** | 1313/1366 |
| 0.02 | 0.024872654543 | 1366/1366 |
| 0.05 | 0.062515637968 | 1366/1366 |
| 0.10 | 0.126136034007 | 1366/1366 |
| 0.20 | 0.256624904072 | 1366/1366 |
| 0.30 | 0.391329764709 | 1366/1366 |
| 0.50 | 0.672812399700 | 1366/1366 |

**§ 9's model is refuted at its own anchor point.** It predicted `gap ≈ margin + 0.026`; at
`margin = 0` the gap is **exactly zero**, so there is no constant offset to explain. And the
reason is § 2: **a standing plant is AT steady state, and rung 48's `margin = 0` schedule IS the
steady-state fuel** — `κ_ss(n_H)·pt3` and the φ floor's cap are then the same number identically.

The residual is not an offset, it is an **exponent**:

    d ln(gap + 1) / d ln(1 + margin)  =  1 / (1 − c) ,     c = d(cap)/dw     [rung 77's scalar]

because `a_cap` is a fixed point, not an evaluation. **Two independent instruments, and the
comparison point is the whole test:**

| `margin` | sweep slope | `1/(1−c)` at **`a_cap`** | miss | `1/(1−c)` at `mf` | miss |
|---|---|---|---|---|---|
| 0.10 | 1.253555469 | 1.253562138 | **5.3e−06** | 1.255578596 | 1.6e−03 |
| 0.30 | 1.277876030 | 1.277881046 | **3.9e−06** | 1.316767236 | 3.0e−02 |

`c` is rung 77's own shipped `_c_at`, unmodified. Read at the **fixed point** — the point whose
response is being swept — the two agree to **six significant figures**. Read at the plant's fuel
(12.6% and 39.1% away in `w`) they miss by 3 and 4 orders more.

**That gradient IS the non-vacuity control.** An instrument that agreed at both points would be
insensitive to where it was evaluated and would therefore be measuring nothing — this project's
recorded failure mode (rung 77 § 8's `1.000e+00`, rung 73's perfect confirmation). It does not
agree at both points; it agrees at exactly one, and that is the one the algebra names.

**VERDICT ON § 9's THREE CLAUSES:**

* *"the obvious candidate is rung 48's `(1 + margin)` factor"* — **HELD**, the factor is there.
* *"with the residual ~2.6% from `κ(n_H)`'s own drift"* — **REFUTED.** `κ` does not drift
  (§ 1: zero spread, clamped abscissa). The residual is rung 76's `solve` cap law being a
  **fixed point**, whose gain is rung 77's `1/(1−c)`.
* *"a **structural** relation between rung 48's schedule and rung 49's floor that no rung has
  named"* — **REFUTED, and it is the interesting half.** There is no relation between the two
  legs. The gap is `(1+margin)` raised to rung 77's stiffness, and rung 49's floor enters only by
  **standing the plant still**, which is what makes `p_phi = mf` and turns a trajectory into an
  identity.

---

## 4. WHAT THIS CORRECTS IN RUNG 79 — AND THE FOURTH VACUITY TRAP

### 4.1 THE `n_distinct` COUNTER DOES NOT SAY WHAT ITS DOCSTRING SAYS

`tests/test_rung79.py::test_the_gap_log_records_distinct_states` asserted `n_distinct > 10` and
its docstring read that as refuting *"ONE state logged 1366 times"*:

> `gap_min ~= gap_med` to 13 digits reads as ONE state logged 1366 times. It is not — 129
> distinct gaps over 42 distinct phi caps. COUNT, never eyeball.

**It is exactly that.** The 129 distinct `p_phi` values are float-level products of a bracketed
solve whose **start point** `mf_sched` sweeps 1.478× while the plant state is constant to 1e−15.
The counter distinguishes distinct **floats**; the claim needed distinct **states**.

So rung 79 § 5.5's *three* instrument failures are **four**, and the fourth survived the ship —
inside the guard § 8.1 is proudest of. **A counter is only as good as the object it counts**,
and "count, never eyeball" does not protect you if you count the wrong noun. The docstring is
corrected and the state-level check added beside it.

### 4.2 WHAT SURVIVES, AND WHAT DOES NOT

**SURVIVES — rung 79's headline is untouched.** *A coordinate is a GAUGE the PLANT cannot
REACH* rests on § 5.3's complementarity, which is an **identity of `_cap_free`'s branch
condition** (`{knob live}` and `{leg reaches applied fuel}` are disjoint because the
short-circuit fires exactly when the cap beats the schedule). That is structural, and a static
plant satisfies it as surely as a moving one. § 5.2's forced-bypass numbers (`6.14e−15` /
`7.64e−16`) are frozen-state measurements by construction and are unaffected. §§ 1–4 are read at
frozen states throughout and require the rig to be exactly as it is (§ 2.3).

**DOES NOT SURVIVE — every § 5 reading that was phrased as being *across the accel*.**
`hits`/`binds` = 1366/1366, `n_live` = 3, `n_reach` = 1363, `flips` = 0, `d_max` = 0.0 are all
**true at one operating point**. P4's `worst = 0.000e+00` was already scored VACUOUS for the
right reason (`d_set = 0` on the plant); it is now vacuous for a **second, independent** reason,
and that one applies to every trajectory statement in the section.

Rung 79 § 6's last concession read *"One operating point, one leg, one coordinate pair"* and
meant **one setting**. It is literally true in the far stronger sense of **one state**.

---

## 5. THE GATES

In `tests/test_rung79.py`, per this repo's convention (rungs 28/29's margin sweeps gate inside
the rung's own file):

* **the standstill is PINNED** — `nu_lp`/`nu_hp` relative spread `== 0.0`. Not a bug being
  blessed: it is the § 5 scope condition, and an edit that unpins it must rescore § 5 rather
  than silently change what the section measured.
* **the positive control** — the same rig at `φ_lim = 0.75` moves. Without it the standstill
  gate would also pass a march that was broken for any other reason.
* **`gap(margin = 0) == 0`** — the identity of § 3, which is what refutes § 9's offset model.
* **the stiffness identity** — sweep slope vs `1/(1−c)` at the fixed point, plus the
  **evaluation-point control** (reading `c` at `mf` must miss by ≫ the tolerance, or the
  agreement is insensitive and therefore vacuous).

---

## 6. WHAT IS STILL OPEN

* **A φ-floor march with NON-ZERO initial margin.** Everything rungs 72–79 claim *about
  trajectories* wants a rig whose wall is below the initial operating point and above the free
  excursion minimum — i.e. `φ_lim ∈ (0.7731, 0.7884)` at these settings, a ~1.5% window. That is
  a new rig, and it would not be comparable to §§ 1–4 (§ 2.3).
* **Rungs 72–77's § 5 sections.** Only rungs 78 and 79 were checked here. They share the rig —
  and that is a reason to look, **not** a result. **NOT MEASURED; this document makes no claim
  about them, and no later reader should quote one.** (A hedge in a project whose discipline is
  *count, don't infer* reads as a finding within two citations.)
* **Whether a discontinuous selector can amplify a coordinate's float noise** — rung 79 § 9's
  third bullet, still untouched and now doubly out of reach: the legs are 12.6% apart *and* the
  plant is static.
