# Rungs 86–89 — The levers together, and how a valve should watch

*Plain-language notes. The formal records are `docs/rung86-spec.md` to `docs/rung89-spec.md`; each names its anchor
document in `docs/plans/`. Numbers below are copied from the specs and belong to the cases they ran; the sandbox
page shows its own.*

By rung 68 the model had three ways to protect a compressor from stall during a throttle slam: hold fuel back
(rungs 46–52), turn the stator vanes (53–60), bleed air overboard (42, 62–65). Each was studied alone or in pairs.
These four rungs, all built as knobs for the sandbox's *Controls* view, put them together on one engine and ask the
practical question the earlier pairs kept running into: when the stators move the stall line, **which reading
should the bleed valve watch?**

**Two rulers, again.** Distance from stall can be read as the **flow coefficient** or as the **blade angle**
(incidence). The stators lower the flow coefficient on purpose while they *raise* the true margin to the blade's
stalling angle (rung 53). Most of this family is what happens when a controller reads the wrong ruler.

---

## Rung 86 — Fuel, bleed and stators on one engine

**In one sentence:** a third lever adds no new kind of interaction, only a new moment at which the existing
stator-and-bleed interaction is read; and a stall floor read as a flow coefficient fights the stators so hard it can
starve the engine, until the bleed rescues it.

**How it works.** Three switches (stator schedule on or off, bleed schedule on or off, one fuel limiter on or off)
make eight runs. The part of the result that exists only with all three on can then be separated exactly. Rung 63
had worried a third lever would make that impossible; with all eight runs it does not.

**What it found.**
- **The acceleration schedule** (rung 48) moves the worst moment earlier, before the stator-bleed interaction has
  built up, so the three-way part is a small share of the pair's (7 to 16 %) with the opposite sign, and rung 58's
  predictor, built from runs without the limiter, recovers 71 to 93 % of it.
- **The turbine temperature limiter** (rung 46) touches a run only if that run is hotter than its redline at its
  worst moment. The levers move the worst moment later and hotter, which decides which runs it reaches.
- **A stall floor on the blade angle** simply pins the margin at its set point (rung 60).
- **A stall floor on the flow coefficient starves a statored engine.** The stators lower the flow coefficient from
  the first instant, so the floor cuts fuel from the start, and at the highest set point there is no run at all. The
  bleed lifts the flow coefficient back and rescues it. Rung 53's warning about rulers, as a stopped engine.

**On the page.** *Controls → Airflow levers → Lever → Both: stators and bleed together (rungs 86 and 87)*, beside
any fuel limiter. The page's note says it: watch the stall floor read as a flow coefficient.

---

## Rung 87 — A valve beside the stators must watch the blade angle

**In one sentence:** a bleed valve that holds the flow coefficient reads the stators' deliberate drop as danger
and bleeds air to undo it; one that holds the blade angle lets the stators carry part of its load.

**What it found.**
- **The flow-coefficient valve pays to undo the stators.** It runs out of opening sooner (its ceiling falls by
  0.042 to 0.053), and set to give the same blade-angle protection it spends 1.27 to 3.50 times the air, and more
  thrust. It must over-protect early in the slam, when the stators are most closed, to be safe late, when they have
  reopened.
- **The blade-angle valve bleeds less with the stators than without them**; the flow-coefficient valve bleeds more.
- **The fuel side must watch the same ruler.** A blade-angle valve beside a flow-coefficient stall floor lets the
  flow coefficient fall, the floor wakes, and the run starves, in every case scored, six of them before the first
  point.

The spec is open that part of this follows from how the blade-angle valve is built (it opens exactly as far as the
protection needs); the content is the size of the difference and that it never reversed.

**What it does to its neighbours.** Rung 64's "the ceiling is the fully-open valve" is a bound that a valve opening
late can fall below, and beside the stators the ceiling depends on the ruler.

**Later.** Rung 88 found the price comparison holds only for a valve that moves instantly.

**On the page.** *Handling bleed valve → Opens on its own → Watches the flow coefficient / Watches blade incidence*,
with the stators on (*Both*). The page's note repeats the pairing advice.

---

## Rung 88 — A slow valve turns the price around

**In one sentence:** the blade-angle valve opens exactly as late as it can, so it has no slack for a slow actuator;
the flow-coefficient valve holds margin it does not need, and that surplus absorbs the delay, so a slow enough
flow-coefficient valve is the *cheaper* one.

**What it found.** Both valves re-tuned to hold the same blade-angle margin: instant, the flow-coefficient valve
costs 1.27 to 3.47 times the air (rung 87); with a delay of 0.1 τ (shaft response times) it costs 0.53 to 0.91
times, in all 19 cases, and less thrust in 18 of them. The crossover sits between 0.02 and 0.1 τ on this engine; the
claim is "a slow enough valve", not a number for real valves.

**And the slow valve's extra air is simple arithmetic:** the delay times the opening the valve starts the slam with,
plus a small drift. So which valve starts open (here, the flow-coefficient one) largely decides the bill.

**What it does to its neighbours.** It bounds rung 87: "watch the blade angle" still holds for protection, but its
price advantage belongs to a fast valve. It explains rung 65's bleed bill and bounds its "a delay is pure loss" to the
valve's own ruler: read in the other ruler, beside the stators, a delay can be a gain.

**On the page.** *Opens on its own → Moves slowly*, with *Time to move*. The page's note gives the reason in words.

---

## Rung 89 — Opening speed and closing speed price different valves

**In one sentence:** give the valve one speed for opening and another for closing, and each valve turns out to be
priced by one of them: the blade-angle valve by how fast it opens, the flow-coefficient valve by how fast it closes.

**Why.** The blade-angle valve's margin is lowest before it ever starts closing, so its closing speed cannot change
its protection; it only changes the air held on the way down. The flow-coefficient valve's margin is lowest while it
is closing, so a slow close lets it run a lower target and bleed less.

**So the closing speed moves the verdict both ways.** A slow close makes the flow-coefficient valve the cheaper one
at every opening speed, in all 19 cases. A fast close hands the blade-angle valve 68 of 95 runs, slow openings
included, which this rung had predicted would not happen. Of the two speeds, the closing one is the stronger dial.

**What it does to its neighbours.** It splits rung 88's single "slow enough" into two dials. It confirms rung 52's
fast-cut, slow-release finding on an airflow lever: the blade-angle valve's closing edge never reaches the margin it
watches, so its closing speed is pure cost.

**On the page.** *Moves slowly → Closes at its own speed*, with *Time to close*. The page's note states this rung's
result.
