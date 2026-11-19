---
name: rung109-turbofan
description: "Rung 109 (2026-11-19), the turbofan design point — process lessons: a new module in lib.rs makes the committed sandbox page stale (wasm changed) even with no page code, so rebuild it in the MODEL commit; an optimiser over a domain with a feasibility wall must find the wall by bisection, not refine only between runnable grid points; check a physical wall BEFORE asking the gas tables to invert; attribute a checker's misses by running it on a `git archive HEAD` copy"
metadata:
  node_type: memory
  type: project
  originSessionId: 86c1c634-fca3-4bea-9c24-fe8da115d6c2
  modified: 2026-11-19T10:13:59.072Z
---

Rung 109 shipped the separate-exhaust turbofan on the design point (spec `docs/rung109-spec.md`, anchor
`docs/plans/rung109-anchor-turbofan.md` at `19a2b6e`, model `b7a80e0`, page `f1712e5`; the charts artifact republished). Decisions:
[[rung109-decisions]]. Follows [[rung108-two-speed-limiter]]. Scored 256 HIT of 276; misses were findings.

**Lessons (process):**
1. **Adding a module to `rust/src/lib.rs` changes the browser build, so `the_committed_page_is_current` fails** even
   when no page code changed (the 245-program gate was all green but that one). **How to apply:** in any commit that
   touches `rust/src/`, run `rust/sandbox-wasm/build.ps1` and commit `docs/sandbox/turbojet-sandbox.html` with it; then
   `test-all.ps1 -Only sandbox-browser` re-checks just that program (~3 min).
2. **An optimiser over a domain with a feasibility wall must locate the wall.** `best_pi_f` first refined only between
   RUNNABLE grid neighbours; where the runnable window was narrower than the grid step it returned a grid point far
   from the wall-limited best. Fixed with edge bisection, found only because scoring hit it. **How to apply:** when
   the best can sit against a "does not run" edge, bisect that edge and report "at the wall" in the result.
3. **Check the physical wall before calling a table inversion.** The first scoring run panicked at `gas.rs:511`
   (an enthalpy below the tables) — the turbine asked to leave the gas colder than outside. A closed-form wall test
   (`Tt5s ≤ T0`) ahead of the solve turned the panic into a refusal in words.
4. **Attribute a checker's misses before acting on them:** `docs/notes/check-numbers.ps1` reported 8 misses (rungs
   46–48, 90, 92) — all present at HEAD too (`git archive HEAD docs | tar -x` into temp, run with `-Repo`). Older drift,
   reported, not fixed in the rung.
5. **Measure the page cost natively before designing around it:** the chart's twelve best-fan searches looked
   expensive; measured 6.6–10 ms a run (best search 0.4–0.8 ms vs 1 ms design run), so it runs live, no button.
6. (Repeat hazard) a `cmd start /belownormal` path with a space breaks in the Bash tool — wrap the command in a
   `.cmd` file under `W:\temp\claude\...`; see [[windows-tooling-file-hazards]].
7. **A knob whose meaning a new effect changes needs words on the page.** The airflow knob stayed the CORE's, so a
   bypass stream silently grew the engine (bypass 8: 20 → 180 kg/s in all); a side note caught that a pinned turbojet
   beside a turbofan looked like free thrust. Fixed with a derived line under the knob ("through the core; with the
   bypass air, N kg/s in all") and a browser check; a total-air knob was offered to the user as a choice, not imposed.
   **How to apply:** when a rung adds a stream or a stage, list every existing knob whose quantity it redefines.
