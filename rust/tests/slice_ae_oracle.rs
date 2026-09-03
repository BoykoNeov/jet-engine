//! SLICE AE step 4 — **THE ORACLE for rung 73**, against PyPy *and* CPython 3.14.
//!
//! `rung73.rs` ports the suite's own 27 gates. Step 3 measured what they can see, and the answer
//! was not the pre-flight's: **P5 was FALSIFIED at 6 of 27** — folding path 2's float-identity
//! branch away is caught by the ported gates, and by **the same 6 in Python, name for name**,
//! because four of the ported bars are EXACT EQUALITIES rather than tolerances. So this file does
//! **not** exist to be the only seat for that injection, which is what AB/AC/AD's oracles were
//! written to be. It exists for the three things a 27-gate suite of RELATIONS still cannot do:
//!
//! 1. **Say how much moved.** A gate answers caught / not caught. Step 2 scored five mutations on
//!    a 5 066-key value seat and step 3 scored them on a gate seat; this file is the value seat
//!    made SHIPPED and made ten times wider, so the ledger survives the session that measured it.
//! 2. **Reach the plant per point.** Sections A–H are the readers' AGGREGATES over a march, and
//!    the suite's own reduce spine compares NINE of the march's THIRTY recorded fields. **P3's
//!    clause (b) — CPython drifting on march values, first in the stator state `v`, at 1–11 ULPs
//!    — is unfalsifiable without section J**, and AD step 5 found its six drifting keys at 2
//!    points of 1 302 only because the equivalent section existed.
//! 3. **Attribute a cross-interpreter difference to the operation that caused it.** Section L
//!    replays `_charpoly4` on the SAME 4×4 matrix and `_quartic_roots_c` on the SAME coefficients,
//!    so a coefficient that differs is `sum()`'s and a root that differs is the root finder's. AD
//!    step 5 could only reach that conclusion by a cross-feed run after the fact, having already
//!    shipped a 5 022-name exemption blaming the wrong stage.
//!
//! # THE GRID IS THE READERS' OWN, AND IT IS FIVE DIFFERENT GRIDS
//!
//! Read off `turbojet/engine.py`'s `def` lines, copied here verbatim from the dumper's header
//! rather than re-derived — transcribing one reader's stride into another moves every number in
//! its section without failing anything, which is [[rust-port-slice-ac-step6]]'s `every = 40`-vs-
//! `10` defect and which this slice already had to avoid once at step 3 § (h):
//!
//! ```text
//! reader              ds       every   clock argument
//! handover_law        0.005    --      `clocks`, the THREE-arm tuple
//! applied_gains       0.002    2       `taus`, one 4-tuple   + an `inc` arm
//! applied_cells       0.002    2       `clocks`, the THREE-arm tuple
//! ref_discriminator   0.002    4       `taus`                + an `inc` arm
//! applied_bill        0.005    --      `taus`                + an `inc` arm
//! ```
//!
//! **Exactly one call in `tests/test_rung73.py` overrides a default** — the broken-instrument
//! probe's `ds = 0.01, every = 8`. It is a DELIBERATELY BROKEN reader and is not a grid this
//! oracle copies; `rung73.rs` gate 24 drives it instead.
//!
//! # THE FOUR STRUCTURAL PROPERTIES THIS FILE IS BUILT AROUND
//!
//! **1. AN AGGREGATE IS LOSSY, SO SECTION J EMITS THE MARCH ITSELF** — every field of every fifth
//! point of every distinct march the readers drive, plus the `min`, `max` and LAST of every float
//! column over ALL points. **That backstop is real and it is weaker than "nothing can hide"**, and
//! this file carries AD's *correction* rather than AD's claim: AD step 5's close-out measured a
//! hidden-point defect moving **0 of 54 116** keys (index 137, against a control at 135 that moved
//! 10). So what section J pins is one point in five plus both extremes and the endpoint — no more.
//!
//! **2. A CELL CAN BREAK BY EMPTYING THE SAMPLE.** § 5.27 (ii) ran a parent's body in a child's
//! slot and the reader **returned successfully with an EMPTY table** — every aggregate `None`, no
//! value differing because there were no values. So every sample-shaped reading emits its ROW
//! COUNT and every `Option` carries a PRESENCE FLAG beside it ([`Cmp::opt`]).
//!
//! **3. A REGIME IS THE ONE THING NO FLOAT WITNESSES.** Every authority label, nozzle branch,
//! stator regime, `ic_order`, `share_law`, ledger cell name and cell parent is emitted as an
//! FNV-1a hash through [`Cmp::s`], and a discrete key that flips between interpreters is a hard
//! failure and never a rounding.
//!
//! **4. A COUNT IS ONLY EVIDENCE IF SOMETHING RE-DERIVES IT.** Sections J, K and L emit their own
//! census. Wherever the Rust can recompute one from its own values it does, and asserts it — the
//! per-path split of section K's replay set is re-derived here from `applied_clip`, not trusted.
//! Where it cannot (a call count over Python's own run) the key is read as an INPUT and tied into
//! a consistency web that must hold. [[rust-port-guessed-census-bars]].
//!
//! # WHAT THIS ORACLE CANNOT SEE, stated so a silent absence does not read as coverage
//!
//! * **`at_lever`.** It is a BUILDER and slice AC step 7 measured it as the LAUNDERER: every
//!   reader rebuilds its machine through it and installs the shipped tables, so no value key can
//!   witness which function pointer sat in a slot. That is step 5's subject.
//! * **`_quad_gains_at`'s POINTER.** § 5.29 (iv) measured its cell observable — 32 keys move and
//!   70 vanish under the parent's body — and P4 assigns that seat to **step 5**, on a DECLARED
//!   EXTRA GRID no shipped test sits in. Its VALUES are covered here: every gains row in B, C, E
//!   and F comes out of it. No pointer swap appears in this file.
//! * **The raises.** Rung 73's two `integrate_fuel` refusals and `_rk4_floor_shared`'s
//!   `ds * rate <= 2.0` are CONTROL FLOW, not values; nothing below passes a `ds` that trips the
//!   floor. `rung73.rs` gates them by their RUNG TAG, which § 5.29 (vii) measured to be the only
//!   discriminating token — the shipped Python needles `"no set point"` and `"FORCED release"`
//!   match rungs 73 does not own, the second reaching NINE classes back to rung 43.
//! * **`shared_bill`'s `own_currency`** — a table of CONSTANT strings that cannot differ between
//!   two runs. AD dropped it for that reason and so does this file. A DECISION, not an omission.
//! * **`_quad_gains_at`'s `s` key.** Python's INTERIOR return carries none and its two
//!   non-interior early returns do; all 540 calls on this grid return the interior dict, so `s` is
//!   absent from every one. The port's `QuadGains` carries it unconditionally — a REPRESENTATION
//!   difference, not a value one. `interior` IS a key, so the premise is itself gated.
//!
//! # THE CROSS-INTERPRETER EXEMPTION — a set of NAMES, measured against the PORT
//!
//! [`EXEMPT`] is read off **Rust-vs-CPython**, never off the diff between the two goldens. AB step
//! 4 measured an exemption taken between the dumps at **67 names wider** than the one taken
//! against the port, and AD step 5 shipped one blaming the root finder for 5 022 golden
//! differences whose cause was upstream. A tolerance is never an acceptable substitute: the set is
//! asserted for EQUALITY, so a key that STOPS drifting fails this file too.
//!
//! Regenerate BOTH arms (through a POSIX shell — PowerShell 5.1 writes a UTF-8 BOM that lands in
//! front of the `#` on line 1, so the header parses as data):
//!
//! ```text
//! .venv/Scripts/python.exe rust/oracle/dump_slice_ae.py > rust/oracle/slice_ae_pypy.tsv
//! C:/Python314/python.exe  rust/oracle/dump_slice_ae.py > rust/oracle/slice_ae_cpython.tsv
//! ```

use std::collections::{BTreeMap, BTreeSet};

use turbojet::applied_reference::{
    applied_bill, applied_cells, applied_gains, build_applied_reference_cascade, handover_law,
    ref_discriminator, AppliedGains, QuadGains, RefDiscriminator, R73_TRIPLE,
};
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{Authority, FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::reference_split::RefScope;
use turbojet::shared_actuator::{
    applied_clip, charpoly4, quartic_roots_c, shared_march, SharedBill, ShareScope,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_ae_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_ae_cpython.tsv");

/// The number of keys `dump_slice_ae.py` emits — **its own stderr tally, not an estimate and not a
/// neighbouring slice's**. [`load`] bars 95 % of it, so a golden truncated mid-write cannot present
/// as a pass — and this step MET that hazard: a waiter watching for a non-empty file fired on a
/// partial buffer flush at 7 310 of the lines, which read exactly like a finished small dump.
/// [[rust-port-guessed-census-bars]], [[windows-tooling-file-hazards]].
const GOLDEN_KEYS: usize = 76_770;

/// **THE CPYTHON EXEMPTION — measured against the PORT, name by name.** See the header.
const EXEMPT: [&str; 0] = [];

// ============================================================================ the suite's constants

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
    eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
};
const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const V_MAX: f64 = 0.20;
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TT4_MAX: f64 = 1200.0;
const CLOCKS: [(f64, f64, f64, f64); 3] = [
    (0.05, 0.05, 0.05, 0.05), (0.20, 0.01, 0.50, 0.05), (0.20, 0.005, 0.80, 0.05),
];

/// **THE FIVE READERS' OWN DEFAULTS, one constant each and never shared.** Spelled apart even
/// where two agree, because a shared constant is how one reader's stride gets transcribed into
/// another's row.
const HL_DS: f64 = 0.005;
const AG_DS: f64 = 0.002;
const AG_EVERY: usize = 2;
const AC_DS: f64 = 0.002;
const AC_EVERY: usize = 2;
const RD_DS: f64 = 0.002;
const RD_EVERY: usize = 4;
const AB_DS: f64 = 0.005;

/// Section J's stride — coprime to BOTH reader samplings, gated below.
const J_STRIDE: usize = 5;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, gamma_t: 1.3, cp_t: 1239.0, h_pr: 42.8e6, ..GasSpec::default()
    })
}

fn lp() -> ComponentMap {
    ComponentMap::new(0.20, 0.05, 0.1, 0.7).with_phi_surge(FLOOR)
}

fn hp() -> ComponentMap {
    ComponentMap::new(0.08, 0.15, 0.1, 1.0).with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

/// Python's `_applied(design, bleed_lim=_valve())`. **A FRESH machine per reader**, which is what
/// the dumper does and what the suite does; AD step 5 measured that the plant carries no state
/// between reader calls, and reusing one here would turn that measurement back into an assumption.
fn machine() -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp(), B, SM, Some(TAU))),
        ..LeverArm::default()
    };
    match build_applied_reference_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0,
                                          &arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

// ============================================================================== the comparator

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, bits) = (it.next().expect("key"), it.next().expect("bits"));
        let v = bits.parse::<u64>().unwrap_or_else(|e| panic!(
            "slice-AE golden line is not `key<TAB>u64` ({e}): {line:?}. A LEADING MINUS means the \
             dumper emitted a signed integer raw instead of masking it to two's complement — \
             `dzeros_B`/`dzeros_C` are DIFFERENCES and reach -1 on this grid. If the second field \
             has text appended, the dump was redirected with `2>&1`. If the FIRST line failed, the \
             file has a UTF-8 BOM: it was redirected through PowerShell, which writes one."));
        assert!(m.insert(k.to_string(), v).is_none(), "dup {k}");
    }
    assert!(m.len() > GOLDEN_KEYS - GOLDEN_KEYS / 20,
            "the slice-AE golden did not parse ({} keys, expected about {GOLDEN_KEYS})", m.len());
    m
}

fn fnv1a(text: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for byte in text.as_bytes() {
        h = (h ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

fn regime_str(r: Regime) -> &'static str {
    match r {
        Regime::Dormant => "dormant",
        Regime::Riding => "riding",
        Regime::Saturated => "saturated",
    }
}

/// Accumulates every disagreement so ONE run reports them all, **and reports every golden key the
/// Rust never asked for** — a field missing from the port is invisible until that half fires.
struct Cmp {
    py: BTreeMap<String, u64>,
    seen: BTreeSet<String>,
    /// **THE RUST's OWN VALUES, kept rather than discarded after each comparison.** Section M is a
    /// census OVER THE VALUES, and a census computed from the golden would be the golden agreeing
    /// with itself — slice U step 4's gate-that-cannot-see-its-own-value, one file on.
    mine: BTreeMap<String, u64>,
    bad: Vec<String>,
    cpython: bool,
    exempted: BTreeSet<String>,
    drifts: Vec<(String, f64)>,
    flips: Vec<String>,
}

impl Cmp {
    fn new(py: BTreeMap<String, u64>, cpython: bool) -> Self {
        Cmp { py, seen: BTreeSet::new(), mine: BTreeMap::new(), bad: Vec::new(), cpython,
              exempted: BTreeSet::new(), drifts: Vec::new(), flips: Vec::new() }
    }

    fn f(&mut self, key: &str, got: f64) {
        assert!(got.is_finite(), "{key} is not finite: {got}");
        self.raw(key, got.to_bits(), false);
    }

    fn d(&mut self, key: &str, got: usize) { self.raw(key, got as u64, true); }

    /// A SIGNED integer, as two's complement — the dumper masks, and `dzeros_C` is `-1` on both
    /// `inc` arms of this grid.
    fn i(&mut self, key: &str, got: i64) { self.raw(key, got as u64, true); }

    fn b(&mut self, key: &str, got: bool) { self.raw(key, got as u64, true); }

    fn s(&mut self, key: &str, got: &str) { self.raw(key, fnv1a(got), true); }

    /// **THE PRESENCE FLAG IS THE POINT, NOT THE VALUE** — header property 2.
    fn opt(&mut self, key: &str, got: Option<f64>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some(x) = got { self.f(key, x); }
    }

    fn opt_s(&mut self, key: &str, got: Option<&str>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some(x) = got { self.s(key, x); }
    }

    fn opt_span(&mut self, key: &str, got: Option<(f64, f64)>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some((lo, hi)) = got {
            self.f(&format!("{key}/lo"), lo);
            self.f(&format!("{key}/hi"), hi);
        }
    }

    fn flist(&mut self, key: &str, xs: &[f64]) {
        self.d(&format!("{key}/n"), xs.len());
        for (i, x) in xs.iter().enumerate() { self.f(&format!("{key}/{i}"), *x); }
    }

    fn dlist(&mut self, key: &str, xs: &[usize]) {
        self.d(&format!("{key}/n"), xs.len());
        for (i, x) in xs.iter().enumerate() { self.d(&format!("{key}/{i}"), *x); }
    }

    fn ilist(&mut self, key: &str, xs: &[i64]) {
        self.d(&format!("{key}/n"), xs.len());
        for (i, x) in xs.iter().enumerate() { self.i(&format!("{key}/{i}"), *x); }
    }

    fn taus4(&mut self, key: &str, t: (f64, f64, f64, f64)) {
        self.f(&format!("{key}/0"), t.0);
        self.f(&format!("{key}/1"), t.1);
        self.f(&format!("{key}/2"), t.2);
        self.f(&format!("{key}/3"), t.3);
    }

    /// **A GOLDEN KEY READ AS AN INPUT AND NOT COMPARED.** Sections K and L replay shipped
    /// functions on arguments Python's interceptors captured; re-emitting those arguments as
    /// assertions would compare a key with itself, which slice U step 4 recorded as a gate that
    /// cannot see its own value. Marking them `seen` keeps [`Cmp::finish`]'s missing-key half
    /// honest without pretending they were checked.
    fn input_f(&mut self, key: &str) -> f64 {
        assert!(self.seen.insert(key.to_string()), "the Rust read {key} twice");
        f64::from_bits(*self.py.get(key)
            .unwrap_or_else(|| panic!("{key}: NO GOLDEN — a declared-grid input is missing")))
    }

    fn input_d(&mut self, key: &str) -> usize {
        assert!(self.seen.insert(key.to_string()), "the Rust read {key} twice");
        *self.py.get(key)
            .unwrap_or_else(|| panic!("{key}: NO GOLDEN — a declared-grid input is missing"))
            as usize
    }

    fn input_s(&mut self, key: &str) -> &'static str {
        let h = self.input_d(key) as u64;
        for cand in ["max", "sum", "sched", "applied"] {
            if fnv1a(cand) == h { return cand; }
        }
        panic!("{key}: hash {h} is none of the four law strings this dump can emit")
    }

    fn raw(&mut self, key: &str, got: u64, discrete: bool) {
        assert!(self.seen.insert(key.to_string()), "the Rust emitted {key} twice");
        self.mine.insert(key.to_string(), got);
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
                "{} DISCRETE keys flipped between interpreters — a flipped count, flag, authority \
                 label, nozzle branch, stator regime, cell parent or ledger cell name is a \
                 different physical reading, never a rounding:\n  {}",
                self.flips.len(),
                self.flips.iter().take(12).cloned().collect::<Vec<_>>().join("\n  "));
        let worst = self.drifts.iter().map(|(_, r)| *r).fold(0.0_f64, f64::max);
        assert!(self.drifts.is_empty(),
                "{} float keys drifted against CPython OUTSIDE the declared exemption (worst \
                 {worst:.3e}). The exemption is a NAMED LIST measured against the PORT and \
                 pre-registered as P3 (§ 5.29 (ix)) — read this file's header before widening it, \
                 and never replace it with a tolerance:\n  {}",
                self.drifts.len(),
                self.drifts.iter().take(12).map(|(l, r)| format!("{l} (rel {r:.3e})"))
                    .collect::<Vec<_>>().join("\n  "));
        if self.cpython {
            let want: BTreeSet<String> = EXEMPT.iter().map(|s| (*s).to_string()).collect();
            assert_eq!(self.exempted, want,
                       "the CPython exemption set MOVED. Expected exactly the names in `EXEMPT`; \
                        got {} names. A key that STOPPED drifting is a change too — it would mean \
                        the port, the dump or CPython's `sum()` moved.\n\
                        only-in-EXEMPT: {:?}\nonly-measured: {:?}",
                       self.exempted.len(),
                       want.difference(&self.exempted).take(20).collect::<Vec<_>>(),
                       self.exempted.difference(&want).take(20).collect::<Vec<_>>());
        }
        if self.bad.is_empty() && missed.is_empty() {
            println!("slice_ae_oracle ({arm}): {} values compared, {} exempt",
                     self.seen.len(), self.exempted.len());
            return;
        }
        panic!(
            "{} of {} compared keys differ:\n  {}\n\n{} golden keys the Rust never asked for (a \
             field missing from the port is invisible until this fires):\n  {:?}",
            self.bad.len(), self.seen.len(),
            self.bad.iter().take(20).cloned().collect::<Vec<_>>().join("\n  "),
            missed.len(), missed.iter().take(20).collect::<Vec<_>>());
    }
}

// ============================================================================== the march point

/// The march point's THIRTY fields, split by type exactly as `dump_slice_ae.py` splits them.
/// Spelled out on both sides rather than looped generically, because a generic loop on one side
/// and a hand-written list on the other is the pair that silently drifts — and the dumper ASSERTS
/// its copy of this list against a live point, so neither side can drift from the plant either.
fn emit_point(cmp: &mut Cmp, q: &str, p: &FuelPoint) {
    let (g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res, ic_order, g_fuel, g_gov,
         required_fuel, required_gov, auth, share_law) = match p.extra {
        PointExtra::Shared { g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res,
                             ic_order, g_fuel, g_gov, required_fuel, required_gov, authority,
                             share_law } =>
            (g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res, ic_order, g_fuel, g_gov,
             required_fuel, required_gov, authority, share_law),
        _ => panic!("{q}: section J marches the SIX-STATE integrator; this point is not `Shared`, \
                     which means the dispatch left through an inherited marcher"),
    };
    for (name, x) in [
        ("s", p.s), ("nu_lp", p.nu_lp), ("nu_hp", p.nu_hp), ("Tt4", p.tt4), ("f", p.f),
        ("pi_lpc", p.pi_lpc), ("pi_hpc", p.pi_hpc), ("phi_lp", p.phi_lp), ("phi_hp", p.phi_hp),
        ("mdot_air", p.mdot_air), ("sp_thrust", p.sp_thrust), ("mf", p.mf),
        ("mf_sched", p.mf_sched), ("g", g), ("required", required), ("g_fuel", g_fuel),
        ("g_gov", g_gov), ("required_fuel", required_fuel), ("required_gov", required_gov),
        ("b", b), ("b_cmd", b_cmd), ("v", v), ("v_cmd", v_cmd), ("ic_res", ic_res),
    ] {
        cmp.f(&format!("{q}/{name}"), x);
    }
    cmp.s(&format!("{q}/branch"), p.branch.label());
    cmp.s(&format!("{q}/authority"), auth.as_str());
    cmp.s(&format!("{q}/ic_order"), ic_order);
    cmp.s(&format!("{q}/share_law"), share_law);
    cmp.opt_s(&format!("{q}/v_regime"), v_regime.map(regime_str));
    cmp.d(&format!("{q}/ic_iters"), ic_iters);
}

fn point_float(p: &FuelPoint, i: usize) -> f64 {
    let (g, required, b, b_cmd, v, v_cmd, ic_res, g_fuel, g_gov, required_fuel, required_gov) =
        match p.extra {
            PointExtra::Shared { g, required, b, b_cmd, v, v_cmd, ic_res, g_fuel, g_gov,
                                 required_fuel, required_gov, .. } =>
                (g, required, b, b_cmd, v, v_cmd, ic_res, g_fuel, g_gov, required_fuel,
                 required_gov),
            _ => panic!("section J's aggregates need a six-state point"),
        };
    [p.s, p.nu_lp, p.nu_hp, p.tt4, p.f, p.pi_lpc, p.pi_hpc, p.phi_lp, p.phi_hp, p.mdot_air,
     p.sp_thrust, p.mf, p.mf_sched, g, required, g_fuel, g_gov, required_fuel, required_gov, b,
     b_cmd, v, v_cmd, ic_res][i]
}

/// `PT_FLOAT`'s names, in the dumper's own order.
const PT_FLOAT: [&str; 24] = [
    "s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp", "mdot_air",
    "sp_thrust", "mf", "mf_sched", "g", "required", "g_fuel", "g_gov", "required_fuel",
    "required_gov", "b", "b_cmd", "v", "v_cmd", "ic_res",
];
const PT_STR: [&str; 4] = ["branch", "authority", "ic_order", "share_law"];
const PT_OPT_STR: [&str; 1] = ["v_regime"];
const PT_INT: [&str; 1] = ["ic_iters"];

fn emit_gains(cmp: &mut Cmp, p: &str, g: &QuadGains) {
    cmp.b(&format!("{p}/interior"), g.interior);
    cmp.b(&format!("{p}/near_switch"), g.near_switch);
    cmp.d(&format!("{p}/off_regime/n"), g.off_regime.len());
    for (i, r) in g.off_regime.iter().enumerate() {
        cmp.s(&format!("{p}/off_regime/{i}"), r);
    }
    cmp.f(&format!("{p}/v_base"), g.v_base);
    for (name, x) in [
        ("F_f", g.f_f), ("F_r", g.f_r), ("F_q", g.f_q), ("F_v", g.f_v),
        ("R_f", g.r_f), ("R_r", g.r_r), ("R_q", g.r_q), ("R_v", g.r_v),
        ("C_f", g.c_f), ("C_r", g.c_r), ("C_v", g.c_v),
        ("V_f", g.v_f), ("V_r", g.v_r), ("V_q", g.v_q),
        ("pair_FR", g.pair_fr), ("pair_RC", g.pair_rc), ("pair_CV", g.pair_cv),
        ("pair_RV", g.pair_rv),
    ] {
        cmp.f(&format!("{p}/{name}"), x);
    }
    cmp.opt_s(&format!("{p}/authority"), g.authority.map(|a| a.as_str()));
    cmp.opt_s(&format!("{p}/masked"), g.masked.map(|a| a.as_str()));
    cmp.opt(&format!("{p}/mask_leak"), g.mask_leak);
    cmp.opt(&format!("{p}/self_masked"), g.self_masked);
    cmp.opt(&format!("{p}/cross_masked"), g.cross_masked);
    cmp.opt(&format!("{p}/self_live"), g.self_live);
}

fn emit_shared_bill(cmp: &mut Cmp, kk: &str, sb: &SharedBill) {
    cmp.b(&format!("{kk}/inc"), sb.inc);
    cmp.taus4(&format!("{kk}/taus"), sb.taus);
    cmp.f(&format!("{kk}/Tt4_full"), sb.tt4_full);
    cmp.f(&format!("{kk}/Tt4_no_fuel"), sb.tt4_no_fuel);
    cmp.f(&format!("{kk}/phi_full"), sb.phi_full);
    cmp.f(&format!("{kk}/phi_no_fuel"), sb.phi_no_fuel);
    cmp.f(&format!("{kk}/fuel_marginal_phi"), sb.fuel_marginal_phi);
    cmp.f(&format!("{kk}/fuel_marginal_Tt4"), sb.fuel_marginal_tt4);
    cmp.opt(&format!("{kk}/handover"), sb.handover);
    cmp.opt(&format!("{kk}/delivered/phi"), sb.delivered_phi);
    cmp.opt(&format!("{kk}/delivered/Tt4"), sb.delivered_tt4);
    cmp.opt(&format!("{kk}/delivered/inc"), sb.delivered_inc);
    for leg in ["F", "G", "V", "S"] {
        let m = sb.marginal.iter().find(|(n, _)| *n == leg).expect("leg").1;
        let a = sb.alone.iter().find(|(n, _)| *n == leg).expect("leg").1;
        let k = sb.kept.iter().find(|(n, _)| *n == leg).expect("leg").1;
        cmp.f(&format!("{kk}/marginal/{leg}"), m);
        cmp.f(&format!("{kk}/alone/{leg}"), a);
        cmp.opt(&format!("{kk}/kept/{leg}"), k);
    }
    cmp.d(&format!("{kk}/n_cells"), sb.cells.len());
    // Python iterates `sorted(s["cells"])` — LEXICOGRAPHIC over the cell names, not the bit order
    // `BILL_KEYS` is tabulated in. Sorting here rather than trusting the Vec's order is the whole
    // difference between comparing the same cell and comparing two different ones.
    let mut names: Vec<&str> = sb.cells.iter().map(|(n, _)| *n).collect();
    names.sort_unstable();
    for name in names {
        let c = &sb.cells.iter().find(|(n, _)| *n == name).expect("cell").1;
        let k3 = format!("{kk}/cell/{name}");
        for (leg, on) in [("F", c.on.0), ("G", c.on.1), ("V", c.on.2), ("S", c.on.3)] {
            cmp.b(&format!("{k3}/on/{leg}"), on);
        }
        cmp.f(&format!("{k3}/I"), c.i);
        cmp.f(&format!("{k3}/E"), c.e);
        cmp.f(&format!("{k3}/M"), c.m);
        cmp.f(&format!("{k3}/min_phi"), c.min_phi);
        cmp.f(&format!("{k3}/max_Tt4"), c.max_tt4);
        cmp.d(&format!("{k3}/n"), c.n);
        cmp.d(&format!("{k3}/auth_fuel"), c.auth_fuel);
        cmp.opt(&format!("{k3}/handover"), c.handover);
        cmp.opt(&format!("{k3}/credit_phi"), c.credit_phi);
        cmp.opt(&format!("{k3}/credit_Tt4"), c.credit_tt4);
        cmp.opt(&format!("{k3}/credit_inc"), c.credit_inc);
    }
}

fn emit_gain_arm(cmp: &mut Cmp, tag: &str, g: &AppliedGains) {
    cmp.b(&format!("{tag}/inc"), g.inc);
    cmp.taus4(&format!("{tag}/taus"), g.taus);
    cmp.f(&format!("{tag}/ds"), g.ds);
    cmp.d(&format!("{tag}/n_riding"), g.n_riding);
    cmp.d(&format!("{tag}/n_sampled"), g.n_sampled);
    cmp.d(&format!("{tag}/skipped/switch"), g.skipped_switch);
    cmp.d(&format!("{tag}/skipped/regime"), g.skipped_regime);
    cmp.d(&format!("{tag}/by_authority/fuel"), g.by_authority_fuel);
    cmp.d(&format!("{tag}/by_authority/gov"), g.by_authority_gov);
    cmp.flist(&format!("{tag}/self_masked"), &g.self_masked);
    cmp.flist(&format!("{tag}/cross_masked"), &g.cross_masked);
    cmp.flist(&format!("{tag}/self_live"), &g.self_live);
    cmp.flist(&format!("{tag}/moved_scaled"), &g.moved_scaled);
    cmp.opt(&format!("{tag}/worst_mask_leak"), g.worst_mask_leak);
    cmp.opt(&format!("{tag}/worst_delta_rest"), g.worst_delta_rest);
    cmp.opt(&format!("{tag}/min_live_gain"), g.min_live_gain);
    cmp.opt_span(&format!("{tag}/det_range"), g.det_range);
    cmp.d(&format!("{tag}/n_boundary"), g.boundary.len());
    for (i, x) in g.boundary.iter().enumerate() {
        let k = format!("{tag}/bnd/{i}");
        cmp.f(&format!("{k}/s"), x.s);
        cmp.f(&format!("{k}/live/F_q"), x.live_f_q);
        cmp.f(&format!("{k}/live/F_v"), x.live_f_v);
        cmp.f(&format!("{k}/live/R_q"), x.live_r_q);
        cmp.f(&format!("{k}/live/R_v"), x.live_r_v);
        cmp.f(&format!("{k}/dead/F_q"), x.dead_f_q);
        cmp.f(&format!("{k}/dead/F_v"), x.dead_f_v);
        cmp.f(&format!("{k}/dead/R_q"), x.dead_r_q);
        cmp.f(&format!("{k}/dead/R_v"), x.dead_r_v);
    }
    cmp.d(&format!("{tag}/n_rows"), g.rows.len());
    for (i, x) in g.rows.iter().enumerate() {
        let k = format!("{tag}/row/{i}");
        cmp.f(&format!("{k}/s"), x.s);
        cmp.opt_s(&format!("{k}/authority"), x.authority.map(|a| a.as_str()));
        cmp.opt_s(&format!("{k}/masked"), x.masked.map(|a| a.as_str()));
        cmp.opt(&format!("{k}/self_masked"), x.self_masked);
        cmp.opt(&format!("{k}/cross_masked"), x.cross_masked);
        cmp.opt(&format!("{k}/self_live"), x.self_live);
        cmp.opt(&format!("{k}/mask_leak"), x.mask_leak);
        cmp.f(&format!("{k}/delta_moved/0"), x.delta_moved.0);
        cmp.f(&format!("{k}/delta_moved/1"), x.delta_moved.1);
        cmp.f(&format!("{k}/delta_rest"), x.delta_rest);
        cmp.f(&format!("{k}/det"), x.det);
        cmp.taus4(&format!("{k}/taus"), x.taus);
        emit_gains(cmp, &format!("{k}/g"), &x.gains);
    }
}

fn emit_disc_arm(cmp: &mut Cmp, tag: &str, dd: &RefDiscriminator) {
    cmp.b(&format!("{tag}/inc"), dd.inc);
    cmp.taus4(&format!("{tag}/taus"), dd.taus);
    cmp.f(&format!("{tag}/ds"), dd.ds);
    cmp.d(&format!("{tag}/n"), dd.n);
    cmp.opt(&format!("{tag}/worst_origin_B"), dd.worst_origin_b);
    cmp.opt(&format!("{tag}/best_origin_C"), dd.best_origin_c);
    cmp.opt(&format!("{tag}/best_origin_72"), dd.best_origin_72);
    cmp.opt(&format!("{tag}/worst_pole_C"), dd.worst_pole_c);
    cmp.opt(&format!("{tag}/worst_pole_72"), dd.worst_pole_72);
    cmp.opt(&format!("{tag}/best_pole_B"), dd.best_pole_b);
    cmp.flist(&format!("{tag}/live_diag_B"), &dd.live_diag_b);
    cmp.flist(&format!("{tag}/live_diag_C"), &dd.live_diag_c);
    for k2 in ["B", "C", "72"] {
        let v = &dd.zeros.iter().find(|(n, _)| *n == k2).expect("zeros arm").1;
        cmp.ilist(&format!("{tag}/zeros/{k2}"), v);
    }
    cmp.ilist(&format!("{tag}/dzeros_B"), &dd.dzeros_b);
    cmp.ilist(&format!("{tag}/dzeros_C"), &dd.dzeros_c);
    cmp.d(&format!("{tag}/n_rows"), dd.rows.len());
    for (i, x) in dd.rows.iter().enumerate() {
        let k = format!("{tag}/row/{i}");
        cmp.f(&format!("{k}/s"), x.s);
        cmp.opt_s(&format!("{k}/authority"), x.authority.map(|a| a.as_str()));
        cmp.opt_s(&format!("{k}/masked"), x.masked.map(|a| a.as_str()));
        cmp.taus4(&format!("{k}/taus"), x.taus);
        cmp.f(&format!("{k}/tau_live"), x.tau_live);
        for (name, val) in [
            ("origin_B", x.origin_b), ("origin_C", x.origin_c), ("origin_72", x.origin_72),
            ("pole_B", x.pole_b), ("pole_C", x.pole_c), ("pole_72", x.pole_72),
            ("live_diag_B", x.live_diag_b), ("live_diag_C", x.live_diag_c),
        ] {
            cmp.f(&format!("{k}/{name}"), val);
        }
        cmp.i(&format!("{k}/zeros_B"), x.zeros_b);
        cmp.i(&format!("{k}/zeros_C"), x.zeros_c);
        cmp.i(&format!("{k}/zeros_72"), x.zeros_72);
    }
}

// ============================================================================== the walk

fn walk(cmp: &mut Cmp) {
    let fl = flight();

    // ------------------------------------------------------------------------ A: handover_law
    let h = handover_law(&machine(), &fl, LO, HI, TT4_MAX, SM, &CLOCKS, R, SETTLE, HL_DS, V_MAX);
    cmp.b("A/always_later", h.always_later);
    cmp.b("A/never_back", h.never_back);
    cmp.b("A/one_handover", h.one_handover);
    cmp.b("A/full_march", h.full_march);
    cmp.f("A/worst_dTt4", h.worst_d_tt4);
    cmp.f("A/worst_dphi", h.worst_d_phi);
    cmp.f("A/worst_delay", h.worst_delay);
    cmp.f("A/ds", h.ds);
    cmp.d("A/n_clocks", h.clocks.len());
    for (i, c) in h.clocks.iter().enumerate() { cmp.taus4(&format!("A/clock/{i}"), *c); }
    cmp.d("A/n_arms", h.arms.len());
    for (i, a) in h.arms.iter().enumerate() {
        let k = format!("A/arm/{i}");
        cmp.b(&format!("{k}/inc"), a.inc);
        cmp.taus4(&format!("{k}/taus"), a.taus);
        cmp.b(&format!("{k}/later"), a.later);
        cmp.opt(&format!("{k}/delay"), a.delay);
        cmp.f(&format!("{k}/dTt4"), a.d_tt4);
        cmp.f(&format!("{k}/dphi"), a.d_phi);
        for (law, p) in [("sched", &a.sched), ("applied", &a.applied)] {
            let kk = format!("{k}/{law}");
            cmp.d(&format!("{kk}/n"), p.n);
            cmp.flist(&format!("{kk}/handovers"), &p.handovers);
            cmp.flist(&format!("{kk}/hands_back"), &p.hands_back);
            cmp.opt(&format!("{kk}/first_gov"), p.first_gov);
            cmp.opt(&format!("{kk}/max_masked"), p.max_masked);
            cmp.f(&format!("{kk}/final_g_fuel"), p.final_g_fuel);
            cmp.f(&format!("{kk}/final_g_gov"), p.final_g_gov);
            cmp.f(&format!("{kk}/max_Tt4"), p.max_tt4);
            cmp.f(&format!("{kk}/min_phi"), p.min_phi);
            cmp.d(&format!("{kk}/ic_iters"), p.ic_iters);
            cmp.f(&format!("{kk}/ic_res"), p.ic_res);
        }
    }

    // ---------------------------------------------------------------------- B, C: applied_gains
    for (tag, inc) in [("B", false), ("C", true)] {
        let g = applied_gains(&machine(), &fl, LO, HI, TT4_MAX, SM, CLOCKS[0], inc, R, SETTLE,
                              AG_DS, V_MAX, AG_EVERY)
            .expect("the readers' own grid marches");
        emit_gain_arm(cmp, tag, &g);
    }

    // ------------------------------------------------------------------------ D: applied_cells
    let c = applied_cells(&machine(), &fl, LO, HI, TT4_MAX, SM, &CLOCKS, R, SETTLE, AC_DS, V_MAX,
                          AC_EVERY).expect("the readers' own grid marches");
    cmp.b("D/law_holds", c.law_holds);
    cmp.b("D/all_four_cells", c.all_four_cells);
    cmp.f("D/ds", c.ds);
    cmp.f("D/worst_parent_gap", c.worst_parent_gap);
    cmp.f("D/worst_parent_gap_hi", c.worst_parent_gap_hi);
    cmp.f("D/worst_v_gap", c.worst_v_gap);
    cmp.f("D/worst_null", c.worst_null);
    cmp.f("D/worst_det", c.worst_det);
    cmp.f("D/worst_lam", c.worst_lam);
    cmp.f("D/pole_at_origin", c.pole_at_origin);
    cmp.d("D/n_clocks", c.clocks.len());
    for (i, cl) in c.clocks.iter().enumerate() { cmp.taus4(&format!("D/clock/{i}"), *cl); }
    for (nm, table) in [("predicted", &c.predicted), ("rung72", &c.rung72)] {
        cmp.d(&format!("D/{nm}/n"), table.len());
        let mut rows: Vec<&((bool, Authority), usize)> = table.iter().collect();
        rows.sort_by_key(|((i, a), _)| (*i, a.as_str()));
        for ((i, a), n) in rows {
            cmp.d(&format!("D/{nm}/{}_{}", usize::from(*i), a.as_str()), *n);
        }
    }
    cmp.d("D/n_cells", c.cells.len());
    let mut seen: Vec<&((bool, Authority), turbojet::applied_reference::AppliedSeenCell)> =
        c.cells.iter().collect();
    seen.sort_by_key(|((i, a), _)| (*i, a.as_str()));
    for ((inc, auth), v) in seen {
        let k = format!("D/cell/{}_{}", usize::from(*inc), auth.as_str());
        cmp.s(&format!("{k}/parent"), v.parent);
        cmp.dlist(&format!("{k}/zeros"), &v.zeros);
        cmp.f(&format!("{k}/gap"), v.gap);
        cmp.f(&format!("{k}/gap_hi"), v.gap_hi);
        cmp.f(&format!("{k}/vgap"), v.vgap);
        cmp.f(&format!("{k}/pole"), v.pole);
        cmp.f(&format!("{k}/null"), v.null);
        cmp.f(&format!("{k}/lam_max"), v.lam_max);
        cmp.f(&format!("{k}/det"), v.det);
        cmp.d(&format!("{k}/n"), v.n);
        cmp.d(&format!("{k}/n_parent"), v.n_parent);
    }
    cmp.d("D/n_arms", c.arms.len());
    for (i, a) in c.arms.iter().enumerate() {
        let k = format!("D/arm/{i}");
        cmp.b(&format!("{k}/inc"), a.inc);
        cmp.taus4(&format!("{k}/taus"), a.taus);
        cmp.d(&format!("{k}/n_riding"), a.n_riding);
        cmp.d(&format!("{k}/n_sampled"), a.n_sampled);
        cmp.d(&format!("{k}/skipped/switch"), a.skipped_switch);
        cmp.d(&format!("{k}/skipped/regime"), a.skipped_regime);
        cmp.d(&format!("{k}/skipped/parent"), a.skipped_parent);
        cmp.d(&format!("{k}/n_cells"), a.cells.len());
        let mut cells: Vec<&(Authority, turbojet::applied_reference::AppliedCellStat)> =
            a.cells.iter().collect();
        cells.sort_by_key(|(auth, _)| auth.as_str());
        for (auth, cc) in cells {
            let kk = format!("{k}/cell/{}", auth.as_str());
            cmp.d(&format!("{kk}/n"), cc.n);
            cmp.d(&format!("{kk}/n_parent"), cc.n_parent);
            cmp.dlist(&format!("{kk}/zeros"), &cc.zeros);
            cmp.f(&format!("{kk}/gap"), cc.gap);
            cmp.f(&format!("{kk}/gap_hi"), cc.gap_hi);
            cmp.f(&format!("{kk}/vgap"), cc.vgap);
            cmp.f(&format!("{kk}/pole"), cc.pole);
            cmp.f(&format!("{kk}/null"), cc.null);
            cmp.f(&format!("{kk}/lam_max"), cc.lam_max);
            cmp.f(&format!("{kk}/det/lo"), cc.det.0);
            cmp.f(&format!("{kk}/det/hi"), cc.det.1);
            cmp.f(&format!("{kk}/s/lo"), cc.s.0);
            cmp.f(&format!("{kk}/s/hi"), cc.s.1);
            cmp.s(&format!("{kk}/parent"), cc.parent);
        }
    }

    // ------------------------------------------------------------------ E, F: ref_discriminator
    for (tag, inc) in [("E", false), ("F", true)] {
        let dd = ref_discriminator(&machine(), &fl, LO, HI, TT4_MAX, SM, CLOCKS[0], inc, R, SETTLE,
                                   RD_DS, V_MAX, RD_EVERY)
            .expect("the readers' own grid marches");
        emit_disc_arm(cmp, tag, &dd);
    }

    // ----------------------------------------------------------------------- G, H: applied_bill
    for (tag, inc) in [("G", false), ("H", true)] {
        let bl = applied_bill(&machine(), &fl, LO, HI, TT4_MAX, SM, CLOCKS[0], inc, R, SETTLE,
                              AB_DS, V_MAX);
        cmp.b(&format!("{tag}/inc"), bl.inc);
        cmp.taus4(&format!("{tag}/taus"), bl.taus);
        cmp.f(&format!("{tag}/ds"), bl.ds);
        for (name, x) in [
            ("debit_sched", bl.debit_sched), ("debit_applied", bl.debit_applied),
            ("phi_marginal_sched", bl.phi_marginal_sched),
            ("phi_marginal_applied", bl.phi_marginal_applied),
            ("phi_full_sched", bl.phi_full_sched), ("phi_full_applied", bl.phi_full_applied),
            ("Tt4_integral_sched", bl.tt4_integral_sched),
            ("Tt4_integral_applied", bl.tt4_integral_applied),
        ] {
            cmp.f(&format!("{tag}/{name}"), x);
        }
        for (name, x) in [
            ("debit_ratio", bl.debit_ratio), ("kept_sched", bl.kept_sched),
            ("kept_applied", bl.kept_applied), ("handover_sched", bl.handover_sched),
            ("handover_applied", bl.handover_applied),
        ] {
            cmp.opt(&format!("{tag}/{name}"), x);
        }
        emit_shared_bill(cmp, &format!("{tag}/sched"), &bl.sched);
        emit_shared_bill(cmp, &format!("{tag}/applied"), &bl.applied);
    }

    // ============================================================== J: the six-state march
    //
    // The SIGNATURES are INPUTS — Python intercepted them at `_shared_march`'s own boundary, so
    // the arms are the readers' own by construction and a reader that changes its grid changes
    // this section with it. **`in/law` is part of the signature and not decoration**: the same
    // eleven arguments are marched under BOTH reference laws by `handover_law`, and `_reference`
    // is live inside the march, so the trajectory is a function of the law too.
    let n_calls = cmp.input_d("J/n_calls");
    let n_sigs = cmp.input_d("J/n_sigs");
    let stride = cmp.input_d("J/stride");
    assert_eq!(stride, J_STRIDE, "the golden's stride and this file's disagree");
    assert!(n_calls >= n_sigs,
            "J/n_calls ({n_calls}) < J/n_sigs ({n_sigs}) — the dump counted distinct signatures \
             it never called");
    cmp.d("J/n_fields", PT_FLOAT.len() + PT_STR.len() + PT_OPT_STR.len() + PT_INT.len());
    cmp.d("J/n_float_fields", PT_FLOAT.len());
    cmp.d("J/n_str_fields", PT_STR.len());
    cmp.d("J/n_opt_str_fields", PT_OPT_STR.len());
    cmp.d("J/n_int_fields", PT_INT.len());
    for (i, name) in PT_FLOAT.iter().chain(PT_STR.iter()).chain(PT_OPT_STR.iter())
        .chain(PT_INT.iter()).enumerate()
    {
        cmp.s(&format!("J/field/{i}"), name);
    }
    let core = machine();
    for i in 0..n_sigs {
        let p = format!("J/sig/{i}");
        let law = cmp.input_s(&format!("{p}/in/law"));
        let tt4_lo = cmp.input_f(&format!("{p}/in/Tt4_lo"));
        let tt4_hi = cmp.input_f(&format!("{p}/in/Tt4_hi"));
        let tt4_max = cmp.input_f(&format!("{p}/in/Tt4_max"));
        let sm = cmp.input_f(&format!("{p}/in/sm"));
        let taus = (cmp.input_f(&format!("{p}/in/tau/0")), cmp.input_f(&format!("{p}/in/tau/1")),
                    cmp.input_f(&format!("{p}/in/tau/2")), cmp.input_f(&format!("{p}/in/tau/3")));
        let r = cmp.input_f(&format!("{p}/in/r"));
        let s_settle = cmp.input_f(&format!("{p}/in/s_settle"));
        let ds = cmp.input_f(&format!("{p}/in/ds"));
        let v_max = cmp.input_f(&format!("{p}/in/v_max"));
        let inc = cmp.input_d(&format!("{p}/in/inc")) != 0;
        let traj = {
            let _r = RefScope::set(&core.fuel.inner, Some(law));
            shared_march(&core, &fl, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max,
                         inc).3
        };
        cmp.d(&format!("{p}/n_points"), traj.len());
        cmp.d(&format!("{p}/n_emitted"), traj.len().div_ceil(J_STRIDE));
        // THE AGGREGATES ARE OVER EVERY POINT, NOT THE STRIDED ONES — the stride's backstop.
        for (k, name) in PT_FLOAT.iter().enumerate() {
            let col: Vec<f64> = traj.iter().map(|p| point_float(p, k)).collect();
            // Python's `min`/`max` over a list return the FIRST extreme and propagate a NaN —
            // `f64::min` does neither, so the fold is written the long way.
            let mut lo = col[0];
            let mut hi = col[0];
            for x in col.iter().skip(1) {
                if *x < lo { lo = *x; }
                if *x > hi { hi = *x; }
            }
            cmp.f(&format!("{p}/agg/{name}/min"), lo);
            cmp.f(&format!("{p}/agg/{name}/max"), hi);
            cmp.f(&format!("{p}/agg/{name}/last"), col[col.len() - 1]);
        }
        for j in (0..traj.len()).step_by(J_STRIDE) {
            emit_point(cmp, &format!("{p}/pt/{j}"), &traj[j]);
        }
    }

    // ============================================================== K: `_reference`, per call
    //
    // **A REPLAY, so this section measures the FUNCTION and not the plant** — the plant is
    // sections A–H and J, which recompute everything. What the Rust does NOT take on trust is the
    // path split: each replayed tuple's path is RE-DERIVED here from `applied_clip` and the
    // per-path counts of the replay set are recomputed and asserted against the golden's own
    // stratification rule.
    let k_calls = cmp.input_d("K/n_calls");
    let k_stride = cmp.input_d("K/stride");
    let n_replay = cmp.input_d("K/n_replay");
    let mut k_path = [0usize; 4];
    for pth in 1..4usize {
        k_path[pth] = cmp.input_d(&format!("K/path/{pth}"));
        // Path 3 must NEVER return `req` bitwise, and paths 1 and 2 must ALWAYS — that is the
        // rung's whole device, and § 5.29 (iii) measured it 0 / 41 346 / 109 537 on the suite.
        let bit = cmp.input_d(&format!("K/returns_req_bitwise/{pth}"));
        if pth == 3 {
            assert_eq!(bit, 0,
                       "a path-3 call returned `req` BITWISE. Path 3 is `(g_own + req) - clip` \
                        with `clip != g_own`; if it can return `req` exactly, the reference is a \
                        no-op somewhere and every A-vs-B reader is differencing the plant against \
                        itself — rung 73's own recorded failure mode.");
        } else {
            assert_eq!(bit, k_path[pth],
                       "path {pth} returns `req` on {bit} of {} calls — it is an IDENTITY branch \
                        and must return the argument bitwise on every one", k_path[pth]);
        }
    }
    assert_eq!(k_path[1] + k_path[2] + k_path[3], k_calls,
               "K's per-path counts do not sum to the call count");
    assert!(k_path.iter().skip(1).all(|n| *n > 0),
            "a `_reference` path is DEAD on this grid: {k_path:?}. All three are live on the \
             suite's own grid (§ 5.29 (iii)), so a zero here means the dump's readers stopped \
             reaching one — and the dispatch gates at step 5 would then be gating a dead branch.");
    // A property of PYTHON's run that this file cannot recompute — read as an input and tied into
    // the consistency web rather than compared against itself, which would be a key agreeing with
    // its own source. § 5.29 (iii) measured 6 380 such calls on the SUITE's grid, where a RELATIVE
    // gap is undefined; this grid has its own number and the bar is only that it is reachable and
    // bounded by the call count.
    let k_zero = cmp.input_d("K/n_req_exactly_zero");
    assert!(k_zero <= k_calls, "more `req == 0.0` calls than calls");
    // The path-3 gap SPREAD, also Python's own run. **Emitted in ABSOLUTE terms and read that
    // way**: 6 380 of the suite's path-3 calls have `req == 0.0` exactly, where a relative gap has
    // no meaning — the pre-flight's first writing divided by a `1e-300` guard and produced a
    // headline of `5.4e+297`. The web here is that there is exactly one gap per path-3 call, that
    // both ends are PRESENT, and that the interval is non-degenerate and strictly positive: a
    // path-3 call whose gap is 0 would be a path-3 call returning `req`, which the assertion above
    // has already refused.
    let gap_n = cmp.input_d("K/gap3/n");
    assert_eq!(gap_n, k_path[3], "the path-3 gap sample is not one per path-3 call");
    assert!(cmp.input_d("K/gap3/min?") == 1 && cmp.input_d("K/gap3/max?") == 1,
            "the path-3 gap aggregates are absent on a grid with {gap_n} path-3 calls");
    let (gmin, gmax) = (cmp.input_f("K/gap3/min"), cmp.input_f("K/gap3/max"));
    assert!(gmin > 0.0 && gmax >= gmin,
            "the path-3 gap interval is degenerate or non-positive: [{gmin:e}, {gmax:e}]");
    let m = machine();
    let mut replay_path = [0usize; 4];
    for i in 0..n_replay {
        let p = format!("K/replay/{i}");
        let share_law = cmp.input_s(&format!("{p}/share_law"));
        let ref_law = cmp.input_s(&format!("{p}/ref_law"));
        let req = cmp.input_f(&format!("{p}/req"));
        let g_own = cmp.input_f(&format!("{p}/g_own"));
        let gf = cmp.input_f(&format!("{p}/gf"));
        let gr = cmp.input_f(&format!("{p}/gr"));
        let (out, path) = {
            let _s = ShareScope::set(&m, share_law);
            let _r = RefScope::set(&m.fuel.inner, Some(ref_law));
            let out = (R73_TRIPLE.reference)(&m.fuel.inner, req, g_own, gf, gr);
            let path = if ref_law != "applied" {
                1
            } else if applied_clip(&m, gf, gr) == g_own {
                2
            } else {
                3
            };
            (out, path)
        };
        replay_path[path] += 1;
        cmp.f(&format!("{p}/out"), out);
        cmp.d(&format!("{p}/path"), path);
    }
    // THE STRATIFICATION RULE, RE-DERIVED. The dumper emits every `K_STRIDE`-th call OF EACH PATH,
    // so the replay set holds `floor(path_n / stride)` of each. Recomputing it here is what turns
    // `K/path/*` from an input this file trusts into one it checks.
    for pth in 1..4usize {
        assert_eq!(replay_path[pth], k_path[pth] / k_stride,
                   "path {pth}: the replay set holds {} tuples, but the dump's stratified stride \
                    of {k_stride} over {} calls gives {}", replay_path[pth], k_path[pth],
                   k_path[pth] / k_stride);
    }

    // ============================================================== L: the inherited arithmetic
    //
    // § 5.29 (vi) measured rung 73 defining NO solver — `_charpoly4` and `_quartic_roots_c` are
    // rung 72's, inherited entire. **"Inherited" is not "driven"**, so the call counts are keys
    // whether or not they are non-zero, and a gate below refuses a silent zero.
    let cp4_calls = cmp.input_d("L/cp4/n_calls");
    let qr_calls = cmp.input_d("L/qr/n_calls");
    let l_stride = cmp.input_d("L/stride");
    let cp4_replay = cmp.input_d("L/cp4/n_replay");
    let qr_replay = cmp.input_d("L/qr/n_replay");
    assert_eq!(cp4_replay, cp4_calls / l_stride, "L/cp4's replay set is not the stride's");
    assert_eq!(qr_replay, qr_calls / l_stride, "L/qr's replay set is not the stride's");
    for i in 0..cp4_replay {
        let p = format!("L/cp4/{i}");
        let mut a = [[0.0f64; 4]; 4];
        for (rr, row) in a.iter_mut().enumerate() {
            for (cc, x) in row.iter_mut().enumerate() {
                *x = cmp.input_f(&format!("{p}/in/{rr}/{cc}"));
            }
        }
        let coef = charpoly4(&a);
        for (k, x) in coef.iter().enumerate() { cmp.f(&format!("{p}/out/{k}"), *x); }
    }
    for i in 0..qr_replay {
        let p = format!("L/qr/{i}");
        let mut coef = [0.0f64; 5];
        for (k, cf) in coef.iter_mut().enumerate() {
            *cf = cmp.input_f(&format!("{p}/in/{k}"));
        }
        let roots = quartic_roots_c(&coef);
        for (k, z) in roots.iter().enumerate() {
            cmp.f(&format!("{p}/out/{k}/re"), z.re);
            cmp.f(&format!("{p}/out/{k}/im"), z.im);
            cmp.f(&format!("{p}/out/{k}/abs"), z.abs());
        }
    }

    // ============================================================== M: the signed-zero census
    //
    // Step 2's M22, re-measured on this grid and on a dump fifteen times wider. Step 2 mutated the
    // four `sorted({...})` sets in `applied_gains` to key by BITS and got **0 of 5 066**, then
    // measured WHY: 101 keys were exactly `-0.0` (every one a `*/g/pair_FR`), 925 exactly `+0.0`,
    // and inside those four sets `+0.0` appeared twice and `-0.0` not at all. The hazard is real
    // in the dump and never enters a set the defence guards. **Booked here, to a wider grid.**
    //
    // **THE CENSUS IS OVER VALUES THIS FILE PRODUCED, wherever it produced one.** Every key
    // section M ranges over is either COMPARED (its Rust value is in `mine`) or a DECLARED INPUT
    // (its only value is the golden's). Taking the compared ones from `mine` is what stops this
    // from being the golden agreeing with itself; taking the inputs from the golden is stated
    // rather than hidden, and the split itself is emitted as a key so it cannot drift silently.
    const SETS: [&str; 4] = ["self_masked", "cross_masked", "self_live", "moved_scaled"];
    let neg0 = (-0.0f64).to_bits();
    // The whole census is computed under an IMMUTABLE borrow and only then emitted, because
    // `Cmp::d` needs `&mut self` and a half-emitted census over a map being written is not a
    // census. Counts only — nothing is cloned.
    let (n_before, n_neg, n_pos, set_stats) = {
        let (mut nb, mut nn, mut np) = (0usize, 0usize, 0usize);
        let mut sets = [(0usize, 0usize, 0usize); 4];
        for (k, gold) in cmp.py.iter() {
            if k.starts_with("M/") {
                continue;
            }
            let v = *cmp.mine.get(k).unwrap_or(gold);
            nb += 1;
            nn += usize::from(v == neg0);
            np += usize::from(v == 0);
            for (j, nm) in SETS.iter().enumerate() {
                let in_set = (k.starts_with(&format!("B/{nm}/"))
                              || k.starts_with(&format!("C/{nm}/")))
                    && !k.ends_with("/n");
                if in_set {
                    sets[j].0 += 1;
                    sets[j].1 += usize::from(v == neg0);
                    sets[j].2 += usize::from(v == 0);
                }
            }
        }
        (nb, nn, np, sets)
    };
    cmp.d("M/n_keys_before_M", n_before);
    cmp.d("M/n_neg_zero", n_neg);
    cmp.d("M/n_pos_zero", n_pos);
    for (j, nm) in SETS.iter().enumerate() {
        cmp.d(&format!("M/set/{nm}/n"), set_stats[j].0);
        cmp.d(&format!("M/set/{nm}/n_neg"), set_stats[j].1);
        cmp.d(&format!("M/set/{nm}/n_pos"), set_stats[j].2);
    }
}

// ============================================================================== the gates

#[test]
fn rust_equals_pypy_on_every_key() {
    let mut cmp = Cmp::new(load(ORACLE_PYPY), false);
    walk(&mut cmp);
    cmp.finish("pypy");
}

#[test]
fn rust_equals_cpython_outside_the_named_exemption() {
    let mut cmp = Cmp::new(load(ORACLE_CPYTHON), true);
    walk(&mut cmp);
    cmp.finish("cpython 3.14");
}

/// **THE TWO GOLDENS MUST HAVE THE SAME KEY SET.** A dump that emitted a key on one interpreter
/// and not the other would make the exemption a comparison between differently-shaped files.
#[test]
fn the_two_goldens_have_the_same_key_set() {
    let (a, b) = (load(ORACLE_PYPY), load(ORACLE_CPYTHON));
    let (ka, kb): (BTreeSet<&String>, BTreeSet<&String>) = (a.keys().collect(), b.keys().collect());
    assert_eq!(ka, kb, "the two arms' goldens have different key sets: {} only in PyPy, {} only \
                        in CPython", ka.difference(&kb).count(), kb.difference(&ka).count());
    assert_eq!(a.len(), GOLDEN_KEYS, "GOLDEN_KEYS is stale");
}

/// **THE STRIDE IS COPRIME TO THE READERS' OWN SAMPLING, AND THAT IS THE WHOLE REASON IT IS 5.**
/// `applied_gains` and `applied_cells` sample at `every = 2`, `ref_discriminator` at 4. A section-J
/// stride of 2 or 4 would emit exactly the points their rows already carry — an extra grid that is
/// not extra. Written as a gate rather than a comment because a later edit to `J_STRIDE` would
/// silently undo it.
#[test]
fn the_march_stride_is_coprime_to_the_readers_sampling() {
    fn gcd(a: usize, b: usize) -> usize { if b == 0 { a } else { gcd(b, a % b) } }
    for every in [AG_EVERY, AC_EVERY, RD_EVERY] {
        assert_eq!(gcd(J_STRIDE, every), 1,
                   "J_STRIDE {J_STRIDE} shares a factor with a reader's `every` of {every}: \
                    section J would re-emit the points that reader's rows already cover");
    }
    assert!(J_STRIDE > 1, "a stride of 1 is not a stride");
}

/// **THE INHERITED ARITHMETIC IS ACTUALLY DRIVEN, AND THAT IS MEASURED RATHER THAN ASSUMED.**
///
/// P3's clause (a) predicts the CPython exemption is dominated by the `sum()`-built polynomial in
/// rung 72's `_charpoly4`. That clause is VACUOUS unless this grid enters the function — and this
/// slice has already watched two confident zeros come out of unreached code (AD's j06, twice) and
/// four probes in its own pre-flight report numbers from runs that measured nothing. So the call
/// counts are gated, in both directions.
#[test]
fn the_inherited_quartic_chain_is_reached() {
    let py = load(ORACLE_PYPY);
    let get = |k: &str| *py.get(k).unwrap_or_else(|| panic!("{k} missing")) as usize;
    assert!(get("L/cp4/n_calls") > 0,
            "`_charpoly4` is not reached on this grid — P3's clause (a) is then vacuous and the \
             header's claim that the exemption is dominated by its `sum()` cannot be scored");
    assert!(get("L/qr/n_calls") > 0,
            "`_quartic_roots_c` is not reached on this grid — section L's second half gates \
             nothing and should be deleted rather than left reading like coverage");
    assert!(get("L/qr/n_calls") <= get("L/cp4/n_calls"),
            "more root solves than characteristic polynomials — the chain runs the other way");
    assert!(get("L/cp4/n_replay") > 0 && get("L/qr/n_replay") > 0,
            "section L's stride emptied a replay set");
}

/// **THE `_reference` CENSUS IS THIS GRID's OWN, and a tripwire against transcribing the
/// suite's.** § 5.29 (iii) measured **41 346 / 109 537 / 109 307** by intercepting every call the
/// whole rung-73 SUITE makes. This dump drives the five readers at their own defaults. Quoting the
/// suite-wide triple against this grid would be [[rust-port-slice-ac-step6]]'s `every = 40`-vs-
/// `10` defect: a number measured at one population, asserted against another.
#[test]
fn the_reference_census_is_this_grids_own() {
    let py = load(ORACLE_PYPY);
    let get = |k: &str| *py.get(k).unwrap_or_else(|| panic!("{k} missing")) as usize;
    let (p1, p2, p3) = (get("K/path/1"), get("K/path/2"), get("K/path/3"));
    assert_ne!((p1, p2, p3), (41_346, 109_537, 109_307),
               "section K's census is the SUITE-WIDE triple from § 5.29 (iii). Those were measured \
                over every call the whole rung-73 suite makes; this dump drives the five readers. \
                If they have genuinely converged, delete this tripwire and SAY SO.");
    assert_eq!(p1 + p2 + p3, get("K/n_calls"), "the paths do not sum to the calls");
    assert_eq!(get("K/returns_req_bitwise/3"), 0,
               "path 3 returned `req` bitwise — see the walk's own assertion for why that is the \
                rung's failure mode and not a rounding");
    assert!(get("K/gap3/n") == p3, "the path-3 gap sample is not one per path-3 call");
}
