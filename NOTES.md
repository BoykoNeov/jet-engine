# NOTES — Turbojet Cycle, in plain language

A running, plain-language companion to the code: what each station does and *why*,
written to learn from. Built up station by station as the cycle is derived. See
`SPEC.md` for the formal equations and `CLAUDE.md` for the working contract.

## The big picture
A turbojet is a **Brayton cycle**: squeeze air (compressor), burn fuel in it
(burner), let it expand through a turbine, and shove what's left out a nozzle
fast enough to make thrust. We track the gas as a **total (stagnation) state** —
`Tt`, `pt` — at six numbered stations (0, 2, 3, 4, 5, 9).

"Total" means *"what the gas would read if you brought it gently (isentropically)
to rest."* That number already includes the energy tied up in the flow's motion,
so we can reason about the whole cycle without carrying velocities around — right
up until the nozzle, where velocity is the entire point and we finally convert
back to **static** (`T`, `p`, `V`).

## Station 0 — Freestream (the air before the engine touches it) ✅
**What it is:** the ambient air the aircraft flies into — its static temperature
and pressure (`T0`, `p0`) and the flight Mach number `M0`.

**What we compute:** the *total* temperature and pressure of that air, plus the
flight speed `V0`.

- `Tt0 = T0 · (1 + (γ-1)/2 · M0²)`
- `pt0 = p0 · (1 + (γ-1)/2 · M0²)^(γ/(γ-1))`
- `V0  = M0 · √(γ R T0)`

**Why (the physics):** stand on the engine. The air rushes at you at `V0` and you
bring it to rest. That kinetic energy can't vanish — it reappears as a rise in
temperature and pressure. This is the **ram effect**. `Tt0`/`pt0` are exactly the
state the air reaches once stopped. The shared factor `(1 + (γ-1)/2 · M0²)` is the
bookkeeping for *"how much hotter and denser does stopping the flow make it."*

**Why it matters downstream:** because the totals already bank the ram energy, the
*ideal inlet* (station 2) has nothing left to do but pass them through unchanged —
which is exactly what we'll see next. And `V0` is the speed the exhaust must beat:
thrust is the engine throwing mass out the back *faster than* `V0`.

**Numbers (validation case):** `T0`=250 K, `p0`=50 kPa, `M0`=0.85 →
`Tt0`=286.1 K, `pt0`=80.19 kPa, `V0`=269.4 m/s. ✓ matches the spec table.

---

## Station 2 — Inlet (the air reaches the compressor face) ✅
**What it is:** the duct between the freestream and the first compressor blade.

**What we compute:** nothing new — `Tt2 = Tt0`, `pt2 = pt0`. The totals just pass
through.

**Why (the physics):** there are two separate reasons the totals don't change, and
only one of them is an idealization. `Tt` stays put for *any* inlet: a plain duct
adds no heat and spins no shaft, and total temperature is exactly the thing that's
conserved when you do neither. `pt` staying put is the *ideal* part — a real inlet
scrubs off a little total pressure to friction and shocks; we set that loss to
zero. The ram compression that heated and squeezed the air already happened back at
station 0 when we stopped the flow, so the ideal inlet has nothing left to do but
hand the bill to the compressor.

**Why it matters downstream:** this is the station that *justifies* working in
totals. Because the inlet is a no-op, the interesting machinery (compressor,
burner, turbine, nozzle) starts from the same numbers the sky handed us.

**Numbers:** `Tt2`=286.1 K, `pt2`=80.19 kPa — identical to station 0. ✓

---

## Station 3 — Compressor (squeeze the air) ✅
**What it is:** the spinning blades that raise the pressure. Our design knob is the
**pressure ratio** `πc = pt3/pt2 = 10`.

**What we compute:** `pt3 = πc · pt2`, and the temperature that *comes with it*,
`Tt3 = Tt2 · πc^g` (with `g = (γ-1)/γ = 0.2857`).

**Why (the physics):** the compressor pours shaft work into the air but trades no
heat with the outside (adiabatic), and in rung 1 it does so perfectly (reversible)
— so the process is **isentropic**, entropy unchanged. For our constant-property
gas, holding entropy fixed welds temperature to pressure: you cannot raise one
without the other rising by a set amount. So we *choose* the pressure ratio, but
the temperature rise is **not free** — isentropy forces it. That temperature rise,
`Tt3 - Tt2`, is the energy the compressor demanded, and it's the exact debt the
turbine will have to repay later.

**Why it matters downstream:** `πc` is the cycle's main efficiency lever. The
thermal efficiency is `1 - 1/πc^g` — the harder you squeeze, the more of the burnt
fuel's heat the cycle can convert to work. (This is the **primary hand-check**:
`1 - Tt2/Tt3` must equal `1 - 1/πc^g`. Both give 0.4821.)

**Numbers:** `pt3`=801.9 kPa, `Tt3`=552.4 K. ✓

---

## Station 4 — Burner (add the heat) ✅
**What it is:** where fuel is sprayed in and burned. The design knob is the
**turbine-inlet temperature** `Tt4 = 1500 K` — the hottest the turbine blades can
survive, and the peak of the whole cycle.

**What we compute:** `pt4 = pt3` (ideal: no pressure lost to the flame), and the
**fuel-air ratio** `f = cp(Tt4 - Tt3) / (hPR - cp·Tt4)` from an energy balance.

**Why (the physics):** all the fuel's chemical energy (`hPR` per kg) goes into
heating the gas from `Tt3` up to `Tt4`. Rearranging that bookkeeping gives `f`. The
telling part is the denominator, `hPR - cp·Tt4`: fuel gets *expensive* as `Tt4`
climbs toward the adiabatic flame ceiling, because each new drop of fuel has to heat
*its own mass* up to `Tt4` too. Note this leg is the one place entropy genuinely
rises — adding heat must raise entropy — which is why the burner is the only
component with *no* isentropic check on it.

**Why it matters downstream:** two things leave the burner that the rest of the
cycle leans on. First, the heat: `Tt4` is the energy reservoir everything after this
draws from. Second, the **extra mass** — the stream is now `(1+f)` kg for every 1 kg
of air, and that `(1+f)` shows up in both the shaft balance and the thrust.

**Numbers:** `f`=0.02304, `pt4`=801.9 kPa, `Tt4`=1500 K. ✓

---

## Station 5 — Turbine (pay the compressor back) ✅
**What it is:** blades on the *same shaft* as the compressor, pulling energy back
out of the hot gas. Its job is fixed by that shaft — see the next section.

**What we compute:** the temperature drop the shaft demands,
`Tt5 = Tt4 - ΔTt` with `ΔTt = (Tt3 - Tt2)/(1+f)`, then the pressure that expansion
costs, `pt5 = pt4 · (Tt5/Tt4)^(1/g)`.

**Why (the physics):** the turbine, like the compressor, is adiabatic and (in rung
1) reversible — isentropic — so as it gives up energy, pressure falls in lockstep
with temperature. It's the compression run backwards. But *how much* it drops isn't
the turbine's choice; it's whatever the compressor needs. That's the shaft balance,
below.

**Numbers:** `Tt5`=1239.7 K, `pt5`=411.5 kPa. ✓

---

## The shaft balance — what it buys us 🔑
This is the one line that makes a turbojet a **machine** instead of two unrelated
processes. Compressor and turbine are bolted to the same shaft, so every watt the
turbine pulls from the gas is spent driving the compressor — no more, no less:

```
mdot_air · cp · (Tt3 - Tt2)  =  (mdot_air + mdot_fuel) · cp · (Tt4 - Tt5)
```

Divide through and that's `Tt4 - Tt5 = (Tt3 - Tt2)/(1+f)`. (The `(1+f)` divides in
because the turbine works the *heavier* burnt stream, so it needs a slightly smaller
temperature drop to make the compressor's power.)

**Now the payoff — and it's the whole point of the document.** The turbine only has
to repay the compressor's *work*, which is a **temperature** rise (`Tt3 - Tt2` ≈ 266
K). But look at *where* the burner did its job: it dumped a huge slug of heat in
**between** the compressor and the turbine, at the cycle's peak pressure. So when the
turbine then gives back just enough to settle the shaft, it expands a far hotter gas
than the compressor ever compressed — and a given temperature drop on a hotter gas
costs **less pressure ratio** than the same drop bought on the way up.

The result: the turbine hands the compressor its energy back and the gas *still*
walks out at `pt5` = 411 kPa, more than 8× the ambient 50 kPa. **That leftover
pressure ratio is the entire thrust.** The nozzle exists to cash it in. If the
burner hadn't fired — if you tried to drive the compressor and turbine on cold air —
the turbine would have to give back the same *temperature*, but on cold gas that
eats *all* the pressure the compressor made, and you'd leave the nozzle with nothing.

So the shaft balance buys us this: **heat added at high pressure lets the turbine
satisfy the shaft and still leave a fat pressure ratio for the jet.** That's why a
turbojet makes thrust, stated in one sentence.

**Numbers:** `ΔTt` = (552.4 − 286.1)/1.02304 = 260.3 K → `Tt5` = 1500 − 260.3 =
1239.7 K. The code re-derives the turbine and compressor powers two independent ways
and asserts they match — that's the shaft *closing*, checked on every run. ✓

---

## Station 9 — Nozzle (cash the pressure in for speed) ✅
**What it is:** the converging duct at the back. This is where we finally leave
totals behind and convert to **static** — because thrust needs an actual *velocity*.

**What we compute:** totals pass through (`Tt9 = Tt5`, `pt9 = pt5`), then we expand
to ambient (`p9 = p0`, "fully expanded") and read off the exit Mach, static
temperature, and velocity:
`M9 = √( ((pt9/p9)^g − 1)/((γ−1)/2) )`, `T9 = Tt9/(1 + (γ−1)/2·M9²)`,
`V9 = M9·√(γ R T9)`.

**Why (the physics):** an ideal nozzle adds no heat and does no work, so it can't
create energy — it only **trades pressure for velocity**. The gas keeps expanding
until its static pressure matches the sky, spending every bit of that 8× pressure
ratio the turbine left behind. As pressure falls the flow accelerates and *cools*:
the thermal energy becomes directed kinetic energy. `V9` is built on the *static*
(cooled) speed of sound, not the total — the jet is genuinely moving, not stopped.

**Why it matters downstream:** `V9` is the number thrust is made of. The engine
took in air at `V0` = 269 m/s and throws it out at `V9` = 1062 m/s; the difference
is what pushes the aircraft forward.

**Numbers:** `M9`=2.033, `T9`=678.8 K, `V9`=1061.5 m/s. (The spec table lists
1061.6; we match to ~0.1%. The tiny gap is a known rounded-constant artifact —
`cp`=1004.0 vs `γR/(γ−1)`=1004.5 — not a physics bug.) ✓

---

## What the cycle delivers — performance in plain words
With every station solved, the top-level numbers fall out:

- **Specific thrust** `(1+f)·V9 − V0` = 816.6 N·s/kg — newtons of thrust per kg/s of
  air. The `(1+f)` is there because we throw out slightly more mass than we took in
  (the fuel rode along). The pressure term is zero *only because* the nozzle is fully
  expanded.
- **TSFC** `f / (F/mdot)` = 2.821e-5 kg/(N·s) ≈ 101.6 kg/(h·kN) — fuel burned per
  unit thrust. Lower is thriftier.
- **Thermal efficiency** `1 − Tt2/Tt3` = 0.4821 — the share of the burnt fuel's heat
  the cycle turns into useful work. Set entirely by `πc` (the primary hand-check).
- **Propulsive efficiency** `η_p` = 0.4073 — how well the jet's kinetic energy becomes
  forward thrust rather than wasted wake. A fast, thin jet (turbojet) is less
  propulsively efficient than a slow, fat one (turbofan) — a preview of why fans win.
- **Overall efficiency** `η_o` = 0.2231 — thrust power per unit fuel power, the
  bottom line.

**One trap to pre-empt:** `η_o` ≠ `η_th × η_p` here (0.2231 ≠ 0.4821 × 0.4073). That
textbook cascade only holds when `η_th` is the *propulsion* thermal efficiency
(KE added to the jet ÷ fuel power ≈ 0.5477), whereas the `η_th` we report is the
ideal-Brayton **cycle** efficiency `1 − Tt2/Tt3`. They're two different quantities;
ram compression and open-cycle heat accounting split the conventions apart. Don't
multiply the reported `η_th` and `η_p` and expect `η_o`.

---
*All six stations and the shaft balance are derived. The companion artifacts are the
station table (`python main.py`) and the T–s diagram (`ts_diagram.png`), where these
same numbers are drawn as a closed Brayton loop.*

---
---

# Rung 2 — Real Components, in plain language

Rung 1 was the *ideal* engine: every squeeze and expansion perfect, no pressure
lost anywhere, one gas throughout. Rung 2 makes each component **real** — it now
*generates entropy* — and, as a package deal, splits the gas into a cold half and
a hot half. The formal derivations are in `docs/rung2-spec.md`; this is the
plain-language version. The one-picture summary: on the T–s diagram the
"isentropic" work legs stop being vertical and **lean to the right**.

## The two ways a component can be "real"
There are exactly two new effects, and keeping them apart is the whole game:

1. **Isentropic efficiency** (`η_c` compressor, `η_t` turbine). The real machine
   reaches the *same pressure* as the ideal one, but at a *worse temperature*. A
   real compressor wastes some work as extra heat, so it ends up **hotter** than
   the ideal `Tt3`; a real turbine gets **less pressure drop** for the same work.
   We measure this against an *ideal substate* (`Tt3s`, `Tt5s`) — the
   rung-1 isentropic answer — and `η` is how far the real machine falls short.

2. **A specified pressure-ratio loss** (`π_d` inlet, `π_b` burner, `π_n` nozzle).
   A flat fractional total-pressure drop you simply *state* as an input, exactly
   like the compressor's `πc`. No temperature coupling — friction and shock loss
   bleeding off `pt`.

Set all of them to 1 and rung 2 collapses *exactly* back to rung 1 — that's the
**reduce-to-ideal check**, the rung-2 analog of the primary hand-check.

## Why "real gas" rides in with "real components"
Once you stop idealizing the components, the textbooks stop idealizing the gas
too — and for a good reason. Hot combustion products are heavier and floppier than
cold air: their `cp` is higher (≈1239 vs 1004 J/kg·K) and `γ` lower (≈1.3 vs 1.4).
So rung 2 carries **two gases**: a *cold* one upstream of the burner (stations
0→3) and a *hot* one downstream (4→9). The burner is the hand-off. (One rung short
of the full truth, where `cp` varies continuously with temperature.)

## What changes in the shaft balance
Same idea as rung 1 — turbine repays the compressor — but now with two `cp`s and a
**mechanical efficiency** `η_m` for the friction in the shaft itself:

```
η_m · (1 + f) · cpt · (Tt4 − Tt5)  =  cpc · (Tt3 − Tt2)
```

The turbine works the hot, heavy stream (`cpt`, `1+f`) and loses a sliver to
friction (`η_m`), so the temperature drop it needs shifts accordingly. The engine
still solves this out in the open and asserts it closes on every run.

## The thermal-efficiency fix — and the trap from rung 1, resolved
Remember the trap at the end of the rung-1 notes: `η_o ≠ η_th × η_p`, because the
rung-1 "thermal efficiency" was the Brayton identity `1 − Tt2/Tt3`, not the real
thing. **Rung 2 fixes this.** We now report *two* numbers:

- `eta_brayton = 1 − Tt2/Tt3` — kept only because it's the rung-1 table value and
  the hand-check. **Watch it mislead you:** with a leaky compressor the air comes
  out *hotter* (`Tt3` rises), so `1 − Tt2/Tt3` goes *up* (0.482 → 0.514 in
  `main.py`) — as if losses *helped*. They didn't. This number moves the wrong way.
- `eta_thermal = [(1+f)V9² − V0²] / (2·f·hPR)` — the **real** thermal efficiency,
  the kinetic energy actually added to the jet per unit fuel burned. It correctly
  *drops* with losses (0.548 → 0.481), and in the ideal limit it's 0.5477 (not
  0.4821 — genuinely a different quantity).

The payoff: under this honest definition the textbook cascade **`η_o = η_thermal ·
η_p` now holds exactly** — the very identity rung 1 had to apologize for. The code
asserts it on every run.

## The under-expanded nozzle (a small extra)
Rung 1's nozzle always expanded fully to ambient (`p9 = p0`). Rung 2 lets you
*specify* the exit pressure. When `p9 ≠ p0` (the jet leaves still pressurized) the
thrust picks up a **pressure term** — `(1+f)·Rt·T9·(1 − p0/p9)/V9` — on top of the
momentum term. It vanishes when `p9 = p0`, so the default is still the rung-1 case.
(Detecting a *choked* nozzle and solving for `p9` is a later rung; here `p9` is
just an input.)

## Did we get it right? The external anchor
Rung 1 checked itself against a spec table. Rung 2 checks against a real textbook:
**Mattingly's *Elements of Propulsion*, Example 7.1** — a Mach-2 turbojet with
adiabatic efficiencies, pressure losses, dual gas, and an under-expanded nozzle.
The book inputs *polytropic* efficiencies; we use *isentropic* ones, so the test
converts between them (exact for this kind of gas). Every headline number —
fuel-air ratio, exit Mach and velocity, specific thrust, TSFC, all three
efficiencies — reproduces to **better than 0.02%** (`tests/test_rung2.py`). That a
single textbook example bundles real components *and* real gas *and* a non-ideal
nozzle is the lesson restated: in the real world they all arrive together.

---
*Rung 2 is wired and anchored. `python main.py` now prints the ideal and real
cycles side by side and overlays them on the T–s diagram (`ts_diagram.png`), where
the real cycle's work legs visibly tilt right — the entropy every real component
generates, drawn to scale.*
