//! Rung 32 — COMPONENT-MAP MATCHING: the map re-labels the choke-pinned work.
//!
//! Port of `tests/test_rung32.py` (phase 5 slice J).
//!
//! Rung 31 found its running line "without a map" and held `eta_c`, `eta_t` at design. Rung 32's
//! finding is that this OVER-CLAIMED. Hang an analytic efficiency island and a family of speed
//! lines on the compressor, close the cycle against them, and:
//!
//! * `eta_c`/`eta_t` become OUTPUTS of an outer secant around rung 31's inner solve;
//! * `pi_c` and `mdot` MOVE first-order, because both run through `eta_c`;
//! * the shaft speed `N` ATTACHES, by inverting the speed line holding the pinned `(m, tau_c)`;
//! * **`tau_c` does not move.** The compressor WORK stays choke-pinned and map-free.
//!
//! Gates (`docs/rung32-spec.md` § Verification gates):
//!
//! 1. **REDUCE TO RUNG 31** — a FLAT map reproduces `OffDesignMatcher::match_point` across a
//!    throttle sweep, on the REACTING gas; `N` present but inert.
//! 2. **CYCLE UNTOUCHED** — the default design run is bit-for-bit rung 6, and building a
//!    `MapMatcher` does not perturb it.
//! 3. **THE FINDING (shape-robust)** — for 3 map shapes, `pi_c` AND `mdot` fall BELOW the flat-map
//!    values off design, same sign, gap growing with throttle.
//! 4. **WORK IS MAP-FREE** — `tau_c` matches rung 31's to `1e-4`, while `pi_c` moves 30x more.
//! 5. **TURBINE PINNED IN CORRECTED SPEED** — `nu_t` stays within 1 % of design, so `eta_t` barely
//!    moves EVEN for a 25x-steep turbine map.
//! 6. **`N` ATTACHES + MONOTONE** — `N/N_d = 1` at design, falls monotonically, and its schedule
//!    is robust across the speed-line `sigma` — bounded spread, but a NONZERO one.
//! 7. **DIRECTION / CONVERGENCE** — hotter `Tt4` => higher `pi_c`, `mdot`, `N`; the secant
//!    converges across the sweep.
//!
//! **THE PYTHON'S BARS ARE COPIED, INCLUDING THE ONES A SUMMARY WOULD DROP.** Three of them are
//! easy to lose and each is load-bearing in a different direction: gate 4's `dpc > 30*rel` holds
//! only at `Tt4 <= 1100` and is a CONDITIONAL; gate 6's spread bar is TWO-SIDED (`< 0.05` and
//! `> 1e-4`, and the lower half is what stops the robustness claim from being a tautology about a
//! quantity that never moved); and gate 1's specific-thrust bar is ABSOLUTE where its neighbours
//! are relative. `map_oracle.rs` being 100 % bit-exact says nothing about any of this — an oracle
//! cannot see a missing gate (§ 4.16).

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::Gas;
use turbojet::map::{ComponentMap, MapMatcher};
use turbojet::matcher::{Branch, OffDesignMatcher};

const PI_C: f64 = 10.0;
const TT4: f64 = 1500.0;
const ETA_C: f64 = 0.88;
const ETA_T: f64 = 0.90;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn real() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: ETA_C, eta_b: 0.99, pi_b: 0.96,
        eta_t: ETA_T, eta_m: 0.99, pi_n: 0.98,
        nozzle_convergent: true,
        ..Losses::default()
    }
}

fn shapes() -> [ComponentMap; 3] {
    [ComponentMap::flow_dominated(), ComponentMap::pressure_dominated(), ComponentMap::tilted()]
}

fn map_matcher(gas: Gas) -> MapMatcher {
    MapMatcher::new(build_turbojet(gas, PI_C, TT4, 50_000.0, real()), flight(), 1.0,
                    ComponentMap::flat())
}

fn r31_matcher(gas: Gas) -> OffDesignMatcher {
    OffDesignMatcher::new(build_turbojet(gas, PI_C, TT4, 50_000.0, real()), flight(), 1.0)
}

/// A map matcher + a rung-31 matcher on the FAST (thermally-perfect) gas.
///
/// The finding gates are gas-independent physics and the fast gas keeps them cheap: the reacting
/// gas re-freezes equilibrium per inner pass and the outer secant multiplies that. Python's
/// `_fast_matchers` makes the same call, and this port keeps it rather than "improving" the
/// coverage — running gates 3-7 on the equilibrium gas would change what is being measured
/// (see `map_oracle.rs`'s narrower equilibrium grid, same reason).
fn fast_matchers() -> (MapMatcher, OffDesignMatcher) {
    (map_matcher(Gas::thermally_perfect()), r31_matcher(Gas::thermally_perfect()))
}

// ------------------------------------------------------------------------------- gate 1
/// GATE 1 — the FLAT map reproduces rung 31 across a throttle sweep, on the REACTING gas.
///
/// The spine of the whole ladder: `X = None` must give the prior code path back. Here the flat
/// map makes the outer secant inert on pass 1, so what comes back is rung 31's own answer.
///
/// **The port measures this BIT-FOR-BIT and the Python does not** (`map_oracle.rs` § 5, 28 of 28
/// choked cells) — but only on the CHOKED branch, because rung 32 predates rung 33's dispatch.
/// The bars here are Python's, so that a future change failing the tighter claim still gets
/// compared against the number the source actually asserts.
#[test]
fn gate1_reduce_to_rung31() {
    let mm = map_matcher(Gas::reacting_equilibrium());
    let base = r31_matcher(Gas::reacting_equilibrium());
    let flat = ComponentMap::flat();
    for tt4 in [1500.0, 1200.0, 900.0] {
        let mo = mm.match_with(&flight(), tt4, &flat);
        let ro = base.match_point(&flight(), tt4);
        assert!((mo.base.pi_c - ro.pi_c).abs() <= 1e-9 * ro.pi_c, "flat pi_c != rung31 at {tt4}");
        assert!((mo.base.mdot_air - ro.mdot_air).abs() <= 1e-9 * ro.mdot_air,
                "flat mdot != rung31 at {tt4}");
        assert!((mo.base.tau_t - ro.tau_t).abs() <= 1e-9 * ro.tau_t, "flat tau_t != rung31 at {tt4}");
        // ABSOLUTE, where its neighbours are relative — Python's bar, kept as written.
        assert!((mo.base.performance.specific_thrust - ro.performance.specific_thrust).abs()
                <= 1e-6);
        for k in ["2", "3", "4", "5"] {
            let (a, b) = (mo.base.station(k), ro.station(k));
            assert!((a.tt - b.tt).abs() <= 1e-8 * b.tt, "station {k} Tt at {tt4}");
            assert!((a.pt - b.pt).abs() <= 1e-8 * b.pt, "station {k} pt at {tt4}");
        }
    }
    // At design the flat map returns pi_c = 10 and N = design. `n_corr` comes back at
    // 0.999999999999928 rather than exactly 1: it is the midpoint of a bracket the bisection
    // stops shrinking at 1e-14, so 1e-8 is the right order of bar and not a slack one.
    let od = mm.match_with(&flight(), TT4, &flat);
    assert!((od.base.pi_c - PI_C).abs() < 1e-8);
    assert!((od.n_ratio - 1.0).abs() < 1e-8);
    assert!((od.n_corr - 1.0).abs() < 1e-8);
}

// ------------------------------------------------------------------------------- gate 2
/// GATE 2 — the default design path is unchanged by rung 32 (bit-for-bit rung 6).
#[test]
fn gate2_cycle_untouched() {
    let plain = Losses { nozzle_convergent: false, ..real() };
    let r = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, 50_000.0, plain)
        .run(&flight(), 1.0);
    assert!((r.performance.specific_thrust - 798.37).abs() < 0.5,
            "{}", r.performance.specific_thrust);
    assert!(r.m9 > 1.8 && (r.p9 - 50_000.0).abs() < 1e-6);   // default nozzle: fully expanded
    // Building a MapMatcher (which runs a CONVERGENT design) must not perturb the default run.
    let _mm = MapMatcher::new(
        build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, 50_000.0, real()),
        flight(), 1.0, ComponentMap::flow_dominated());
    let r2 = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, 50_000.0, plain)
        .run(&flight(), 1.0);
    assert!((r2.performance.specific_thrust - r.performance.specific_thrust).abs() < 1e-9);
}

// --------------------------------------------------------------------- gate 3 (THE FINDING)
/// GATE 3 — a peaked map DROOPS `pi_c` AND `mdot` off design, same sign across 3 shapes.
///
/// This is rung 32's correction to rung 31: the map bites the pressure ratio and the mass flow
/// FIRST-ORDER, so "found without a map" was true only of the work.
#[test]
fn gate3_finding_pi_c_mdot_droop_shape_robust() {
    let (mm, base) = fast_matchers();
    for cmap in shapes() {
        let mut dpc_prev = 0.0;
        for tt4 in [1300.0, 1100.0, 900.0] {
            let mo = mm.match_with(&flight(), tt4, &cmap);
            let ro = base.match_point(&flight(), tt4);
            let dpc = (mo.base.pi_c - ro.pi_c) / ro.pi_c;
            let dmd = (mo.base.mdot_air - ro.mdot_air) / ro.mdot_air;
            assert!(dpc < 0.0, "pi_c must droop below rung31 off-design ({cmap:?}): {dpc}");
            assert!(dmd < 0.0, "mdot must droop below rung31 off-design: {dmd}");
            assert!(dpc < dpc_prev, "the droop must grow with throttle (deeper Tt4 => larger gap)");
            dpc_prev = dpc;
        }
        // At the design point the peaked map still sits AT the peak, so there is no droop —
        // which is what makes the droop above a statement about being off design.
        let od = mm.match_with(&flight(), TT4, &cmap);
        assert!((od.eta_c - ETA_C).abs() < 1e-6, "eta_c must equal design at the design point");
    }
}

// ------------------------------------------------------------------------------- gate 4
/// GATE 4 — `tau_c` (the compressor WORK) is choke-pinned: it matches rung 31 to ~`1e-4`.
///
/// The gate that ISOLATES what the map moves. Note the second bar is CONDITIONAL — `dpc > 30*rel`
/// is asserted only at `Tt4 <= 1100`, because near design both differences collapse toward zero
/// and their ratio stops meaning anything. Dropping the condition would make the gate fail for a
/// reason that has nothing to do with the claim.
#[test]
fn gate4_work_tau_c_is_map_free() {
    let (mm, base) = fast_matchers();
    for cmap in shapes() {
        for tt4 in [1300.0, 1100.0, 900.0] {
            let mo = mm.match_with(&flight(), tt4, &cmap);
            let ro = base.match_point(&flight(), tt4);
            let rel = (mo.base.tau_c - ro.tau_c).abs() / ro.tau_c;
            assert!(rel < 1e-4, "tau_c should be map-free ({cmap:?}, Tt4 {tt4}): rel {rel:.2e}");
            let dpc = (mo.base.pi_c - ro.pi_c).abs() / ro.pi_c;
            if tt4 <= 1100.0 {
                assert!(dpc > 30.0 * rel, "the map must move pi_c far more than tau_c");
            }
        }
    }
}

// ------------------------------------------------------------------------------- gate 5
/// GATE 5 — `nu_t` barely moves (single spool), so `eta_t` barely moves EVEN for a STEEP map.
///
/// `a_t = 0.5` is 25x the representative turbine curvature — the point being that the turbine
/// stays pinned for a STRUCTURAL reason (`nu_t = (N/N_d)·sqrt(Tt4_d/Tt4)` on one shaft), not
/// because the map was chosen flat.
#[test]
fn gate5_turbine_pinned_in_corrected_speed() {
    let (mm, _) = fast_matchers();
    let steep = ComponentMap { a: 0.25, b: 0.05, c: 0.0, sigma: 0.3, a_t: 0.5 };
    for tt4 in [1300.0, 1100.0, 900.0, 700.0] {
        let mo = mm.match_with(&flight(), tt4, &steep);
        assert!((mo.nu_t - 1.0).abs() < 0.01,
                "turbine corrected speed should stay within 1%: {}", mo.nu_t);
        let d_eta_t = (mo.eta_t - ETA_T).abs();
        let d_eta_c = (mo.eta_c - ETA_C).abs();
        assert!(d_eta_t < 1e-3, "turbine eta must barely move even for a steep map: {d_eta_t:.2e}");
        assert!(d_eta_t < 0.02 * d_eta_c, "turbine droop must be orders below the compressor droop");
    }
}

// ------------------------------------------------------------------------------- gate 6
/// GATE 6 — `N/N_d = 1` at design, falls monotonically, and its schedule is `sigma`-robust.
///
/// **The spread bar is TWO-SIDED and the lower half is the interesting one.** `< 0.05` says the
/// leading schedule does not depend on the loading law; `> 1e-4` says `N` genuinely DOES depend
/// on it — without which the robustness claim would be a tautology about a quantity that never
/// moved. `sigma` is the map's only genuine speed-line content (at `sigma = 0` the inversion
/// collapses to a closed-form square root), so a zero spread would mean the map bought nothing.
#[test]
fn gate6_n_attaches_monotone_and_schedule_robust() {
    let (mm, _) = fast_matchers();
    let flat = ComponentMap::flat();
    let od = mm.match_with(&flight(), TT4, &flat);
    assert!((od.n_ratio - 1.0).abs() < 1e-8, "N/N_d must equal 1 at the design point");
    let ns: Vec<f64> = [1500.0, 1300.0, 1100.0, 900.0, 700.0].iter()
        .map(|&t| mm.match_with(&flight(), t, &flat).n_ratio)
        .collect();
    assert!(ns.windows(2).all(|w| w[0] > w[1]), "N/N_d must fall monotonically as Tt4 falls");

    let variants: Vec<ComponentMap> = [0.0, 0.3, 0.6, 1.0].iter()
        .map(|&s| ComponentMap { sigma: s, ..ComponentMap::flat() })
        .collect();
    let n_sig: Vec<f64> = variants.iter()
        .map(|v| mm.match_with(&flight(), 900.0, v).n_ratio)
        .collect();
    let hi = n_sig.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let lo = n_sig.iter().copied().fold(f64::INFINITY, f64::min);
    let spread = (hi - lo) / n_sig[0];
    assert!(spread < 0.05, "N schedule should be robust across sigma (spread {spread:.3})");
    assert!(spread > 1e-4, "but N IS genuinely sigma-dependent (not a tautology)");
}

// ------------------------------------------------------------------------------- gate 7
/// GATE 7 — hotter `Tt4` => higher `pi_c`, `mdot`, `N`; the outer secant converges.
#[test]
fn gate7_direction_and_convergence() {
    let (mm, _) = fast_matchers();
    let cmap = ComponentMap::flow_dominated();
    let ods: Vec<_> = [1500.0, 1300.0, 1100.0, 900.0].iter()
        .map(|&t| mm.match_with(&flight(), t, &cmap))
        .collect();
    for w in ods.windows(2) {
        assert!(w[0].base.pi_c > w[1].base.pi_c, "pi_c must fall as Tt4 falls");
        assert!(w[0].base.mdot_air > w[1].base.mdot_air, "mdot must fall as Tt4 falls");
        assert!(w[0].n_ratio > w[1].n_ratio, "N must fall as Tt4 falls");
        assert!(w[0].base.nozzle_choked, "these points are on the choked branch");
    }
}

// ============================================================================================
// THE SLICE-I IOU, DISCHARGED
// ============================================================================================

/// `test_rung33.py`'s GATE 7, SECOND HALF — deferred in writing at `rung33.rs::slice_j_deferrals`
/// because it is a claim ABOUT rung 32, which did not exist in the Rust when slice I shipped.
///
/// Rung 32 overrides `match` and therefore does NOT inherit rung 33's subsonic dispatch: below
/// the unchoke boundary a map-matched point comes back flagged `nozzle_choked = false` **and**
/// labelled `Choked` — a label its own flag contradicts. That is not a defect to tidy; it is the
/// port faithfully reproducing that rung 32 is OLDER than rung 33, and subsonic + map is out of
/// scope in both languages.
///
/// The same operating point on rung 31's own matcher DOES dispatch, and that contrast is what
/// makes this a statement about inheritance rather than about the point being unremarkable.
#[test]
fn rung33_gate7_second_half_map_does_not_inherit_subsonic() {
    let mm = map_matcher(Gas::reacting_equilibrium());
    let deep = mm.match_with(&flight(), 560.0, &ComponentMap::flat());
    assert!(!deep.base.nozzle_choked, "560 K is below the unchoke boundary");
    assert_eq!(deep.base.branch, Branch::Choked,
               "rung 32 must NOT re-solve on rung 33's branch — it never sets the label at all");

    // The contrast: rung 31/33's matcher, same gas, same point, DOES dispatch.
    let r31 = r31_matcher(Gas::reacting_equilibrium());
    let same = r31.match_point(&flight(), 560.0);
    assert_eq!(same.branch, Branch::Subsonic,
               "the deferral is only meaningful if the ancestor dispatches here");
    assert!(!same.nozzle_choked);
}
