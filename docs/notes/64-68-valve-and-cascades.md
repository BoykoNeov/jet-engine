# Rungs 64–68 — The valve that decides for itself, and loops that share a job

*Plain-language notes. The formal derivations, tests and every number's source are in `docs/rung64-spec.md`
to `docs/rung68-spec.md`. Numbers below are copied from those specs and belong to the cases the specs ran; the
sandbox page shows its own.*

Up to rung 63 the bleed valve was told what to do: held at one opening (rung 42) or moved by a schedule on shaft
speed (rung 62). These five rungs give it a **feedback loop**: it watches the low-pressure compressor's distance
from stall and opens just enough to keep it at a target. Then they ask what happens when more than one controller
works on the same job.

**Three things to know first.**
- **A loop, a floor, a target.** A feedback loop watches a reading (here the low-pressure compressor's flow
  coefficient, rung 36) and acts to keep it at or above a set value. The fuel-side version is rung 49's stall
  floor; this is the same idea on the bleed valve.
- **A lag.** A real valve takes time to move. The model gives it one number, how long it takes to close most of
  the gap to where it is told to be, in shaft response times (written **τ** on the page).
- **What these rungs are.** From rung 66 on the questions are about how control loops combine, more than about
  the engine. The results are exact and general; the roadmap notes this drift away from what a user sees.

---

## Rung 64 — A smarter rule cannot protect more, only more cheaply

**In one sentence:** a valve that watches the stall margin and opens just enough cannot protect the compressor
any better than simply holding the valve wide open, because the limit is how far the valve can open; what the
smarter rule buys is a much smaller bill.

**The limit.** The most protection any valve rule can give is what the valve gives held fully open for the whole
move. That depends on how big the valve is, which is hardware, not on the rule. Set the target above it (in the
spec's case, 10 % above) and the valve opens all the way and still misses its target.

**The bill.** Hold the compressor's lowest flow coefficient at exactly the same value three ways: a fixed opening,
rung 62's speed schedule, and this feedback valve. The feedback valve bleeds about half the air of the schedule and
a quarter of the fixed opening's, and because it shuts itself once the danger passes, it costs no thrust by the end
of the move (+0.028 % against the schedule's −0.464 %).

**Two side effects.**
- **A second controller on the same reading loses its grip.** Put rung 49's fuel floor on the same reading: while
  the valve is holding it, more or less fuel no longer changes the reading at all, so the fuel floor's sum has no
  single answer.
- **The worst moment has no single place.** A loop holding a reading exactly at its target holds it there over a
  stretch of the move, so "when was the margin lowest" no longer has an answer. That bounds rungs 44–52, whose
  findings are about where the worst moment falls; their own results stand, since their limiters were not holding
  where they read.

**What it does to its neighbours.** It turns rung 61 around from the other side: rung 61 compared two levers and
found the bill stays; this compares rules for one lever and finds only the bill moves. It bounds the fuel limiters
of rungs 46–52 on a third side: rung 53 bounded how they measure, rung 57 their timing, this their ceiling.

**Later.**
- **Rung 87:** "the ceiling is the fully-open valve" is a bound that a valve opening late can fall below, and beside
  the stators the ceiling depends on which ruler the valve watches.
- **Rung 90:** air taken for the cabin lowers the ceiling.

**On the page.** *Controls → Airflow levers → Lever → Handling bleed valve (rungs 62 and 64) → Opens on its own*,
with *Holds the LP at* (a flow coefficient; the stall line is 0.55). The page's own warning is this rung's side
effect: set a stall floor a little below the valve, because at the same number the two share the work in a way the
model's rounding decides.

---

## Rung 65 — A slow valve: worse protection and more air

**In one sentence:** give the valve a realistic delay and it both protects less and bleeds more, since it opens late
and closes late; and two loops holding one reading still share the job in an undecided way, the indecision just
moves from the solver into the valve's own position.

**What it found.**
- **Delay is pure loss.** Over delays from 0.01 to 0.4 τ, the compressor's lowest flow coefficient falls short of
  the target, by about ten times more at the slowest than at the fastest, while the air bled rises. It opens late and misses the worst moment; it closes late
  and keeps dumping air after the need has passed.
- **The worst moment has a place again.** A delayed valve cannot hold the reading flat, so rung 64's lost "when"
  comes back.
- **The shared job stays undecided.** With the fuel floor and the delayed valve both holding the same reading, the
  valve's position freezes wherever it started. The fuel cut comes out identical to seven digits at delays of
  0.01, 0.05 and 0.2 τ: the delay cannot reach the split of the work, which only the starting state decides.

**Later corrections.**
- **Rung 66:** the valve freezing is not the root fact; it froze because the fuel floor held the reading exactly.
  Give the fuel side a delay too and the valve drifts along the same undecided direction.
- **Rung 88:** "pure loss" holds in the valve's own ruler. Beside the stators, measured by blade angle, a slow valve
  that watches the flow coefficient holds the same margin with less air than an instant one.

**On the page.** *Opens on its own → Moves slowly*, with *Time to move*. The page notes that a slow valve cannot
run beside the temperature limiter's lag or the realistic release.

---

## Rung 66 — Two loops on one reading are one loop

**In one sentence:** a delayed valve and a delayed fuel floor, both holding the same reading at the same target,
behave exactly like a single loop whose speed is the two speeds added together; so the second one adds speed, not
protection.

**What it found.**
- **One loop, rates added.** This follows from an exact identity, at every delay and every operating point. One
  practical consequence: the simulation's time step has to be up to twice as fine as for one loop alone.
- **The second loop adds almost nothing.** On its own the delayed fuel floor delivers 60.46 % of the protection;
  added on top of the valve it adds 1.59 %, about 38 times less. The pair (94.09 %) still beats either alone.

**On the page.** Not shown: the page's slow valve does not run beside a delayed fuel limiter.

---

## Rung 67 — Two loops on two readings both keep their power

**In one sentence:** pair the delayed valve with the delayed turbine-temperature limiter (rung 47) instead, two
loops watching *different* readings, and each keeps essentially all its own protection.

**What it found.** What a second loop buys depends on whether it watches a different reading, not on its rule, its
actuator or its speed. Here each loop's protection changes by a factor close to 1 (0.93 and 1.26) instead of rung
66's 38-fold loss. In principle the pair could now oscillate; in practice the same number that allows the
oscillation damps it (damping ratio 0.990, one swing taking 44 τ), so it dies out long before it could be seen.

**On the page.** Not shown, for the same reason as rung 66.

---

## Rung 68 — Three loops on one reading are still one loop

**In one sentence:** add a third controller, a stator limiter, on the same reading at the same target, and the
three are still one loop with all three speeds added; and how far the stators can turn, which decides the
protection when they act alone, makes no difference with the other two beside them.

**Later correction.** Rung 69 found that what decides is the reading each loop watches, not the number of loops.
Aim the stator limiter at blade angle instead of flow coefficient and it stops being redundant: it adds real
authority. That is the start of the control-structure study in rungs 69–84.

**On the page.** Not shown.
