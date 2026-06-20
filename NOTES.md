# NOTES — Turbojet Cycle, in plain language

A running, plain-language companion to the code: what each station does and *why*,
written to learn from. Built up station by station as the cycle is derived. See
`SPEC.md` for the formal equations and `CLAUDE.md` for the working contract.

## The big picture
A turbojet is a **Brayton cycle**: squeeze air (compressor), burn fuel in it
(burner), let it expand through a turbine, and shove what's left out a nozzle
fast enough to make thrust. We track the gas as a **total (stagnation) state** —
`Tt`, `pt` — at six numbered stations (0, 2, 3, 4, 5, 9).

"Total" means *"what the gas would read if you brought it gently (isentropically)
to rest."* That number already includes the energy tied up in the flow's motion,
so we can reason about the whole cycle without carrying velocities around — right
up until the nozzle, where velocity is the entire point and we finally convert
back to **static** (`T`, `p`, `V`).

## Station 0 — Freestream (the air before the engine touches it) ✅
**What it is:** the ambient air the aircraft flies into — its static temperature
and pressure (`T0`, `p0`) and the flight Mach number `M0`.

**What we compute:** the *total* temperature and pressure of that air, plus the
flight speed `V0`.

- `Tt0 = T0 · (1 + (γ-1)/2 · M0²)`
- `pt0 = p0 · (1 + (γ-1)/2 · M0²)^(γ/(γ-1))`
- `V0  = M0 · √(γ R T0)`

**Why (the physics):** stand on the engine. The air rushes at you at `V0` and you
bring it to rest. That kinetic energy can't vanish — it reappears as a rise in
temperature and pressure. This is the **ram effect**. `Tt0`/`pt0` are exactly the
state the air reaches once stopped. The shared factor `(1 + (γ-1)/2 · M0²)` is the
bookkeeping for *"how much hotter and denser does stopping the flow make it."*

**Why it matters downstream:** because the totals already bank the ram energy, the
*ideal inlet* (station 2) has nothing left to do but pass them through unchanged —
which is exactly what we'll see next. And `V0` is the speed the exhaust must beat:
thrust is the engine throwing mass out the back *faster than* `V0`.

**Numbers (validation case):** `T0`=250 K, `p0`=50 kPa, `M0`=0.85 →
`Tt0`=286.1 K, `pt0`=80.19 kPa, `V0`=269.4 m/s. ✓ matches the spec table.

---
*Next: Station 2 (inlet), then 3, 4, 5, 9, then the shaft balance and what it
buys us. Each gets a section here as it's derived.*
