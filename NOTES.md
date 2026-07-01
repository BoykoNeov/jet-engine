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
