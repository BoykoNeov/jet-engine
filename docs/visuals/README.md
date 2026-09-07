# Interactive visuals

A single self-contained HTML page that makes the model visible: an animated
engine cutaway (flow particles colored by local total temperature), the
ideal-vs-real T–s diagram, the NOx bell + finite-quench story (rungs 7–10, 19),
the mixing-optimum J-sweeps (rungs 11–13), the rung-22 J→C collapse animation,
the rung-23 ξ–τ correlation, the rung-17/20 exhaust-NO clamp ladder, and the
click-to-expand 29-rung map.

The computed panels cover rungs 1–23. Rungs 24–29 (the locally-resolved mixing
time, the finite-rate / freeze-out / NO-freeze-out / coupled-march nozzle
chain, and the shifting turbine) appear in the rung map only — they are not
plotted here. Their numbers
live in `main.py`'s per-rung panels and in `tests/`.

**Every curve is computed by the model in this repository** — nothing is
sketched. Open `turbojet-visuals.html` in a browser (it is fully offline,
light/dark aware, keyboard-navigable, and every chart has a data-table twin).

## Regenerating

```
python extract_data.py    # runs the turbojet package at the design point (~10 min)
python build.py           # splices data.json into template.html
python build_cutaway.py   # splices a trimmed data.json into cutaway-template.html
```

Then **republish both artifacts** — the pages live at fixed URLs recorded in
`memory/visuals-artifact.md` and `memory/cutaway-artifact.md`, and rebuilding a
file in the repo does not update a published page.

- `extract_data.py` — runs the actual `turbojet` package at `main.py`'s design
  point (the rung-1 case: M0=0.85, pi_c=10, Tt4=1500 K) and dumps `data.json`.
- `template.html` — the page (markup, styles, chart code) with a
  `/*__DATA_JSON__*/` placeholder.
- `build.py` — splices the two into `turbojet-visuals.html`.

## What binds these pages to the model

`tests/test_visuals_data.py` runs in the ordinary `pytest` gate and checks the
**joints**, not the physics:

- **The built pages are the splice of their template and `data.json`.** Catches a
  hand-edited `.html`, and a template or data change committed without a rebuild.
- **`data.json`'s design point is `main.py`'s.** `extract_data.py` now *imports*
  `FLIGHT` / `PI_C` / `TT4` / `REAL_LOSSES` rather than keeping its own copy, and
  the gate pins both that import and the committed dump — the import stops the two
  declarations diverging, the dump check catches a `data.json` left behind after
  the design point moved.
- **The `ideal` / `real` blocks are recomputed** against a live
  `build_turbojet(...).run(...)` and compared at 1e-5 relative — the floor set by
  `extract_data.py` rounding its dump to 6 significant figures. This is what makes
  "every number is the model's" a checked claim rather than a promise.
- **Every field the cutaway reads survives `build_cutaway.py`'s `KEEP` trim**, and
  every dumped field is either kept or named as deliberately dropped. A trimmed
  field renders `undefined` in the browser and is invisible to every other test.
- **The sweep blocks are checked for shape only** — present, non-empty, finite.
  They cost ~10 minutes to produce and are never recomputed by a test.

The cutaway also **renders** the design point and the loss factors from
`data.json` rather than carrying them as typed markup, so moving the project's
design point moves that page's chips and footer with it.

Deliberately *not* bound: the sweep curves' numbers (illustration grids, below);
the cutaway's geometry, blade counts and primary-zone temperature (drawn — the
page's own footer says which is which); and the rung map's coverage — the ladder
is at **rung 84** while the computed panels cover **1–23** and the map lists
**29**. That last one is a scope statement, not a defect, and nothing gates it: a
gate on it would fire on every new rung.

## Honesty note

Sweep grids are **reduced** vs the production defaults (e.g. quench
`ngrid=60/nsteps=400` vs 240/2000) — these are illustration curves, shape not
digits, matching the spirit of main.py's coarse rung-17 panel. The verification
gates live in `tests/`, not here. The clamp ladder uses the rung-17 panel's own
design point (rich phi_p=1.5, J=225).

One grid goes the other way: the rung-23 dwell sweep uses `n_quad=120`, above
main.py's 56, because 56 trips the beta-PDF mean-preservation assert at J=36 —
`g` is not monotone in J (it collapses to `g_min` at C_opt and climbs back out,
rung 22's own result), and the quadrature is hardest to converge on that climb.
main.py's J grid steps 16 -> 64, straight over the spot. The correlation signal
is only ~1-5%, so the same rule runs at every point.

## The engine cutaway page

`turbojet-cutaway.html` is a second, stand-alone page: an animated 2-D
cutaway of the single-spool engine (rotor rows sweeping on one shaft, fixed
stators, spinner and shaft helices, a two-zone annular combustor with fuel
spray, flame and dilution jets, a convergent–divergent nozzle and plume),
with air parcels colored by temperature, a hover probe, the six station
readouts, the ideal-vs-real T–s diagram and a performance table. Its numbers
are the `ideal` / `real` blocks of `data.json`, and it renders the design point
and losses from that same block; its geometry, parcel speeds, blade counts and
the primary-zone temperature are illustrative, and the page's own footer carries
that split for a reader who never sees this file.

```
python build_cutaway.py   # splices data.json into cutaway-template.html
```
