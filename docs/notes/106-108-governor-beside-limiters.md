# Rungs 106–108 — The governor beside the limiters

*Plain-language notes. The formal derivations, tests and every number's source are in `docs/rung106-spec.md` to
`docs/rung108-spec.md`. Numbers below are copied from those specs and belong to the cases the specs ran; the
sandbox page shows its own.*

Rung 95's governor asks for thrust and pours in fuel to get it; rungs 46–52's limiters hold fuel back to protect
the engine. In a real control both run at once, and the lower fuel wins. Rung 95 put the governor beside *instant*
limiters. These three rungs put it beside the *realistic*, slow ones, and find that a slow limiter **stores up** a
cut it pays back after the move. The thrust climbs, then **sags**.

Time is in shaft response times, **τ**; the thrust "step" is the change in thrust the move asks for.

---

## Rung 106 — The governor beside the slow temperature limiter

**In one sentence:** beside a temperature limiter that reacts with a delay, the governor makes the engine both
hotter on the way up and slower to settle, because the limiter's delayed cut is paid back after the move.

**How it works.** Rung 47's limiter builds its cut up gradually toward what the redline needs. Beside the governor,
that "need" is measured against the governor's demand, which rises faster than the plain ramp. So during the move
the limiter falls further behind (letting more through) and builds up a bigger cut (storing more). Control
engineers call this **windup**: a correction built up while it could not act, released later.

**What it found** (the spec's default slam):
- **Hotter.** The redline overshoot about doubles: 272 K against rung 47's 138 K at the default delay.
- **Then a sag.** After the move the governor's demand falls back, and the stored cut lags behind it. The thrust
  reaches 83 % of its step at the end of the move, *falls* to 78 %, and reaches 95 % at 1.53 τ against 1.32 τ with
  an instant limiter.
- On a plain ramp, or with an instant limiter, the thrust never falls.

**It also moved rung 47's verdict.** Beside the governor the limiter starts cutting earlier (0.14 τ instead of
0.30 τ), before the LP compressor's closest approach to stall, so it now buys that compressor a little relief.

**On the page.** *Controls → Holds thrust, Govern during the move*, then *Turbine temperature limiter* and *…that
reacts with a lag*.

---

## Rung 107 — The governor beside the realistic release

**In one sentence:** beside rung 52's cut-fast, give-back-slowly limiter, the thrust sags too, but at the page's
old slowest cut-in the sag belongs mostly to the slow *cut*, not the slow give-back.

**How it works.** The limiter here is rung 52's on the acceleration schedule (or the stall floor), cutting in at
one speed and letting go at another.

**What it found.**
- The limiter cuts more than twice as deep beside the governor (2.35–2.40×), and earlier.
- **With a cut-in of 0.2 τ** (the page's floor when this rung was built), fuel gets through before the cut catches
  up, so the engine *runs ahead*: 78 % of its thrust step at the end of the move, against 33 % with an instant
  schedule. Then the governor's demand falls back and the thrust falls with it, by 3.3–4.9 % of the step. A slower
  give-back deepens that only a little (0.4 points at 0.4 τ, 1.1 at 1.6 τ).
- **With a fast cut-in** the give-back does decide the sag: at a cut-in of 0.005 τ the sag is 0 / 0.5 / 1.5 / 3.0 %
  for a give-back of 0.005 / 0.1 / 0.4 / 1.6 τ. A real limiter cuts in fast, so this is the realistic case; the page
  has since lowered its floor to 0.02 τ.
- **What a slow give-back costs is time:** the thrust reaches 95 % of its step at 1.17, 1.56, 2.33 τ for a give-back
  of 0.2, 0.4, 0.8 τ.

This holds for the governor that holds thrust directly; holding speed, or reading thrust from a gauge, the picture
changes.

**On the page.** *Controls → Govern during the move* with *Realistic release*: *Cutting in* and *Letting go*.

---

## Rung 108 — The two-speed temperature limiter

**In one sentence:** give the temperature limiter a fast cut and a slow give-back, and the fast cut ends most of
the overshoot, but it stores more, so beside the governor the slow give-back is paid in **time to full thrust**.

**What it found.**
- **The overshoot mostly goes.** At a cut-in of 0.02 τ the governed slam runs 59–63 K over the redline, against
  265–282 K for rung 106's single delay of 0.2 τ (and 0 K for an instant limiter). On the open throttle, 24 K
  against 135–141 K.
- **The sag stays.** The fast cut follows its target to a higher peak, so there is more to give back. Given back
  slowly, the thrust still sags after the move (2.0–4.0 % of the step at a 0.02 τ cut-in) and reaches 95 % of its
  step at 2.1–2.2 τ, against 1.2–1.3 τ for the instant limiter (give-back 0.2 τ). With a give-back of 0.4 or 0.8 τ it
  does not reach 95 % within the 2.5 τ run at all.
- On the open throttle it never sags.

**On the page.** *Turbine temperature limiter → …that reacts with a lag → …cuts fast, gives back slowly*, with
*Cutting in*; the lag above it becomes the give-back time.
