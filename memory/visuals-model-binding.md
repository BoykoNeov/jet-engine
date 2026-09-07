---
name: visuals-model-binding
description: tests/test_visuals_data.py binds docs/visuals/ to the model — and being IN SYNC is not the same as being BOUND
metadata: 
  node_type: memory
  type: project
  originSessionId: 8a730ff9-b3d5-49e7-abba-db5f62833a49
  modified: 2026-09-07T08:32:13.767Z
---

On 2026-09-07 the two published pages in `docs/visuals/` (charts + engine
cutaway, see [[visuals-artifact]] and [[cutaway-artifact]]) had **no test at
all**. Their numbers happened to still agree with the model to 3.3e-6 — which is
just `extract_data.py`'s 6-significant-figure rounding — so nothing looked wrong.
That is the trap: *in sync* and *bound* are different properties, and only the
second survives the next cycle change.

`tests/test_visuals_data.py` now gates the **joints** (12 tests, ~1.5 s, in the
fast subset): the built `.html` is the splice of its template and `data.json`;
`data.json`'s design point equals `main.py`'s `FLIGHT`/`PI_C`/`TT4`/`REAL_LOSSES`
(`extract_data.py` re-declares them); the `ideal`/`real` blocks are recomputed
live at 1e-5 relative; every field the cutaway reads survives `build_cutaway.py`'s
`KEEP` trim, and every dumped field is kept or named as deliberately dropped. The
~10-minute sweep blocks are shape-checked only and never recomputed.

**Why:** a page whose whole claim is "these are the model's numbers" needs the
claim checked, and the failure was silent in both directions — a stale dump has
no symptom, and a field trimmed out of `KEEP` renders `undefined` in a browser
where no Python test can see it.

**How to apply:**
- Three lessons the build itself taught, worth carrying forward:
  1. **The page's own statement of which engine it shows was typed markup.** The
     design-point chips and the loss list were literal HTML. They are now
     rendered from `DATA.design`, so moving the project's design point moves the
     page. Look for typed physical constants in any page that claims to be
     data-driven — the census that finds field *reads* (`I.x`, `D().x`) will not
     find them.
  2. **A fact I "handled" was inferred, not measured.** I wrote a both-None-or-
     both-numbers branch for `far` on the strength of the page showing 0 upstream
     of the burner. `FlowState` declares `far: float = 0.0` — never None. The
     page's 0 was the *page's* display choice. Mutating my own gate is what
     exposed the dead branch; a second mutation that "passed" turned out to be my
     harness replacing a string in the template *and* the built file, so the
     splice still matched. **Mutate every gate, then check the mutation was real.**
  3. **`build.py` writes without `newline="\n"`** while `build_cutaway.py` pins
     it, so `template.html` / `turbojet-visuals.html` are CRLF on disk and
     `data.json` / the cutaway files are LF. Compare with `read_text()`, never
     bytes. Same hazard as [[windows-tooling-file-hazards]].
- **Rebuild is only half the loop — republish too.** A rebuilt file in the repo
  does not update a published page; `docs/visuals/README.md` and CLAUDE.md's
  § Layout now both say so.
- What is deliberately *not* gated: the sweep curves' digits (illustration
  grids), the cutaway's geometry / blade counts / primary-zone temperature (drawn
  — the page footer now carries that split for readers who never see the repo),
  and the rung map's coverage. The ladder is at rung 84, the computed panels
  cover 1–23 and the map lists 29; a gate on that gap would fire on every new
  rung, so it is a stated scope, not a check.
