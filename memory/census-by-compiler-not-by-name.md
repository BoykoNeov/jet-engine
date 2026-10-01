---
name: census-by-compiler-not-by-name
description: "To find every caller that still reaches a panicking function, rename it on a scratch copy and let the compiler list them — a name or signature grep misses indirect routes like a hook"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 8ead8015-824c-4a4d-9aca-a289c2f0b8ce
  modified: 2026-10-01T12:41:31.885Z
---

When the `sonic_throat` panic was made catchable (2026-10-01, plan § 8.0), the census booked at
slice T classified the 28 call sites by the ENCLOSING function's name/signature (`try_…`). That
misses every INDIRECT route: the turbine-solve hook (`MatcherHooks.solve_turbine`, a fn pointer)
called `choked_mfp` inside its residual and was reached from inside `try_instant_tail` and
`try_plenum_state` — both `Result` functions — so the panic still escaped there. Also missed:
`Nozzle::try_apply` called the panicking spelling from inside a fallible body.

**Why:** a call graph through fn pointers, closures and wrappers is not visible to grep; the compiler
sees all of it.

**How to apply:** after converting, copy `rust/src` + `Cargo.toml` to a scratch dir under
`W:\temp\claude`, rename the panicking wrappers (and drop their imports), `cargo check` with its own
`CARGO_TARGET_DIR`. Every error is a remaining panicking caller; for each, show no `Result` caller /
Python `except` sits above it. Then check the reverse direction too (every Rust `Err` handler has a
Python catch behind it — here all Python catches are `except AssertionError`, so that held). Gate
the end-to-end cell, and prove the gate by reverting ONE converted site (it must fail).
Related: [[rust-port-documented-gate-that-doesnt-exist]], [[instrument-fed-by-what-it-certifies]].
