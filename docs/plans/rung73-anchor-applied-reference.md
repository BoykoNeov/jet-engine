# Rung 73 anchor — THE APPLIED REFERENCE (rung 72 § 11's sharpest seam)

Scored in `docs/rung73-spec.md` § 9. The rule this project runs under: **a prediction that is
edited after the measurement is not a prediction.**

**AND THIS ANCHOR DISCLOSES ITS OWN ORDER, as rungs 71's and 72's did — and it has MORE to
disclose than either.** Two things were measured before this document existed, both in
`M:\claud_projects\temp\rung73\` (`precheck0.py`, `precheck1.py`, `precheck2.py`):

1. **A FEASIBILITY KILL-CHECK (§ 0.1–0.3), and it was not optional.** A leg referenced to the
   applied fuel is an *integrator* on the clip, and a masked integrator is the textbook
   min-select **windup**: `gf` could ramp over 100+ masked points with only a floor at zero
   under it, and the hand-over would then slam a wound-up clip onto the actuator and starve the
   engine — which is exactly how rung 72 § 4's SUM law died (84 points of 341). If that had
   happened, the rung would have needed an anti-windup device and would have been a *different*
   rung. It did not happen, and the reason is derivable (§ 0.2).
2. **THE ENTRYWISE `J(73) − J(72)` (§ 0.4).** This is the measurement that decides whether the
   framing is right at all, and running it late would have meant writing an anchor around a
   structure that might not exist. **It is reported as measurement and is NOT scored as
   prediction.** What § 2B scores is everything that measurement does not settle: the zero
   counts per cell, the determinant, the polynomial form, the eigenvector, the ledger, the
   instrument, and the reduce.

§ 2 is therefore split as rung 72's was. **§ 2A is DERIVED** — worked out on paper from the two
inherited laws, before any spectrum existed — and is listed as derivation, **not scored as
prediction**, except where § 9 finds a derivation measured false (rung 72's D5 precedent: that
section exists so paperwork is not passed off as prediction, **not** so a wrong derivation
escapes correction).

---

## 0. THE KILL-CHECK

### 0.1 The seam, and what "applied-referenced" has to mean

Rung 72 § 1.1: both fuel-side laws compute their clip from the **scheduled** fuel — rung 47's
discipline (*`required` is what the clip WOULD have to be*) and rung 52's (*solved from the
scheduled fuel so arming one leg cannot perturb another's bracket*). Rung 72 § 6 concedes and
§ 11 names the seam: *a leg referenced to the applied fuel would give `F_r ≠ 0`, both fuel rows
would couple, and the block would not be triangular.*

**The first thing to settle is what the seam even means, because the naive reading is not one
plant but three.** Every cap in this ladder — `_topping_fuel`, `_surge_fuel`, `_sched_fuel` — is
a **SET-POINT SOLVE**: it returns the fuel at which the constraint is exactly met, which is a
function of `(ν, q, v)` and **not of the fuel it was asked about**. So `∂required/∂mf` is not a
gradient; it is the **branch indicator** `{0, 1}` (dormant / binding). Three readings follow:

| | law | fixed point when the leg HOLDS | verdict |
|---|---|---|---|
| **A** | only the DORMANCY TEST moves to `mf_app` | unchanged | not a plant — the guard half, which B and C both inherit |
| **B** | `req = g_own + (mf_app − cap)` — incremental | `mf_app = cap` ✓ | **THE PLANT** |
| **C** | `req = mf_app − cap` — literal | `g = (mf_sched − cap)/2` ✗ | a P-controller with **2× droop**; § 3's instrument |

**B is the plant** because it is the only reading under which the leg still reaches its own set
point — and because it coincides with rung 72 **exactly** wherever the leg holds authority.
**C is not refused as broken** — it is a well-posed proportional law — but it is *degenerate for
this ladder*: a leg that structurally cannot reach its own floor makes every currency in the
rung-46…72 ledger measure a different object. It is carried as § 3's isolation instrument, the
role rung 72's own SUM law and rungs 50/51's forced release edges played. **Refusing it silently
is what rung 63's lesson is about.**

**AND B HAS A CLOSED FORM ON THE INHERITED LAW, WHICH IS WHY NO SOLVER CHANGES.** Because `cap`
is fuel-independent, `mf_app − cap = req_sched − applied_clip` identically, so

    req_applied = g_own + req_sched − max(gf, gr)          [the hook, in one line]

with `req_sched` the **shipped** rung-47 / rung-52 `required`. Nothing is re-bracketed, nothing
is re-solved, and rungs 46–52's laws are untouched. **When the leg HOLDS, `max(gf, gr) == g_own`
and the hook returns `req_sched` FLOAT-IDENTICALLY** — an explicit branch, not an arithmetic
coincidence, which is rung 48's `_sched_fuel` device (*returns `mf_sched` ITSELF, no solve —
that is what makes the reduce bit-for-bit rather than merely equal*).

### 0.2 It does NOT wind up — and the reason is the law, not the numbers

Measured, both arms, matched clocks, 341 of 341 points, `ic_iters = 1`, `ic_res = 0.0`:

| | `φ` arm | incidence arm |
|---|---|---|
| final `g_fuel` (masked, late) | **0.0** | **0.0** |
| max `g_fuel` (rung 72 / rung 73) | 8.86e−3 / **1.43e−3** | 8.98e−3 / **2.69e−3** |

The masked leg winds **DOWN**, and that is forced: masked means `gr > gf ≈ req_f`, so
`dgf/ds = (req_f − gr)/τ_f < 0`. **A masked integrator referenced to the applied fuel is
self-anti-winding**, because the quantity it integrates is the *actual* exceedance and the
actual exceedance is already being cured by the leg that holds. The advisor's blocker — and the
anti-windup device it would have forced — **does not arise**, and the reason is worth more than
the reprieve: it is why min-select and an applied reference compose at all.

### 0.3 The hand-over MOVES, and so does the plant

| | `φ` arm | incidence arm |
|---|---|---|
| hand-over `s` (rung 72 → rung 73) | 0.205 → **0.235** | 0.245 → **0.300** |
| `max Tt4` (rung 72 → rung 73) | 1283.36 → **1315.22** | 1282.76 → **1353.74** |
| `min φ_lp` (rung 72 → rung 73) | 0.795155 → **0.795155** | 0.791380 → **0.791380** |

So the reference is **decisive in `Tt4` and invisible in `φ`**, and the hand-over is late.
The *sign* is the interesting part and it is derivable: rung 72's masked governor races toward
`req_sched`, the clip the **schedule** would need — i.e. it is given credit for a cut the fuel
leg already made. Rung 73's integrates `req_sched − gf`, the cut still **owed**. The
physically-correct governor is therefore the *slower* one. Whether that means rung 72's ledger
mis-reported its own `Tt4` debit is § 2B's P6, not § 0's.

### 0.4 THE ENTRYWISE `J(73) − J(72)` — measured, not predicted

At the **same** base points (rung 71's device, rung 72 § 4's: one law swapped, nothing else),
14 gains per point, both arms, every point interior and regime-checked, `switch_guard = 4·dg`:

| cell | non-zero entries of `J(73) − J(72)` | all other 14 entries |
|---|---|---|
| `φ`, fuel holds (17 pts) | `rf` and `rr`, **both exactly 20.0 = 1/τ_masked** | **exactly 0.0** |
| `φ`, gov holds (39 pts) | `ff` and `fr`, **both exactly 20.0** | **exactly 0.0** |
| `M_i`, fuel holds (24 pts) | `rf` and `rr`, **both exactly 20.0** | **exactly 0.0** |

The masked leg's row moves from `(−1/τ_m, 0, ·, ·)` to `(0, −1/τ_m, ·, ·)`. **The masked COLUMN
is zero under BOTH references** — which is the whole thing, and § 1 is where it is derived.

### 0.5 Cell coverage, and a THIRD clock arm, disclosed

The applied reference delays the hand-over (§ 0.3), so rung 72's coverage does not transfer:
at matched clocks the **incidence / governor** cell is **EMPTY** (0 points, against rung 72's 1).
Rung 72 § 2.3's WIDE-CELL arm reaches it with only 4. A third arm is therefore added and it is
rung 72's own device pushed one notch — governor twice as fast, valve 1.6× slower:

| arm | `(τ_f, τ_g, τ_q, τ_s)` | `φ` fuel/gov | `M_i` fuel/gov |
|---|---|---|---|
| MATCHED (rung 72's) | (0.05, 0.05, 0.05, 0.05) | 17 / 39 | 24 / **0** |
| WIDE-CELL (rung 72's) | (0.20, 0.01, 0.50, 0.05) | 4 / 29 | 16 / **4** |
| **DEEP-CELL (rung 73's)** | **(0.20, 0.005, 0.80, 0.05)** | — | **7 / 13** |

All four are swept march coordinates, no new physical constant enters, and the RK4 floor is
live: `(0.40, 0.002, 1.00, 0.08)` **trips it** (`ds·Σ(1/τ) = 2.58`). Counts above are at
`ds = 0.005`; § 3 reads at `ds = 0.002`.

---

## 1. THE DERIVATION (paper, after § 0's measurements, before any spectrum)

### 1.1 Rung 72 § 11's premise is RIGHT and its conclusion is WRONG, and one line separates them

Take the governor holding authority (`gr > gf`), states ordered `(gf, gr, q, v)`. Under reading
B the masked fuel leg's law is `F = gf + req_f(q,v) − gr`, so

    F_f = 1  and  F_r = −1      EXACTLY, and F_r ≠ 0 is rung 72 § 11's own prediction, HELD.

But triangularity was never a property of the masked leg's **row**. It is a property of its
**COLUMN**, and `F_r` sits in the **authoritative** one:

    C, V read mf_app = mf_sched − max(gf, gr)          flat in gf
    R reads the scheduled fuel (rung 47, unchanged)     flat in gf
    F reads gf only through its own `+gf`               (§ 1.2)
    ⇒  column_gf(M) = (0, 0, 0, 0)ᵀ    — ZERO, where rung 72 had (−1, 0, 0, 0)ᵀ

**`M` is still block upper-triangular.** The coupling the seam anticipated is real, is exactly
`−1/τ_m`, and **points the wrong way to break anything**: the masked leg is *driven by* the
authoritative one and reaches it through nothing. Rung 62's ONE-WAY, in a fourth shape.

### 1.2 What the reference actually buys: the pole, not the block

    eig(M₄) = { 0 } ∪ eig(M₃)          against rung 72's { −1 } ∪ eig(M₃)
    M₃ = the parent rung's own 3×3 block, ENTRY FOR ENTRY (§ 0.4: 14 entries exactly 0.0)

because the **authoritative** leg's applied reference is the identity (§ 0.1) and `C`, `V`, and
the other fuel law are untouched. So:

* **rung 72's free pole at `−1/τ_masked` moves to EXACTLY the origin.** The masked leg stops
  being a decaying lag and becomes a **pure integrator running open loop** — min-select windup,
  in the spectrum, in its textbook form. Rung 72 saw windup's *lag*; this is windup itself.
* **the zero count rises by exactly one in every cell**: `zeros = n_live − m_live + n_masked`.
* **`det J ≡ 0` in ALL FOUR cells** — including rung 71's, the one cell in the whole family
  where rung 72 found a live determinant (`+5.9e4`). The applied reference **kills it.**

### 1.3 THE POLE IS STILL NOT A GATEABLE MEASUREMENT, AND THE FIX IS TO MOVE THE INSTRUMENT

Rung 72 § 1.2 refuses to gate its free pole because `_jac4` writes `−1/τ_i` on the diagonal *by
construction*. **A pole at the origin sits in exactly the same position** — if `F_f = 1` came
from a diagonal this rung also constructs, reporting `λ = 0` would be the shipped instrument
agreeing with itself for the **fourth** time (rung 67 gate 9, rung 71 § 1.4's `c1`, rung 72
§ 4's matched-clock confound).

So the instrument changes: **`_jac4` no longer constructs the fuel-side diagonal.** It reads a
**measured** `F_f` and `R_r` — two central differences rung 72 never took, because rung 72 never
needed them. `(F_f − 1)/τ_f` reproduces rung 72's `−1/τ_f` when `F_f = 0` is *measured*, so the
change is a strict weakening of the instrument's assumptions and rung 72's readers are
bit-unchanged. **§ 4 gates the zero COUNT and the zero EIGENVECTOR's DIRECTION; the pole
location is reported.**

### 1.4 The three readings move DISJOINT halves of the same matrix

|  | masked row's diagonal | masked row's cross | authoritative row | pole | `M₃` |
|---|---|---|---|---|---|
| rung 72 (sched) | `−1/τ_m` | 0 | rung 72's | `−1/τ_m` | parent's |
| **B (applied)** | **0** | **`−1/τ_m`** | unchanged | **0** | **parent's** |
| C (literal) | `−1/τ_m` | `−1/τ_m` | **`−2/τ_f`** | `−1/τ_m` | **NOT parent's** |

**B moves the pole and keeps the parent; C keeps the pole and moves the parent.** That is what
makes C an isolation instrument with content rather than a strawman, and it is § 3.

---

## 2A. DERIVED, NOT SCORED

* **D1** — the composition of the two devices: under `sum` **and** an applied reference the hook
  never takes its float-identical branch (`gf + gr ≠ g_own`), so both fuel rows gain a cross
  term and the block form goes. That is a fourth plant, and it is **refused**, not measured:
  two declared laws swapped at once is rung 63's lesson in its plainest form.
* **D2** — the RK4 floor needs a **SIXTH** justification and the previous five do not carry: a
  zero eigenvalue is **neutrally stable**, so "the dominant root is below the rate sum" is no
  longer the argument. The new one: the masked leg contributes `λ = 0` **exactly**, which is
  inside every explicit stability region, and the other three share a trace strictly more
  negative than rung 72's — so the inherited constant is **more** conservative here, not less.
* **D3** — `n_live` is still 3 and the `(4, m)` cells stay a mirage. Rung 72 closed that seam by
  refuting its premise; an applied reference does **not** re-open it, because the masked leg
  still reaches the plant through nothing. Rung 69 § 11's fourth-LP-lever route stays the only
  one.
* **D4** — the initial condition is unchanged (`gf = gr = 0`, both legs dormant), so the hook is
  the identity there and rung 72's P9 carries: **1 iteration, residual exactly 0**. (Measured in
  § 0.2 as part of the kill-check.)

---

## 2B. THE PREDICTIONS — genuinely open, and what § 9 scores

**STATED PER CELL, DELIBERATELY.** Rung 72's three misses (P1, P5, D5) had one root cause: it
assumed the governor held authority throughout. Any prediction here that does not name its cell
is repeating that mistake.

| | prediction |
|---|---|
| **P1** | `zeros` = **3 / 2** on the `φ` arm (fuel / gov authority) and **2 / 1** on the incidence arm — rung 72's per-cell counts **each plus one** |
| **P2** | the coefficient identity holds with the pole at the origin: `p₄(λ) = λ·p₃(λ)`, `p₃` from the **shipped** rung-68/69/70/71 `_invariants`, worst gap ≤ 1e−15 in all four cells |
| **P3** | `c0 = 0` and `c1 = −c0(parent)` in all four cells — so **`det J` is dead everywhere, including rung 71's cell**, where rung 72 measured `+5.9e4` |
| **P4** | the zero eigenvector lies **ON the masked leg's own axis**: the null space contains `e_masked` to ≤ 1e−12, and it does so in every cell |
| **P5** | reading **C** moves the AUTHORITATIVE row and **not** the pole — the mirror of B (§ 1.4). Specifically: `C`'s spectrum keeps a root at `−1/τ_masked` and `C`'s `M₃` differs from the parent's, in both cases at every interior point |
| **P6** | **rung 72 UNDER-REPORTED its own `max Tt4` debit by more than 10×.** Its § 5 ledger gives the fuel leg's marginal peak debit as **+0.29 K / +1.86 K**; under the correct reference I predict **> +3 K on the `φ` arm and > +19 K on the incidence arm** (a 10× floor on both), with the `φ`-credit column **unmoved to 4 significant figures** |
| **P7** | `F_f = 1` and `R_r = 1` **exactly** (`== 1.0`, not "≈") at every interior **masked** point, and `== 0.0` exactly at every interior **authoritative** point — the branch indicator of § 0.1, measured |
| **P8** | the five inherited reduce arms stay **bit-for-bit** (rungs 67/68/69/70/71 by dispatch), and a sixth arm — `_ref_law = "sched"` — is **bit-for-bit rung 72** |
| **P9** | the hand-over is **LATE on both arms and at every clock arm** (§ 0.3 measured it on one); no arm hands authority BACK |
| **P10** | the refusals hold: D1's double swap; a `_ref_law` that is neither; rung 72's own five; and the RK4 floor at `(0.40, 0.002, 1.00, 0.08)` |

**AND ONE PREDICTION THIS ANCHOR HAS ALREADY REFUTED WITH PAPER, RECORDED RATHER THAN DROPPED.**
The first design of this rung expected the applied reference to make the plant *four-dimensional*
— rung 72 § 11's own words, "the spectrum would then be genuinely four-dimensional and `n_live`
might reach 4 after all." § 1.1 works out that it cannot: the coupling lands in the wrong column.
That refutation is the rung, and it was on paper before § 0.4 confirmed it.
