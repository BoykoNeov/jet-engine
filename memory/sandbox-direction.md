---
name: sandbox-direction
description: "User decision 2026-10-06: the project's direction is a SANDBOX (change components/designs, watch the engine respond), not separate pass/fail lessons; new effects ship as knobs, no NEGATIVE gates; planned form is an interactive web page running the Rust model live"
metadata:
  node_type: memory
  type: project
  originSessionId: fc49c174-1ea6-4775-96a6-b721880afed4
  modified: 2026-10-06T21:55:30.488Z
---

**Decision (user, 2026-10-06):** "generally i want more of a sandbox, not just lessons. sandbox,
where the user can change components and designs and see how parameters of the engine change …
this will also diminish the number of 'rejected' parts."

Answers to the follow-up: (1) rung 85 is reframed NOW as knobs — blade `h`, tip-Mach level,
material, overspeed factor, `Φ_d`, map shape — every result shown, its ship-or-NEGATIVE gate and
"default shape only" rule retired, all five shapes equal (anchor § 4.4 A1); the sandbox TOOL comes
after rung 85. (2) Form: an **interactive web page**, the Rust model running live in the page
(compiled for the browser — the crate stays dependency-free), beside the charts/cutaway pages.

**Why:** the user wants to explore designs hands-on; pass/fail rungs turned honest misses into
"rejected" work (the eight NEGATIVE docs).

**How to apply:** design new rungs as knobs + readouts. A prediction still gets registered and
scored HIT/MISS, but a miss is a finding, never a reason to withhold the knob or to write a
negative instead of shipping. Don't start the web sandbox until rung 85 is done, unless the user
says so; when it starts, plan it first (which knobs, how the model reaches the browser).
**Rung 85 SHIPPED 2026-10-06** — the web sandbox is now the next work; plan it first.
**Plan written + pushed 2026-10-06: `docs/plans/sandbox-plan.md`** (a Node spike proved the crate
runs as wasm with no deps; ~2 ms/equilibrium run; one last-place drift vs native). Its § 9 holds six
user decisions — ANSWERED the same day (altitude<->T0/p0 linked, equilibrium default, local only,
tolerance OK, browser check in the gate, slices 2+3 both). **Slice 1 SHIPPED 2026-10-06** —
[[sandbox-slice1]]; next: slice 2 (blade speeds) or 3 (off-design), user: either order.
**User, 2026-10-06, on rung 85:** "make it closer to reality, maybe don't round so much" — when a
model is forced to round or quantise (whole blade rows), ask what a real designer lets ABSORB the
leftover and offer that as a knob, rather than letting one quantity jump. See [[rung85-blade-speed]].
CLAUDE.md carries a one-line "Direction" note under its intro. See
[[visuals-artifact]], [[cutaway-artifact]].
**Slice 3 "Fly it" (off-design) SHIPPED 2026-10-07** — [[sandbox-slice3]]; the user chose: opens on
the thermally perfect gas (live), equilibrium on release; stall line as a knob + map shape, labelled
"trend only"; the memo-cache speed-up shipped first. Next: slice 2 (rung 85 blade speeds).
**Slice 2 "Size the blades" SHIPPED 2026-10-07** — [[sandbox-slice2]]; slices 1–3 all built. Next per plan § 5: slice 4 (transient), slice 5 (combustor).
**Slice 4 (transient) PLANNED + ANSWERED 2026-10-07** — plan § 12; user: all four as recommended —
(A) a throttle slam on the Fly-it engine FIRST (temperature-commanded vs fuel-metered, equilibrium gas
refused), then (B) a two-shaft *Controls* view on the Blades rig, perfect gas only, the limiter set +
ONE airflow lever; time in spool time constants. **(A) the slam SHIPPED 2026-10-07** — [[sandbox-slice4a]]; next: (B) Controls.
