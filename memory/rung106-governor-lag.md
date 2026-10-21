---
name: rung106-governor-lag
description: "Rung 106 (2026-10-21), the governor beside rung 47's slow temperature limiter — process lessons: a crash map's slow tail was rung 95's route regressing 20–150× since it shipped (time on the shipping commit); a claim about a comparison case must be re-run on the SAME configurations (my first 'the open ramp never gets there' rested on 6 runs); a side note can catch an unrealistic setup a result rests on — ask before predicting; an all-HIT sweep earns a look past the grid (H4 flipped at lag ~0.9); `cmd start /wait` on a missing exe hangs forever — check the build before launching"
metadata:
  node_type: memory
  type: project
  originSessionId: a8f5c403-d6a4-452b-be07-1955f109c760
  modified: 2026-10-21T17:19:26.281Z
---

Rung 106 shipped the governor beside rung 47's lagged redline clip (spec `docs/rung106-spec.md`, anchor
`docs/plans/rung106-anchor-governor-lag.md` committed `ca5504f` before code). Decisions: [[rung106-decisions]]. Follows
[[rung95-governor]]. Headline: a stateless governor beside a lagged limiter WINDS UP — overshoot ~2×, a larger stored
cut paid back after the move, the thrust climbs then SAGS (only in this combination).

**Lessons (process):**
1. **Re-run a comparison claim on the SAME configurations before telling the user.** I said "the open ramp never gets
   near the lean wall" from 6 runs on one map; the 399 lean configurations spanned four maps, three ramps, three
   redlines. The advisor caught it; the re-run (all 2 520) held — but 679 open runs had STOPPED early (the prototype's
   missing solvable-ceiling fallback), so I also had to check none of the 399 were among them.
2. **A side note named the setup a result rested on.** All 399 lean hits started ABOVE the redline — a state the
   limiter would never allow. Asked; the user refused that start. Before predicting a mechanism, ask whether its
   reachable cases are physical. **The crash map then found the same case in disguise** (#896): the metal cap held the
   start low while the governor's REQUEST was above the redline — check the quantity the state is measured off (the
   demand), not only where the engine sits; asked, refused too.
3. **All HIT ⇒ look past the grid at the named risk.** H4 ("later to 95 % thrust") was 20/20 on τ ≤ 0.8 but the
   reading jumps at τ ≈ 0.9 (the first climb clears 95 %); looking for it found the real finding — the thrust SAGS
   after the move, which no prediction named. Report the flip with the grid.
4. **The step check moved the knob floor twice** (one step → 0.06 → 0.1): two fast clocks (lag and the speed law's
   τ_N) compound; check the floor at every hold's own fastest knob, not the default.
5. **A crash map's slow tail can be someone else's regression — time it on the shipping commit.** The 15–20 s
   requests were the same with the lag off; on rung 95's commit (a worktree, back to back) they took 0.2–1.8 s — 20–150×
   slower today, untraced (sandbox plan § 43, OPEN). A side note pushed for the check; it took ten minutes.
6. **`cmd start /b /wait` on an exe that failed to build waits FOREVER** (documented in
   [[windows-tooling-file-hazards]] for a mistyped path; here a compile error). Gate the run on the exe existing
   (`test -f … &&`). And AGAIN the `\$e` trap: a loop variable after a backslash inside a cmd string stays literal —
   write each path out. Also: Python's default cp1252 rewrote a UTF-8 source (em-dashes) — pass `encoding='utf-8'`.
