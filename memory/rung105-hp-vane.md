---
name: rung105-hp-vane
description: "Rung 105 (2026-10-20), the HP turbine vane's reaction law beside the A4 pin, a knob OFF by default — process lessons: re-ask scope BEFORE freezing once a prototype measures cost and effect; check a knob's definition against the one its decision was made on; label a probe's reference explicitly; restructure a nested solve so the expensive law is read once"
metadata:
  node_type: memory
  type: project
  originSessionId: dda0c953-01cb-4b73-a8e4-c586c6990234
  modified: 2026-10-20T20:50:45.877Z
---

Rung 105 shipped the HP vane's reaction law (two rows in series, textbook reaction, default 0.3 when on) as a setting
that defaults to the old choked pin — spec `docs/rung105-spec.md`, anchor `docs/plans/rung105-anchor-hp-vane.md`
(`865c338`). Headline: the premise (choked) was wrong at any usual reaction, the consequences nearly right (≤0.2 %
choked). Decisions: [[rung105-decisions]]. Follows [[rung104-turbine-law]].

**Lessons (process):**
1. **Prototype, measure cost AND effect, then re-ask the scope — before the anchor freezes.** The user's first answer
   ("on everywhere") was given blind; the prototype showed Fly it 50–110× slower for a ≤0.4 % effect, and asked again
   the user chose a default-OFF knob (no oracle retirement, no test sort). Rung 104 re-asked only after building.
2. **A knob's definition must match the one the decision was made on.** "Reaction" was coded on the isentropic drop;
   the published range and the choke formula use the actual drop — the knob's 0.3 was a textbook ~0.24 and chose a
   choke at 0.26 instead of 0.19. Caught (advisor) by bisecting the capture's choke flip against the closed form;
   now gate R7.
3. **Label a probe's reference explicitly.** P1 called the first law that SOLVED "pin"; where the pin refused, a
   reaction-vs-reaction difference was reported to the user as "vs the pin" (+2–4 % instead of +0.07 %). Corrected in
   the next message; probes now tag `(REF)`.
4. **A nested solve's cost is fixed by moving the expensive law out of the inner search.** Under the law the closure
   solved "vane = nozzle" per compressor trial (~10 vane flows, each a nested interstage search). Given the trial's
   flow, the nozzle (or LP side) alone fixes the turbine; read the vane ONCE — 10× fewer law calls, same root. Warm
   starts help only if scoped per instant (cleared at entry) so a point never depends on what ran before.
5. **A hand-built probe input can be a named catalogue entry.** The "mixed" map pair was shape `flow/press`, so the
   anchor's "never probed" list was wrong (disclosed in an addendum). Check probe inputs against the named shapes.
6. **An old solver bug found by the new law's probe:** rung 33's subsonic residual has two roots at hot Mach 0.3 (the
   pin too) and its two-ended march refuses "SUB-IDLE" — recorded as the next seam, not fixed in this rung (user).
