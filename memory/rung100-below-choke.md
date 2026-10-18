---
name: rung100-below-choke
description: "Rung 100 (formula below choke + slow grid slices) process lessons — measure a user-chosen approximation BEFORE freezing and answer their 'why'; one exact term can remove a compensating error; pick scoring variants whose own miss doesn't dwarf the bar; a new check point can trip a wasm/native bar"
metadata:
  node_type: memory
  type: project
  originSessionId: 8918ac28-5a13-4265-984a-bd8007156ca4
  modified: 2026-10-18T20:18:19.696Z
---

Rung 100 (2026-10-18): `unchoked_formula.rs` + two slow slices in `mach_axis::GRID`; anchor `4b5c769`; 242 HIT,
35 MISS, 40 VACUOUS of 317. Follows [[rung100-decisions]], [[rung99-speed-axis]].

**Lesson 1 — measure the user's chosen approximation before the anchor, and show them the why.** The user picked a
fan-speed approximation (A) "to be measured". The pre-anchor probe found it removes only 25–40 %; I offered a second
device (B) and the user asked WHY. Answering with a measured decomposition (true HP side → LP re-solve exact to 1e-13,
so the whole miss is the engine leaving the table's one-parameter line) led the user to a better fix (slow grid
slices: 33 % → 0.6 %). A scope question answered with a measurement beats one answered with a guess.

**Lesson 2 — making one term exact can REMOVE a compensating error.** H4 (0/3): at Mach 1.4 from 700 K rung 99's push
and airflow came from the same unchoked design row and cancelled; the exact push left the airflow guess alone
(−1.7 % vs +0.09 %). Before claiming "exact X ⇒ better", ask whether X's old error was correlated with another term.

**Lesson 3 — a scoring variant whose OWN miss exceeds the bar measures nothing about the device.** H1's 28 misses were
all the LP generator on the fan-speed reading: +4…+10 % even with tables made at this flight (rung 96's clean
table on a loaded fan). Check a new variant's miss at the reduce setting (tables at this flight) before scoring it.

**Lesson 4 — a new check point can trip a wasm/native bar without any code being at fault.** The schedule request at
Mach 0.1 from 700 K read 1.36e-11 at its unchoked STEADY START (reads no table); re-measured, bar 1e-11 → 1e-10 with
the reason in `check.mjs`. Also: `browser.mjs` compared `settled_*` relatively while `check.mjs` did absolutely — a
near-zero gap exposed it.

**Tooling:** a perl `-0pi` substitution that silently fails to match leaves the rest of the edit referencing a name
never defined — grep the result after every scripted edit.

**Gate note:** in the full gate two `browser.mjs` knob-kill checks failed under the 8-at-a-time load (the run done before the knob moved) and passed alone — no earlier record; recorded in sandbox-plan § 32 as a pair to watch.
