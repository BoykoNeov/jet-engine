# Rungs 34–37 — The engine in time: one shaft speeding up and slowing down

*Plain-language notes. The formal derivations, tests and every number's source are in `docs/rung34-spec.md`
to `docs/rung37-spec.md`. Numbers below are copied from those specs and belong to the cases the specs ran; the
sandbox page shows its own.*

Rungs 31–33 found where a built engine runs **when it has settled**. But an engine never jumps from one
setting to another: the shaft is a heavy spinning rotor, and it takes time to speed up. These four rungs put
**time** into the model. What happens *between* two steady points turns out to matter more than the points
themselves: that is where engines overheat and where compressors stall.

A word on the clock. The model knows the engine's power but not the weight of its rotor, so it cannot say how
many seconds a change takes. It measures time instead in **shaft response times**: the time the shaft's own
inertia takes to respond to a power imbalance. The page writes this unit as **τ** (the Greek letter tau).
"0.5 τ" means half a shaft response time. (Roadmap item B2 would turn it into seconds.)

---

## Rung 34 — The shaft speed becomes a state

**In one sentence:** when the throttle moves, the shaft is out of balance for a while, and how far the engine
strays from its running line depends on one ratio — how fast the fuel changes compared with how fast the shaft
can follow.

**How it works.** In a steady engine the turbine's power exactly pays for the compressor. Move the throttle and
that balance breaks: the extra (or missing) power speeds up (or slows down) the rotor. Everything else in the
engine — the gas through the passages, the flame — settles in milliseconds, far faster than the rotor. So at
each instant the model solves the gas path as if it were steady, *at the shaft speed the rotor has right now*,
and lets the power imbalance move the shaft speed forward in time. The shaft speed has become the engine's
memory.

**The finding is a ratio.** It would be tempting to report "the shape of the response does not depend on the
rotor's weight; the weight only sets the clock". That is true but empty: it is what measuring time in shaft
response times means. The real content appears when there is a *second* clock: the time the fuel change takes.
- If the fuel changes much faster than the shaft can follow, the shaft is effectively frozen while the fuel
  arrives. The compressor is pushed up its map at constant speed, far above its running line, toward stall.
  This is the biggest excursion possible.
- If the fuel changes much more slowly, the shaft keeps up and the engine slides along its running line.
- The change-over is around a ratio of 1: fuel time equal to shaft time.

That is *why real engines schedule their fuel*: slowing the fuel keeps the compressor away from stall.

**Spooling down.** Cut the fuel and the shaft slows; the nozzle unchokes on the way (rung 33) and the engine
heads toward idle.

**Later corrections.** Rung 35 showed that rung 34's way of driving the engine (commanding the turbine
temperature directly) underestimated how far toward stall a real engine goes. Rung 37 showed that one of rung
34's simplifications, "every other part is fast", holds for one effect and fails for another.

**On the page.** *Fly it → Slam the throttle* moves the throttle over the time you set and draws the engine
following. *Try this → Slam the throttle faster* (experiment 10 in `docs/sandbox/EXPERIMENTS.md`).

---

## Rung 35 — Fuel is the control, so the temperature overshoots

**In one sentence:** a real throttle meters fuel, not temperature; fuel arrives at once but the air the shaft
pumps arrives late, so the turbine gets hotter than intended, and that heat pushes the compressor further toward
stall.

**How it works.** Rung 34 commanded the turbine inlet temperature directly, as textbooks do. No real engine can:
it can only set the fuel flow. The temperature then follows from how much air there is to burn the fuel in, and
that air depends on the shaft speed, which lags. So right after a fuel increase there is full fuel in
part-speed airflow, and the gas burns hotter than the new setting. Worse, the turbine's entrance is a choked
throat, and a hotter gas passes less of itself through a choked throat: the airflow is choked back further,
which pushes the compressor further up its map toward stall. The two classic limits on accelerating an engine —
**stall** and **turbine temperature** — are coupled, not separate.

**Numbers from the spec's case** (a sudden fuel step from the fuel of the 1100 K point to that of the 1400 K
point, on one of the test maps, fast gas model): the compressor's excursion toward stall is **10.16 %** with
fuel control against **5.39 %** with the temperature commanded — rung 34 had under-counted it. The turbine
inlet peaks at **1754 K**, **+25 %** over the 1400 K target. Slow the fuel to three shaft response times and the
difference between the two ways of driving the engine almost vanishes (**0.11 %**). The *direction* held on every
map shape tried; the sizes depend on the invented map.

**On the page.** *Fly it → Slam the throttle* runs the same move both ways side by side, temperature commanded
and fuel metered; the fuel-metered line overshoots. Rungs 46–52 (the fuel limiters, on *Controls*) exist to stop
this overshoot.

**Later.** Rung 46 sharpened "the two limits are coupled": they share a cause but arrive one after the other.
So a limiter that holds the turbine temperature does *not* relieve the compressor that would stall first (see the
rungs 46–52 note).

---

## Rung 36 — The stall line: the margin is thinnest at low power

**In one sentence:** measured against a stall line, the compressor's safety margin is **thinnest at low power**
and widest at full power — so a slam from low power is the dangerous one on both counts.

**How it works.** A compressor stalls when its airflow gets too small for its speed: the blades meet the air at
too steep an angle, like a wing at too high an angle of attack. The model has no measured compressor, so the
stall line is drawn where you say: a minimum ratio of airflow to blade speed, the **flow coefficient**. The
*margin* is how much more pressure the compressor could make at its present speed before reaching that line. The
model's hope of finding the stall point from its own map failed (the invented maps put it at impossible,
negative flows), so this one number is imposed. The rung therefore claims only what does not depend on where you
put it.

What survives is the **trend**: whatever stall line you choose, the margin shrinks as you throttle back. The
fixed hardware decides how the running line approaches the stall line, and the stall line's position cannot
change that direction. And rung 34 had already found that a slam from low power makes the biggest excursion. The
two point the same way: the low-power slam is the most dangerous, on both counts. That is why real engines carry
bleed valves and variable stators for starting and low power.

**What is not claimed: whether a given slam actually stalls.** That depends entirely on the imposed stall line.
In the spec's example, a slam from 700 K uses 0.098 of margin; with the stall line at 0.55 the margin there is
0.109 (no stall), with it at 0.65 the margin is 0.073 (stall). So the page labels its stall margin *trend only*.

**Later corrections.**
- **Rung 41 corrected the reason, not the verdict.** Rung 36 said the margin shrinks because the running line
  walks toward the stall line. Rung 41 found two comparable causes: that walk, *and* the compressor's speed lines
  flattening at low speed. Below a certain pressure ratio the walk even reverses while the flattening keeps
  eating margin. The conclusion, thin at low power whatever the stall line, stands.
- **Rung 53 bounded it.** A margin is a *distance* to the stall line, so it depends on how you measure distance.
  Once a lever can move the stall line itself (variable stators), margins in different measures stop agreeing.
- **Rung 55 bounded it again.** The single-block compressor stands for its *front* stage; once the compressor is
  split into stages, these margins turn out to describe the front stage only.

**On the page.** *Fly it*'s *Stall margin* tile and the stall line on the map (you set it). *Try this → Throttle
back to 900 K* shows the margin shrinking (experiment 9).

---

## Rung 37 — Two more clocks: one confirms, one corrects

**In one sentence:** rung 34 assumed everything except the shaft is fast; the burner filling with gas really is
fast and changes nothing that matters, but the metal soaking up heat is as slow as the shaft and changes the
answer.

**The burner as a tank.** The burner is a volume between the compressor and the turbine. Gas can pile up in it
for a moment, so the compressor and the turbine need not pass the same flow at every instant. During a fill the
two differ by up to about **22 %**. But the fill takes about 5–30 ms, far shorter than the shaft, so it fills
before the shaft can move, and the peak excursion toward stall is exactly rung 35's (difference **−0.0000 %**).
Rung 34's simplification holds here.

**The metal as a sponge.** The burner and turbine metal take up heat from the gas. Its own time scale is about
1–20 s, about as slow as the shaft, so it is not a fast effect at all. It adds a second memory: the metal's
temperature. Now the response depends on what the engine was doing before, not just on the throttle move.
- **The main cost is time.** Cold metal steals heat the turbine would have used, so a cold acceleration is about
  **2.5×** slower in the spec's case (**5.55** shaft response times against **2.15** with no heat soak).
- **For stall, cold metal helps here.** Cooler gas passes more easily through the turbine's choked entrance, so
  the excursion toward stall shrinks: on one map, **9.224 %** cold, **9.660 %** for a re-slam with hot metal,
  **10.156 %** with no heat soak. So rung 35's no-soak answer was the worst case.
- **An honest warning.** Real engines know the opposite effect: a re-slam with a hot engine can be *more*
  stall-prone. That comes from heat soaking into the *compressor* and changing its blade tip gaps, which this
  rung does not model. What this rung shares with it is only that the past matters.

**On the page.** Not shown: no view runs these two clocks. Running them on the two-shaft engine, and both
together, is still open.
