//! SLICE AF step 5 (b) — **THE ORACLE for rung 74**, against PyPy *and* CPython 3.14.
//!
//! `rung74.rs` ports the suite's 16 gates and adds one. Those gates are RELATIONS — *the spectrum
//! agrees to 1e-8*, *the plant does not accelerate*, *the arrest ends at saturation*. This file is
//! the VALUE seat: every number every reader publishes, plus the plant underneath them, compared
//! bit for bit against a Python golden.
//!
//! # THE THREE THINGS THE 17 GATES CANNOT DO
//!
//! 1. **Say how much moved.** A gate answers caught / not caught. Step 4 scored nineteen mutations
//!    on a 3 731-key value seat that was DELETED with its throwaway harness; this file is that seat
//!    made shipped, so the ledger survives the session that measured it.
//! 2. **Reach the plant per point.** A–F are the readers' AGGREGATES over marches, and the suite's
//!    reduce spine compares NINE of the march's THIRTY-FIVE recorded fields. AD step 5 found six
//!    drifting keys at 2 points of 1 302 only because the equivalent of section G existed.
//! 3. **See a SIGN.** Rung 74's `g_fuel`/`required_*` are unfloored projections and go NEGATIVE
//!    (21 of 341 on `demand`, 0 on `demand-latched` — step 3's leading finding). No aggregate in
//!    A–F records that; `G/*/neg/*` counts it per column per arm.
//!
//! # P2 WAS PRE-REGISTERED IN THE DUMPER BEFORE THE CPYTHON ARM RAN, **AND IT IS FALSIFIED**
//!
//! § 5.30 (iii) attributes **two of rung 74's four `sum()` calls to `forcing_openloop`** — the
//! largest share, and the only reader whose published quantity is an average over the ramp. So P2
//! named that reader's three published averages in advance, with a falsifier: *if the CPython arm
//! differs on a key outside that set and outside section G's plant keys, P2 is wrong and the cause
//! is not `sum()`.*
//!
//! **The falsifier fired.** The three named keys are bit-identical; **49 keys differ and all are
//! `demand_gains`'s charpoly readings**, whose origin is rung 72's INHERITED `_charpoly4`. See
//! [`EXEMPT`] for the mechanism and for what § 5.30 (iii) got wrong about its own census. The
//! exemption below is read off the diff, and it is labelled as such — AD's P7 and AE's
//! pre-registered exemption are the precedent for how it SHOULD have gone, not for what happened
//! here.
//!
//! # WHAT IS COMPARED, AND WHAT IS ONLY COUNTED
//!
//! Every key here is COMPUTED by the port and compared. **Nothing is read out of the golden as an
//! input** — unlike AD's section H and AE's sections K/L, this slice replays no captured argument,
//! so the success line's number is a count of comparisons with no declared-read term to subtract.
//! AE step 4's success line had been calling 5 726 golden READS "values compared"; the distinction
//! is kept alive here by stating that the term is zero rather than by omitting it.
//!
//! # THE THREE HAZARDS THIS RUNG SUPPLIES, AND WHERE EACH IS ANSWERED
//!
//! * **`None` ON BOTH SIDES AGREES PERFECTLY AND MEASURES NOTHING.** Step 4 § (a) measured four
//!   legitimately-`None` keys — every one a `first_gov`, on the two ARREST arms' two demand tags,
//!   where the plant never accelerates so the governor never takes the actuator. Their fourteen
//!   siblings are `Some`, so the presence flag DISCRIMINATES; and `Z/n_none` emits the COUNT, so
//!   a port that turned every `Option` into `None` would fail on one key rather than pass on all.
//! * **SIGNED ZERO IS INVISIBLE TO EVERY RELATIVE BAR.** Values are compared as BIT PATTERNS, so
//!   `-0.0` and `+0.0` differ; `Z/n_neg_zero` / `Z/n_pos_zero` count them, because AE step 4 found
//!   63 keys flipping `+0.0`-ness between the two goldens.
//! * **`D/cell1/why` IS THE ONLY KEY THAT WITNESSES A MESSAGE.** One of `windup_law`'s four cells
//!   raises (`demand × applied` has no interior equilibrium — § 4's finding) and its text is
//!   hashed. Step 4 § (c) measured the port's refusal messages FOUR formatting divergences wide
//!   against Python's, found only because a reader first read one as a value. **An oracle that
//!   drops this key loses that entirely**, which is why step 4 booked it here by name.

use std::collections::{BTreeMap, BTreeSet};

use turbojet::bleed_transient::LeverArm;
use turbojet::demand_coordinate::{
    build_demand_coordinate_cascade, coord_march, demand_gains, demand_law,
    flat_schedule_identity, forcing_openloop, latch_discriminator, windup_law, CoordRead,
    DemandLaw, WindupCell,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::shared_actuator::REF_LAW_DEFAULT;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_af_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_af_cpython.tsv");

/// **THE MEASURED CPython EXEMPTION — 49 NAMES, AND P2 IS FALSIFIED BY ALL OF THEM.**
///
/// P2 pre-registered `F/mean_delta_late`, `F/ratio_late`, `F/worst_rel_late` — `forcing_openloop`,
/// picked because § 5.30 (iii) attributes to it TWO of rung 74's four `sum()` calls, the largest
/// share. **Those three keys are bit-identical across the two interpreters.** Every one of the 49
/// that do differ is `demand_gains`'s, and all are `poly_gap` / `poly_scale` — the characteristic
/// polynomial's coefficients and their spread. The falsifier written into the dumper's header
/// fired exactly as stated, so the exemption below is READ OFF THE DIFF, disclosed rather than
/// dressed up as a prediction.
///
/// **AND THE MECHANISM IS NOT `sum()` AT THIS RUNG AT ALL.** Of rung 74's four calls, two are
/// `sum(1 for ...)` over a generator of ones (`engine.py:18303`, `:18459`) — INTEGER counts, which
/// no compensated summation can move, so they were never candidates; and the two that really are
/// float folds are `forcing_openloop`'s, which differ on nothing. The drift enters upstream, in
/// `_charpoly4`'s float `sum()` — **rung 72's, INHERITED** — which AD step 5 and AE step 4 each
/// measured independently as the sole origin of this crate's interpreter drift.
///
/// § 5.30 (iii) called its four call sites *"attributed rather than counted"*. They were counted:
/// the line numbers were right and nothing asked what each call SUMS. **A `sum()` census is not an
/// attribution until the summand's TYPE and CONDITIONING are in it** — `poly_gap` is a difference
/// of two nearly-equal polynomials, so an upstream ULP arrives here as a relative gap of
/// **2.48e-04**, four orders larger than the input perturbation.
const EXEMPT: [&str; 49] = [
    "B/row0/poly_scale", "B/row10/poly_gap", "B/row11/poly_gap", "B/row12/poly_gap", "B/row13/poly_gap",
    "B/row13/poly_scale", "B/row14/poly_gap", "B/row14/poly_scale", "B/row15/poly_gap", "B/row15/poly_scale",
    "B/row16/poly_scale", "B/row18/poly_scale", "B/row19/poly_gap", "B/row20/poly_gap", "B/row20/poly_scale",
    "B/row22/poly_scale", "B/row23/poly_gap", "B/row24/poly_gap", "B/row26/poly_gap", "B/row26/poly_scale",
    "B/row27/poly_gap", "B/row27/poly_scale", "B/row28/poly_gap", "B/row28/poly_scale", "B/row29/poly_gap",
    "B/row3/poly_gap", "B/row30/poly_scale", "B/row31/poly_gap", "B/row32/poly_gap", "B/row32/poly_scale",
    "B/row33/poly_gap", "B/row33/poly_scale", "B/row34/poly_scale", "B/row35/poly_gap", "B/row36/poly_gap",
    "B/row36/poly_scale", "B/row37/poly_gap", "B/row37/poly_scale", "B/row38/poly_gap", "B/row38/poly_scale",
    "B/row40/poly_gap", "B/row40/poly_scale", "B/row5/poly_gap", "B/row6/poly_gap", "B/row6/poly_scale",
    "B/row7/poly_gap", "B/row9/poly_gap", "B/worst_poly_gap", "B/worst_poly_rel",
];

// ============================================================================== the grid
//
// `tests/test_rung74.py`'s module constants, and `oracle/dump_slice_af.py`'s copy of them.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const R: f64 = 0.5;
const SETTLE: f64 = 1.2;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TT4_MAX: f64 = 1200.0;
const PHI_ARREST: f64 = 0.80;
const PHI_BOTH: f64 = 0.76;
const PHI_GOV: f64 = 0.70;
const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
/// `demand_law(..., floors=(0.80, 0.76, 0.70))` — the reader's OWN default, read off
/// `turbojet/engine.py:18038`, not off this module's three constants in some other order.
const FLOORS: [f64; 3] = [0.80, 0.76, 0.70];
const COORDS3: [&str; 3] = ["clip", "demand-latched", "demand"];
/// The reader defaults. `demand_gains` is the one whose `ds` is not `0.005`.
const DG_DS: f64 = 0.002;
const DG_EVERY: usize = 4;
const RDR_DS: f64 = 0.005;
/// Section G's stride — the dumper's `G_STRIDE`. **What it pins is one point in five plus both
/// extremes and the endpoint. No more**: AD step 5's close-out measured a hidden-point defect
/// moving 0 of 54 116 keys against a control that moved 10.
const G_STRIDE: usize = 5;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// `dump_slice_af.py`'s `rig(phi_lim, inc)` — the RECEIVER the readers are called on.
fn rig(phi_lim: f64, inc: bool) -> ScheduledStatorCore {
    let sm = phi_lim / FLOOR - 1.0;
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: if inc { None }
                    else { Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))) },
        stator_inc: if inc {
            Some(StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S)))
        } else { None },
        ..Default::default()
    };
    let m = full_of(build_demand_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm));
    m.fuel.inner.lag_coord.set("demand");
    m.fuel.inner.ref_law.set(REF_LAW_DEFAULT);
    m
}

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        if line.is_empty() { continue; }
        let (k, v) = line.split_once('\t').expect("every golden line is `key<TAB>u64`");
        assert!(out.insert(k.to_string(), v.parse::<u64>().expect("a u64")).is_none(),
                "DUPLICATE GOLDEN KEY {k}");
    }
    out
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
    bad: Vec<String>,
    cpython: bool,
    exempted: BTreeSet<String>,
    /// Every key whose two sides differ AND which is exempt, with the relative gap, so the
    /// exemption is reported with a size rather than as a word.
    drifts: Vec<(String, f64)>,
    /// **SECTION Z's THREE CENSUSES, ACCUMULATED BY THE PORT's OWN EMITTERS.** The dumper counts
    /// the same three as it runs; comparing the two is a real comparison. Reading them back out
    /// of the golden would be the golden agreeing with itself, which is § 5.30 (viii) item 1.
    n_none: usize,
    n_neg_zero: usize,
    n_pos_zero: usize,
}

impl Cmp {
    fn new(py: BTreeMap<String, u64>, cpython: bool) -> Self {
        Cmp { py, seen: BTreeSet::new(), bad: Vec::new(), cpython,
              exempted: BTreeSet::new(), drifts: Vec::new(),
              n_none: 0, n_neg_zero: 0, n_pos_zero: 0 }
    }

    fn raw(&mut self, key: &str, got: u64, discrete: bool) {
        assert!(self.seen.insert(key.to_string()), "the Rust emitted {key} twice");
        let want = match self.py.get(key) {
            Some(w) => *w,
            None => { self.bad.push(format!("{key}: NO GOLDEN (the Rust emits a key Python does \
                                             not)")); return; }
        };
        if got == want { return; }
        if self.cpython && EXEMPT.contains(&key) {
            self.exempted.insert(key.to_string());
            if !discrete {
                let (a, b) = (f64::from_bits(got), f64::from_bits(want));
                let rel = if b == 0.0 { f64::INFINITY } else { ((a - b) / b).abs() };
                self.drifts.push((key.to_string(), rel));
            }
            return;
        }
        self.bad.push(if discrete {
            format!("{key}: got {got}, want {want}  (DISCRETE — a label, a count or a flag, and \
                     no interpreter difference can move one)")
        } else {
            format!("{key}: got {:?} ({got:#x}), want {:?} ({want:#x})",
                    f64::from_bits(got), f64::from_bits(want))
        });
    }

    /// **EVERY float goes through here, including the ones inside `optf`** — which is exactly
    /// how `dump_slice_af.py`'s own `f()` counts, so the two zero censuses are comparable.
    fn f(&mut self, key: &str, got: f64) {
        let bits = got.to_bits();
        if got == 0.0 { if bits == 0 { self.n_pos_zero += 1 } else { self.n_neg_zero += 1 } }
        self.raw(key, bits, false);
    }
    fn d(&mut self, key: &str, got: usize) { self.raw(key, got as u64, true); }
    fn b(&mut self, key: &str, got: bool) { self.raw(key, got as u64, true); }
    fn s(&mut self, key: &str, got: &str) { self.raw(key, fnv1a(got), true); }

    /// **THE PRESENCE FLAG IS THE POINT, NOT THE VALUE.**
    fn optf(&mut self, key: &str, got: Option<f64>) {
        self.b(&format!("{key}?"), got.is_some());
        match got { Some(x) => self.f(key, x), None => self.n_none += 1 }
    }

    fn optd(&mut self, key: &str, got: Option<usize>) {
        self.b(&format!("{key}?"), got.is_some());
        match got { Some(x) => self.d(key, x), None => self.n_none += 1 }
    }

    fn optb(&mut self, key: &str, got: Option<bool>) {
        self.b(&format!("{key}?"), got.is_some());
        match got { Some(x) => self.b(key, x), None => self.n_none += 1 }
    }

    fn taus4(&mut self, key: &str, t: (f64, f64, f64, f64)) {
        self.f(&format!("{key}/0"), t.0);
        self.f(&format!("{key}/1"), t.1);
        self.f(&format!("{key}/2"), t.2);
        self.f(&format!("{key}/3"), t.3);
    }

    fn finish(self, arm: &str) {
        let missing: Vec<&String> = self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        let mut fail = String::new();
        if !self.bad.is_empty() {
            fail.push_str(&format!("\n{} KEYS DIFFER on the {arm} arm:\n", self.bad.len()));
            for line in self.bad.iter().take(60) { fail.push_str(&format!("  {line}\n")); }
            if self.bad.len() > 60 {
                fail.push_str(&format!("  ... and {} more\n", self.bad.len() - 60));
            }
        }
        if !missing.is_empty() {
            fail.push_str(&format!("\n{} GOLDEN KEYS THE RUST NEVER EMITTED ({arm}):\n",
                                   missing.len()));
            for k in missing.iter().take(40) { fail.push_str(&format!("  {k}\n")); }
            if missing.len() > 40 {
                fail.push_str(&format!("  ... and {} more\n", missing.len() - 40));
            }
        }
        assert!(fail.is_empty(), "{fail}");
        // THE SUCCESS LINE, AND IT SAYS WHAT THE NUMBER IS A COUNT OF. Nothing here is read out
        // of the golden as an input, so the declared-read term is ZERO rather than absent.
        eprintln!("slice AF oracle [{arm}]: {} keys COMPARED, 0 read as declared inputs, \
                   {} exempted", self.seen.len(), self.exempted.len());
        if self.cpython {
            assert_eq!(self.exempted.len(), self.drifts.len(),
                       "every exempted key on this grid is a FLOAT — a discrete key in the \
                        exemption set would be absorbed in silence, which is what this compares");
            for (k, rel) in &self.drifts {
                eprintln!("  P2 exemption {k}: relative gap {rel:.3e}");
            }
        }
    }
}

// ============================================================================== the emitters

fn emit_demand_law(cmp: &mut Cmp, tag: &str, a: &DemandLaw) {
    cmp.taus4(&format!("{tag}/taus"), a.taus);
    cmp.f(&format!("{tag}/ds"), a.ds);
    cmp.d(&format!("{tag}/floors/n"), a.floors.len());
    for (i, x) in a.floors.iter().enumerate() { cmp.f(&format!("{tag}/floors/{i}"), *x); }
    cmp.f(&format!("{tag}/Tt4_max"), a.tt4_max);
    cmp.d(&format!("{tag}/n_arms"), a.arms.len());
    for (nm, list) in [("redline_flips", &a.redline_flips), ("arrested", &a.arrested)] {
        cmp.d(&format!("{tag}/{nm}/n"), list.len());
        for (i, (inc, phi)) in list.iter().enumerate() {
            cmp.b(&format!("{tag}/{nm}/{i}/inc"), *inc);
            cmp.f(&format!("{tag}/{nm}/{i}/phi"), *phi);
        }
    }
    for (k, arm) in a.arms.iter().enumerate() {
        let p = format!("{tag}/arm{k}");
        cmp.b(&format!("{p}/inc"), arm.inc);
        cmp.f(&format!("{p}/phi_lim"), arm.phi_lim);
        cmp.f(&format!("{p}/sm"), arm.sm);
        for (c, read) in COORDS3.iter().zip([&arm.clip, &arm.latched, &arm.demand]) {
            let cp = format!("{p}/{c}");
            match read {
                CoordRead::Failed(why) => {
                    cmp.b(&format!("{cp}/failed?"), true);
                    cmp.s(&format!("{cp}/failed"), why);
                    cmp.d(&format!("{cp}/failed/len"), why.chars().count());
                }
                CoordRead::Read(v) => {
                    cmp.b(&format!("{cp}/failed?"), false);
                    cmp.d(&format!("{cp}/n"), v.n);
                    cmp.f(&format!("{cp}/max_Tt4"), v.max_tt4);
                    cmp.f(&format!("{cp}/min_phi"), v.min_phi);
                    cmp.f(&format!("{cp}/overshoot"), v.overshoot);
                    cmp.f(&format!("{cp}/breach"), v.breach);
                    cmp.d(&format!("{cp}/handovers/n"), v.handovers.len());
                    for (i, x) in v.handovers.iter().enumerate() {
                        cmp.f(&format!("{cp}/handovers/{i}"), *x);
                    }
                    cmp.optf(&format!("{cp}/first_gov"), v.first_gov);
                    cmp.b(&format!("{cp}/arrested"), v.arrested);
                    cmp.f(&format!("{cp}/max_clip"), v.max_clip);
                    cmp.d(&format!("{cp}/ic_iters"), v.ic_iters);
                }
            }
        }
        cmp.optf(&format!("{p}/dTt4_coord"), arm.dtt4_coord);
        cmp.optf(&format!("{p}/dphi_coord"), arm.dphi_coord);
        cmp.optb(&format!("{p}/holds_redline"), arm.holds_redline);
        cmp.optf(&format!("{p}/dTt4_floor"), arm.dtt4_floor);
    }
}

/// The march point's fields, split by type exactly as `dump_slice_af.py` splits them. Spelled out
/// on both sides rather than looped generically: a generic loop on one side and a hand list on the
/// other is the pair that silently drifts.
///
/// Returns `None` for the four keys a `Shared` point does not carry, which is Python's `k in p`.
#[allow(clippy::type_complexity)]
fn point_fields(p: &FuelPoint)
    -> ((f64, f64, f64, f64, f64, f64, usize, f64, f64, f64, f64, f64),
        Option<(f64, f64, f64, f64)>, Option<Regime>, &'static str, &'static str,
        Option<&'static str>) {
    match p.extra {
        PointExtra::Demand { g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res,
                             ic_order, g_fuel, g_gov, required_fuel, required_gov, authority,
                             share_law, w_fuel, w_gov, cap_fuel, cap_gov, lag_coord } =>
            ((g, required, b, b_cmd, v, v_cmd, ic_iters, ic_res, g_fuel, g_gov, required_fuel,
              required_gov),
             Some((w_fuel, w_gov, cap_fuel, cap_gov)), v_regime, authority.as_str(), ic_order,
             { let _ = lag_coord; Some(share_law) }),
        PointExtra::Shared { g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res,
                             ic_order, g_fuel, g_gov, required_fuel, required_gov, authority,
                             share_law } =>
            ((g, required, b, b_cmd, v, v_cmd, ic_iters, ic_res, g_fuel, g_gov, required_fuel,
              required_gov),
             None, v_regime, authority.as_str(), ic_order, Some(share_law)),
        _ => panic!("section G marches the SIX-STATE integrator; this point is neither `Demand` \
                     nor `Shared`, which means the dispatch left through an inherited marcher"),
    }
}

fn lag_coord_of(p: &FuelPoint) -> Option<&'static str> {
    match p.extra { PointExtra::Demand { lag_coord, .. } => Some(lag_coord), _ => None }
}

/// One float column of a march point, by the dumper's own name.
fn column(p: &FuelPoint, name: &str) -> Option<f64> {
    let ((g, required, b, b_cmd, v, v_cmd, _it, ic_res, g_fuel, g_gov, required_fuel,
          required_gov), w, _, _, _, _) = point_fields(p);
    Some(match name {
        "s" => p.s, "nu_lp" => p.nu_lp, "nu_hp" => p.nu_hp, "Tt4" => p.tt4, "f" => p.f,
        "pi_lpc" => p.pi_lpc, "pi_hpc" => p.pi_hpc, "phi_lp" => p.phi_lp, "phi_hp" => p.phi_hp,
        "mdot_air" => p.mdot_air, "sp_thrust" => p.sp_thrust, "mf" => p.mf,
        "mf_sched" => p.mf_sched, "g" => g, "required" => required, "b" => b, "b_cmd" => b_cmd,
        "v" => v, "v_cmd" => v_cmd, "ic_res" => ic_res, "g_fuel" => g_fuel, "g_gov" => g_gov,
        "required_fuel" => required_fuel, "required_gov" => required_gov,
        "w_fuel" => return w.map(|t| t.0), "w_gov" => return w.map(|t| t.1),
        "cap_fuel" => return w.map(|t| t.2), "cap_gov" => return w.map(|t| t.3),
        _ => panic!("section G has no column {name:?}"),
    })
}

const G_FLOATS: [&str; 24] = [
    "s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp", "mdot_air",
    "sp_thrust", "mf", "mf_sched", "g", "required", "b", "b_cmd", "v", "v_cmd", "ic_res",
    "g_fuel", "g_gov", "required_fuel", "required_gov",
];
const G_FLOATS_74: [&str; 4] = ["w_fuel", "w_gov", "cap_fuel", "cap_gov"];

fn drive(py: BTreeMap<String, u64>, cpython: bool, arm: &str) {
    let fl = flight();
    let mut cmp = Cmp::new(py, cpython);

    // ---------------------------------------------------------------------------- A: demand_law
    let a = demand_law(&rig(PHI_ARREST, false), &fl, LO, HI, TT4_MAX, 0.0, TAUS, &FLOORS,
                       R, SETTLE, RDR_DS, V_MAX);
    emit_demand_law(&mut cmp, "A", &a);

    // -------------------------------------------------------------------------- B: demand_gains
    let g = demand_gains(&rig(PHI_ARREST, false), &fl, LO, HI, TT4_MAX, PHI_ARREST, TAUS, false,
                         R, SETTLE, DG_DS, V_MAX, DG_EVERY);
    cmp.b("B/inc", g.inc);
    cmp.f("B/phi_lim", g.phi_lim);
    cmp.taus4("B/taus", g.taus);
    cmp.f("B/ds", g.ds);
    cmp.d("B/n", g.n);
    cmp.d("B/skipped/regime", g.skipped.0);
    cmp.d("B/skipped/switch", g.skipped.1);
    cmp.optf("B/worst_poly_gap", g.worst_poly_gap);
    cmp.optf("B/worst_poly_rel", g.worst_poly_rel);
    cmp.optf("B/worst_flip", g.worst_flip);
    cmp.optf("B/worst_keep", g.worst_keep);
    cmp.optf("B/worst_pairs_gap", g.worst_pairs_gap);
    cmp.optf("B/worst_mask_leak", g.worst_mask_leak);
    cmp.optf("B/biggest_moved", g.biggest_moved);
    cmp.optd("B/min_sign_changed", g.min_sign_changed);
    cmp.d("B/rows/n", g.rows.len());
    for (i, r) in g.rows.iter().enumerate() {
        let p = format!("B/row{i}");
        cmp.f(&format!("{p}/s"), r.s);
        cmp.s(&format!("{p}/authority"), r.authority.as_str());
        cmp.f(&format!("{p}/poly_gap"), r.poly_gap);
        cmp.f(&format!("{p}/poly_scale"), r.poly_scale);
        cmp.f(&format!("{p}/worst_flip"), r.worst_flip);
        cmp.f(&format!("{p}/worst_keep"), r.worst_keep);
        cmp.d(&format!("{p}/n_sign_changed"), r.n_sign_changed);
        cmp.f(&format!("{p}/biggest_moved"), r.biggest_moved);
        cmp.optf(&format!("{p}/mask_leak_w"), r.mask_leak_w);
        cmp.optf(&format!("{p}/mask_leak_g"), r.mask_leak_g);
        cmp.f(&format!("{p}/pairs_gap"), r.pairs_gap);
    }

    // ------------------------------------------------------------------- C: latch_discriminator
    let c = latch_discriminator(&rig(PHI_BOTH, false), &fl, LO, HI, TT4_MAX, PHI_BOTH, TAUS,
                                false, R, SETTLE, RDR_DS, V_MAX);
    cmp.b("C/inc", c.inc);
    cmp.f("C/phi_lim", c.phi_lim);
    cmp.taus4("C/taus", c.taus);
    cmp.f("C/ds", c.ds);
    cmp.d("C/n", c.n);
    cmp.f("C/slope", c.slope);
    cmp.f("C/forcing", c.forcing);
    cmp.f("C/coord_dTt4", c.coord_dtt4);
    cmp.optf("C/coord_dg_ramp", c.coord_dg_ramp);
    cmp.optf("C/coord_dg_post", c.coord_dg_post);
    cmp.optf("C/coord_dg_at_mid", c.coord_dg_at_mid);
    cmp.optf("C/forcing_ratio", c.forcing_ratio);
    cmp.f("C/floor_dTt4", c.floor_dtt4);
    cmp.optf("C/floor_dg_riding", c.floor_dg_riding);
    cmp.d("C/n_both_riding", c.n_both_riding);
    let maxes = [c.max_tt4.0, c.max_tt4.1, c.max_tt4.2];
    let mins = [c.min_phi.0, c.min_phi.1, c.min_phi.2];
    for (i, nm) in COORDS3.iter().enumerate() {
        cmp.f(&format!("C/max_Tt4/{nm}"), maxes[i]);
        cmp.f(&format!("C/min_phi/{nm}"), mins[i]);
    }

    // ----------------------------------------------------------------------------- D: windup_law
    let w = windup_law(&rig(PHI_BOTH, false), &fl, LO, HI, TT4_MAX, PHI_BOTH, TAUS, false,
                       R, SETTLE, RDR_DS, V_MAX);
    cmp.b("D/inc", w.inc);
    cmp.f("D/phi_lim", w.phi_lim);
    cmp.taus4("D/taus", w.taus);
    cmp.f("D/ds", w.ds);
    cmp.b("D/no_equilibrium_without_a_stop", w.no_equilibrium_without_a_stop);
    cmp.b("D/both_sched_exist", w.both_sched_exist);
    let names = ["demand|sched", "demand|applied",
                 "demand-latched|sched", "demand-latched|applied"];
    for (i, cell) in w.cells.iter().enumerate() {
        let p = format!("D/cell{i}");
        cmp.s(&format!("{p}/name"), names[i]);
        match cell {
            WindupCell::Absent { why } => {
                cmp.b(&format!("{p}/exists"), false);
                cmp.s(&format!("{p}/why"), why);
                cmp.d(&format!("{p}/why/len"), why.chars().count());
            }
            WindupCell::Present(v) => {
                cmp.b(&format!("{p}/exists"), true);
                cmp.d(&format!("{p}/n"), v.n);
                cmp.d(&format!("{p}/ic_iters"), v.ic_iters);
                cmp.f(&format!("{p}/ic_res"), v.ic_res);
                cmp.optf(&format!("{p}/max_masked_w"), v.max_masked_w);
                cmp.optf(&format!("{p}/max_masked_over_sched"), v.max_masked_over_sched);
                cmp.f(&format!("{p}/max_Tt4"), v.max_tt4);
            }
        }
    }

    // ------------------------------------------------------------------ E: flat_schedule_identity
    let e = flat_schedule_identity(&rig(PHI_BOTH, false), &fl, 1150.0, PHI_BOTH, TAUS, false,
                                   SETTLE, RDR_DS, V_MAX, 1200.0, 0.94);
    cmp.b("E/inc", e.inc);
    cmp.f("E/phi_lim", e.phi_lim);
    cmp.d("E/n", e.n);
    cmp.f("E/nu0/0", e.nu0.0);
    cmp.f("E/nu0/1", e.nu0.1);
    for (i, nm) in turbojet::demand_coordinate::FLAT_KEYS.iter().enumerate() {
        cmp.f(&format!("E/worst/{i}"), e.worst[i]);
        cmp.s(&format!("E/worst/{i}/name"), nm);
    }
    cmp.f("E/worst_any", e.worst_any);
    cmp.b("E/bit_identical", e.bit_identical);
    cmp.d("E/riding", e.riding);
    cmp.b("E/non_vacuous", e.non_vacuous);
    cmp.f("E/span_Tt4/0", e.span_tt4.0);
    cmp.f("E/span_Tt4/1", e.span_tt4.1);

    // ----------------------------------------------------------------------- F: forcing_openloop
    let fo = forcing_openloop(&rig(PHI_BOTH, false), &fl, LO, HI, TT4_MAX, PHI_BOTH, TAUS, false,
                              R, SETTLE, RDR_DS, V_MAX);
    cmp.b("F/inc", fo.inc);
    cmp.f("F/phi_lim", fo.phi_lim);
    cmp.taus4("F/taus", fo.taus);
    cmp.f("F/ds", fo.ds);
    cmp.d("F/n", fo.n);
    cmp.f("F/slope", fo.slope);
    cmp.f("F/predicted", fo.predicted);
    cmp.d("F/n_on_ramp", fo.n_on_ramp);
    cmp.d("F/n_post", fo.n_post);
    cmp.optf("F/mean_delta_late", fo.mean_delta_late);
    cmp.optf("F/ratio_late", fo.ratio_late);
    cmp.optf("F/worst_rel_late", fo.worst_rel_late);
    cmp.optf("F/delta_post_first", fo.delta_post_first);
    cmp.optf("F/delta_post_last", fo.delta_post_last);
    cmp.optb("F/decayed", fo.decayed);
    cmp.d("F/rows/n", fo.rows.len());
    for (i, r) in fo.rows.iter().enumerate() {
        let p = format!("F/row{i}");
        cmp.f(&format!("{p}/s"), r.s);
        cmp.b(&format!("{p}/on_ramp"), r.on_ramp);
        cmp.f(&format!("{p}/cap"), r.cap);
        cmp.f(&format!("{p}/req"), r.req);
        cmp.f(&format!("{p}/g_clip"), r.g_clip);
        cmp.f(&format!("{p}/g_dem"), r.g_dem);
        cmp.f(&format!("{p}/delta"), r.delta);
        cmp.b(&format!("{p}/riding"), r.riding);
    }

    // ------------------------------------------------------------------------------- G: THE PLANT
    for (gi, phi) in [PHI_ARREST, PHI_BOTH].iter().enumerate() {
        let sm = phi / FLOOR - 1.0;
        for coord in COORDS3 {
            let tag = format!("G/{gi}/{coord}");
            cmp.f(&format!("{tag}/phi_lim"), *phi);
            cmp.s(&format!("{tag}/coord"), coord);
            let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                coord_march(&rig(*phi, false), &fl, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE,
                            RDR_DS, V_MAX, false, coord, REF_LAW_DEFAULT, None).3
            }));
            let traj = match ran {
                Ok(t) => t,
                Err(e) => {
                    let why = match e.downcast_ref::<String>() {
                        Some(s) => s.clone(),
                        None => e.downcast_ref::<&str>().map(|s| (*s).to_string())
                                 .unwrap_or_else(|| "<non-string panic>".into()),
                    };
                    let why: String = why.chars().take(240).collect();
                    cmp.b(&format!("{tag}/marched"), false);
                    cmp.s(&format!("{tag}/why"), &why);
                    cmp.d(&format!("{tag}/why/len"), why.chars().count());
                    continue;
                }
            };
            cmp.b(&format!("{tag}/marched"), true);
            cmp.d(&format!("{tag}/n"), traj.len());
            assert!(!traj.is_empty(), "an empty march is not a measurement: {tag}");
            cmp.d(&format!("{tag}/keys"), traj[0].key_count());
            for i in (0..traj.len()).step_by(G_STRIDE) {
                let p = &traj[i];
                let q = format!("{tag}/p{i}");
                for name in G_FLOATS {
                    cmp.f(&format!("{q}/{name}"), column(p, name).expect("always present"));
                }
                for name in G_FLOATS_74 {
                    let x = column(p, name);
                    cmp.b(&format!("{q}/{name}?"), x.is_some());
                    if let Some(v) = x { cmp.f(&format!("{q}/{name}"), v); }
                }
                let ((_, _, _, _, _, _, ic_iters, ..), _, v_regime, auth, ic_order, share_law) =
                    point_fields(p);
                cmp.d(&format!("{q}/ic_iters"), ic_iters);
                cmp.s(&format!("{q}/branch"), p.branch.label());
                cmp.s(&format!("{q}/authority"), auth);
                cmp.s(&format!("{q}/share_law"), share_law.expect("six-state points carry it"));
                cmp.s(&format!("{q}/ic_order"), ic_order);
                if let Some(lc) = lag_coord_of(p) { cmp.s(&format!("{q}/lag_coord"), lc); }
                cmp.b(&format!("{q}/v_regime?"), v_regime.is_some());
                if let Some(rg) = v_regime { cmp.s(&format!("{q}/v_regime"), regime_str(rg)); }
            }
            for name in G_FLOATS.iter().chain(G_FLOATS_74.iter()) {
                let col: Vec<f64> = traj.iter().filter_map(|p| column(p, name)).collect();
                cmp.d(&format!("{tag}/col/{name}/n"), col.len());
                if col.is_empty() { continue; }
                cmp.f(&format!("{tag}/col/{name}/min"),
                      col.iter().copied().fold(f64::INFINITY, f64::min));
                cmp.f(&format!("{tag}/col/{name}/max"),
                      col.iter().copied().fold(f64::NEG_INFINITY, f64::max));
                cmp.f(&format!("{tag}/col/{name}/last"), col[col.len() - 1]);
            }
            for name in ["g_fuel", "g_gov", "required_fuel", "required_gov"] {
                let n = traj.iter().filter_map(|p| column(p, name))
                    .filter(|x| *x < 0.0).count();
                cmp.d(&format!("{tag}/neg/{name}"), n);
            }
        }
    }

    // -------------------------------------------------------------- H: A DECLARED EXTRA GRID
    for (hi_, phi) in [PHI_ARREST, PHI_BOTH, PHI_GOV].iter().enumerate() {
        let one = [*phi];
        let h = demand_law(&rig(*phi, false), &fl, LO, HI, TT4_MAX, 0.0, TAUS, &one,
                           R, SETTLE, RDR_DS, V_MAX);
        emit_demand_law(&mut cmp, &format!("H/{hi_}"), &h);
    }

    // ------------------------------------------------------------------------- Z: THE CENSUSES
    //
    // Recomputed from the GOLDEN's own key set rather than from the Rust's tallies, deliberately:
    // the count of `None`s and of signed zeros is a property of the emitted stream, and computing
    // it from the port's own emitters would be the port agreeing with itself.
    z_census(&mut cmp);

    cmp.finish(arm);
}

/// **SECTION Z — the three censuses, COMPUTED BY THE PORT and compared against the dumper's.**
///
/// `Option`-that-is-`None` agrees with `None` on every key and measures nothing, and a `+0.0`
/// where a `-0.0` belongs is invisible to every relative bar. Both are turned into a NUMBER that
/// one side computes and the other checks — which is the only form in which either is a test.
///
/// Rung 74 has EIGHT legitimately-`None` keys on this grid, every one a `first_gov` on an arrested
/// arm's demand tag (§ 5.30.4 (a) measured four over sections A–F; H's three extra `demand_law`
/// calls supply the rest). Their siblings are `Some`, so the flag DISCRIMINATES.
fn z_census(cmp: &mut Cmp) {
    let (none, neg, pos) = (cmp.n_none, cmp.n_neg_zero, cmp.n_pos_zero);
    cmp.d("Z/n_none", none);
    cmp.d("Z/n_neg_zero", neg);
    cmp.d("Z/n_pos_zero", pos);
}

// ============================================================================== the gates

#[test]
fn the_two_goldens_have_the_same_key_set() {
    let (p, c) = (load(ORACLE_PYPY), load(ORACLE_CPYTHON));
    let only_p: Vec<&String> = p.keys().filter(|k| !c.contains_key(*k)).collect();
    let only_c: Vec<&String> = c.keys().filter(|k| !p.contains_key(*k)).collect();
    assert!(only_p.is_empty() && only_c.is_empty(),
            "the two arms disagree on WHICH keys exist, which is a structural difference and not \
             an arithmetic one: PyPy-only {only_p:?}, CPython-only {only_c:?}");
    assert_eq!(p.len(), c.len());
    eprintln!("slice AF goldens: {} keys per arm", p.len());
}

/// **HOW MUCH THE TWO GOLDENS DIFFER, BY NAME** — reported before either is compared against the
/// port, because AD step 5's headline was a 5 022-key golden-vs-golden gap that the port-vs-golden
/// run could not have attributed.
#[test]
fn the_golden_diff_is_reported_by_name_before_the_port_is_blamed() {
    let (p, c) = (load(ORACLE_PYPY), load(ORACLE_CPYTHON));
    let diff: Vec<&String> = p.keys().filter(|k| p.get(*k) != c.get(*k)).collect();
    eprintln!("slice AF golden-vs-golden: {} of {} keys differ", diff.len(), p.len());
    for k in diff.iter().take(40) { eprintln!("  {k}"); }
    // **THE MEASURED SHAPE, PINNED SO IT CANNOT WIDEN IN SILENCE.** Every differing key is in
    // `EXEMPT`, every one is `demand_gains`'s, and every one is a charpoly coefficient reading.
    let unexplained: Vec<&&String> = diff.iter()
        .filter(|k| !EXEMPT.contains(&k.as_str()))
        .collect();
    assert!(unexplained.is_empty(),
            "{} golden keys differ outside the measured 49: {unexplained:?}", unexplained.len());
    assert_eq!(diff.len(), EXEMPT.len(),
               "the exemption is EXACTLY the measured diff — a name that stops differing is as \
                much a change as one that starts");
    assert!(diff.iter().all(|k| k.starts_with("B/")),
            "the drift is `demand_gains`'s ALONE — P2 named `forcing_openloop` and was wrong");
    assert!(diff.iter().all(|k| k.ends_with("poly_gap") || k.ends_with("poly_scale")
                               || *k == "B/worst_poly_gap" || *k == "B/worst_poly_rel"),
            "every differing key is a CHARPOLY reading, which is what puts the origin in rung \
             72's inherited `_charpoly4` and not in any of rung 74's own four `sum()` calls");
    // **`forcing_openloop`'s THREE PRE-REGISTERED KEYS ARE BIT-IDENTICAL** — asserted, not merely
    // absent from the list above, because "it is not in the diff" is also what a key the dumper
    // forgot to emit looks like.
    for k in ["F/mean_delta_late", "F/ratio_late", "F/worst_rel_late"] {
        assert!(p.contains_key(k), "{k}: the dumper must actually emit it");
        assert_eq!(p.get(k), c.get(k),
                   "{k}: P2 predicted this would need the exemption and it does NOT differ");
    }
}

#[test]
fn rust_equals_pypy_on_every_key() {
    drive(load(ORACLE_PYPY), false, "PyPy");
}

#[test]
fn rust_equals_cpython_except_on_the_pre_registered_exemption() {
    drive(load(ORACLE_CPYTHON), true, "CPython");
}
