# Rungs 69–84 — How control loops combine (one short page)

*Plain-language summary. These sixteen rungs are a careful study of control loops and of the model's own solver.
Their formal records are `docs/rung69-spec.md` to `docs/rung84-spec.md`. None of them is shown on the sandbox page,
and none changes an engine number; the roadmap counts them as correct but far from what a user can see. This page
says what they asked and what came out, without the algebra.*

By rung 68 the model had up to five controllers on one engine: a fuel limiter, a temperature limiter, a stator
limiter, a bleed valve, and the throttle's own schedule. Rungs 66–68 had found that loops holding the *same*
reading at the same target behave as one loop. These rungs ask the general question: when several loops act at
once, how many of them really count, and how far can the numbers the solver reports be trusted?

---

## Part 1 (rungs 69–71) — What a loop watches decides whether it counts

- **Rung 69.** Re-aim rung 68's stator limiter at the blade angle instead of the flow coefficient, change nothing
  else, and it stops being redundant: it now adds real authority. The number of loops that genuinely count is the
  number of *independent things being held*, not the number of loops. Freeing a loop this way can create an
  oscillation, but only where one lever fights itself, and it is always damped.
- **Rung 70.** Swap in the turbine temperature governor as the odd loop out. The pairs it frees are again damped to
  exactly the level rung 67 found; the oscillation needs one lever to be reading two limits at once.
- **Rung 71.** All three loops watching different things: in the arithmetic all three count, yet the third one acts
  over only 7.9 % of the slam, because holding the second limit already implies the third. "Counts in the
  arithmetic" and "ever gets to act" are different things.

## Part 2 (rungs 72–76) — Loops that share one fuel valve

A real fuel control picks the **lowest** of the fuel amounts its limiters allow. So only one limiter is in charge
at any instant; the others are overruled.

- **Rung 72.** Two fuel limiters sharing the fuel valve do not make a new loop. At every instant the engine is one
  of the earlier rungs' engines, according to which limiter is in charge, and it switches at the hand-over without
  anything else moving. Four loops all in charge at once is impossible.
- **Rungs 73–75** each tried a way round that: let the overruled limiter read the fuel actually burnt (73), delay the
  fuel it allows rather than the amount it cuts (74), or give it a standard "anti-windup" device that stops an
  overruled controller drifting (75). Each changed something real (rung 74, for instance, showed that rung 47's
  temperature overshoot depends on *what* is delayed), but none let a fourth loop count.
- **Rung 76** found why. A device inside a limiter's own rule can only reach the limiter that is overruled; a device
  in the engine both limiters read (a sensed fuel cap) can only reach the one in charge. The obstacle is the
  take-the-lowest selection itself. Every route tried is closed.

## Part 3 (rungs 77–84) — How far the solver's own numbers can be trusted

These rungs read the model's results rather than adding physics.

- **Rungs 77–79.** A sensitivity that looked like a gain the controller buys turned out to be the slope of one
  limiter's own equation (77); that slope is a free dial that can be set to anything without the engine noticing,
  though a second solution appears where it crosses zero (78); and a change of the reading a limiter uses cannot
  reach the engine, because whenever that limiter is in charge the original reading is what it uses (79).
- **Rung 80.** A run that cannot start accelerating (rung 74's "arrest") comes from two limits being set at the
  *same* level. Give the airflow levers their own limit and it goes away.
- **Rung 81.** Which of two fuel limiters is in charge is decided by their delays, not only by their set points:
  slow the fuel limiter enough and it takes over. And a limiter that is overruled for the whole slam can have its
  own delay changed tenfold without moving the run by a single digit.
- **Rungs 82–84.** How to find the setting at which one limiter takes over from another. A forecast read forward
  from a starting guess errs toward the side that guess was on; only the self-consistent answer lands (82). A search that brackets a change of sign finds a jump, not a true
  crossing, and on one of five slams there is no crossing at all (83). The reason is the simulation's own time step:
  the lowest point of a stepped run moves in small jumps as the setting changes, a sawtooth riding on the true
  curve (84).

**Later checks** (not rungs; `docs/rungs72-77-march-audit.md`, `docs/rung74-arrest-interval.md`,
`docs/rung79-gap-margin.md`) found the "arrest" is a band of settings rather than one, that no run the project
ships actually arrests, and corrected parts of rungs 78 and 79.

**The lesson the roadmap draws.** These results are sound and general, but they are about control structure and
the solver rather than the engine. Later rungs, from rung 85 on, went back to effects a user can see by moving a knob.
