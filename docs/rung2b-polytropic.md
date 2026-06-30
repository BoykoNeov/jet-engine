# Rung 2b — Polytropic Efficiency as a First-Class Knob

Rung 2 made the compressor and turbine real with an **isentropic** (adiabatic)
efficiency `η_c, η_t`. Rung 2b adds the *other* efficiency knob the canon uses —
**polytropic** (small-stage) efficiency `e_c, e_t` — as a first-class input,
side by side with the isentropic one. It is a *small, contained* sub-rung: one
extra parameter on two existing components, no solver change, the reduce-to-ideal
gate untouched. Read `docs/rung2-spec.md` first; this only states what changes.

> **Why a separate knob and not just a conversion?** The two efficiencies are
> related by an exact, closed-form conversion (rung 2 used it inside the anchor
> test). We could keep converting `e → η` at the door and reuse the isentropic
> code verbatim — that is *feasible*. We make `e` first-class anyway because the
> polytropic relation `Tt/Tt = (pt/pt)^(g/e)` is the *per-stage definition* and
> belongs **in the forward path**, where it is the thing being taught — not
> hidden behind a one-shot conversion. (And for the turbine it removes the
> rung-2 anchor's "provisional pass to recover `τt`" dance; see below.)

---

## The two efficiency *definitions* — same machine, different question

Both `η` and `e` answer "how far from reversible is this real machine?" but they
ask it at different scales:

- **Isentropic** `η` compares the *whole* component against a single ideal
  endpoint at the *same overall pressure ratio*. It depends on that pressure
  ratio.
- **Polytropic** `e` is the isentropic efficiency of each *infinitesimal* stage,
  taken in the limit and the same for every stage. It is a property of the
  *blading/technology*, independent of how many stages (how much pressure ratio)
  you stack up.

For a calorically perfect gas both integrate to closed forms, and each converts
exactly to the other. The point of this sub-rung is to carry `e` natively and let
`η` fall out as a *diagnostic*, not the reverse.

---

## Derivation (derive before code)

`g` is the section exponent `(γ−1)/γ`: `gc` cold (compressor), `gt` hot (turbine).

### Compressor (polytropic, native)
Each infinitesimal stage is isentropic-efficient `e_c`: the actual temperature
rise for a pressure rise `dpt` is the ideal rise `Tt·gc·dpt/pt` divided by `e_c`:
```
dTt/Tt = (1/e_c)·gc·(dpt/pt)
```
Integrate 2→3 (constant `e_c`, calorically perfect gas):
```
Tt3 = Tt2 · πc^(gc/e_c)          # actual exit temperature, DIRECTLY
```
Compare the rung-2 isentropic form `Tt3 = Tt2 + (Tt3s − Tt2)/η_c`. The polytropic
form needs **no substate** to get `Tt3` — the exponent `gc/e_c` already carries
the loss. The ideal substate `Tt3s = Tt2·πc^gc` is still computed, but now as a
*diagnostic* (it feeds the same rung-2 asserts).

*Implied isentropic efficiency (derived, for the cross-check):*
```
η_c = (Tt3s − Tt2)/(Tt3 − Tt2) = (πc^gc − 1) / (πc^(gc/e_c) − 1)
```
At `πc=10, gc=0.2857, e_c=0.9`: `η_c = (1.9307−1)/(2.0771−1) = 0.8641`.

### Turbine (polytropic, native)
The shaft still sets the work, i.e. the drop `ΔTt` (engine-owned, dual-cp + η_m,
*independent of turbine efficiency*). So `Tt5 = Tt4 − ΔTt` is known before any
efficiency enters — and with `Tt5` in hand the polytropic relation gives `pt5`
directly. Each stage delivers a fraction `e_t` of its ideal work:
```
dTt/Tt = e_t·gt·(dpt/pt)   →   Tt5/Tt4 = (pt5/pt4)^(e_t·gt)
```
Invert for the pressure (this is the forward step):
```
τt  = Tt5/Tt4
pt5 = pt4 · τt^(1/(e_t·gt))      # actual exit pressure, DIRECTLY
```
The ideal substate `Tt5s = Tt4·(pt5/pt4)^gt = Tt4·τt^(1/e_t)` is again a
diagnostic feeding the rung-2 asserts.

*Implied isentropic efficiency (derived, for the cross-check):*
```
η_t = (Tt4 − Tt5)/(Tt4 − Tt5s) = (1 − τt) / (1 − τt^(1/e_t))
```
At `τt=0.8155, e_t=0.9`: `η_t = (1−0.8155)/(1−0.7972) = 0.9099`, with `π_t =
τt^(1/(e_t·gt)) = 0.3746`.

> **Contrast with the rung-2 anchor.** Under the isentropic knob, converting the
> book's `e_t` needed `τt`, which the *shaft* fixes — so the rung-2 anchor ran a
> *provisional pass* (`η_t=1`) just to read `τt`, then converted, then ran for
> real. With `e_t` native there is no conversion and no provisional pass: `τt`
> appears naturally at apply time and `pt5` follows in one line. That removal is
> the concrete payoff of making polytropic first-class — especially for turbines.

---

## The asymmetry — `η_c < e_c < e_t < η_t` — *this is the lesson*

Feed the **same** polytropic efficiency to both machines (`e_c = e_t = 0.9`) and
the implied isentropic efficiencies split apart and straddle it:

```
η_c = 0.8641   <   e_c = 0.90   =   e_t = 0.90   <   η_t = 0.9099
```

The compressor's isentropic efficiency falls **below** its polytropic one; the
turbine's rises **above**. Same hardware quality, opposite-signed gap. Why:

- The constant-pressure lines on a `T–s` diagram **diverge** as temperature rises
  (`∂T/∂s|_p = T/cp`). In a compressor, each stage's lost work reheats the gas, so
  the *next* stage starts hotter and its isobars are farther apart — the stage
  ideal works sum to **more** than the single overall-ratio ideal work. The
  machine looks **worse** as a whole: `η_c < e_c` (the *preheat* penalty).
- In a turbine the same reheat is partly **recovered**: friction heat dumped in an
  early stage is still hot gas the *later* stages expand and extract work from. The
  stage ideal works sum to **less** than the overall ideal, so the machine looks
  **better** than any single stage: `η_t > e_t` (the *reheat* benefit).

Both gaps **grow with pressure ratio** (more stages, more reheat to accumulate)
and **vanish as π→1** (`η → e` in the single-stage limit). That pressure-ratio
dependence is exactly why `e` exists as a distinct knob: quote a compressor's
`η_c` and you must also quote the `πc` it was measured at; quote `e_c` and it
travels. (Verified directionally in `tests/test_polytropic.py`.)

---

## API — one parameter, mutually exclusive with `η`

`Compressor(pi_c, eta_c=1.0, e_c=None)` and `Turbine(eta_t=1.0, e_t=None)`:

- Give **neither** → ideal (`η=1`), exactly rung 1.
- Give **`η`** → the rung-2 isentropic path, unchanged.
- Give **`e`** → the rung-2b polytropic path.
- Give a non-default **`η` and an `e`** together → **error** (contradictory; the
  two knobs are alternatives, not composable). Only this needs a new guard.

`e > 1` needs *no* new guard: it implies `Tt3 < Tt3s` (or `Tt5 < Tt5s`), which the
rung-2 entropy-generation asserts (`Tt3 ≥ Tt3s`, `Tt5 ≥ Tt5s`) already reject.

`build_turbojet(..., e_c=None, e_t=None)` threads the two through.

---

## Verification gates (in priority order)

1. **Reduce-to-ideal.** `e_c = e_t = 1` → `πc^(gc/1)=πc^gc` and
   `τt^(1/(1·gt))`, i.e. the isentropic `η=1` expressions exactly → the rung-1
   table to the digit. The gate is structurally untouched.
2. **Polytropic ⇄ isentropic equivalence (the strongest gate).** A component at
   `e_c=0.9` and one at the converted `η_c = (πc^gc−1)/(πc^(gc/e_c)−1)` are
   **algebraically identical**, not merely close — both land on `Tt3 =
   Tt2·πc^(gc/e_c)`, and `ΔTt` is independent of turbine efficiency so `pt5`
   matches too. Assert the full station states agree to **~1e-9 relative**, not at
   the anchor's 1e-3.
3. **Implied-η cross-check (free, every run).** In polytropic mode the component
   computes its implied `η` from the realized states and asserts it equals the
   closed-form conversion. This validates the conversion formula continuously, not
   just in a test.
4. **Polytropic-native external anchor — Mattingly Example 7.1.** Re-run the rung-2
   anchor feeding `e_c = e_t = 0.9` **directly** (no conversion, no provisional
   pass). It reproduces the book *and* matches the rung-2 isentropic anchor to
   machine precision. Keep **both** anchors: the side-by-side is the payoff.
5. **Asymmetry / directional.** `η_c < e_c < e_t < η_t` at the anchor; both gaps
   shrink toward 0 as `πc → 1`.

---

## Scope — what this sub-rung is and is not

- **Is:** one extra parameter on `Compressor` and `Turbine`, the native relation in
  the forward path, the implied-η diagnostic + cross-check, the mutual-exclusivity
  guard. No subclasses (a `PolytropicCompressor` would duplicate the substate and
  the asserts).
- **Is not:** variable `cp(T)`, off-design, the choked nozzle, afterburner — all
  still deferred, seams kept (see `CLAUDE.md`). The `T–s` diagram in `main.py` is
  left alone: the `η/e` story is a *table/number* point, not a leg-tilt one.
