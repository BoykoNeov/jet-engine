# Rungs 95–101 — Asking for thrust: governors, and a control that cannot weigh its own thrust

*Plain-language notes. The formal derivations, tests and every number's source are in `docs/rung95-spec.md`
to `docs/rung101-spec.md`. Numbers below are copied from those specs and belong to the cases the specs ran; the
sandbox page shows its own.*

A pilot does not want a turbine temperature. A pilot wants **thrust**. Until rung 94, every throttle move in the
model set a burner temperature (or a fuel flow) at each end and ramped the fuel between them. These seven rungs
put a **governor** in charge: a controller that keeps adjusting the fuel, all through the move, to deliver the
thrust (or shaft speed) asked for. Then they take away its cheat. An engine in flight cannot weigh its own
thrust; its control has to work it out from something it can measure, using tables of steady running points made
in advance. Each rung after 95 makes that working-out more realistic, and each finds what the previous one could
not see.

**From earlier rungs.** The engine has two shafts (rung 38): the fan, or low-pressure (LP) shaft, and the
high-pressure (HP) shaft. Time is in shaft response times, **τ** (see the rungs 34–37 note). Distance from stall
is the flow coefficient (rung 36).

---

## Rung 95 — A governor in the slam

**In one sentence:** a governor that holds thrust along a fast throttle move makes the slam **hotter**, not
gentler, because the only way to get thrust from shafts that have not yet sped up is more fuel.

**How it works.** The throttle now asks for a thrust (or an HP shaft speed, or a turbine temperature) at every
instant of the move, and the governor finds the fuel that gives it. The fuel limiters of rungs 46–52 still cut
it when they must.

**What it found.** On a steady engine, more thrust comes partly from more fuel and partly from the shafts running
faster. In a fast move the shafts have not caught up yet, so to raise thrust on schedule the governor must burn
more fuel than the plain ramp at every moment of an acceleration (and less at every moment of a slowdown). On the
spec's default slam (1000 → 1400 K in half a shaft response time) the turbine inlet peaks near **1 930 K**
against the plain ramp's ~1 700 K, and both compressors come nearer stall. In exchange the thrust arrives: 95 % of
the change at **0.48 τ** against the ramp's **1.39 τ**. With the 1480 K temperature limiter on, which then holds
the fuel for most of the move, it still gets there about **18 %** sooner.

**A second finding, about holding shaft speed.** A governor that closes the speed gap in proportion to its size
(a fixed "gain") is about **1.9×** slower at idle than near full power: the turbine has less power to spare there.
One that commands a rate of speed-up closes it at the same pace everywhere.

**Later.** Rung 95 found that nothing in the governor itself stores up a correction to pay back later. Rung 106
bounded that to the governor alone: beside a limiter that reacts with a delay, the pair does store one, and the
thrust sags after the move (see the rungs 106–108 note).

**On the page.** *Controls → Holds thrust* (or *Holds HP speed*) and *Govern during the move*. *Try this → Ask
for thrust, not temperature*. In the page's slam (`docs/sandbox/EXPERIMENTS.md`, experiment 16, a 0.1 τ move) the
thrust reaches 90 % of its change in 0.14 τ instead of 0.86 τ, and the turbine inlet peaks at 2594 K instead of
1943 K. *Thrust asked for, beside the limiter* adds the temperature limiter: rungs 106–108 study that pair.

---

## Rung 96 — The control works thrust out from a reading

**In one sentence:** an accelerating engine is a loaded engine, so a control that reads thrust from the pressure
ratio or the fan speed asks for more thrust than it believes it is getting.

**How it works.** Real engines are controlled on one of two readings: the **pressure ratio** across the engine
(turbine exit over intake), or the **fan speed** (the LP shaft). A table of steady running points turns the
reading into a thrust.

**What it found.** While both nozzles are choked, the pressure ratio reads the jet's push *exactly*. What it
cannot see is the drag of the air the engine swallows. At a given pressure ratio, an engine whose shafts also need
power to spare, to speed up or to drive a generator, burns hotter and passes less air than the steady table says,
so it gives more thrust than the control believes. On the spec's default move that is up to **12 %** of the thrust
change more (and 16 % less on a slowdown); the run is hotter than rung 95's, and both compressors come nearer
stall. The fan speed misses by far more, **26–46 %** of the change accelerating and **32–59 %** slowing, because
the fan shaft is slow to follow the fuel.

With a generator on and the table made on the engine without it, even the steady ends miss: the pressure ratio by
0.35–1.25 %, the fan speed by −9 % to +26 %, its sign set by which shaft carries the generator.

**On the page.** Under *Govern during the move*: *Thrust read directly / From the pressure ratio / From the fan
speed*. The thrust chart draws what the control believes beside what the engine gives.

---

## Rung 97 — The exhaust nozzle unchokes on the two-shaft engine

**In one sentence:** at a low throttle on a hot day the exhaust nozzle stops being choked, the outside air pushes
back on the LP turbine, and the LP shaft runs slower.

**How it works.** Rung 33 did this on one shaft. Until rung 97 the two-shaft engine had simply assumed its nozzle
choked. Here the LP turbine may expand the gas only as far as still leaves the nozzle enough pressure to push it
out.

**What it found.** On a day 40 K hotter than standard, at sea level and a 700 K throttle, the LP shaft runs
**1.6–1.8 %** slower than the old model said, the HP shaft only 0.02–0.03 % slower, and the engine gives
**3.6–4.8 %** *more* thrust: the jet leaves faster from the higher pressure behind the turbine. On a standard day
the 700 K point is barely unchoked and every difference is about a hundred times smaller.

**Later correction.** Rung 97 said the back-pressure slows the LP shaft *alone*. Rung 104 found that holds only
under rung 38's assumption that the LP turbine's entrance is choked. It is not, and below choke the back-pressure
slows **both** shafts (see the rungs 103–105 note).

**On the page.** The *Controls* charts draw the run thin and dashed where the nozzle is unchoked and full where it
chokes.

---

## Rung 98 — Tables made once, read everywhere: the corrections

**In one sentence:** a control's tables are made at one flight and corrected for the air's pressure and
temperature elsewhere, and those corrections undo exactly what changes the engine's inside, but a change of
flight *speed* changes only the thrust, which no correction reaches.

**How it works.** The standard trick divides thrust and fuel by how much denser the air at the engine is, and
speeds by the square root of how much warmer it is. Done that way, one table serves any altitude and day.

**What it found.** The pressure correction is exact on this model and the temperature one nearly so (the pressure
ratio's thrust is off by under 0.1 %, the fan speed's by under 0.3 %). Each is needed where it applies: without
the pressure one the pressure-ratio control is off by 47 % in air at 0.53 of the design's pressure. But change
the flight Mach number at the same air entering the engine, and the engine's inside is identical to the last
digit, while the jet's push against the outside air and the drag of the air swallowed both change. A table made
at Mach 0.85 gives **8.4–9.1 %** too much thrust at Mach 0.5 and **4.6–9.7 %** too little at Mach 1.4. That is
why real schedules carry a Mach number axis.

**On the page.** *Control's tables made at the design flight / Made at this flight*, *Corrected for the air's
pressure*, *Corrected for the air's temperature*. *Try this → The same slam at sea level* moves the flight.

---

## Rung 99 — The speed axis

**In one sentence:** tables at several flight speeds, blended, leave only the **bend** of thrust against speed;
recomputing the thrust for the speed leaves only the guess of how much air the engine swallows.

**How it works.** Two remedies for rung 98's Mach miss. One: make the tables at a range of flight speeds and blend
the two around the speed flown. Two: from the pressure-ratio reading, work the jet's push and the intake drag out
afresh for this flight speed, using the table only for the air swallowed.

**What it found.** Blending lands a little short where the thrust-against-speed curve bends up (Mach 0.5:
−0.27 to −0.54 %) and a little over where the nozzle bends it down (Mach 1.4 at a low throttle: +0.29 to +0.32 %),
against rung 98's 4.6–9.7 %. Recomputing misses by 0.02 % at Mach 0.5 and 0.18 % at Mach 1.4, but where the
nozzle is not choked it gives too little thrust read from the pressure ratio (−1.0 to −1.4 %) and too much read
from the fan speed (+4.4 to +5.1 %).

**On the page.** *Speed: tables at ten speeds / Thrust recomputed / Not corrected*.

---

## Rung 100 — Below choke, the table's one line is not enough

**In one sentence:** a table made where the nozzle chokes describes the engine by one number, its throttle; below
choke the engine settles somewhere else, and only a second number, the flight speed, brings it back.

**How it works.** Unchoked, the outside air pushes back on the LP turbine (rung 97). At a given burner temperature
the fan turns slower and feeds the core about **1.4 %** less pressure; at a given fan speed the engine burns
hotter. The jet's push is still exact from the pressure ratio and the outside air, choked or not. What goes wrong
is the air swallowed, read off a line the engine has left.

**What it found.** Two extra tables made at slow flight speeds, where the nozzle unchokes, take the worst miss below
Mach 0.3 from **16 %** (pressure ratio) and **33 %** (fan speed) to **under 1 %** on both readings.

**On the page.** Rung 99 made its tables at eight flight speeds; rung 100 added the two slow ones (Mach 0.05 and
0.15), which is why the page offers *tables at ten speeds*.

---

## Rung 101 — The air swallowed, off the line

**In one sentence:** no steady table can see an accelerating engine, but a temperature sensor in the exhaust can,
until it is slow.

**How it works.** Rung 100's remaining error was the air the control believes the engine swallows. Two ways to get
it: from the speed tables, or from a sensor measuring the exhaust temperature, which with the nozzle's pressure and
area gives the nozzle's own flow, less the fuel the control meters and any air it dumps.

**What it found.**
- **From the tables,** the steady miss falls almost to nothing (Mach 1.4 from 700 K: −1.7 % to −0.01 %), but the
  error during the move does not change at all (+39 % either way, the spec's case): at a given pressure ratio an
  accelerating engine passes less air than any steady table says (rung 96).
- **From an instant exhaust sensor,** read with the pressure ratio, the control knows the **true thrust** at every
  moment and governs exactly as rung 95's governor that read thrust directly.
- **A slow sensor gives it back.** A real probe lags the gas, which heats as the engine accelerates. The governor
  still holds what it *believes*, so the error moves into the thrust delivered: **+48 %** at Mach 1.4 with a probe
  as slow as one shaft response time, worse than the tables.

**On the page.** With *Thrust recomputed*: *Air swallowed: from the table / From the speed tables / From an
exhaust sensor*, and *The sensor responds slowly* with its response time.
