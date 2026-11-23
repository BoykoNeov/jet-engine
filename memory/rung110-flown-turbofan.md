---
name: rung110-flown-turbofan
description: "Rung 110 (2026-11-20..24), the turbofan flown — process lessons: a silenced panic hook is PROCESS-WIDE (it hid another test's failure); a long probe must print each result as it lands and use the cores (27 min blind on one core); a heredoc-fed python step can hang an hour waiting on stdin; a pre-registered prediction that scores 0 cells is VACUOUS, asserted as such, never re-thresholded"
metadata:
  type: project
---

Rung 110 shipped the fixed turbofan flown off design on the two-shaft engine (spec `docs/rung110-spec.md`, anchor
`docs/plans/rung110-anchor-flown-turbofan.md`; model commits d316954 → 03065bd; docs/row/notes/splice after; charts
artifact republished, version 4). Score 480 HIT of 632, P8 vacuous. Decisions: [[rung110-decisions]].

**Lessons:**
- **A panic hook is process-wide.** A test helper (`panic_words`) and the grid silenced it around `catch_unwind`; tests
  run in parallel threads, so gate 7's real failure printed NO message. Never touch the hook in a test file.
- **A long probe prints as it goes and uses the cores.** The first refusal census ran 27 min on one core with no output;
  rewritten with `std::thread::scope` (14 threads) and one `println!` per line it finished in 16 min with live counts.
- **A bash heredoc into `python -` with a broken heredoc sat an hour waiting on stdin** before the real step; write
  edits with a checked script (assert each old string occurs once), and check a background job's processes early.
- **A refusal you plan to gate may be unreachable.** FAN_NOZZLE_DRY never fired across 420 throttle lines; the gate was
  dropped and the census recorded, not a point invented.
- **Vacuous prediction:** P8 scored 0 cells (the turbojet's effect 2.7e-5 < the anchor's 1e-4); the test asserts the
  vacuity so a later change that gives it cells fails loudly. The threshold was NOT moved after seeing data.
- **Launching `powershell` (5.1) from bash/cmd under pwsh 7** loses `Get-FileHash` (PSModulePath): the sandbox check
  "fails" without running. Run `rust\sandbox-wasm\check.ps1` from pwsh to judge the page.
- **Open, asked 2026-11-23, unanswered:** at the edge of running the turbofan says "did not settle" instead of the
  physical reason (169 of 420 lines). Recorded as OPEN in the spec; rung 111 must settle it. Ask the user again.
- **Kept on purpose:** `W:\temp\claude\rung110-flown-turbofan\probe\` (the anchor's § 0 probe scripts, cited by the
  committed anchor) and `book\` (gate 5's source text read from the user's scan). Not orphans; ask before deleting.
- **Read the cited rung's spec before writing "corrects".** Gate 7's equilibrium drift was first written as correcting
  rung 38; rung 38's spec already named it (the shafts talk through `f` on the equilibrium gas) — it CONFIRMS.
