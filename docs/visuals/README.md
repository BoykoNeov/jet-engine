# Interactive visuals

A single self-contained HTML page that makes the model visible: an animated
engine cutaway (flow particles colored by local total temperature), the
ideal-vs-real T–s diagram, the NOx bell + finite-quench story (rungs 7–10, 19),
the mixing-optimum J-sweeps (rungs 11–13), the rung-22 J→C collapse animation,
the rung-23 ξ–τ correlation, the rung-17/20 exhaust-NO clamp ladder, and the
click-to-expand 23-rung map.

**Every curve is computed by the model in this repository** — nothing is
sketched. Open `turbojet-visuals.html` in a browser (it is fully offline,
light/dark aware, keyboard-navigable, and every chart has a data-table twin).

## Regenerating

```
python extract_data.py    # runs the turbojet package at the design point (~10 min)
python build.py           # splices data.json into template.html
```

- `extract_data.py` — runs the actual `turbojet` package at the rung-1 design
  point (M0=0.85, pi_c=10, Tt4=1500 K) and dumps `data.json`.
- `template.html` — the page (markup, styles, chart code) with a
  `/*__DATA_JSON__*/` placeholder.
- `build.py` — splices the two into `turbojet-visuals.html`.

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
