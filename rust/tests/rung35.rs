//! RUNG 35 — FUEL METERING: `Tt4` becomes an OUTPUT.
//!
//! Rung 34 commanded `Tt4(t)` by fiat. A real engine meters FUEL, and `Tt4` falls out of the
//! burner balance against the airflow the spool can currently pump. **That inversion is the
//! rung**, and its consequence is a CORRECTION of rung 34: a fuel step drives the airflow down
//! while `(1+f)` rises, so `f` spikes, `Tt4` overshoots, and the over-temperature amplifies the
//! very airflow deficit that sets the surge excursion. The two acceleration limits are COUPLED,
//! and commanding `Tt4` structurally hid it.
//!
//! The four gates of `tests/test_rung35.py`, in file order. All four port.
//!
//! | # | `tests/test_rung35.py` | here |
//! |---|---|---|
//! | 1 | `test_reduce_control_invariance` | [`gate1_control_invariance`] |
//! | 2 | `test_reduce_Tt4_control_untouched_and_cycle` | [`gate2_the_tt4_control_path_is_untouched`] |
//! | 3 | `test_the_finding_fuel_enlarges_surge_and_the_TIT_overshoot` | [`gate3_fuel_enlarges_surge_and_opens_the_tit_axis`] |
//! | 4 | `test_forward_burner_inverse_and_fuel_closure_recovers_point` | [`gate4_the_forward_burner_inverts_the_f_solve`] |

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::Gas;
use turbojet::map::{ComponentMap, MapMatcher};
use turbojet::spool::SpoolTransient;

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

fn design() -> Engine {
    build_turbojet(Gas::thermally_perfect(), PI_C, TT4, 50_000.0, real())
}

fn fast_transient(comp_map: ComponentMap) -> (SpoolTransient, MapMatcher) {
    (
        SpoolTransient::new(design(), flight(), 1.0, comp_map),
        MapMatcher::new(design(), flight(), 1.0, ComponentMap::flow_dominated()),
    )
}

fn surge_shapes() -> [ComponentMap; 3] {
    [ComponentMap::surge_flow(), ComponentMap::surge_pressure(), ComponentMap::surge_tilted()]
}

// ------------------------------------------------------------------------------------ gate 1
/// GATE 1 — the NON-TAUTOLOGICAL reduce. A steady point does not care whether it is named by its
/// `Tt4` or by its fuel flow.
///
/// Command the fuel `mdot_fuel = f_eq*mdot_air_eq` of a `Tt4`-control running-line point; the
/// fuel-control equilibrium must return the SAME `(nu, pi_c, tau_t, mdot_air)` and hand `Tt4` back
/// out at the commanded value — through the forward-burner-plus-fuel closure, which never pins
/// `Tt4` anywhere. **Two genuinely different closures onto one operating point**, which is the
/// only kind of reduce worth writing.
#[test]
fn gate1_control_invariance() {
    let (st, _) = fast_transient(ComponentMap::surge_tilted());
    let shape = ComponentMap::surge_tilted();
    for tt4 in [1500.0f64, 1300.0, 1100.0] {
        let eq_t = st.equilibrium(&flight(), tt4, Some(&shape));
        let mdot_fuel = eq_t.f * eq_t.mdot_air;
        let eq_f = st.equilibrium_fuel(&flight(), mdot_fuel, Some(&shape));
        // Machine-zero at design; tight on the sweep (nested root finds at 1e-11/1e-12).
        let tol = if tt4 == TT4 { 1e-9 } else { 1e-6 };
        assert!(
            (eq_f.nu - eq_t.nu).abs() < tol * eq_t.nu,
            "control-invariance nu at Tt4={tt4}: {} vs {}", eq_f.nu, eq_t.nu
        );
        assert!((eq_f.pi_c - eq_t.pi_c).abs() < tol * eq_t.pi_c, "control-invariance pi_c");
        assert!((eq_f.tau_t - eq_t.tau_t).abs() < tol * eq_t.tau_t, "control-invariance tau_t");
        assert!(
            (eq_f.mdot_air - eq_t.mdot_air).abs() < tol * eq_t.mdot_air,
            "control-invariance mdot"
        );
        assert!(
            (eq_f.tt4 - tt4).abs() < 1e-5,
            "Tt4 must fall back out of the fuel closure: {}", eq_f.tt4
        );
    }
    let mf_d = st.fuel_for_tt4(&flight(), TT4, Some(&shape));
    let eq_f_d = st.equilibrium_fuel(&flight(), mf_d, Some(&shape));
    assert!(
        (eq_f_d.nu - 1.0).abs() < 1e-9 && (eq_f_d.pi_c - PI_C).abs() < 1e-7,
        "design fuel-control point must be nu=1, pi_c=10: nu={}, pi_c={}", eq_f_d.nu, eq_f_d.pi_c
    );
}

// ------------------------------------------------------------------------------------ gate 2
/// GATE 2 — the `Tt4`-control path is UNTOUCHED (so rung 34 stays bit-for-bit) and the design
/// cycle is unperturbed. The rung-35 fuel methods are a separate entry point; adding them must not
/// move any steady number. The witness is that the `Tt4`-control equilibrium still reduces to
/// rung 32.
#[test]
fn gate2_the_tt4_control_path_is_untouched() {
    let (st, _) = fast_transient(ComponentMap::surge_tilted());
    let mm = MapMatcher::new(design(), flight(), 1.0, ComponentMap::flow_dominated());
    let shape = ComponentMap::surge_tilted();
    for tt4 in [1400.0f64, 1100.0] {
        let eq = st.equilibrium(&flight(), tt4, Some(&shape));
        let res = mm.match_with(&flight(), tt4, &shape);
        assert!(
            (eq.pi_c - res.base.pi_c).abs() < 1e-6 * res.base.pi_c,
            "Tt4-control equilibrium must still == rung-32 MapMatcher at Tt4={tt4}"
        );
        assert!(
            (eq.nu - res.n_ratio).abs() < 1e-6,
            "Tt4-control nu must still == rung-32 N_ratio"
        );
    }

    // Python re-runs ONE engine object before and after building a transient off it. The Rust
    // constructor CONSUMES its engine, so the equivalent statement is that two design runs off
    // two identically-built engines agree while a transient exists between them — which is the
    // same claim about global state and a weaker one about aliasing. Recorded, not glossed.
    let eng = design();
    let before = eng.run(&flight(), 1.0).performance.specific_thrust;
    let _ = SpoolTransient::new(design(), flight(), 1.0, ComponentMap::surge_flow());
    let after = eng.run(&flight(), 1.0).performance.specific_thrust;
    assert!(
        (after - before).abs() < 1e-12,
        "building a SpoolTransient must not perturb the design run"
    );
}

// ------------------------------------------------------------------------------------ gate 3
/// GATE 3 — THE RUNG. Two claims on the SAME fuel trajectory.
///
/// **(a) THE CORRECTION** — fuel control ENLARGES the surge excursion: `E_surge_fuel > E_Tt4` at
/// matched `r`, the gap largest at `r -> 0` and vanishing as `r -> inf`, and the SIGN robust
/// across three surge maps. Rung 34 under-counted surge because commanding `Tt4` suppressed the
/// over-temperature that amplifies the airflow deficit.
///
/// **(b) THE NEW AXIS** — the turbine-inlet-temperature overshoot `E_temp > 0` (`Tt4` floats above
/// its steady endpoint), monotone-decreasing in `r`, with the `r -> 0` limit an algebraic map
/// property. This axis does not exist under `Tt4` control, by construction.
#[test]
fn gate3_fuel_enlarges_surge_and_opens_the_tit_axis() {
    const LO: f64 = 1250.0;
    const HI: f64 = 1450.0;

    // (a1) SHAPE-ROBUST sign at r -> 0 — algebraic, no integration.
    for shape in surge_shapes() {
        let (st, _) = fast_transient(shape);
        let e0_t = st.constant_speed_excursion(&flight(), LO, HI, Some(&shape));
        let (e_surge0, e_temp0, _, _) =
            st.constant_speed_excursion_fuel(&flight(), LO, HI, Some(&shape));
        assert!(e0_t > 0.0, "the Tt4-control accel excursion must be positive: {e0_t} ({shape:?})");
        assert!(
            e_surge0 > e0_t + 1e-4,
            "fuel control must ENLARGE the surge excursion at r->0: {e_surge0} vs {e0_t}"
        );
        assert!(e_temp0 > 0.05, "the TIT overshoot must be a meaningful positive number: {e_temp0}");
    }

    // (a2)+(b) INTEGRATED: the gap persists at finite r and shrinks toward r -> inf.
    let shape = ComponentMap::surge_flow();
    let (st, _) = fast_transient(shape);
    let (e_surge0, e_temp0, _, _) =
        st.constant_speed_excursion_fuel(&flight(), LO, HI, Some(&shape));

    let fast = st.ramp_excursion_fuel(&flight(), LO, HI, 0.3, Some(&shape), 4.0, 0.1);
    let slow = st.ramp_excursion_fuel(&flight(), LO, HI, 3.0, Some(&shape), 4.0, 0.1);
    let et_fast = st.ramp_excursion(&flight(), LO, HI, 0.3, Some(&shape), 4.0, 0.1).e;
    let et_slow = st.ramp_excursion(&flight(), LO, HI, 3.0, Some(&shape), 4.0, 0.1).e;

    assert!(fast.e_surge <= e_surge0 + 1e-6, "E_surge(r) must not exceed the r->0 limit");
    assert!(fast.e_temp <= e_temp0 + 1e-6, "E_temp(r) must not exceed the r->0 limit");
    assert!(fast.e_surge > slow.e_surge, "E_surge must fall with r: {} !> {}", fast.e_surge, slow.e_surge);
    assert!(fast.e_temp > slow.e_temp, "E_temp must fall with r: {} !> {}", fast.e_temp, slow.e_temp);
    assert!(
        fast.e_surge > et_fast + 1e-4,
        "fuel control must enlarge the surge excursion at r=0.3: {} vs {et_fast}", fast.e_surge
    );
    let gap_fast = fast.e_surge - et_fast;
    let gap_slow = slow.e_surge - et_slow;
    assert!(gap_slow < gap_fast, "the surge-excursion gap must shrink with r: {gap_slow} !< {gap_fast}");
    assert!(gap_slow < 0.4 * gap_fast, "the gap must nearly close at r=3: {gap_slow} vs {gap_fast}");
}

// ------------------------------------------------------------------------------------ gate 4
/// GATE 4 — the INSTANT-LEVEL inverse: the fuel/`Tt4` analogue of rung 34's map-inverse gate.
///
/// **(a)** The forward burner `Tt4(f)` is the EXACT inverse of the shipped burner `f`-solve —
/// solving `f` back from `Tt4(f)` recovers it to machine zero.
/// **(b)** At a FIXED shaft speed, closing the compressor with the fuel of a `Tt4`-control instant
/// recovers that instant's `(Tt4, pi_c, mdot_air)`. The two closures agree OFF the running line
/// too, which gate 1 (an equilibrium comparison) cannot say.
#[test]
fn gate4_the_forward_burner_inverts_the_f_solve() {
    let (st, _) = fast_transient(ComponentMap::surge_tilted());
    let shape = ComponentMap::surge_tilted();

    for (tt3, f) in [(650.0f64, 0.020f64), (700.0, 0.025), (600.0, 0.030)] {
        let tt4 = st.tt4_from_f(tt3, f);
        // pt4 is inert for the non-equilibrium burner f-solve.
        let f_back = st.inner.inner.solve_f(tt3, 1.0e6, tt4);
        assert!((f_back - f).abs() < 1e-10, "Tt4(f) must invert the burner f-solve: {f_back} vs {f}");
    }

    let nu = 0.92;
    let inst = st.instant(&flight(), nu, 1350.0, Some(&shape));
    let mdot_fuel = inst.f * inst.mdot_air;
    let inst_f = st.instant_fuel(&flight(), nu, mdot_fuel, Some(&shape));
    assert!((inst_f.tt4 - 1350.0).abs() < 1e-6, "fuel closure Tt4 off-line: {}", inst_f.tt4);
    assert!(
        (inst_f.pi_c - inst.pi_c).abs() < 1e-8 * inst.pi_c,
        "fuel closure pi_c off-line"
    );
    assert!(
        (inst_f.mdot_air - inst.mdot_air).abs() < 1e-8 * inst.mdot_air,
        "fuel closure mdot off-line"
    );
}

/// The record of what ported.
#[test]
fn rung35_roster() {
    let roster: [(&str, bool); 4] = [
        ("test_reduce_control_invariance", true),
        ("test_reduce_Tt4_control_untouched_and_cycle", true),
        ("test_the_finding_fuel_enlarges_surge_and_the_TIT_overshoot", true),
        ("test_forward_burner_inverse_and_fuel_closure_recovers_point", true),
    ];
    assert_eq!(roster.len(), 4, "tests/test_rung35.py has 4 test functions");
    assert_eq!(roster.iter().filter(|(_, p)| *p).count(), 4, "all four port");
    println!("rung35.rs: 4 ported + 0 added, 0 slow markers");
}
