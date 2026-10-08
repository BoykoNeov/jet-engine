
#### 5.31.4 SLICE AG step 4 — rung 76's cap, and **A DEFECT I DECLARED SILENT IN MY OWN DOC COMMENT, KILLED BY A REFUSAL PORTED THREE STEPS BEFORE ANY CALLER COULD TRIP IT**

**SHIPPED**: `src/sensed_cap.rs` **305 → 578 lines** — `cap_march`, `CapScope` (`_with_cap`),
`accel_for`, `c_at`, plus the two carried defaults `ACCEL_SCHEDULE_N` and `C_AT_REL`.
`_sensed_cap`, the fifth name § 5.31 (vi) lists at this step, landed at step 1 with the re-cut
recorded in its own doc comment. **NO gate file**: step 3's precedent, itself slice AF step 4's —
a bodies step with no readers of its own proves itself by DRIVING, and the ported gates are step
6's. **`Rust == PyPy` bit for bit on all 1 422 keys, both key sets equal, no port fix.** Sweep: **14
injections, 9 KILLED, 5 SURVIVED, 14 of 14 VERDICTS right and THREE mechanisms or counts wrong** —
and every one of those three is a finding.

**THE STEP IS THIN AND WAS NOT MERGED.** Four methods, 53 Python lines against step 3's 934 Rust.
Folding step 5's readers in would have made the step look sized and would have settled **P7** — the
7-versus-6 disagreement between the class-count law and the method-count law — by convenience, on
the one prediction whose whole content is the step count. It ships thin.

##### (a) THE LEADING FINDING — **THE BLINDNESS I WROTE DOWN HAS A DOMAIN, AND THE ARM THIS RUNG IS ABOUT IS DEFENDED BY A REFUSAL ALREADY IN THE FILE**

`cap_march` is `_windup_march` plus two lines, and the load-bearing one is the leg:
`accel: Some(accel)` where rung 75 has `None`. The doc comment written before the sweep said so,
and then said what dropping it would cost:

> **Drop it and the failure is silent in the worst available way**: `r76_sensed_cap` is never
> dispatched, the cap is rung 48's set-point solve, and the trajectory is rung 75's — which IS this
> rung's reduce-arm answer, so every reduce comparison the crate owns goes on passing.

**Injection 1 deleted the arm. The drive did not diff — it PANICKED**, at `sensed_cap.rs:300`,
which is `r76_integrate_fuel`'s THIRD refusal:

> rung-76: `sensed` re-reads rung 48's `Wf/pt3` schedule, so there must BE one. The phi leg and the
> governor have no sensed form in any rung — a floor on a STATE is not a formula for a FUEL.

That refusal was ported at **step 1**, three steps before this step wrote the first caller capable
of reaching it with `accel = None`. So the sentence is right about the `solve` arm — where the
march is `windup_march`'s exactly and every reduce in the crate passes — and **wrong about the
`sensed` arm, which is the arm the rung exists for**. The repair is ADDITIVE, § 5.31 (ix)'s shape
for the third time in this slice: the reduce-blindness paragraph stays, with its DOMAIN named and
the measured panic beside it.

**The generalisation is step 3 § (a)'s, running the other way.** There, a plain write I proved
inert with the PREVIOUS rung's evidence was killed by a refusal born at THIS rung. Here, a defect I
proved silent from the reduce spine's shape was killed by a refusal shipped THREE STEPS EARLY.
Both are the same question asked at the wrong boundary: *what reads this?* has a sibling, **what
REFUSES this — and when was that enumeration last taken?** An enumeration of what a refusal
protects expires at the step that adds its caller, exactly as an enumeration of a field's readers
expires at the rung that adds one.

##### (b) THE SECOND — **NEITHER WRONG RESTORE POLICY MOVES A SINGLE VALUE, AND ONE OF THEM MOVES ONE KEY IN 1 422**

`CapScope` is `_with_cap`, and it restores the PREVIOUS law where `_c_at`'s two state guards three
items below it restore `None` — two policies, one file, because the fields are of different kinds.
Both wrong policies were injected, and both predictions said *KILLED, exactly 2 keys*:

| injection | predicted | measured |
|---|---|---|
| `Drop` restores the constant `"solve"` | KILLED, 2 keys | **KILLED, exactly ONE** — `D/scope/after` |
| `Drop` restores nothing | KILLED, 2 keys | **KILLED, 19** — 18 `law_after` tags + `D/scope/inside_outer_again` |

**The restore-a-constant defect is invisible at 18 of its 19 sites for a reason that is not about
this guard at all**: the drive's marched machine rests at `"solve"`, so the injected constant
COINCIDES with the value it displaced everywhere except the single nested read whose outer scope
was armed `"sensed"`. That is step 3 § (h) row 13's shape — a gate that picks one cell to keep the
runtime down scores the bug as correct — moved from a clock onto a law.

**And not one of the 20 keys either injection moves is a cap VALUE.** Section D drives the guard
end to end through `_cap_fuel` at 18 marched states under both laws, and every one of those numbers
is identical under both defects, because every caller SETS the law immediately before reading it,
so a leaked or wrongly-restored law is always overwritten before anything consults it. **A step-6
gate that drives `_with_cap` through a real reader — the natural shape, and the one this drive
started with — is blind to both.** What catches them is reading the field WITHOUT setting it first,
on a machine whose resting law is not the one the guard arms. Written into `CapScope`'s doc as a
requirement on step 6 rather than left for step 6 to rediscover.

##### (c) THE THIRD — **THE PACKAGE'S ONE EXPRESSION-FIRST `max` IS UNDRIVABLE, AND THE STEP THAT DISCHARGES THE OBLIGATION IS THE STEP THAT MEASURES IT CANNOT BE CHECKED**

Step 1 § (c) censused all 268 n-ary `max`/`min` calls in `engine.py`, found 103 literal-first (every
one faithful under `lit.max(x)`, because Python seeds its fold at argument 0 and replaces only on a
strict comparison), and booked the single expression-first `1e-9` fold — `_c_at`'s
`dw = rel * max(w, 1e-9)` — forward to **this step**, naming the method and the spelling:
`if 1e-9 > w { 1e-9 } else { w }`, because `w.max(1e-9)` is `1e-9` for a NaN `w` where Python's is
`nan`.

**The obligation is discharged and the discharge is unverifiable, which is now measured rather than
assumed.** The drive calls `c_at` at `w =` 1e-12, 1e-10, 1e-9, 1e-8 and the marched `mf`:
**`C/fold/0..3? = 0` — all four sub-hinge cases ABORT**, in both languages identically, because the
hinge sits at `1e-9` and the plant refuses every fuel below about 1e-2 (`rung-43 fuel closure does
not bracket … off the modeled speed-line region`). And **injection 8 — the fold replaced by
`w.max(1e-9)` — SURVIVES at 0 of 1 422.** So the two spellings are indistinguishable by any value
this plant can produce, and the faithful one is held in place by the census and the doc comment
alone. That is a **defence with no reader** ([[rust-port-slice-aa-steps2345]]) — disclosed here at
the step that shipped it, rather than found by a later slice wondering why the line is written the
long way.

##### (d) `accel_for`'s HOOK DISPATCH IS A RULE, NOT A MEASURED DIFFERENCE — AND IT SAYS SO

`accel_for` reaches `_shared_rig` through `core.triple_hooks()`. The name has eight definers and
rung 76 re-aims the cell precisely so the rig carries `_cap_law`, so the dispatch is correct by the
port's standing rule. **Injection 4 replaced it with the direct rung-75 pointer: 0 of 1 422 keys
moved**, because a schedule is read off EQUILIBRIA and no cap law touches those. Two of its
siblings behave the same way and are recorded beside it — `tau_rel = tau_f` instead of `3·tau_f`
(injection 6) and `tau_att` taken from the GOVERNOR's clock (injection 7) are both invisible here,
while **the identical `tau_rel` mutation inside `cap_march` kills 688 keys** (injection 13). A lag
is a march parameter; an equilibrium does not have one. The doc comment names which of the two
reasons each line is there for, which is the point of the measurement.

##### (e) THE PRE-FLIGHT'S `341 OF 341` AND THIS DRIVE'S `333` ARE ONE MEASUREMENT ON TWO POPULATIONS

§ 5.31 (i) published *341 of 341 trajectory points differ* at `PHI_BOTH`. This drive first measured
**333**. The gap is not a port defect and not a settings drift: `probe_d_arming.py:179` compares
the **ELEVEN-KEY TUPLE**, where the drive's `n_diff` compared `Tt4` alone, and the eight agreeing
points are exactly the first eight (`s = 0 … 0.035`), where `mf` and `w_fuel` have already parted
while `Tt4` is still bit-equal. Both counts are now emitted with their populations named —
`n_diff` 333, `n_diff_tuple` **341**, `n_diff_mf` 333 — which is § 5.31 (i)'s own leading lesson
applied to its own number.

**And `max|ΔTt4| = 10.254967637311893` reproduces the pre-flight to every digit through a DIFFERENT
entry point.** The pre-flight marched `_stator_march` directly; this drives `_cap_march`. So
`_cap_march`'s rig is CONFIRMED identical to the one the pre-flight measured, rather than assumed
to be, and the number independently reproduces `docs/rung76-spec.md` § 1.3's shipped
`1179.24 → 1168.98 K` for the second time.

##### (f) THE TWO CARRIED DEFAULTS, AND WHY THEY ARE NAMED CONSTANTS

`accel_schedule`'s `n = 13` and `_c_at`'s `rel = 1e-6` are Python defaults that no call site in
`engine.py` spells, so in Rust — which has no defaults — they would be bare literals at the one
place each is used. Both are named. `ACCEL_SCHEDULE_N` is the crate's ONLY reliance on that default
(every other `accel_schedule` caller passes a value its own Python line writes out), and injection
5 — `n = 12` — **kills 496 keys**, so the number is load-bearing rather than decorative.

##### (g) THE SWEEP — 14 injections, predictions typed first, **9 KILLED / 5 SURVIVED, verdicts 14 of 14**

Instrument: patch `src/sensed_cap.rs`, rebuild, re-run the drive, diff its 1 422 keys against the
PyPy golden. Source SHA-256 verified back to pristine after every run
(`da3653cbf3255c4e690751d13d60605acf9b96c40686c57198eeff4ab72d4468`; that hash PREDATES § (a)'s,
§ (b)'s and § (d)'s doc repairs, so it is not the shipped file's — the shipped one is
`289487e0e2f2c6abb79ece41abecaed35032a3e9384115ee975ff360781d58fe`, and the drive was re-run
against it before the harness was deleted, because a reasoned *a doc comment cannot move a float*
is worth exactly one run).

| # | injection | predicted | measured |
|---|---|---|---|
| 1 | `cap_march`'s leg takes `accel: None` | KILLED, large | **KILLED by PANIC** → § (a) |
| 2 | `cap_march` drops `cap_law.set(cap_law)` | KILLED | **KILLED**, 168 keys |
| 3 | `cap_march` drops `ic_cap.set(…)` | **SURVIVES** — no reader here raises the cap | **SURVIVED**, as registered |
| 4 | `accel_for` reaches `R75_TRIPLE` directly | **SURVIVES** — a schedule is read off equilibria | **SURVIVED** → § (d) |
| 5 | `accel_schedule` `n = 12` | KILLED | **KILLED**, 496 keys |
| 6 | `accel_for` `tau_rel = tau_f` | **SURVIVES** — a lag is a march parameter | **SURVIVED** → § (d) |
| 7 | `accel_for` `tau_att` = the governor's clock | **SURVIVES** — same, and the grid is uniform | **SURVIVED** → § (d) |
| 8 | `c_at`'s fold becomes `w.max(1e-9)` | **SURVIVES** — the hinge is unreachable | **SURVIVED** → § (c) |
| 9 | `c_at` differences one-sidedly | KILLED | **KILLED**, 55 keys |
| 10 | `c_at`'s two state guards SWAPPED | KILLED | **KILLED**, 27 keys |
| 11 | `CapScope::drop` restores `"solve"` | KILLED, exactly 2 | **KILLED, exactly 1** → § (b) |
| 12 | `CapScope::drop` restores nothing | KILLED, exactly 2 | **KILLED, 19** → § (b) |
| 13 | `cap_march` `tau_rel = tau_f` | KILLED | **KILLED**, 688 keys |
| 14 | `c_at` drops the `/ pi_b` | KILLED | **KILLED**, 55 keys |

**14 of 14 verdicts right is the WEAK reading of this table, and the three misses are the strong
one** — [[rung81-authority-clock]]'s *the first grid's 100% was the weaker measurement*, one slice
on. The verdict axis carried nothing this step did not already know from the shape of the code; the
MECHANISM axis (row 1) and the COUNT axis (rows 11, 12) carried §§ (a) and (b).

##### (h) SIZING — the step is 3.15× on BODIES and 5.09× on the FILE, and P1 is about neither yet

270 added lines, of which **85 are code and 175 are doc**, against Python's 53 lines of which 27 are
code. So this step is **3.15×** by body and **5.09×** by region, both outside **P1**'s 2.0–2.6×
band. That is not P1 falsified — P1 is a SLICE total, settled at the last step — but it is the data
point that says a four-small-method step is doc-dominated and will pull the slice ratio up, which is
worth having on the record before the ratio is computed rather than after.

##### (i) GATES

`cargo test --release`: **CARGO_GATE_PLACEHOLDER**. The throwaway harness
(`rust/tests/slice_ag_step4_drive.rs`, archived to `W:\temp\claude\slice-ag-step4\`) was **deleted
before the gate ran**, so this step adds no binary and no test — the delta against step 3 is
expected to be ZERO, and a change in it would have been the finding.

**Step 2's four `clippy::type_complexity` warnings on `RhsLaws` are CLOSED**, with the `#[allow]`
its rung-74 twin `DemandLaws` has always carried. § 5.31.3 (i) recorded them as slice AG's own and
the fix is one line; the two `clippy::eq_op` errors elsewhere in the crate are unrelated, are
deliberate NaN self-comparisons ported faithfully, and remain a lint-configuration debt.

##### (j) WHAT STEP 5 INHERITS

* **Rung 76's readers** — `_cap_rows`, `cap_gains`, `cap_bill`, `solve_gain` — every one of which
  consumes step 4's four methods, so step 5 re-drives them rather than re-proving them.
* **§ (b)'s requirement on step 6**, stated now so it is not rediscovered: a gate on `CapScope`
  must read `_cap_law` WITHOUT setting it first, on a machine whose resting law is not the one the
  guard arms. Driving the guard through `_cap_fuel` sees neither wrong restore policy.
* **§ (c)'s disclosure**, which step 6 cannot repair and should not try to: no value gate can
  distinguish `_c_at`'s two `max` spellings on this plant.
* **P2's exemption**, still open and still pointed at `cap_bill` — which is step 5's reader, and
  which the ORACLE settles at step 6. This step drove PyPy only, deliberately.
