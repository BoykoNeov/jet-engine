# Retiring Python — the sequencing, and the one thing that has a deadline

**STATUS: ANALYSIS ONLY. Nothing here has been applied.** No file was deleted, no test removed, no
`Cargo.toml` key changed to produce it. Written 2026-08-31, immediately after slice AC step 3.

This document does **not** invent a retirement path. `docs/plans/todo-rust-port.md` already carries
one as **phase 8** ("`main.py` replacement; adjudicate the fragile rungs; re-anchor the fingerprint;
**delete the Python**", exit criterion *full suite green on Rust alone*). What this document does is
**revise phase 8 under three decisions taken today**, and **correct a framing given verbally the day
before, which had the dependency backwards.**

---

## 0. THE THREE DECISIONS, DATED

Taken 2026-08-31, in the user's own terms:

1. **No second Python interpreter.** *"If we need it, then our design is flawed, nothing else."*
2. **Retiring Python is a priority, in reasonable terms.** The aim is Python out of **the main
   program and the tests**. Two prongs: faster execution, faster testing.
3. **Python may survive in the lesson-writing layer**, *"if the source is done by Rust."*

Decision 3 is not a new position — it is **exactly the port plan's own § 9 decision 2**, taken
2026-08-12: *"the ENGINE is pure Rust… one small Python script owns the matplotlib chart only — it
does no physics, reads no engine code, and is the single permitted exception."* The two statements
agree line for line. Nothing about the display layer needs re-deciding; it needs building.

Decision 1 **overrules § 3 of `docs/plans/todo-test-runtime.md`**, which refused to collapse the two
oracle arms. Recorded as overruled, with the date and the reason, rather than defended. § 4 below
traces what it actually buys, because it is less than the framing "two suites become one" suggests.

---

## 1. THE CORRECTION — THE DEPENDENCY WAS BACKWARDS, AND IT INVERTS THE ORDER OF WORK

On 2026-08-30 the answer given to *"can we retire the parts already rewritten in Rust"* named the
blocking item as *"a Rust program that writes the reference tables"* — reasoning that 37 of the 38
`rust/oracle/dump_*.py` scripts import `turbojet`, so deleting the Python would make the committed
tables unregenerable.

**That is wrong, and the error is not a detail — it reverses the sequence.**

The oracle's job is to prove **Rust ≡ Python**. Once Python is gone, that job does not need a new
implementation; it **ceases to exist**. A table regenerated *by Rust* proves nothing about the Rust
that generated it — it is a regression snapshot, a strictly weaker instrument, and a perfectly fine
one. So the Rust dump binary is **not a prerequisite for deletion**. It is a cheap consequence of it,
and it shares a body with the display shell (§ 7).

What actually carries a deadline is the opposite thing:

> **Python is the only witness that Rust ≡ Python, and that witness can only be collected while
> both still exist.** Every rung whose oracle is not built before the Python is deleted can never
> have one built. There is no catching up afterwards.

**Consequence for the schedule.** "Finish rungs 70–84" was item 4 of 4 in what was said. It is item
1. It is the *only* item with an ordering constraint at all; everything else in phase 8 can be done
before, during, or after, in any order.

---

## 2. WHERE THE WITNESS STANDS TODAY — measured, not estimated

| | Python | Rust |
|---|---|---|
| model source | `turbojet/` **29 038 lines** | `rust/src/` **42 550 lines**, 27 files |
| display / entry | `main.py` **7 029 lines** + `docs/visuals/extract_data.py` | **none** — `Cargo.toml` declares a library, no `[[bin]]`, no `rust/src/bin/` |
| tests | `tests/` **31 962 lines**, 80 `test_rung*.py`, gate = 1 364 tests | `rust/tests/` **128 files**, gate = 1 268 tests over 130 targets |
| rungs covered | 1–84 | **1–69**; 70/71 in flight as slice AC; **72–84 unported** |
| reference tables | — | `rust/oracle/`, **72 TSVs, 31 MB** (43 PyPy / 17 MB, 29 CPython / 14 MB) |

**Rungs still owing a Python-generated oracle: 70–84, i.e. slices AC (in flight) and AD–AJ.**
From `docs/plans/todo-rust-port.md`'s own phase-7 slice table:

| slice | rungs | new cells |
|---|---|---|
| **AC** | 70–71 | 0 — *in flight, steps 3 of 7 done* |
| **AD** | 72 | 3 |
| **AE** | 73 | 0 |
| **AF** | 74 | 3 |
| **AG** | 75–76 | 0 |
| **AH** | 77–78 | 0 (+ `_legs`, the slice-AC sweep's find) |
| **AI** | 79–80 | 0 |
| **AJ** | 81–84 | 0 — the reader-only rungs |

**Seven slices after AC.** The plan sized the whole of phase 7 at 15–20 sessions; V…AC is eight
slices spent, so **the remaining half of phase 7 is the whole of the deletion schedule**, and
nothing after AD adds more than three cells. That is the honest answer to *"in reasonable terms"*:
**Python's deletion date is seven slices away, and no measure taken elsewhere moves it.**

---

## 3. THE HARVEST — name it, because it is currently a byproduct

Each slice already ends by committing a PyPy-generated TSV that the Rust reproduces bit for bit.
Today that reads as a per-slice artifact. **Under retirement it is the migration's load-bearing
output**, and it should be treated as such from slice AD onward:

* the TSV is the *frozen testimony of the retired implementation*, not a cache;
* it stays in git after the Python goes, and the oracle test keeps running against it unchanged —
  it simply stops being a cross-language check and becomes a regression anchor;
* **regenerating one after Python is deleted is a deliberate, reviewed act**, because at that point
  regeneration destroys the evidence it replaces. Worth a one-line note in each oracle's header
  saying so, added as each slice closes rather than retrofitted across 30 files later.

This also settles the fingerprint question in § 5.

---

## 4. WHAT DROPPING THE SECOND INTERPRETER BUYS — and what it does not

**Deletable, immediately and safely:** the CPython arm of 33 `rust/tests/*.rs` files, and 29
`*_cpython.tsv` — **14 MB of the 31 MB**. Plus the exemption bookkeeping that goes with them: slice
AB alone curates **194 named CPython exemptions**, slice AA four, slice Z eight. That curation is
real recurring work at every slice, and it stops.

**What it does NOT buy is wall clock, and this must be said plainly or a number will be expected
that never arrives.** `docs/plans/todo-test-runtime.md` § 3 measured it: the two arms of an oracle
binary **already run concurrently**, so the binary's wall time is one sweep either way. Deleting an
arm halves that binary's CPU and saves ~nothing on the clock. The runtime win in this project lives
somewhere else entirely — it is `cargo-nextest`, § 2 of that document, worth about 6×.

**So decision 1 is a maintenance and clarity decision, not a speed one.** It is still right on those
grounds. It just should not be booked against the "faster testing" prong; that prong is paid by
nextest, and by Python's deletion removing a 1 364-test suite outright.

**One thing is lost with the CPython arm, and it should be lost knowingly.** Three findings in this
project came *only* from having two interpreters: CPython 3.12+'s `sum()` is Neumaier-compensated
where PyPy's is naive (slice W); `_illinois` taking 8 iterations vs 7 from bit-identical inputs
(slice AA); `_invariants`' `c1` drifting on 23 of 256 instances where `c2`, built identically at the
same site from the same three numbers, drifts on none (slice AB). Each falsified something a comment
or a pre-registration asserted. After retirement no such finding is possible — which is consistent,
because after retirement there is no second implementation for them to be findings *about*.

---

## 5. WHAT DIES QUIETLY WITH `tests/` — one line each, so none becomes a surprise

* **`tests/test_numeric_fingerprint.py`** — the project's **only absolute-value gate**, and its
  entire doctrine is *"the goldens are CPython's and they stay CPython's… the tolerances are
  LOAD-BEARING PERMANENTLY."* It is a Python test, so it dies with the suite. **Its function is
  absorbed by the frozen oracle TSVs of § 3** — those are absolute bit patterns, far more of them,
  and they need no tolerance at all. **This only holds if the harvest happens first**, which is
  another way of stating § 1's deadline.
* **`tests/test_claude_md_reference.py`** — the size guard on `CLAUDE.md`. Retiring `tests/` removes
  it silently. A recorded lesson in this project is titled *a documented gate that doesn't exist*;
  this is the next candidate. Either port it (it is a file-size check — trivial as a Rust test) or
  delete the guard's paragraph from `CLAUDE.md` in the same commit.
* **`conftest.py`** — the `slow`-marker policy, the below-normal-priority hook, and the written
  rationale for why three-gate tiering was retired. All of it documents a suite that will not exist.
  The *policy* it encodes (ONE gate, nothing silently deselected) must move to the Rust side, and
  `todo-test-runtime.md` § 2 already names the exact hazard there: nextest's filter and partition
  machinery is a live route to silent deselection, so **the run count must be pinned**.
* **`requirements.txt`, `.venv`, the PyPy doctrine** (`docs/plans/todo-pypy-switch.md`, `CLAUDE.md`
  § Stack, `psutil` as a hard requirement) — all become historical. They describe how to run
  something that is gone.
* **`docs/rungN-spec.md` × 84** — these name Python classes and methods throughout. **Not a
  blocker and not worth rewriting**: they are the teaching deliverable, they describe physics, and
  the Rust type of the same name is one grep away. A single disclosure paragraph, once, is the right
  cost.
* **`CLAUDE.md` § Layout and § Commands** — both describe the Python tree as the project. They are
  the last thing to change, and they change in the same commit as the deletion.

---

## 6. THE ONE OPEN BLOCKER, ALREADY MEASURED

The phase-8 row carries it, and it is still open at HEAD (`rust/src/components.rs:689`):

> **`components::sonic_throat`'s bracket `assert!` is a `panic!` where Python's is a catchable
> `AssertionError`** — and every marcher's `except AssertionError: break` in the ladder relies on
> catching it. Measured at slice T step 1: **28 call sites, ≥10 already in fallible chains.**

The repair is prescribed at the definition site (`try_sonic_throat` / `try_choked_mfp` returning the
`Result<_, Abort>` chain that already exists for exactly this), and it is disclosed by a live gate,
`rung46.rs::disclosed_divergence_a_python_catchable_assert_panics_in_rust`. It is a divergence
between the two implementations, so — like the oracles — **it is cheapest to settle while both still
run**. It is not large; it is just ordered.

---

## 7. THE DISPLAY SHELL — one binary, two jobs, cost it once

`rust/src/bin/` does not exist. Two separate needs both land there:

1. the **post-Python golden regenerator** of § 1 (weak instrument, but needed the first time a Rust
   body legitimately changes after deletion); and
2. the **station-table / plot-data emitter** that phase 8 decision 2 already specifies — Rust prints
   the tables the working contract requires every run to print, and dumps the plot's data as JSON for
   a small Python chart script that does no physics.

**These are the same binary**: both walk the model and serialise numbers. Building it once serves
both, and it is what lets `main.py`'s 7 029 lines and `docs/visuals/extract_data.py` become thin
renderers instead of a second model. It has no ordering constraint against anything else here.

**Assumption stated so it can be rejected in one line:** decision 3 means *Python renders, Rust
computes*. If instead Python is to go including the chart, this item grows by a Rust plotting stack —
and the crate's zero-dependency rule would have to be revisited, which is a real cost. Say so and it
gets re-costed.

---

## 8. THE ONE QUESTION THIS DOCUMENT DOES NOT DECIDE

After deletion there is **one implementation and one set of frozen tables**. That is internally
consistent and it is what decision 1 asks for. But it trades away a capability rather than merely
saving work, so it belongs on the record as a choice and not as an omission:

> Are the frozen oracle tables accepted as the **permanent** absolute anchor — a Rust checked
> against its own past — or is a second independent check of *something* wanted (the published
> reference cases the rungs are anchored to, say, re-asserted as absolutes)?

Decision 1 reads as answering this too, and the literature anchors already exist in
`docs/plans/rungN-anchor-*.md`. Naming it here means nobody later discovers it by accident.

---

## 9. ORDER OF WORK

1. **Slices AC (finish) → AD → AE → AF → AG → AH → AI → AJ.** Rungs 70–84 ported, each with its
   Python-generated oracle. **This is the schedule; nothing else moves the date.**
2. **From AD onward, harvest deliberately** (§ 3): one line in each oracle header saying the table is
   frozen testimony, not a cache.
3. **Settle `sonic_throat`** (§ 6) in whichever slice's path it sits on — while both languages run.
4. **Build `rust/src/bin/`** (§ 7) — the emitter. Any time; no ordering constraint.
5. **Port the display**: `main.py` and `docs/visuals/extract_data.py` become renderers over that
   binary's JSON.
6. **Drop the CPython arms** (§ 4) — 33 test files, 29 TSVs. Any time after step 1 completes for a
   given slice; simplest as one sweep at the end.
7. **Port or retire the two orphan gates** of § 5 — `test_claude_md_reference.py`, and the ONE-gate
   policy into the Rust runner with a pinned run count.
8. **Delete `turbojet/` and `tests/`** — last, together, in one commit with the `CLAUDE.md` rewrite.
   The per-rung Python tests go **last, not first**: the Rust gates were derived from them, and this
   project has recorded **two** instances of a ported gate going vacuous (slice U step 4, slice AB
   step 2) that were only caught because the source was still there to compare against.

**Do none of it inside a slice step.** Same rule as `todo-test-runtime.md` § 7, and for the same
reason: a deletion invalidates the very run a step is waiting on, which is how a green gate stops
meaning anything.
