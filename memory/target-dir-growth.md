---
name: target-dir-growth
description: rust\target grew to 13 GB because every build CONFIGURATION keeps its own ~1 GB set of test programs forever; test-all.ps1 now prunes/throws away (2026-11-22)
metadata:
  node_type: memory
  type: project
  originSessionId: e6db3069-4993-4b31-94eb-7fc3017b4fd2
  modified: 2026-11-21T23:06:53.535Z
---

2026-11-22: `rust\target` was 13 GB — `debug\` 8.2 GB (unused since 10-15; the workflow is all `--release`)
and `release\deps` 911 test programs where ~245 were live. Cargo keys a test program's hash on the build
CONFIGURATION (features, toolchain, profile), not on source, so each of default / `pin-default` /
`hp-law-default` / an older toolchain left a full ~1 GB set that nothing ever deleted.

Fix shipped in `rust\test-all.ps1`: a `-Features` run builds in `target\feat-<name>` and deletes it at the
end (sandbox-browser check gets no `CARGO_TARGET_DIR`); a default run deletes older-hash copies of every
program it just built (stem `turbojet` skipped — shared with the CLI binary). Both verified by a planted
stale file and a `-Features pin-default -Only rung104` run.

**Why:** the user asked for the space back and for the project to stay lean.
**How to apply:** if `target\` balloons again, look for a new configuration axis first (a new feature,
a `cargo test` without `--release` filling `debug\`), not for source churn. `.pdb` files are ~2/3 of
`release\deps`; turning them off was NOT done (config change ⇒ full rebuild; backtraces lose names).
Related: [[always-commit-and-push]], [[run-tests-below-normal]].
