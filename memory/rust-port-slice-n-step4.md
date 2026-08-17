---
name: rust-port-slice-n-step4
description: "Slice N step 4 (the rungs 55/56 oracle) — one plan section held two censuses measured on two different grids, and a count derived from a superset is a guess"
metadata: 
  node_type: memory
  type: project
  originSessionId: f3020d26-d553-4cfe-9501-d7447c6aea9f
  modified: 2026-08-17T12:41:57.486Z
---

Slice N step 4 shipped the rung-55/56 value oracle: `rust/tests/slice_n_oracle.rs` +
`rust/oracle/dump_slice_n.py`, **72 520 keys bit-exact against PyPy on the first run**, plus a
5 649-key equilibrium arm and a 41 560-key CPython arm. Five gates.

**The main lesson: a pre-registered census is only as good as the grid it was measured on, and one
plan section can hold two censuses from two grids while reading as one.** § 5.10's `(iv)` table
(which row binds, which stalls) reproduced exactly, because the probe that produced it swept the
same 640 cells. Its `(iii)`/`(vi)` tables — constant liveness, stack construction — came from a
*different* probe sweeping 240 cells with one axis missing, so their numbers were off by 6–14×
while every verdict they carried was still right. **Fix: have Python COUNT on the dump's own grid
and emit the counts as keys, so the Rust compares instead of restating.** Same shape as
[[rust-port-slice-l-step4]]'s copied bar and [[rust-port-guessed-census-bars]], one level up — the
bar was not copied from the source, it was copied from an earlier measurement of a *neighbouring*
grid.

**Three more, each worth the same move — measure, don't derive:**
- **A count derived from a SUPERSET is a guess.** The CPython arm's two argmin-population bars
  were computed as "all discrete keys minus the design ones" (7 481 / 2 280); measured over the
  argmin subset alone they are **4 680 / 1 560**. Every *value* tier passed first run; only these
  two failed.
- **A flag can disarm itself where no count can see it.** The dump's `rows` switch was shadowed by
  a `rows = m.throat_walk(...)` local in the same function, so it stopped applying after the first
  cell: 71 504 keys instead of 41 560 — which beside the full arm's 72 520 reads as *about the
  same*. Only a key-**SET** diff found the 856 missing, all in one shape. See
  [[two-indexes-one-spine]]'s sibling rule: coverage is a name → parameter-set diff, never a count.
- **Measure every detector, even when the dump passes first try** ([[rust-port-slice-n-step2]]).
  Re-spelling the argmin as a last-of-equals fold moved **47 of 72 520 keys and not one value key**
  — index and flag keys only. Perturbing only the interior rows by 2 ULP moved **7 046: 7 040
  per-row, 6 aggregate** — which is what justifies dumping both argmin currencies for every row of
  every half-row rather than for the winning row.

**And one tier that had to split.** [[rust-port-slice-m]]'s CPython rule was *discrete → bits,
because a difference is a BRANCH and not drift*. Here **520 discrete keys differ between
interpreters and every one is an argmin index at the design throttle**, where all the per-row
margins collapse to 1–2 ULP and which row wins is the last bit of an accumulation. So argmin keys
are held to bits off design (4 680, zero differing) and *counted* at design. Drift can wear a
branch's clothes.
