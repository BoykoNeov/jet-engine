# Rungs 57–63 — Schedules: the airflow levers on a moving engine

*Plain-language notes. The formal derivations, tests and every number's source are in `docs/rung57-spec.md`
to `docs/rung63-spec.md`. Numbers below are copied from those specs and belong to the cases the specs ran; the
sandbox page shows its own. Rung 61 belongs to the airflow-levers family and is told in
`53-56-61-airflow-levers.md`.*

Rungs 53–56 built the variable stators and studied them on an engine running steady. A real engine moves them
during a throttle move, on a **schedule**: closed at low shaft speed, opening as the shaft speeds up. These rungs
put that schedule, and then a bleed-valve schedule, onto the moving two-shaft engine of rungs 40–52, and ask how
they get on with the fuel limiters of rungs 46–52.

**Three things to know first.**
- **Two kinds of lever.** Fuel limiters, the bleed valve and the throttle move the engine's **running point**
  toward or away from the stall line. The stators move the **stall line** itself (rung 53). This family keeps
  finding that the two kinds obey different laws.
- **Two rulers.** Distance from stall measured as **flow coefficient** or as **blade angle** (incidence); with
  the stators moving, only the blade angle is a fair ruler (rung 53).
- **The test move** in most of these specs is the two-shaft engine slammed from 1000 K to 1400 K at the turbine
  inlet, timed in shaft response times (written **τ** on the page).

---

## Rung 57 — A lever that moves the stall line has no clock

**In one sentence:** the fuel limiters' help depended on *when* they acted (rungs 46–52); the stators' help does
not, because a lever that moves the stall line is credited by where the engine is, not by when.

**What it found.**
- **No clock.** Hold the stators at a fixed setting and slam the throttle at speeds spanning twenty times. The
  margin itself swings by about half, but the share of the stators' turning that turns into real margin moves by
  only 1.05 points. Rung 53's steady formula predicts that share within 3.9 %, far from the design point and
  mid-move.
- **About a third arrives.** Turning the vanes moves the stall line by the full amount, but the shaft speeds up
  and the running point slides back toward the line, eating about two thirds of it, whatever the speed of the move.
- **A schedule partly undoes itself.** Closing the stators makes the shaft spin faster; the schedule reads that
  higher speed and opens the stators again. It gives up 10 to 25 % of its own effect this way.

**What it does to its neighbours.** It **bounds rungs 46–52**: their "a limiter helps only if it acts before
the worst moment" is a law about levers that move the running point, not about limiters in general. It also
**corrects rung 53**: on the moving engine the low-pressure stators do reach the high-pressure shaft (in the
spec's case they shift the air temperature between the compressors by 13.05 K), because the steady solution's
shaft balance, which absorbed that, is gone.

**On the page.** *Controls → Airflow levers → Lever → Variable stators, scheduled on speed (rung 57)*, with
*Low-pressure stators / High-pressure stators*, *Furthest they close* and *Fully closed below × design speed*.

---

## Rung 58 — Two levers in one controller do not simply add

**In one sentence:** put the stator schedule beside a fuel limiter and the limiter changes what the stators
deliver, while the stators barely touch the limiter; and a limiter that watches the flow coefficient cannot be
paired with the stators at all.

**One way, not both.** In the spec's case, arming the acceleration schedule (rung 48) changes the stators' help by
9.5 %, while the stators move the moment the schedule starts cutting by only 0.16 %, about 59 times less. Why: the
fuel cut moves the worst moment earlier in the slam, to a lower shaft speed, and there the speed-driven stator
schedule happens to be more closed. A stator held at a fixed setting has no such link, and its interaction is about
ten times smaller.

**The flow-coefficient floor (rung 49) cannot be paired.** A floor must sit between the engine's starting flow
coefficient and its lowest one, or it either acts from the start or never acts. The stators lower the whole
running line by more than the slam's own swing, so the allowed range with stators and without them do not even
overlap. No single floor setting means the same thing on both engines.

**Later.**
- **Rung 59** showed the matched-schedule experiment this rung refused would have given the same numbers, and also
  that "the stators do not move the schedule" holds only for the low-pressure stators.
- **Rung 60** found the non-overlap was a symptom, not the cause.
- **Rung 63** traced the one-way arrow to what the fuel schedule's table is protected by.
- **Rung 86** extended the analysis to three levers at once.

**On the page.** *Controls*: switch on the stator schedule and *Acceleration schedule* together.

---

## Rung 59 — The fuel table cannot see the stators, only where it is read

**In one sentence:** the acceleration schedule is a table of allowed fuel against shaft speed; the stators cannot
change the table's values, only which speed a given engine state is read at, so re-deriving the table for a
statored engine is a pure re-labelling.

**Why the values cannot move.** The table's allowed fuel (per unit of compressor delivery pressure) depends only
on the turbine inlet temperature. The turbine's entrance vanes are choked, which fixes how much gas passes for a
given pressure and temperature, and the two shafts' power balances fix the compressor exit temperature. Neither
can feel the stators: the stators change how fast the shaft must spin to do the work, not the work itself.

**Where it can.** The table is read against the high-pressure shaft speed.
- **Low-pressure stators** cannot move that speed (rung 39's one-way link from the high-pressure spool to the
  low-pressure one), so the matched table is the original table exactly.
- **High-pressure stators** speed that shaft up (by 3.32 % and 6.69 % at the two settings tried), so the same
  table is read at a different place. Here using the unmatched table invents an interaction: 48 to 96 times too big
  on the low-pressure spool, and with the wrong sign on the high-pressure one.

**What it does to rung 58.** Rung 58 ran low-pressure stators, so its refused experiment was the one it had already
run. Its "the stators do not move the schedule" is a low-pressure-stator statement.

**Later.** Rung 63 found the table's protection holds only for levers that keep every kilogram of air in the
engine; a bleed valve moves both the values and the index.

**On the page.** Not on its own; the *Acceleration schedule* on *Controls* is always derived for the engine you run.

---

## Rung 60 — A floor pins what it watches

**In one sentence:** re-aim rung 49's stall floor at blade angle instead of flow coefficient and it can now be set
the same way on both engines, but pairing it with the stators still measures nothing, because a floor that is
acting holds its own reading exactly at the floor.

**Matching a floor is not well defined.** A schedule has a rule you can re-run on the new engine; a floor's set
point does not. Two sensible rules ("the same margin from the moved stall line", "the same blade angle") differ by
14 % of the slam's whole swing in the spec's case. Rung 58's own finding picks the blade angle.

**Re-aimed, the overlap problem goes away.** On blade angle, the gap between the allowed ranges shrinks 24 times
with stators at a fixed setting, and closes completely with rung 58's schedule.

**But the floor pins.** While a floor is cutting fuel, its reading sits exactly on the floor. So the "difference
the stators make" beside it is fixed in advance: exactly the stator setting for a flow-coefficient floor, exactly
zero for a blade-angle floor. A leg that sets a minimum cannot be paired with a stall-line-moving lever in any
ruler.

**What it does to rung 58.** Confirms its verdict, corrects its diagnosis, and turns its ranking over: the
fixed-setting stator, with the smaller interaction, is the one that cannot be paired; the schedule can.

**On the page.** *Controls → Stall floor → Watches the flow coefficient / Watches blade incidence*. The page's note
says it: a floor on the flow coefficient stays put while the stators move the stall line; one on the blade angle
moves with it.

---

## Rung 62 — A bleed schedule pushes itself further

**In one sentence:** a schedule driven by shaft speed feeds back on itself through that speed, and the sign of the
feedback is set by which way the lever moves the shaft: the stators slow their own schedule down, the bleed valve
speeds its own up.

**How it works.** Closing the stators makes the shaft spin faster, so the schedule reads a higher speed and opens
them again (rung 57). Opening the bleed valve makes the low-pressure shaft spin *slower*, so its schedule reads a
lower speed and opens the valve *further*. In the spec's case the stator schedule gives up 17 to 23 % of its own
effect while the bleed schedule gains 9 to 10 % on its own. Both signs could be read off earlier published tables
before anything was run.

**Two loops through one shaft speed do not combine evenly.** Run both schedules together and the bleed triples the
stators' self-cancelling, while the stators leave the bleed's self-boost alone to within 0.7 %. The arrow runs from
the boosting lever to the cancelling one.

**It corrects rung 61.** On the steady engine the two devices' benefits added up within 2.3 %. On the moving engine
the pair falls short of the sum by 9.1 to 29.1 %. The near-additivity came from the steady solution's shaft
balance, the same artifact rung 57 found under rung 53's exact zeros. Rung 61's finding about the extra shaft speed
survives.

**On the page.** *Controls → Airflow levers → Lever → Handling bleed valve (rungs 62 and 64) → On a speed
schedule*, with *Most air bled* and *Fully open below × design speed*. Choose *Both: stators and bleed together*
for the pair.

---

## Rung 63 — Fuel limiter and bleed valve: the lever that removes air

**In one sentence:** the acceleration schedule's table is protected against every lever that keeps all the air in
the engine, and the bleed valve, the only lever that throws air away, gets past both protections.

**How it works.** Rung 59 found two guards on the table: the choked turbine vanes protect its values, and the
one-way link between the shafts protects its index. Both rest on the air entering the compressor being the air
that reaches the burner. The bleed breaks exactly that, upstream of both guards, so it shifts both halves of the
table by about a percent, where the stators shift neither at all.

**What it does to the fuel limiter.** In all six cases tried, the bleed makes the schedule start cutting 2.9 to
4.2 % later. The stators can also shift that moment (by up to 1.28 %) without touching the table, because the
moment depends on the path the engine takes, not only on the table. What separates the two is that the bleed's
shift is always later and larger.

**A stall floor and the valve have no middle ground.** The bleed helps only by raising the flow coefficient. So
beside a flow-coefficient floor there are just two outcomes: either the valve keeps the engine above the floor and
the floor never needs to cut, or the floor is acting and pins the reading (rung 60's result, reproduced exactly). Set
the floor high without the valve and the fuel-only run cuts fuel until the engine nears flame-out and stops; with
the valve every run finishes.

**What it does to its neighbours.** It bounds rung 59's "the table cannot see a lever" to levers that keep the air
in, and rung 58's one-way arrow to the same class. It extends rung 60 with the second regime, where the lever
switches the floor off.

**Later.** Rung 86 put all three, fuel, bleed and stators, on one engine; that closes this rung's open question.

**On the page.** *Controls*: the bleed valve beside *Acceleration schedule* or *Stall floor*.
