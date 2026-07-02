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

---
---

# Rung 2b — Polytropic Efficiency, in plain language

Rung 2 measured a compressor or turbine's quality with an **isentropic
efficiency** `η`: compare the real machine to one perfect machine spanning the
*whole* pressure ratio. Rung 2b adds the engineer's *other* yardstick —
**polytropic efficiency** `e` — as an equal, first-class input. The formal
derivation is in `docs/rung2b-polytropic.md`; this is the plain-language version.

## Two ways to grade the same machine
Picture compressing air in one big squeeze versus a hundred tiny squeezes back to
back. Isentropic efficiency `η` grades the *one big squeeze* against a single ideal
one. Polytropic efficiency `e` grades *each tiny stage* — it's the efficiency of an
infinitesimal step, assumed the same for every step. So `e` is really a property of
the **blade technology**, while `η` is a property of *this machine at this pressure
ratio*. Quote a compressor's `η` and you must also say what `πc` it was measured
at; quote its `e` and the number travels to any pressure ratio. That portability is
the whole reason `e` earns its own knob.

In the code, `e` goes straight into the forward path: a compressor at polytropic
`e_c` lands at `Tt3 = Tt2·πc^(gc/e_c)` directly — the loss is baked into the
exponent, no "ideal substate" needed to get there. (The substate is still computed,
but now only as a cross-check.) For the turbine it's even cleaner, and that's a
genuine payoff: because the *shaft* already fixed the turbine's temperature drop,
`pt5` follows from `e_t` in one line — `pt5 = pt4·(Tt5/Tt4)^(1/(e_t·gt))`. The
rung-2 anchor had to run a throwaway "provisional pass" just to discover the drop
before it could convert the book's `e_t` into an `η_t`; with `e_t` first-class, that
dance disappears. Polytropic is the *natural* knob for a turbine.

## The surprise — same `e`, but `η_c < e < η_t`
Here's the part worth slowing down for. Feed **the same** polytropic efficiency to
both machines — `e_c = e_t = 0.90` — and ask what isentropic efficiency each one
*shows*. They don't both read 0.90. They straddle it:

```
η_c = 0.864   <   e = 0.90   <   η_t = 0.910      (at πc = 10)
```

The compressor looks **worse** than its per-stage quality; the turbine looks
**better**. Same blades, opposite-signed gap. Why? On a T–s diagram the
constant-pressure lines **fan apart** as the gas gets hotter. In the compressor,
each stage's friction dumps heat that makes the *next* stage start hotter, where the
isobars are wider — so the little ideal steps add up to *more* work than one ideal
big squeeze, and the machine grades out **below** `e` (a *preheat penalty*). In the
turbine the very same reheat is a **gift**: heat lost to friction early is still hot
gas the later stages get to expand and harvest, so the machine beats its per-stage
number (the classic *reheat effect*).

And the gap is set by pressure ratio — it grows as you stack more stages and
collapses to nothing in the single-stage limit:

| `πc` | `η_c` (gap below 0.90) | `η_t` (gap above 0.90) |
|---|---|---|
| ~1 | 0.900 (0.000) | 0.900 (0.000) |
| 5 | 0.876 (0.025) | 0.906 (0.006) |
| 10 | 0.864 (0.036) | 0.910 (0.010) |
| 40 | 0.840 (0.060) | 0.923 (0.023) |

So `η` and `e` are not two names for one number — they're the same machine seen at
two scales, and the spread between them *is* the pressure ratio. That's why a real
data sheet quotes `e`: it's the part that doesn't move.

## Did we get it right?
Two checks, both exact rather than approximate. First, a polytropic machine at
`e=0.9` and an isentropic one at the **converted** `η` are not just close — they're
algebraically the *same engine*, agreeing on every station and every performance
number to ~1e-9 (`tests/test_polytropic.py`). Second, the Mattingly Example 7.1
anchor is re-run feeding `e_c = e_t = 0.9` **directly** — no conversion, no
provisional pass — and reproduces the book. The contrast between the two anchor
tests (the rung-2 one's provisional-pass detour vs. this one's single line) is the
lesson restated in code: for turbines, polytropic is the knob the math wants.

---
*Rung 2b is a small, contained sub-rung: one extra parameter on the compressor and
turbine, the reduce-to-ideal gate untouched, the `T–s` diagram left alone (the
`η`-vs-`e` story is a table, not a leg-tilt). The deferred seams — variable
`cp(T)`, off-design, the choked nozzle, the afterburner — stay deferred.*

---
---

# Rung 3 — Variable `cp(T)`, in plain language

Every rung so far froze the gas's heat capacity: `cp`, `γ`, `R` were constants
(rung 2 used two sets of them, cold and hot, but each set was still frozen). Rung 3
lets **`cp` vary with temperature** — a *thermally-perfect* gas. It's still an
ideal gas (`p = ρRT`, `R` fixed per section), but now `cp = cp(T)`, so `γ = γ(T)`
too. This is the rung where the tidy algebra of the first three finally breaks, and
the model has to grow up into the **gas-table** method that real cycle codes use.

## Why constant `cp` ever worked
Air's `cp` really does climb with temperature — about 1004 J/(kg·K) at 300 K, ~1100
at 800 K, past 1200 by 1500 K — because hot molecules wake up extra vibrational modes to
soak energy into. So why did three rungs of constant `cp` give good answers? Because
over any *one* leg the temperature swing is modest and a well-chosen average `cp`
covers it; rung 2's trick of one cold value and one hot value was exactly that —
two averages bracketing the burner. It works until you want the numbers to be
*right* across a 200→1800 K cycle with one coherent gas, and then the averaging
seams start to show.

## What `h(T)` and `pr(T)` buy
Once `cp` varies you can't write `(energy) = cp·(temperature)` anymore, because
*which* `cp`? The honest replacement is to integrate. Two functions carry the whole
load:

- **Enthalpy** `h(T) = ∫cp dT` — the gas's heat content. Every "energy" in the
  cycle (compressor work, the burner balance, the shaft, the jet's kinetic energy)
  becomes a *difference in `h`*, no `cp` in sight.
- **The entropy function** `φ(T) = ∫cp/T dT`, packaged as **reduced pressure**
  `pr(T) = exp(φ(T)/R)`. This is the clever one. For *any* two states,
  `Δs = φ(T2) − φ(T1) − R·ln(p2/p1)`. Set `Δs = 0` (isentropic) and it collapses to

  ```
  p2/p1 = pr(T2)/pr(T1)
  ```

  So **every** isentropic pressure↔temperature step in the engine — ram, compressor
  substate, turbine substate, nozzle expansion — is one `pr` ratio. `pr` is a
  pure lookup that *replaces* the power law. (And `cp(T) > 0` makes `h` and `pr`
  strictly increasing, so we can always invert them: `T_from_h`, `T_from_pr`.)

## Why the isentropic power law had to go
The familiar `Tt3 = Tt2·πc^((γ−1)/γ)` is **only** true for constant `cp`. It is the
closed-form solution of `p2/p1 = pr(T2)/pr(T1)` in the special case `pr(T) = T^(cp/R)`.
The moment `cp` bends with temperature, `pr` is no longer a clean power of `T`, the
exponent stops being a single number, and the formula quietly lies. The gas-table
form `T = T_from_pr(pr(T1)·π)` is what's actually exact — the power law was the
training-wheels version all along.

The visible payoff is the **gas-table effect**. Compress air `πc = 10` from 300 K:
the constant-`cp` law says `300·10^0.2857 = 579 K`; the real gas table says **~574 K**.
Why cooler? Climbing pressure heats the gas, and the hotter it gets the *more* `cp`
it has, so the same compression work shows up as a *smaller* temperature rise. In
the full cycle (`python main.py`, the rung-3 table) at `πc = 10`, `Tt4 = 1500 K`,
compared against the **rung-2 frozen-`cp` dual gas** — the baseline you just left,
`cp_c = 1004`, `cp_t = 1239`:

| | rung-2 frozen `cp` | rung-3 `cp(T)` |
|---|---|---|
| `Tt3` (compressor exit) | 552.4 K | **548.6 K** (cooler — cold-air `cp` rises with T) |
| `f` (fuel-air) | 0.0318 | **0.0279** (*less* fuel — see below) |
| `Tt5` (turbine exit) | 1290.9 K | **1294.5 K** (about the same) |

The `Tt3` row is the pure cold-section effect (both columns are cold air; only `cp`
differs). The fuel row repays a closer look, because it runs *opposite* to a naive
guess. Rung 2 had to **pick** one hot `cp`, and it froze `cp_t = 1239` — but that's a
value the real `cp(T)` only reaches near the turbine inlet (~1240 K). Averaged across
the burner's whole enthalpy climb the true `cp` is lower (~1130), so the frozen value
*overstated* how much heat the products carry, and therefore overstated the fuel.
Thawing `cp` into `cp(T)` corrects that, and the cycle needs slightly **less** fuel.
That's the rung's lesson in one number: freezing `cp` forces you to choose a
representative value that is never right across the whole range; integrating removes
the guess. (Against the rung-1 *single* gas, where `cp_t` is an even-less-realistic
1004, the comparison would flip — which is exactly why the honest baseline is rung 2.)

None of these is a new *kind* of physics — it's the same Brayton cycle, told with
the honest gas. (That's also why the T–s diagram is left alone: variable `cp`
curves the isobars a little but doesn't tilt a leg, so it rides as a table, not a
new picture.)

## The trap that shaped the code — keep the constant-`cp` branch closed-form
Here's the subtlety that's worth the most. Rungs 1–2 reproduce their tables *to the
digit* using `γ = 1.4` (exponent `(γ−1)/γ = 0.28571`) and a **rounded** `R = 287`.
But the gas-table exponent in the constant-`cp` limit is `R/cp = 287/1004 = 0.28586`
— and those differ by ~0.05%, because `287` isn't exactly `(γ−1)/γ·cp = 286.86`.
So if you route a *constant-`cp`* gas through the new integral machinery, it lands
~3e-4 off the old answer — enough to break "to the digit" and threaten the rung-2
anchor.

The fix is a fork hidden inside the `Gas`: a **calorically-perfect section keeps the
exact closed forms** (`h = cp·T`, `pr = T^(1/g)`), while a **thermally-perfect
section integrates**. Reduce-to-ideal then picks the closed-form branch and the old
tables come back bit-for-bit. Most components never see the fork — the compressor,
burner, and turbine work in totals only, and their `pr`-ratio / enthalpy-ratio forms
collapse to the rung-2 algebra *exactly* (the rounded `R` never enters). The fork
only bites at the **two velocity↔enthalpy stations, the freestream and the nozzle**,
where stopping or accelerating the flow couples `cp` to `γ` and `R` — there the
constant-`cp` path keeps the `γ`-based closed form, and the variable-`cp` path uses
the enthalpy split `V² = 2(h(Tt) − h(T))`. (We even built a test that *requires* a
flat-`cp` polynomial to miss the closed-form answer by ~3e-4 — proof the integral
path is genuinely `pr = exp(φ/R)` and not secretly the power law in disguise.)

## Did we get it right?
The air model is NASA's 7-coefficient polynomial fit (a mole-weighted N₂/O₂/Ar
mixture), and it's pinned against published gas tables three ways, all to **≤0.11%**:

- **Çengel & Boles 9-89** (an air Brayton cycle): isentropic compression 295 K → 564.9 K,
  expansion 1240 K → 689.6 K, cycle efficiency 0.301 — matched.
- **Mattingly Examples 2.7 / 2.8** (the project's anchor book's *own* gas-table
  examples): a compression (627.6 K) and a nozzle (2377.7 °R, `P2/P1 = 0.376`) —
  matched. Mattingly uses the identical `pr = exp(φ/R)` formalism, so the design is
  confirmed against the chosen book — just at the *process* level.

**An honest scope note.** Those anchors are all single-gas **air**, so they pin the
air `cp(T)` end-to-end but pin **nothing** in the hot products — there is no worked
variable-`cp` *turbojet* in the standard texts to match thrust against (Çengel is a
power cycle; Mattingly's turbojets are constant-`cp`). So the products are modeled as
a fixed-composition lean-combustion mixture and the products coefficients **float**
(they land `cp_t` near the rung-2 ~1239 neighborhood on their own, via the
temperature dependence — not by tuning). The turbojet *topology* is still anchored
by reduce-to-ideal, and the variable-`cp` path through it by the dual-section
discriminating check. Anchoring the products to the digit needs a reacting-gas
source — which is the *next* rung.

---
*Rung 3 swaps the frozen gas for `cp(T)` behind a four-function property interface
(`h`, `pr`, and their inverses), rewrites every `cp·T` and `π^g` through it, and
keeps the constant-`cp` branch closed-form so the prior tables survive untouched.
`python main.py` prints the constant-vs-variable cycle side by side. The deferred
seams — reacting/variable-composition products, off-design, the choked nozzle, the
afterburner — stay deferred, now behind a cleaner seam: the next gas upgrade is just
a new `Gas` behind the same four functions.*

---

# Rung 4 — Reacting Products, in plain language

Rung 3 let the heat capacity `cp` bend with temperature, but it still burned the
*same* fixed puff of exhaust at every throttle setting — one frozen recipe of CO₂,
water, leftover oxygen, and nitrogen. Rung 4 makes the **recipe itself depend on how
much fuel you burn**. Push more fuel in and the burned gas genuinely changes: more
CO₂ and water vapour, less spare oxygen, a heavier heat sink. That's the last thing
still frozen in the working fluid, and it's the rung the ladder was pointed at.

## Why the frozen recipe worked anyway
Here's the honest surprise: at the design point, tracking the real composition barely
moves the needle. The reacting run lands `far = 0.02781` where the rung-3 frozen
mixture gave `0.02787`, and the thrust is identical to a decimal place. Frozen
composition worked because a well-chosen lean mixture *is* close to the truth over the
narrow band of fuel/air ratios a main burner actually runs (roughly 2–4%). The payoff
of rung 4 isn't a big number change — it's that the composition is no longer a guess
you have to *pick*. It falls out of the chemistry, and it moves correctly the moment
you sweep the throttle: burn hotter and CO₂/H₂O climb, spare O₂ falls, `cp` rises
(`python main.py` prints the sweep). You buy *honesty and reach*, not a design-point
correction — and the reach is what rung 5 (dissociation) needs.

## The chemistry, kept deliberately simple
The fuel is idealised as `(CH₂)ₙ` (Jet-A is about C₁₂H₂₃, close enough), and it burns
**lean and complete**: `CH₂ + 1.5 O₂ → CO₂ + H₂O`. Per mole of air you know exactly
how much fuel went in (from `f`), so you know exactly how many moles of each product
come out — nitrogen and argon ride through untouched, every mole of fuel makes one CO₂
and one water and eats 1.5 oxygen. Feed those mole numbers to the *same* mole-weighting
the frozen gas already used and you get `cp(T, f)`, `R(f)`, `γ(T, f)` for free. No new
physics in the property layer — just a composition that now has an `f` in it. The
stoichiometry checks out: the model's stoichiometric fuel/air ratio (where the spare
oxygen hits zero) is `0.0677`, versus Mattingly's `0.0676`.

*(A counter-intuitive detail worth stating: `R` goes* up *slightly with fuel, not down.
"More/heavier products" sounds like it should raise the molar mass and lower `R`, but
each mole of fuel swaps 1.5 O₂ for one CO₂ **and one light water molecule**, and the
net mean molar mass drops a touch. Mattingly's own `R(f)` formula rises with `f` for
the same reason. We assert the rise in the tests and flag it, because the tidy story is
backwards.)*

## Why the burner had to go implicit
This is the one real mechanical change. In rung 3 the fuel balance was a one-liner: you
knew the enthalpy of the products at the turbine-inlet temperature, so you solved for
`f` in a single step. But now the products' enthalpy `h_t(Tt4, f)` **depends on `f`** —
the very thing you're solving for sits on both sides of the equation: `f = g(f)`. You
can't isolate it algebraically.

The fix is the oldest trick for `f = g(f)`: guess, plug in, get a better guess, repeat.
It converges because burning a little more fuel changes the products' `cp` by only a
few percent, and that change enters weakly (through a `~f/(1+f)` weight), so each pass
shrinks the error by about ten-fold — five or six passes and it's nailed. Reassuringly,
this isn't our invention: it's Mattingly's own Eq. 6.36, right down to his footnote
that "the solution … is iterative." And it degrades gracefully — hand it a non-reacting
gas (constant or frozen `cp`) and `g` stops depending on `f`, so the loop lands on the
old one-shot answer in two passes. The rung-1/2/3 tables never notice.

## The datum trap we deliberately didn't spring (Fork A vs Fork B)
There's a fork in the road here, and we took the simple branch on purpose. Every
enthalpy in this model is measured from the same zero (`h(0) = 0`, "sensible" heat), and
the fuel's chemical energy is booked as one fixed number, `hPR = 42.8 MJ/kg`. That's
**Fork A**. The grown-up alternative — **Fork B** — gives every species an absolute
*formation* enthalpy, so the heat released by combustion is *derived* from the chemistry
instead of assumed. Fork B is the natural partner for dissociation (products tearing
back apart at high temperature), so both belong to rung 5.

Does Fork A cost us anything at rung 4? The burner is the *one* place two enthalpies
measured on possibly-different datums get subtracted (hot products minus cold air), so
it's the place to check. We solved the same burner two ways — our `h(0)=0` model and
Mattingly's tables, which carry a real +32 Btu/lbm offset between products and air — and
they agree on `f` to **0.17%**. So the cross-datum energy is negligible at lean fuel/air
ratios, `hPR` needs no re-tuning, and Fork A is *validated*, not just assumed. Crucially,
the property layer is written so Fork B is *additive* (bolt a formation enthalpy onto
each species) rather than a rewrite — the seam is kept.

## Did we get it right?
The reacting machinery is anchored to Mattingly's own products example — **Example 6.3**,
a turbine expansion of real combustion gas at `f = 0.0338`. Running our production
stoichiometry, property functions, and turbine code reproduces his datum-independent
results — `η_t = 0.9057`, `Tt5 = 2677.5 °R`, `π_t = 0.5650` — to **~0.02%**, tighter
than the rung-3 air anchor. That single example exercises the whole new path: the
`(CH₂)ₙ` stoichiometry, the mole-weighted `cp(T, f)`, and the `pr`-ratio expansion. A
second, independent property model (Mattingly's Table 2.2 `f`-blend, built test-only)
reproduces the same textbook enthalpies *to the digit*, so the anchor numbers are
confirmed two ways before a line of production code trusted them.

---
*Rung 4 makes the exhaust recipe a function of the fuel/air ratio: explicit
lean-combustion stoichiometry feeds the same mole-weighting rung 3 already had, so the
hot-section property functions simply gain an `f` argument. The only structural change
is the burner, which becomes an implicit `f = g(f)` fixed-point solve (Mattingly's own).
The frozen paths are untouched — reacting is a separate `Gas.reacting()` — so every
prior table survives to the digit. `python main.py` prints the frozen-vs-reacting
comparison and an `f`-sweep. Now split off as its own step: formation-enthalpy
bookkeeping (rung 5, below); high-temperature dissociation (rung 6); off-design, the
choked nozzle, the afterburner still deferred.*

---

# Rung 5 — Fork B: Derived Heat Release, in plain language

Rung 4 left a promissory note: the fuel's chemical energy was one typed-in number,
`hPR = 42.8 MJ/kg`. Where does that number *come from*? Rung 5 answers it. Every species
now carries its **formation enthalpy** — the energy locked in its chemical bonds,
measured from a common zero (the elements at 25 °C). Burn `CH₂ + 1.5 O₂ → CO₂ + H₂O`
and the heat released is just the bookkeeping difference: the bonds in the products hold
*less* energy than the bonds in the fuel + air, and the surplus becomes heat. The heating
value stops being an input and **falls out of the chemistry**.

## The one honest surprise: the numbers don't move (and that's the point)
Here is the twist. We derived the heating value from formation enthalpies, plugged it in,
and the whole cycle came out **bit-for-bit identical** to rung 4 — same `f`, same `Tt5`,
same thrust, to the last decimal. Not "close." *Identical.* That is not a bug; it is a
small theorem. For complete combustion the energy released is *exactly* `f × LHV` for
every `f`, so the fancy absolute-enthalpy balance is algebraically the rung-4 balance
with `hPR` set to the derived LHV. Since we pinned the fuel's formation enthalpy to
reproduce Mattingly's 42.8 MJ/kg, the two are the same number and nothing shifts.

So what did we buy? Not digits — **structure**. Three things:
- **The heating value is now explained, not asserted.** `42.8 MJ/kg` emerges from
  `ΔHf(CO₂) = −393.5` and `ΔHf(H₂O) = −241.8 kJ/mol` plus one fuel calibration; it is no
  longer a magic constant.
- **Enthalpies now live on the absolute scale.** That is the scale the *next* rung needs:
  to know whether `CO₂` wants to break into `CO + ½O₂` at high temperature, you need the
  formation energies — the sensible "how hot" numbers can't tell you.
- **Heat release is now composition-aware.** While the products stay fixed it makes no
  difference; but the moment rung 6 lets them shift, the energy books follow for free.

We are careful **not** to oversell "derived, not assumed": the fuel's formation enthalpy
(`≈ −35 kJ/mol`) carries exactly the information `hPR` did — it *is* the calibration knob,
just wearing chemistry's clothes. The win is that the release is now structural.

## The mechanism — a constant we had dropped on purpose
The NASA polynomials each have seven coefficients. Rungs 3–4 used only the first five —
the ones that set the *shape* of `cp(T)`. The sixth, `a6`, is the **formation constant**,
and we'd left it out because it cancels in every enthalpy *difference* — and the turbine
and nozzle only ever use differences. Rung 5 restores it. The elegant consequence:
because `a6` cancels in a difference, **only the burner** — the single place a hot enthalpy
is subtracted from a cold one across the combustion hand-off — ever sees it. The turbine
and nozzle stay bit-for-bit rung 4. We even *derived* each `a6` from the tabulated
formation enthalpy rather than transcribe a new column, so there's no fresh chance to fat-
finger a number, and elements land exactly at zero enthalpy at 25 °C by construction.

## A physical check we can't fully cash yet
With absolute enthalpies we can compute an **adiabatic flame temperature** — burn the
fuel with no heat loss and no shaft work, and ask how hot the products get. Ours comes out
around **2375 K** at stoichiometric. Real kerosene flames top out ~2250 K. We're *high* —
on purpose. The gap is dissociation: at those temperatures `CO₂` and `H₂O` tear partly
back apart, and tearing them apart *absorbs* heat, capping the peak. Our complete-
combustion model can't see that. That missing 100-plus kelvin is exactly rung 6's job,
and it's *why* Fork B had to come first: the equilibrium that lowers the flame temperature
is driven by the very formation enthalpies we just installed.

## Did we get it right?
Three checks, all green. (1) The derived heating value reproduces Mattingly's assumed
`hPR = 42.8 MJ/kg` to six figures. (2) Each species' absolute enthalpy equals its
tabulated formation enthalpy at 25 °C by construction, and the elements sit at zero.
(3) The reduce-to-rung-4 gate: a Fork-B engine matches a Fork-A engine to **machine
precision** — the theorem above, made a standing test. And a live-knob test confirms the
chemistry actually drives the burner: hand it a lower-energy fuel and it correctly burns
*more* of it.

---
*Rung 5 restores the NASA formation constant `a6` that rungs 3–4 dropped, putting
enthalpies on an absolute scale so the burner's heat release is derived from an
energy balance rather than an assumed `hPR`. Because released energy is identically
`f·LHV` for complete combustion, the cycle is unchanged to the digit — Fork B buys
structure (explained heating value, the absolute scale rung 6 needs, composition-aware
release), not new numbers, and only the burner touches `a6`. `Gas.reacting_forkb()` is a
separate factory, so every prior table survives untouched. `python main.py` prints the
Fork-A-vs-Fork-B panel. Next, on this absolute-enthalpy substrate: high-temperature
dissociation and the `Kp` equilibrium solve (rung 6, which also restores `a7`), then
off-design, the choked nozzle, the afterburner.*

---

# Rung 6 — High-Temperature Dissociation, in plain language

Every rung so far burned to a **fixed recipe** of finished products — `CH₂ + 1.5 O₂ →
CO₂ + H₂O`, full stop. But real flames at their hottest don't finish the job: above
~2000 K the products **tear partly back apart**. Some `CO₂` splits into `CO + ½O₂`, some
water into `OH`, `H₂`, even lone `O` and `H` atoms. Rung 6 stops assuming the recipe and
**computes** it — the composition is whatever **chemical equilibrium** says it is at the
local temperature and pressure. This is the last thing the burner was still faking, and
it's the payoff the rung-5 notes promised.

## The honest headline (again): the cycle barely moves
Just like rungs 4 and 5, tracking the truer physics **barely changes the engine's
numbers** — the fuel/air ratio shifts by well under a percent and thrust by a hair. And
the size of even that tiny shift is *steeply* temperature-dependent, which is itself the
lesson: at the `main.py` panel's `Tt4 = 1500 K` it's a mere **+0.02 %**, but at the hotter
`Tt4 = 1800 K` supersonic anchor it grows to **+0.15 %** — because dissociation is a
hot-end effect, and even 1800 K is near the metallurgical ceiling. Run it hotter still and
it would climb further. There's a beautiful reason it stays this small, worth slowing down
for: dissociation is suppressed **twice over** exactly where the engine runs:

- **The engine runs lean.** With spare oxygen around, Le Chatelier pushes `CO₂⇌CO+½O₂`
  back to the left — the excess O₂ crowds out the split.
- **The engine runs at high pressure.** Splitting one molecule into one-and-a-half makes
  *more* molecules, and squeezing a gas fights any reaction that makes more molecules. At
  the combustor's ~8-13 bar, dissociation is stomped flat.

Add the fact that turbine blades cap `Tt4` **below** where dissociation really bites, and
station 4 sits in a triple-safe corner. So the cycle inherits rung 5's numbers almost
exactly. **The drama is elsewhere.**

## Where the drama actually is: the flame temperature finally drops
Rung 5 left a loose end: our no-dissociation **adiabatic flame temperature** came out
~2375 K at stoichiometric, but real kerosene-air flames top out ~2250 K. We were *high on
purpose* — and rung 6 pays it off. Tearing `CO₂`/`H₂O` apart **absorbs** heat (it's the
combustion reaction run backwards), so it caps the peak. Turn dissociation on and the
stoichiometric flame temperature falls **~115 K, from 2375 to 2259 K** — right into the
real band (`python main.py` prints the drop). That missing 115 K was never a bug; it was
the physics we hadn't modeled yet. Now we do.

The contrast *is* the lesson: dissociation is a **near-stoichiometric, ~1-atm** phenomenon
(a flame in open air), and the engine deliberately runs **lean, high-pressure, and
temperature-capped** — as far from that corner as it can get. So the same physics that
rewrites the flame temperature leaves the cycle alone.

## The mechanism — one more NASA constant, and a pressure knob
To know *which way* `CO₂⇌CO+½O₂` wants to go, you need the **Gibbs free energy** `ΔG° =
ΔH° − TΔS°`, and the equilibrium constant `Kp = exp(−ΔG°/RuT)`. Rung 5 installed the
absolute-**enthalpy** half (`ΔH°`, via the formation constant `a6`). Rung 6 adds the
absolute-**entropy** half: the seventh NASA coefficient `a7`, derived from each species'
tabulated standard entropy exactly the way `a6` was derived from its formation enthalpy
(no new columns transcribed, so no fresh chance to fat-finger one). That's *why* Fork B
had to come first — you cannot compute equilibrium from sensible "how hot" numbers alone;
the formation energies **are** the driving force.

The one piece of algebra worth seeing is the **pressure factor**. Equilibrium isn't just
a ratio of concentrations — because dissociation changes the molecule count, it carries a
`(p/p°)^Δν` term:

```
Kp(T) = Πᵢ (xᵢ)^νᵢ · (p/p°)^Δν
```

That `(p/p°)^Δν` is the whole reason high combustor pressure suppresses dissociation, and
`main.py` shows it live: hold a stoichiometric flame at 2300 K and squeeze it — the `CO`
fraction falls from 11 % at 1 atm to 5 % at 13 atm. Get this factor wrong and you get a
plausible-looking but wrong flame; it's the classic trap, so we reproduce a published
equilibrium point before trusting the solver (below).

## One subtlety we had to get exactly right: which zero for enthalpy
There are two consistent ways to place the enthalpy "zero," and they differ by ~1 % in the
burner. The equilibrium constant `Kp` **demands** the textbook formation scale (elements =
0 at 25 °C) — anything else gives the wrong reaction `ΔG°` and a ~20 %-wrong `Kp`. But the
cycle's energy bookkeeping (from rungs 4-5) lives on a different zero, and switching *that*
would break the promise that rung 6 reduces to rung 5. The resolution is clean once you see
it: **`Kp` is a datum-free physical constant** — computing it correctly needs the formation
scale, but its *output* is just mole numbers, which carry no zero at all. So the model keeps
**one** energy datum (rung 5's), feeds it the datum-free composition, and computes `Kp` on
the formation scale off to the side. We prove there's no hidden seam with a standing test:
in the cold-flame limit the rung-6 fuel/air ratio snaps back to rung 5's to one part in a
million — a leaked datum would show up as a constant 1 % gap, and it doesn't. (This is the
same split rung 5 already used: production energy on one scale, the flame-temperature
diagnostic on the formation scale.)

## Kept deliberately simple, with the seams marked
We model the C/H/O dissociation set — `CO, H₂, OH, O, H` beside the finished products — and
**freeze** the equilibrium composition at the burner, carrying it unchanged through turbine
and nozzle. Two honest omissions, each its own future rung: **thermal NOx** (`N₂+O₂⇌NO`) is
left out because in reality it's rate-limited, not equilibrium, and it's a pollutant-
formation story of its own; and **re-equilibrating the expansion** (the classic "frozen vs
equilibrium nozzle flow" contrast, where recombination gives heat back) is deferred because
at our lean, high-pressure station 4 there's almost nothing to recombine — but it's
genuinely rich and deserves its own treatment.

## Did we get it right?
Three ways. **(1)** The independent physics anchor: our solver reproduces the well-known
**CEA methane-air flame temperature** — 2231.7 K against CEA's ~2226 K — with textbook
dissociation-product fractions, using the exact same machinery the cycle uses. **(2)** The
formation and entropy self-checks land each species' enthalpy and entropy on its tabulated
value at 25 °C to the last digit, and the derived `a7` matches GRI-Mech's own tabulated
constant. **(3)** The reduce-to-rung-5 gate: turn the flame cold and the whole cycle snaps
back to Fork B to one part in a million — dissociation adds *only* what dissociation should.

---
*Rung 6 replaces the burner's fixed recipe with a chemical-equilibrium solve: five
dissociation reactions, element conservation, and `Kp(T) = exp(−ΔG°/RuT)` on the absolute
entropy `a7` (derived like rung 5's `a6`). The products dissociate, but at the engine's
lean, high-pressure, temperature-capped station 4 the cycle barely moves (+0.15 % on far);
the drama is the adiabatic flame temperature, which finally falls ~115 K into the real
band. The cycle delta is steeply Tt4-dependent (+0.02 % at the 1500 K panel, +0.15 % at the
1800 K anchor) — a hot-end effect. `Gas.reacting_equilibrium()` is a separate factory, so
every prior table survives to the digit. `python main.py` prints the Fork-B-vs-equilibrium
panel, the flame-temperature drop, and the pressure-suppression line. Still deferred: thermal NOx and equilibrium-vs-
frozen nozzle flow (both ride this same `a6`+`a7`+`Kp` substrate); off-design, the choked
nozzle, the afterburner.*

---

# Rung 7 — Thermal NOx: Kinetically-Limited NO, in plain language

Every rung so far asked *what mixture is the gas*, and answered with **equilibrium** — the
composition chemistry settles into, given enough time. Rung 7 asks a different question:
*how much of a pollutant actually forms*, and the answer is the first one in the whole ladder
that is **not** set by equilibrium. Thermal **nitric oxide** (NO) — the "NOx" an engine is
regulated on — forms so *slowly* that in a combustor it never gets close to its equilibrium
amount. This is the rung where **kinetics** (rates, and a residence *time*) enters, and its
lesson is a clean **inversion of rung 6**.

## The headline: the lesson flips
- **Rung 6:** the major species (CO₂, H₂O, and their dissociation fragments) *do* reach
  equilibrium — fast reactions, plenty of time. The cycle barely moved; the drama was the
  flame temperature.
- **Rung 7:** NO does **not** reach equilibrium. At a realistic combustor residence time
  (~3 ms) it stalls at just a **few percent** of its equilibrium value. The one reaction that
  makes it is a bottleneck, and the gas is out the door long before it finishes.

So the same modelling substrate now teaches the opposite point: *knowing the equilibrium
composition is not enough — sometimes the rate is everything.*

## Why NO is so slow: one stubborn reaction
Thermal NO forms by the **extended Zeldovich mechanism**, three reactions, but the whole thing
is gated by the first one:

```
O + N₂ → NO + N     (slow: it must crack the triple bond in N₂)
N + O₂ → NO + O      (fast — the N atom made above is consumed instantly)
N + OH → NO + H      (fast)
```

Breaking the N₂ triple bond costs an enormous **activation energy** (~319 kJ/mol), so the first
step's rate constant carries a brutal `exp(−38370/T)`. The nitrogen atom it produces is so
reactive it's gone the instant it appears (we treat it as **quasi-steady** — never accumulating),
which collapses the three reactions into a single rate for NO. The telling number is the
**characteristic time** to reach equilibrium NO: about **90 ms at 2300 K**, and nearly a
**full second at 2100 K** — against a combustor residence of a few ms. NO simply runs out of
time. (`main.py` shows it directly: hold a 2300 K flame and stretch the residence — NO climbs
from ~17 ppm at 0.5 ms toward its 3083 ppm equilibrium, reaching it only past ~1 second.)

## The payoff: NO is exponentially temperature-sensitive
Because that `exp(−38370/T)` sits on the rate (and the oxygen-atom pool feeding it is itself
steeply temperature-dependent), the NO formation rate is savagely sensitive to temperature —
about **30× for every 200 K**, a ~500× swing from 2000 to 2400 K (the `main.py` sweep). This
is *the* number, and it explains a real engineering fact: it is the **peak flame temperature**,
not the turbine-blade limit alone, that governs NOx. You can cap the *mixed-out* turbine-inlet
temperature all you like (metallurgy already forces that), but if the flame has a hot
near-stoichiometric zone, NO pours out of it. That is exactly why modern low-NOx combustors
chase *temperature* — lean-premixed burning, staging — rather than anything about the shaft or
the nozzle.

## The cycle doesn't move — and station 4 makes almost none
NO is a **trace** species (parts per million), so it neither steals meaningful heat nor changes
the mole count: the cycle stays **bit-for-bit rung 6**, and NO rides as a pure **diagnostic**
layered on top (the same way rung 6's flame-temperature drop was a diagnostic beside an
unmoved cycle). And at *this* engine's station 4 — capped at Tt4, running lean, and taken at
the **mixed-out** average — thermal NO is essentially nil (a few *ten-thousandths* of a ppm at
the 1500 K panel point). That's honest, and it flags the model's edge: real engine NOx is made
in the combustor's **hot primary zone**, near stoichiometric and far above the mixed-out Tt4,
during the millisecond before dilution air quenches it. This single-`Tt4`, mixed-out cycle
model doesn't resolve that zone — so the *diagnostic* (evaluated at flame temperature) carries
the physics, while the *station-4 number* honestly reports "not here." Resolving the primary
zone (a rich-front → dilution model) is a stated next rung.

## A second, quieter inversion — pressure
Rung 6's dissociation was **suppressed by pressure**: splitting molecules makes *more*
molecules, and squeezing fights that (`(p/p°)^Δν`). NO is different — `½N₂ + ½O₂ ⇌ NO`
conserves the molecule count (`Δν = 0`), so **equilibrium NO carries no pressure factor at
all**. High combustor pressure, which stomped dissociation flat, does **not** directly save you
from NOx. A small thing, but it's the same knob giving opposite answers for two different
chemistries — worth noticing.

## Did we get it right?
The clever check ties the *new* kinetics back to the *already-verified* thermodynamics. The
first two Zeldovich reactions sum to `N₂ + O₂ ⇌ 2 NO`, and thermodynamics demands that the
rate constants and the equilibrium constant agree: `k1f·k2f/(k1r·k2r)` must equal
`exp(−ΔG°/RuT)`, computed from the very same formation/entropy constants (`a6`, `a7`) rung 6
installed — needing only NO's thermochemistry (the nitrogen atom cancels). They agree to
**~4 %** across the whole flame band (a gross transcription slip would be off by orders of
magnitude), which certifies the transcribed **rate constants and NO's formation data
together** — rung 7's version of rung 6's derived-vs-tabulated check. Two more internal gates
back it: stretch the residence time to infinity and the kinetic integrator must recover the
independently-computed equilibrium NO (it does, exactly); and equilibrium NO at a stoichiometric
2300 K flame lands ~3000 ppm, squarely in the known band. Two honest wrinkles, stated rather
than papered over: (1) NO's enthalpy of formation has a real ~1 kJ/mol literature spread — we
take JANAF's 90.29 kJ/mol, and the K-check confirms that pick over GRI-Mech's slightly higher
value; (2) unlike every rung 2–6 anchor, the *absolute* NO formation **rate** has no local
textbook digit to match here. What's hard-certified is the rate constants and NO thermochemistry
*relatively*, through the K-check; the absolute rate rests on order-of-magnitude literature
(~34 ppm forms in the first millisecond at 2300 K) fenced by a two-sided sanity gate — not the
0.02 %-to-the-book agreement the earlier rungs enjoy. Pinning it to a specific worked example
*read from the text* is a clean future tightening.

---
*Rung 7 adds thermal NO as the first **kinetically-limited** quantity and the first with a
**time** in it (a residence-time knob, stated like the specified exit pressure was). Two new
species (NO, N) join the data tables inertly; the extended Zeldovich mechanism, a superimposed
equilibrium-NO layer, and a one-equation kinetic integrator ride on the rung-6 frozen pool and
the `a6`+`a7`/`Kp` substrate — NO never enters the equilibrium solve, so every rung 1–6 table
survives untouched and the cycle is bit-for-bit rung 6. `python main.py` prints the NOx panel:
the flame-temperature sweep (equilibrium vs kinetic vs residence, the ~500× temperature
sensitivity), the honest near-zero station-4 number, and the pressure-independence contrast.
Still deferred, all on this same substrate: super-equilibrium O / prompt NO (a richer radical
pool and the Fenimore path), combustor zoning (to make station-4 NOx engine-realistic), and the
rung-6 equilibrium-vs-frozen nozzle seam; plus off-design, the choked nozzle, the afterburner.*

---

# Rung 8 — Combustor Zoning: the Primary-Zone NOx Effect, in plain language

## The headline: the zero was an averaging artifact
Rung 7 ended on an honest anticlimax — at the real turbine inlet (station 4), thermal NO was
essentially nil, a few ten-thousandths of a ppm — and pointed at the reason: *real NOx is made
in a hot primary zone this single-`Tt4` model averages away.* Rung 8 stops pointing and
**resolves that zone**, and the near-zero jumps into the measured band. The lesson **completes
rung 7's inversion**: it was never the capped, mixed-out 1500 K turbine inlet that made the NO —
it was a ~2400 K zone upstream that you blended into the average before you looked. Same
integrator, same chemistry, same substrate; the *only* thing that changed is **where** the
chemistry is evaluated.

## What a real combustor actually does
A gas-turbine combustor does not burn its fuel uniformly at Tt4. It burns nearly all of it in a
compact **primary zone** with only *part* of the air, close to stoichiometric, at ~2000–2450 K —
the peak flame temperature. Then it dumps in the rest of the air as **dilution**, cooling the
mixed-out gas down to the metallurgical Tt4 the blades can survive. NO is made in that hot
primary during the millisecond before the dilution air quenches it — and once made it is
kinetically frozen (rung 7), so the dilution cools the gas but **cannot un-make the NO**. Our
model is the simplest honest version of this: split the air (fraction α to the primary with all
the fuel), burn the primary adiabatically from the actual compressor-exit Tt3, run the rung-7
Zeldovich integrator there, then add the remaining air and mix out — freezing the NO.

## The result: ~6 orders of magnitude, purely from *where*
At this design point (Tt3≈584 K, Tt4=1500 K, 7.5 bar), a near-stoichiometric primary (φ_p ≈
0.9–1.0) reaches ~2360–2440 K and makes **EI_NO ≈ 16–21 g NO / kg fuel** — squarely inside the
ICAO take-off band (18–64 g/kg for real turbofans). The *same* far, evaluated mixed-out at Tt4,
gave ~8×10⁻⁶ g/kg. That is a **~6-order-of-magnitude lift with no new physics** — just resolving
the zone instead of the average. And the temperature sensitivity from rung 7 shows through
undiluted: dropping the primary from φ_p=1.0 to 0.7 (AFT 2442→2068 K, −374 K) collapses EI_NO
**30×**. This is *why* every low-NOx combustor architecture — lean-premixed, staged, RQL — is a
fight against **peak flame temperature**, not against the average the cycle designed to the blade
limit.

## The conservation gate: mixing back to the same station 4
There is a check that makes the whole two-zone picture trustworthy: the **mixed-out temperature
is split-independent**. Whether you route 57 % or 40 % of the air through the primary, `T_mix`
comes out identical (≈1517 K here) — because enthalpy is conserved and the total fuel and total
air are fixed, so *how* you divided the air can't change the mixed-out state. And that `T_mix`
returns to ≈ Tt4 — the zoned combustor mixes back to the very station 4 the cycle already
computed. This only works because the majors **re-equilibrate** on dilution: the dissociated
primary products (CO, H₂, OH, O, H) recombine as the gas cools and *release* their stored
dissociation energy back into heat. If you instead **froze** that dissociated composition
through the mix-out, the energy stays trapped in the bonds and the gas lands ~60 K cooler,
missing Tt4 — the discriminating test that proves the re-equilibration is real, not cosmetic.

## Concentration is not emission: the dilution wrinkle
Dilution air drops the NO **mole fraction** — from ~1250 ppm in the primary down to ~530 ppm
mixed-out at φ_p=1.0 — which looks like it "reduced the NOx." It didn't. All the fuel and all
the NO were made in the primary; the dilution only spread the same NO *moles* through more gas.
The **emission index** (grams of NO per kg of fuel — what an ICAO measurement reports, and what
the environment sees) is set entirely in the primary and is **unchanged by dilution**. Watching
the ppm fall while the EI holds is the clean separation between a *concentration* and an
*emission index*, and it's exactly why you can't dilute your way out of a NOx problem.

## One honest surprise: an 8 K datum crack, found by looking from Tt3
Computing the primary flame temperature *from Tt3* (not from 298 K, as every earlier
flame-temperature diagnostic did) exposed something the cycle had been quietly hiding. On the
**formation datum** (scale A — the physically-correct, CEA-matching one this model uses for
adiabatic flame temperatures), the true adiabatic flame temperature of the burner's fuel/air is
~1508 K, about **8 K above** the 1500 K the cycle labels Tt4. That gap is *not* a bug in rung 8;
it is the Fork-B burner's **energy datum** (scale B — a 0 K-sensible + formation-enthalpy
reference) differing from the formation datum, and the two disagree because the mole count
changes across combustion so the per-species offset doesn't cancel. Downstream it is invisible —
the turbine and nozzle work on *sensible* enthalpy differences where the offset cancels exactly,
which is why the cycle is still bit-for-bit correct — but a *from-Tt3 flame temperature* is the
first quantity that straddles reactants and products on one absolute scale, so it is the first to
see it. We note it, keep scale A (the honest flame temperature drives the honest NO rate), and
leave Fork B's datum alone — chasing 8 K into the burner would re-litigate a rung-5 invariant for
a number the cycle never uses.

## Did we get it right?
The load-bearing check is **reduce-to-rung-7**, in two honest halves. (1) *Exact:* send all the
air to the primary (α→1) and the two-zone diagnostic becomes the rung-7 single pool — the zoned
EI_NO equals `thermal_nox` evaluated at the *same* primary flame temperature to machine
precision, which certifies the air split, the fuel bookkeeping, and the mole-freeze scaling are
all right. (2) *Physical:* at α→1 the primary flame temperature really does land just above Tt4
(the 8 K datum offset plus a ~9 K combustion-efficiency piece), and the zoned EI is within an
O(1) factor of the honest rung-7 mixed-out number — so it is genuinely *reduce-to-rung-7*, not
reduce-to-itself. The rest of the gates hold: EI_NO lands in the ICAO band at φ_p≈1; `T_mix` is
split-independent and re-equilibrates back to Tt4 (with the frozen-majors contrast to prove it);
the NO **moles** are conserved through dilution while the fraction falls; EI_NO rises >10× with
primary φ_p; and rung 7's thermo-kinetic K-check still binds at the hotter primary temperature.
Every rung 1–7 test stays green, untouched — the cycle never moved. What stays **un-anchored**,
stated plainly: we cap the primary at φ_p ≤ 1 (our stoichiometry is lean-complete-combustion), so
a **rich** primary and the real **rich-quench-lean (RQL)** low-NOx combustor are the next seam;
super-equilibrium O and prompt (Fenimore) NO are still deferred, so even the resolved primary
under-counts the true flame-front rate; and the EI_NO *band* is an order-of-magnitude landing
zone, not a to-the-digit anchor (the absolute Zeldovich rate is un-pinned, per rung 7). Our
in-band landing is therefore partly the lean-stoich cap standing in for rich + super-equilibrium
O — the right orders of magnitude for the right reason, honestly fenced.

---
*Rung 8 makes rung 7's "real NOx is a hot primary-zone effect" concrete **without touching the
cycle**: a two-zone (near-stoichiometric primary → dilution) combustor diagnostic
(`Gas.zoned_nox`) that runs the **same** rung-7 Zeldovich integrator on a hot primary pool
instead of the mixed-out station 4. A reusable primary-AFT solve (from Tt3, scale A) and a
re-equilibrating mix-out step ride entirely on the rung-6/7 primitives; NO is still trace, so
every station is bit-for-bit rung 6 and the whole rung 1–7 suite stays green. `python main.py`
prints the zoning panel: the φ_p sweep lifting EI_NO from the mixed-out ~zero into the ICAO band,
the split-independent `T_mix` returning to Tt4, and the dilution NO-fraction drop at conserved
EI. Still deferred, all on this same substrate: a **rich primary / RQL** combustor (rich CO/H₂
stoichiometry), **super-equilibrium O / prompt NO** (the Fenimore path), **finite-rate mix-out**
(a secondary-zone Zeldovich instead of a frozen NO), and the rung-6 **equilibrium-vs-frozen
nozzle** seam; plus off-design, the choked nozzle, the afterburner.*

---

# Rung 9 — Rich Primary / RQL: the Rich Flank of the NOx Bell, in plain language

## The headline: the bell has two sides, and we'd only seen one
Rung 8 resolved the hot primary zone and watched EI_NO climb into the ICAO band as the primary
approached stoichiometric — but it held the primary **lean-to-stoich** (φ_p ≤ 1). It was climbing
the *lean* side of a hill without seeing over the top. Rung 9 lets the primary run **rich** and
shows the whole hill: the classic **NO-versus-equivalence-ratio bell**. EI_NO peaks right around
stoichiometric and then **collapses** as the primary goes rich — and *that collapse is the entire
reason a modern low-NOx combustor exists*.

## Why going rich kills NO — two effects, both pointing the same way
Thermal NO needs two things: a very hot gas, and free oxygen atoms to start the chain
(`O + N₂ → NO + N`). Going rich past stoichiometric attacks **both**. First, the flame temperature
**rolls over** — it peaks a hair rich of stoich (~φ 1.05) and then falls, because the extra fuel
is unburnt ballast soaking up heat. Second, and more sharply, a rich mixture is **oxygen-starved**:
there simply isn't enough O₂ to go around, so the atomic-O pool the Zeldovich initiation depends on
crashes by orders of magnitude. Cooler *and* O-starved — the NO rate falls off a cliff. In our
worked example a rich primary at φ_p = 1.4 makes about **1800× less NO** than the stoichiometric
one, even though it burns exactly the same fuel. (That factor is a **model lower bound**, not the
real RQL benefit: our rich pool has only *equilibrium* O — super-equilibrium O and prompt NO,
both strongest in the rich primary, are deferred, so the true reduction is smaller. The *shape*
of the collapse is the result; the exact multiple is a floor.)

## This is why RQL burns rich on purpose
Real engines exploit this directly with a **Rich-burn / Quick-Quench / Lean-burn (RQL)**
combustor: burn the primary **rich** (low NO — cool and O-starved), then add air to finish lean.
The catch is the transition: to get from rich to lean you must pass *through* stoichiometric — the
top of the NO bell. So the quench has to be **quick**: mix the air in fast enough that the gas
doesn't *dwell* at the peak and re-make the NO the rich primary so carefully avoided. "Quick" is
the whole design. (Our model does the *ideal* — infinitely-fast — quench, so it freezes the low
rich-primary NO cleanly; the finite-rate quench, where a *slow* mix spikes the NO, is the next
rung. We say so plainly rather than claiming RQL is finished.)

## What "rich" required in the code: almost nothing
The satisfying part is how little had to change. The equilibrium solver was **already** capable of
rich combustion — CO and H₂ were always among its unknowns, and two of its five reactions together
*are* the water-gas shift (CO + H₂O ⇌ CO₂ + H₂) that governs rich products. The only thing that
assumed lean was the solver's **starting guess**, which allocated all carbon to CO₂ and left over
oxygen — nonsense when there isn't enough oxygen to begin with. So rung 9 is essentially a **new
starting guess** for the rich case (water first, then carbon to CO, upgrade to CO₂ with whatever
oxygen is left), branched on a single sign test. The lean starting guess is left **byte-for-byte
untouched**, which is what makes "the model still contains rung 8 exactly" a *proof* rather than a
hope — every earlier rung burns lean and never touches the new branch.

## Where the model stops: soot
The five reactions make only gases — no soot, no solid carbon. That's fine up to a point, and then
it isn't: real rich flames start making soot around φ ≈ 1.8–2, and the equilibrium math itself goes
singular when there's exactly enough carbon to consume all the oxygen (C/O = 1, which for this fuel
is φ = 3). So we draw a hard line at **φ_p ≤ 2**: it covers the real rich-primary range with room to
spare, sits at the practical soot limit, and stays well clear of the singularity. Past it the model
would cheerfully return a soot-free answer that doesn't physically exist — so we don't return one,
we stop with an assertion. Knowing where a model *stops being true* is part of the model.

## Did we get it right?
The rich equilibrium anchors to **NASA-CEA**: methane-air flame temperature peaks slightly rich at
~2231 K (we get ~2238, the same ~7 K we've run high since rung 6 for deferring NO/N), and falls on
both flanks. The rich products pass a **water-gas-shift self-check** to a part in a million — the
thermodynamic tell that the solver found the *real* equilibrium, not merely an atom-balanced
guess. The EI_NO bell peaks near stoichiometric in the ICAO band and collapses on the rich flank
for the two physical reasons above. `T_mix` still returns to the same station 4 for *every* split —
now the dilution air also burns out the rich CO and H₂, releasing their chemical energy on the way.
And **reduce-to-rung-8 is bit-for-bit** because the lean path never changed: all 58 earlier tests
stay green, and the cycle never moves — NO is still a trace diagnostic riding on top of it. What
stays **un-anchored**, stated plainly: the **finite-rate quench** (the dwell-at-stoich spike) is
the next seam, so this is "rich primary + ideal quench," not "RQL done"; **super-equilibrium O and
prompt NO** are still deferred and matter *most* in the rich primary, so our rich flank is an
equilibrium-O lower bound; and the EI band remains an order-of-magnitude landing zone, not a book
digit. The bell's **shape** is the result — and the shape is right.

---
*Rung 9 lets the two-zone `Gas.zoned_nox` primary run **rich** (φ_p up to 2.0): the same rung-7
Zeldovich integrator on a rich (CO/H₂-major) equilibrium pool, produced by a single **branched
Newton seed** in `_equil_solve` (lean branch byte-identical → reduce-to-rung-8 is provable). The
payoff is the **rich flank of the NO-vs-φ bell**: EI_NO peaks near stoich and collapses rich
(~1800× lower at φ_p=1.4), which is *why* RQL combustors burn rich then quick-quench past the peak.
NO is still trace, so every station is bit-for-bit rung 6 and the whole rung 1–8 suite stays green.
`python main.py` prints the rung-9 RQL panel: the φ_p sweep across the bell, the rich CO/H₂, the
AFT rollover, EI_NO peaking then collapsing, and T_mix returning to Tt4. Still deferred on this
substrate: the **finite-rate quench** (a secondary-zone Zeldovich — the dwell-at-stoich NO spike),
**super-equilibrium O / prompt (Fenimore) NO**, and the rung-6 **equilibrium-vs-frozen nozzle**
seam; plus off-design, the choked nozzle, the afterburner.*

---

# Rung 10 — The Finite-Rate Quench: the RQL Hazard, in plain language

## The headline: rung 9's rich win was on credit — the quench has to pay for it
Rung 9 said a rich primary is low-NOx and left it there. But it cheated on *one* thing: it froze
the NO the instant the primary was done, as if the dilution air appeared everywhere at once. Real
mixing takes time. And here is the trap — as the quench air blends in, the gas doesn't jump from
rich straight to lean; it passes **through stoichiometric**, the exact top of the NO bell. So the
rich primary that so carefully sat *off* the peak now slides **up and over it** on the way down,
and the Zeldovich clock — which never stopped — makes NO the whole time it dwells there. Rung 10
puts a stopwatch on the quench (a mixing time `τ_q`) and asks: how much of rung 9's rich win
survives?

## The smoking gun: a rich flame gets *hotter* while it's being put out
The load-bearing picture is a temperature that goes the "wrong" way. A rich primary at φ_p=1.5
burns at ~2110 K. Start diluting it and — for a while — it gets **hotter**, peaking at ~2453 K as
the local mixture crosses φ≈1.05 (slightly-rich stoichiometric, where the flame temperature
maxes), before finally falling to the mixed-out Tt4. A lean or stoichiometric primary starts at
the top and only cools; only a **rich** primary climbs through the peak. That up-and-over
excursion — plus the flood of atomic O right at stoich — is the NO spike. If the model showed a
rich primary cooling straight down, it would be wrong; the *rise* is the check.

## The result: a slow quench un-does the rich win, an order of magnitude at a time
At φ_p=1.5 the rung-9 ideal quench reads 0.0013 g/kg. Give the quench a finite time and the NO
comes back:

| quench time `τ_q` | EI_NO | vs the ideal quench |
|---|---|---|
| ideal (instant) | 0.0013 g/kg | 1× |
| 0.1 ms | 0.11 | ~80× |
| 1 ms | 1.1 | ~830× |
| 3 ms | 3.3 | ~2500× |
| 10 ms | 10.6 | ~8000× |

Three orders of magnitude, re-made purely by quenching *slowly*. This is the RQL design tension in
one column: the rich primary buys you a low-NOx start, and a lazy quench spends all of it. "Quick"
in quick-quench is not a slogan — it is the entire mechanism. Sweep the whole bell and the rich
flank that *collapsed* to zero in rung 9 gets **filled back in** to a nearly flat ~3 g/kg floor
under a 3 ms quench, because every rich mixture passes through the *same* stoich peak and dwells
there about equally. A rich primary is low-NOx **only if the quench is fast**.

## One honest surprise: the clamp we had to drop never actually fired
Rung 7's integrator capped NO at its local equilibrium — sensible when NO is climbing toward a
fixed ceiling at one temperature. On a *cooling* path that cap is a bug: as the gas chills, its
equilibrium NO drops *below* the NO already present, and that surplus is real — it freezes
(Heywood's super-equilibrium NO). A cap would delete it and hand back a too-low number with every
assertion still smugly green. So the quench integrator drops the cap; the reverse-rate form
self-limits (it runs *backwards* when NO overshoots equilibrium). The surprise: **at this engine's
lean design point the cap never would have fired anyway.** Because the overall mixture is so lean
(φ≈0.40), the cold mixed-out gas is oxygen-rich and its equilibrium NO stays *high* — above the
frozen NO — the whole way down. Measured, the NO never gets past 68% of local equilibrium
(`max_a = 0.677 < 1`) anywhere in the sweep. We dropped the cap **on principle** — it is wrong for
a cooling path, and a dormant-but-wrong assumption is exactly the kind this project exists to drag
into the light — and then *proved it dormant* with a guarded number, so if a future, hotter
operating point ever crosses into the super-equilibrium regime, the test will say so instead of
lying quietly. (Where it *will* bite: the near-stoichiometric exhaust cooling in the still-open
nozzle seam.)

## Did we get it right?
The reduce-to-rung-9 is **exact by construction**, not by luck: `τ_q=None` short-circuits to the
literal rung-9 code before any quench math runs, so every existing call — the whole rung 1–9 suite
— stays bit-for-bit, and the four new quench outputs simply read `None`. The finite quench is
opt-in and additive; the cycle never moves (NO is still a trace diagnostic). The trajectory's
K-check stays in band at *every* temperature the quench visits, now down to the cold mixed-out
~1518 K that rung 7's single-point check never saw. What stays **un-anchored**, said plainly: `τ_q`
and the linear mixing schedule are knobs (we model the *time at stoich*, not the jets-in-crossflow
that set it); **super-equilibrium O and prompt NO** are still deferred and matter *most* right here,
in the rich primary and the radical-rich stoich crossing, so even this spike is an equilibrium-O
**lower bound**; and the EI band is still an order-of-magnitude landing zone. The **shape** — the
temperature that rises while the flame is quenched, the spike that grows with dwell time, the rich
flank that re-fills — is the result, and the shape is right.

---
*Rung 10 resolves rung 9's **ideal (infinitely-fast) quench** into a **finite-rate** one: pass
`Gas.zoned_nox(..., tau_q=<seconds>)` and the NO is re-integrated (clamp-free) along a
cooling/mixing trajectory whose local fuel/air ratio sweeps `far_p → f_stoich → far_overall`, so a
rich primary's temperature rises through the stoichiometric NO-bell peak and the Zeldovich rate
re-makes NO as the gas **dwells at stoich**. EI_NO rises monotonically with `τ_q` (φ_p=1.5:
0.0013 → ~3.3 g/kg at 3 ms) and the rung-9 rich-flank collapse fills back to a ~φ_p-independent
floor — a rich primary is low-NOx **only if the quench is fast**. The fast chemistry (majors + T)
is a function of the mix fraction alone, so the trajectory is `τ_q`-independent and built once; the
equilibrium clamp is dropped (super-equilibrium NO freezes on cooling — Heywood) and proved dormant
here (`max_a = 0.677 < 1`, a guarded number). `τ_q=None` is the exact rung-9 path, so every station
is bit-for-bit rung 6 and the whole rung 1–9 suite stays green. `python main.py` prints the rung-10
finite-quench panel: the T-rises-through-the-peak, the EI_NO-vs-`τ_q` spike, and the re-filled bell.
Still deferred on this substrate: **super-equilibrium O / prompt (Fenimore) NO**, the rung-6
**equilibrium-vs-frozen nozzle** seam (where the dropped clamp earns its keep), and a **physical
mixing model** to retire the `τ_q`/linear-schedule knobs; plus off-design, the choked nozzle, the
afterburner.*

---

# Rung 11 — The Physical Mixing Model: a Jet-Entrainment Quench, in plain language

## The headline: "quick quench" was a wish — now it's a jet you can design
Rung 10 proved a rich primary is low-NOx *only if the quench is fast*, and then left "fast" as a
number you dial in (`τ_q`) with the air blended in on a straight line. But nothing in the engine
sets a quench time by decree — the dilution air comes in through **holes in the liner, as jets
punching into a hot crossflow**, and how fast those jets penetrate and stir the hot gas is what
sets the quench. Rung 11 asks the obvious next question: **what actually sets `τ_q`?** — and answers
it with the one dimensionless group that governs a jet in a crossflow, the **momentum-flux ratio**
`J = ρ_j U_j² / (ρ_c U_c²)`. A high-`J` jet is a hard, fast jet: it drives deep across the duct and
mixes the hot core out quickly. So `τ_q` stops being a knob and becomes **derived**:
`τ_q = H/(C_e·√J·U_c)`. "Quick quench" is no longer a wish — it's "**use a high-momentum jet**."

## The result: crank the jet, drop the NO — monotonically
Sweep the jet strength at the rich primary (φ_p=1.5) and the re-made NO falls straight down as the
jet gets stronger:

| momentum ratio `J` | derived `τ_q` | EI_NO | the jet |
|---|---|---|---|
| 4 | 3.3 ms | 2.1 g/kg | weak — mixes slow |
| 25 | 1.3 ms | 0.83 | — |
| 100 | 0.67 ms | 0.42 | strong (the RQL target) |

A stronger jet shortens the quench, the gas spends less time crossing the stoichiometric peak, and
less NO is re-made — a clean **monotone** "more momentum → less NO." The `τ_q` rung 10 swept by hand
is now **read straight off the jet design**, and it lands right where real RQL quench zones live:
sub-millisecond to a few milliseconds.

## The quieter result: the straight line was pessimistic — *if* entrainment decelerates
Rung 10 added the dilution air *linearly* in time. Real entrainment likely isn't linear — a fresh
jet entrains **fast** where the shear and the concentration gradient are steepest, then **slows** as
the gas mixes out. That matters because the stoichiometric crossing — where the NO is made — happens
**early**, at only ~16 % of the way through the mix. A decelerating jet blows *through* that early
crossing fast and dwells there less, so at the same `τ_q` it re-makes **less** NO. So **if
entrainment decelerates** (as gradient-collapse suggests), rung 10's straight line **over-counted**
the spike — by ~2× here. But be honest about the sign: that conclusion rides on the *shape*, which
is itself a modeling choice — an *accelerating* schedule (`n<1`) would go the other way and make
*more* NO than linear. So "rung 10 was conservative" is a *contingent* claim, not a bare fact:
contingent on decelerating entrainment, and inside a shape uncertainty (~2×) that is small next to
the orders-of-magnitude pull of `τ_q`/`J`. (The linear schedule isn't wrong so much as the
*constant-entrainment limit* — the `n=1` member of the family, which is exactly why `n=1` reduces to
rung 10 bit-for-bit.)

One tidy consequence worth naming: in `τ_q = H/(C_e·√J·U_c)`, the crossflow speed `U_c` **cancels**.
Because the jet velocity is `U_j = U_c·√(J·ρ_c/ρ_j)`, the group `√J·U_c` is really `∝ U_j`, so the
derived quench time is `τ_q ∝ H/U_j` — **duct height over jet velocity**, independent of how fast the
core is moving. So "sweep `J` at fixed `U_c`" is physically "**sweep the jet velocity**": it is the
*jet's* momentum, not the crossflow's, that sets how quickly the hot core is stirred out.

## The honest ceiling: this model can only go one way
Here is the line rung 11 deliberately does **not** cross, stated plainly: this is a **mean-field**
model. It treats the gas as one well-mixed core diluting on an average schedule, so it can only ever
say "faster jet → less NO." It **cannot** find the real dilution-jet **optimum** — the fact that an
*over*-penetrating jet is also bad, because it slams the cold air into the far wall and leaves a hot,
near-stoichiometric core sitting *un*-mixed. That's a **spatial-variance** effect (some gas dwells at
stoich far longer than the average), and a mean-field model has no variance by definition. The famous
Holdeman mixing criterion `(S/H)√J ≈ 2.5` is a *uniformity* rule, not a rate — so we deliberately do
**not** bolt it on here; dressing a mean-field model in optimum-finding clothing would be incoherent.
The optimum is real, and it is **rung 12**: give the quench at least two streams (a lingering core
plus the bulk) and the NO-vs-`J` curve will finally turn back up on the far side.

## Did we get it right?
The reduce is **exact by construction, twice over**: `mixing=None` runs the literal rung-10 code, so
the whole rung 1–10 suite stays bit-for-bit; and a jet with `shape_n=1` (constant entrainment)
reproduces rung 10's linear quench at the *derived* `τ_q` to the last bit (the schedule returns the
identity exactly at `n=1`). The cycle never moves — NO is still a trace diagnostic, opt-in via
`mixing`. What stays **un-anchored**, said plainly: the *absolute* `τ_q` rides on order-of-magnitude
choices (`C_e ~ 0.1`, the duct `H`, the bulk velocity `U_c`), so what's certified is the **√J
scaling** and the **monotone direction**, not a book quench time; the entrainment *shape* (the
`decelerating` exponent) is a residual modeling choice the NO is only mildly sensitive to (~2×),
next to the orders-of-magnitude pull of `τ_q`/`J`. And `super-equilibrium O / prompt NO` is still
deferred — the spike remains an equilibrium-O **lower bound**. The **shape** is the result: `τ_q`
falls as `1/√J`, the NO falls monotonically with jet strength, and a realistic entrainment makes
*less* NO than the straight line. That direction is right; the optimum waits for rung 12.

---
*Rung 11 replaces rung 10's two quench knobs with **jet-in-crossflow physics**: pass
`Gas.zoned_nox(..., mixing=JetMixing(J=<momentum-flux ratio>))` and the quench time is **derived**
`τ_q = H/(C_e·√J·U_c)` (a stronger jet penetrates and entrains faster → shorter quench) and the
dilution air is added on a **decelerating entrainment** schedule `β(t)=1−(1−t/τ_q)^n` instead of
rung 10's linear one. EI_NO falls **monotonically** as `J` rises — "quick quench" quantified as "a
high-momentum jet" — and *if* entrainment decelerates (gradient-collapse), it clears the early
stoich crossing faster than the linear schedule, so rung 10 over-predicted the spike by ~2× (a
claim contingent on the shape — an accelerating schedule would go the other way). `mixing` and `tau_q` are
mutually exclusive; `mixing=None` keeps the exact rung-9/10 paths (bit-for-bit rung 6), and a
`shape_n=1` jet reduces to the rung-10 linear quench to the last bit. This is a **mean-field** model —
one well-mixed core — so the `J`-sweep is monotone with **no mixing optimum**; the optimum is a
spatial-variance effect (an over-penetrating jet leaves an un-mixed hot core), the explicit **rung-12**
seam. `python main.py` prints the rung-11 jet-mixing panel: the derived-`τ_q` `J`-sweep and the
schedule-shape contrast. Still deferred on this substrate: the **unmixedness / Holdeman optimum**
(rung 12), **super-equilibrium O / prompt (Fenimore) NO**, the rung-6 **equilibrium-vs-frozen
nozzle**; plus off-design, the choked nozzle, the afterburner.*

---

# Rung 12 — Spatial Unmixedness: the Variance that Turns the Curve Back Up, in plain language

## The headline: rung 11's "stronger jet, always better" was a mean-field lie
Rung 11 crank the jet and NO fell — monotonically, forever. We said so out loud that this was a
**ceiling**, not the truth: a mean-field model (one well-mixed core) has no way to know that an
*over*-penetrating jet is *also* bad. Real dilution jets have an **optimum**. Push too little air in
and it hugs the wall, leaving a hot core it never reaches; push too *hard* and it slams across to the
far wall (or opposed jets collide in the middle), leaving a hot core the air overshot. **Both** ways,
some gas ends up in a near-stoichiometric pocket that **misses the fast jet mixing and lingers** —
dwelling right at the NO-bell peak, re-making the NO the rich primary had avoided. The mean field
averages that pocket away. Rung 12 puts it back.

## The fix: give the quench two streams, and make the CORE worse two ways off the optimum
The smallest honest model of "some gas mixes slower than the average" is **two streams**: a
mean-field **bulk** that quenches at the rung-11 jet time `τ_mean(J)` (the still-falling reference —
it depends on the jet, not on how uniform the mixing is), and an under-mixed **core** (mass fraction
`w`) that misses the jet and lingers. The total NO is the mass-weighted sum. Away from the Holdeman
optimum `C_opt≈2.5`, **both** parts of the core get worse:
**more** gas ends up segregated (the fraction `w(C)` rises) and it **lingers longer** (the core dwell
`τ_core(C)` grows). At `C_opt` the jets tile the cross-section perfectly, `w=0`, and the two-stream
total sits exactly on the mean-field curve.

Two things had to be right, and the naïve versions get **both backwards**:

- *How the core lingers.* The tempting move is a "spread" of quench times around the mean (convexity
  would then add NO). But in this regime NO piles up as **rate × dwell**, so `EI ∝ τ_q` (the rung-10
  table: `EI/τ_q` is flat-to-falling, *not* convex) — a mean-preserving spread adds **nothing, or
  the wrong sign**. And if the core's dwell were a *multiple* of the jet time, it would **vanish** as
  the jet time does (`∝1/√J → 0`) and the curve would stay monotone. The core must dwell on an
  **absolute** clock — the dilution-zone residence — so its penalty stays finite (and *grows*) as
  the jet strengthens. That is what lets the total turn back up.
- *Where the bottom lands.* If the unmixedness rose **smoothly** (a parabola) from the optimum, the
  bottom would drift to a **stronger** jet than `C_opt` — just past the optimum the mean-field bulk
  is still getting cleaner faster than the little core penalty grows. To pin the minimum **at** the
  Holdeman optimum, the unmixedness has a **kink** at `C_opt` (it grows like `|ln(C/C_opt)|`, not its
  square). The kink gives the segregated fraction a real slope right at `C_opt`, so the moment you
  leave the optimum the core penalty out-runs the penetration benefit and the curve turns up. In
  plain terms: the kink *is* the statement that an emissions optimum **exists** at the uniformity
  point — if penetration always won, there'd be no bottom at all.

## The result: the NO-vs-J curve finally has a bottom — and it sits AT the Holdeman optimum
At the design point, with a rich `φ_p=1.5` primary and the default unmixedness (`S=0.0625 m`,
`τ_res=2.5 ms`), EI_NO falls from ~2.4 g/kg at a weak jet (`J=4`) down to a **minimum ~1.0 at `J=16`**
— exactly the Holdeman uniformity optimum `C=(S/H)√J=C_opt=2.5` — then **climbs to ~2.2** at `J=100`,
even as the mean-field bulk keeps falling to 0.42. The un-mixed core, which lingers *longer* the
further you push from the optimum, is what lifts the total. That is the recovered **Holdeman
optimum**: a bottom at `C_opt`, with both flanks going up. Be square about what's *earned* here: we
*placed* the kink at Holdeman's empirical `C_opt≈2.5`, so the model **reproduces** an emissions
optimum there rather than **predicting** 2.5 from first principles — a calibrated result at this
tower's altitude (like `C_e`, `τ_q`). What the model genuinely delivers is the *shape*: a two-sided
turn-up that a mean field cannot make, and a min that rides the Holdeman group as `(H/S)²`.

And it is genuinely the Holdeman *group*, not a bare dip in `J`. Shrink the jet spacing `S` (so you
need a deeper-penetrating, stronger jet for the same uniformity) and the whole optimum **moves to
higher `J`** — landing on `J_opt=(C_opt·H/S)²` for every spacing, so it shifts **exactly as `(H/S)²`**
(16 → 25 when `S` goes 0.0625 → 0.05 m). That shift is the fingerprint: the min lives on the Holdeman
group, not on `J` alone.

## Did we get it right?
The reduce is **exact by construction**: `unmixedness=None` runs the literal rung-11 code (the whole
rung 1–11 suite stays bit-for-bit), and switching the variance off (`k_u=0` ⇒ `w≡0`) collapses the
two-stream total back onto the mean-field bulk at *every* `J`, to the last bit. At a jet whose group
is exactly `C_opt`, `w=0`, so the optimum point sits *precisely* on the rung-11 curve — a clean seam.
The cycle never moves (NO is still a trace diagnostic, opt-in via `unmixedness`), and the clamp we
dropped in rung 10 stays dormant on the core too (`max_a≈0.04–0.07≪1`). What stays **un-anchored**,
said plainly: the absolute knobs (`S`, `τ_res`, `k_u`, `b_u`, `w_max`) are order-of-magnitude, like
`C_e`/`τ_q` before them; the *pin* at `C_opt` is a modeling choice (the kink), disclosed — and the
total EI is *not* a function of `C` alone (the bulk rides `J`), it's the optimum *location* that
lands on the group. What's certified is the **turn-up**, the **minimum at `C_opt`**, and its
**`(H/S)²` shift** — not a book NO. This is a **two-stream** closure — the minimal variance model —
not a resolved mixing PDF; that (and super-equilibrium/prompt NO, which lives in exactly this
under-mixed core) is the rung-13 seam.

---
*Rung 12 adds the **spatial-variance** layer rung 11 deferred: pass `Gas.zoned_nox(...,
mixing=JetMixing(J=...), unmixedness=Unmixedness(S=<jet spacing>))` and the quench splits into a
mean-field **bulk** (quenched at the rung-11 jet time `τ_mean∝1/√J`, the still-falling reference)
plus an under-mixed **core** (fraction `w(C)=min(w_max, k_u·|ln(C/C_opt)|)`, quenched at an
**absolute** dwell `τ_core(C)=τ_res·(1+b_u·|ln(C/C_opt)|)`; the bulk stays the mean-field reference).
The CORE worsens **two ways** away from the **Holdeman group** `C=(S/H)√J` — its fraction AND its
dwell both grow off `C_opt≈2.5` — and the **kinked** unmixedness pins the bottom AT `C_opt`. So `ei_no_unmixed=(1−w)·EI(τ_mean)+w·EI(τ_core)` **falls to a
minimum at `C_opt` then rises**: the recovered **Holdeman dilution-jet optimum**, `J_min=J_opt`
**shifting as `(H/S)²`** with the spacing. The core dwell is an **absolute** clock (not the vanishing
jet time), which is why the turn-up survives strong jets. `unmixedness` **requires** `mixing`;
`unmixedness=None` keeps the exact rung-11 mean field (bit-for-bit rung 6), and `k_u=0` reduces to the
mean-field bulk to the last bit. `python main.py` prints the rung-12 panel: the turn-up `J`-sweep
(bulk vs two-stream) and the `(H/S)²` optimum shift. Still deferred on this substrate: a **resolved
mixing PDF** (>2 streams), **super-equilibrium O / prompt (Fenimore) NO** (richest in this under-mixed
core), the rung-6 **equilibrium-vs-frozen nozzle**; plus off-design, the choked nozzle, the afterburner.*

---

# Rung 13 — The Resolved Mixing PDF: Composition Variance, Isolated from Dwell, in plain language

## The headline: replace the two hand-tuned lumps with the whole distribution — and learn something
Rung 12 modelled "some gas mixes slower than average" with the crudest honest thing: **two lumps**, a
bulk and a core, with the segregated fraction `w(C)` and the core dwell tuned by hand. The obvious
next move is to stop lumping and resolve the **whole distribution** of local mixture: a continuous
**β-PDF** of mixture fraction, the standard object turbulent-combustion models use, fixed by its mean
and **one** width — the **segregation** `g`. Ride that width on the same Holdeman kink `g(C)`,
integrate the ideal NO bell over the distribution, and see what a *continuous* variance says. What it
says turns out to be sharper than "rung 12 with a PDF" — it **separates two mechanisms** that rung 12
had bundled.

## The lesson, said correctly: it is NOT "the bell is convex, so Jensen"
It is tempting to say "average-of-EI beats EI-of-average because EI is convex." **That's wrong**: the
NO-vs-φ bell is convex on its flanks but **concave at its peak** — there is no global convexity to
lean on. The true statement is more specific and more useful: **NO is sharply peaked at stoich, so
spreading the local mixture around a fixed mean raises the average NO whenever that mean is OFF the
peak** — the stoich-ward tail of the spread reaches up onto the peak while the mean itself sits low in
a wing. Our combustor mean is **lean** (the dilution zone conserves mass, so the mean mixture is just
the overall lean value), so segregation **raises** NO. The tell that this framing is the right one:
put the mean **at stoich** and the sign **reverses** — now spreading moves mass *off* the peak and
*lowers* the average. A blind "convexity always raises it" claim would get that backwards. The panel
prints both columns side by side so you can watch the sign flip.

## The result: a sharp optimum pinned AT C_opt — but the over-penetration climb is GONE
Sweep the jet. At the Holdeman optimum `C_opt` the mixing is perfect (`g=0`), the distribution
collapses to a spike at the lean mean, and the NO is the **well-mixed lean value ≈ 0** — a sharp
**notch**. Step to *either* side and segregation lifts the average by **four to five orders of
magnitude** (the convexity jump, localized). That notch, pinned at `C_opt` and shifting as `(H/S)²`
with the jet spacing, is the recovered Holdeman optimum **location** — now from a continuous PDF, no
lumps.

But — and this is the honest surprise — the curve does **not** climb back up on the far
over-penetration flank the way rung 12's did. Past a point it **descends** again. That is real
physics, not a bug: as the segregation grows large the β-PDF goes **bimodal**, piling its mass at
pure air (`ξ→0`) and rich (both *off* the stoich peak), so the average NO falls. So ⟨EI⟩ vs the
segregation is **humped** — it peaks at *moderate* unmixedness, not extreme.

## Why the climb is missing — and why that's the point (two mechanisms, separated)
Rung 12's over-penetration climb came from the **dwell** effect: an under-mixed pocket that lingers on
an **absolute** clock, re-making NO the longer it sits — a **time** mechanism, and rung 12 built it in
deliberately (the absolute `τ_core`). Rung 13 isolates the **composition** mechanism — the mixture
variance — and **drops the quench chain entirely** (the bell it integrates is the *ideal*,
instantly-quenched primary). With no dwell, there is nothing to make the climb; composition variance
can only pin the optimum *location* and lift the immediate flanks. So the two rungs are telling us
two different truths:

- **Composition variance** (rung 13) says *where* the optimum is — at `C_opt`, both flanks up.
- **Dwell** (rung 12) says *why over-penetration keeps getting worse* — the stranded pocket lingers.

Bundling them — carrying the resolved PDF **through** the finite quench so the distribution both
samples the peak *and* dwells — is exactly the next rung (14). That the composition-only model can't
climb isn't a weakness; it's the model telling us the climb was never a composition effect.

## Did we get it right?
Two exact reduces, and no false third one. `pdf=None` runs the literal rung-12 code (the whole rung
1–12 suite stays bit-for-bit; the cycle never moves — NO is still a trace diagnostic, opt-in via
`pdf`). And `g→0` collapses the PDF to a spike, giving the well-mixed point value exactly — the value
the notch pins to at `C_opt`. We **do not** claim a bit-for-bit reduce to the two-stream model: a
continuous PDF is a *different* closure, not two lumps plus a knob. The one piece of real numerical
care is **mean-preservation**: a presumed β-PDF exists to hold the mean fixed while the variance
varies, but a lean mean makes the density blow up (integrably) at `ξ→0`, and a naïve grid quietly
integrates at the *wrong* mean (a 35–95% error in the one number that must be exact). A change of
variable `u=ξ^a` cancels the singularity so the mean is preserved to machine precision — and the code
**asserts** `⟨ξ⟩≈ξ̄` on every call. That assertion is the deliverable as much as the number is. What
stays un-anchored, said plainly: `S`, `k_g`, `g_max` are order-of-magnitude, and the β shape is
**presumed**, not transported — its width `g(C)` is still modelled on the Holdeman group, not solved
from a mixing equation. What's certified is the optimum pinned at `C_opt` (both flanks up), the
`(H/S)²` shift, the humped variance response, and the sign of the effect (and its stoich-mean
reversal) — not a book NO.

---
*Rung 13 replaces rung-12's parameterised **segregation** with a continuous **β-PDF of mixture
fraction**: pass `Gas.zoned_nox(..., mixing=JetMixing(J=...), pdf=MixingPDF(S=<jet spacing>))` and
`ei_no_pdf = ∫ EI_bell(φ(ξ))·P_β(ξ; ξ̄, g(C)) dξ` integrates the **ideal** primary NO bell over the
distribution, its width the segregation `g(C)=min(g_max, k_g·|ln(C/C_opt)|)` — the same kinked
Holdeman distance as rung-12's `w`. The lesson framed right: NO is **peaked at stoich**, so
segregation **raises** the mean whenever the mean is **off-stoich** (our lean dilution mean),
**reversing** at a stoich mean — not generic convexity. The emissions minimum pins **AT `C_opt`** (a
sharp notch → the well-mixed lean value ≈0), both immediate flanks lifting by orders, `J_min=J_opt`
**shifting as `(H/S)²`**. A **mechanism separation**: this isolates the **composition** mechanism and
drops the dwell chain, so it pins the optimum but **cannot climb** — the far over-penetration flank
**descends** (⟨EI⟩(g) is humped: the β-PDF goes bimodal). Rung-12's over-penetration climb was the
**dwell** effect; combining them (the PDF through the quench) is the **rung-14** seam. `pdf` is
**mutually exclusive** with `unmixedness` and **requires** `mixing`; `pdf=None` keeps the exact
rung-12 path (bit-for-bit rung 6), and `g→0` gives the well-mixed point value. The quadrature is
mean-preserving (a `u=ξ^a` change of variable across the lean-mean singularity) and **asserts**
`⟨ξ⟩≈ξ̄` every run. `python main.py` prints the rung-13 panel: the peaked×off-mean mechanism (lean vs
stoich columns, the sign flip), and the `J`-sweep (notch at `C_opt`, flanks up, the humped far flank).
Still deferred on this substrate: the **PDF through the finite quench** (rung-14 — where composition
and dwell combine and the ≈0 floor becomes finite bulk NO), a **transported/CFD PDF**,
**super-equilibrium O / prompt (Fenimore) NO** (richest in exactly the near-stoich pockets this PDF
resolves), the rung-6 **equilibrium-vs-frozen nozzle**; plus off-design, the choked nozzle, the afterburner.*
