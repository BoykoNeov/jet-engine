---
name: run-tests-below-normal
description: "Always launch test runs (pytest, cargo test) at BELOW-NORMAL process priority"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 0c22f3fc-4861-40a2-b65c-12746bba5bf2
  modified: 2026-08-28T15:17:39.444Z
---

Every test run — the `pytest` gate, `cargo test`, any long build or suite — must be launched at
**below-normal** process priority, not at the default.

**Why:** stated by the user on 2026-08-28, mid-run, while the full Rust gate was saturating the
box. These suites run for many minutes across every core; at normal priority they make the machine
unresponsive for whatever the user is doing at the same time. Below-normal costs the run almost
nothing (it still gets the idle cores) and gives the user their desktop back.

**How to apply:** launch it low, do not start it normal and fix it afterwards.

* PowerShell: `$p = Start-Process cargo -ArgumentList 'test' -PassThru -NoNewWindow; $p.PriorityClass = 'BelowNormal'; $p.WaitForExit()` — set the class on the PARENT before it spawns children; cargo's test binaries and pytest's `-n auto` workers inherit it.
* Git Bash: `cmd //c start //belownormal //b //wait cargo test` (doubled slashes are required — the shell rewrites single ones as paths).
* If a run is already going at normal priority, raise nothing and kill nothing: set `PriorityClass` on the PIDs whose executable path is inside this repo's `target\` (or the venv), per [[windows-tooling-file-hazards]]'s rule that a process is only yours to touch if you can identify it by path or by a PID you captured.

Also capture the run's full output (`| tee`), not just `| tail` — see [[never-run-the-gate-for-timing]] for what the gate's numbers are and are not for.
