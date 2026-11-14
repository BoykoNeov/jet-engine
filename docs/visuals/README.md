# Interactive visuals

A single self-contained HTML page that makes the model visible: an animated
engine cutaway (flow particles colored by local total temperature), the
ideal-vs-real T–s diagram, the NOx bell + finite-quench story (rungs 7–10, 19),
the mixing-optimum J-sweeps (rungs 11–13), the rung-22 J→C collapse animation,
the rung-23 ξ–τ correlation, the rung-17/20 exhaust-NO clamp ladder, and a
click-to-expand map of **every** rung, grouped by family.

The computed panels cover rungs 1–23. Every later rung appears in the rung map
only — it is not plotted here. Their numbers live in the CLI's per-rung panels
(`cargo run --release`), in `rust/tests/`, and (for rungs 25 on) on the
sandbox page `docs/sandbox/`.

## The rung map is generated (roadmap A5, 2026-11-15)

Until 2026-11-15 the map was a hand-kept array of the first 29 rungs, 79 rungs
behind the model. It is now written at splice time by `rust/src/rung_map.rs`
into the template's second placeholder, `/*__RUNG_MAP__*/`, from:

- **`CLAUDE.md`'s rung table** — which rungs exist, their order (sorted by
  number: the table lists 73 before 72, and 2b sorts after 2), each one's spec,
  and a fallback title and text (the row's hook, split at its first ` — `);
- **its families line** — the foldable groups; **its "Read-only diagnostics"
  line** — the "read-only check" label (1–6 are "cycle", the rest "engine
  behaviour"); **its "current scope" sentence** — the count in the heading;
- **the plain-language notes** — a rung whose note has a
  `## Rung N — title` section with an `**In one sentence:**` line shows that title
  and sentence instead of the table's hook (rungs 25–108 all do; 69–84 use the
  `- **Rung N.**` bullets of their one-page note, or the shared `**Rungs A–B**`
  bullet, label kept). Rungs 1–24 show the table's hook and point at `NOTES.md`
  where it has a section (not for 17, 20, 21, 23, 24).

So **a new rung, an edited rung row, or an edited note changes the page**, and
`visuals_html_is_the_current_splice` fails until it is re-spliced
(`cargo run --release -- splice`) — then republish. `rust/tests/rung_map.rs`
holds the generator to facts it does not compute (a plain count of table rows,
CLAUDE.md's second scope sentence, hand-checked spot pins) and the template to
typing no rung at all.

**Every curve is computed by the model in this repository** — nothing is
sketched. Open `turbojet-visuals.html` in a browser (it is fully offline,
light/dark aware, keyboard-navigable, and every chart has a data-table twin).

## Regenerating

From the repository root:

```
cargo run --release --manifest-path rust/Cargo.toml -- visuals   # run the model, write data.json, splice both pages (~20 s)
cargo run --release --manifest-path rust/Cargo.toml -- splice    # template-only edit: re-splice both pages from data.json
```

That is `rust/src/visuals.rs` (phase 8 slice AR, `docs/plans/todo-rust-port.md` § 8.9). It
replaced three Python scripts — `extract_data.py` (ran the model, wrote `data.json`), `build.py`
and `build_cutaway.py` (the two splices) — and wrote `data.json` and both pages
**byte-identical** to them before they were deleted at slice AU (they are kept at the git tag
`python-final`).

Then **republish both artifacts** — the pages live at fixed URLs recorded in
`memory/visuals-artifact.md` and `memory/cutaway-artifact.md`, and rebuilding a
file in the repo does not update a published page.

- `template.html` — the charts page (markup, styles, chart code) with a
  `/*__DATA_JSON__*/` placeholder; `cutaway-template.html` — the cutaway page, which
  receives a trimmed copy (`visuals::KEEP`).
- The design point is the CLI's (the rung-1 case: M0=0.85, pi_c=10, Tt4=1500 K).

## What binds these pages to the model

`rust/tests/visuals.rs` runs in the ordinary `cargo test --release` gate and checks
the **joints**, not the physics:

- **The built pages are the splice of their template and `data.json`.** Catches a
  hand-edited `.html`, and a template or data change committed without a rebuild.
- **`data.json`'s design point is the CLI's.** `visuals.rs` *imports* the panels'
  `flight` / `PI_C` / `TT4` / `real_losses` rather than keeping its own copy, and
  the gate pins both that import and the committed dump.
- **The whole of `data.json` is regenerated** (~20 s) and compared byte for byte,
  so a model change that leaves any block stale — the sweeps included — fails a
  test. The Python could afford only the cycle blocks.
- **Every field the cutaway reads survives the `KEEP` trim**, and every dumped
  field is either kept or named as deliberately dropped. A trimmed field renders
  `undefined` in the browser and is invisible to every other test.
- **Every `getElementById(...)` target is declared in the same template.** An
  unguarded lookup (`getElementById('chips').innerHTML = ...`) throws when the div
  is renamed and takes the rest of the script with it; a guarded one renders
  nothing. Both are invisible to the splice check, which compares the built page
  against the template that already has the defect.

Its template censuses are hand-written (the crate has no regex dependency), so
each is pinned to the exact set Python's own pattern found on the committed
templates.

**Both pages now RENDER the design point** rather than carrying it as typed
markup: the cutaway's chips and loss footer (`paintChrome()`), and the charts
page's figure subtitle, footer provenance paragraph and T-s lede
(`paintDesignPoint()`). Moving the project's design point moves both pages with
it. This is gated separately because the splice check structurally cannot see
it — a constant typed into a *template* is copied into the *built page*, so the
splice matches perfectly while the sentence is wrong.

Deliberately *not* bound: the sweep curves' numbers (illustration grids, below);
the cutaway's geometry, blade counts and primary-zone temperature (drawn — the
page's own footer says which is which). The rung map's COVERAGE is now bound
(above): it is generated, and the splice gate fires on every new rung by design.

What the rung map's words say is the source's: the notes' sentences are held to
their specs by `docs/notes/check-numbers.ps1`; the table hooks shown for rungs
1–24 are `CLAUDE.md`'s own rows.

## Honesty note

Sweep grids are **reduced** vs the production defaults (e.g. quench
`ngrid=60/nsteps=400` vs 240/2000) — these are illustration curves, shape not
digits, matching the spirit of the CLI's coarse rung-17 panel. The verification
gates live in `rust/tests/`, not here. The clamp ladder uses the rung-17 panel's own
design point (rich phi_p=1.5, J=225).

One grid goes the other way: the rung-23 dwell sweep uses `n_quad=120`, above
the CLI's 56, because 56 trips the beta-PDF mean-preservation assert at J=36 —
`g` is not monotone in J (it collapses to `g_min` at C_opt and climbs back out,
rung 22's own result), and the quadrature is hardest to converge on that climb.
The CLI's J grid steps 16 -> 64, straight over the spot. The correlation signal
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
cargo run --release --manifest-path rust/Cargo.toml -- splice   # re-splices both pages from data.json
```
