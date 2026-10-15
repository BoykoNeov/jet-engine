# Rung 92 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\jet-rung92-probe`, deleted after the rung).

- `givens.rs` → `givens.txt` — anchor § 0: the unloaded line's compressor slopes and shaft powers, rung 42's and rung 90's MEASURED responses (Richardson), the first-order offtake response rebuilt from them, and anchor § 1's linear model (existing code only, run before any offtake code existed).
- `score.rs` — every pre-registered quantity: part A (steady, flat + `flow/press`, Richardson-extrapolated from 0.01 and 0.02, share and power modes on each shaft, plus the reported thrust/TSFC and floating shares) → `score_a.txt`; part B (rung 64's valve rig, 6 cells × 3 demands — none, HP power 0.02, LP power 0.02 — with the valve-shut march's HP 90 % rise time) → `score_b.txt`.
- `peek.rs` — which power loads converge, and which are refused at the root ("half or more") or at the fold ("no steady point"); with `650`, why the 650 K power-mode reading fails (the nozzle unchokes).
- `crash.rs` — the sandbox crash map (1 000 random Controls requests, each carrying a generator load), with every endpoint refusal re-run with the generator OFF to tell a load-caused refusal from an ordinary one → `crash.txt` (counts in `docs/plans/sandbox-plan.md` § 21).
