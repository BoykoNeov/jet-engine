//! SLICE Z step 5 — **THE MANUFACTURED GATES**: everything 35 335 oracle keys and 57 ported /
//! smoke gates provably cannot see.
//!
//! # THE LIST IS MEASURED, NOT ARGUED
//!
//! Step 3 ran eight injections twice each (a LIVENESS build with a `panic!` marker, then the
//! SEMANTIC edit) and step 4 re-ran them with the oracle as a fourth target. Four injections
//! survive **all 64 gates and all 35 335 keys**, and they are exactly the four § 5.24
//! pre-registered:
//!
//! | injection | what it breaks | prediction |
//! |---|---|---|
//! | I2 | rung 66's inline undamped joint IC → rung 67's damped solver | **P6** |
//! | I3 | rung 67's `assert lag is None` reads the CARRIER | **P11** |
//! | I4 | the `_lag` guard restores `None` | **P7a** |
//! | I5 | the `_tau_gov` guard restores `None` | **P7b** |
//!
//! **THREE OF THE FOUR ARE *PROVABLY* INVISIBLE, NOT MERELY UNNOTICED.** I3's, I4's and I5's
//! liveness markers never fire anywhere in the slice: `ft.inner.lag` is never `Some` where rung
//! 67's refusal would read it, and neither guard's `prev` is ever `Some`. That is probe 3's *max
//! nesting depth 1, 0 nested events* re-measured in the PORT rather than inherited from a Python
//! reading — and it is why the nests below are MANUFACTURED.
//!
//! P4's five dead arms and P5's complex arm are gated here too, on their ARGUMENTS, the way probe
//! 7 exhibits them.
//!
//! # WHAT THIS FILE READS, ASKED OF EVERY ASSERTION
//!
//! Slice V step 5's lesson: *ask of every assertion in a manufactured-bug gate WHAT FILE IT READS
//! — the four that read nothing survive a regenerated golden.* **Nothing here reads a golden.**
//! Every assertion is a counter, a same-run difference between two dispatch arms, or a property of
//! a shipped function at arguments this file chooses. Regenerating `slice_z_pypy.tsv` cannot make
//! one of them pass or fail, which is the point: they cover exactly what those keys cannot.
//!
//! [`CensusZ`] is thread-local with no per-test reset, so every test that reads it resets first.
//!
//! # AND THESE GATES WERE THEMSELVES MUTATED
//!
//! Four slices running found a defect in their closing gates only by mutating them, so it was
//! budgeted here as a task. **Eleven mutations, three survivors, and all three are recorded rather
//! than repaired away:**
//!
//! * **MU9 is a real defect in a gate of mine** — `gains` returns a CENTRAL DIFFERENCE, and the
//!   central difference of any constant is zero, so `assert_eq!(r_none, 0.0)` proves the empty-caps
//!   arm returns a CONSTANT and never that the constant is `0.0`. The name claimed the literal.
//!   Corrected at [`p4_the_two_dead_gains_arms_are_exhibited_on_their_arguments`].
//! * **MU6 and MU7 are not gate defects at all** — the two branches they delete are VALUE-INERT,
//!   so nothing can pin them. That reclassifies two of § 5.24 (v)'s five *dead* arms as
//!   *unobservable*, which is a stronger statement about the source than the pre-registration made.
//!   Recorded at [`p4_the_three_dead_leaf_arms_are_exhibited_beside_their_live_siblings`].
//!
//! The full table is in the step-5 write-up.

use turbojet::bleed_transient::LeverArm;
use turbojet::cross_loop::{build_cross_loop_cascade, joint_fixed_point, window};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{asym_extra, AsymmetricLag, Floor, FuelPoint, PointExtra,
                               SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::valve_of;
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::stator_transient::{MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient,
                                 StatorLeg};
use turbojet::two_lag::{build_two_lag_cascade, eig, violation, CensusZ, GAINS_DG, GAINS_DQ};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{LaggedFuel, LaggedGovernor};

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
const TMAX: f64 = 1200.0;
const TAU: f64 = 0.05;

fn sm() -> f64 { PHI / FLOOR - 1.0 }
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

fn full(b: ScheduledStatorTransient) -> ScheduledStatorCore {
    match b {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn t66(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_two_lag_cascade(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

fn t67(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_cross_loop_cascade(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                                  arm))
}

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }
fn armed() -> LeverArm { LeverArm::floored(BleedLimiter::with_tau(PHI, B, Some(TAU))) }
fn fuel() -> SurgeLimiter { SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm()) }
fn lag_a() -> AsymmetricLag { AsymmetricLag::new(0.05, 0.15) }
fn surge_leg() -> StatorLeg<'static> {
    StatorLeg { accel: None, surge: Some(Floor::Phi(fuel())), tt4_max: None }
}
fn gov_leg() -> StatorLeg<'static> {
    StatorLeg { accel: None, surge: None, tt4_max: Some(TMAX) }
}

/// The per-point key count — 14 / 16 / 20 / 21. Which MARCHER ran, read off the one thing a
/// trajectory carries that a float cannot fake.
fn route(t: &[FuelPoint]) -> usize { t[0].key_count() }

/// The seven keys the suites compare, as raw bits so a comparison is exact.
fn keys(t: &[FuelPoint]) -> Vec<[u64; 7]> {
    t.iter().map(|p| [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
                      p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits()]).collect()
}

// =============================================================================================
// P6 — THE TWO JOINT-IC SOLVES STAY SEPARATE
// =============================================================================================

/// **WHICH SOLVER RAN, COUNTED — because nothing else can tell.**
///
/// § 5.24 (iv): rung 66 iterates INLINE and UNDAMPED, capped at 60, and asserts on failure,
/// because its own identity pins the contraction at 1 so a stall there genuinely IS the
/// degeneracy. Rung 67 sweeps `w ∈ (1, 0.5, 0.25)`, because on cascade A `|P|` is pinned by
/// nothing and a stall would be a SOLVER failure. **Routing rung 66 through rung 67's solver is
/// bit-exact on the shipped grid** (`w = 1.0` on 36 of 39 calls) — step 3's injection I2 measured
/// it at **0 of 64 gates** and step 4 at **0 of 35 335 oracle keys**.
///
/// **MUTATION FOUND ONE DEFECT HERE, AND IT IS THE INSTRUCTIVE ONE.** The first draft asserted
/// only `jfp_calls == 0` on the rung-66 arm. That passes if rung 66's march never runs at
/// all — a `return` at the top of the cascade marcher reddens nothing. `r66_inline_ic > 0` is the
/// liveness half, and the rung-67 arm's `r66_inline_ic == 0` is the mirror: a port that solved the
/// IC twice would pass a one-sided version of this gate.
#[test]
fn p6_rung_66_never_reaches_the_damped_solver_and_rung_67_always_does() {
    CensusZ::reset();
    let (t, _) = t66(&armed()).stator_march_scoped(
        &flight(), &ramp(DS), None, &surge_leg(),
        &MarchScope { lag: Some(lag_a()), ..MarchScope::DEFAULT });
    let c = CensusZ::take();
    assert_eq!(route(&t), 20, "the rung-66 cascade marcher did not run");
    assert!(c.r66_inline_ic > 0, "rung 66's INLINE joint IC never ran: {c:?}");
    assert_eq!(c.jfp_calls, 0,
               "rung 66's march reached `joint_fixed_point`. That is bit-exact on this grid and \
                WRONG: it converts an assert that reports the degeneracy into a damped retry that \
                hides it. {c:?}");

    CensusZ::reset();
    let (x, _) = t67(&armed()).stator_march_scoped(
        &flight(), &ramp(DS), None, &gov_leg(),
        &MarchScope { tau_gov: Some(TAU), ..MarchScope::DEFAULT });
    let c = CensusZ::take();
    assert_eq!(route(&x), 21, "the rung-67 cross marcher did not run");
    assert!(c.jfp_calls > 0, "rung 67's march never reached the damped solver: {c:?}");
    assert_eq!(c.r66_inline_ic, 0,
               "rung 67's march ALSO ran rung 66's inline loop — the IC was solved twice, which \
                no value key would show because both converge to the same point. {c:?}");
}

/// **AND THE TWO SOLVERS ARE GENUINELY DIFFERENT FUNCTIONS, WHERE IT MATTERS.**
///
/// The counter above says which one ran; this says why that is worth counting. At `|P| = 5` the
/// damped sweep converges (on `w = 1/4`, whose composite multiplier is `(1−w) + wP`), while the
/// undamped recurrence — rung 66's, re-spelled here at its own cap of 60 — diverges. So the
/// substitution the counter forbids would turn rung 66's *"this is the degeneracy locus"* assert
/// into a silent success.
#[test]
fn p6_the_damped_sweep_converges_where_rung_66s_undamped_loop_cannot() {
    let (g_star, q_star, a_lin) = (3.0e-3, 0.04, 1.0e-3);
    let p = -5.0;
    let required_of = |q: f64| g_star + a_lin * (q - q_star);
    let command_of = |g: f64| q_star + (p / a_lin) * (g - g_star);

    let r = joint_fixed_point(&required_of, &command_of, q_star + 0.01, false, 1e-12, 60);
    assert_eq!(r.w, 0.25, "the ladder did not descend to the damping |P| = 5 needs: {r:?}");
    assert!(r.res <= 1e-9 && (r.g - g_star).abs() < 1e-9, "{r:?}");

    // Rung 66's spelling, at the same laws and the same cap: `g` and `q` chase each other by the
    // full step every iteration, so the error grows by |P| per pass.
    let (mut g, mut q, mut res) = (0.0f64, q_star + 0.01, f64::INFINITY);
    for _ in 1..=60usize {
        let gn = required_of(q);
        let qn = command_of(gn);
        res = (gn - g).abs().max((qn - q).abs());
        g = gn;
        q = qn;
        if res <= 1e-12 { break; }
    }
    assert!(res > 1e-9,
            "the undamped recurrence converged at |P| = 5, so the two solvers are \
             interchangeable here and this gate proves nothing: res = {res:.3e}");
}

// =============================================================================================
// P11 — THE CARRIER DISCARD: rung 67's refusal reads the ARGUMENT, and the march drops the lag
// =============================================================================================

/// **A REFUSAL THAT CANNOT SEE WHAT IT REFUSES — the port reproduces it, and one token would
/// change that.**
///
/// Step 1's finding, gated. A rung-67 `_stator_march` forwards `lag` to rung **66's** carrier, and
/// rung 67's own armed branch returns before `super()`, so that carrier is never read: arming BOTH
/// clocks through the march on a rung-67 machine is **silently discarded**, where the same pairing
/// through a direct `integrate_fuel` is refused. Python does exactly this (`assert lag is None`
/// reads the ARGUMENT), so the port is a translation and not a repair — but `lim.lag` →
/// `lim.lag.or_else(|| ft.inner.lag.get())` is a one-token change that **no value key and no
/// oracle key can see** (step 3's I3: 0 of 64 gates, and its liveness marker never fires).
///
/// **THE SECOND HALF IS THE ONE THAT MAKES THE FIRST MEAN ANYTHING.** A discarded lag and a lag
/// that never left the scope look identical from outside — [[rust-port-slice-v-step4]], *an
/// injection whose only trace is OBJECT STATE reads exactly like one that never applied*. So the
/// same channel is run ONE RUNG DOWN, where the carrier IS read, and there it changes the marcher.
#[test]
fn p11_a_rung_67_march_discards_the_fuel_lag_and_the_same_channel_delivers_one_rung_down() {
    let (fl, m) = (flight(), t67(&armed()));
    let gov_only = MarchScope { tau_gov: Some(TAU), ..MarchScope::DEFAULT };
    let both = MarchScope { tau_gov: Some(TAU), lag: Some(lag_a()), ..MarchScope::DEFAULT };

    let (a, _) = m.stator_march_scoped(&fl, &ramp(DS), None, &gov_leg(), &gov_only);
    let (b, _) = m.stator_march_scoped(&fl, &ramp(DS), None, &gov_leg(), &both);
    assert_eq!(route(&a), 21, "the gov-only march did not reach the cross marcher");
    assert_eq!(route(&b), 21, "arming the lag through the march changed the MARCHER");
    assert_eq!(keys(&a), keys(&b),
               "the fuel lag armed through a rung-67 march is NOT being discarded. Python's \
                refusal reads the ARGUMENT, so this route must be bit-for-bit the governor-only \
                one; a port that resolved `lag` against the carrier would either refuse here or \
                march a different trajectory.");

    // THE WITNESS. The scope -> carrier channel is not dead — one rung down it decides the route.
    let m66 = t66(&armed());
    let (c, _) = m66.stator_march_scoped(&fl, &ramp(DS), None, &surge_leg(), &MarchScope::DEFAULT);
    let (d, _) = m66.stator_march_scoped(
        &fl, &ramp(DS), None, &surge_leg(),
        &MarchScope { lag: Some(lag_a()), ..MarchScope::DEFAULT });
    assert_eq!(route(&c), 16, "rung 66 with no lag is rung 65's valve march");
    assert_eq!(route(&d), 20,
               "rung 66 did NOT read the carrier the march set, so the equality above is a dead \
                channel rather than a discarded lag and this gate measures nothing");
    assert_ne!(keys(&c), keys(&d), "the two rung-66 routes marched the same trajectory");
}

// =============================================================================================
// P7 — THE TWO GUARDS RESTORE THE PREVIOUS VALUE
// =============================================================================================

/// **THE MANUFACTURED NEST, HALF ONE.** Python is `prev, self._lag = self._lag, lag` …
/// `finally: self._lag = prev`. Probe 3 measured max nesting depth **1** with **0** nested events
/// over rungs 62–67, and step 3's I4 confirmed it from the other side: swapping the `Drop` body to
/// restore `None` moves 0 of 64 gates and 0 of 35 335 oracle keys, **and its liveness marker
/// never fires**, so `prev` is provably never `Some` on any shipped path.
///
/// The inner scope is a REAL march, not a bare guard pair: `stator_march_scoped` builds its own
/// [`LaggedFuel`] from the scope it is handed, so this nests the SHIPPED guard through the SHIPPED
/// cell.
#[test]
fn p7a_the_lag_guard_restores_the_previous_value_through_a_manufactured_nest() {
    let m = t66(&armed());
    let core = &m.fuel.inner;
    assert!(core.lag.get().is_none(), "a fresh machine carries no fuel lag");

    let outer = AsymmetricLag::new(0.03, 0.09);
    let inner = AsymmetricLag::new(0.05, 0.15);
    assert_ne!(outer, inner, "the nest needs two DIFFERENT values to be about anything");
    {
        let _g = LaggedFuel::set(core, Some(outer));
        assert_eq!(core.lag.get(), Some(outer));

        let (t, _) = m.stator_march_scoped(
            &flight(), &ramp(DS), None, &surge_leg(),
            &MarchScope { lag: Some(inner), ..MarchScope::DEFAULT });
        assert_eq!(route(&t), 20, "the inner march never ran, so nothing was nested");

        assert_eq!(core.lag.get(), Some(outer),
                   "the guard restored `None` and CLEARED the outer scope. Python restores the \
                    PREVIOUS value, and no value key anywhere in this slice separates the two.");
    }
    assert!(core.lag.get().is_none(), "the outer guard did not restore on drop");
}

/// **THE MANUFACTURED NEST, HALF TWO** — [`LaggedFuel`]'s shape verbatim on the other field, asked
/// SEPARATELY rather than inherited from rung 66's answer ([[rust-port-slice-n-step4]]: *a carrier
/// claim on ONE hook says nothing about the next*). Step 3's I5 measured the same zero.
#[test]
fn p7b_the_tau_gov_guard_restores_the_previous_value_through_a_manufactured_nest() {
    let m = t67(&armed());
    let core = &m.fuel.inner;
    assert!(core.tau_gov.get().is_none(), "a fresh machine carries no governor clock");

    let (outer, inner) = (0.02, TAU);
    assert_ne!(outer, inner);
    {
        let _g = LaggedGovernor::set(core, Some(outer));
        assert_eq!(core.tau_gov.get(), Some(outer));

        let (t, _) = m.stator_march_scoped(
            &flight(), &ramp(DS), None, &gov_leg(),
            &MarchScope { tau_gov: Some(inner), ..MarchScope::DEFAULT });
        assert_eq!(route(&t), 21, "the inner march never ran, so nothing was nested");

        assert_eq!(core.tau_gov.get(), Some(outer),
                   "the guard restored `None` and CLEARED the outer scope.");
    }
    assert!(core.tau_gov.get().is_none(), "the outer guard did not restore on drop");
}

// =============================================================================================
// P5 — `eig`'s COMPLEX ARM SHIPS LIVE AND RUNG 66 CANNOT REACH IT
// =============================================================================================

/// § 5.24 (vi): probe 7 split rung 66's `_eig` census by CALLING FUNCTION and found **80 of 80
/// real — the complex arm never runs on rung 66 at all.** Step 3's injection I1 re-measured that
/// in the Rust from the other side: panicking inside the complex branch reddens **five gates, and
/// every one is in `rung67.rs`**.
///
/// So the arm is gated by a DIRECT call here, and the second half is what makes it a statement
/// about rung 66 rather than about a chosen argument: **under rung 66's identity `R_q·C_g ≡ 1`
/// the discriminant is `tr² ≥ 0` identically**, at every clock pair, so no rung-66 gain pair can
/// reach the complex branch. The reciprocal pairs below are exact in binary, so `det` is exactly
/// `0.0` and the claim is not a tolerance.
#[test]
fn p5_the_complex_arm_of_eig_is_live_and_no_rung_66_gain_pair_can_reach_it() {
    // matched clocks and P < 0 — cascade A's regime, where the pair is complex
    let e = eig(1.0, -0.5, 0.05, 0.05);
    assert!(!e.real && e.lam.is_none(), "the complex arm did not run: {e:?}");
    assert!(e.disc < 0.0 && e.det > 0.0, "{e:?}");
    assert_eq!(e.rho, e.det.abs().sqrt(), "the complex branch's radius is sqrt|det|");

    // …and rung 66's own regime cannot get there, at any clocks or any gain magnitude.
    for (r_q, c_g) in [(-2.0, -0.5), (-0.5, -2.0), (-4.0, -0.25), (-1.0, -1.0), (2.0, 0.5)] {
        for (tg, tv) in [(0.05, 0.05), (0.005, 0.5), (0.5, 0.005), (0.2, 0.01)] {
            let x = eig(r_q, c_g, tg, tv);
            assert_eq!(x.det, 0.0, "the identity R_q C_g = 1 must give det EXACTLY 0: {x:?}");
            assert!(x.real && x.disc >= 0.0, "rung 66 reached the complex branch: {x:?}");
            assert_eq!(x.disc, x.tr * x.tr, "det = 0 makes the discriminant tr^2 exactly");
        }
    }
}

// =============================================================================================
// P4 — THE FIVE DEAD ARMS, EXHIBITED ON THEIR ARGUMENTS
// =============================================================================================

/// **`gains`'s TWO DEAD ARMS** (§ 5.24 (v)): every one of the 80 shipped calls arrives with
/// `accel = None, surge = Some(...)`, so the `accel` branch and the `caps.is_empty()`
/// fall-through are both **0 of 80**. Probe 7 exhibits them on the arguments and so does this.
///
/// The base point is taken from a real cascade march rather than invented, so the closures are
/// evaluated where they are actually defined.
#[test]
fn p4_the_two_dead_gains_arms_are_exhibited_on_their_arguments() {
    let m = t66(&armed());
    let (t, _) = m.stator_march_scoped(
        &flight(), &ramp(DS), None, &surge_leg(),
        &MarchScope { lag: Some(lag_a()), ..MarchScope::DEFAULT });
    // a point where the fuel leg is actually riding, so the surge arm is not itself dormant
    let p = t.iter().find(|p| asym_extra(p).1 > 0.0).expect("the fuel leg never rode");
    let (g, q) = (asym_extra(p).0, valve_of(p).0);
    let (a, h, mf_sched) = (p.nu_lp, p.nu_hp, p.mf_sched);
    let floor = Floor::Phi(fuel());
    // **`margin = 0.0`, AND THE VALUE IS A MEASUREMENT RATHER THAN A CHOICE.** The first draft
    // used `1.10` — a cap 10 % above the steady line — and at a mid-ramp point that cap sits ABOVE
    // `mf_sched`, so `max(0, mf_sched - cap)` clamps to zero on BOTH sides of the difference and
    // `R_q` reads EXACTLY 0. The `assert_ne!(r_accel, r_surge)` below still passed, because
    // `r_surge` is non-zero — i.e. the accel exhibit was vacuous and only the reference arm was
    // being tested. `margin = 0` is Python's own "never exceed the steady fuel/pressure ratio",
    // which binds on an accelerating ramp, and the `r_accel != 0.0` bar is what keeps this honest.
    let sched = m.fuel.accel_schedule(&flight(), LO, HI, 0.0, 6);

    let (r_surge, c_surge) =
        m.gains(&flight(), a, h, g, q, mf_sched, None, Some(&floor), GAINS_DQ, GAINS_DG);
    let (r_accel, c_accel) =
        m.gains(&flight(), a, h, g, q, mf_sched, Some(&sched), None, GAINS_DQ, GAINS_DG);
    let (r_none, c_none) =
        m.gains(&flight(), a, h, g, q, mf_sched, None, None, GAINS_DQ, GAINS_DG);

    // (i) the ACCEL arm runs and computes a DIFFERENT law — it is not silently the surge one
    assert!(r_accel.is_finite() && c_accel.is_finite(), "{r_accel} {c_accel}");
    assert_ne!(r_accel, r_surge,
               "the accel arm returned the surge arm's gain: `caps` is not being fed from the \
                schedule at all");
    assert!(r_surge != 0.0, "the reference arm is itself dormant, so the comparison is vacuous");

    // (ii) the `caps.is_empty()` FALL-THROUGH. **WHAT THIS LINE PROVES IS THAT THE FUEL LAW HAS NO
    //      GAIN THERE, NOT THAT THE BRANCH RETURNS THE LITERAL `0.0`** — and the difference was
    //      found by mutating this gate (MU9): `gains` hands back a CENTRAL DIFFERENCE, and the
    //      central difference of ANY constant is zero, so changing the branch to `return 1.0`
    //      leaves `r_none` at exactly `0.0` and this assertion green. `gains` exposes only the
    //      derivative, so the literal is out of reach from here; it is the oracle's, through the
    //      marches that consume `required`. The claim is stated at what it covers.
    assert_eq!(r_none, 0.0,
               "with NEITHER min-select leg armed the fuel law must be CONSTANT in the valve \
                position, so its central difference is exactly zero; got {r_none}");
    assert!(r_accel != 0.0,
            "the accel arm is dormant too, so `r_none == 0` is a property of the DIFFERENCING \
             and not of the empty-caps branch: {r_accel}");

    // (iii) …and `C_g` IS THE SAME NUMBER ON ALL THREE ARMS, BIT FOR BIT. `big_c` roots
    //       `r64_solve_b` over TRIAL positions at the applied fuel `mf_sched - g` and never looks
    //       at `caps` — so the valve's law is blind to which fuel leg is armed, which is the
    //       *"neither closure knows the other exists"* half of rung 66's identity, stated as an
    //       equality rather than in prose. A port that threaded the legs into `big_c` would still
    //       measure a product near 1 and would no longer be MEASURING anything.
    assert_eq!(c_surge.to_bits(), c_accel.to_bits(),
               "the valve's law moved when the fuel leg changed: {c_surge} vs {c_accel}");
    assert_eq!(c_surge.to_bits(), c_none.to_bits(),
               "the valve's law moved when the fuel leg was removed: {c_surge} vs {c_none}");
    assert!(c_surge.is_finite() && c_surge != 0.0, "the valve's own gain is dormant: {c_surge}");
}

/// **THE THREE DEAD LEAF ARMS**, each beside a call that takes the LIVE arm — so a gate cannot
/// pass by the branch having been deleted (slice X step 5's lesson: *a zero-count assertion is
/// satisfied by DELETING the branch it names*).
///
/// # TWO OF THE THREE ARE NOT MERELY DEAD, THEY ARE **UNOBSERVABLE**, AND MUTATION IS WHAT SAID SO
///
/// § 5.24 (v) called five arms *dead*, meaning UNEXERCISED on the shipped grid. Mutating this
/// gate (MU6, MU7) found that two of them are something stronger and different: **deleting the
/// branch changes no output on any input at all.**
///
/// * `window`'s `P == 0` guard — Rust's `2π / 0.0.sqrt()` is already `+inf`, which is exactly what
///   the guard writes out. The branch exists because **Python's** float division by zero RAISES,
///   so it is load-bearing THERE and value-inert HERE.
/// * `sign_changes`'s `peak <= 0` early return — with `peak = 0` the floor is `0.0`, `x.abs() <
///   0.0` is never true and `prev != 0.0` is never true, so both spellings return `0`. Inert in
///   BOTH languages: the guard is defensive, not functional.
///
/// **So no gate can pin those two branches' existence**, manufactured or otherwise — only a
/// counter could, and a counter on a branch that cannot change a value is testing the source text
/// rather than the program. The assertions below therefore claim the VALUE Python produces there,
/// which is the port's actual contract, and say so instead of reading as branch coverage. The
/// third arm — `violation`'s upper-limit break — is genuinely observable and MU8 caught it.
#[test]
fn p4_the_three_dead_leaf_arms_are_exhibited_beside_their_live_siblings() {
    // (i) `sign_changes`'s `peak <= 0` early return — 0 of 10 shipped calls
    assert_eq!(turbojet::cross_loop::sign_changes(&[0.0, 0.0, 0.0]), 0);
    assert_eq!(turbojet::cross_loop::sign_changes(&[0.0, -0.0, 0.0]), 0, "`-0.0` is still zero");
    assert_eq!(turbojet::cross_loop::sign_changes(&[1.0, -1.0, 1.0]), 2, "the live arm");

    // (ii) `window`'s `P == 0` guard — Python's float division RAISES there, so the `inf` is
    //      written out. 0 of 31 shipped calls take it.
    let w0 = window(0.0);
    assert!(w0.t_over_tau.is_infinite() && w0.t_over_tau > 0.0, "{w0:?}");
    assert_eq!(w0.zeta, 1.0, "at P = 0 the damping is exactly critical-free");
    assert!(!w0.opens, "P >= 0 opens no window");
    // the guard is a BRANCH and not a limit: an arbitrarily small non-zero P is finite
    assert!(window(-1e-300).t_over_tau.is_finite(),
            "the `P == 0` test was widened into a magnitude test");

    // (iii) `violation`'s `s > s_hi` break NOT taken — 0 of 41 shipped calls, because every
    //       shipped march ends past `r`. Reached with an `s_hi` beyond the last point.
    let traj: Vec<FuelPoint> = (0..6).map(|i| pt(i as f64 * 0.1, 0.80 - 0.05 * i as f64)).collect();
    let whole = violation(&traj, 0.80, 99.0);
    let cut = violation(&traj, 0.80, 0.25);
    assert!(whole > cut && cut > 0.0,
            "the un-broken arm integrated no more than the broken one: {whole} vs {cut}");
    assert_eq!(violation(&traj, 0.80, 0.5), whole,
               "`s_hi` exactly at the last point must not break either — the test is `>`, not \
                `>=`, and rung 67's `exceed` uses the other one on purpose");
}

/// A synthetic point carrying only `s` and `phi_lp`; the rest are `NaN` so a reader that started
/// touching a third field returns `NaN` instead of a plausible number.
fn pt(s: f64, phi_lp: f64) -> FuelPoint {
    FuelPoint {
        s, phi_lp, nu_lp: f64::NAN, nu_hp: f64::NAN, tt4: f64::NAN, f: f64::NAN,
        pi_lpc: f64::NAN, pi_hpc: f64::NAN, phi_hp: f64::NAN, mdot_air: f64::NAN,
        sp_thrust: f64::NAN, branch: Branch::Choked, mf: f64::NAN, mf_sched: f64::NAN,
        extra: PointExtra::None,
    }
}
