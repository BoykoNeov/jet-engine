---
name: sandbox-slice2
description: "Web sandbox slice 2 'Size the blades' SHIPPED 2026-10-07 — process lessons: 'it runs on the table gas' hid a scalar gamma; classify failures through the PRODUCT's request path; a browser wait satisfied by the PREVIOUS run; the CLI panel sat on a different rig; backslash hazard x3"
metadata:
  node_type: memory
  type: project
  originSessionId: a4e9cec9-7dea-4c97-8456-715f0ddb3203
  modified: 2026-10-07T02:47:56.845Z
---

Slice 2 of the web sandbox ([[sandbox-direction]], after [[sandbox-slice1]], [[sandbox-slice3]]):
the "Size the blades" view of `docs/sandbox/turbojet-sandbox.html` — rung 85 as knobs on its OWN
two-spool engine. Plan § 11 of `docs/plans/sandbox-plan.md` (§ 11.8 = what the build found).
Code `rust/src/sandbox_blades.rs`, tests `rust/tests/sandbox_blades.rs`.

**User decisions (2026-10-07):** a separate two-spool view; BOTH gases (perfect + thermally
perfect, after a sizing fix); material as a number with Ti-6Al-4V the one cited preset; one blade
knob set with an "unlink" switch.

**Process lessons.**
- **"It runs" is not "it is right" — ask what SUPPLIES the value** ([[instrument-fed-by-what-it-certifies]]).
  The table gases ran through rung 85's sizing, but `size` read the gas's SCALAR gamma (the spec
  default 1.4) while the work came from the tables. Fixed (`face_props`), pinned to a published air
  table — and the bar had to allow rung 3's own NASA-fit offset (0.45 % at 250 K) while still
  separating the old path (tamper proved it). Same scalar-gamma shortcut lives in rung 55's stack:
  labelled, not changed (not agreed).
- **Classify failures through the product's own path.** A raw-model sweep and a sweep through
  `sandbox::call` gave different pictures: the pre-test (`try_match_point`) turned panics into
  refusals and changed WHICH message arrives; "jet below outside pressure" turned out to be a lever
  THROTTLE failure, not a design one, so the first plain words were wrong.
- **A browser wait can be satisfied by the PREVIOUS run.** Twice: a "9 points done" counter still
  read 9 before the new sweep reset it; a blade result passed a "no shaft speed ⇒ design result"
  test. Wait on something unique to THIS request (its first throttle, a field only it has).
- **A published panel can sit on a different rig.** The CLI panel's gas constant (286.9) is not the
  test rig's (286.857); at lambda 1 it moves one printed digit. Check the rig before pinning printed
  numbers; pin the cause, not a loosened bar.
- **Screenshots again** found five layout faults behind 27 green browser checks ([[sandbox-slice1]]).
- **Backslash hazard x3** ([[windows-tooling-file-hazards]]): `\t` in a path became a TAB, `\n` in a JS
  string became a real newline (broken script), a Rust `\` line continuation vanished (runs of
  spaces — now guarded by a test). All from Python heredoc edits; the Edit tool had none.
