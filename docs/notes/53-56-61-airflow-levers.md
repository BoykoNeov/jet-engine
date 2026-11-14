# Rungs 53–56 and 61 — Airflow levers: turning the vanes, and what that really buys

*Plain-language notes. The formal derivations, tests and every number's source are in `docs/rung53-spec.md`
to `docs/rung56-spec.md` and `docs/rung61-spec.md`. Numbers below are copied from those specs and belong to the
cases the specs ran; the sandbox page shows its own.*

Rungs 46–52 protected the compressors from stall by holding fuel back. Real engines also have two devices that
act on the **air** instead:
- **Variable stators.** In front of each spinning row of compressor blades sits a row of fixed vanes. On a real
  engine the front rows of vanes can be turned. Turning them ("closing" them) gives the air a swirl in the
  direction the blades spin, so the blades meet it at a gentler angle.
- **A bleed valve** (rung 42): a door between the two compressors that dumps some air overboard, so the
  low-pressure compressor can pass more air than the high-pressure one will take.

Both exist because a compressor running slowly tends to stall at its front. These five rungs put them into the
model. Rung 61 sits here because it is the steady-running test of both devices together; rungs 57–63 then put
them on the moving engine.

**Two ways to measure distance from stall.** Until now the model measured it with the **flow coefficient**: the
air's speed through the compressor against the blades' speed (rung 36; lower is closer to stall, and the stall
line is a number you choose, 0.55 by default). What actually makes a blade stall is the **incidence**: the angle at
which the air meets it. Past a certain angle, set by the blade's metal shape, the air breaks away from the blade.
For every lever before rung 53 the two measures always agree. The stators are the first lever where they do not,
and that is this family's main finding.

---

## Rung 53 — A lever that moves the stall line itself

**In one sentence:** the variable stator is the first lever that moves the stall line rather than the engine's
running point, and once the line moves, "distance from stall" depends on the ruler you measure it with: the
flow coefficient says closing the stators *loses* margin, the blade angle says it *gains* margin, and only the
blade angle is right.

**Why the two rulers disagree.** A margin is a distance to a boundary. Measured as a blade angle, the boundary
is fixed: it is the angle the metal can take. Measured as a flow coefficient, the boundary moves whenever the
stators move, because pre-swirled air lets the blade run at a lower flow coefficient before it stalls. So "the
margin in flow coefficient fell" mixes two things: the engine moved toward the line, and the line moved. Read in
blade angle, the model says what engineering practice says: **closing the stators buys stall margin.**

This also gives rungs 36–52 a licence they lacked. For every lever that does *not* move the line, the two rulers
always agree in direction, so measuring stall by flow coefficient was correct there. It bounds those rungs; it
does not overturn them.

**What it costs: shaft speed, not thrust.** Closing the stators lightens the compressor's work per turn, so to
do the same work the shaft spins faster. Over the range tried, thrust and fuel per thrust change by less than
0.13 % while the low-pressure shaft speeds up by 19 %.

**A schedule that holds the design angle.** Turn the stators by just enough at each throttle to keep the blade
meeting the air at its design angle, which is what a real stator schedule is for. Along that schedule the true
margin stays exactly constant, while the flow coefficient reports a 74 % *loss* of margin. One trajectory, one
boundary, opposite verdicts. The price in this simple model is high: at 1000 K the low-pressure shaft runs 26 %
above its design speed. The low-pressure spool needs 6.7 times the high-pressure one's stator setting, because it
is the spool that takes the throttle's swing (rung 41).

**Later corrections.**
- **Rung 54** showed the stators' benefit can peak and turn back on some compressor shapes, inside the range the
  engine runs in; this rung had said it only levels off.
- **Rung 55** showed most of the speed price is an artifact of treating the compressor as one block.
- **Rung 57** found that this rung's "the low-pressure stators cannot touch the high-pressure spool, exactly" holds
  only for the engine running steady; on a moving engine a small effect leaks through.
- **Rung 61** found that the stator's clean one-spool action is lost once it is paired with a bleed valve.
- **Rung 85** found the speed price of holding the angle does not depend on the compressor map's slope at all;
  quoting it against the unscheduled speed brings the slope back in through the comparison.

**On the page.** *Size the blades → A stator lever → Which vanes turn → The compressor as one block (rung 53)*. On
*Controls*, the stall floor's choice *Watches the flow coefficient / Watches blade incidence* is this rung's two
rulers.

---

## Rung 54 — The vanes' own throat: a hard limit that costs little

**In one sentence:** turning a row of vanes narrows the gap between them, so a far-turned row can choke; that
cuts the stator's usable travel by 30 % but its benefit by only 4 %, because the travel it removes was worth almost
nothing.

**How it works.** The opening between two vanes shrinks as they turn, by a simple geometric rule that needs no
new constant. What does need one is how close to choking the vane row already runs at design, which the spec
reports as a threshold rather than a fixed answer.

**What it found** (the default compressor shape, 1000 K, a row designed at 0.90 of its choking flow).
- The row chokes at 30.4 % less travel than the model could otherwise reach, yet the blade-angle margin falls only
  4.0 %. Across all five map shapes and throttles the stator keeps at least 78 % of its benefit.
- **A throat can only block, never help.** It stops the engine reaching some settings; it cannot change where the
  engine runs at the settings it allows. So adding the throat cannot buy back rung 53's overspeed.
- **Rung 53's concession was wrong in its reason.** On 3 of the 5 shapes the benefit peaks and turns back before
  the travel runs out (on the steep shape it falls 11 % past its peak). On two shapes the design-angle schedule
  stops existing at part power. Rung 53's verdict, that the stators' power is finite, stands.
- Above about 865 K at the turbine inlet the schedule asks *less* of the throat than the design point does, so
  every published number of rung 53 is untouched by it.

**Later correction.** Rung 56 found that "the throat limit is a low-pressure-spool problem" holds for the
compressor as one block but not row by row.

**On the page.** Not shown on its own.

---

## Rung 55 — Split the compressor into stages: move only the front row

**In one sentence:** treat the compressor as a stack of stages and move only the front row of vanes, as real
engines do, and holding the design angle costs about one twenty-ninth of the shaft speed; but the more rows you
move, the more you push the problem onto the rows you did not move.

**How it works.** The compressor becomes several stages in a row, each doing its share of the work. As the
throttle comes back the air is less squeezed at the front than the design expected, so the front stages meet the
air at too steep an angle (toward stall) and the rear ones too shallow (toward choking). *Front stages stall, rear
stages choke* is the textbook picture, and one lumped block cannot show either end of it.

**What it found** (eight stages, 1000 K, the default shape).
- **Front row only: +2.3 % shaft speed** to hold the front stage's design angle, against +66.7 % for rung 53's
  whole-block lever (the same figure as rung 53's +26 %, measured from the unscheduled speed at that throttle
  rather than from design). The front row is one eighth of the machine, *and* it needs less turning because it does
  not fight its own speed rise.
- **The fix lands on the row you move, not the machine.** Fixing the front stage makes the second stage the worst,
  and the machine's worst margin improves by only +9.5 %.
- **The best number of rows to move is in the middle.** Moving more rows helps up to 4 (+20.94 %), then hurts: at 6
  rows the worst margin is 19.78 % *worse* than not moving any, because the worst row has moved to the back, into
  the rows the stators do not reach.
- **Rungs 36–53 were slightly optimistic.** Their single-block reading of the front stage's distance from stall
  was 2–4 % generous, more so at low power, and the high-pressure compressor's rear stage runs 25.6 % above its
  design flow coefficient at the lowest power. Their trends stand, sharpened.

**Later.** Rung 85 bounded "the stage count is only a resolution": once blade-speed limits fix the design, the
stage count comes out of the design. A study of per-row blade shapes (`docs/per-row-blading-negative.md`, not a
rung) found that this rung's "work split between stages" was really a blade-shape choice; the verdicts stand.

**On the page.** *Size the blades → A stator lever → Which vanes turn*: *Every row's vanes (rung 55, at the sized
stage count)* against *The front row's vanes only*, with *Low-pressure spool / High-pressure spool*. The chart shows
both shaft speeds.

---

## Rung 56 — The two worst rows are on different shafts

**In one sentence:** row by row, the row nearest to stall is the low-pressure compressor's *front* row and the
row nearest to choking is the high-pressure compressor's *rear* row, and a front-row stator helps the first while
hurting the second, through the shaft speed they share.

**What it found** (eight stages per compressor, 800 K at the turbine inlet).
- The worst blade-angle margin in the whole machine is the low-pressure front row (0.349); the worst choking
  margin is the high-pressure rear row (0.164). Different ends, different shafts.
- **The binding row moves with power.** Near design the front row is closest to choking (it runs the fastest
  air); at part power the rear row is. On the low-pressure compressor the switch comes between 1400 and 1300 K.
- **It corrects rung 54.** Measured on the compressor as a whole, the high-pressure compressor's choking margin
  *grows* as the throttle comes back (0.2163 to 0.3647 from 1200 to 800 K). Measured on its rear row, it *shrinks*,
  to 0.1636. A rear row designed at more than 0.913 of its choking flow would choke at 800 K.
- **A lever's cost depends on the ruler too.** Measured in shaft speed, the front-row lever costs about the same
  fraction of the whole-block lever at any setting; measured in choking margin, its advantage grows with the
  setting.

**Later.** This rung's open question, letting each row have its own blade shape, was investigated and closed
negative (`docs/per-row-blading-negative.md`): the answer depends on what is held fixed while the blades change,
and the one sensible choice leaves the choking side essentially unmoved.

**On the page.** Not shown.

---

## Rung 61 — Stators and bleed together: undoing a side effect is not undoing its cost

**In one sentence:** the bleed valve can exactly cancel the stators' drop in flow coefficient, but the extra shaft
speed the stators caused mostly stays; past a moderate setting the "corrected" engine spins faster than the
uncorrected one.

**The idea it tested.** The long-standing plan was a hand-over: the stators work until they run out of travel,
then the bleed takes over. That is refuted. Opening the valve *shrinks* the stators' remaining reach, because the
valve has already spent part of the same margin. The two are substitutes drawing on one budget: their benefits
add up to within 2.3 %.

**The headline.** Open the valve just enough to bring the flow coefficient back to where it was without stators.
That flow-coefficient drop is gone, exactly. But the overspeed is still 73 % to 102 % of what the stators caused
alone, and the valve adds its own thrust cost (up to 30 % lost). Why: the drop in flow coefficient was itself
giving back some of the work the stators removed, so restoring it gives that back too. "Undoing a lever" undoes a
reading, not a machine.

**Only on one shaft.** The valve can buy back a low-pressure stator's effect, but on the high-pressure shaft the
whole valve range is 4 to 17 times too weak.

**It corrects rung 53.** Alone, the low-pressure stators leave the high-pressure shaft exactly untouched. Paired
with the valve they do not, because the valve's own effect reaches both shafts through the air temperature between
them.

**Later corrections.**
- **Rung 62:** on the moving engine the two devices' benefits do *not* add up nearly exactly: the pair falls short
  of the sum by 9.1 to 29.1 %. The near-additivity was a feature of the steady solution. The speed-cost finding
  survives.
- **Rung 64:** turned the headline around from the other side: hold the margin fixed and compare *ways of driving*
  the valve, and what differs is only the cost.

**On the page.** *Controls → Airflow levers → Lever → Both: stators and bleed together (rungs 86 and 87)* puts both
on the moving engine; the steady comparison of this rung is not shown.
