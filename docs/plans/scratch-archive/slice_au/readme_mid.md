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
