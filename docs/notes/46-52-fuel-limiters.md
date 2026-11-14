# Rungs 46–52 — Fuel limiters: holding fuel back, and handing it back

*Plain-language notes. The formal derivations, tests and every number's source are in `docs/rung46-spec.md`
to `docs/rung52-spec.md`. Numbers below are copied from those specs and belong to the cases the specs ran; the
sandbox page shows its own.*

Rung 35 found the trouble: a slammed throttle pours fuel into air the slow shaft has not yet caught up with,
so the turbine overheats and the compressor is pushed toward stall. Real engines therefore never let the pilot's
throttle reach the fuel directly. A controller sits between them and **limits** the fuel. These seven rungs
build those limiters one at a time and ask what each one actually protects.

**Three things to know first.**
- **Two shafts.** From rung 38 the engine has two: a front, low-pressure compressor driven by a rear turbine on
  one shaft, and a high-pressure compressor driven by its own turbine on a second shaft inside it. Each
  compressor has its own distance from stall. Rung 41 found the low-pressure one is the more exposed: it is the
  **binding** one, the one that would stall first.
- **Distance from stall** is read as the **flow coefficient** (rung 36): the compressor's airflow measured
  against its blade speed. Lower is closer to stall. The changes quoted below are changes in it, so +0.003
  means "a little further from stall".
- **Time** is in shaft response times, written **τ** on the page (see the rungs 34–37 note). The test move in
  most of these specs is a slam from the fuel of a 1000 K point to that of a 1400 K point.

The whole family turns out to be about **timing**. During a slam each compressor has one moment when it comes
closest to stall, and the two compressors reach theirs at different moments. A limiter can only protect a
compressor whose worst moment comes *after* the limiter starts cutting fuel.

**How far this reaches.** Three later rungs drew its edges, each from a different side:
- **Rung 53** bounded the yardstick. Distance from stall measured as flow coefficient stops meaning one thing once a
  lever (variable stators) can move the stall line itself.
- **Rung 57** bounded the timing law. It holds for levers that move the engine's running point, like fuel. A lever
  that moves the stall line instead has no such clock: its effect is the same whenever it acts.
- **Rung 64** bounded the protection. The most any limiter can protect is set by how much its lever can do at full
  stretch. The limiter's rule decides only what that protection costs. It also bounded the *where*: a limiter that
  holds its reading exactly at its floor holds it over a stretch of the slam, so "when was the margin lowest" has
  no single answer there. This family's findings stand, because its limiters were not holding where they read.

Those three rungs are told in `53-56-61-airflow-levers.md`, `57-63-schedules.md` and `64-68-valve-and-cascades.md`.

---

## Rung 46 — The temperature limiter protects the wrong compressor

**In one sentence:** a limiter that caps the turbine temperature at a **redline** holds that redline exactly,
but its stall protection goes to the high-pressure compressor, while the low-pressure one, the one that would
stall first, gets none at all.

**How it works.** At every instant the controller checks: would the scheduled fuel push the turbine inlet above
the redline? If so it cuts the fuel to exactly what holds the redline. It takes the lower of the two, the
schedule or the cap. The model makes the limiter react instantly, on purpose, so the finding is about where
the cut falls, not about a reaction delay.

**Why it misses.** In the spec's case (the move taking 0.5 τ, a redline of 1480 K against a peak of about
1645 K with no limiter), the low-pressure compressor comes closest to stall early, while the turbine inlet is
only at about **1374 K**: below any sensible redline. By the time the gas is hot enough for the limiter to act,
that moment has passed and the compressor is already recovering. The high-pressure compressor's worst moment
comes later, near **1558 K**, inside the limiter's window. So the limiter helps the high-pressure compressor
(+0.00279 on one map) and leaves the low-pressure one **identical to the last digit**: up to its worst moment the
two runs are the same run, because the limiter has not yet acted.

Rung 35 said the two limits on a slam, stall and turbine temperature, are coupled. Rung 46 sharpens that: they
share a cause but arrive **in sequence**, and a temperature limiter acts on the later one.

**The exception.** Slam faster and the low-pressure compressor's worst moment slides later and hotter (at
0.15 τ, to about 1745 K). Then the limiter does reach it (+0.01922). It helps most where a slam is most violent.

**On the page.** *Controls → Turbine temperature limiter*, with its *Redline* slider. *Try this → Switch on the
temperature limiter*. In the page's own slam (`docs/sandbox/EXPERIMENTS.md`, experiment 15) it takes the peak
from 1695 K to 1480 K and costs response time: 90 % of the thrust change in 1.28 τ instead of 1.08 τ.

---

## Rung 47 — A limiter that reacts late

**In one sentence:** give the temperature limiter a realistic reaction delay and it can only act later, never
earlier: it still misses the low-pressure compressor, and now it lets the temperature overshoot the redline too.

**How it works.** The amount of fuel the limiter wants to cut now builds up gradually toward its target instead
of jumping there. The hope rung 46 left open was that a "smeared" limiter might reach back to the early
low-pressure moment. It cannot: a delay only postpones. At moderate slams the low-pressure compressor still
gets exactly nothing, and at fast slams the delay wears away the help rung 46 found there.

**The cost.** The temperature now overshoots the redline while the cut catches up: in the spec's case by about
**55 to 191 K**, more as the delay grows. The page shows the same: a delay of 0.2 τ lets the peak reach 1618 K
against the instant limiter's 1480 K (experiment 15). A slow limiter lets most of the overshoot through *and*
still costs response time.

**A detail that matters.** The overshoot comes from the slowness of the *limiter's own loop* (sensing and
deciding), not from the fuel valve. A delay on the valve alone does nothing here, because during a slam the fuel
the limiter allows only rises.

**Later corrections.**
- **Rung 74:** the overshoot is not a law of delays; it depends on *what* is delayed. Delay the amount cut (as
  rung 47 did) and its cases overshoot by **+79 to +83 K**; delay the fuel allowed instead and they stay just
  *under* the redline (by **1.85 to 2.32 K**).
- **Rung 106:** "never reaches the low-pressure compressor" was a matter of timing only. When the throttle asks
  for thrust instead of temperature (rung 95's governor), fuel rises earlier, the limiter starts cutting at
  0.14 τ instead of 0.30 τ, which is before the low-pressure worst moment, and it does buy a small relief.
- **Rung 108:** the redline overshoot belongs to a *slow cut*. Give the limiter a fast cut-in (0.02 τ) and on the
  open throttle it overshoots by only 24 K. Rung 47's other verdict survives: the LP compressor's worst moment still
  comes before the limiter acts.

**On the page.** *…that reacts with a lag*, under the temperature limiter, with its delay in τ. *Try this → A limiter
that reacts slowly*.

---

## Rung 48 — The acceleration schedule: one law for all of it

**In one sentence:** a limiter helps a compressor **if and only if it starts cutting before that compressor's
own worst moment**, and rung 46's and 47's results are both just cases of this.

**How it works.** Real engine controllers do not wait for the turbine to get hot. They cap the fuel flow in
proportion to the pressure the compressor is delivering. That pressure says how much air is passing, so the cap
keeps the fuel in step with the air. One knob sets how far above the steady-running fuel the cap allows. It
acts *ahead* of trouble, from a measurement, rather than reacting to a temperature.

**The finding.** Turning that knob down makes the cap start cutting earlier in the slam, so one dial sweeps the
cutting start across both compressors' worst moments. In the spec's case (the move taking 0.5 τ), the
low-pressure compressor's worst moment is at 0.24 τ, the high-pressure one's at 0.40 τ. Once the cutting starts
after 0.24 τ, the low-pressure help is **exactly zero** while the high-pressure one still gets +0.0075; the
high-pressure help dies only once the start passes 0.40 τ. It is *not* simply a slower throttle in disguise:
at one setting the same cut, removing the same fuel, gives the high-pressure compressor +0.0034 and the
low-pressure one 0.000000.

Rung 46's limiter is late by construction (it waits for heat), and rung 47's delay makes it later still. Both
fall out of this one law.

**The honest edge.** Turn the knob too far down and the cap holds the engine back over the whole slam: it never
finishes accelerating.

**Later.** Rung 76 found that the way the model had computed this cap since rung 48 let through 10.25 K more
peak temperature and 1.2 % more fuel than the schedule as written. Rung 48's findings, all about *when* the cap
cuts in, are untouched.

**On the page.** *Controls → Acceleration schedule*, with *Allowed above the steady line*. In experiment 15 it
peaks at 1546 K and reaches 90 % thrust in 1.46 τ. *Try this → The acceleration schedule*.

---

## Rung 49 — Watching stall itself: cutting helps, handing back hurts

**In one sentence:** a limiter that watches a compressor's distance from stall directly does protect it, but
when it lets go of the fuel it pushes the *other* compressor toward stall, and that harm runs on a different
clock from the help.

**How it works.** A **stall floor**: cut the fuel whenever it would take the watched compressor's flow
coefficient below a floor you set. Unlike every earlier limiter, this one lets go *during* the slam, as soon as
the compressor recovers. So it has two edges:
- **Cutting in** stops a compressor sliding toward stall: a help, exactly rung 48's law.
- **Letting go** hands fuel back all at once, and that sends the compressors toward stall again: a **harm**,
  new.

The help is timed by each compressor's own worst moment. The harm is timed by the **end of the throttle
move**. In the spec's case (the move taking 2.0 τ), letting go near the end of the move did **8×** more harm than
letting go near the high-pressure compressor's own worst moment.

**What it does to rung 48.** Not wrong, but **bounded**: rung 48's law is the special case where the limiter lets
go long after the move, which is all its schedule ever did.

**On the page.** *Controls → Stall floor*, watching the low- or high-pressure compressor (*Low-pressure /
High-pressure*), with the floor in flow coefficient (the stall line is 0.55).

---

## Rung 50 — Letting go, in isolation

**In one sentence:** move only the moment the limiter lets go, and each compressor's worst moment jumps to that
moment, whichever compressor the limiter watches.

**How it works.** This rung is an instrument, not a control: it forces the limiter to let go at a chosen time,
with everything before that unchanged. That pulls the letting-go edge apart from everything rung 49 could only
move together.

**What it found.**
- Letting go is the dominant effect. Whenever it comes after a compressor's own worst moment, that compressor's
  new worst moment is the letting-go moment.
- The harm really is timed by the end of the move (2.6× larger letting go at the end than at the other
  compressor's worst moment, in the spec's case of a 2.0 τ move).
- Forced to let go early, a limiter even harms the compressor it watches (−0.064 in that case).
- Why rung 48's schedule never did harm: not its shape. It simply always let go after the move. Forced to let go
  during it, it harms both compressors like the stall floor.

**Later correction.** Rung 52 showed the "harms the compressor it watches" result needs the forcing: no real
limiter that decides for itself when to let go can do it.

**On the page.** Not shown: it is a measuring instrument, not a control.

---

## Rung 51 — How fast you hand the fuel back

**In one sentence:** hand the fuel back gradually instead of all at once, and the harm depends on *how fast* you
give it back, not on how much fuel you held back.

**The test.** Compare three runs (the spec's case, a 2.0 τ move): handing the fuel back at once at 1.10 τ;
handing it back gradually from 1.10 τ to 1.30 τ; and handing it back at once at 1.30 τ. The gradual run's fuel
lies between the other two at every instant, and so does the total held back. Yet its harm lies *outside* both,
and is smaller: on the low-pressure compressor −0.03121, against −0.05740 and −0.06379. No rule based on the
amount of fuel can give a result outside the range of its two neighbours. The rate is what matters.

**Scope, said plainly.** This holds where the compressors dive deep toward stall. Where they barely dip, the
gradual run lands between the other two, and no claim is made.

**On the page.** Not on its own: it led to rung 52, which is.

---

## Rung 52 — A realistic limiter: cut fast, give back slowly

**In one sentence:** a real limiter cuts fast and gives back slowly, and such a limiter **cannot harm the
compressor it watches**; rung 50's harm was an artifact of forcing.

**How it works.** The amount cut builds up toward its target at one speed going in (*cutting in*) and relaxes
at another letting go (*letting go*). Real limiters are built this way, to protect quickly and then recover
without stirring up the thing they protect. Everything is from the specs' test engine (a two-shaft engine at
cruise, slammed from 1000 K to 1400 K).

**What it found.**
- **It sets its own release.** The release speed is never even consulted before the limiter starts letting go,
  so nothing before that moment can depend on it.
- **It cannot harm what it watches.** Its own cut-in already pinned the watched compressor's worst moment at the
  start of the cut. The search for a counter-example covered 7 floors and 2 slam speeds and found none. This
  holds for the simple, smooth cut-in modelled here; a cut-in shaped differently could escape it.
- **The two speeds are not independent, and the half that fails is the protective one.** The cutting-in speed
  alone decides the help, exactly. The harm to the *other* compressor depends on both speeds together. So the
  letting-go speed cannot be chosen for the other compressor without knowing the cutting-in speed.
- **Slower letting go means less harm**, in the end even a small help (+0.001031 with both speeds at 0.40 τ,
  the 2.0 τ move): rung 51's finding, now on a limiter that could be built.

**Later.** Rung 89 confirmed the fast-cut, slow-release finding on the bleed valve (`86-89-levers-together.md`).
Rung 107 bounded the last point: beside the thrust governor (rung 95) the letting-go harms neither
compressor, so the release-speed finding is about a plain throttle slam. Rung 108 fitted the same cut-fast,
give-back-slowly behaviour to the temperature limiter.

**On the page.** *Controls → Realistic release*, with *Cutting in* and *Letting go* in τ. It shapes the
acceleration schedule's and the stall floor's cut.
