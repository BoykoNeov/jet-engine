//! **Rung 34's low-speed wall — the repair's gate (2026-10-21; `docs/rung34-spec.md` § The low-speed wall).**
//!
//! Below about 0.6 of design speed the instant's choked-star rebuild (the turbine at the choke `pi_t`) cannot
//! reach the ambient pressure, and the nozzle refused before the subsonic solve could run. On a hot day at a low
//! throttle the running point sits just above that wall, so the speed search refused points that run. The search
//! now runs UNCHANGED first; only where it refuses does a second pass search on the instant with the wall lifted.
//!
//! 1. **Non-vacuous:** on these cells the first pass (the old search) DOES refuse.
//! 2. **The stable running point:** the second pass's answer is subsonic, with thrust, and the shaft speeds up
//!    just below it and slows just above it (`Phi` crosses zero downward).
//! 3. **Where the first pass runs, nothing moves** — the equilibrium is the old search's root, bit for bit.
//! 4. **Below the lowest running throttle it still refuses, in the first pass's own words.**
//!
//! On the flat map the second pass's answer is rung 33's working root: `tests/rung33_double_root.rs`.

use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::sandbox::{FlySettings, GasModel};
use turbojet::spool::SpoolTransient;

fn hot() -> FlightCondition {
    FlightCondition::new(318.15, 101_325.0, 0.30)
}

/// The Fly-it engine's design (the page's opening one) on `gas`, with `cmap`.
fn spool(gas: GasModel, cmap: Option<ComponentMap>) -> SpoolTransient {
    let s = FlySettings { gas, ..FlySettings::defaults() };
    let d = s.capture();
    let e = build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses());
    SpoolTransient::new(e, d.flight(), d.mdot, cmap.unwrap_or_else(|| s.map.map()))
}

/// The cells: the page's map at the two lowest running throttles on this day, and the flat map below the
/// first pass's reach on both gases.
fn cells() -> Vec<(&'static str, SpoolTransient, f64)> {
    let mut v = Vec::new();
    for tt4 in [700.0, 725.0] {
        v.push(("page map, thermally perfect", spool(GasModel::ThermallyPerfect, None), tt4));
    }
    for g in [GasModel::ThermallyPerfect, GasModel::Perfect] {
        for tt4 in [675.0, 700.0] {
            v.push((if g == GasModel::Perfect { "flat map, perfect" } else { "flat map, thermally perfect" },
                    spool(g, Some(ComponentMap::flat())), tt4));
        }
    }
    v
}

/// The old search's answer, or its refusal's words.
fn first_pass(st: &SpoolTransient, fl: &FlightCondition, tt4: f64) -> Result<f64, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        st.find_equilibrium_nu(|nu| Ok(st.try_instant(fl, nu, tt4, None)?.phi))
    }))
    .map_err(|e| e.downcast_ref::<String>().cloned().unwrap_or_default())
}

#[test]
fn the_old_search_refuses_these_cells() {
    for (what, st, tt4) in cells() {
        let words = first_pass(&st, &hot(), tt4).expect_err(&format!("{what} {tt4}: the old search runs — vacuous"));
        assert!(words.starts_with("rung-34 equilibrium does not bracket"), "{what} {tt4}: {words}");
    }
}

#[test]
fn the_second_pass_finds_the_stable_running_point() {
    for (what, st, tt4) in cells() {
        let eq = st.equilibrium(&hot(), tt4, None);
        let phi = |nu: f64| st.try_instant_below_choke(&hot(), nu, tt4, None).expect("evaluable beside the root").phi;
        let (below, above) = (phi(eq.nu - 0.01), phi(eq.nu + 0.01));
        println!("{what} {tt4}: nu {:.4}, pi_c {:.4}, Phi {below:+.5} / {above:+.5}, thrust {:.1}", eq.nu, eq.pi_c, eq.thrust);
        assert_eq!(eq.branch, Branch::Subsonic, "{what} {tt4}");
        assert!(eq.thrust > 0.0, "{what} {tt4}: no thrust");
        assert!(below > 0.0 && above < 0.0, "{what} {tt4}: not the stable point ({below}, {above})");
    }
}

#[test]
fn where_the_old_search_runs_nothing_moves() {
    let st = spool(GasModel::ThermallyPerfect, None);
    let design = FlySettings::defaults().flight();
    for (fl, tt4) in [(design, 900.0), (design, 1300.0), (hot(), 800.0), (hot(), 1100.0)] {
        let old = first_pass(&st, &fl, tt4).expect("the old search runs here");
        let eq = st.equilibrium(&fl, tt4, None);
        assert_eq!(eq.nu.to_bits(), old.to_bits(), "{tt4}: the equilibrium moved off the old search's root");
    }
}

#[test]
fn below_the_lowest_running_throttle_it_refuses_in_the_old_words() {
    let st = spool(GasModel::ThermallyPerfect, None);
    let old = first_pass(&st, &hot(), 650.0).expect_err("650 K: the old search runs — pick a lower throttle");
    let now = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| st.equilibrium(&hot(), 650.0, None)))
        .err().and_then(|e| e.downcast_ref::<String>().cloned()).expect("650 K is below the lowest running throttle");
    assert_eq!(now, old, "the refusal's words moved");
}

/// The afterburner's own speed search (rung 102, Fly it with an afterburner fitted) is a copy of rung 34's and had the
/// same wall; it has the same second pass. Unlit on the same hot day the duct's loss raises the lowest running throttle
/// (700 K is below idle there), so the cell is 725 K.
#[test]
fn the_afterburner_search_has_the_same_second_pass() {
    use turbojet::afterburner::{try_find_equilibrium_nu, AbMode, FlyAfterburner, Throat};
    use turbojet::sandbox::fly_solver;
    let s = FlySettings { gas: GasModel::ThermallyPerfect, t0: 318.15, p0: 101_325.0, m0: 0.3, tt4: 725.0,
                          ab: Some(AbMode::Unlit), ..FlySettings::defaults() };
    let (st, fl, cmap) = (fly_solver(&s), s.flight(), s.map.map().with_phi_surge(s.phi_surge));
    let fa = FlyAfterburner::capture(&st, s.afterburner());
    let old = try_find_equilibrium_nu(|nu| Ok(fa.try_instant(&fl, nu, s.tt4, &cmap, AbMode::Unlit, fa.a8_dry)?.eq.phi));
    assert!(old.is_err(), "the old search runs here — vacuous");
    let p = fa.try_point(&fl, s.tt4, &cmap, AbMode::Unlit, Throat::Held(1.0)).expect("725 K runs");
    assert!(fa.try_instant(&fl, p.eq.nu, s.tt4, &cmap, AbMode::Unlit, fa.a8_dry).is_err(),
            "the old instant solves the answer's own speed — the second pass did not find it");
    let phi = |nu: f64| fa.try_instant_below_choke(&fl, nu, s.tt4, &cmap, AbMode::Unlit, fa.a8_dry).expect("evaluable").eq.phi;
    let (below, above) = (phi(p.eq.nu - 0.01), phi(p.eq.nu + 0.01));
    println!("afterburner unlit 725: nu {:.4}, pi_c {:.4}, Phi {below:+.5} / {above:+.5}, thrust {:.1}", p.eq.nu, p.eq.pi_c, p.eq.thrust);
    assert_eq!(p.eq.branch, Branch::Subsonic);
    assert!(p.eq.thrust > 0.0 && below > 0.0 && above < 0.0, "not the stable running point ({below}, {above})");
}
