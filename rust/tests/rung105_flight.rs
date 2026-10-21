//! RUNG 105's seam — the HP vane's law on the paths its rung left unwired (spec § The next seam, its first bullet; R1b's
//! "law refusing on a path not wired for it"). A repair, not a rung: `docs/rung105-spec.md` § The paths wired after the
//! rung.
//!
//! * **Rung 37's metal soak** now runs the law: its closure reads rung 35's vane flow at the turbine-inlet temperature
//!   the metal leaves. Gate: rung 37's own reduce (the soak equilibrium IS rung 35's) holds under the law, and the law
//!   moved the point (not a pin in disguise).
//! * **The two-shaft fuel rigs (rungs 43–64)** read the flight's `p0` at each closure's ENTRY, only for the compressor
//!   face the engine's own inlet step gave: a missing or another flight's pressure is refused in words, never swallowed
//!   by a trial-skipping scan and re-worded "does not bracket". Rung 57's arrow toggle hands its sibling the flight.
//! * **Rung 102's one-shaft afterburner** refuses the law in its own words at every entry (its turbine needs the nozzle
//!   behind the afterburner, a solve not built), before any speed search runs an instant.

use turbojet::afterburner::{AbMode, Afterburner, FlyAfterburner, Throat};
use turbojet::combustor::CombustorTransient;
use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::fuel_transient::TwoSpoolFuelTransient;
use turbojet::gas::{Gas, GasSpec};
use turbojet::hp_vane::{census, HpVane};
use turbojet::map::ComponentMap;
use turbojet::sandbox::{fly_solver, FlySettings};
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const R30: HpVane = HpVane::Rows { reaction: 0.30 };

fn rel(a: f64, b: f64) -> f64 { (a / b - 1.0).abs() }
fn fd() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn cpg() -> Gas {
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0, gamma_t: 1.3, cp_t: 1239.0,
                       r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() })
}

// ------------------------------------------------------------------------------------------------- rung 37's soak

/// Rung 37's rig (`tests/rung37.rs`): the thermally perfect one-shaft engine at `π_c` 10, `Tt4` 1500 K.
fn soak_rig(law: HpVane) -> CombustorTransient {
    let l = Losses { pi_d: 0.97, eta_c: 0.88, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, e_t: None, eta_m: 0.99,
                     pi_n: 0.98, p_exit: None, nozzle_convergent: true };
    let d: Engine = build_turbojet(Gas::thermally_perfect(), 10.0, 1500.0, 50_000.0, l);
    let ct = CombustorTransient::new(d, fd(), 1.0, ComponentMap::surge_flow(), 0.0, 0.1, 3.0);
    ct.inner.inner.inner.set_hp_vane(law);
    ct
}

#[test]
fn the_soak_equilibrium_is_rung_35s_under_the_law() {
    let cmap = ComponentMap::surge_flow();
    let (pin, law) = (soak_rig(HpVane::Pin), soak_rig(R30));
    for tt4 in [1400.0f64, 1100.0] {
        let mf = law.inner.fuel_for_tt4(&fd(), tt4, Some(&cmap));
        census::take();
        let a = law.equilibrium_soak(&fd(), mf, Some(&cmap));
        let vane_flows = census::take()[1];
        let b = law.inner.equilibrium_fuel(&fd(), mf, Some(&cmap));
        let p = pin.equilibrium_soak(&fd(), pin.inner.fuel_for_tt4(&fd(), tt4, Some(&cmap)), Some(&cmap));
        eprintln!("soak Tt4 {tt4}: vane flows {vane_flows}; soak vs rung 35 pi_c {:.2e} nu {:.2e}; law vs pin pi_c {:.2e}",
                  rel(a.inst.pi_c, b.pi_c), (a.inst.nu - b.nu).abs(), rel(a.inst.pi_c, p.inst.pi_c));
        assert!(vane_flows > 0, "the soak closure read the vane's law (not the pin's line)");
        assert!(rel(a.inst.pi_c, b.pi_c) <= 1e-9 && (a.inst.nu - b.nu).abs() <= 1e-9, "Tt4 {tt4}: soak != rung 35");
        assert!(rel(a.inst.pi_c, p.inst.pi_c) > 1e-6, "Tt4 {tt4}: the law moved nothing — a pin in disguise");
    }
}

#[test]
fn the_soak_excursion_runs_under_the_law() {
    let cmap = ComponentMap::surge_flow();
    let law = soak_rig(R30);
    let cold = law.soak_excursion(&fd(), 1100.0, 1400.0, turbojet::combustor::Theta0::Cold, Some(&cmap), 0.05, 6.0);
    let hot = law.soak_excursion(&fd(), 1100.0, 1400.0, turbojet::combustor::Theta0::Hot, Some(&cmap), 0.05, 6.0);
    let ad = law.adiabatic_excursion(&fd(), 1100.0, 1400.0, Some(&cmap), 0.05, 6.0);
    eprintln!("soak excursion under the law: cold {} hot {} adiabatic {}", cold.e_surge, hot.e_surge, ad.e_surge);
    assert!(cold.e_surge < hot.e_surge && hot.e_surge < ad.e_surge, "rung 37's ordering under the law");
}

// ------------------------------------------------------------------------------------------- the two-shaft fuel rigs

fn rig_losses() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
fn lp_map() -> ComponentMap { ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() } }
fn hp_map() -> ComponentMap { ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() } }
/// Rung 43's rig (`π` 3/6, `Tt4` 1500 K) on the thermally perfect gas, where the law moves the point.
fn fuel_rig(law: HpVane) -> TwoSpoolFuelTransient {
    let d: TwoSpoolEngine = build_two_spool_turbojet(Gas::thermally_perfect(), 3.0, 6.0, 1500.0, 50_000.0, rig_losses())
        .with_hp_vane(law);
    TwoSpoolFuelTransient::new(d, fd(), 1.0, lp_map(), hp_map(), 1.0)
}

#[test]
fn rung_43s_fuel_equilibrium_is_rung_40s_point_under_the_law() {
    // Rung 43's gate 1 (two closures, one point) with the law on: before the repair the fuel closure's scan swallowed
    // the law's refusal and said "does not bracket".
    let (pin, law) = (fuel_rig(HpVane::Pin), fuel_rig(R30));
    for tt4 in [1500.0f64, 1300.0, 1100.0] {
        let eq = law.core().inner.equilibrium(&fd(), tt4);
        let mf = eq.close.f * eq.close.mdot_air;
        census::take();
        let (fq, _) = law.core().equilibrium_fuel(&fd(), mf, None);
        let vane_flows = census::take()[1];
        let ep = pin.core().inner.equilibrium(&fd(), tt4);
        eprintln!("rung 43 Tt4 {tt4}: vane flows {vane_flows}; fuel vs Tt4 control nu_lp {:.2e} nu_hp {:.2e} Tt4 {:.2e}; \
                   law vs pin pi_hpc {:.2e}", rel(fq.base.nu_lp, eq.nu_lp), rel(fq.base.nu_hp, eq.nu_hp),
                  rel(fq.base.tt4, tt4), rel(eq.close.pi_hpc, ep.close.pi_hpc));
        assert!(vane_flows > 0, "the fuel closure read the vane's law");
        assert!(rel(fq.base.nu_lp, eq.nu_lp) < 1e-10 && rel(fq.base.nu_hp, eq.nu_hp) < 1e-10 && rel(fq.base.tt4, tt4) < 1e-10,
                "Tt4 {tt4}: the fuel equilibrium is not rung 40's point under the law");
        // At the design throttle the law IS the design run (rung 105's R3); off it, the law must have moved the point.
        assert!(tt4 == 1500.0 || rel(eq.close.pi_hpc, ep.close.pi_hpc) > 1e-6, "Tt4 {tt4}: the law moved nothing");
    }
}

#[test]
fn a_closure_fed_another_flights_face_is_refused_in_words() {
    let fl_b = FlightCondition::new(288.15, 101_325.0, 0.5);
    for law in [HpVane::Pin, R30] {
        let t = fuel_rig(law);
        let c = t.core();
        let (tt2_b, pt2_b, _) = c.inner.inlet(&fl_b);
        let (tt2_a, pt2_a, _) = c.inner.inlet(&fd()); // the engine's inlet step last read flight A
        let eq = c.inner.equilibrium(&fd(), 1300.0);
        let mf = eq.close.f * eq.close.mdot_air;
        c.inner.inlet(&fd());
        let own = c.try_close_fuel(eq.nu_lp, eq.nu_hp, mf, tt2_a, pt2_a);
        let other = c.try_close_fuel(eq.nu_lp, eq.nu_hp, mf, tt2_b, pt2_b);
        assert!(own.is_ok(), "{law:?}: its own face closes: {:?}", own.err());
        match law {
            // The pin never reads the flight: another face is just another closure.
            HpVane::Pin => assert!(!matches!(&other, Err(e) if e.0.contains("rung-105")), "{:?}", other.err()),
            _ => {
                let e = other.err().expect("another flight's face under the law must be refused").0;
                assert!(e.starts_with("rung-105: rung 43's fuel closure") && e.contains("ambient pressure"), "{e}");
            }
        }
    }
}

#[test]
fn a_fresh_engines_closure_names_the_law_not_a_bracket() {
    // R1b's swallowed class: a closure called on an engine whose inlet step never ran. The scan used to skip every
    // refusing trial and report "does not bracket"; the refusal now comes from the entry, in the law's words.
    let t = fuel_rig(R30);
    let fresh = fuel_rig(R30);
    let (tt2, pt2, _) = t.core().inner.inlet(&fd());
    let e = fresh.core().try_close_fuel(0.85, 0.88, 0.0154, tt2, pt2).err().expect("refused").0;
    assert!(e.starts_with("rung-105: rung 43's fuel closure") && !e.contains("does not bracket"), "{e}");
}

fn stator(law: HpVane) -> ScheduledStatorCore {
    let d = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, rig_losses()).with_hp_vane(law);
    let (lp, hp) = (lp_map().with_phi_surge(0.55), hp_map().with_phi_surge(0.55));
    match ScheduledStatorTransient::new(d, fd(), 1.0, Some(lp), Some(hp), 1.0, StatorArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("both shafts"),
    }
}

#[test]
fn rung_57s_arrow_toggle_hands_its_sibling_the_flight() {
    // The one library path R1b's probe found without the flight: the toggle closes a flow on a sibling machine it
    // builds itself. On the perfect gas with the nozzle choked the law's own search lands on the pin's expansion (rung
    // 105's R2, bar 1e-12) — off the design speeds the turbines leave their design efficiency, so the shortcut is off
    // and the law's search runs (measured ≤ 9.3e-14 here). Before the repair it refused.
    let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds: 0.01 };
    let p = stator(HpVane::Pin).arrow_toggle(&fd(), &ramp, 0.20, Spool::Lp, None);
    census::take();
    let l = stator(R30).arrow_toggle(&fd(), &ramp, 0.20, Spool::Lp, None);
    assert!(census::take()[1] > 0, "the toggle read the vane's law");
    for (k, a, b) in [("d_phi_lp", p.d_phi_lp, l.d_phi_lp), ("d_phi_hp", p.d_phi_hp, l.d_phi_hp), ("d_tt25", p.d_tt25, l.d_tt25),
                      ("nu_lp", p.state.0, l.state.0), ("nu_hp", p.state.1, l.state.1), ("mf", p.state.2, l.state.2)] {
        assert!(rel(b, a) <= 1e-12, "{k}: pin {a:e} law {b:e}");
    }
}

// ------------------------------------------------------------------------------------------ rung 102's afterburner

#[test]
fn the_one_shaft_afterburner_refuses_the_law_in_its_own_words_before_any_search() {
    let s = FlySettings::defaults();
    let (fl, cmap) = (s.flight(), s.map.map().with_phi_surge(s.phi_surge));
    let st = fly_solver(&s);
    st.inner.inner.set_hp_vane(R30);
    let fa = FlyAfterburner::capture(&st, Afterburner::default());
    turbojet::afterburner::take_census();
    for (mode, throat) in [(AbMode::Unlit, Throat::Held(1.0)), (AbMode::Lit, Throat::Held(1.0)), (AbMode::Lit, Throat::Match)] {
        let e = fa.try_point(&fl, s.tt4, &cmap, mode, throat).err().expect("refused").0;
        assert!(e.starts_with("rung-105: rung 102's afterburner") && !e.contains("bracket"), "{e}");
    }
    assert_eq!(turbojet::afterburner::take_census(), (0, 0), "no trial and no instant ran before the refusal");
    st.inner.inner.set_hp_vane(HpVane::Pin);
    assert!(fa.try_point(&fl, s.tt4, &cmap, AbMode::Unlit, Throat::Held(1.0)).is_ok(), "the pin still runs");
}
