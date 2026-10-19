# Below-idle floor probes (sandbox plan § 38; archived here — the scratch crates and the worktree they ran in were temporary and are deleted)

Base commit: `6e49d33` (the code before the floor). All but `crash.rs` are `[[bin]]`s of one scratch crate
(`crate_Cargo.toml.txt`) built against the repo; their temp-folder paths (`W:/temp/claude/idle-rule/…`) point at
folders now deleted.

- `offmap.rs` → `offmap.txt` — the crash map's three "off the maps" refusals (requests 158/349/399 of seed 36,
  `../low-start-repair/crash_offmap_remaining.tsv`): which end fails, the Newton from 54 starts over both shafts,
  a residual sign map over `(ν_LP, ν_HP)` (A ++, B +-, C -+, D --, x the closure fails — a root needs all four to
  meet), and the load swept down. Run before the floor and the customer-bleed sentence existed.
- `replies.rs` — the page's current full replies for those three (the archive's TSV carried older words).
- `unique_neg.rs` → `unique_neg_bare.txt`, `unique_neg_loaded.txt` — every below-idle root the FULL-SPEED start
  returns (four maps, 34 flights, 500–1200 K), and 48 starts over both shafts at each: 242 bare + 780 loaded
  (5 % customer bleed, cooled), no second root. The first run panicked on the loaded engines (built without the
  page's builder); the source shown is the fixed one, run with `bare` to skip the bare engine.
- `floors2.rs` → `floors_after.txt` — the Controls page's lowest start per variant (plain, bleed lever, stator
  lever, 5 % customer bleed, cooling), map and flight, with the floor on, and the refusal just below it.
- `fly_neg.rs` → `fly_neg.txt` — the one-shaft Fly view: no running point with negative thrust (438 points).
- `crash.rs` → `crash_summary.txt`, `crash_moved.tsv` — seed 36, 1 200 Controls requests (the HP-wall repair's
  source, unchanged), built against a worktree of `6e49d33` and against the repo; `crash_moved.tsv` is (index,
  request, old reply — a run's cut to its start —, new reply) for the 38 that moved.
- `../rung101/crash101.rs` (unchanged; seed 99, 1 500) built against the worktree and the repo → `crash101_moved.tsv`
  (index, old outcome, new outcome — the script's own classes; the floor's words sort as "refused: Mach" because they
  quote flight speeds); `c101_why.rs` → `crash101_run_to_refused.txt`: each of the 178 runs now refused, re-asked
  for its Mach, the end refused, its thrust per kg/s and its lever.
