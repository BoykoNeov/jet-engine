//! Rung-11 verification: the physical mixing model — a jet-entrainment quench.
//!
//! Rung 10 resolved the quench in TIME but left "how fast" a free knob (`tau_q`) with an
//! arbitrary LINEAR mixing schedule. Rung 11 asks what physically sets the quench rate: the
//! dilution air enters through JETS IN CROSSFLOW, and the mixing rate scales with the jet
//! momentum-flux ratio `J = ρ_j U_j²/(ρ_c U_c²)`. So rung 11 RETIRES both knobs —
//! `τ_q = H/(C_e·√J·U_c)` is DERIVED from J (a MEAN-FIELD entrainment rate), and the linear
//! schedule becomes a decelerating entrainment shape `β(t) = 1 − (1 − t/τ_q)^n`. "Quick quench"
//! = a high-momentum jet. Still a pure diagnostic: bit-for-bit rung 6.
//!
//! Gates (`docs/rung11-spec.md`), priority order:
//!
//! 1. **reduce-to-rung-10 (LOAD-BEARING, exact by construction)** — no `mixing` is the exact
//!    rung-9/10 path; and a `shape_n = 1` jet matches the rung-10 linear quench at the DERIVED
//!    `τ_q`, bit-for-bit.
//! 2. **the monotone J-sweep (THE lesson)** — `ei_no_quenched` falls MONOTONICALLY as J rises.
//! 3. **`τ_q ∝ 1/√J`** — the derived time; stays in the RQL sub-ms–few-ms band for physical J.
//! 4. **the schedule-shape discriminator** — at the same J a decelerating schedule makes LESS
//!    NO than linear: NO is re-made at the EARLY/low-β stoich crossing, which a decelerating
//!    entrainment clears fast.
//! 5. **cycle untouched** — a jet-mixing call must not perturb station 4.
//! 6. **clamp dormancy persists** + mutual-exclusivity + positivity guards.
//! 7. **the mean-field ceiling is a DOCUMENTED invariant** — the monotonicity assertion in
//!    gate 2 IS the statement that the mixing optimum is out of scope (rung 12's seam).

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, f_stoich, hf_fuel_default, Gas};
use turbojet::nox::{
    kcheck_ratio, primary_aft, quench_no, quench_trajectory, thermal_no, JetMixing, QuenchOpts,
    QuenchPoint, QuenchResult, ZonedNoxOpts,
};

const TAU: f64 = 3e-3;
const NG: usize = 33;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}
fn losses() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    }
}
fn opts() -> ZonedNoxOpts {
    ZonedNoxOpts { tau: TAU, quench_ngrid: NG, ..ZonedNoxOpts::default() }
}

/// The jet the Python gates use — `C_e = 0.20` rather than the default, which is what puts the
/// derived `τ_q` in the RQL band across the whole J sweep (gate 3).
fn jet(j: f64, shape_n: f64) -> JetMixing {
    JetMixing { j, c_e: 0.20, shape_n, ..JetMixing::default() }
}

fn design_point() -> (Gas, f64, f64, f64, f64) {
    let g = Gas::reacting_equilibrium();
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    (g, r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt)
}

struct Traj {
    comp: Vec<(&'static str, f64)>,
    t_p: f64,
    alpha: f64,
    n0: f64,
    tab: Vec<QuenchPoint>,
    tt3: f64,
    far: f64,
    p: f64,
}

/// The trajectory is reused VERBATIM from rung 10 — the fast chemistry is a function of β alone,
/// so a whole J/shape sweep rides one build. That reuse is not an optimisation here; it is why
/// rung 11 can sweep J at all.
fn reusable_traj(phi_p: f64) -> Traj {
    let (_g, tt3, _tt4, far, p) = design_point();
    let far_p = phi_p * f_stoich();
    let alpha = far / far_p;
    let t_p = primary_aft(far_p, p, tt3, hf_fuel_default());
    let comp = equilibrium_composition(far_p, t_p, p);
    let nox = thermal_no(&comp, t_p, p, TAU, far_p, 4000, 1.0);
    let ntot: f64 = comp.iter().map(|&(_, v)| v).sum();
    let n0 = alpha * nox.x_no * ntot;
    let tab = quench_trajectory(&comp, t_p, alpha, far, tt3, p, NG);
    Traj { comp, t_p, alpha, n0, tab, tt3, far, p }
}

impl Traj {
    /// The schedule-aware quench on the prebuilt table for a `JetMixing` (derived `τ_q` +
    /// entrainment schedule).
    fn jet_quench(&self, m: &JetMixing) -> QuenchResult {
        let sched = |x: f64| m.schedule(x);
        self.at(m.tau_q(), Some(&sched))
    }
    /// The rung-10 path: the same `τ_q`, no schedule (identity).
    fn at(&self, tau_q: f64, schedule: Option<&dyn Fn(f64) -> f64>) -> QuenchResult {
        quench_no(
            &self.comp, self.t_p, self.alpha, self.far, self.tt3, self.p, self.n0, tau_q,
            QuenchOpts { ngrid: NG, tab: Some(&self.tab), schedule, ..QuenchOpts::default() },
        )
    }
}

// --------------------------------------------------------------------------------------
// GATE 1 — reduce-to-rung-10.
// --------------------------------------------------------------------------------------

/// No `mixing` must be the EXACT rung-9/10 path.
#[test]
fn reduce_mixing_none_is_bit_for_bit_rung10() {
    let (g, tt3, tt4, far, p) = design_point();
    for phi_p in [0.8, 1.0, 1.5, 2.0] {
        let a = g.zoned_nox(far, tt3, tt4, p, phi_p, ZonedNoxOpts::default());
        let b = g.zoned_nox(far, tt3, tt4, p, phi_p, ZonedNoxOpts { mixing: None, ..opts() });
        for s in [&a, &b] {
            assert!(s.mixing.is_none() && s.tau_q.is_none() && s.ei_no_quenched.is_none());
        }
        assert_eq!(a.ei_no().to_bits(), b.ei_no().to_bits());
        assert_eq!(a.x_no_mix.to_bits(), b.x_no_mix.to_bits());
        assert_eq!(a.t_primary.to_bits(), b.t_primary.to_bits());
        assert_eq!(a.t_mix.to_bits(), b.t_mix.to_bits());
    }
}

/// A `shape_n = 1` jet is CONSTANT entrainment = rung 10's linear schedule, so at the DERIVED
/// `τ_q` it must reproduce the rung-10 quench BIT-FOR-BIT.
///
/// This is the reduce that could plausibly have failed, and the Python guards it deliberately:
/// `schedule` returns `tfrac` itself at `shape_n == 1` rather than evaluating `1 − (1−x)^1`,
/// which drifts a ULP. Compared at the derived `τ_q` — never at a round number — because the
/// contract is about the SCHEDULE, and feeding both sides a hand-picked time would let a wrong
/// `tau_q()` pass.
#[test]
fn reduce_shape_n1_matches_rung10_linear_bit_for_bit() {
    let t = reusable_traj(1.5);
    for j in [9.0, 25.0, 64.0] {
        let m = jet(j, 1.0);
        let r11 = t.jet_quench(&m);
        let r10 = t.at(m.tau_q(), None);
        assert_eq!(r11.ei.to_bits(), r10.ei.to_bits(),
                   "J={j}: shape_n=1 must be bit-for-bit rung 10 ({} vs {})", r11.ei, r10.ei);
    }
}

// --------------------------------------------------------------------------------------
// GATE 2 — the monotone J-sweep (THE lesson).
// --------------------------------------------------------------------------------------

/// Higher jet momentum → shorter DERIVED `τ_q` → the gas escapes the stoich peak faster → LESS
/// re-made NO. Monotone-DECREASING in J, and by construction there is NO optimum: this
/// assertion IS the mean-field ceiling, and rung 12 is what breaks it.
#[test]
fn j_sweep_ei_no_falls_monotonically() {
    let t = reusable_traj(1.5);
    let js = [4.0, 9.0, 16.0, 25.0, 49.0, 100.0];
    let eis: Vec<f64> = js.iter().map(|&j| t.jet_quench(&jet(j, 2.0)).ei).collect();
    for w in eis.windows(2) {
        assert!(w[1] < w[0],
                "EI_NO must fall monotonically as J rises (no optimum — mean-field): {eis:?}");
    }
    assert!(eis[0] > 3.0 * eis[5], "J=4 vs J=100 should differ by a real factor: {eis:?}");
}

// --------------------------------------------------------------------------------------
// GATE 3 — τ_q ∝ 1/√J, in the RQL band.
// --------------------------------------------------------------------------------------

/// `τ_q = H/(C_e·√J·U_c)`: 4× the momentum-flux ratio halves `τ_q`. And the DERIVED time lands
/// in the RQL sub-ms–few-ms band for physical J.
#[test]
fn derived_tau_q_scales_as_inv_sqrt_j_in_rql_band() {
    let base = jet(16.0, 2.0);
    let quad = jet(64.0, 2.0); // 4× J → √J doubles → τ_q halves
    assert!((quad.tau_q() - 0.5 * base.tau_q()).abs() < 1e-12 * base.tau_q(),
            "τ_q must scale as 1/√J: {} vs {}", base.tau_q(), quad.tau_q());
    for j in [4.0, 25.0, 100.0] {
        let tq = jet(j, 2.0).tau_q();
        assert!(3e-4 < tq && tq < 5e-3,
                "J={j}: derived τ_q {:.3} ms outside the RQL sub-ms–few-ms band", tq * 1e3);
    }
}

// --------------------------------------------------------------------------------------
// GATE 4 — schedule shape: decelerating entrainment re-makes LESS NO.
// --------------------------------------------------------------------------------------

/// At the SAME J (`τ_q` does not depend on `shape_n`), a decelerating entrainment clears the
/// EARLY/low-β stoich crossing faster than the linear schedule → LESS re-made NO. So rung 10's
/// linear schedule was CONSERVATIVE. The last assertion confirms the crossing really is at low
/// β, which is WHY the shape matters — without it the test would pass on a coincidence.
#[test]
fn decelerating_schedule_makes_less_no_than_linear() {
    let t = reusable_traj(1.5);
    let j = 25.0;
    let lin = t.jet_quench(&jet(j, 1.0)).ei; // linear = rung 10
    let dec2 = t.jet_quench(&jet(j, 2.0)).ei;
    let dec3 = t.jet_quench(&jet(j, 3.0)).ei;
    assert!(dec2 < lin, "decelerating (n=2) must re-make less than linear: {dec2:.4} vs {lin:.4}");
    assert!(dec3 < dec2, "more-decelerating (n=3) must re-make even less: {dec3:.4} vs {dec2:.4}");
    assert!(lin < 3.0 * dec3,
            "shape sensitivity should be O(few), not orders of magnitude: {lin:.4} vs {dec3:.4}");
    let mut ipk = 0usize;
    for (i, r) in t.tab.iter().enumerate() {
        if r.t > t.tab[ipk].t {
            ipk = i;
        }
    }
    let beta_pk = ipk as f64 / (t.tab.len() - 1) as f64;
    assert!(beta_pk < 0.35, "stoich crossing should be at low β, got β={beta_pk:.2}");
}

// --------------------------------------------------------------------------------------
// GATE 5 — cycle untouched.
// --------------------------------------------------------------------------------------

#[test]
fn cycle_untouched_by_jet_mixing_quench() {
    let run = || {
        build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
            .run(&flight(), 50.0)
    };
    let r1 = run();
    let (tt3, tt4, far1, p) =
        (r1.station("3").tt, r1.station("4").tt, r1.station("4").far, r1.station("4").pt);
    let g = Gas::reacting_equilibrium();
    g.zoned_nox(far1, tt3, tt4, p, 1.5,
                ZonedNoxOpts { mixing: Some(jet(25.0, 2.0)), ..opts() });
    assert_eq!(run().station("4").far.to_bits(), far1.to_bits(),
               "jet-mixing quench perturbed the cycle far — must stay rung-6");
}

// --------------------------------------------------------------------------------------
// GATE 6 — dormancy, mutual exclusivity, positivity.
// --------------------------------------------------------------------------------------

#[test]
fn clamp_dormancy_persists_over_j_sweep() {
    let mut overall = 0.0f64;
    for phi_p in [1.0, 1.5] {
        let t = reusable_traj(phi_p);
        for j in [4.0, 25.0, 100.0] {
            overall = overall.max(t.jet_quench(&jet(j, 2.0)).max_a);
        }
    }
    assert!(overall < 1.0,
            "max_a={overall:.3} ≥ 1 — the super-eq regime; the dropped clamp is now load-bearing");
}

#[test]
fn mixing_and_tau_q_mutually_exclusive() {
    let r = std::panic::catch_unwind(|| {
        let (g, tt3, tt4, far, p) = design_point();
        g.zoned_nox(far, tt3, tt4, p, 1.5,
                    ZonedNoxOpts { tau_q: Some(1e-3), mixing: Some(jet(25.0, 2.0)), ..opts() })
    });
    assert!(r.is_err(), "passing BOTH tau_q and mixing must be rejected (mutually exclusive)");
}

/// `JetMixing`'s positivity guards. Rust cannot run a constructor on struct-literal syntax, so
/// `validate()` is called at every point of use inside `zoned_nox` — which means these are
/// exercised through the same door the Python's `__post_init__` is.
#[test]
fn jetmixing_positivity_guards() {
    jet(25.0, 2.0).validate(); // defaults accepted
    let bad: [JetMixing; 6] = [
        JetMixing { j: 0.0, ..JetMixing::default() },
        JetMixing { j: -1.0, ..JetMixing::default() },
        JetMixing { j: 25.0, h: 0.0, ..JetMixing::default() },
        JetMixing { j: 25.0, u_c: -5.0, ..JetMixing::default() },
        JetMixing { j: 25.0, c_e: 0.0, ..JetMixing::default() },
        JetMixing { j: 25.0, shape_n: 0.0, ..JetMixing::default() },
    ];
    for m in bad {
        let r = std::panic::catch_unwind(move || m.validate());
        assert!(r.is_err(), "JetMixing {m:?} should be rejected (positivity guard)");
    }
}

// --------------------------------------------------------------------------------------
// GATE 7 — the K-check binds along the whole trajectory.
// --------------------------------------------------------------------------------------

#[test]
fn kcheck_binds_along_the_trajectory() {
    let t = reusable_traj(1.5);
    let ts: Vec<f64> = t.tab.iter().map(|r| r.t).collect();
    let (lo, hi) = (ts.iter().cloned().fold(f64::MAX, f64::min),
                    ts.iter().cloned().fold(f64::MIN, f64::max));
    for tt in [lo, hi] {
        let r = kcheck_ratio(tt);
        assert!(0.90 < r && r < 1.15, "K-check {r:.4} at trajectory T={tt:.0} K out of band");
    }
}
