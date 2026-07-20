# The finite-rate turbine march — INVESTIGATED, **NOT SHIPPED**

> **Status.** This is a **negative-result record**, not a rung spec. It is the attack on rung 29's
> open item (a) — *"a finite-rate turbine march"* (`docs/rung29-spec.md` § Deferred), the seam rungs
> 14/25/26 all named. It was derived, prototyped and tested, returned a negative verdict, and was
> **deliberately not added to the ladder** — no `*-spec.md`, no `gas.py`/`main.py`/test code, no rung-table
> entry, by design. It sits in the `docs/tau-res-negative.md` / `docs/mixing-scale-negative.md` family.
>
> **Why this file exists:** so the negative is not re-investigated from scratch, and so its two positive
> by-products (rung 25's `(R−I)` gap *localized*; rung 29's `S` *sharpened* from "bound" to "attainable")
> are not lost. **If you are about to build a `Da`-swept turbine march, read this first — it was done.**

## What this investigation was

Rung 25 built the finite-rate **nozzle** march: one Damköhler knob `Da` sliding the expansion between
rung-14's frozen (`Da→0`) and shifting-equilibrium bounds. The build **inverted** into a three-state
picture — the `Da→∞` **irreversible-fast ceiling (I)** sits *strictly below* the reversible bound (R),
because the nozzle entry (station 5, frozen-cooled through the turbine) is **super-equilibrium** and a
real flow must relax it **irreversibly even at infinite rate**. That inversion is what made rung 25 a
rung despite an un-anchored rate: the ceiling **I** is a rate-law-**independent closed form**
(`_irreversible_fast_expand`), so its existence and sign are certified while its magnitude is not.

Rung 29 then bracketed the **turbine** the same way rung 14 bracketed the nozzle — frozen (**F**) vs
fully-shifting reversible (**S**), at the shaft-set work `delta_h` — but deferred the finite-rate flow
*between* them, flagging the turbine `τ_res` as un-anchored.

This investigation built that between: **can the rung-25 dodge be repeated at the turbine?** March the
species vector on the work-limited spine, find the `Da→∞` irreversible-fast turbine ceiling `I_turb`,
and see whether it too sits strictly below the reversible bound — a rate-independent finding that would
survive the un-anchored `τ_res`.

**It does not. `I_turb ≡ S` exactly, and the reason is structural.**

## The physics — the inversion of rung 25, and why (one premise, load-bearing)

Rung 25's `(R−I)` gap exists **because the nozzle entry is super-equilibrium**. The turbine entry is
**not**: it is station 4, the fresh burner exit, which `Gas.reacting_equilibrium` solves to be **on the
equilibrium manifold** —

```
  gas.py:4830   comp_entry = _equilibrium_composition(far, Tt4, pt4)     # shifting_turbine
```

An entry already at equilibrium has **nothing to relax irreversibly**. A `Da→∞` march sets the
composition to local equilibrium at every step, so `Σμ_i dn_i = 0` throughout, `T ds = dh − v dp = 0`,
and the process is **reversible** — it lands exactly on rung-29's `S` (which is *defined* by
`S_mix(exit) = S_mix(entry)`). So the irreversible-fast ceiling and the reversible bound **coincide**:

```
  (F)  FROZEN            — Da→0.   rung-29 lower bound.   THE REDUCE.
  (S)  REVERSIBLE-SHIFT  — Da→∞.   rung-29 upper bound.   ← the Da→∞ ceiling lands HERE
  I_turb ≡ S            — no third state. No (R−I) sliver. No inversion.
```

This is **not** a numerical coincidence but a tautology of the construction, and rung 29 already named
it in a different context ("an entry pinned at equilibrium has no super-equilibrium left to relax
irreversibly"). The finite-rate turbine march is therefore a **plain two-state interpolation** `F → S`,
not a three-state picture.

**The premise is load-bearing and was verified**: the entire negative flips if station 4 were handed a
frozen-from-elsewhere mixture. It is not — `comp_entry = _equilibrium_composition(far, Tt4, pt4)` by
construction (asserted directly against the shipped `shifting_turbine`).

## The two un-anchored knobs — why the interior is a negative

The march has **one** new degree of freedom beyond rung-29's endpoints: *where in `[F, S]` the flow
lands.* That single quantity rides on **two** un-anchored inputs, one worse than the nozzle case:

1. **`τ_res` is un-anchored** (the knob rung 29 already flagged). `Da_turb = τ_res/τ_chem(4)` with the
   rung-26 anchored clock giving `τ_chem(4) = 9.1e-4 s` (Tt4=1500) to 5.7e-5 s (2400), against a turbine
   passage residence guessed at `5e-5 … 5e-4 s` ⇒ `Da_turb` between **0.05 and 8.8** — transitional, and
   swinging a full order across defensible guesses. Anchoring it needs blade-row geometry + the same
   choked-flow seam `docs/tau-res-negative.md` named. That search does **not** converge (confirmed there).

2. **The parametrization itself is ambiguous** (NEW — the turbine is *worse* than the nozzle here). The
   nozzle march has a *known* pressure schedule `pt9 → p9`, a natural progress coordinate. The turbine is
   **work-limited**: `p5` is an *unknown*, solved by the work target, so there is **no natural coordinate
   to march on**. The interior `[F,S]` curve's shape depends on the arbitrary schedule chosen (the probe
   marches a geometric `p(s)=pt4·(p_floor/pt4)^s` and normalizes `Da` to it — a different `p_floor`
   reshapes the interior). This is a **second** un-anchored freedom *stacked on* `τ_res`, and it is the
   cleaner argument than `τ_res` alone: even given a perfect rate, the curve is not defined.

The **endpoints** are schedule-independent (`Da→0` freezes the composition ⇒ `F`; `Da→∞` pins it to
equilibrium ⇒ `S`), which is why the negative is sharp: *only* the endpoints are certifiable, and they
are exactly rung 29. Everything the march would newly add is un-anchored on two axes.

## The deciding computation

Design point as rung 29 (`π_c=10`, `M0=0.85`, real losses), `Gas.reacting_equilibrium`. The march
(geometric-`p` schedule, work-target stop, implicit `dh=v·dp`) reduces to **F** at `Da→0` and reaches
**S** at `Da→∞`, in *state* — confirming `I_turb ≡ S`:

| `Tt4` | rung-29 `T5_F` | rung-29 `T5_S` | `ΔT5/T5` | march `Da→0` err vs F | march `Da→∞` err vs S |
|------|-----------|-----------|----------|-----------|-----------|
| 1500 | 1262.691 | 1262.826 | +0.0107% | +0.0000 K | −0.0001 K |
| 1800 | 1576.200 | 1577.406 | +0.0765% | +0.0001 K | −0.0025 K |
| 2100 | 1888.035 | 1894.982 | +0.3680% | +0.0007 K | −0.0211 K |
| 2400 | 2200.058 | 2240.962 | +1.8592% | +0.0037 K | −0.1864 K |

(`Da→∞` err vs S is O(1/nstep) truncation: at `Tt4=2400` it falls −0.186 → −0.061 → −0.030 → −0.024 K
as nstep goes 1200 → 4800 → 19200 → 50000. The march reaches S; it does not merely approach a distinct
ceiling.)

### The entropy contrast — the one non-tautological result, from EXACT closed forms

The signature that distinguishes "no inversion" from "a hidden ceiling" is the entropy of the `Da→∞`
limit. Read off the **shipped, exact closed forms** (not the hand-march, whose reported `dS` carries an
O(1)-persistent composition-lag artifact — see § reproduction):

| `Tt4` | `dS(I_turb)=dS(S)` turbine | `dS(I)` nozzle (rung-25 closed form) |
|------|-----------|-----------|
| 1500 | +1.2e-12  (machine zero) | +4.33e-4 |
| 1800 | −1.3e-12  (machine zero) | +2.44e-3 |
| 2100 | +1.4e-12  (machine zero) | +9.79e-3 |
| 2400 | +4.0e-13  (machine zero) | +4.14e-2 |

*(J·mol⁻¹dry-air·K⁻¹.)* The turbine's `Da→∞` ceiling is **isentropic at every `Tt4`** (rung-29's `S`
solves `dS=0` by construction — measured 4e-13). The nozzle's ceiling carries **physical** entropy that
**grows with `Tt4`** (rung 25's dormant-lean → earns-hot arc). Eight to eleven orders of magnitude apart:
**same machinery, opposite entry.** The nozzle's irreversibility is **manufactured by the freeze** (which
delivers the super-equilibrium entry); the turbine, entering at equilibrium, has none.

## The design-point-robust corollary — why the un-anchored knobs don't bite at design

Because `F ≈ S` at the design point (`ΔT5/T5 = 0.011%` at `Tt4=1500`), the **entire** `[F,S]` interior is
trapped in a band an order below the cycle's own `η_t/π_b` precision — **regardless of `Da`, `τ_res`, or
the schedule choice.** So the finite-rate march, whatever its two un-anchored knobs do, **cannot overturn
rung 29's "freeze earned at design."** The knobs are un-anchorable *and irrelevant* at design; they would
matter only hot, where the answer is genuinely un-pinnable (the interior of a 1.9% band on two guessed
axes). That is the rate-independent statement worth carrying — and it is rung 29's, not a new one.

## The positive by-products — rungs 25 and 29 sharpened

- **Rung 25's `(R−I)` gap is manufactured by the freeze, not intrinsic to expansion.** The turbine
  (equilibrium entry) has **no** entry-irreversibility; the nozzle has it *only because* the frozen
  turbine delivers a super-equilibrium entry. This **localizes** rung 25's central finding: the
  "sliver of entry irreversibility" is a property of *the freeze*, produced at the turbine→nozzle handoff,
  and there is exactly one such handoff in the frozen model. Upstream of it (in the turbine) there is
  nothing to relax; downstream (in the nozzle) there is.
- **Rung 29's `S` is *attainable*, not an unreachable ceiling.** Unlike rung 25's `R` (which no real
  nozzle reaches, the `(R−I)` gap always open), the turbine's reversible bound `S` **is** the genuine
  `Da→∞` limit. A one-line sharpening of rung 29: its upper bracket is a limit a fast-enough turbine
  would actually hit, not a bound it falls short of.

## What this means for the seam (the corrected takeaway)

Rung 29's open item (a) asked for a finite-rate turbine march. It **gets no rung**, for a *structural*
reason (not merely the un-anchored `τ_res` rung 29 cited):

- **No rate-independent finding to anchor it.** The rung-25 dodge (a closed-form `Da→∞` ceiling strictly
  inside the bracket) **cannot be repeated** — the turbine's `Da→∞` ceiling *is* the bracket's upper
  bound, because the entry is at equilibrium. There is no third state to certify.
- **Two un-anchored knobs, not one.** The interior landing rides on `τ_res` **and** an ambiguous
  progress-coordinate; the endpoints are already rung 29. A `Da`-swept turbine march would report an
  interior curve that is a cartoon on two axes — precisely the "`τ_res`-style negative" `docs/rung29-spec.md`
  predicted.

**Do NOT** re-run: the geometric-`p` work-limited march with a normalized scalar `Da` and a hand-picked
`p_floor`. That is exactly this investigation. **A genuine attempt is only worthwhile** if it brings a
**real turbine passage geometry** — a resolved `A(x)` through the blade rows with a physical residence
`τ_res(x)` (which drags in the choked-flow seam `docs/tau-res-negative.md` named) — i.e. the thing that
would anchor *both* knobs at once. Absent that, the two certifiable endpoints are rung 29's `F` and `S`,
and the honest statement is the design-point-robust corollary above.

(Precedent: rung 18's "a 0-D transport **cannot derive** `C_opt`", and `docs/tau-res-negative.md`'s "a
physical residence cannot be derived from the pressure schedule alone" — shipped negatives of the same
shape. The seam over-promised a rung; this is the honest ceiling.)

## Reproduction

Probes lived in `M:\claud_projects\temp\rung29-turbine-march\` (`probe_turbine_march.py` — the work-limited
finite-rate march, the `F`/`S` endpoint table; `probe_convergence.py` — the `dS(Da→∞)` grid study;
`FINDINGS.md`). That folder is **outside git** (project temp policy), so this tracked file is the durable
record. Every number here came from the **shipped** API (`_work_limited_expand`, `_irreversible_fast_expand`,
`_finite_rate_expand`, `_mix_entropy_molar`) read through the existing seams — **nothing in the shipped code
was changed.**

One honest caveat on the probe (kept, not buried): the hand-rolled turbine march reports a spurious
`dS(Da→∞)` floor `~1.1e-2` that does **not** vanish under grid refinement — a bookkeeping artifact of the
persistent one-step composition lag (the marched endpoint composition is never fully re-equilibrated at
`(T5,p5)` before entropy is read). It is **not** physics: rung-29's exact `S` is isentropic to 4e-13
(measured directly), and the march reaches `S`'s *state*. The entropy contrast in this doc therefore uses
the **exact closed forms**, never the march's `dS` — which is why it is machine-zero, not `1.1e-2`.
