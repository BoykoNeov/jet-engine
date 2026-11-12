# Rungs 38–45 — Two shafts

*Plain-language notes. The formal derivations, tests and every number's source are in `docs/rung38-spec.md` to
`docs/rung45-spec.md`. Numbers below are copied from those specs and belong to the cases the specs ran; the
sandbox page shows its own.*

Every turbojet above modest size splits its compressor in two. A **low-pressure (LP)** compressor at the front is
driven by the rear turbine on one shaft; a **high-pressure (HP)** compressor behind it is driven by the front
turbine on a second shaft, running inside the first. The two shafts are not geared together: each finds its own
speed, tied to the other only through the gas. These eight rungs rebuild rungs 31–37 on that engine: where it
runs, how it moves in time, and where it comes close to stall.

All of the page's *Controls* and *Cool the turbine* views run this engine. Time is in shaft response times, written
**τ** on the page (see the rungs 34–37 note).

---

## Rung 38 — Where a two-shaft engine runs

**In one sentence:** the two-shaft engine can be solved in a chain, front to back, with no two unknowns that must be
found together, but that turns out to be a property of having no compressor maps, not of the engine.

**How it works.** A third choked throat appears: the LP turbine's entrance, between the two turbines. With the HP
turbine's entrance and the exhaust nozzle, the three throats pin both turbines' expansions, just as two throats
pinned the one turbine in rung 31. Each shaft's turbine work then fixes how much its compressor heats the air, and
each compressor's efficiency only turns that heating into a pressure ratio at the very end. Neither compressor's
efficiency reaches the other's pressure ratio.

**Honest limit.** The spec says so itself: a real compressor map makes airflow depend on pressure ratio too, which
should tie the two compressors together. That is rung 39.

**Later correction.** Rung 104 found the LP turbine's entrance never actually chokes. With the nozzle choked the old
assumption gives the same answer; below choke it does not (see the rungs 103–105 note).

---

## Rung 39 — Two shafts with compressor maps

**In one sentence:** compressor maps do not tie the two compressors into one joint problem as rung 38 expected;
they open exactly **one** link, from the HP compressor to the LP one, and none the other way.

**How it works.** With maps, the HP compressor's efficiency reaches the LP compressor's pressure ratio (by
1.5 × 10⁻⁴ to 6.7 × 10⁻⁴ in the spec's cases), while the LP compressor's efficiency leaves the HP pressure ratio
**exactly** unchanged. The reason: the HP compressor's front face sees only its own pressure ratio, while the LP
compressor's sees the product of both. So rung 38's verdict (no joint problem) survives, and rung 38's reason for
expecting it to fail is refuted.

**New output: slip.** With two shafts the ratio of their speeds becomes an output, the **slip**, which changes with
the throttle.

**Later.** Rung 103 found the link does run the other way once the nozzle unchokes.

---

## Rung 40 — Two shafts in time

**In one sentence:** with two shafts that each take time to speed up, the ratio of their response times cannot make
the engine unstable, but a shaped LP compressor map lets the two shafts swing against each other.

**How it works.** Both shaft speeds are now states that move under their own power imbalances. The new parameter is
the ratio of the LP shaft's response time to the HP shaft's.

**What it found.**
- **Stability does not depend on that ratio.** Over a 2000× range of it, on every map shape and gas tried, the engine
  settles. (The signs that guarantee this were measured, not derived.)
- **Oscillation does.** With a shaped LP compressor map, speeding the HP shaft helps the LP shaft's power balance
  strongly, while speeding the LP shaft hurts the HP's. That push-pull lets the two speeds overshoot and swing back
  in a band of the ratio. With a flat LP map it never happens. The swing is created by the map.

---

## Rung 41 — The two-shaft stall line

**In one sentence:** two shafts do not halve the low-power stall problem: they concentrate it on the LP compressor.

**How it works.** Over a 2:1 throttle range the LP compressor's flow coefficient (rung 36: airflow against blade
speed; lower is nearer stall) falls about **29 %**, the HP compressor's only about **7 %**, and the HP's even turns
back up. The HP compressor's face sees only its own pressure ratio; the LP compressor's carries the product of
both.

**A number with no knob in it.** For a compressor behind a choked turbine entrance, there is a pressure ratio at
which its running line stops moving in flow coefficient: about **3.25** for air (3.2467 to four places). Every efficiency
and engine constant drops out; only a property of air remains. Below that ratio the flow coefficient turns back
up. Rung 36 had hoped for such a knob-free anchor and could not find one; this is it.

**What it does to rung 36.** Rung 36's verdict (thin at low power) stands; its reason does not. The margin shrinks
through two comparable causes, not one (see the rungs 34–37 note).

---

## Rung 42 — Bleeding air between the compressors

**In one sentence:** a valve that lets air out between the two compressors gives the LP compressor a genuinely new
freedom, and gives the HP compressor none.

**How it works.** Real engines carry such a valve to protect the LP compressor at low power. Opening it moves the LP
compressor's flow coefficient by **8–12 %** at a fixed throttle, off its running line onto a new one. The HP
compressor stays on the same line, to within 0.01–0.016 %: the valve only slides it along the line the throttle
already slides it along. The guess that bleeding would penalise the HP compressor was refuted.

**Later.** Rung 64 made this valve a closed-loop protection; rung 92 found that a bleed is exactly a shaft power
offtake plus air thrown away, which explains the HP compressor's indifference (see the rungs 90–94 note).

---

## Rung 43 — Metering fuel on two shafts

**In one sentence:** with fuel metered, the turbine temperature overshoots as on one shaft, and **neither** shaft's
delay alone governs how much.

**How it works.** The fuel arrives at once; the air it burns in is set by the LP compressor's face; the hot gas is
let through by the HP turbine's choked entrance, fed by the HP compressor. So the two shafts sit at different points
of the one overshoot loop. Freeze either shaft in the model and the overshoot gets worse: both shafts' motion
relieves it, and their shares trade as the response-time ratio changes.

**A trap, recorded.** Trying to fit "the overshoot follows the LP shaft's clock" or "the HP shaft's" gave whichever
answer matched the shaft used to measure the overshoot. Measured against a neutral yardstick (the peak temperature
over its target), neither clock fits, and no single blend of them does either.

---

## Rung 44 — The stall line during a two-shaft slam

**In one sentence:** a slam pushes both compressors toward stall, the LP one about **1.6–2.2×** as far as the HP
one, and how far is set by how fast you slam, not by the two shafts' response times.

**How it works.** The excursion toward stall stays the same to within 2 % over a 25× range of the response-time
ratio, and the swinging of rung 40 has nothing to do with it (the map with no swing shows the biggest LP-to-HP
ratio). The two things rung 40 studied are both irrelevant to stall. What matters is the plain one: the speed of the
throttle move.

**On the page.** *Controls*, the move's duration. *Try this → Slam faster: 0.1 τ* and *A gentle move: 2 τ*; in
`docs/sandbox/EXPERIMENTS.md` (experiment 14) the peak turbine temperature runs 1943 / 1695 / 1477 K for a move of
0.1 / 0.5 / 2 τ.

---

## Rung 45 — The same, with fuel metered

**In one sentence:** with fuel metered, the temperature overshoot *does* depend on the shafts' response-time ratio
(about 12 % over a 25× range), but the closest approach to stall still does not (under 2 %).

**What it means.** The effect is real in the engine but never reaches the stall margin. It shows up only in
yardsticks measured against a moving reference, which swing by about 40 %. That is rung 43's trap again, now on the
stall side. Fuel metering also enlarges the approach to stall, as rung 35 found on one shaft.

**Next.** Rungs 46–52 put limiters on this fuel (see that note).
