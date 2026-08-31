# Test runtime — where it goes, and what is worth doing about it

**STATUS: ANALYSIS ONLY. Nothing in this document has been applied.** No profile key, no runner,
no test file was changed to produce it. It exists so the next person who reaches for "make the
gate faster" starts from a measurement instead of a guess, and so the two things that look like
wins but are not are written down beside the one that is.

Written 2026-08-31, during slice AC step 3, off that step's own two gate runs.

---

## 0. THE BASELINE IS CONTENDED, AND EVERY ABSOLUTE BELOW IS SOFT

Both suites ran **concurrently** on the same box. The Python gate came in at **44:43** against the
**~17:21** `CLAUDE.md` records — about **2.6x contended**. So every absolute second in this
document is inflated by roughly that factor, and none of them should be quoted as a timing.
(`docs`' *never run the gate for timing* rule is why they were taken from a run that was happening
anyway rather than from one launched to measure.)

**The RATIOS survive contention**, because contention scales everything together — the oracle
family's share, the top four binaries' share, the idle-core count during a two-test binary. The
case below is built on the ratios. The first person to re-measure on an idle box should expect the
absolutes to fall by ~2.6x and the ratios to stand.

---

## 1. WHERE THE RUST TIME GOES

`cargo test --release`: **3m49s to compile**, then **130 targets run ONE AT A TIME** (128
integration binaries, the lib's unit tests, and a doctest phase that holds none). Cargo's scheduler
is per-binary; tests parallelise only *within* a binary. Sum of the per-binary `finished in`
figures: **3277.8 s**, so the whole gate is about **58 min** on this box.

**That 3277.8 s is WALL-SUMMED, NOT CPU.** A 42 s binary holding 26 tests was already using up to
16 cores for those 42 s. So "55 minutes of CPU onto 16 cores" is not a floor, and the long tail is
not the problem — the long tail is fine.

The problem is the binaries with too few tests to fill the box. Every target with **five or fewer
tests and thirty seconds or more of wall**:

| binary | tests | wall | cores idle |
|---|---|---|---|
| `tests/slice_l_oracle.rs` | 2 | 265.5 s | ~14 |
| `tests/slice_m_oracle.rs` | 3 | 215.1 s | ~13 |
| `tests/fuel_transient_oracle.rs` | 5 | 160.1 s | ~11 |
| `tests/slice_o_oracle.rs` | 2 | 155.1 s | ~14 |
| `tests/two_spool_oracle.rs` | 2 | 73.0 s | ~14 |
| `tests/topping_oracle.rs` | 4 | 67.9 s | ~12 |
| `tests/slice_w_oracle.rs` | 2 | 58.3 s | ~14 |
| `tests/offdesign_oracle.rs` | 2 | 42.3 s | ~14 |
| `tests/slice_s_smoke.rs` | 1 | 40.7 s | ~15 |
| `tests/slice_z_oracle.rs` | 2 | 32.8 s | ~14 |
| `tests/slice_n_oracle.rs` | 5 | 31.8 s | ~11 |

**Eleven binaries, thirty tests, 1142.6 s — 35 % of the run — with eleven to fifteen of sixteen
cores parked.** The 30 oracle binaries together are **1392.4 s, 42 %** of the run. The whole gate
is 1268 tests over 130 targets.

## 2. THE ONE LEVER THAT MATTERS: SCHEDULE BY TEST, NOT BY BINARY

`cargo-nextest` runs every test as its own process under a single scheduler across all cores,
instead of cargo's binary-at-a-time. Nothing about *what* is asserted changes.

**The floor is one test, and it is higher than the core count suggests.** `slice_l_oracle`'s two
tests both reported ">60 s" and finished together at 265 s, so **each** is ~265 s — they are not
two halves of it. No scheduler can beat its longest single test. Realistic target:

> **~4.5 min of tests + 3m49s of compile = 8–10 min, against ~58 min today. About 6x.**

### Compatibility — two hazards cleared, one to pin

* **No doctests.** The `Doc-tests turbojet` target exists but reports `running 0 tests` — so
  nextest, which does not run doctests, loses nothing. Verified off the run, not assumed; the
  first look at this counted the *phase* rather than the tests in it and briefly read as "no such
  target", which is the weaker claim and the wrong one.
* **Every census counter is `thread_local!`** (`bleed_transient`, `combustor`, `cross_split`,
  `fuel_transient`, `full_split`, `lagged_bleed`, `limited_bleed`, `map`, …). Per-process
  isolation is therefore *strictly stronger* than what the tests already rely on — nextest cannot
  break a counter, and would close the interference window one test file's own comment already
  warns about (*"two tests building sections concurrently would steal each other's tallies"*).
* **PIN THE RUN COUNT.** nextest reads `.config/nextest.toml` and carries its own
  filter/partition machinery — a live route to exactly the **silent deselection** the project's
  ONE-gate rule forbids. There is no such file today (checked). Any adoption must assert nextest's
  printed `Summary [ x/y tests run ]` **y** against the `cargo test` total, so a stray config
  cannot quietly shrink the gate. Without that assertion this is a downgrade, not a win.

---

## 3. WHAT LOOKS LIKE A WIN AND IS NOT

**Sharing the oracle sweep between its two arms — DO NOT.** Each oracle binary calls
`rust_values()` (the whole Rust recomputation) **once per test**, and there are two tests: the PyPy
bit-equality arm and the CPython measured-bar arm. On a two-arm binary that is HALF its CPU spent
twice, and the oracle family is 1392 s of the run. Two routes, both refused, and the arithmetic is
the reason:

* **A `OnceLock` memo** (the idiom `rung48.rs` / `rung49.rs` / `rung50.rs` already use, mirroring
  Python's `_SWEEPS`) would halve that binary's CPU — and save **zero wall-clock**, because the two
  tests already run concurrently and the binary's wall time is one sweep either way. Under nextest
  it is inert, since the arms become separate processes.
* **Merging the two arms into one test** would save the CPU under nextest — a few hundred core-seconds
  spread over 16 cores, **well under a minute of wall-clock** — at the price of the two-bar split. PyPy at
  bit-equality is the *detector*; CPython at the wider measured bar is the sanity check; and
  `slice_l_oracle.rs`'s own header is built around **which arm moved** (its calibration found one
  key over the bar, and it was an `n_pass`, not a value). Trading that for under a minute is a bad
  trade.

---

## 4. COMPILE TIME BECOMES ITEM 2 THE MOMENT ITEM 1 LANDS

3m49s against ~4.5 min of tests. `[profile.release]` sets `codegen-units = 1` (serialises lib
codegen) and `lto = "thin"` (runs at link time, for each of **128** test binaries, every one of
which links the whole lib). There is **no `.cargo/config.toml`** (checked), so the linker is
whatever the toolchain defaults to.

**BOTH ARE BIT-SAFE TO CHANGE, AND THIS HAS TO BE SAID OUT LOUD OR THE BIT-EXACTNESS BAR WILL
SCARE SOMEONE OFF A FREE WIN.** Rust has no fast-math and does not reassociate floating point;
`opt-level`, `lto` and `codegen-units` do not alter IEEE results on x86-64. The port's
digit-for-digit agreement with PyPy does not depend on any of the three. What it *does* depend on
is spelling (`powp` vs multiply-the-square, `py_max3`, the format-and-parse `roundN`), which is
source, not codegen.

To measure, on an idle box, one variable at a time: `codegen-units = 16`, then `lto = false`, then
a linker swap; keep whichever pays, and **re-run the oracle gates** after each — not because the
theory is doubtful but because that is the cheap way to find out the theory was mis-stated.

---

## 5. A FINDING IN PASSING: `[profile.test]` MAY BE DEAD UNDER THE COMMAND THE PROJECT GATES ON

`Cargo.toml` carries:

```toml
[profile.test]
opt-level = 2
```

with a comment arguing that `assert!` must survive optimisation because the conservation checks
*are* the model. Two problems, neither yet verified, both cheap to check:

1. **Cargo selects `[profile.bench]`, not `[profile.test]`, for `cargo test --release`** — `test`
   inherits from `dev` and is consulted by a bare `cargo test`; `bench` inherits from `release`.
   The gate runs `--release`. If so, the block never fires under the command that matters, and the
   test targets have been built at the release profile's `opt-level = 3` all along.
2. **`assert!` is never stripped at any optimisation level** — only `debug_assert!` is, and only by
   `-C debug-assertions=off`. So `opt-level = 2` is not a defence of the conservation checks; the
   comment guards a property no `opt-level` can turn off.

**Verify before writing either down as fact**: `cargo test --release -v` and read the
`-C opt-level=` on a test target's rustc line. If it says 3, the block is dead config with a
justification that does not justify it — and it should be deleted *with the finding*, on slice AC
step 1's precedent, not silently.

---

## 6. THE PYTHON SIDE

The gate is `pytest` under xdist `-n auto` on PyPy, 1364 tests. The known hazard is already
recorded (`docs`' *xdist module-fixture cost*): **a module-scoped fixture is rebuilt once per
worker.**

Two measurements, and **neither may be taken by re-running the gate** — the rule is that a quoted
gate time is documentation, not a signal. Both belong on the **iterate tier**
(`pytest -m "not slow"`, 977 tests), which is not the gate:

1. `--durations=25` on that tier, on an idle box, to find out whether the cost is in a few marches
   or spread flat. Everything after this depends on the answer.
2. `--dist loadscope` against the current default, same tier, same box. loadscope pins a module to
   one worker, so each module's fixture is built **once** instead of once per worker — but it also
   costs packing efficiency, because one slow test now blocks its whole module's worker. **It can
   go either way; it needs the measurement, not a prediction.**

---

## 7. ORDER OF WORK, IF THIS IS EVER PICKED UP

1. Re-measure the Rust baseline on an **idle** box (§ 0 says today's absolutes are ~2.6x inflated).
2. `cargo-nextest`, **with the run-count assertion** of § 2. This is the whole win; everything
   after it is a rounding error by comparison.
3. Only then compile time (§ 4), one variable at a time.
4. § 5's profile check — a five-minute verification that either clears the block or produces a
   finding; it is independent of the rest.
5. The Python tier measurements of § 6, on the iterate tier, never on the gate.

**Do none of it inside a slice step.** A profile or runner change invalidates the very run a step
is waiting on, which is how a green gate stops meaning anything.
