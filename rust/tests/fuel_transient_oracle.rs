//! SLICE S step 4 — THE ORACLE for [`TwoSpoolFuelTransient`] (rungs 43 + 45), over BOTH suites'
//! grids, against `oracle/dump_fuel_transient.py`.
//!
//! Step 1's smoke touched every path once on cells chosen to REACH a path. This file runs the
//! grids `test_rung43.py` and `test_rung45.py` actually sweep, because steps 2 and 3 measured
//! that those twenty gates cannot see several things the port could get wrong:
//!
//! * **seven of rung 43's eleven gates are blind to a wrong LP derivative** (step 2 finding 2),
//!   and rung 45 has no dynamical reduce at all (step 3 finding 4);
//! * **`npts` is the ONLY channel that witnesses the march bound** — step 3 finding 3 measured
//!   dropping the `r` from `r + s_settle` leaving `min_phi_lp` bit-identical at all four ramp
//!   rates while the lengths moved 351/326/316/306 → 301. § 5.16 booked it here as an oracle key
//!   and it is carried on EVERY marched cell;
//! * **`s_lp`/`s_hp`, `min_phi_hp`, `ratio`, `E_temp_*`** are returned by the shipped methods and
//!   read by nothing in either suite.
//!
//! # Four registered numbers died on the grid this file runs
//!
//! § 5.16's predictions 4 and 6 quote a census `probe_s2.py` took on a grid its own header calls
//! "rungs 43 and 45's OWN grids" and which is a cross-product of its own choosing. On the grids
//! the suites really sweep:
//!
//! | quantity | § 5.16 registered | measured here |
//! |---|---|---|
//! | `integrate_fuel` calls | 162 | **140** in this file's cells |
//! | …on the `412.5` tie | 21 of 162 | **52 of 143** across both suites |
//! | high wall `literal / map / hi0` | 24 033 / 200 193 / 3 663 | **1 398 / 228 801 / 1 210** |
//! | CPG float keys moving under CPython (pred. 2) | **0** | **15** — see [`is_libm_score`] |
//!
//! This file's own cells give **1 398 / 223 890 / 1 210** over 140 marches — it folds rung 43
//! gate 10's `freeze_channels` call into section F rather than repeating it, which costs three
//! marches and 4 911 map-arm calls and no literal or `hi0` ones. Both numbers are stated because
//! quoting either for the other is the mistake this whole table is about.
//!
//! **An instrument's own docstring is not evidence about what it measured.** Probe 2's grid is not
//! a superset of the suites' and not a subset either — it is a different grid, and the header
//! saying otherwise is why nobody re-derived it. Fourth time in one slice that a census turned out
//! to be a property of the grid rather than of the code.
//!
//! **AND THE HIGH WALL'S TWO RARE ARMS LIVE IN ONE CELL OF THE TWENTY GATES.** Every literal hit
//! and every `hi0` hit comes from `test_rung45.py`'s `hp-only` shape, whose LP map is
//! `ComponentMap::flat()`: a flat map has no `phi_max` ceiling, so `2.5` binds on the accel and
//! the decel's low fuel drops `hi0` under both. [`the_high_walls_rare_arms_live_in_one_cell`]
//! asserts that, and section L's census is emitted PER CELL so a section total cannot let one
//! shape's 228 801 map hits bury another's 1 301 literal ones.
//!
//! # The three arms
//!
//! * **main** (`fuel_transient_pypy.tsv`) — the CPG grids of both suites, sections A…S.
//! * **gas** (`fuel_transient_gas_pypy.tsv`) — the NON-CPG gases: the three admitted TPG ones and
//!   the one `Gas::reacting_equilibrium()` the fuel path REFUSES. An ADDED arm; neither suite runs
//!   a TPG gas through the fuel path, and it is here because § 5.16 probe 3 measured
//!   `equilibrium_fuel`'s Newton pass count — an ABSOLUTE `1e-12` bar under the gas sub-solve's own
//!   ~1e-10 noise — swinging 16-fold between interpreters.
//! * **cpython** (`fuel_transient_cpython.tsv`) — main + gas under CPython 3.14, read as a
//!   DETECTOR with a measured sensitivity, never as coverage. See [`oracle_matches_cpython`].
//!
//! Regenerate with:
//!     .venv\Scripts\python.exe rust/oracle/dump_fuel_transient.py main    rust/oracle/fuel_transient_pypy.tsv
//!     .venv\Scripts\python.exe rust/oracle/dump_fuel_transient.py gas     rust/oracle/fuel_transient_gas_pypy.tsv
//!     C:\Python314\python.exe  rust/oracle/dump_fuel_transient.py cpython rust/oracle/fuel_transient_cpython.tsv

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::fuel_transient::{
    counters as fcount, ConstantSpeedExcursionFuel, FreezeChannels, FuelInstant, FuelLimiters,
    FuelPoint, FuelTransientCore, PhiExcursionFuel, RampExcursionFuel, TransientSurgeMarginFuel,
    TwoSpoolFuelTransient,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::spool::SpoolTransient;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{counters as tcount, TwoSpoolTransientCore};

const ORACLE_MAIN: &str = include_str!("../oracle/fuel_transient_pypy.tsv");
const ORACLE_GAS: &str = include_str!("../oracle/fuel_transient_gas_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/fuel_transient_cpython.tsv");

// ---------------------------------------------------------------------------- the CPython tiers
/// A key's bar on the CPython arm. Against PyPy every key is [`Tier::Bits`].
enum Tier {
    Bits,
    Rel(f64),
}

/// **THE FRAGILE HALF IS CHOSEN BY GAS, NOT BY FILE.** Anything under the `T/` (three admitted TPG
/// gases) or `U/` (the refused equilibrium gas) sections is a declared fragile set: § 5.16 probe 1
/// measured 391 of 398 float keys moving CPython-vs-PyPy at ~1e-10 relative on exactly these
/// gases. Measured again on this file's own grid: **731 of 801 float keys move, median 2.6e-12,
/// p90 5.2e-11**, and 24 of them are the Newton RESIDUALS `Phi_lp`/`Phi_hp`, whose relative
/// deviation is meaningless because they are converged to ~1e-13 ABSOLUTE.
///
/// It is published as a distribution and NEVER summed with the CPG half.
fn is_fragile_gas(key: &str) -> bool {
    let s = key.strip_prefix("census/").unwrap_or(key);
    s.starts_with("T/") || s.starts_with("U/")
}

/// **ITERATION COUNTS ARE NOT INTERPRETER-INVARIANT — slice N's rule, third instance.** Slice R
/// measured one CPG `illinois_evals` key moving by exactly one (38 513 vs 38 512) with every value
/// those root finds produce bit-identical. Measured here: **1 of the 66 CPG-half iteration-count
/// keys moves** — `census/L/tilted/t40/illinois_evals`, 5178 → 5177, on a rung-44 `phi_excursion`
/// call, i.e. in the INHERITED class rather than in this slice's own.
///
/// **THE CLASS IS EXEMPTED, NOT THE KEY, AND DELIBERATELY WIDER THAN THE MEASUREMENT.** What makes
/// a key exempt is BEING an iteration count; pinning the one that happens to move today would turn
/// a CPython point release into a port regression. The width is not hidden —
/// [`oracle_matches_cpython`] prints how many of each class actually moved.
fn is_iteration_count(key: &str) -> bool {
    key.ends_with("/illinois_evals") || key.ends_with("/illinois_calls")
}

/// **A NEW EXEMPT CLASS THIS STEP HAD TO INVENT, AND § 5.16 PREDICTION 2 DIED OF IT.**
///
/// Prediction 2 registered that the CPG half moves **0** float keys. It moves **15**, and all 15
/// are `collapse_exponent`'s scored curve. The reason is not the plant: `spread(q)` computes
/// `r / rho.powf(q)` and `ln`, so it is a composite of two LIBM calls, and libm is the one thing a
/// port shares with neither interpreter. Every value that reaches this file THROUGH THE PLANT is
/// bit-identical CPython-to-PyPy; the movers are pure arithmetic over already-agreed inputs, at
/// ≤ 3.14e-16 relative — one ULP.
///
/// **`J/collapse/*/q` IS NOT IN THIS CLASS, AND THAT IS A MEASUREMENT.** The argmin is a TIE — every
/// currency's minimum is attained by two adjacent `q` at a gap of exactly `0.000e+00` (step 1
/// finding 5) — so a one-ULP move on either tied score would BREAK the tie and hand the fold a
/// different exponent. It did not: all three argmins are bit-identical across interpreters, which
/// is why `q` stays on the bit bar while the curve it is read off does not.
fn is_libm_score(key: &str) -> bool {
    key.starts_with("J/collapse/") && (key.contains("/score/") || key.ends_with("/spread"))
}

fn tier(key: &str) -> Tier {
    if is_fragile_gas(key) {
        // Published, not gated: the bar is wide enough to catch a STRUCTURAL error (a wrong
        // branch, a wrong constant) and nothing tighter, and the distribution is printed.
        Tier::Rel(1e-3)
    } else if is_libm_score(key) {
        Tier::Rel(1e-13)
    } else {
        Tier::Bits
    }
}

/// The two keys the Rust deliberately never compares — section K's UNARMED degenerate refusals.
/// See [`the_unarmed_degenerate_refusal_is_a_disclosed_divergence`]; the count is asserted so a
/// third cannot join them by accident.
fn is_python_only(key: &str) -> bool {
    key.contains("/pyonly/")
}

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, b) = (it.next().expect("key"), it.next().expect("bits"));
        assert!(m.insert(k.to_string(), b.parse::<u64>().expect("u64")).is_none(), "dup {k}");
    }
    m
}

/// Accumulates `(key, got, want)` so ONE run reports every disagreement, **and reports every golden
/// key the Rust never asked for**.
///
/// Both halves panic together. Step 1 found this comparator's ancestor asserting the VALUE diffs
/// BEFORE the never-compared ones, which made the half that exists to catch a field missing from
/// the PORT unreachable whenever any value also moved.
struct Cmp {
    py: BTreeMap<String, u64>,
    seen: BTreeSet<String>,
    bad: Vec<String>,
    /// Set on the CPython arm, where a TPG-gas disagreement is content rather than a failure.
    cpython: bool,
    discrete_flips: Vec<String>,
    float_drifts: Vec<(String, f64)>,
    /// How many `pyonly` keys THIS golden is expected to carry — 2 for the main arm, 0 for the
    /// gas one. Asserted rather than tolerated, so an exemption cannot grow without being argued
    /// for.
    expect_pyonly: usize,
}

impl Cmp {
    fn new(py: BTreeMap<String, u64>) -> Self {
        Cmp { py, seen: BTreeSet::new(), bad: Vec::new(), cpython: false,
              discrete_flips: Vec::new(), float_drifts: Vec::new(), expect_pyonly: 0 }
    }

    fn f(&mut self, key: &str, got: f64) {
        assert!(got.is_finite(), "{key} is not finite: {got}");
        if !self.cpython {
            return self.cmp_bits(key, got.to_bits(), false);
        }
        assert!(self.seen.insert(key.to_string()), "the Rust emitted {key} twice");
        let Some(&want) = self.py.get(key) else {
            self.bad.push(format!("{key}: NO GOLDEN (the dump never emitted it)"));
            return;
        };
        let exp = f64::from_bits(want);
        let d = (got - exp).abs();
        let (over, scale) = match tier(key) {
            Tier::Bits => (got.to_bits() != want, 0.0),
            Tier::Rel(bar) => {
                // The Newton RESIDUALS are converged to ~1e-13 ABSOLUTE, so a relative bar on
                // them measures nothing but the size of a number that is meant to be zero. They
                // fall back to an absolute one — named here rather than buried in `tier`, because
                // it is a property of the QUANTITY and not of the arm.
                let residual = key.ends_with("/Phi_lp") || key.ends_with("/Phi_hp");
                if residual {
                    (d > 1e-9, d)
                } else {
                    let rel = if exp == 0.0 { d } else { d / exp.abs() };
                    (rel > bar, rel)
                }
            }
        };
        if d > 0.0 {
            self.float_drifts.push((key.to_string(), scale));
        }
        if over {
            self.bad.push(format!("{key}: rust {got:e} vs cpython {exp:e} (dev {scale:e})"));
        }
    }

    fn d(&mut self, key: &str, got: u64) {
        self.cmp_bits(key, got, true);
    }

    fn cmp_bits(&mut self, key: &str, got: u64, discrete: bool) {
        assert!(self.seen.insert(key.to_string()), "the Rust emitted {key} twice");
        match self.py.get(key) {
            None => self.bad.push(format!("{key}: NO GOLDEN (the dump never emitted it)")),
            Some(&want) if want != got => {
                let tolerated = discrete && self.cpython
                    && (is_fragile_gas(key) || is_iteration_count(key));
                if tolerated {
                    self.discrete_flips.push(format!("{key}: rust {got} vs cpython {want}"));
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

    fn finish(&self) {
        let missed: Vec<&String> =
            self.py.keys().filter(|k| !self.seen.contains(*k) && !is_python_only(k)).collect();
        let pyonly = self.py.keys().filter(|k| is_python_only(k)).count();
        assert_eq!(pyonly, self.expect_pyonly,
                   "the declared Python-only keys are section K's two unarmed \
                    degenerate refusals and nothing else — one more means an exemption grew \
                    without being argued for");
        if self.bad.is_empty() && missed.is_empty() {
            return;
        }
        panic!(
            "{} of {} compared keys differ:\n  {}\n\n{} golden keys the Rust never asked for (a \
             field missing from the port is invisible until this fires):\n  {:?}",
            self.bad.len(), self.seen.len(), self.bad.join("\n  "), missed.len(),
            missed.iter().take(24).collect::<Vec<_>>());
    }
}

// ---------------------------------------------------------------------------------- the grid
const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
/// RUNG 43's band. Rung 45 ramps `1000 → 1400` instead — a different pair, spelled separately.
const LO: f64 = 1250.0;
const HI: f64 = 1450.0;
const LO45: f64 = 1000.0;
const HI45: f64 = 1400.0;
/// `phi_excursion(flight, Tt4_lo, dTt4, …)` takes a STEP where `phi_excursion_fuel` takes an
/// ENDPOINT. Step 3 measured that porting rung 45's `400.0` as an endpoint is caught by exactly
/// ONE of `rung45.rs`'s ten tests.
const DTT4: f64 = HI45 - LO45;
/// Both rung-45 methods' silent defaults (`engine.py:5346`).
const S_SETTLE45: f64 = 6.0;
/// `ramp_excursion_fuel`'s (`engine.py:5180`) — and the reason the `r = 0.25` cells sit on the tie:
/// `(0.25 + 8.0) / 0.02 = 412.5` exactly.
const S_SETTLE43: f64 = 8.0;
const DS: f64 = 0.02;

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
/// `test_rung43.py`'s `SINGLE` — `eta_c = 0.88`, and it DOES carry `nozzle_convergent`. Rung 45's
/// is a different dict (`eta_c = 0.90`, no `nozzle_convergent`) and is not needed here: the only
/// single-spool object this file builds is rung 43's gate-2 degenerate one.
const SINGLE43: Losses = Losses {
    pi_d: 0.97, eta_c: 0.88, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, e_t: None,
    eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// `test_rung43.py:62` — `R_c` HARD-CODED at 286.9.
fn gas43() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

/// `test_rung45.py:83` — `R_c` DERIVED. Built by THIS suite's own expression, never by copying the
/// other's literal: the two gases' whole fuel-path dump is bit-identical and only a THRUST key
/// witnesses the difference, which is what section A is for.
fn gas45() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

fn tilted() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
}

/// `test_rung43.py`'s three shapes, in Python's dict order.
fn shapes43() -> [(&'static str, ComponentMap, ComponentMap); 3] {
    let f = ComponentMap::flat();
    let m = |a, b, sigma, l| ComponentMap { a, b, sigma, l, ..f };
    [
        ("flow/press", lp_shaped(), hp_shaped()),
        ("press/flow", m(0.05, 0.20, 0.1, 1.0), m(0.20, 0.05, 0.1, 0.7)),
        ("tilted", tilted(), tilted()),
    ]
}

/// …plus rung 45's fourth — rung 40's DISCRIMINATOR, an LP map that is FLAT, and the ONE cell in
/// either suite that reaches two of the high wall's three arms.
fn shapes45() -> [(&'static str, ComponentMap, ComponentMap); 4] {
    let [a, b, c] = shapes43();
    [a, b, c, ("hp-only", ComponentMap::flat(), hp_shaped())]
}

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

fn core(d: &TwoSpoolEngine, ml: ComponentMap, mh: ComponentMap, rho: f64) -> FuelTransientCore {
    FuelTransientCore::new(d.clone(), flight(), 1.0, ml, mh, rho)
}

// ---------------------------------------------------------------------------- the emitters
/// All 45 of `_instant_fuel`'s keys, by PYTHON's sorted name order. The key LIST comes from the
/// dump's enumeration of Python's own dict, so a field forgotten in the port shows up as a
/// never-compared golden key rather than as a silently narrower comparison.
fn put_eqf(c: &mut Cmp, p: &str, i: &FuelInstant, passes: usize) {
    let b = &i.base;
    let k = &b.close;
    c.f(&format!("{p}/M9"), b.m9);
    c.f(&format!("{p}/Pc_hp"), b.pc_hp);
    c.f(&format!("{p}/Pc_lp"), b.pc_lp);
    c.f(&format!("{p}/Phi_hp"), b.phi_hp_dot);
    c.f(&format!("{p}/Phi_lp"), b.phi_lp_dot);
    c.f(&format!("{p}/Pt_hp"), b.pt_hp);
    c.f(&format!("{p}/Pt_lp"), b.pt_lp);
    c.f(&format!("{p}/Tt2"), k.tt2);
    c.f(&format!("{p}/Tt25"), k.tt25);
    c.f(&format!("{p}/Tt3"), k.tt3);
    c.f(&format!("{p}/Tt4"), b.tt4);
    c.f(&format!("{p}/Tt45"), b.tt45);
    c.f(&format!("{p}/Tt5"), b.tt5);
    c.d(&format!("{p}/branch_choked"), u64::from(b.branch == Branch::Choked));
    c.f(&format!("{p}/eta_hpc"), k.eta_hpc);
    c.f(&format!("{p}/eta_hpt"), b.eta_hpt);
    c.f(&format!("{p}/eta_lpc"), k.eta_lpc);
    c.f(&format!("{p}/eta_lpt"), b.eta_lpt);
    c.f(&format!("{p}/f"), k.f);
    c.f(&format!("{p}/m_hp"), k.m_hp);
    c.f(&format!("{p}/m_imp"), k.m_imp);
    c.f(&format!("{p}/m_lp"), k.m_lp);
    c.f(&format!("{p}/mdot4"), k.mdot4);
    c.f(&format!("{p}/mdot_air"), k.mdot_air);
    c.f(&format!("{p}/mdot_air_face"), i.mdot_air_face);
    c.f(&format!("{p}/n_hp"), k.n_hp);
    c.f(&format!("{p}/n_lp"), k.n_lp);
    c.f(&format!("{p}/nu_hp"), b.nu_hp);
    c.f(&format!("{p}/nu_hpt"), b.nu_hpt);
    c.f(&format!("{p}/nu_lp"), b.nu_lp);
    c.f(&format!("{p}/nu_lpt"), b.nu_lpt);
    c.f(&format!("{p}/phi_hp"), k.phi_hp);
    c.f(&format!("{p}/phi_lp"), k.phi_lp);
    c.f(&format!("{p}/pi_hpc"), k.pi_hpc);
    c.f(&format!("{p}/pi_hpt"), b.pi_hpt);
    c.f(&format!("{p}/pi_lpc"), k.pi_lpc);
    c.f(&format!("{p}/pi_lpt"), b.pi_lpt);
    c.f(&format!("{p}/pt4"), k.pt4);
    c.f(&format!("{p}/slip"), b.slip);
    c.f(&format!("{p}/sp_thrust"), b.sp_thrust);
    c.f(&format!("{p}/tau_hpc"), k.tau_hpc);
    c.f(&format!("{p}/tau_hpt"), b.tau_hpt);
    c.f(&format!("{p}/tau_lpc"), k.tau_lpc);
    c.f(&format!("{p}/tau_lpt"), b.tau_lpt);
    c.d(&format!("{p}/passes"), passes as u64);
}

fn put_point(c: &mut Cmp, p: &str, pt: &FuelPoint) {
    c.f(&format!("{p}/Tt4"), pt.tt4);
    c.d(&format!("{p}/branch_choked"), u64::from(pt.branch == Branch::Choked));
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
}

/// A ramp cell: the seven reported fields, `npts`, and the LOCATION of the peak plus the peak
/// point itself.
///
/// The location key is why this is more than a summary. § 5.16 prediction 4 measured the peak
/// attained at point 13 of 413 — 3 % into a march that is 95 % settling tail — so all seven
/// reported fields are decided by a handful of early points, and a port whose interior drifted
/// after them would agree on every one. `i_peak` pins WHERE and the point pins WHAT.
fn put_ramp(c: &mut Cmp, p: &str, e: &RampExcursionFuel) {
    c.f(&format!("{p}/r"), e.r);
    c.f(&format!("{p}/rho"), e.rho);
    c.f(&format!("{p}/Tt4_peak"), e.tt4_peak);
    c.f(&format!("{p}/X"), e.x);
    c.f(&format!("{p}/E_temp_H"), e.e_temp_h);
    c.f(&format!("{p}/E_temp_L"), e.e_temp_l);
    c.d(&format!("{p}/complete"), u64::from(e.complete));
    c.d(&format!("{p}/npts"), e.traj.len() as u64);
    // Python's `max(range(n), key=…)` keeps the FIRST of equals; so does `max_by` on a reversed
    // comparison? No — `max_by` keeps the LAST. Spelled as a fold that keeps the first, which is
    // Python's rule and one keystroke from the wrong one.
    let mut ip = 0usize;
    for (j, pt) in e.traj.iter().enumerate() {
        if pt.tt4 > e.traj[ip].tt4 {
            ip = j;
        }
    }
    c.d(&format!("{p}/i_peak"), ip as u64);
    put_point(c, &format!("{p}/peak"), &e.traj[ip]);
    put_point(c, &format!("{p}/last"), e.traj.last().expect("a non-empty trajectory"));
}

/// All NINE of `phi_excursion_fuel`'s keys, including the four no Python gate reads.
fn put_exc(c: &mut Cmp, p: &str, e: &PhiExcursionFuel) {
    c.f(&format!("{p}/ext_lp"), e.ext_lp);
    c.f(&format!("{p}/ext_hp"), e.ext_hp);
    c.f(&format!("{p}/s_lp"), e.s_lp);
    c.f(&format!("{p}/s_hp"), e.s_hp);
    c.f(&format!("{p}/min_phi_lp"), e.min_phi_lp);
    c.f(&format!("{p}/min_phi_hp"), e.min_phi_hp);
    c.f(&format!("{p}/Tt4_peak"), e.tt4_peak);
    c.f(&format!("{p}/ratio"), e.ratio);
    c.d(&format!("{p}/npts"), e.npts as u64);
}

fn put_sm(c: &mut Cmp, p: &str, m: &TransientSurgeMarginFuel) {
    c.f(&format!("{p}/margin_min_lp"), m.margin_min_lp);
    c.f(&format!("{p}/margin_min_hp"), m.margin_min_hp);
    c.f(&format!("{p}/steady_min_lp"), m.steady_min_lp);
    c.f(&format!("{p}/steady_min_hp"), m.steady_min_hp);
    c.f(&format!("{p}/min_phi_lp"), m.min_phi_lp);
    c.f(&format!("{p}/min_phi_hp"), m.min_phi_hp);
    c.f(&format!("{p}/phi_surge_lp"), m.phi_surge_lp);
    c.f(&format!("{p}/phi_surge_hp"), m.phi_surge_hp);
    c.d(&format!("{p}/crossed_lp"), u64::from(m.crossed_lp));
    c.d(&format!("{p}/crossed_hp"), u64::from(m.crossed_hp));
    c.d(&format!("{p}/npts"), m.npts as u64);
}

fn put_fc(c: &mut Cmp, p: &str, f: &FreezeChannels) {
    c.f(&format!("{p}/both"), f.both);
    c.f(&format!("{p}/lp"), f.lp);
    c.f(&format!("{p}/hp"), f.hp);
    c.f(&format!("{p}/d_lp"), f.d_lp);
    c.f(&format!("{p}/d_hp"), f.d_hp);
    c.f(&format!("{p}/r"), f.r);
    c.f(&format!("{p}/rho"), f.rho);
}

fn put_cs(c: &mut Cmp, p: &str, x: &ConstantSpeedExcursionFuel) {
    c.f(&format!("{p}/Tt4_peak"), x.tt4_peak);
    c.f(&format!("{p}/E_temp"), x.e_temp);
    c.f(&format!("{p}/E_lp"), x.e_lp);
    c.f(&format!("{p}/E_hp"), x.e_hp);
    c.f(&format!("{p}/f"), x.f);
}

/// Read (and reset) rung 34's Illinois counters — the ONE counter set this slice shares with an
/// earlier one.
/// A float spelled into a KEY the way Python's f-string spells it — `1.0`, not `1`. Rust's
/// `Display` drops the trailing `.0` and `Debug` keeps it, which is the whole difference between
/// a golden that compares and one that reports 200 keys "NO GOLDEN".
fn fk(x: f64) -> String {
    format!("{x:?}")
}

fn take_illinois() -> (u64, u64, u64) {
    let c = turbojet::spool::counters::take();
    (c.illinois_calls, c.illinois_evals, c.illinois_exhausted)
}

/// Running totals the per-section keys cannot express, kept so the sweep's SHAPE claims can be
/// asserted once at the end rather than 33 times.
#[derive(Default)]
struct Totals {
    close_calls: u64,
    march_calls: u64,
    march_points: u64,
    der_calls: u64,
    rw_calls: u64,
    npts_sum: u64,
    tie_marches: u64,
    hi_literal: u64,
    hi_map: u64,
    hi_hi0: u64,
    eq_calls: u64,
    eq_passes: u64,
    collapse_ties: u64,
    sections: u64,
}

/// Close a census section: emit the keys Python can see, gate the ones it cannot, accumulate the
/// totals — and drain the shared counters so nothing leaks into the next section.
fn census(c: &mut Cmp, p: &str, t: &mut Totals, npts_here: Option<u64>, bracket_fails: u64) {
    let cs = fcount::take();
    let ill = take_illinois();
    let _ = tcount::take();

    c.d(&format!("census/{p}/close_calls"), cs.close_calls);
    c.d(&format!("census/{p}/instant_calls"), cs.instant_calls);
    c.d(&format!("census/{p}/eq_calls"), cs.eq_calls);
    c.d(&format!("census/{p}/eq_passes"), cs.eq_passes);
    c.d(&format!("census/{p}/march_calls"), cs.march_calls);
    c.d(&format!("census/{p}/march_points"), cs.march_points);
    c.d(&format!("census/{p}/topping_calls"), cs.topping_calls);
    c.d(&format!("census/{p}/sched_calls"), cs.sched_calls);
    c.d(&format!("census/{p}/sched_dormant"), cs.sched_dormant);
    c.d(&format!("census/{p}/surge_calls"), cs.surge_calls);
    c.d(&format!("census/{p}/surge_dormant"), cs.surge_dormant);
    c.d(&format!("census/{p}/rw_calls"), cs.rw_calls);
    c.d(&format!("census/{p}/rw_one"), cs.rw_one);
    c.d(&format!("census/{p}/rw_interior"), cs.rw_interior);
    c.d(&format!("census/{p}/rw_zero"), cs.rw_zero);
    c.d(&format!("census/{p}/lo_floor_hits"), cs.lo_floor_hits);
    // THE THREE-ARM HIGH WALL, compared rather than merely summed — a partition check passes
    // identically whether the third arm binds or is ABSENT FROM THE SOURCE.
    c.d(&format!("census/{p}/hi_wall_literal"), cs.hi_wall_literal);
    c.d(&format!("census/{p}/hi_wall_map"), cs.hi_wall_map);
    c.d(&format!("census/{p}/hi_wall_hi0"), cs.hi_wall_hi0);
    c.d(&format!("census/{p}/interp_low"), cs.interp_low);
    c.d(&format!("census/{p}/interp_mid"), cs.interp_mid);
    c.d(&format!("census/{p}/interp_high"), cs.interp_high);
    c.d(&format!("census/{p}/illinois_calls"), ill.0);
    c.d(&format!("census/{p}/illinois_evals"), ill.1);
    c.d(&format!("census/{p}/illinois_exhausted"), ill.2);

    // THE DEAD ARMS, and the ones no Python wrapper can see.
    assert_eq!(cs.march_in_advances, 0, "{p}: the low-wall march-in is DEAD on CPG");
    assert_eq!(cs.march_in_refusal, 0, "{p}");
    assert_eq!(cs.march_in_inverse, 0, "{p}");
    assert_eq!(cs.march_in_offmap, 0, "{p}");
    assert_eq!(cs.march_in_other, 0, "{p}: an UNCLASSIFIED fourth arm fired");
    assert_eq!(cs.close_bracket_fails, bracket_fails, "{p}");
    assert_eq!(cs.eq_damped, 0, "{p}: the Newton damper is DEAD");
    assert_eq!(cs.eq_damp_floor, 0, "{p}");
    assert_eq!(cs.eq_exhausted, 0, "{p}");
    assert_eq!(cs.march_break_k1, 0, "{p}: both truncation arms are DEAD");
    assert_eq!(cs.march_break_rk, 0, "{p}");
    assert_eq!(cs.interp_fallthrough, 0, "{p}: `_interp`'s fall-through is DEAD");
    assert_eq!(cs.cap_fallthrough, 0, "{p}: `cap`'s fall-through is DEAD");
    assert_eq!(cs.collapse_nan, 0, "{p}: the 9e9 NaN guard is DEAD");
    assert_eq!(cs.collapse_empty, 0, "{p}: the `if sp else nan` fall-back is DEAD");
    assert_eq!(cs.topping_skips, 0, "{p}");
    assert_eq!(cs.topping_exhausted, 0, "{p}");
    assert_eq!(cs.sched_skips, 0, "{p}");
    assert_eq!(cs.surge_skips, 0, "{p}");
    assert_eq!(cs.mf_floor_hits, 0, "{p}");
    assert_eq!(cs.der_caps_1 + cs.der_caps_2 + cs.der_caps_3 + cs.der_resolves, 0,
               "{p}: NO phase-6 gate arms a limiter keyword, so `der` builds ZERO caps");

    // THE SHAPE CLAIMS PYTHON CANNOT MAKE. `der_calls` counts a local closure Python offers no
    // handle on, so instead of a copied number it is tied to keys that ARE compared: RK4 runs
    // four `der` per loop iteration and each iteration appends exactly one point, and
    // `release_weight` is called unconditionally once per `der`.
    assert_eq!(cs.der_calls, 4 * cs.march_points,
               "{p}: RK4 runs four `der` per marched point");
    assert_eq!(cs.rw_calls, cs.der_calls,
               "{p}: `release_weight` is called unconditionally, once per `der`");
    // …and the relation between the COUNTER and what the caller actually receives: `march_points`
    // counts appends inside the marcher, `npts_here` is the sum of the `traj.len()` values this
    // section handed out. They can disagree — a marcher that appends and then truncates, or a
    // method that returns a trajectory other than the one it marched, moves one and not the
    // other. It is NOT the `round_ties_even` gate: both sides come off the same march, so a naive
    // `f64::round` moves them together. That gate is the `npts` KEYS, compared against Python's.
    //
    // **SECTION F PASSES `None`, AND THAT IS A DISCLOSED HOLE RATHER THAN A SATISFIED CHECK.**
    // `freeze_channels` runs THREE marches per cell and reports only their peak temperatures, so
    // there is no length to publish. The first draft recomputed the expected count locally as
    // `((r + s_settle)/ds).round_ties_even() + 1` and compared THAT — which is the very
    // expression a naive-`round` port would get wrong, so the two sides moved in lockstep and the
    // assertion could not fail. F's lengths are gated by `census/F/march_points` against PYTHON's
    // own counter, and by nothing local; G, H and J carry the length gate that has teeth.
    if let Some(n) = npts_here {
        assert_eq!(cs.march_points, n,
                   "{p}: march_points must equal the sum of the lengths this section published");
    }
    assert_eq!(cs.hi_wall_literal + cs.hi_wall_map + cs.hi_wall_hi0, cs.close_calls,
               "{p}: the high wall's three arms must partition the closure calls");
    assert!(cs.close_g_evals >= 2 * (cs.close_calls - cs.close_bracket_fails),
            "{p}: a successful closure evaluates `g` at least twice");

    t.close_calls += cs.close_calls;
    t.march_calls += cs.march_calls;
    t.march_points += cs.march_points;
    t.der_calls += cs.der_calls;
    t.rw_calls += cs.rw_calls;
    t.npts_sum += npts_here.unwrap_or(cs.march_points);
    t.hi_literal += cs.hi_wall_literal;
    t.hi_map += cs.hi_wall_map;
    t.hi_hi0 += cs.hi_wall_hi0;
    t.eq_calls += cs.eq_calls;
    t.eq_passes += cs.eq_passes;
    t.collapse_ties += cs.collapse_ties;
    t.sections += 1;
}

/// `(0.25 + 8.0)/0.02 = 412.5` EXACTLY — `round_ties_even` gives 412 and `f64::round` 413. A
/// march on the tie is counted so the exposure has a number, because § 5.16 sized it at 21 of 162
/// on a grid neither suite runs.
fn on_tie(s_end: f64, ds: f64) -> bool {
    (s_end / ds).fract() == 0.5
}

// ==================================================================================== the sweep
#[allow(clippy::too_many_lines)]
fn sweep_main(c: &mut Cmp) -> Totals {
    let fl = flight();
    let mut t = Totals::default();
    fcount::reset();
    let _ = take_illinois();
    let _ = tcount::take();

    let d43 = design(gas43());
    let d45 = design(gas45());

    // -------------------------------------------------------------- A: the two suites' GASES
    for (tag, g) in [("r43", gas43()), ("r45", gas45())] {
        c.f(&format!("A/{tag}/R_c"), g.r_c());
        c.f(&format!("A/{tag}/R_t"), g.r_t_at(0.0));
        c.f(&format!("A/{tag}/cp_c"), g.cp_c_at(300.0));
        c.f(&format!("A/{tag}/gamma_c"), g.gamma_c_at(300.0));
    }
    fcount::reset();
    let _ = take_illinois();
    let _ = tcount::take();
    let f43 = core(&d43, lp_shaped(), hp_shaped(), 1.0);
    let f45 = core(&d45, lp_shaped(), hp_shaped(), 1.0);
    for (tag, f) in [("r43", &f43), ("r45", &f45)] {
        let i = f.instant_fuel(&fl, 1.0, 1.0, 0.020);
        c.f(&format!("A/{tag}/sp_thrust"), i.base.sp_thrust);
        c.f(&format!("A/{tag}/Tt4"), i.base.tt4);
        c.f(&format!("A/{tag}/nu_lpt"), i.base.nu_lpt);
    }
    census(c, "A", &mut t, Some(0), 0);

    // ----------------------------------------------- B: RUNG 43 GATE 1 — control invariance
    for (it, tt4) in [1500.0f64, 1300.0, 1100.0].iter().enumerate() {
        let eq = f43.inner.equilibrium(&fl, *tt4);
        let mf = eq.close.f * eq.close.mdot_air;
        c.f(&format!("B/{it}/mf"), mf);
        for (k, v) in [
            ("nu_lp", eq.nu_lp), ("nu_hp", eq.nu_hp), ("pi_lpc", eq.close.pi_lpc),
            ("pi_hpc", eq.close.pi_hpc), ("Tt4", eq.tt4), ("mdot_air", eq.close.mdot_air),
            ("f", eq.close.f), ("tau_hpt", eq.tau_hpt), ("tau_lpt", eq.tau_lpt),
            ("sp_thrust", eq.sp_thrust),
        ] {
            c.f(&format!("B/{it}/eq40/{k}"), v);
        }
        let (inst, passes) = f43.equilibrium_fuel(&fl, mf, None);
        put_eqf(c, &format!("B/{it}/eqf"), &inst, passes);
    }
    census(c, "B", &mut t, Some(0), 0);

    // --------------------------------------------------- C: RUNG 43 GATE 2 — lp_disabled
    let single43 = build_turbojet(gas43(), PI_HPC, TT4, 50_000.0, SINGLE43);
    let st = SpoolTransient::new(single43.clone(), fl, 1.0, hp_shaped());
    let deg = TwoSpoolFuelTransient::lp_disabled(single43, fl, 1.0, hp_shaped());
    fcount::reset();
    let _ = take_illinois();
    let _ = tcount::take();
    for (it, tt4) in [1500.0f64, 1300.0, 1150.0].iter().enumerate() {
        let mf = st.fuel_for_tt4(&fl, *tt4, None);
        c.f(&format!("C/{it}/mf"), mf);
        let a = st.equilibrium_fuel(&fl, mf, None);
        let b = deg.equilibrium_fuel_lp_disabled(&fl, mf, None);
        for (k, x, y) in [
            ("nu", a.nu, b.nu), ("pi_c", a.pi_c, b.pi_c), ("Tt4", a.tt4, b.tt4),
            ("mdot_air", a.mdot_air, b.mdot_air), ("f", a.f, b.f), ("tau_t", a.tau_t, b.tau_t),
            ("sp_thrust", a.sp_thrust, b.sp_thrust),
        ] {
            c.f(&format!("C/{it}/rung35/{k}"), x);
            c.f(&format!("C/{it}/deg/{k}"), y);
        }
        // Python's dump records `-1` here: the degenerate forward runs rung 35's own solve, which
        // never touches this class's Newton, so there is no pass count to recover.
        c.d(&format!("C/{it}/deg_passes"), u64::MAX);
    }
    census(c, "C", &mut t, Some(0), 0);

    // ------------------------------------- D: RUNG 43 GATE 3 — rung 40's control untouched
    let t40 = TwoSpoolTransientCore::new(d43.clone(), fl, 1.0, lp_shaped(), hp_shaped(), 1.0);
    put_cs(c, "D/exercise", &f43.constant_speed_excursion_fuel(&fl, LO, HI));
    for (it, tt4) in [1500.0f64, 1300.0, 1150.0].iter().enumerate() {
        let a = t40.equilibrium(&fl, *tt4);
        let b = f43.inner.equilibrium(&fl, *tt4);
        for (k, x, y) in [
            ("nu_lp", a.nu_lp, b.nu_lp), ("nu_hp", a.nu_hp, b.nu_hp),
            ("pi_lpc", a.close.pi_lpc, b.close.pi_lpc),
            ("pi_hpc", a.close.pi_hpc, b.close.pi_hpc), ("Tt4", a.tt4, b.tt4),
            ("mdot_air", a.close.mdot_air, b.close.mdot_air), ("f", a.close.f, b.close.f),
            ("tau_hpt", a.tau_hpt, b.tau_hpt), ("tau_lpt", a.tau_lpt, b.tau_lpt),
            ("sp_thrust", a.sp_thrust, b.sp_thrust),
        ] {
            c.f(&format!("D/{it}/t40/{k}"), x);
            c.f(&format!("D/{it}/ft/{k}"), y);
        }
    }
    census(c, "D", &mut t, Some(0), 0);

    // ------------------------------------------- E: RUNG 43 GATE 4 — the DYNAMICAL reduce
    let mf_hi = f43.fuel_for_tt4(&fl, HI);
    let eq_hi = f43.inner.equilibrium(&fl, HI);
    let eq_lo = f43.inner.equilibrium(&fl, LO);
    c.f("E/mf_hi", mf_hi);
    for (k, x, y) in [("nu_lp", eq_hi.nu_lp, eq_lo.nu_lp), ("nu_hp", eq_hi.nu_hp, eq_lo.nu_hp),
                      ("Tt4", eq_hi.tt4, eq_lo.tt4)] {
        c.f(&format!("E/eq_hi/{k}"), x);
        c.f(&format!("E/eq_lo/{k}"), y);
    }
    let traj = f43.integrate_fuel(&fl, |_s| mf_hi, (eq_lo.nu_lp, eq_lo.nu_hp), 14.0, DS,
                                  &FuelLimiters::default());
    c.d("E/npts", traj.len() as u64);
    put_point(c, "E/first", &traj[0]);
    put_point(c, "E/last", traj.last().expect("a non-empty settle march"));
    let mut ip = 0usize;
    while ip < traj.len() {
        put_point(c, &format!("E/at/{ip}"), &traj[ip]);
        ip += 100;
    }
    census(c, "E", &mut t, Some(traj.len() as u64), 0);

    // ---------------------------------------------- F: RUNG 43 GATE 5 — freeze_channels
    for name in ["flow/press", "tilted"] {
        let (_, ml, mh) = shapes43().into_iter().find(|s| s.0 == name).expect("a named shape");
        for rho in [0.5f64, 1.0, 2.0] {
            let g = core(&d43, ml, mh, rho);
            for r in [0.25f64, 1.0] {
                let fc = g.freeze_channels(&fl, LO, HI, r, S_SETTLE43, DS);
                put_fc(c, &format!("F/{name}/{}/{}", fk(rho), fk(r)), &fc);
                // THREE marches per cell, and `freeze_channels` reports only their peaks — the
                // one place in this file where a length cannot be published. `on_tie` asks only
                // whether `s_end/ds` has a fractional half, which is a property of the ARGUMENTS
                // and not of the rounding rule, so this stays a real count.
                if on_tie(r + S_SETTLE43, DS) {
                    t.tie_marches += 3;
                }
            }
        }
    }
    census(c, "F", &mut t, None, 0);

    // ---------------------------------------- G: RUNG 43 GATE 6 — the rho-free ceiling
    let mut g_npts = 0u64;
    for name in ["flow/press", "tilted"] {
        let (_, ml, mh) = shapes43().into_iter().find(|s| s.0 == name).expect("a named shape");
        for r in [0.25f64, 1.0] {
            for rho in [1.0f64, 7.0, 50.0] {
                let e = core(&d43, ml, mh, rho)
                    .ramp_excursion_fuel(&fl, LO, HI, r, Some(Spool::Lp), S_SETTLE43, DS);
                g_npts += e.traj.len() as u64;
                t.tie_marches += u64::from(on_tie(r + S_SETTLE43, DS));
                put_ramp(c, &format!("G/{name}/{}/ceil/{}", fk(r), fk(rho)), &e);
            }
            for rho in [1.0f64, 8.0, 32.0] {
                let e = core(&d43, ml, mh, rho)
                    .ramp_excursion_fuel(&fl, LO, HI, r, None, S_SETTLE43, DS);
                g_npts += e.traj.len() as u64;
                t.tie_marches += u64::from(on_tie(r + S_SETTLE43, DS));
                put_ramp(c, &format!("G/{name}/{}/free/{}", fk(r), fk(rho)), &e);
            }
        }
    }
    census(c, "G", &mut t, Some(g_npts), 0);

    // ------------------------------------------- H: RUNG 43 GATE 7 — rho-monotonicity
    let mut h_npts = 0u64;
    for (name, ml, mh) in shapes43() {
        for r in [0.25f64, 1.0] {
            for rho in [0.25f64, 0.5, 1.0, 2.0, 4.0] {
                let e = core(&d43, ml, mh, rho)
                    .ramp_excursion_fuel(&fl, LO, HI, r, None, S_SETTLE43, DS);
                h_npts += e.traj.len() as u64;
                t.tie_marches += u64::from(on_tie(r + S_SETTLE43, DS));
                put_ramp(c, &format!("H/{name}/{}/{}", fk(r), fk(rho)), &e);
            }
        }
    }
    census(c, "H", &mut t, Some(h_npts), 0);

    // ------------------------------- I: RUNG 43 GATE 8 — the r -> 0 limit, EXACTLY rho-free
    for (name, ml, mh) in shapes43() {
        put_cs(c, &format!("I/{name}/base"),
               &core(&d43, ml, mh, 1.0).constant_speed_excursion_fuel(&fl, LO, HI));
        put_cs(c, &format!("I/{name}/rho0.2"),
               &core(&d43, ml, mh, 0.2).constant_speed_excursion_fuel(&fl, LO, HI));
        put_cs(c, &format!("I/{name}/rho5.0"),
               &core(&d43, ml, mh, 5.0).constant_speed_excursion_fuel(&fl, LO, HI));
    }
    census(c, "I", &mut t, Some(0), 0);

    // ------------------------------- J: RUNG 43 GATE 9 — the withdrawn clock, ON THE TIE
    let (_, ml, mh) = shapes43()[0];
    let mut pts: Vec<(f64, f64, RampExcursionFuel)> = Vec::new();
    let mut j_npts = 0u64;
    for rho in [0.25f64, 1.0, 4.0, 8.0] {
        let g = core(&d43, ml, mh, rho);
        for r in [0.25f64, 0.5, 1.0, 2.0] {
            let e = g.ramp_excursion_fuel(&fl, LO, HI, r, None, S_SETTLE43, DS);
            j_npts += e.traj.len() as u64;
            t.tie_marches += u64::from(on_tie(r + S_SETTLE43, DS));
            put_ramp(c, &format!("J/{}/{}", fk(rho), fk(r)), &e);
            if e.complete {
                pts.push((r, rho, e));
            }
        }
    }
    c.d("J/n_points", pts.len() as u64);
    for (key, pick) in [
        ("E_temp_H", (|e: &RampExcursionFuel| e.e_temp_h) as fn(&RampExcursionFuel) -> f64),
        ("X", |e| e.x),
        ("E_temp_L", |e| e.e_temp_l),
    ] {
        let rows: Vec<(f64, f64, f64)> =
            pts.iter().map(|(r, rho, e)| (*r, *rho, pick(e))).collect();
        let (q, sp) = FuelTransientCore::collapse_exponent(&rows, 6, None);
        c.f(&format!("J/collapse/{key}/q"), q);
        c.f(&format!("J/collapse/{key}/spread"), sp);
        for i in 0..25u32 {
            let (_, s_i) =
                FuelTransientCore::collapse_exponent(&rows, 6, Some(f64::from(i) / 20.0));
            c.f(&format!("J/collapse/{key}/score/{i}"), s_i);
        }
    }
    census(c, "J", &mut t, Some(j_npts), 0);

    // ============================================================ RUNG 45
    // ------------------------------------ K: GATE 1 — read-only, and the lp_disabled refusals
    let mut k_npts = 0u64;
    for name in ["flow/press", "tilted"] {
        let (_, ml, mh) = shapes45().into_iter().find(|s| s.0 == name).expect("a named shape");
        let bare = core(&d45, ml, mh, 1.0);
        let armed = core(&d45, ml.with_phi_surge(0.70), mh.with_phi_surge(0.55), 1.0);
        for (tag, o) in [("bare", &bare), ("armed", &armed)] {
            let mfh = o.fuel_for_tt4(&fl, HI);
            let e0 = o.inner.equilibrium(&fl, LO);
            let tr = o.integrate_fuel(&fl, |_s| mfh, (e0.nu_lp, e0.nu_hp), 2.0, DS,
                                      &FuelLimiters::default());
            k_npts += tr.len() as u64;
            c.d(&format!("K/{name}/{tag}/npts"), tr.len() as u64);
            put_point(c, &format!("K/{name}/{tag}/last"), tr.last().expect("a march"));
            let (inst, passes) = o.equilibrium_fuel(&fl, mfh, None);
            put_eqf(c, &format!("K/{name}/{tag}/eqf"), &inst, passes);
        }
    }
    // The ARMED degenerate object: both rung-45 methods must refuse it, and WHICH refusal escapes
    // is the only thing there is to compare. (The UNARMED one is a disclosed divergence — see
    // `the_unarmed_degenerate_refusal_is_a_disclosed_divergence`.)
    let dega = TwoSpoolFuelTransient::lp_disabled(
        build_turbojet(gas45(), PI_HPC, TT4, 50_000.0, SINGLE43), fl, 1.0,
        hp_shaped().with_phi_surge(0.55));
    c.d("K/deg/armed/phi_excursion_fuel", refusal_kind(|| {
        dega.phi_excursion_fuel(&fl, LO45, HI45, 0.5, S_SETTLE45, DS, None, None, None, None);
    }));
    c.d("K/deg/armed/transient_surge_margin_fuel", refusal_kind(|| {
        dega.transient_surge_margin_fuel(&fl, LO45, HI45, 0.5, S_SETTLE45, DS, None, None, None,
                                         None);
    }));
    census(c, "K", &mut t, Some(k_npts), 0);

    // ------------------------------- L: GATE 2 — the four shapes, PER-CELL census
    for (name, ml, mh) in shapes45() {
        let o = core(&d45, ml, mh, 1.0);
        let tt = TwoSpoolTransientCore::new(d45.clone(), fl, 1.0, ml, mh, 1.0);
        let acc = o.phi_excursion_fuel(&fl, LO45, HI45, 0.5, S_SETTLE45, DS, None, None, None,
                                       None);
        put_exc(c, &format!("L/{name}/acc"), &acc);
        census(c, &format!("L/{name}/acc"), &mut t, Some(acc.npts as u64), 0);
        let dec = o.phi_excursion_fuel(&fl, HI45, LO45, 0.5, S_SETTLE45, DS, None, None, None,
                                       None);
        put_exc(c, &format!("L/{name}/dec"), &dec);
        census(c, &format!("L/{name}/dec"), &mut t, Some(dec.npts as u64), 0);
        // RUNG 44, SAME maps — a DELTA against rung 45's ENDPOINT, and rung 44's own `s_end = 3.0`.
        let ex44 = tt.phi_excursion(&fl, LO45, DTT4, 0.5, 3.0, DS);
        for (k, v) in [("ext_lp", ex44.ext_lp), ("ext_hp", ex44.ext_hp),
                       ("min_phi_lp", ex44.min_phi_lp), ("min_phi_hp", ex44.min_phi_hp)] {
            c.f(&format!("L/{name}/t40/{k}"), v);
        }
        // Spelled `npts44` so it cannot join the fuel marcher's own point sum: it comes off a
        // DIFFERENT class's marcher and `march_points` never sees it.
        c.d(&format!("L/{name}/t40/npts44"), ex44.npts as u64);
        census(c, &format!("L/{name}/t40"), &mut t, Some(0), 0);
    }

    // ------------------------------------------- M: GATE 3(a) — the five-rho sweep
    let mut m_npts = 0u64;
    for rho in [0.2f64, 0.5, 1.0, 2.0, 5.0] {
        let e = core(&d45, lp_shaped(), hp_shaped(), rho)
            .phi_excursion_fuel(&fl, LO45, HI45, 0.5, S_SETTLE45, DS, None, None, None, None);
        m_npts += e.npts as u64;
        put_exc(c, &format!("M/{}", fk(rho)), &e);
    }
    census(c, "M", &mut t, Some(m_npts), 0);

    // ------------------ N: GATE 3(b) — the 19-point running line and the OUTPUT reference
    let ftg = core(&d45, lp_shaped(), hp_shaped(), 1.0);
    let grid: Vec<f64> = (0..19).map(|k| 1000.0 + 50.0 * f64::from(k)).collect();
    let ys_l: Vec<f64> = grid.iter().map(|&t| ftg.inner.equilibrium(&fl, t).close.phi_lp).collect();
    for (ik, y) in ys_l.iter().enumerate() {
        c.f(&format!("N/grid/{ik}"), *y);
    }
    census(c, "N/grid", &mut t, Some(0), 0);

    // Python's gate writes its OWN interpolator here rather than calling `_interp`, so this is a
    // transcription of the GATE and not a reuse of the plant's — the two clamp identically and
    // scan differently, and copying `_interp` in would be a different computation.
    let interp = |x: f64| -> f64 {
        if x <= grid[0] {
            return ys_l[0];
        }
        if x >= grid[grid.len() - 1] {
            return ys_l[ys_l.len() - 1];
        }
        for i in 0..grid.len() - 1 {
            if grid[i] <= x && x <= grid[i + 1] {
                let u = (x - grid[i]) / (grid[i + 1] - grid[i]);
                return ys_l[i] + u * (ys_l[i + 1] - ys_l[i]);
            }
        }
        ys_l[ys_l.len() - 1]
    };

    for rho in [0.2f64, 1.0, 5.0] {
        let o = core(&d45, lp_shaped(), hp_shaped(), rho);
        let e = o.phi_excursion_fuel(&fl, LO45, HI45, 0.5, S_SETTLE45, DS, None, None, None, None);
        put_exc(c, &format!("N/{}/exc", fk(rho)), &e);
        let mf_lo = o.fuel_for_tt4(&fl, LO45);
        let mf_hi2 = o.fuel_for_tt4(&fl, HI45);
        c.f(&format!("N/{}/mf_lo", fk(rho)), mf_lo);
        c.f(&format!("N/{}/mf_hi", fk(rho)), mf_hi2);
        let eq0 = o.inner.equilibrium(&fl, LO45);
        let sched = |s: f64| mf_lo + (mf_hi2 - mf_lo) * 1.0f64.min(s / 0.5);
        let tr = o.integrate_fuel(&fl, sched, (eq0.nu_lp, eq0.nu_hp), 6.5, DS,
                                  &FuelLimiters::default());
        c.d(&format!("N/{}/out/npts", fk(rho)), tr.len() as u64);
        let (mut oe, mut i_oe) = (0.0f64, usize::MAX);
        for (ip, p) in tr.iter().enumerate() {
            let e_lp = p.phi_lp - interp(p.tt4);
            if e_lp.abs() > oe.abs() {
                oe = e_lp;
                i_oe = ip;
            }
        }
        c.f(&format!("N/{}/out_ext", fk(rho)), oe);
        c.d(&format!("N/{}/i_out_ext", fk(rho)), i_oe as u64);
        put_point(c, &format!("N/{}/out_at", fk(rho)), &tr[i_oe]);
        census(c, &format!("N/{}", fk(rho)), &mut t, Some(e.npts as u64 + tr.len() as u64), 0);
    }

    // ------------------------- O: GATE 4 — fuel vs Tt4 control at three ramp rates
    let o45 = core(&d45, lp_shaped(), hp_shaped(), 1.0);
    let tt45 = TwoSpoolTransientCore::new(d45.clone(), fl, 1.0, lp_shaped(), hp_shaped(), 1.0);
    let mut o_npts = 0u64;
    for r in [1.0f64, 0.5, 0.3] {
        let e = o45.phi_excursion_fuel(&fl, LO45, HI45, r, S_SETTLE45, DS, None, None, None, None);
        o_npts += e.npts as u64;
        put_exc(c, &format!("O/fuel/{}", fk(r)), &e);
        let ex44 = tt45.phi_excursion(&fl, LO45, DTT4, r, 3.0, DS);
        c.f(&format!("O/t40/{}/min_phi_lp", fk(r)), ex44.min_phi_lp);
        c.d(&format!("O/t40/{}/npts44", fk(r)), ex44.npts as u64);
    }
    census(c, "O", &mut t, Some(o_npts), 0);

    // ------------------------------------------- P: GATE 5 — THE `npts` GATE
    // Step 3 finding 3 measured `min_phi_lp` bit-identical at all four rates with the march
    // bound's `r` deleted, and only these four lengths (351/326/316/306 against a flat 301)
    // witnessing it.
    let mut p_npts = 0u64;
    for r in [1.0f64, 0.5, 0.3, 0.1] {
        let e = o45.phi_excursion_fuel(&fl, LO45, HI45, r, S_SETTLE45, DS, None, None, None, None);
        p_npts += e.npts as u64;
        put_exc(c, &format!("P/{}", fk(r)), &e);
    }
    census(c, "P", &mut t, Some(p_npts), 0);

    // ------------------------------------------------- Q: GATE 6 — the crossing
    let oq = core(&d45, lp_shaped().with_phi_surge(0.746), hp_shaped().with_phi_surge(0.55), 1.0);
    let sm = oq.transient_surge_margin_fuel(&fl, LO45, HI45, 0.3, S_SETTLE45, DS, None, None,
                                            None, None);
    put_sm(c, "Q/acc", &sm);
    let obare = core(&d45, lp_shaped(), hp_shaped(), 1.0);
    c.d("Q/unarmed", refusal_kind(|| {
        obare.transient_surge_margin_fuel(&fl, LO45, HI45, 0.5, S_SETTLE45, DS, None, None, None,
                                          None);
    }));
    census(c, "Q", &mut t, Some(sm.npts as u64), 0);

    // ------------------------- R: GATE 1/cycle's own fuel-path calls
    let or = core(&d45, lp_shaped().with_phi_surge(0.60), hp_shaped().with_phi_surge(0.55), 1.0);
    let e = or.phi_excursion_fuel(&fl, LO45, 1300.0, 0.5, S_SETTLE45, DS, None, None, None, None);
    put_exc(c, "R/exc", &e);
    let sm = or.transient_surge_margin_fuel(&fl, LO45, 1300.0, 0.5, S_SETTLE45, DS, None, None,
                                            None, None);
    put_sm(c, "R/sm", &sm);
    census(c, "R", &mut t, Some(e.npts as u64 + sm.npts as u64), 0);

    // ------------------------------------- S: the accel schedule and the two table readers
    let acc = f43.accel_schedule(&fl, LO, HI, 0.15, 5);
    c.d("S/n", acc.n_h.len() as u64);
    for (i, (nh, kappa)) in acc.n_h.iter().zip(acc.kappa.iter()).enumerate() {
        c.f(&format!("S/n_H/{i}"), *nh);
        c.f(&format!("S/kappa/{i}"), *kappa);
    }
    c.f("S/margin", acc.margin);
    let (first, last) = (acc.n_h[0], acc.n_h[acc.n_h.len() - 1]);
    for (i, nh) in [first * 0.9, first, 0.5 * (first + last), last, last * 1.1].iter().enumerate() {
        c.f(&format!("S/read/{i}"), acc.cap(*nh, 250_000.0));
    }
    census(c, "S", &mut t, Some(0), 0);

    t
}

/// The NON-CPG gases. An ADDED arm — neither suite runs one through the fuel path.
fn sweep_gas(c: &mut Cmp) -> Totals {
    let fl = flight();
    let mut t = Totals::default();
    fcount::reset();
    let _ = take_illinois();
    let _ = tcount::take();

    for (name, g) in [("tpg", Gas::thermally_perfect()), ("reacting", Gas::reacting()),
                      ("forkb", Gas::reacting_forkb())] {
        c.d(&format!("T/{name}/equilibrium_flag"), u64::from(g.is_equilibrium()));
        let d = design(g);
        let f = core(&d, lp_shaped(), hp_shaped(), 1.0);
        fcount::reset();
        let _ = take_illinois();
        let _ = tcount::take();
        for (it, tt4) in [1300.0f64, 1400.0, 1450.0, 1500.0].iter().enumerate() {
            let mf = f.fuel_for_tt4(&fl, *tt4);
            c.f(&format!("T/{name}/{it}/mf"), mf);
            let (inst, passes) = f.equilibrium_fuel(&fl, mf, None);
            put_eqf(c, &format!("T/{name}/{it}/eqf"), &inst, passes);
        }
        census(c, &format!("T/{name}/eqf"), &mut t, Some(0), 0);

        let mf0 = f.fuel_for_tt4(&fl, 1300.0);
        let mf1 = f.fuel_for_tt4(&fl, 1450.0);
        let eq0 = f.inner.equilibrium(&fl, 1300.0);
        let sched = |s: f64| mf0 + (mf1 - mf0) * 1.0f64.min(s / 0.5);
        let tr = f.integrate_fuel(&fl, sched, (eq0.nu_lp, eq0.nu_hp), 1.0, DS,
                                  &FuelLimiters::default());
        c.d(&format!("T/{name}/npts"), tr.len() as u64);
        let mut ip = 0usize;
        while ip < tr.len() {
            put_point(c, &format!("T/{name}/at/{ip}"), &tr[ip]);
            ip += 10;
        }
        put_point(c, &format!("T/{name}/last"), tr.last().expect("a march"));
        census(c, &format!("T/{name}/march"), &mut t, Some(tr.len() as u64), 0);
    }

    // ----------------------------- U: THE REFUSAL, on the gas the fuel path REJECTS
    let geq = Gas::reacting_equilibrium();
    c.d("U/equilibrium_flag", u64::from(geq.is_equilibrium()));
    let deq = design(geq);
    let feq = core(&deq, lp_shaped(), hp_shaped(), 1.0);
    fcount::reset();
    let _ = take_illinois();
    let _ = tcount::take();
    // A census PER CALL: `probe_s6.py` measured the 46 on ONE call, and a section total would let
    // the second call's advances hide inside it.
    let mfeq = feq.fuel_for_tt4(&fl, 1400.0);   // Tt4-control: allowed on EVERY gas
    c.f("U/mf_smoke", mfeq);
    {
        let cs = fcount::take();
        let ill = take_illinois();
        let _ = tcount::take();
        assert_eq!(cs.march_in_advances, 0, "Tt4-control has not reached the FUEL path yet");
        put_gas_census(c, "U/setup", &cs, ill);
    }
    let mut mi = Vec::new();
    for (it, mf) in [mfeq, 0.020, 0.017].iter().enumerate() {
        let e = feq.try_equilibrium_fuel(&fl, *mf, None)
            .expect_err("the equilibrium gas must be refused");
        c.d(&format!("U/eqf/{it}"), abort_kind(&e.0));
        let cs = fcount::take();
        let ill = take_illinois();
        let _ = tcount::take();
        mi.push((cs.march_in_advances, cs.march_in_refusal, cs.march_in_inverse,
                 cs.march_in_offmap, cs.close_bracket_fails));
        put_gas_census(c, &format!("U/{it}"), &cs, ill);
        t.close_calls += cs.close_calls;
        t.eq_calls += cs.eq_calls;
        // Accumulated even though it is 0 here — the refusals abort before a pass completes. A bar
        // read off one run must not silently stop covering a section that later starts feeding it.
        t.eq_passes += cs.eq_passes;
        t.sections += 1;
    }
    // The DIRECT poke raises the refusal ITSELF, where the two above raise the BRACKET error
    // naming a cause that is not the actual one — `_close_fuel` SWALLOWS the refusal inside its
    // scan and reports its own failure instead.
    let e = feq.try_tt4_from_f(700.0, 0.02).expect_err("the direct poke must refuse");
    c.d("U/direct", abort_kind(&e.0));
    let cs = fcount::take();
    let ill = take_illinois();
    let _ = tcount::take();
    put_gas_census(c, "U/direct", &cs, ill);

    // WHAT PYTHON CANNOT SEE, gated here against `probe_s6.py`'s instrumentation of the SHIPPED
    // Python body — and gated as the SPLIT, never as the summed 46.
    // **AND THE 38 / 8 IS A PROPERTY OF THE FUEL FLOW, NOT OF THE GAS.** § 5.16 and step 1 both
    // record the swallowed advances as one number pair; measured across three flows spanning 15 %,
    // the TOTAL moves by one and the SPLIT moves by two. Each cell is gated on its own numbers,
    // and the first is the smoke's own flow so the 46 / 38 / 8 is reproduced here independently
    // rather than quoted. *A census is a property of the grid — fourth instance in this slice.*
    assert_eq!(mi[0], (46, 38, 8, 0, 1),
               "probe_s6's own cell, `fuel_for_Tt4(1400)`: 46 swallowed advances — 38 the refusal                 and 8 `inverse: root not bracketed` out of the HPC ideal-temperature inversion");
    assert_eq!(mi[1], (46, 39, 7, 0, 1), "at mdot_fuel = 0.020 the SAME 46 splits 39 / 7");
    assert_eq!(mi[2], (47, 40, 7, 0, 1), "…and at 0.017 it is 47 = 40 / 7");
    for (i, m) in mi.iter().enumerate() {
        assert_eq!(m.1 + m.2, m.0, "cell {i}: the two live arms must PARTITION the advances");
        assert_eq!(m.3, 0, "cell {i}: the off-map guard stays dead even on this gas");
    }
    assert_eq!(cs.march_in_advances, 0, "the DIRECT poke never enters the closure at all");

    t.sections += 1;
    t.close_calls += cs.close_calls;

    t
}

/// Section U's census emitter. Separate from [`census`] because the equilibrium-gas cells are the
/// ONE place in this file where the march-in arms are non-zero, so they cannot share a body that
/// gates every one of them against zero — and because there is no march there to hang the RK4
/// shape claims on.
fn put_gas_census(c: &mut Cmp, p: &str, cs: &fcount::Census, ill: (u64, u64, u64)) {
    c.d(&format!("census/{p}/close_calls"), cs.close_calls);
    c.d(&format!("census/{p}/instant_calls"), cs.instant_calls);
    c.d(&format!("census/{p}/eq_calls"), cs.eq_calls);
    c.d(&format!("census/{p}/eq_passes"), cs.eq_passes);
    c.d(&format!("census/{p}/march_calls"), cs.march_calls);
    c.d(&format!("census/{p}/march_points"), cs.march_points);
    c.d(&format!("census/{p}/topping_calls"), cs.topping_calls);
    c.d(&format!("census/{p}/sched_calls"), cs.sched_calls);
    c.d(&format!("census/{p}/sched_dormant"), cs.sched_dormant);
    c.d(&format!("census/{p}/surge_calls"), cs.surge_calls);
    c.d(&format!("census/{p}/surge_dormant"), cs.surge_dormant);
    c.d(&format!("census/{p}/rw_calls"), cs.rw_calls);
    c.d(&format!("census/{p}/rw_one"), cs.rw_one);
    c.d(&format!("census/{p}/rw_interior"), cs.rw_interior);
    c.d(&format!("census/{p}/rw_zero"), cs.rw_zero);
    c.d(&format!("census/{p}/lo_floor_hits"), cs.lo_floor_hits);
    c.d(&format!("census/{p}/hi_wall_literal"), cs.hi_wall_literal);
    c.d(&format!("census/{p}/hi_wall_map"), cs.hi_wall_map);
    c.d(&format!("census/{p}/hi_wall_hi0"), cs.hi_wall_hi0);
    c.d(&format!("census/{p}/interp_low"), cs.interp_low);
    c.d(&format!("census/{p}/interp_mid"), cs.interp_mid);
    c.d(&format!("census/{p}/interp_high"), cs.interp_high);
    c.d(&format!("census/{p}/illinois_calls"), ill.0);
    c.d(&format!("census/{p}/illinois_evals"), ill.1);
    c.d(&format!("census/{p}/illinois_exhausted"), ill.2);
}

/// The dump's `kind_of`, for an [`Abort`](turbojet::matcher::Abort) message.
fn abort_kind(s: &str) -> u64 {
    if s.contains("non-equilibrium") {
        0
    } else if s.contains("inverse: root not bracketed") {
        1
    } else if s.contains("off-map compressor trial") {
        2
    } else if s.contains("does not bracket") {
        3
    } else if s.contains("needs a surge line on BOTH maps") {
        5
    } else if s.contains("inherently two-shaft") {
        6
    } else {
        4
    }
}

/// …and for a PANIC. Returns `u64::MAX` (the dump's `-1`) if the call did not refuse at all — a
/// gate whose expected result is a raise passes when nothing raises, so the no-raise case gets its
/// own value rather than being unrepresentable.
fn refusal_kind<F: FnOnce()>(f: F) -> u64 {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(AssertUnwindSafe(f));
    std::panic::set_hook(hook);
    match out {
        Ok(()) => u64::MAX,
        Err(e) => {
            let msg = e.downcast_ref::<String>().map(String::as_str)
                .or_else(|| e.downcast_ref::<&str>().copied())
                .unwrap_or("");
            abort_kind(msg)
        }
    }
}

// ======================================================================== the gates
/// The CPG grids of BOTH suites, bit for bit against PyPy.
#[test]
fn oracle_main_is_bit_exact_against_pypy() {
    let mut c = Cmp::new(load(ORACLE_MAIN));
    c.expect_pyonly = 2;
    let t = sweep_main(&mut c);
    c.finish();
    assert_eq!(t.der_calls, 4 * t.march_points, "RK4 over the whole sweep");
    assert_eq!(t.rw_calls, t.der_calls, "one `release_weight` per `der`, over the whole sweep");
    assert_eq!(t.march_points, t.npts_sum, "every marched point is inside a published length");
    assert_eq!(t.march_calls, 140, "the two suites' own grids — NOT probe 2's 162");
    // § 5.16 prediction 4 sized the `round_ties_even` exposure at 21 of 162 on a grid neither
    // suite runs. It is 49 of THIS file's 140 (and 52 of the 143 both suites make, the other
    // three being rung 43 gate 10's `freeze_channels`, whose cell this file folds into F).
    assert_eq!(t.tie_marches, 49, "marches landing on `8.25/0.02 = 412.5` EXACTLY");
    // Sections B (3) and K (4). Section C's three do NOT count: the degenerate forward runs rung
    // 35's own solve and never reaches this class's Newton, which is exactly why the dump's
    // wrapper had to stop counting it too.
    assert_eq!(t.eq_calls, 7, "sections B and K, and no more");
}

/// The NON-CPG gases, bit for bit against PyPy — INCLUDING the pass counts probe 3 measured
/// swinging 16-fold between interpreters. That is the sharpest single detector in the slice, and
/// it fires on the port and agrees.
#[test]
fn oracle_gas_is_bit_exact_against_pypy() {
    let mut c = Cmp::new(load(ORACLE_GAS));
    let t = sweep_gas(&mut c);
    c.finish();
    assert_eq!(t.eq_calls, 15, "twelve `equilibrium_fuel` cells plus section U's three refused");
    assert_eq!(t.eq_passes, 157,
               "the Newton pass total on the three admitted TPG gases — probe 3's amplifier, \
                gated as a NUMBER because a bit-exact value dump cannot see an iteration count");
}

/// **THE HIGH WALL'S TWO RARE ARMS LIVE IN ONE CELL OF THE TWENTY GATES.**
///
/// § 5.16 prediction 6 registered the three-arm split as 24 033 / 200 193 / 3 663 "on the dump's
/// grid" — measured on `probe_s2.py`'s cross-product, not on either suite's sweep. On the grids
/// the suites really run it is **1 398 / 228 801 / 1 210**, and every one of those 2 608 non-map
/// hits comes from `test_rung45.py`'s `hp-only` shape: `ComponentMap::flat()` has no `phi_max`
/// ceiling, so the literal `2.5` binds on the accel (1 301 of 1 304) and the decel's low fuel
/// drops `hi0` under both (1 207 of 1 304).
///
/// Step 1 finding 4 had to ADD a section to the smoke to reach `hi0` at all. It turns out one cell
/// of the source's own suite reaches it — and that cell is one shape in one gate, so a port that
/// dropped rung 43's third wall arm would pass nineteen of the twenty gates and this file's other
/// thirty-two census sections.
#[test]
fn the_high_walls_rare_arms_live_in_one_cell() {
    let mut c = Cmp::new(load(ORACLE_MAIN));
    c.expect_pyonly = 2;
    let t = sweep_main(&mut c);
    // **THE COMPARATOR IS FINISHED, NOT DISCARDED.** The first draft ran the whole sweep and then
    // dropped `c` without calling `finish()`, so every value diff AND the never-compared half were
    // silently thrown away — the exact vacuity shape step 1 finding 3 and step 2 finding 3 both
    // paid for. It costs one extra sweep; a comparator that runs and is not read costs a gate.
    c.finish();
    assert_eq!((t.hi_literal, t.hi_map, t.hi_hi0), (1398, 223890, 1210),
               "the three-arm split on the suites' own grids");
    assert_eq!(t.hi_literal + t.hi_map + t.hi_hi0, t.close_calls, "a SPLIT, never a sum");
    // …and it is ONE cell: re-run section L's four shapes alone and check the other three
    // contribute nothing, so "the rare arms are rare" cannot be satisfied by them being spread
    // thinly everywhere.
    let py = load(ORACLE_MAIN);
    let key = |n: &str, d: &str, a: &str| {
        *py.get(&format!("census/L/{n}/{d}/hi_wall_{a}")).unwrap_or_else(|| panic!("{n}/{d}/{a}"))
    };
    for n in ["flow/press", "press/flow", "tilted"] {
        for d in ["acc", "dec"] {
            assert_eq!((key(n, d, "literal"), key(n, d, "hi0")), (0, 0),
                       "{n}/{d} must take the MAP arm on every call");
        }
    }
    assert_eq!((key("hp-only", "acc", "literal"), key("hp-only", "acc", "hi0")), (1301, 3));
    assert_eq!((key("hp-only", "dec", "literal"), key("hp-only", "dec", "hi0")), (97, 1207));
}

/// **A DISCLOSED PORT DIVERGENCE, MEASURED RATHER THAN LEFT TO A SENTENCE.**
///
/// On an UNARMED `lp_disabled` object Python's `transient_surge_margin_fuel` raises the SURGE-LINE
/// assert (kind 5): its body reads `self.map_lp`/`map_hp` and checks `phi_surge` BEFORE
/// `_fuel_ramp_march`'s two-shaft refusal (kind 6) can fire. Rust raises kind 6, and that is not a
/// bug in the port — step 2 finding 4 recorded that EVERY `lp_disabled` constructor in the project
/// takes `map_hp` ALONE, so the degenerate variant has no `map_lp` to read a `phi_surge` off and
/// must refuse on degeneracy first.
///
/// Neither suite reaches the combination (`test_rung45.py`'s degenerate gate arms both maps), so
/// nothing was hidden by it. The dump records Python's answer under a `pyonly` key that the
/// comparator names and skips; this asserts the Rust's, so the divergence has a gate on BOTH sides
/// rather than a comment on one.
#[test]
fn the_unarmed_degenerate_refusal_is_a_disclosed_divergence() {
    let fl = flight();
    let deg = TwoSpoolFuelTransient::lp_disabled(
        build_turbojet(gas45(), PI_HPC, TT4, 50_000.0, SINGLE43), fl, 1.0, hp_shaped());
    assert_eq!(refusal_kind(|| {
        deg.phi_excursion_fuel(&fl, LO45, HI45, 0.5, S_SETTLE45, DS, None, None, None, None);
    }), 6, "the accel excursion refuses on DEGENERACY in both languages");
    assert_eq!(refusal_kind(|| {
        deg.transient_surge_margin_fuel(&fl, LO45, HI45, 0.5, S_SETTLE45, DS, None, None, None,
                                        None);
    }), 6, "…and so does this one in Rust, where Python reaches its surge-line assert (5) first");
    let py = load(ORACLE_MAIN);
    assert_eq!(py["K/deg/pyonly/transient_surge_margin_fuel"], 5,
               "the golden must still record Python's OTHER answer, or this note is unfalsifiable");
    assert_eq!(py["K/deg/pyonly/phi_excursion_fuel"], 6, "…and the half that DOES agree");
}

/// **THE CPython ARM — a DETECTOR with a measured sensitivity, never coverage.**
///
/// § 5.16 prediction 2 registered that the CPG half moves **0** float keys and that the TPG half
/// moves the exit pass count in **≥ 10 of 12** cells. The second half holds — **12 of 12**. The
/// first is REFUTED, and the refutation names a class rather than a fluke:
///
/// | half | float keys | moved | worst |
/// |---|---:|---:|---|
/// | CPG | 3 411 | **15** | 3.14e-16 rel — all of them `collapse_exponent`'s scored curve |
/// | TPG + refusal | 801 | **731** | median 2.6e-12, p90 5.2e-11 |
///
/// Every CPG mover is a composite of `powf` and `ln`, i.e. LIBM — the one thing a port shares with
/// neither interpreter. Everything that reaches this file THROUGH THE PLANT is bit-identical, so
/// prediction 2's intent survives and its letter does not. See [`is_libm_score`], and note that
/// `J/collapse/*/q` stays on the BIT bar: the argmin is a tie at a gap of exactly `0.000e+00`, so
/// a one-ULP move on either tied score would flip it, and it did not.
///
/// The discrete side reproduces slice R's `illinois_evals` finding at a third site: **1 of the 66
/// CPG-half iteration-count keys moves**, by one, on an inherited rung-44 call. The class is
/// exempted and its width printed.
#[test]
fn oracle_matches_cpython() {
    let cpy = load(ORACLE_CPYTHON);
    let mut c = Cmp::new(cpy);
    c.cpython = true;
    c.expect_pyonly = 2;
    let _ = sweep_main(&mut c);
    let _ = sweep_gas(&mut c);
    c.finish();

    let cpg_floats: Vec<&(String, f64)> =
        c.float_drifts.iter().filter(|(k, _)| !is_fragile_gas(k)).collect();
    let tpg_floats: Vec<&(String, f64)> =
        c.float_drifts.iter().filter(|(k, _)| is_fragile_gas(k)).collect();
    let cpg_disc = c.discrete_flips.iter().filter(|s| !is_fragile_gas(s)).count();
    let tpg_disc = c.discrete_flips.len() - cpg_disc;

    // EVERY CPG-half float mover must be inside the declared libm class, so a NEW drifter fails
    // instead of joining a tolerated crowd.
    let outside: Vec<&String> =
        cpg_floats.iter().filter(|(k, _)| !is_libm_score(k)).map(|(k, _)| k).collect();
    assert!(outside.is_empty(),
            "a CPG-half float moved OUTSIDE the libm class — the plant is supposed to be \
             bit-identical across interpreters: {outside:?}");
    assert_eq!(cpg_floats.len(), 15,
               "prediction 2 said 0; it is 15, and all 15 are `collapse_exponent`'s scored curve");
    assert_eq!(cpg_disc, 1,
               "…and exactly ONE CPG discrete key moves, an iteration count, by one");
    assert!(tpg_floats.len() > 700 && tpg_disc >= 12,
            "the TPG half is the declared fragile set and must stay LOUD, or the detector has \
             gone blind: {} floats, {} discretes", tpg_floats.len(), tpg_disc);

    let worst_cpg = cpg_floats.iter().map(|(_, d)| *d).fold(0.0f64, f64::max);
    let mut tpg: Vec<f64> = tpg_floats.iter().map(|(_, d)| *d).collect();
    tpg.sort_by(f64::total_cmp);
    println!(
        "[cpython arm] CPG half: {} of {} floats moved (worst {worst_cpg:.3e} rel, all libm), \
         {cpg_disc} discrete (iteration counts). TPG half — PUBLISHED, NEVER SUMMED WITH IT: \
         {} floats moved, median {:.3e}, p90 {:.3e}; {tpg_disc} discrete.",
        cpg_floats.len(), 3411, tpg.len(), tpg[tpg.len() / 2], tpg[tpg.len() * 9 / 10]);
}
