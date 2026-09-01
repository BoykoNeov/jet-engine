//! SLICE AB step 4 — **THE ORACLE for rung 69**, against PyPy *and* CPython 3.14.
//!
//! `rung69.rs` ports the suite's own 25 gates, and most of them are RELATIONS — a reduce arm equal
//! to a rung-68 march, `pair_RC` near 1, a sign table that flips between two references. Relations
//! are agreement, not correctness: two marches of the same binary can agree with each other and
//! both be wrong. **This file is the other half**: every value `oracle/dump_slice_ab.py` emits at
//! the SUITE's OWN GRID, nothing coarsened, every float compared as its IEEE-754 bit pattern.
//!
//! # THIS FILE EXISTS BECAUSE OF ONE MEASUREMENT
//!
//! Step 3 injected a cut Newton budget into [`cubic_roots_c`] — 80 steps to 20 — and **every
//! slice-AB binary stayed green**, while the injection moved 56 of 243 root components and 24 of
//! 81 `worst_zero` values. The one derived key a gate reads, `n_zero`, did not move, and § 5.26
//! (iii) had measured why before a line of Rust was written: the threshold sits 3.5 decades away.
//! So P4 — *"Rust reproduces `_cubic_roots_c` bit-for-bit on all 256 triples, the 72 exhausted ones
//! included"* — was **unsettleable before this step**. Two independent constructions settle it:
//!
//! * **sections D/E/F** emit every root the readers compute, as `re`/`im`/`abs` bits — the plant's
//!   own call stream, roots included; and
//! * **section I** replays the SHIPPED solver on the coefficient triples the dump INTERCEPTED at
//!   those same call sites, with the plant taken out of the loop.
//!
//! Section I's `c2`/`c1`/`c0` are **INPUTS, not assertions** — [`Cmp::input_f`] reads them from the
//! golden and marks them consumed rather than comparing them against themselves, which would be
//! the vacuity slice U step 4 recorded (*a gate comparing a key with ITSELF cannot see its
//! value*). What is asserted is the nine root components each triple produces, plus a count that
//! the Rust computes for itself: `reference_modes`, `damping_floor` and `rk4_margin` are the only
//! three callers, so `I/ncalls` must equal the root-carrying rows sections D, E and F emit.
//!
//! **The intercepted count is 94, not § 5.26 (iii)'s 256**, and the difference is not a gap: 256 is
//! what the whole `pytest` session makes, where several gates call the same reader again, and this
//! dump calls each reader once. **24 of the 94 spend all 80 Newton steps without converging**, so
//! the exhausted arm — the only one a cut budget moves — is covered rather than assumed to be.
//!
//! # THE CROSS-INTERPRETER EXEMPTION — a set of NAMES, measured from the diff
//!
//! § 5.26 (i) measured a **three-element** `sum()` diverging between interpreters, which slice
//! AA's own explanation says cannot happen, and refuted the obvious replacement (cancellation)
//! with the same probe: whether Neumaier's correction survives the final rounding is a bit-pattern
//! property of the particular summands. `_invariants`' `c1` differs on 23 of 256 instances under
//! CPython 3.14 while `c2` — built the same way, at the same site, from the same three numbers —
//! agrees on all 256. See [`EXEMPT`], which is read off this dump's own diff and carries what each
//! subtree is. **The set is checked in BOTH directions**: a key that STOPS drifting is as much a
//! change as a new one. **The port is held to PyPy**, where nothing is exempt.
//!
//! # WHAT THIS ORACLE CANNOT SEE, NAMED HERE SO STEP 5 OWNS IT
//!
//! * **The ten cells' DISPATCH.** No value key can witness a hook table — a cell that computes the
//!   same number a different way passes every key here. That is `slice_ab_dispatch.rs`'s subject,
//!   and § 5.26 (ii) measured that four of those cells (`_solve_v`, `_manifold_v`, `_triple_rig`,
//!   and `_rk4_floor` through its message) are observable ONLY by panic, which no dump reaches.
//! * **`_ref`'s restore policy.** § 5.26 (vi) measured 29 sets and 29 restores-to-`None` with zero
//!   nesting, so no reachable reader distinguishes restore-to-`None` from restore-to-previous;
//!   `slice_ab_cells.rs` manufactures it.
//! * **The four `__init__` guards**, which are raises and therefore not values.
//! * **`RefModesRow::zeta == None`** — the fourth degenerate branch of § 5.26.2 (h), and the one
//!   section K does NOT reach. It is unreachable by construction rather than unreached: the roots
//!   sum to `c2 = -(1/tau_g + 1/tau_q + 1/tau_s)`, non-zero for every finite positive clock, so
//!   they cannot all be zero and the dominant one has non-zero modulus. The other three ARE
//!   reached, by section K's declared extra grid.
//!
//! Regenerate both:
//! ```text
//! .venv/Scripts/python.exe rust/oracle/dump_slice_ab.py > rust/oracle/slice_ab_pypy.tsv
//! C:/Python314/python.exe  rust/oracle/dump_slice_ab.py > rust/oracle/slice_ab_cpython.tsv
//! ```
//! **Through a POSIX shell, not PowerShell 5.1** — it writes a UTF-8 BOM that lands in front of
//! the `#` on line 1, so the header parses as data. [[windows-tooling-file-hazards]].

use std::collections::{BTreeMap, BTreeSet};

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::reference_split::{
    build_reference_split_cascade, cubic_roots_c, damping_floor, reference_bill, reference_gains,
    reference_modes, ring_visibility, rk4_margin, RingArm, StatorIncidenceLimiter,
};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{
    build_three_loop_cascade, riding, violation_inc, StatorLimiter, TripleBill, TripleGains,
    TripleRigArm,
};
use turbojet::two_lag::violation;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_ab_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_ab_cpython.tsv");

/// The number of keys `dump_slice_ab.py` emits — **its own stderr tally, not a neighbouring
/// slice's**. `load` bars 95 % of it, so a golden truncated mid-write cannot present as a pass.
const GOLDEN_KEYS: usize = 15_957;

/// The key NAMES the CPython 3.14 arm is allowed to differ on — **MEASURED off this dump, and
/// § 5.26's P3 IS FALSIFIED: there are TWO causes and P3 named one.**
///
/// P3 read *"the exemption is the set of names downstream of `_invariants`' `c1`, plus the
/// inherited `ic_family` `withheld` subtree"*. The `withheld` subtree does not appear at all — no
/// rung-69 reader calls `ic_family` — and **134 of these 194 names have nothing to do with any
/// `sum()`**. The two causes, each measured rather than reasoned:
///
/// **1. THE THREE-ELEMENT `sum()` — 60 names, every one descended from SIX `c1` values.**
/// § 5.26 (i) measured CPython's Neumaier-compensated `sum` diverging from a naive left fold on a
/// list of THREE, which slice AA's own explanation says cannot happen. Here 6 of section D's `inc`
/// rows carry a diverging `c1`, and the rest of the group is what they feed: `c1_rel`, the three
/// roots, `zeta`, `worst_zero`, and the `mod`/`ratio`/`zeta` keys of sections E and F, whose rows
/// call the same `_invariants` without emitting its coefficients. `c2` — built the same way, at
/// the same site, from the same three numbers — diverges on NOTHING.
///
/// **2. CPython 3.14 CHANGED WHAT MIXED float/complex ARITHMETIC MEANS ON A SIGNED ZERO — 134
/// names, and it is a semantics change, not a rounding.** Measured directly in both interpreters:
///
/// | expression | PyPy 3.11 | CPython 3.14 |
/// |---|---|---|
/// | `(-60.0) - (4+0j)`, `im` | `0.0 - 0.0` = **`+0.0`** | **`-0.0`** — the `0.0 -` is skipped |
/// | `0.5 * (-0-4j)`, `re` | `0.5*-0.0 - 0.0*-4` = **`+0.0`** | **`-0.0`** — no cross term |
/// | `(-60.0) + (4+0j)`, `im` | `+0.0` | `+0.0` — addition is unaffected |
///
/// CPython 3.14 takes a real-operand fast path in `complex.__rsub__` and `__rmul__` that drops the
/// zero cross-terms; PyPy promotes the float to a complex and runs the full formula. **The port is
/// held to PyPy** — which is also pre-3.14 CPython — so these are an audit-arm note and not a
/// defect: `Rust ≡ PyPy` on all 15 957 keys. Every one of the 134 is a root's `im`, except the
/// single `J/5/root/2/re` that [`py_half`](turbojet::reference_split) was written for.
///
/// **AND SECTION I CARRIES ZERO NAMES FROM CAUSE 1, WHICH IS THE POINT OF ITS DESIGN.** It replays
/// the solver on the coefficients the DUMP intercepted, so in this arm it is fed CPython's own
/// `c1` and the sum drops out — **67 names that differ between the two TSVs do not differ between
/// Rust and CPython.** An exemption transcribed from a PyPy-vs-CPython diff, which is the obvious
/// move and was this file's first draft, would have listed those 67 and asserted nothing about
/// them.
const EXEMPT: [&str; 194] = [
    // --- cause 1: the three-element `sum()`, all descended from six `c1` values ---
    "D/0/inc/2/c1",
    "D/0/inc/2/c1_rel",
    "D/0/inc/2/root/0/abs",
    "D/0/inc/2/root/0/re",
    "D/0/inc/2/root/1/abs",
    "D/0/inc/2/root/1/im",
    "D/0/inc/2/root/2/abs",
    "D/0/inc/2/root/2/im",
    "D/0/inc/2/worst_zero",
    "D/0/inc/2/zeta",
    "D/0/inc/4/c1",
    "D/0/inc/4/c1_rel",
    "D/0/inc/4/root/0/abs",
    "D/0/inc/4/root/0/re",
    "D/0/inc/4/root/1/im",
    "D/0/inc/4/root/2/im",
    "D/0/inc/4/worst_zero",
    "D/0/inc/5/c1",
    "D/0/inc/5/c1_rel",
    "D/0/inc/5/root/0/abs",
    "D/0/inc/5/root/0/re",
    "D/0/inc/5/root/1/abs",
    "D/0/inc/5/root/1/im",
    "D/0/inc/5/root/2/abs",
    "D/0/inc/5/root/2/im",
    "D/0/inc/5/worst_zero",
    "D/0/inc/5/zeta",
    "D/0/inc/6/c1",
    "D/0/inc/6/c1_rel",
    "D/0/inc/6/root/0/abs",
    "D/0/inc/6/root/0/re",
    "D/0/inc/6/worst_zero",
    "D/1/inc/11/c1",
    "D/1/inc/11/c1_rel",
    "D/1/inc/11/root/0/abs",
    "D/1/inc/11/root/0/re",
    "D/1/inc/11/root/1/abs",
    "D/1/inc/11/root/1/re",
    "D/1/inc/11/worst_zero",
    "D/1/inc/8/c1",
    "D/1/inc/8/c1_rel",
    "D/1/inc/8/root/0/abs",
    "D/1/inc/8/root/0/re",
    "D/1/inc/8/root/1/abs",
    "D/1/inc/8/root/1/re",
    "D/1/inc/8/root/2/abs",
    "D/1/inc/8/root/2/re",
    "D/1/inc/8/worst_zero",
    "E/3/mod",
    "E/3/zeta",
    "E/tightest/zeta",
    "F/0/mod",
    "F/0/ratio",
    "F/3/mod",
    "F/3/ratio",
    "F/6/mod",
    "F/6/ratio",
    "F/ds_lambda",
    "F/max_mod",
    "F/max_ratio",
    // --- cause 2: CPython 3.14's real-operand fast path, a SIGNED ZERO only ---
    "D/0/phi/0/root/2/im",
    "D/0/phi/1/root/2/im",
    "D/0/phi/10/root/2/im",
    "D/0/phi/11/root/2/im",
    "D/0/phi/12/root/2/im",
    "D/0/phi/2/root/2/im",
    "D/0/phi/3/root/2/im",
    "D/0/phi/4/root/2/im",
    "D/0/phi/5/root/2/im",
    "D/0/phi/6/root/2/im",
    "D/0/phi/7/root/2/im",
    "D/0/phi/8/root/2/im",
    "D/0/phi/9/root/2/im",
    "D/1/inc/0/root/2/im",
    "D/1/inc/1/root/2/im",
    "D/1/inc/10/root/2/im",
    "D/1/inc/11/root/2/im",
    "D/1/inc/12/root/2/im",
    "D/1/inc/2/root/2/im",
    "D/1/inc/3/root/2/im",
    "D/1/inc/4/root/2/im",
    "D/1/inc/5/root/2/im",
    "D/1/inc/6/root/2/im",
    "D/1/inc/7/root/2/im",
    "D/1/inc/8/root/2/im",
    "D/1/inc/9/root/2/im",
    "D/1/phi/0/root/2/im",
    "D/1/phi/1/root/2/im",
    "D/1/phi/10/root/2/im",
    "D/1/phi/11/root/2/im",
    "D/1/phi/12/root/2/im",
    "D/1/phi/13/root/2/im",
    "D/1/phi/14/root/2/im",
    "D/1/phi/15/root/2/im",
    "D/1/phi/16/root/2/im",
    "D/1/phi/17/root/2/im",
    "D/1/phi/18/root/2/im",
    "D/1/phi/2/root/2/im",
    "D/1/phi/3/root/2/im",
    "D/1/phi/4/root/2/im",
    "D/1/phi/5/root/2/im",
    "D/1/phi/6/root/2/im",
    "D/1/phi/7/root/2/im",
    "D/1/phi/8/root/2/im",
    "D/1/phi/9/root/2/im",
    "D/2/phi/0/root/2/im",
    "D/2/phi/1/root/2/im",
    "D/2/phi/2/root/2/im",
    "D/2/phi/3/root/2/im",
    "D/2/phi/4/root/2/im",
    "D/2/phi/5/root/2/im",
    "D/2/phi/6/root/2/im",
    "D/3/phi/0/root/2/im",
    "D/3/phi/1/root/2/im",
    "D/3/phi/10/root/2/im",
    "D/3/phi/11/root/2/im",
    "D/3/phi/2/root/2/im",
    "D/3/phi/3/root/2/im",
    "D/3/phi/4/root/2/im",
    "D/3/phi/5/root/2/im",
    "D/3/phi/6/root/2/im",
    "D/3/phi/7/root/2/im",
    "D/3/phi/8/root/2/im",
    "D/3/phi/9/root/2/im",
    "I/10/root/2/im",
    "I/11/root/2/im",
    "I/12/root/2/im",
    "I/13/root/2/im",
    "I/14/root/2/im",
    "I/15/root/2/im",
    "I/16/root/2/im",
    "I/17/root/2/im",
    "I/18/root/2/im",
    "I/19/root/2/im",
    "I/20/root/2/im",
    "I/21/root/2/im",
    "I/22/root/2/im",
    "I/23/root/2/im",
    "I/24/root/2/im",
    "I/25/root/2/im",
    "I/26/root/2/im",
    "I/27/root/2/im",
    "I/28/root/2/im",
    "I/29/root/2/im",
    "I/30/root/2/im",
    "I/31/root/2/im",
    "I/32/root/2/im",
    "I/33/root/2/im",
    "I/34/root/2/im",
    "I/35/root/2/im",
    "I/36/root/2/im",
    "I/37/root/2/im",
    "I/38/root/2/im",
    "I/39/root/2/im",
    "I/40/root/2/im",
    "I/41/root/2/im",
    "I/42/root/2/im",
    "I/43/root/2/im",
    "I/44/root/2/im",
    "I/45/root/2/im",
    "I/46/root/2/im",
    "I/47/root/2/im",
    "I/48/root/2/im",
    "I/49/root/2/im",
    "I/50/root/2/im",
    "I/51/root/2/im",
    "I/57/root/2/im",
    "I/58/root/2/im",
    "I/59/root/2/im",
    "I/60/root/2/im",
    "I/61/root/2/im",
    "I/62/root/2/im",
    "I/63/root/2/im",
    "I/69/root/2/im",
    "I/7/root/2/im",
    "I/70/root/2/im",
    "I/71/root/2/im",
    "I/72/root/2/im",
    "I/73/root/2/im",
    "I/74/root/2/im",
    "I/75/root/2/im",
    "I/76/root/2/im",
    "I/77/root/2/im",
    "I/78/root/2/im",
    "I/79/root/2/im",
    "I/8/root/2/im",
    "I/80/root/2/im",
    "I/9/root/2/im",
    "J/0/root/2/im",
    "J/1/root/2/im",
    "J/2/root/2/im",
    "J/5/root/2/re",
    "J/6/root/2/im",
    "J/8/root/2/im",
];

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, bits) = (it.next().expect("key"), it.next().expect("bits"));
        let v = bits.parse::<u64>().unwrap_or_else(|e| panic!(
            "slice-AB golden line is not `key<TAB>u64` ({e}): {line:?}. If the second field has \
             text appended, the dump was redirected with `2>&1` and its stderr interleaved. If \
             the FIRST line failed, the file has a UTF-8 BOM: it was redirected through \
             PowerShell, which writes one."));
        assert!(m.insert(k.to_string(), v).is_none(), "dup {k}");
    }
    // MEASURED off THIS dump's own emitted count — never inherited from a neighbouring slice.
    assert!(m.len() > GOLDEN_KEYS - GOLDEN_KEYS / 20,
            "the slice-AB golden did not parse ({} keys, expected about {GOLDEN_KEYS})", m.len());
    m
}

/// Accumulates every disagreement so ONE run reports them all, **and reports every golden key the
/// Rust never asked for** — a field missing from the port is invisible until that half fires.
struct Cmp {
    py: BTreeMap<String, u64>,
    seen: BTreeSet<String>,
    bad: Vec<String>,
    cpython: bool,
    exempted: BTreeSet<String>,
    drifts: Vec<(String, f64)>,
    flips: Vec<String>,
}

impl Cmp {
    fn new(py: BTreeMap<String, u64>, cpython: bool) -> Self {
        Cmp { py, seen: BTreeSet::new(), bad: Vec::new(), cpython,
              exempted: BTreeSet::new(), drifts: Vec::new(), flips: Vec::new() }
    }

    fn f(&mut self, key: &str, got: f64) {
        assert!(got.is_finite(), "{key} is not finite: {got}");
        self.raw(key, got.to_bits(), false);
    }

    fn d(&mut self, key: &str, got: usize) { self.raw(key, got as u64, true); }

    fn b(&mut self, key: &str, got: bool) { self.raw(key, got as u64, true); }

    /// The dump's `s(...)` — a string as its FNV-1a 64-bit hash. `v_regime`, `ic_order`, `branch`
    /// and the off-regime ARM NAMES are the non-floats a rung-69 reading carries, and the regime
    /// is the one thing no float can witness.
    fn s(&mut self, key: &str, got: &str) { self.raw(key, fnv1a(got), true); }

    fn opt(&mut self, key: &str, got: Option<f64>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some(x) = got { self.f(key, x); }
    }

    /// **A GOLDEN KEY READ AS AN INPUT AND NOT COMPARED.** Section I replays the shipped solver on
    /// the coefficients Python's interceptor captured; re-emitting those coefficients as
    /// assertions would compare a key with itself, which slice U step 4 recorded as a gate that
    /// cannot see its own value. Marking them `seen` keeps the missing-key half of [`Cmp::finish`]
    /// honest without pretending they were checked.
    fn input_f(&mut self, key: &str) -> f64 {
        assert!(self.seen.insert(key.to_string()), "the Rust read {key} twice");
        f64::from_bits(*self.py.get(key)
            .unwrap_or_else(|| panic!("{key}: NO GOLDEN — section I's input is missing")))
    }

    fn input_d(&mut self, key: &str) -> usize {
        assert!(self.seen.insert(key.to_string()), "the Rust read {key} twice");
        *self.py.get(key)
            .unwrap_or_else(|| panic!("{key}: NO GOLDEN — section I's input is missing")) as usize
    }

    fn raw(&mut self, key: &str, got: u64, discrete: bool) {
        assert!(self.seen.insert(key.to_string()), "the Rust emitted {key} twice");
        match self.py.get(key) {
            None => self.bad.push(format!("{key}: NO GOLDEN (the dump never emitted it)")),
            Some(&want) if want != got => {
                if self.cpython && EXEMPT.contains(&key) {
                    self.exempted.insert(key.to_string());
                } else if self.cpython && discrete {
                    self.flips.push(format!("{key}: rust {got} vs cpython {want}"));
                } else if self.cpython {
                    let (a, b) = (f64::from_bits(got), f64::from_bits(want));
                    let rel = if b == 0.0 { (a - b).abs() } else { ((a - b) / b).abs() };
                    self.drifts.push((format!("{key}: {a:.17e} vs {b:.17e}"), rel));
                } else if discrete {
                    self.bad.push(format!("{key}: rust {got} vs python {want}"));
                } else {
                    self.bad.push(format!(
                        "{key}: rust {:.17e} ({got:016x}) != py {:.17e} ({want:016x})",
                        f64::from_bits(got), f64::from_bits(want)));
                }
            }
            Some(_) => {}
        }
    }

    fn finish(&self, arm: &str) {
        let missed: Vec<&String> = self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        assert!(self.flips.is_empty(),
                "{} DISCRETE keys flipped between interpreters -- a flipped count, flag, regime \
                 label or off-regime arm name is a different physical reading, never a \
                 rounding:\n  {}",
                self.flips.len(),
                self.flips.iter().take(12).cloned().collect::<Vec<_>>().join("\n  "));
        let worst = self.drifts.iter().map(|(_, r)| *r).fold(0.0_f64, f64::max);
        assert!(self.drifts.is_empty(),
                "{} float keys drifted against CPython OUTSIDE the declared exemption (worst \
                 {worst:.3e}). The exemption is a NAMED LIST rooted in ONE three-element `sum()` \
                 -- read this file's header before widening it, and never replace it with a \
                 tolerance:\n  {}",
                self.drifts.len(),
                self.drifts.iter().take(12).map(|(l, r)| format!("{l} (rel {r:.3e})"))
                    .collect::<Vec<_>>().join("\n  "));
        if self.cpython {
            let want: BTreeSet<String> = EXEMPT.iter().map(|s| (*s).to_string()).collect();
            assert_eq!(self.exempted, want,
                       "the CPython exemption set MOVED. Expected exactly the names in `EXEMPT`; \
                        got {} names. A key that STOPPED drifting is a change too -- it would \
                        mean the port's fold, the dump or CPython's `sum()` moved.\n\
                        only-in-EXEMPT: {:?}\nonly-measured: {:?}",
                       self.exempted.len(),
                       want.difference(&self.exempted).take(20).collect::<Vec<_>>(),
                       self.exempted.difference(&want).take(20).collect::<Vec<_>>());
        }
        if self.bad.is_empty() && missed.is_empty() {
            println!("slice_ab_oracle ({arm}): {} values compared, {} exempt",
                     self.seen.len(), self.exempted.len());
            return;
        }
        panic!(
            "{} of {} compared keys differ:\n  {}\n\n{} golden keys the Rust never asked for (a \
             field missing from the port is invisible until this fires):\n  {:?}",
            self.bad.len(), self.seen.len(), self.bad.join("\n  "), missed.len(),
            missed.iter().take(24).collect::<Vec<_>>());
    }
}

fn fnv1a(text: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for byte in text.bytes() {
        h = (h ^ byte as u64).wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

// ------------------------------------------------------------------------------------ the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const V_MAX: f64 = 0.20;
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
const DISP: f64 = 0.05;
/// `reference_modes`' own default, which the suite takes wholesale — written `(tau_v, tau_att,
/// tau_s)`, which is NOT the `(g, q, v)` order the arms report their `taus` in.
const CLOCKS: [(f64, f64, f64); 4] =
    [(0.05, 0.05, 0.05), (0.05, 0.005, 0.05), (0.05, 0.5, 0.05), (0.02, 0.05, 0.10)];
/// `damping_floor`'s own default.
const DAMP_GRID: [(f64, f64, f64); 6] =
    [(0.05, 0.05, 0.05), (0.05, 0.05, 0.025), (0.05, 0.05, 0.10),
     (0.10, 0.10, 0.05), (0.02, 0.20, 0.05), (0.20, 0.02, 0.05)];

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn t_c() -> f64 { lp().tan_beta1_crit() }
fn m_lim() -> f64 { t_c() - 1.0 / PHI }

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn full(b: ScheduledStatorTransient) -> ScheduledStatorCore {
    match b {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this grid never disables LP"),
    }
}

fn split_of(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_reference_split_cascade(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn three(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_three_loop_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn valve(tau: Option<f64>) -> BleedLimiter { BleedLimiter::with_tau(PHI, B, tau) }
fn inc(tau: Option<f64>, v_max: f64) -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp(), v_max, SM, tau)
}
fn phi_stator(tau: Option<f64>, v_max: f64) -> StatorLimiter {
    StatorLimiter::from_margin(&lp(), v_max, SM, tau)
}
fn fuel_floor() -> Floor { Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, SM)) }
fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }
fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }
fn ramp_to(hi: f64, ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: hi, r: R, s_settle: SETTLE, ds } }

fn march(m: &ScheduledStatorCore, surge: Option<Floor>, lg: Option<AsymmetricLag>,
         scope: MarchScope) -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge, tt4_max: None };
    m.stator_march_scoped(&flight(), &ramp(DS), None, &leg, &MarchScope { lag: lg, ..scope }).0
}

fn rig() -> TripleRigArm { TripleRigArm { sm: SM, ..TripleRigArm::default() } }

// ------------------------------------------------------------- the dump's `put_point`, mirrored
//
// **THE DUMP ITERATES THE DICT AND EMITS `nkeys`**, rather than walking a typed list. A key the
// port forgets therefore shows up as a COUNT mismatch AND as a golden key the Rust never asked
// for, which is two independent detectors on the same defect.

fn put_point(c: &mut Cmp, p: &str, pt: &FuelPoint) {
    c.d(&format!("{p}/nkeys"), pt.key_count());
    c.f(&format!("{p}/Tt4"), pt.tt4);
    c.s(&format!("{p}/branch"), match pt.branch {
        Branch::Choked => "choked",
        Branch::Subsonic => "subsonic",
    });
    c.f(&format!("{p}/f"), pt.f);
    c.f(&format!("{p}/mdot_air"), pt.mdot_air);
    c.f(&format!("{p}/mf"), pt.mf);
    c.f(&format!("{p}/mf_sched"), pt.mf_sched);
    c.f(&format!("{p}/nu_hp"), pt.nu_hp);
    c.f(&format!("{p}/nu_lp"), pt.nu_lp);
    c.f(&format!("{p}/phi_hp"), pt.phi_hp);
    c.f(&format!("{p}/phi_lp"), pt.phi_lp);
    c.f(&format!("{p}/pi_hpc"), pt.pi_hpc);
    c.f(&format!("{p}/pi_lpc"), pt.pi_lpc);
    c.f(&format!("{p}/s"), pt.s);
    c.f(&format!("{p}/sp_thrust"), pt.sp_thrust);
    match pt.extra {
        PointExtra::None => {}
        PointExtra::Asym { g, required } => {
            c.f(&format!("{p}/g"), g);
            c.f(&format!("{p}/required"), required);
        }
        // SLICE AD: a rung-72 point cannot reach a slice-AB dump. It is REFUSED
        // rather than skipped -- a skipped variant makes a golden agree by recording
        // nothing, which is the one way an oracle can pass while measuring less.
        PointExtra::Shared { .. } => panic!(
            "slice AB's oracle received a rung-72 SHARED-actuator point: this march \n             dispatched to the wrong integrator"),
        PointExtra::Valve { b, b_cmd } => {
            c.f(&format!("{p}/b"), b);
            c.f(&format!("{p}/b_cmd"), b_cmd);
        }
        PointExtra::Cascade { g, required, b, b_cmd, ic_iters, ic_res } => {
            c.f(&format!("{p}/b"), b);
            c.f(&format!("{p}/b_cmd"), b_cmd);
            c.f(&format!("{p}/g"), g);
            c.d(&format!("{p}/ic_iters"), ic_iters);
            c.f(&format!("{p}/ic_res"), ic_res);
            c.f(&format!("{p}/required"), required);
        }
        PointExtra::CrossCascade { g, required, b, b_cmd, ic_iters, ic_res, ic_damp } => {
            c.f(&format!("{p}/b"), b);
            c.f(&format!("{p}/b_cmd"), b_cmd);
            c.f(&format!("{p}/g"), g);
            c.f(&format!("{p}/ic_damp"), ic_damp);
            c.d(&format!("{p}/ic_iters"), ic_iters);
            c.f(&format!("{p}/ic_res"), ic_res);
            c.f(&format!("{p}/required"), required);
        }
        PointExtra::Triple { g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res,
                             ic_order } => {
            c.f(&format!("{p}/b"), b);
            c.f(&format!("{p}/b_cmd"), b_cmd);
            c.f(&format!("{p}/g"), g);
            c.d(&format!("{p}/ic_iters"), ic_iters);
            c.s(&format!("{p}/ic_order"), ic_order);
            c.f(&format!("{p}/ic_res"), ic_res);
            c.f(&format!("{p}/required"), required);
            c.f(&format!("{p}/v"), v);
            c.f(&format!("{p}/v_cmd"), v_cmd);
            c.s(&format!("{p}/v_regime"), regime_name(v_regime));
        }
    }
}

fn regime_name(r: Regime) -> &'static str {
    match r {
        Regime::Dormant => "dormant",
        Regime::Riding => "riding",
        Regime::Saturated => "saturated",
    }
}

fn put_traj(c: &mut Cmp, p: &str, traj: &[FuelPoint], stride: usize) {
    c.d(&format!("{p}/npts"), traj.len());
    for (i, pt) in traj.iter().enumerate() {
        if i % stride == 0 || i == traj.len() - 1 {
            put_point(c, &format!("{p}/{i}"), pt);
        }
    }
}

fn regime_count(traj: &[FuelPoint], want: Regime) -> usize {
    traj.iter().filter(|p| matches!(p.extra, PointExtra::Triple { v_regime, .. }
                                    if v_regime == want)).count()
}

fn v_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Triple { v, .. } => v,
        _ => panic!("this point carries no stator state"),
    }
}

fn put_off(c: &mut Cmp, p: &str, names: &[&'static str]) {
    c.d(&format!("{p}/n_off"), names.len());
    for (i, n) in names.iter().enumerate() {
        c.s(&format!("{p}/off/{i}"), n);
    }
}

/// The dump's `put_gains`, and **its two branches carry DIFFERENT KEY SETS.** Python's
/// non-interior early return has `s` and no gains; its interior return has the gains and NO `s`.
/// The Rust struct carries `s` on both and NaN gains on the off branch, so emitting either
/// unconditionally would invent a golden key on one side and demand a NaN on the other.
fn put_gains(c: &mut Cmp, p: &str, gg: &TripleGains) {
    c.b(&format!("{p}/interior"), gg.interior);
    put_off(c, p, &gg.off_regime);
    c.f(&format!("{p}/v_base"), gg.v_base);
    if gg.interior {
        for (k, v) in [("R_q", gg.r_q), ("R_v", gg.r_v), ("C_g", gg.c_g), ("C_v", gg.c_v),
                       ("V_g", gg.v_g), ("V_q", gg.v_q), ("cyclic", gg.cyclic),
                       ("pair_RC", gg.pair_rc), ("pair_RV", gg.pair_rv),
                       ("pair_CV", gg.pair_cv)] {
            c.f(&format!("{p}/{k}"), v);
        }
    } else {
        c.f(&format!("{p}/s"), gg.s);
    }
}

fn put_bill(c: &mut Cmp, p: &str, bill: &TripleBill) {
    c.f(&format!("{p}/phi_lim"), bill.phi_lim);
    c.f(&format!("{p}/m_lim"), bill.m_lim);
    c.f(&format!("{p}/sum_singles"), bill.sum_singles);
    c.f(&format!("{p}/delivered"), bill.delivered);
    for name in ["bare", "F", "V", "S", "FV", "FS", "VS", "FVS"] {
        let cell = *bill.cell(name);
        for (k, v) in [("I", cell.i), ("I_inc", cell.i_inc), ("min_phi", cell.min_phi),
                       ("end_s", cell.end_s), ("v_min", cell.v_min),
                       ("v_max_used", cell.v_max_used), ("b_max_used", cell.b_max_used),
                       ("credit", cell.credit), ("credit_inc", cell.credit_inc)] {
            c.f(&format!("{p}/{name}/{k}"), v);
        }
        c.d(&format!("{p}/{name}/npts"), cell.npts);
        c.b(&format!("{p}/{name}/v_saturated"), cell.v_saturated);
    }
    for (i, k) in ["fuel", "valve", "stator"].into_iter().enumerate() {
        let pick = |t: (f64, f64, f64)| [t.0, t.1, t.2][i];
        c.f(&format!("{p}/marginal/{k}"), pick(bill.marginal));
        c.f(&format!("{p}/marginal_inc/{k}"), pick(bill.marginal_incidence));
        c.f(&format!("{p}/singles/{k}"), pick(bill.singles));
        c.f(&format!("{p}/erosion/{k}"), pick(bill.erosion));
    }
}

fn put_ring(c: &mut Cmp, p: &str, a: &RingArm) {
    c.d(&format!("{p}/n"), a.n);
    c.d(&format!("{p}/n_riding"), a.n_riding);
    c.d(&format!("{p}/crossings"), a.crossings);
    c.f(&format!("{p}/e0"), a.e0);
    c.opt(&format!("{p}/survives"), a.survives);
    c.opt(&format!("{p}/counter"), a.counter);
    c.f(&format!("{p}/v_lo"), a.v_range.0);
    c.f(&format!("{p}/v_hi"), a.v_range.1);
}

/// The dump's `tuple(sorted(p.items()))` equality, as bits — used only by section A's `identical`
/// flag, which is a statement about two marches of THIS binary and is pinned against Python's own
/// answer to the same question.
fn point_bits(p: &FuelPoint) -> Vec<u64> {
    let mut v = vec![p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.tt4.to_bits(),
                     p.f.to_bits(), p.pi_lpc.to_bits(), p.pi_hpc.to_bits(), p.phi_lp.to_bits(),
                     p.phi_hp.to_bits(), p.mdot_air.to_bits(), p.sp_thrust.to_bits(),
                     p.mf.to_bits(), p.mf_sched.to_bits(),
                     u64::from(p.branch == Branch::Choked)];
    match p.extra {
        PointExtra::None => {}
        PointExtra::Asym { g, required } => v.extend([g.to_bits(), required.to_bits()]),
        PointExtra::Valve { b, b_cmd } => v.extend([b.to_bits(), b_cmd.to_bits()]),
        PointExtra::Cascade { g, required, b, b_cmd, ic_iters, ic_res } =>
            v.extend([g.to_bits(), required.to_bits(), b.to_bits(), b_cmd.to_bits(),
                      ic_iters as u64, ic_res.to_bits()]),
        PointExtra::CrossCascade { g, required, b, b_cmd, ic_iters, ic_res, ic_damp } =>
            v.extend([g.to_bits(), required.to_bits(), b.to_bits(), b_cmd.to_bits(),
                      ic_iters as u64, ic_res.to_bits(), ic_damp.to_bits()]),
        PointExtra::Triple { g, required, b, b_cmd, v: vv, v_cmd, v_regime, ic_iters, ic_res,
                             ic_order } =>
            v.extend([g.to_bits(), required.to_bits(), b.to_bits(), b_cmd.to_bits(),
                      vv.to_bits(), v_cmd.to_bits(), fnv1a(regime_name(v_regime)),
                      ic_iters as u64, ic_res.to_bits(), fnv1a(ic_order)]),
        // SLICE AD: a rung-72 point cannot reach a slice-AB dump. It is REFUSED
        // rather than skipped -- a skipped variant makes a golden agree by recording
        // nothing, which is the one way an oracle can pass while measuring less.
        PointExtra::Shared { .. } => panic!(
            "slice AB's oracle received a rung-72 SHARED-actuator point: this march \n             dispatched to the wrong integrator"),
    }
    v
}

// ------------------------------------------------------------------------------------ the arms
fn run(golden: &str, arm: &str, cpython: bool) {
    let mut c = Cmp::new(load(golden), cpython);
    let fl = flight();

    // ------------------------------------------- A -- THE REDUCE, and BOTH SIDES' ABSOLUTES
    // The suite's reduce gates compare two machines in ONE run, which is blind to a defect that
    // moves both. Each side's own values are pinned here as well.
    let cases: [(LeverArm, Option<Floor>, Option<AsymmetricLag>); 6] = [
        // rung 68 — a `phi` stator on a rung-69 object
        (LeverArm { bleed_lim: Some(valve(Some(TAU))),
                    stator_lim: Some(phi_stator(Some(TAU_S), V_MAX)), ..Default::default() },
         Some(fuel_floor()), Some(lag())),
        (LeverArm { bleed_lim: Some(valve(Some(TAU))), ..Default::default() },      // rung 66
         Some(fuel_floor()), Some(lag())),
        (LeverArm { bleed_lim: Some(valve(Some(TAU))), ..Default::default() },      // rung 65
         Some(fuel_floor()), None),
        (LeverArm::default(), Some(fuel_floor()), Some(lag())),                     // rung 52
        (LeverArm { bleed_lim: Some(valve(None)), ..Default::default() }, None, None), // rung 64
        (LeverArm { bleed_sched: Some(BleedSchedule::new(B, 0.65)), ..Default::default() },
         None, None),                                                               // rung 62
    ];
    for (i, (a, surge, lg)) in cases.into_iter().enumerate() {
        let ta = march(&split_of(&a), surge, lg, MarchScope::DEFAULT);
        let tb = march(&three(&a), surge, lg, MarchScope::DEFAULT);
        put_traj(&mut c, &format!("A/{i}/split"), &ta, 37);
        put_traj(&mut c, &format!("A/{i}/three"), &tb, 37);
        let same = ta.len() == tb.len()
            && ta.iter().zip(&tb).all(|(x, y)| point_bits(x) == point_bits(y));
        c.b(&format!("A/{i}/identical"), same);
        c.b(&format!("A/{i}/carries_v"), matches!(ta[0].extra, PointExtra::Triple { .. }));
    }

    // ------------------------------------------- B -- THE ARMED FIVE-STATE MARCH, and THE BAND
    let m = split_of(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                                 stator_inc: Some(inc(Some(TAU_S), V_MAX)),
                                 ..Default::default() });
    let traj = march(&m, Some(fuel_floor()), Some(lag()), MarchScope::DEFAULT);
    put_traj(&mut c, "B/traj", &traj, 1);
    c.f("B/violation", violation(&traj, PHI, R));
    c.f("B/violation_inc", violation_inc(&traj, m_lim(), t_c(), R));
    c.d("B/n_riding", riding(&traj, B).len());
    c.f("B/v_min", traj.iter().map(v_of).fold(f64::INFINITY, f64::min));
    c.f("B/v_max_seen", traj.iter().map(v_of).fold(f64::NEG_INFINITY, f64::max));
    for (name, r) in [("dormant", Regime::Dormant), ("riding", Regime::Riding),
                      ("saturated", Regime::Saturated)] {
        c.d(&format!("B/regime/{name}"), regime_count(&traj, r));
    }
    let displaced = march(&m, Some(fuel_floor()), Some(lag()),
                          MarchScope { v0: Some(DISP), ..MarchScope::DEFAULT });
    put_traj(&mut c, "B/v0", &displaced, 37);

    let ms = split_of(&LeverArm { stator_inc: Some(inc(Some(TAU_S), 0.02)), ..Default::default() });
    let ts = march(&ms, None, None, MarchScope::DEFAULT);
    put_traj(&mut c, "B/sat", &ts, 17);
    for (name, r) in [("dormant", Regime::Dormant), ("riding", Regime::Riding),
                      ("saturated", Regime::Saturated)] {
        c.d(&format!("B/sat/regime/{name}"), regime_count(&ts, r));
    }

    let sib = m.at_lever(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                                     stator_inc: Some(inc(Some(TAU_S), V_MAX)),
                                     ..Default::default() });
    put_traj(&mut c, "B/at_lever", &march(&sib, Some(fuel_floor()), Some(lag()),
                                          MarchScope::DEFAULT), 37);

    let lim = inc(Some(TAU_S), V_MAX);
    c.f("B/lim/m_lim", lim.m_lim);
    c.f("B/lim/v_max", lim.v_max);
    c.f("B/lim/tau", lim.tau.expect("the shipped limiter is lagged"));
    c.f("B/lim/phi_lim_at", lim.phi_lim_at(&lp()));
    c.f("B/lim/from_phi_m_lim", StatorIncidenceLimiter::from_phi(&lp(), V_MAX, PHI, None).m_lim);
    for (j, (phi, v)) in [(0.80, 0.0), (0.80, 0.20), (0.60, 0.05), (1.20, -0.05)]
        .into_iter().enumerate() {
        c.f(&format!("B/lim/margin/{j}"), StatorIncidenceLimiter::margin(t_c(), phi, v));
    }

    // ------------------------------------------- C -- § 1, THE PAIRWISE SPLIT
    let g = reference_gains(&m, &fl, &ramp(DS), SM, &rig(), 10);
    c.d("C/n_riding", g.n_riding);
    c.d("C/n_sampled", g.n_sampled);
    c.d("C/n_rows", g.rows.len());
    c.d("C/n_skipped", g.skipped.len());
    c.opt("C/worst_RC_inc", g.worst_rc_inc);
    c.opt("C/worst_RC_phi", g.worst_rc_phi);
    c.opt("C/worst_pair_gap", g.worst_pair_gap);
    c.opt("C/worst_RC_own", g.worst_rc_own);
    c.opt("C/k_lo", g.k_range.0);
    c.opt("C/k_hi", g.k_range.1);
    if let Some((lo, hi)) = g.s_window {
        c.f("C/s_lo", lo);
        c.f("C/s_hi", hi);
    }
    for (i, row) in g.rows.iter().enumerate() {
        c.f(&format!("C/{i}/s"), row.s);
        c.f(&format!("C/{i}/k"), row.k);
        c.f(&format!("C/{i}/pair_gap"), row.pair_gap);
        c.f(&format!("C/{i}/v_base"), row.v_base);
        put_gains(&mut c, &format!("C/{i}/inc"), &row.inc);
        put_gains(&mut c, &format!("C/{i}/phi"), &row.phi);
        put_gains(&mut c, &format!("C/{i}/own"), &row.own);
    }
    for (i, (s, off_inc, off_phi)) in g.skipped.iter().enumerate() {
        c.f(&format!("C/skip/{i}/s"), *s);
        put_off(&mut c, &format!("C/skip/{i}/inc"), off_inc);
        put_off(&mut c, &format!("C/skip/{i}/phi"), off_phi);
    }

    // ------------------------------------------- D -- § 1/3, THE SPECTRUM. EVERY ROOT.
    let modes = reference_modes(&m, &fl, &ramp(0.002), SM, &CLOCKS, V_MAX, 3.0, 20);
    c.d("D/n_arms", modes.arms.len());
    c.f("D/ds", modes.ds);
    // The number of root-carrying rows, accumulated for section I's own count check.
    let mut n_cubic = 0usize;
    for (i, clock) in modes.arms.iter().enumerate() {
        for (j, t) in [clock.taus.0, clock.taus.1, clock.taus.2].into_iter().enumerate() {
            c.f(&format!("D/{i}/tau/{j}"), t);
        }
        for (name, x) in clock.refs() {
            let p = format!("D/{i}/{name}");
            c.f(&format!("{p}/rate_sum"), x.rate_sum);
            c.d(&format!("{p}/n"), x.n);
            c.d(&format!("{p}/n_sampled"), x.n_sampled);
            c.d(&format!("{p}/skipped"), x.skipped);
            c.d(&format!("{p}/n_rows"), x.rows.len());
            c.d(&format!("{p}/n_zeros"), x.zeros.len());
            for (j, z) in x.zeros.iter().enumerate() {
                c.d(&format!("{p}/zeros/{j}"), *z);
            }
            c.opt(&format!("{p}/max_c0_rel"), x.max_c0_rel);
            c.opt(&format!("{p}/min_c1_rel"), x.min_c1_rel);
            c.b(&format!("{p}/all_complex?"), x.all_complex.is_some());
            if let Some(ac) = x.all_complex {
                c.b(&format!("{p}/all_complex"), ac);
            }
            c.opt(&format!("{p}/zeta_lo"), x.zeta_range.0);
            c.opt(&format!("{p}/zeta_hi"), x.zeta_range.1);
            n_cubic += x.rows.len();
            for (j, row) in x.rows.iter().enumerate() {
                let q = format!("{p}/{j}");
                for (k, v) in [("s", row.s), ("c1", row.c1), ("c0", row.c0), ("c2", row.c2),
                               ("k", row.k), ("pair_RC", row.pair_rc), ("cyclic", row.cyclic),
                               ("worst_zero", row.worst_zero), ("c1_rel", row.c1_rel),
                               ("c0_rel", row.c0_rel)] {
                    c.f(&format!("{q}/{k}"), v);
                }
                c.d(&format!("{q}/n_zero"), row.n_zero);
                c.b(&format!("{q}/complex_pair"), row.complex_pair);
                c.opt(&format!("{q}/zeta"), row.zeta);
                for (kk, rt) in row.roots.iter().enumerate() {
                    c.f(&format!("{q}/root/{kk}/re"), rt.re);
                    c.f(&format!("{q}/root/{kk}/im"), rt.im);
                    c.f(&format!("{q}/root/{kk}/abs"), rt.abs());
                }
            }
        }
    }

    // ------------------------------------------- E -- § 3, THE DAMPING FLOOR
    let df = damping_floor(&m, &fl, &ramp(DS), SM, &DAMP_GRID, V_MAX, 3.0);
    c.d("E/n_rows", df.rows.len());
    c.b("E/holds", df.holds);
    c.opt("E/worst_pred_err", df.worst_pred_err);
    c.b("E/tightest?", df.tightest.is_some());
    if let Some(t) = df.tightest {
        c.f("E/tightest/s", t.s);
        c.f("E/tightest/zeta", t.zeta);
        c.f("E/tightest/floor", t.floor);
    }
    for (i, row) in df.rows.iter().enumerate() {
        let p = format!("E/{i}");
        for (j, t) in [row.taus.0, row.taus.1, row.taus.2].into_iter().enumerate() {
            c.f(&format!("{p}/tau/{j}"), t);
        }
        c.d(&format!("{p}/n"), row.n);
        c.b(&format!("{p}/live"), row.live.is_some());
        put_off(&mut c, &p, &row.off_regime);
        if let Some(l) = row.live {
            n_cubic += 1;
            for (k, v) in [("s", l.s), ("k", l.k), ("A", l.a), ("z", l.z),
                           ("A_over_z", l.a_over_z), ("det2", l.det2),
                           ("zeta_pred", l.zeta_pred), ("zeta", l.zeta), ("floor", l.floor),
                           ("mod", l.modulus), ("mod_pred", l.mod_pred),
                           ("rate_sum", l.rate_sum)] {
                c.f(&format!("{p}/{k}"), v);
            }
            c.b(&format!("{p}/complex_pair"), l.complex_pair);
        }
    }

    // ------------------------------------------- F -- THE RK4 GUARD, MEASURED
    let rk = rk4_margin(&m, &fl, &ramp(DS), SM, &rig(), 10);
    c.f("F/rate_sum", rk.rate_sum);
    c.d("F/n", rk.n);
    c.d("F/n_rows", rk.rows.len());
    c.opt("F/max_mod", rk.max_mod);
    c.opt("F/max_ratio", rk.max_ratio);
    c.opt("F/max_bound", rk.max_bound);
    c.f("F/ds_lambda", rk.ds_lambda);
    n_cubic += rk.rows.len();
    for (i, row) in rk.rows.iter().enumerate() {
        for (k, v) in [("s", row.s), ("mod", row.modulus), ("k", row.k), ("ratio", row.ratio),
                       ("bound", row.bound)] {
            c.f(&format!("F/{i}/{k}"), v);
        }
    }

    // ------------------------------------------- G -- § 4, THE LEDGER, BOTH REFERENCES
    let bill = reference_bill(&m, &fl, &ramp(DS), SM, &rig());
    for (name, bl, cr, del, del_inc) in
        [("inc", &bill.inc, bill.stator_credit_inc, bill.delivered.0, bill.delivered_inc.0),
         ("phi", &bill.phi, bill.stator_credit_phi, bill.delivered.1, bill.delivered_inc.1)] {
        put_bill(&mut c, &format!("G/{name}"), bl);
        for (k, v) in [("alone", cr.alone), ("alone_inc", cr.alone_inc),
                       ("marginal", cr.marginal), ("marginal_inc", cr.marginal_inc)] {
            c.f(&format!("G/credit/{name}/{k}"), v);
        }
        c.f(&format!("G/delivered/{name}"), del);
        c.f(&format!("G/delivered_inc/{name}"), del_inc);
    }
    c.f("G/common_max_rel", bill.common_max_rel);
    for (name, (a, b)) in &bill.common {
        c.f(&format!("G/common/{name}/inc"), *a);
        c.f(&format!("G/common/{name}/phi"), *b);
    }

    // ------------------------------------------- H -- IS THE MODE OBSERVABLE?
    let rv = ring_visibility(&m, &fl, &ramp(0.002), SM, &rig(), DISP);
    for (name, r) in [("inc", rv.inc), ("phi", rv.phi)] {
        put_ring(&mut c, &format!("H/{name}/base"), &r.base);
        put_ring(&mut c, &format!("H/{name}/displaced"), &r.displaced);
    }

    // ------------------------------------------- I -- THE ROOT FINDER, REPLAYED
    // **THE STEP's REASON FOR EXISTING.** The coefficients are INPUTS read from the golden (see
    // `Cmp::input_f`); the roots are the assertion. The count is not decorative: those three
    // readers are the only callers of `_cubic_roots_c`, so the number Python intercepted must
    // equal the root-carrying rows D, E and F emitted above, which the Rust counted for itself.
    let ncalls = c.input_d("I/ncalls");
    assert_eq!(ncalls, n_cubic,
               "the dump intercepted {ncalls} `_cubic_roots_c` calls and this binary's own D/E/F \
                rows account for {n_cubic}. Those three readers are the ONLY callers, so the two \
                counts are the same number seen from two sides -- a mismatch means a reader \
                skipped or added a row, which no per-row key can show.");
    let mut n_complex = 0usize;
    for i in 0..ncalls {
        let c2 = c.input_f(&format!("I/{i}/c2"));
        let c1 = c.input_f(&format!("I/{i}/c1"));
        let c0 = c.input_f(&format!("I/{i}/c0"));
        let roots = cubic_roots_c(c2, c1, c0);
        if roots.iter().any(|r| r.im != 0.0) {
            n_complex += 1;
        }
        for (j, rt) in roots.iter().enumerate() {
            c.f(&format!("I/{i}/root/{j}/re"), rt.re);
            c.f(&format!("I/{i}/root/{j}/im"), rt.im);
            c.f(&format!("I/{i}/root/{j}/abs"), rt.abs());
        }
    }
    c.d("I/n_complex", n_complex);

    // ------------------------------------------- J -- THE ROOT FINDER, DECLARED EXTRA TABLE
    // NOT the suite's grid, and said so in the dump. Row 0 is § 5.26 (iii)'s EXHAUSTED shape --
    // 80 Newton steps that never converge, whose exit value is an arbitrary point of a chaotic
    // march and which the port owes bit-for-bit.
    const CUBICS: [(f64, f64, f64); 9] = [
        (-60.0, 9.7e-08, -1.7e-12), (-60.0, 1.0e-08, -1.0e-14), (-60.0, 0.0, 0.0),
        (0.0, 1.0, -1.0), (-3.0, 3.0, -1.0), (-2.0, 5.0, -10.0), (-6.0, 11.0, -6.0),
        (1.0, -1.0, 1.0), (-240.0, 1.0e-04, -1.0e-09)];
    for (i, (c2, c1, c0)) in CUBICS.into_iter().enumerate() {
        c.f(&format!("J/{i}/c2"), c2);
        c.f(&format!("J/{i}/c1"), c1);
        c.f(&format!("J/{i}/c0"), c0);
        for (j, rt) in cubic_roots_c(c2, c1, c0).iter().enumerate() {
            c.f(&format!("J/{i}/root/{j}/re"), rt.re);
            c.f(&format!("J/{i}/root/{j}/im"), rt.im);
            c.f(&format!("J/{i}/root/{j}/abs"), rt.abs());
        }
    }

    // ------------------------------------------- K -- THE DEGENERATE BRANCHES, EXTRA GRID
    // Step 2 § (h) disclosed four branches nothing reached and step 3 left them a standing hole.
    // THREE of the four are reached here, and it is the RAMP that reaches them and not the clocks.
    let flat = Ramp { tt4_lo: LO, tt4_hi: LO, r: R, s_settle: SETTLE, ds: DS };
    let kd0 = damping_floor(&m, &fl, &flat, SM, &[(0.05, 0.05, 0.05)], V_MAX, 3.0);
    c.d("K/0/n_rows", kd0.rows.len());
    c.d("K/0/n", kd0.rows[0].n);
    c.b("K/0/live", kd0.rows[0].live.is_some());
    c.b("K/0/holds", kd0.holds);
    c.b("K/0/tightest?", kd0.tightest.is_some());
    c.b("K/0/worst_pred_err?", kd0.worst_pred_err.is_some());
    let flat2 = Ramp { tt4_lo: LO, tt4_hi: LO, r: R, s_settle: SETTLE, ds: 0.002 };
    let km0 = reference_modes(&m, &fl, &flat2, SM, &[(0.05, 0.05, 0.05)], V_MAX, 3.0, 20);
    for (name, x) in km0.arms[0].refs() {
        c.d(&format!("K/0/{name}/n"), x.n);
        c.d(&format!("K/0/{name}/n_rows"), x.rows.len());
        c.b(&format!("K/0/{name}/all_complex?"), x.all_complex.is_some());
        c.b(&format!("K/0/{name}/zeta_lo?"), x.zeta_range.0.is_some());
        c.b(&format!("K/0/{name}/max_c0_rel?"), x.max_c0_rel.is_some());
    }
    let kd1 = damping_floor(&m, &fl, &ramp_to(1010.0, DS), SM, &[(0.005, 0.05, 0.05)],
                            V_MAX, 3.0);
    c.d("K/1/n", kd1.rows[0].n);
    c.b("K/1/live", kd1.rows[0].live.is_some());
    put_off(&mut c, "K/1", &kd1.rows[0].off_regime);
    c.b("K/1/holds", kd1.holds);
    c.b("K/1/tightest?", kd1.tightest.is_some());

    c.finish(arm);
}

#[test]
fn rung69_is_bit_exact_against_pypy() {
    run(ORACLE_PYPY, "pypy", false);
}

/// **THE ONE PORT DEFECT THIS ORACLE FOUND, GIVEN A GATE THAT NAMES IT.**
///
/// Section J's `(-2, 5, -10)` row disagreed with PyPy on ONE key of 15 957: the third root's real
/// part, `-0.0` in Rust against `+0.0` in Python. `0.5 * z` in Python is `complex.__rmul__`, a
/// four-multiply complex product whose cross term `0.0 * z.im` flips the sign of a zero real part;
/// the port scaled the two floats. Step 2 § (e) had spelled the sign-of-zero decision out for the
/// ADDITION in the same expression and stopped there.
///
/// The golden already carries it, so why a second gate: the golden says *"key `J/5/root/2/re`
/// differs"* and this says *why*, and it keeps working if the dump is ever regenerated on a build
/// carrying the defect. **The second assertion is what makes the first non-vacuous** — it measures
/// that the naive spelling really does land on the other sign, so a passing first line is a
/// statement about [`py_half`](turbojet::reference_split) and not about zero being unsigned.
#[test]
fn the_half_is_a_complex_product_and_a_signed_zero_can_tell() {
    let z = cubic_roots_c(-2.0, 5.0, -10.0)[2];
    let naive = 0.5 * -0.0_f64;
    assert_eq!(naive.to_bits(), (-0.0_f64).to_bits(),
               "the naive spelling must reach -0.0, or the line below proves nothing");
    assert_eq!(z.re.to_bits(), 0.0_f64.to_bits(),
               "Python's `0.5 * z` is a complex product: re = 0.5*z.re - 0.0*z.im, and on this \
                triple the deflated pair is -0 +/- 4.472j, so the real part is (-0.0) - (-0.0) = \
                +0.0. Got {:.17e} ({:016x}).", z.re, z.re.to_bits());
}

/// The CPython arm. **The exemption is a named list rooted in ONE three-element `sum()`** — read
/// the header before touching it, and note the assertion runs in BOTH directions.
#[test]
fn rung69_against_cpython_with_the_declared_exemption() {
    run(ORACLE_CPYTHON, "cpython", true);
}
