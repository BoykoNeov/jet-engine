---
name: rung101-airflow-source
description: "Rung 101 (the formula's airflow from the table / speed tables / exhaust sensor + lag) process lessons — read the build path before claiming what a probe's start is; a probe at one regime generalizes badly across choke; a shared root search can reach where only the new device fails; gate what the loop does NOT hold"
metadata:
  node_type: memory
  type: project
  originSessionId: bf0d7d09-e096-4e3f-8c52-c91f6b04c6b1
  modified: 2026-10-19T02:26:42.483Z
---

Rung 101 (2026-10-19): `airflow_source.rs`; anchor `604f6a2` (Status corrected `eab1563`); 778 HIT, 46 MISS,
56 VACUOUS of 880 (first score 746/84/50). Follows [[rung101-decisions]], [[rung100-below-choke]].

**Lesson 1 — read the build path before writing what a probe measured.** The anchor first said the probe's steady
start "is the device's own"; the advisor had warned it might not be, and the code showed rung 98 CALIBRATES each end on
the belief, so a new belief moves the start. Corrected in a second anchor commit before any code. Check how the run's
ends are made before claiming a start-size transfers.

**Lesson 2 — a measurement taken in one regime does not generalize across a regime boundary.** H8 (the speed tables make
the fan speed WORSE) was measured only at 700 K starts, all below choke; 42 of 80 scored cells missed, every one where
the start and its design rows were choked — there B's push is exact and the grid removes error. Ask "which side of
choke (or any switch) did the probe sit on?" before freezing a claim at new cells.

**Lesson 3 — a shared search can walk into where only the new device fails.** Rung 94's end root walks up from 600 K;
the bare reading answers there, the sensor-fed fan formula does not (its extrapolated row has no LP solution) — 19 cells
refused in the first score. Fixed by starting at the set throttle; both scores archived, diffed (only those 19 lines).
When a new belief replaces a cheap read inside an old search, check the search's whole range, not the root.

**Lesson 4 — gate the quantity the loop does NOT hold.** The lag-trend gate first compared BELIEFS: the governor holds
the belief on the demand line, so they were 3e−12 apart whatever the lag; the lag lives in the true thrust. And its bar
(ratio 2, "first order") was typed — measured 1.32–1.69 (sublinear: the exhaust temperature JUMPS at the move's start).
[[rust-port-guessed-census-bars]] again.

**Tooling:** a perl `-CSD` edit with a UTF-8 literal double-encoded CLAUDE.md's em-dashes (reverted, redone with Edit);
plain `python` hung (the Store stub) — never call it. The census caught a NaN in a reply (the "demand" column read the
sensor's demand without the instant) that no unit test reached.
