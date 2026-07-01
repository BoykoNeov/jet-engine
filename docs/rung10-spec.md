# Rung 10 — The finite-rate quench: the RQL hazard, quantified

Rung 9 burned a **rich** primary and showed EI_NO collapses on the rich flank of the NO-vs-φ
bell — *why* a Rich-burn / Quick-Quench / Lean-burn (RQL) combustor exists. But rung 9's mix-out
was the **ideal, infinitely-fast quench**: NO frozen at the primary value. That is a fiction. In a
real combustor the quench air mixes over a finite time, and while it does the **local mixture
passes through stoichiometric** — the exact peak of the NO bell. So a rich primary's temperature
**rises through the stoich peak on the way down**, and the extended-Zeldovich rate **re-makes NO**
along that path. Rung 10 resolves the quench in **time** and quantifies the hazard: a **slow**
quench dwells at stoich and re-makes the NO the rich primary avoided; only a **fast** quench
escapes past the peak. The "quick" in quick-quench is the whole game.

> **Read `docs/rung9-spec.md` first**, and `docs/plans/rung10-anchor-quench.md` (numbers-before-
> code: the τ_q knob, the linear mixing schedule, the machine-checked worked example — smoking-gun
> T(β), the NO spike vs τ_q, the re-filled rich flank — the reduce-to-rung-9 short-circuit, and
> **the equilibrium-clamp trap**). This file states only what *changes*; the Zeldovich rate
> constants, the two-zone `zoned_nox` scaffold, the `a6`/`a7`/`Kp`/`_equil_solve` substrate, and
> the "derive before you code" / conservation-assert contract all carry over **unchanged**.

---

## What rung 10 adds (and what it deliberately does not)

**Adds:**

- **A finite-time quench trajectory** (`_quench_trajectory`). As dilution air is added **linearly**
  over the quench, the air present at β ∈ [0,1] is `a(β) = α + β(1−α)` mol per mol total air, so
  the **local** fuel/air ratio `far_local = far_ov/a` sweeps `far_p → f_stoich → far_ov`. The
  majors + T are **instantaneous equilibrium** at each β (the rung-8 re-equilibrating `_mixed_out_T`
  on the *current* air), so `T`, `[O]`, `[N2]`, `[H]`, `[NO]_e` are functions of **β alone** — a
  genuine teaching point: the fast chemistry doesn't know how fast the mixing is, so the trajectory
  is **τ_q-independent** and can be built once and swept.
- **A clamp-free NO integrator** (`_quench_no`). The **same** extended-Zeldovich reverse-rate form
  as `_thermal_no`, RK4 over the trajectory, but on **extensive** NO (moles per mol total-final-air:
  mixing conserves it, only chemistry changes it) and **with the `cNO ≤ cNOe` cap dropped** (§ the
  clamp trap). Started from the primary's frozen NO (the rung-9 value).
- **A `τ_q` knob on `zoned_nox`** (`tau_q`, default `None`). `None` is the ideal quench — the exact
  rung-9 path. A finite `tau_q` re-integrates NO through the cooling/mixing trajectory. New
  `ZonedNOxState` outputs: `ei_no_quenched`, `x_no_quenched`, `T_peak`, `max_a_quench`.
- **The RQL-hazard payoff** — for a rich primary, `T_peak > T_primary` (T *rises* through the
  stoich peak), EI_NO rises monotonically with τ_q, and the rung-9 rich-flank collapse is **filled
  back in** to a ~φ_p-independent floor by a finite quench. "A rich primary is low-NOx" is
  **contingent on a fast quench**.
- **`main.py` panel + `NOTES.md` section + `tests/test_rung10.py`.**

**Deliberately does NOT:**

- **Touch the cycle.** NO is still trace and decoupled; the finite quench is opt-in via `tau_q`.
  Every cycle station is **bit-for-bit rung 6** (the whole rung 1–9 suite stays green). The reduce
  is a **short-circuit**, not an empirical limit: `tau_q=None` runs the exact rung-9 code and the
  four new state fields stay `None`.
- **Anchor τ_q or the mixing schedule.** `τ_q` and the **linear** air-addition schedule are
  **knobs**, like `α`, `φ_p`, `τ` — swept to expose the mechanism (the *time at stoich*), not
  fitted to a combustor's jets-in-crossflow mixing field.
- **Add super-equilibrium O / prompt (Fenimore) NO** — still deferred (rung-7 seam), and it matters
  *most* here (rich primary + the radical-rich stoich crossing), so even the finite quench is an
  **equilibrium-O lower bound** on the spike.
- **Model soot** — held **φ_p ≤ 2.0** (carried from rung 9, a hard assert).

---

## The clamp trap (why the quench needs a *separate*, clamp-free integrator)

Rung 7's `_thermal_no` clamps `cNO → cNOe` and asserts `cNO ≤ cNOe`. That is valid when NO builds
toward a **fixed** equilibrium *from below* at one temperature. On a **cooling** trajectory the gas
can drop below the inherited NO's equilibrium, so NO is legitimately **super-equilibrium and
frozen** (Heywood): a hard cap would delete exactly that NO — a plausible-but-wrong **low** number
with every assert still green. So the quench integrator **drops the cap** and relies on the
`(1−a²)` factor (which already goes *negative* when `a = [NO]/[NO]_e > 1` — super-eq NO decomposes)
plus the Arrhenius freeze-out (the rate constants collapse as T falls) to self-limit. Two design
consequences, both verified:

- **`_thermal_no` is untouched — byte-identical.** Its rung-6/7/8/9 reduce gates depend on its
  *exact* capped RK4 trajectory. Rung 10 adds a **separate** clamp-free `_quench_no`.
- **At the lean main.py design point the cap is *dormant*** — a clean surprise, stated plainly.
  Because φ_overall ≈ 0.40 is very lean, the cold mixed-out state is O₂-rich and its equilibrium
  NO stays **high** — above the frozen NO — so NO lags *below* equilibrium the whole way (the
  rung-7 lesson persists through the quench). Measured **`max_a = 0.677` across the entire in-scope
  φ_p×τ_q sweep** (exposed as `max_a_quench`, guarded by a test), and clamp-on ≡ clamp-off to 4 sig
  figs. We drop the cap **anyway** — it is physically wrong for a cooling path, and a
  dormant-but-wrong clamp is exactly the hidden assumption this teaching project exists to expose.
  Super-equilibrium freeze *does* bite in near-stoichiometric exhaust/expansion cooling (the
  still-open rung-6 nozzle seam), not in this lean mixed-out state. The `ei_no_quenched ≥ ei_no`
  ordering seen here is therefore a **property of this lean point (a<1)**, not a general guarantee.

---

## The equations — a trajectory + a clamp-free integrator, no station changes

Every cycle station is **bit-for-bit rung 6**. `zoned_nox` is the rung-8/9 flow through the frozen
primary NO; rung 10 only *continues* past it when `tau_q` is finite:

```
IDEAL quench (tau_q = None):   freeze NO at the primary value        → exact rung 9
FINITE quench (tau_q > 0):     n_NO(0) = α · x_NO,primary · ntot_p    (the rung-9 frozen value)
  trajectory β∈[0,1]:  a(β) = α + β(1−α);  far_local = far_ov/a;  α_local = α/a
                       T(β), pool = _mixed_out_T / _equilibrium_composition  (fast chemistry)
  integrate (RK4, no cap):  d n_NO/dt = 2 R1 (1 − a²)/(1 + β a) · V,   a = (n_NO/V)/[NO]_e
                            R1 = k1f[O][N2],  β = R1/(k2r[NO]_e[O] + k3r[NO]_e[H])
  freeze at β=1;  EI = 1000·n_NO·M_NO/(n_fuel·M_CH2)
```

- **Standing asserts (rung-10 deltas):** the rung-7 **K-check** and **trace guard** now bind at
  **every trajectory T** (T ∈ ~[1518, 2453] K; K-check 1.031–1.043, inside the 0.90–1.15 band) —
  the transcribed rates stay tied to the a6/a7 thermo along the whole cooling path; `tau_q > 0`
  when finite; `n_NO ≥ 0` guard (negatives only — **no** equilibrium cap).

---

## Verification gates (priority order)

1. **Reduce-to-rung-9 (load-bearing, exact by construction).** `tau_q=None` short-circuits to the
   rung-9 path *before* any quench code, and the four new state fields stay `None`. So `ei_no`,
   `x_no_mix`, `T_primary`, `T_mix` are **bit-for-bit rung 9** (hence bit-for-bit rung 6) — the
   whole rung 1–9 suite stays green, untouched. Not an empirical `τ_q → 0` limit (that has an
   ~8 % tiny-denominator artifact at φ_p=1.5 — *do not chase it*; the short-circuit makes it exact).
2. **The smoking gun — T(β) shape.** For a **rich** primary `T_peak > T_primary` (the trajectory
   *rises* through the stoich peak, φ ≈ 1.05, ~2453 K); for **lean/stoich** `T_peak == T_primary`
   (monotone fall). This shape is the load-bearing physical check — a wrong trajectory fails it.
3. **The NO spike vs τ_q (THE lesson).** `ei_no_quenched` rises **monotonically** with τ_q and a
   slow quench re-makes the NO a rich primary avoided (φ_p=1.5: rung-9 ideal ~0.001 → 3 ms quench
   ~3.3 g/kg, three orders of magnitude). The slow-quench penalty and the ideal-quench limit are
   the two ends of one curve.
4. **The finite-quench bell re-fills the rich flank.** The rung-9 ideal collapse (φ_p=1.5 →
   ~0.001) becomes a ~φ_p-independent ~3 g/kg floor under a 3 ms quench — the discriminator that NO
   is re-made *at the stoich crossing*, not carried from the primary.
5. **Clamp dormancy is guarded, not just prose.** `max_a_quench < 1` across the in-scope φ_p×τ_q
   sweep (max 0.677) — so a future operating point that crosses into the super-equilibrium regime
   **flags the regime change** rather than silently passing.
6. **K-check + trace guard hold along the whole trajectory** (asserted at every β, T down to T_mix).
7. **Soot-bound guard** (φ_p ≤ 2.0) carried from rung 9; one explicit φ_p=2.0 finite-τ_q run.

## Conservation asserts (rung-10 deltas)
Carry over rung 6/7/8/9's, plus: the K-check/trace guards now bind at **every trajectory T** (not
just the primary); `tau_q > 0` when finite; the clamp-free integrator guards **negatives only**
(no equilibrium cap) — the physical correctness the whole rung turns on.

## Done when
Reduce-to-rung-9 holds exactly (short-circuit; rungs 1–9 green, untouched; cycle bit-for-bit rung
6); the smoking-gun T(β) rises through the stoich peak for a rich primary and is monotone for
lean/stoich; EI_NO rises with τ_q and re-fills the rich flank; the clamp-dormancy `max_a < 1`
guard binds; the K-check/trace gates hold along the trajectory; the soot-bound guard trips.
`main.py` gains a rung-10 finite-quench panel (τ_q sweep: the T(β) rise, the NO spike, the
re-filled bell); `NOTES.md` gains a rung-10 section (ideal vs finite quench, why "quick" is the
whole game); `CLAUDE.md` scope + deferred-seams updated (finite-rate quench done — with
super-equilibrium O / prompt-NO and the frozen-vs-equilibrium nozzle honestly carved out as the
next seams).

## The rung-11+ seam (keep it additive)
Rung 10 resolves the quench in time but on an **equilibrium-O** pool with a **linear** mixing
schedule. Next seams, all still additive on this substrate: (a) **super-equilibrium O / prompt
(Fenimore) NO** — the richer radical pool in the mixing shear layer, which lifts the spike above
this equilibrium-O lower bound and matters *most* here; (b) the still-open **equilibrium-vs-frozen
nozzle expansion** (rung-6 seam) — where the dropped clamp (§ the clamp trap) actually earns its
keep, as near-stoichiometric exhaust cools and NO freezes super-equilibrium; (c) a **physical
mixing model** (jets-in-crossflow) to replace the τ_q/linear-schedule knobs. Only *where*, *on what
pool*, and *how fast* the chemistry runs changes.
