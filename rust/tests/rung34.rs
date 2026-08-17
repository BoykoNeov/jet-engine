//! RUNG 34 — THE SPOOL TRANSIENT: `N` becomes a STATE, not an output.
//!
//! The eight gates of `tests/test_rung34.py`, in file order. All eight port; the roster at
//! [`rung34_roster`] is the record, on `rung41.rs`'s precedent — *a count nobody can re-derive is
//! how a port drops one silently*.
//!
//! | # | `tests/test_rung34.py` | here |
//! |---|---|---|
//! | 1 | `test_reduce_equilibrium_is_the_steady_matcher` | [`gate1_reduce_equilibrium_is_the_steady_matcher`] |
//! | 2 | `test_stability_running_line_is_an_attractor` | [`gate2_the_running_line_is_an_attractor`] |
//! | 3 | `test_the_finding_excursion_vs_ratio` | [`gate3_the_finding_excursion_falls_with_the_clock_ratio`] |
//! | 4 | `test_direction_shape_robust` | [`gate4_direction_is_shape_robust_and_the_loading_slope_is_necessary`] |
//! | 5 | `test_I_is_only_the_clock` | [`gate5_inertia_is_only_the_clock`] |
//! | 6 | `test_forward_backward_map_inverse` | [`gate6_the_forward_speed_line_inverts_solve_n`] |
//! | 7 | `test_spooldown_crosses_into_subsonic` | [`gate7_a_spooldown_crosses_into_the_subsonic_branch`] |
//! | 8 | `test_cycle_untouched` | [`gate8_the_design_cycle_is_untouched`] |
//!
//! **WHAT THE PORT ADDS.** Two gates have no Python counterpart and are listed in
//! [`rung34_additions`] so the name diff is symmetric in both directions:
//! [`the_hook_table_fires_and_rung31s_does_not`] (§ 5.13 prediction 2 — slice N FINDING 3's
//! unreachable hook, and step 1's discovery that this one is DEAD on the subsonic branch) and
//! [`phi_max_is_read_at_both_flow_search_caps`].
//!
//! **NO `slow` MARKER IS EARNED** — § 5.13 prediction 10. Python marks none of these eight, and
//! the whole file runs in well under a second; the measurement is printed by
//! [`rung34_roster`] rather than assumed.

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::Gas;
use turbojet::map::{ComponentMap, MapMatcher};
use turbojet::matcher::{Branch, OffDesignMatcher};
use turbojet::spool::{counters, SpoolTransient};

const PI_C: f64 = 10.0;
const TT4: f64 = 1500.0;

fn flight() -> FlightCondition {
    FlightCondition { t0: 250.0, p0: 50_000.0, m0: 0.85 }
}

fn real() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, e_t: None,
        eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
    }
}

/// Python's `REAL` without rung 30's convergent nozzle — gate 8's design run uses the DEFAULT
/// fully-expanding one, which is the whole reason its `M9 > 1.8` clause means anything.
fn real_expanding() -> Losses {
    Losses { nozzle_convergent: false, ..real() }
}

fn design(gas: Gas) -> Engine {
    build_turbojet(gas, PI_C, TT4, 50_000.0, real())
}

fn surge_shapes() -> [ComponentMap; 3] {
    [ComponentMap::surge_flow(), ComponentMap::surge_pressure(), ComponentMap::surge_tilted()]
}

/// A `SpoolTransient` plus the two steady matchers, all on the FAST gas and all off the SAME
/// captured hardware — Python's `_fast_transient`.
fn fast_transient(comp_map: ComponentMap) -> (SpoolTransient, OffDesignMatcher, MapMatcher) {
    let st = SpoolTransient::new(design(Gas::thermally_perfect()), flight(), 1.0, comp_map);
    let base = OffDesignMatcher::new(design(Gas::thermally_perfect()), flight(), 1.0);
    let mm = MapMatcher::new(design(Gas::thermally_perfect()), flight(), 1.0, comp_map);
    (st, base, mm)
}

// ------------------------------------------------------------------------------------ gate 1
/// GATE 1 — the transient EQUILIBRIUM reproduces the rung-31/32 matched point.
///
/// FLAT map == `OffDesignMatcher` (rung 31); SHAPED map == `MapMatcher` (rung 32). The
/// equilibrium reaches it through the FORWARD closure and never calls either matcher, which is
/// what makes this a reduce rather than a tautology — a genuinely different code path onto the
/// same operating point. It includes a SUBSONIC point at `Tt4 = 520`, below the nozzle-unchoke
/// boundary, where the equilibrium meets rung 31's auto-dispatched subsonic branch (rung 33), and
/// one REACTING design point to say the equivalence is gas-independent.
#[test]
fn gate1_reduce_equilibrium_is_the_steady_matcher() {
    let (st, base, mm) = fast_transient(ComponentMap::flow_dominated());
    let flat = ComponentMap::flat();
    for tt4 in [1500.0f64, 1300.0, 1100.0, 900.0, 700.0, 520.0] {
        let eq = st.equilibrium(&flight(), tt4, Some(&flat));
        let ro = base.match_point(&flight(), tt4);
        assert!((eq.pi_c - ro.pi_c).abs() <= 1e-8 * ro.pi_c, "flat eq pi_c != rung31 at {tt4}");
        assert!((eq.tau_t - ro.tau_t).abs() <= 1e-8 * ro.tau_t, "flat eq tau_t != rung31 at {tt4}");
        assert!(
            (eq.mdot_air - ro.mdot_air).abs() <= 1e-8 * ro.mdot_air,
            "flat eq mdot != rung31 at {tt4}"
        );
        assert_eq!(eq.branch, ro.branch, "branch label mismatch at {tt4}");
    }
    // The design point returns exactly pi_c = 10, nu = 1.
    let d = st.equilibrium(&flight(), TT4, Some(&flat));
    assert!((d.pi_c - PI_C).abs() < 1e-7 && (d.nu - 1.0).abs() < 1e-7);
    assert_eq!(d.branch, Branch::Choked);

    // SHAPED map == MapMatcher (rung 32), including the shaft speed.
    let shape = ComponentMap::surge_flow();
    for tt4 in [1500.0f64, 1200.0, 900.0] {
        let eq = st.equilibrium(&flight(), tt4, Some(&shape));
        let mo = mm.match_with(&flight(), tt4, &shape);
        assert!(
            (eq.pi_c - mo.base.pi_c).abs() <= 1e-8 * mo.base.pi_c,
            "shaped eq pi_c != rung32 at {tt4}"
        );
        assert!(
            (eq.nu - mo.n_ratio).abs() <= 1e-8 * mo.n_ratio,
            "shaped eq nu != rung32 N at {tt4}"
        );
    }

    // One REACTING design point — the equivalence is gas-independent.
    let st_r = SpoolTransient::new(design(Gas::reacting_equilibrium()), flight(), 1.0, flat);
    let base_r = OffDesignMatcher::new(design(Gas::reacting_equilibrium()), flight(), 1.0);
    let er = st_r.equilibrium(&flight(), TT4, Some(&flat));
    let rr = base_r.match_point(&flight(), TT4);
    assert!(
        (er.pi_c - rr.pi_c).abs() <= 1e-7 * rr.pi_c,
        "reacting equilibrium != rung31 at design"
    );
}

// ------------------------------------------------------------------------------------ gate 2
/// GATE 2 — `Phi` is decreasing through its zero (a restoring sign), and an off-equilibrium `N`
/// relaxes BACK onto the running line. The running line is an ATTRACTOR, which is what makes the
/// equilibrium above a physical operating point rather than an algebraic root.
#[test]
fn gate2_the_running_line_is_an_attractor() {
    let (st, _, _) = fast_transient(ComponentMap::surge_flow());
    let shape = ComponentMap::surge_flow();
    for tt4 in [1300.0f64, 1100.0, 900.0] {
        let nu_eq = st.equilibrium(&flight(), tt4, Some(&shape)).nu;
        let below = st.instant(&flight(), nu_eq - 0.04, tt4, Some(&shape)).phi;
        let above = st.instant(&flight(), nu_eq + 0.04, tt4, Some(&shape)).phi;
        assert!(
            below > 0.0 && 0.0 > above,
            "Phi must decrease through 0 at Tt4={tt4} (accel below, decel above)"
        );
    }
    let tt4 = 1100.0;
    let nu_eq = st.equilibrium(&flight(), tt4, Some(&shape)).nu;
    let traj = st.integrate(&flight(), |_| tt4, nu_eq * 1.12, 12.0, 0.1, Some(&shape));
    let last = traj.last().expect("the relaxation march must produce points");
    assert!(
        (last.nu - nu_eq).abs() < 1e-3,
        "an off-equilibrium N must relax back onto the running line"
    );
    assert!(last.nu < traj[0].nu, "starting above equilibrium, N must decelerate toward it");
}

// ------------------------------------------------------------------------------------ gate 3
/// GATE 3 — THE FINDING. The peak above-running-line excursion `E(r)` falls monotonically from
/// the constant-`N` (`r -> 0`) displacement toward ~0 as `r -> inf`, and the `r -> 0` limit equals
/// the ALGEBRAIC constant-`N` map displacement computed with **no integration**.
///
/// That equality is the rung: the step excursion is a MAP property, and the dynamical content is
/// the RATIO of two clocks — which is why the finding is `r = tau_fuel/tau_spool` and not the
/// tautological "the shape is `I`-independent" (gate 5).
#[test]
fn gate3_the_finding_excursion_falls_with_the_clock_ratio() {
    let (st, _, _) = fast_transient(ComponentMap::surge_flow());
    let shape = ComponentMap::surge_flow();

    let e0 = st.constant_speed_excursion(&flight(), 1100.0, 1400.0, Some(&shape));
    assert!(
        e0 > 0.03,
        "the constant-N acceleration excursion must be a meaningful positive number: {e0}"
    );

    let es: Vec<f64> = [0.1f64, 0.5, 1.5, 4.0]
        .iter()
        .map(|&r| st.ramp_excursion(&flight(), 1100.0, 1400.0, r, Some(&shape), 5.0, 0.1).e)
        .collect();
    for w in es.windows(2) {
        assert!(w[0] > w[1], "the excursion must fall monotonically with r: {es:?}");
    }
    assert!(
        0.9 < es[0] / e0 && es[0] / e0 <= 1.0 + 1e-9,
        "E(r->0) must approach the algebraic E0: {}",
        es[0] / e0
    );
    assert!(
        es[es.len() - 1] < 0.4 * e0,
        "a slow ramp must nearly stay on the running line: {}",
        es[es.len() - 1] / e0
    );
}

// ------------------------------------------------------------------------------------ gate 4
/// GATE 4 — acceleration drives the point ABOVE the running line, deceleration BELOW, with the
/// SAME sign across three surge-realistic map shapes. Magnitude disclaimed (there is no surge
/// line until rung 36); the claim is worded toward/away from surge.
///
/// **AND THE DISCOVERY THAT MOTIVATES THE LINEAR LOADING SLOPE `l`.** Running rung 32's PARABOLIC
/// map (`l = 0`, which peaks at design) FORWARD gives the WRONG surge-side slope — the accel
/// excursion comes out NEGATIVE, `pi_c` falling toward low flow, which is non-physical. `l > 0`
/// supplies the physical negative speed-line slope. So the direction claim is not tuning to a
/// chosen answer: it required fixing a real deficiency of the backward-only rung-32 map when that
/// map is run forwards.
#[test]
fn gate4_direction_is_shape_robust_and_the_loading_slope_is_necessary() {
    let (st, _, _) = fast_transient(ComponentMap::flow_dominated());
    for shape in surge_shapes() {
        let accel = st.constant_speed_excursion(&flight(), 1100.0, 1400.0, Some(&shape));
        let decel = st.constant_speed_excursion(&flight(), 1300.0, 1000.0, Some(&shape));
        assert!(accel > 0.0, "acceleration must move ABOVE the running line: {shape:?}");
        assert!(decel < 0.0, "deceleration must move BELOW the running line: {shape:?}");
    }
    let parabolic = st.constant_speed_excursion(
        &flight(), 1100.0, 1400.0, Some(&ComponentMap::flow_dominated()),
    );
    let flat =
        st.constant_speed_excursion(&flight(), 1100.0, 1400.0, Some(&ComponentMap::flat()));
    assert!(
        parabolic < 0.0,
        "running rung-32's PEAKED (l=0) map forward gives the WRONG surge-side slope: {parabolic}"
    );
    assert!(flat.abs() < 1e-9, "a flat map has no speed-line slope, so no excursion: {flat}");
}

// ------------------------------------------------------------------------------------ gate 5
/// GATE 5 — the anti-tautology WITNESS. Illustrative, and deliberately NOT falsifiable.
///
/// In a one-state model `I` cannot appear in the `s`-dynamics at all: the march works purely in
/// `s = t/tau_spool`, so `nu(s)` is `I`-free BY CONSTRUCTION and physical time scales with `I`
/// trivially. That is why *"the shape is `I`-independent"* is vacuous, and why the real finding
/// lives on the ratio (gate 3), where a SECOND clock makes `I` load-bearing.
///
/// **Python's witness line is `abs(3.0*p.s - 3.0*p.s) < 1e-12`, which cannot fail by
/// construction — that IS its point.** It is not reproduced as an assertion here, because a Rust
/// `assert!(x - x < eps)` is a warning, not a comment. What IS asserted is what Python asserts
/// beside it: the ramp genuinely accelerates the spool, and `nu(s)` is reproducible.
#[test]
fn gate5_inertia_is_only_the_clock() {
    let (st, _, _) = fast_transient(ComponentMap::surge_flow());
    let shape = ComponentMap::surge_flow();
    let ramp = |s: f64| {
        if s <= 0.0 {
            1100.0
        } else if s >= 1.0 {
            1300.0
        } else {
            1100.0 + 200.0 * s
        }
    };
    let nu0 = st.equilibrium(&flight(), 1100.0, Some(&shape)).nu;
    let traj = st.integrate(&flight(), ramp, nu0, 6.0, 0.1, Some(&shape));
    assert!(
        traj.last().unwrap().nu > nu0 + 0.02,
        "the fuel ramp must actually accelerate the spool"
    );
    let traj2 = st.integrate(&flight(), ramp, nu0, 6.0, 0.1, Some(&shape));
    assert_eq!(traj.len(), traj2.len(), "nu(s) must be reproducible — even in LENGTH");
    for (a, b) in traj.iter().zip(traj2.iter()) {
        assert_eq!(a.nu.to_bits(), b.nu.to_bits(), "nu(s) must be reproducible");
    }
}

// ------------------------------------------------------------------------------------ gate 6
/// GATE 6 — the forward speed line is the EXACT inverse of rung 32's `solve_n`.
///
/// § 5.13 prediction 7 registered this as EXACT rather than merely tight, on the ground that
/// slice J ported the inverse of this very equation. Python's bar is `1e-9`; the measured worst
/// residual is printed by [`rung34_roster`], and the assertion below keeps Python's bar while a
/// second, tighter clause records what was actually achieved. **The tight clause is the port's,
/// deliberately, and it is stated as a separate assertion so a future loosening is visible.**
#[test]
fn gate6_the_forward_speed_line_inverts_solve_n() {
    let (st, _, _) = fast_transient(ComponentMap::flow_dominated());
    let mut worst = 0.0f64;
    let mut shapes: Vec<ComponentMap> = surge_shapes().to_vec();
    shapes.push(ComponentMap::flat());
    for shape in &shapes {
        for n in [0.6f64, 0.75, 0.9, 1.0, 1.1] {
            for m in [0.5f64, 0.8, 1.0, 1.2] {
                let tau_c = st.tau_c_forward(shape, n, m);
                let n_back = shape.solve_n(m, tau_c, st.inner.tau_c_d);
                worst = worst.max((n_back - n).abs());
            }
        }
    }
    assert!(worst < 1e-9, "solve_n(m, tau_c_forward(n,m)) must return n to machine zero: {worst:e}");
    assert!(worst < 1e-14, "PORT: prediction 7 registered this inverse as EXACT, not merely \
                            tight — slice J ported the inverse of this equation. Worst {worst:e}");
}

// ------------------------------------------------------------------------------------ gate 7
/// GATE 7 — a fuel-cut spool-down decreases `N` monotonically, flips the branch choked ->
/// subsonic as `pt9/p0` falls through critical, and approaches thrust-neutral idle. The rung-33
/// handshake, reached dynamically.
#[test]
fn gate7_a_spooldown_crosses_into_the_subsonic_branch() {
    let (st, _, _) = fast_transient(ComponentMap::surge_flow());
    let shape = ComponentMap::surge_flow();
    let r = 6.0;
    let sched = |s: f64| {
        if s <= 0.0 {
            900.0
        } else if s >= r {
            460.0
        } else {
            900.0 - (900.0 - 460.0) * (s / r)
        }
    };
    let nu0 = st.equilibrium(&flight(), 900.0, Some(&shape)).nu;
    let traj = st.integrate(&flight(), sched, nu0, r + 15.0, 0.1, Some(&shape));
    for w in traj.windows(2) {
        assert!(w[1].nu <= w[0].nu + 1e-9, "spool-down: N must not increase");
    }
    let last = traj.last().unwrap();
    assert!(last.nu < nu0 - 0.1, "N must decay meaningfully");
    assert_eq!(traj[0].branch, Branch::Choked);
    assert!(
        traj.iter().any(|p| p.branch == Branch::Subsonic),
        "must cross the unchoke boundary"
    );
    let i = (1..traj.len())
        .find(|&k| traj[k].branch != traj[k - 1].branch)
        .expect("a branch flip must occur");
    assert_eq!(traj[i - 1].branch, Branch::Choked);
    assert_eq!(traj[i].branch, Branch::Subsonic, "flip must be choked->subsonic");
    assert!((traj[i].m9 - 1.0).abs() < 0.02, "the branch flip must occur at M9 ~ 1 (continuous)");
    assert!(
        last.sp_thrust < 0.15 * traj[0].sp_thrust,
        "the spool-down approaches thrust-neutral idle"
    );
}

// ------------------------------------------------------------------------------------ gate 8
/// GATE 8 — the default design run is bit-for-bit rung 6; building a `SpoolTransient` does not
/// perturb it. The rungs-7-and-up invariant, re-asserted at the first DYNAMIC rung.
#[test]
fn gate8_the_design_cycle_is_untouched() {
    let r = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, 50_000.0, real_expanding())
        .run(&flight(), 1.0);
    assert!(
        (r.performance.specific_thrust - 798.37).abs() < 0.5,
        "{}",
        r.performance.specific_thrust
    );
    assert!(r.m9 > 1.8 && (r.p9 - flight().p0).abs() < 1e-6, "default nozzle: fully expanded");

    let _ = SpoolTransient::new(
        design(Gas::reacting_equilibrium()), flight(), 1.0, ComponentMap::surge_flow(),
    );

    let r2 = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, 50_000.0, real_expanding())
        .run(&flight(), 1.0);
    assert!(
        (r2.performance.specific_thrust - r.performance.specific_thrust).abs() < 1e-9,
        "building a SpoolTransient perturbed the design run"
    );
}

// ============================================================================ port additions
/// ADDED BY THE PORT — § 5.13 prediction 2, and step 1's own finding.
///
/// § 5.3's inheritance census made phase 5 ship `MatcherHooks` a phase early for exactly one
/// name, and this module is what finally overrides it. Two things have to be true and NEITHER is
/// a value:
///
/// 1. **The hook FIRES.** Slice N's FINDING 3 caught a hook that compiled and was unreachable
///    through the matcher, and only a count could see it.
/// 2. **It is rung 34's function, not rung 31's.** Step 1 measured the discriminator: swapping
///    the table moves five of the smoke's eight value gates at ~9e-12 — but **not the subsonic
///    one**, because `instant_tail` re-solves `pi_t` from nozzle continuity on that branch and
///    discards the choked-star answer entirely. So on a subsonic cell the table could be wired to
///    anything at all, and a value oracle would report agreement.
#[test]
fn the_hook_table_fires_and_rung31s_does_not() {
    counters::take(); // this test's own tallies (thread-local, one thread per #[test])
    let (st, _, _) = fast_transient(ComponentMap::surge_flow());
    let before = counters::take();
    assert_eq!(before.r34_solve_turbine, 0, "constructing must not solve a turbine");

    st.equilibrium(&flight(), 1300.0, None);
    let c = counters::take();
    assert!(
        c.r34_solve_turbine > 0,
        "RUNG 34's turbine solve never fired — the inner matcher is carrying rung 31's table, \
         and every value gate on a CHOKED cell would still be ~9e-12 out rather than failing \
         loudly (§ 5.13 probe 2)"
    );

    // The subsonic branch: the hook still fires (the choked star is always solved first) but its
    // answer is DISCARDED. This clause records that, so a later reader does not conclude from
    // the count above that the table is load-bearing everywhere.
    let sub = st.equilibrium(&flight(), 520.0, Some(&ComponentMap::flat()));
    assert_eq!(sub.branch, Branch::Subsonic);
    let c2 = counters::take();
    assert!(c2.r34_solve_turbine > 0, "the choked star is solved before the dispatch, always");
    assert!(
        c2.subsonic_fallbacks + c2.subsonic_escalations <= c2.r34_solve_turbine,
        "a fallback cannot happen without a choked-star solve preceding it"
    );
}

/// ADDED BY THE PORT — `ComponentMap::phi_max` is read at BOTH forward flow-search caps.
///
/// The symbol was owed to this slice since slice M, its deferral note described a branch that does
/// not exist, and § 5.13 probe 1 measured 16 508 calls with `vsv == 0.0` at every one. The value
/// gate for the arms it never reaches lives in `slice_p_smoke.rs`; what this one says is that the
/// cap is READ at all — a `hi` wall silently replaced by the constant `2.5` would leave every
/// number in this file unchanged on the shapes where `phi_max*n > 2.5`.
#[test]
fn phi_max_is_read_at_both_flow_search_caps() {
    let shape = ComponentMap::surge_flow();
    // At n = 1 the shaped map's cap is BELOW 2.5, so it — not the constant — decides the wall.
    let cap = shape.phi_max(0.1);
    assert!(cap < 2.5, "surge_flow's phi_max must bind against the 2.5 constant: {cap}");
    // ...and the flat map's does not, which is the other half of the `min`.
    assert!(
        ComponentMap::flat().phi_max(0.1) > 2.5,
        "a flat map has no positive-work edge, so the 2.5 constant must be what binds"
    );
}

// ================================================================================ the roster
/// The record of what ported, on `rung41.rs`'s precedent.
#[test]
fn rung34_roster() {
    let roster: [(&str, bool); 8] = [
        ("test_reduce_equilibrium_is_the_steady_matcher", true),
        ("test_stability_running_line_is_an_attractor", true),
        ("test_the_finding_excursion_vs_ratio", true),
        ("test_direction_shape_robust", true),
        ("test_I_is_only_the_clock", true),
        ("test_forward_backward_map_inverse", true),
        ("test_spooldown_crosses_into_subsonic", true),
        ("test_cycle_untouched", true),
    ];
    assert_eq!(
        roster.len(),
        8,
        "tests/test_rung34.py has 8 test functions — if that changed, this roster is stale and \
         the port is gating against a file that no longer exists"
    );
    assert_eq!(roster.iter().filter(|(_, p)| *p).count(), 8, "all eight of rung 34's gates port");

    // ADDED here, listed so the name diff is symmetric in both directions.
    let added = ["the_hook_table_fires_and_rung31s_does_not",
                 "phi_max_is_read_at_both_flow_search_caps"];
    assert_eq!(added.len(), 2);

    // § 5.13 prediction 10: Python marks NONE of these `slow`, and nothing here earns one.
    // Measured rather than assumed — slice M's rule.
    println!("rung34.rs: {} ported + {} added, 0 slow markers", roster.len(), added.len());
}
