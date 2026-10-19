//! RUNG 103 — THE STEADY TWO-SHAFT MATCHERS BELOW CHOKE: the refusal becomes a solve.
//!
//! Anchor: `docs/plans/rung103-anchor-steady-unchoked.md` (frozen as `8bf63b9` before any rung-103 code). Spec:
//! `docs/rung103-spec.md`. Gates R1–R8 and the two scored claims, at the anchor's own cells and bars:
//!
//!   R1  the switch is off by default on every matcher (and then nothing moves: the whole gate);
//!   R2  switch on at a choked point = switch off, bit for bit, every variant;
//!   R3  an INDEPENDENT root — the LP turbine held at a trial `π`, an outer bisection on the nozzle's mass balance read
//!       off the result — equals the shipped `π_LPT` (≤ 1e−11); calibrated on choked cells first;
//!   R4  rung 40's transient equilibrium (rung 97's branch) at the same `Tt4` (≤ 1e−8), its choked floor measured;
//!   R5  a hand-written perfect-gas solve of the whole steady point (≤ 1e−9);
//!   R6  the boundary: just below unchoke the branch meets the pin (≤ 1e−9);
//!   R7  mass at the nozzle (≤ 1e−11);
//!   R8  no re-solve state is left behind — success, refusal, panic;
//!   H1  THE HEADLINE — below choke a better LP compressor LOWERS the HP pressure ratio (38 cells);
//!   H2  the LP shaft runs slower than the pin said (174 cells, six variants).

use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::components::ram_recovery;
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::steady_unchoked::SteadyLp;
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolCore, TwoSpoolLosses, TwoSpoolMapCore, TwoSpoolMapMatcher,
                          TwoSpoolMapResult, TwoSpoolMatcher};
use turbojet::two_spool_transient::TwoSpoolTransientCore;
use turbojet::unchoked_lp::PI_TOP;

// --------------------------------------------------------------------------------- the cells

fn flight(dt: f64, h: f64, m0: f64) -> FlightCondition {
    let (t, p) = standard(h);
    FlightCondition::new(t + dt, p, m0)
}
/// H2's three flights (anchor § 3).
fn h2_flights() -> [(&'static str, FlightCondition); 3] {
    [("SL+20 M0.60", flight(20.0, 0.0, 0.60)), ("2km M0.50", flight(0.0, 2000.0, 0.50)),
     ("SL-10 M0.40", flight(-10.0, 0.0, 0.40))]
}
fn h2_tt4() -> Vec<f64> { (0..6).map(|i| 575.0 + 50.0 * i as f64).collect() }

/// The Controls engine's six steady variants at H2's settings, on one map shape.
fn variants(shape: &str) -> Vec<(&'static str, TwoSpoolMapCore)> {
    let bs = BladeSettings::defaults();
    let fd = bs.flight();
    let (ml, mh) = shape_maps(shape).unwrap();
    let d = || bs.design();
    vec![
        ("plain", match TwoSpoolMapMatcher::new(d(), fd, 1.0, ml, mh) { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!() }),
        ("bleed", TwoSpoolBleedMatcher::new(d(), fd, 1.0, ml, mh, 0.08).core),
        ("customer", CustomerBleedMatcher::new(d().with_customer_bleed(CustomerBleed::share(0.03)), fd, 1.0, ml, mh, 0.0).core),
        ("cooling", TurbineCoolingMatcher::new(d().with_turbine_cooling(TurbineCooling::new(0.02, 0.04)), fd, 1.0, ml, mh, 0.0).core),
        ("gen-HP", ShaftOfftakeMatcher::new(d().with_shaft_offtake(ShaftOfftake::hp(Demand::Power(0.02))), fd, 1.0, ml, mh, 0.0).core),
        ("gen-LP", ShaftOfftakeMatcher::new(d().with_shaft_offtake(ShaftOfftake::lp(Demand::Power(0.02))), fd, 1.0, ml, mh, 0.0).core),
    ]
}

fn off(c: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64) -> Result<TwoSpoolMapResult, Abort> { c.try_match_point(fl, tt4) }
fn on(c: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64) -> Result<TwoSpoolMapResult, Abort> {
    c.base.below_choke.set(true);
    let r = c.try_match_point(fl, tt4);
    c.base.below_choke.set(false);
    r
}
fn pin(c: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64) -> Result<TwoSpoolMapResult, Abort> {
    c.base.pin_past_choke.set(true);
    let r = c.try_match_point(fl, tt4);
    c.base.pin_past_choke.set(false);
    r
}
fn unchoked(r: &Result<TwoSpoolMapResult, Abort>) -> bool { matches!(r, Err(e) if e.0.contains("UNCHOKED")) }
/// The anchor's scored-cell rule: refused unchoked OFF, solves ON, and the pin runs.
fn scored(c: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64) -> Option<(TwoSpoolMapResult, TwoSpoolMapResult)> {
    if !unchoked(&off(c, fl, tt4)) { return None }
    Some((on(c, fl, tt4).ok()?, pin(c, fl, tt4).ok()?))
}
fn clean(b: &TwoSpoolCore) -> bool { b.steady_lp.get() == SteadyLp::Pin && !b.dispatching.get() }

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
fn cpg() -> Gas {
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9, gamma_t: 1.3, cp_t: 1239.0,
                       r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() })
}
fn fd39() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
/// Rung 39's own engine (H1, R5).
fn mm39(gas: Gas, ml: ComponentMap, mh: ComponentMap) -> TwoSpoolMapMatcher {
    TwoSpoolMapMatcher::new(build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, 50_000.0, real()), fd39(), 1.0, ml, mh)
}

// --------------------------------------------------------------------------------- R1, R2, R8

#[test]
fn r1_the_switch_is_off_by_default_on_every_matcher() {
    for shape in ["steep", "flat-eta"] {
        for (n, c) in variants(shape) {
            assert!(!c.base.below_choke.get() && !c.base.pin_past_choke.get() && clean(&c.base), "{n}");
        }
    }
    let m = TwoSpoolMatcher::new(build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, real()), fd39(), 1.0);
    assert!(!m.core().below_choke.get());
}

#[test]
fn r2_choked_points_are_bit_for_bit_with_the_switch_on_and_r8_no_state_is_left() {
    let mut n = 0;
    for shape in ["steep", "flat-eta"] {
        for (vn, c) in variants(shape) {
            for (fname, fl) in h2_flights() {
                for tt4 in h2_tt4().into_iter().chain([850.0, 900.0, 1000.0, 1100.0, 1300.0]) {
                    let a = off(&c, &fl, tt4);
                    let b = on(&c, &fl, tt4);
                    assert!(clean(&c.base), "{vn} {fname} {tt4}: state left after the call");
                    let Ok(a) = a else { continue };
                    let b = b.unwrap_or_else(|e| panic!("{vn} {fname} {tt4}: OFF ran, ON refused: {}", e.0));
                    for (k, x, y) in [("thrust", a.base.thrust, b.base.thrust), ("nL", a.n_lp_ratio, b.n_lp_ratio),
                                      ("nH", a.n_hp_ratio, b.n_hp_ratio), ("phiL", a.phi_lp, b.phi_lp),
                                      ("phiH", a.phi_hp, b.phi_hp), ("piLPC", a.base.pi_lpc, b.base.pi_lpc),
                                      ("piHPC", a.base.pi_hpc, b.base.pi_hpc)] {
                        assert_eq!(x.to_bits(), y.to_bits(), "{vn} {shape} {fname} {tt4}: {k}");
                    }
                    n += 1;
                }
            }
        }
    }
    assert!(n >= 100, "only {n} choked cells");
}

#[test]
fn r8_a_panicking_rung38_branch_leaves_no_state() {
    // Rung 38's matcher is infallible: a below-idle re-solve panics in its own words. Find one and check the state.
    let m = TwoSpoolMatcher::new(build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, real()), fd39(), 1.0);
    let c = m.core();
    c.below_choke.set(true);
    let hush = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut words = Vec::new();
    for tt4 in [300.0, 350.0, 400.0, 450.0] {
        if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.match_point(&fd39(), tt4))) {
            words.push(e.downcast_ref::<String>().cloned().unwrap_or_default());
        }
        assert!(clean(c), "state left at {tt4}");
    }
    std::panic::set_hook(hush);
    c.below_choke.set(false);
    assert!(words.iter().any(|w| w.contains("rung-103: below idle") || w.contains("rung-97")),
            "no branch failure reached: {words:?}");
}

// --------------------------------------------------------------------------------- R3

/// The independent instrument: the variant's own body with the LP turbine HELD at `pi`, the nozzle's mass residual off
/// the result (station-5 flow against `A8·ρ9·V9`). Never calls rung 97's helper.
fn held_resid(c: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64, pi: f64) -> Option<f64> {
    c.base.steady_lp.set(SteadyLp::Held(pi));
    let r = c.try_match_point(fl, tt4);
    c.base.steady_lp.set(SteadyLp::Pin);
    let r = r.ok()?;
    let s5 = *r.base.station("5");
    let demand = c.base.a8 * r.base.p9 / (c.gas().r_t_at(s5.far) * r.base.t9) * r.base.v9;
    Some((s5.mdot - demand) / s5.mdot)
}

fn held_root(c: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64, lo: f64) -> Option<f64> {
    let rlo = held_resid(c, fl, tt4, lo)?;
    // The top marched down (rung 33's march): with a generator an LP turbine barely expanding cannot carry the load.
    let mut hi = PI_TOP;
    let rhi = loop {
        if let Some(r) = held_resid(c, fl, tt4, hi) { break r }
        hi -= 0.01;
        if hi <= lo { return None }
    };
    // Choked, the residual at the pin is rounding (±1e−15): the pin IS the root.
    if rlo.abs() <= 1e-12 { return Some(lo) }
    if rlo * rhi > 0.0 { return None }
    let mut lo = lo;
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        match held_resid(c, fl, tt4, mid) { Some(r) if r * rlo > 0.0 => lo = mid, _ => hi = mid }
        if hi - lo <= 1e-14 { break }
    }
    Some(0.5 * (lo + hi))
}

#[test]
fn r3_an_independent_root_equals_the_shipped_one_on_every_variant() {
    let mut worst: f64 = 0.0;
    let mut per = std::collections::BTreeMap::new();
    for shape in ["steep", "flat-eta"] {
        for (vn, c) in variants(shape) {
            for (_, fl) in h2_flights() {
                for tt4 in h2_tt4().into_iter().chain([1000.0]) {
                    let Ok(p) = pin(&c, &fl, tt4) else { continue };
                    let Ok(o) = on(&c, &fl, tt4) else { continue };
                    let Some(root) = held_root(&c, &fl, tt4, p.base.pi_lpt) else {
                        panic!("{vn} {shape} {tt4}: the held search found no root") };
                    let d = (root - o.base.pi_lpt).abs();
                    if o.base.m9 >= 1.0 - 1e-9 {
                        // Calibration: choked, the root IS the pin.
                        assert!(d <= 1e-12, "{vn} {shape} {tt4}: choked root {root} vs pin {}", o.base.pi_lpt);
                    } else {
                        worst = worst.max(d);
                        *per.entry(vn).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    eprintln!("R3: unchoked cells per variant {per:?}, worst |root - shipped| {worst:.2e}");
    assert!(per.len() == 6 && per.values().all(|&k| k >= 5), "a variant without unchoked cells: {per:?}");
    assert!(worst <= 1e-11, "R3 worst {worst:e}");
}

// --------------------------------------------------------------------------------- R4

#[test]
fn r4_the_steady_branch_is_rung40s_equilibrium() {
    let bs = BladeSettings::defaults();
    let fd = bs.flight();
    let (mut floor, mut worst, mut n) = (0.0f64, 0.0f64, 0);
    // The probe maps, where rung 40's equilibrium converges below choke (anchor G2: on steep / flat-eta it does not).
    for shape in ["flow/press", "press/flow", "tilted"] {
        let (ml, mh) = turbojet::sandbox_controls::control_maps(shape).unwrap();
        let m = match TwoSpoolMapMatcher::new(bs.design(), fd, 1.0, ml, mh) { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!() };
        let tr = TwoSpoolTransientCore::new(bs.design(), fd, 1.0, ml, mh, 1.0);
        for fl in [flight(40.0, 0.0, 0.85), flight(0.0, 0.0, 0.30), h2_flights()[0].1, h2_flights()[2].1] {
            for tt4 in h2_tt4().into_iter().chain([900.0, 1100.0]) {
                let (Ok(o), Ok((i, _, _))) = (on(&m, &fl, tt4), tr.try_equilibrium(&fl, tt4, None)) else { continue };
                let rel = |a: f64, b: f64| ((a - b) / b).abs();
                let d = rel(i.nu_lp, o.n_lp_ratio).max(rel(i.nu_hp, o.n_hp_ratio)).max(rel(i.pi_lpt, o.base.pi_lpt));
                if o.base.m9 >= 1.0 - 1e-9 { floor = floor.max(d) } else { worst = worst.max(d); n += 1 }
            }
        }
    }
    eprintln!("R4: choked floor {floor:.2e}; unchoked worst {worst:.2e} over {n} cells");
    assert!(n >= 10, "only {n} unchoked cells where rung 40's equilibrium converges");
    assert!(worst <= 1e-8, "R4 worst {worst:e} (choked floor {floor:e})");
}

// --------------------------------------------------------------------------------- R5

/// The whole steady point on the perfect gas, by hand: closed-form turbines, a sonic A4 and A45, the subsonic nozzle,
/// an outer bisection on `π_LPT` around a fixed point on the two shaft balances and the burner. Returns
/// `(π_LPT, π_LPC, π_HPC)`.
fn hand_cpg(c: &TwoSpoolCore, fl: &FlightCondition, tt4: f64) -> (f64, f64, f64) {
    let (gc, cpc, gt, cpt, rt, hpr) = (1.4, 1004.0, 1.3, 1239.0, (1.3 - 1.0) / 1.3 * 1239.0, 42.8e6);
    let (ec, et) = ((gc - 1.0) / gc, (gt - 1.0) / gt);
    let tt2 = fl.t0 * (1.0 + 0.5 * (gc - 1.0) * fl.m0 * fl.m0);
    let pt2 = c.pi_d_max * ram_recovery(fl.m0) * fl.p0 * (tt2 / fl.t0).powf(1.0 / ec);
    let tt_out = |tin: f64, pi: f64, eta: f64| tin * (1.0 - eta * (1.0 - pi.powf(et)));
    // HP pin: pi*sqrt(Tt4/Tt45) = A4/A45 (one gas, one MFP*).
    let k = c.a4 / c.a45;
    let (mut lo, mut hi) = (0.05, 0.9999);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if mid * (tt4 / tt_out(tt4, mid, c.eta_hpt)).sqrt() > k { hi = mid } else { lo = mid }
    }
    let pi_hpt = 0.5 * (lo + hi);
    let tt45 = tt_out(tt4, pi_hpt, c.eta_hpt);
    let mfp = (gt / rt).sqrt() * (2.0 / (gt + 1.0)).powf((gt + 1.0) / (2.0 * (gt - 1.0)));
    let point = |pi_lpt: f64| -> (f64, f64, f64, f64) {
        let tt5 = tt_out(tt45, pi_lpt, c.eta_lpt);
        let (mut f, mut pt4, mut out) = (c.f_design, 0.0, (0.0, 0.0, 0.0, 0.0));
        for _ in 0..400 {
            let tt25 = tt2 + c.eta_m * (1.0 + f) * cpt * (tt45 - tt5) / cpc;
            let tt3 = tt25 + c.eta_m * (1.0 + f) * cpt * (tt4 - tt45) / cpc;
            let pi_lpc = (1.0 + c.eta_lpc * (tt25 / tt2 - 1.0)).powf(1.0 / ec);
            let pi_hpc = (1.0 + c.eta_hpc * (tt3 / tt25 - 1.0)).powf(1.0 / ec);
            let pt4n = c.pi_b * pi_hpc * pi_lpc * pt2;
            let fnew = (cpt * tt4 - cpc * tt3) / (c.eta_b * hpr - cpt * tt4);
            let done = (fnew - f).abs() <= 1e-15 && (pt4n - pt4).abs() <= 1e-12 * pt4n;
            f = fnew;
            pt4 = pt4n;
            out = (pi_lpc, pi_hpc, f, pt4);
            if done { break }
        }
        let pt45 = pi_hpt * pt4;
        let supply = c.a45 * pt45 * mfp / tt45.sqrt();
        let pt9 = c.pi_n * pi_lpt * pt45;
        let m9 = (((pt9 / fl.p0).powf(et) - 1.0) / (0.5 * (gt - 1.0))).max(0.0).sqrt();
        let t9 = tt5 / (1.0 + 0.5 * (gt - 1.0) * m9 * m9);
        let demand = c.a8 * fl.p0 / (rt * t9) * m9 * (gt * rt * t9).sqrt();
        ((supply - demand) / supply, out.0, out.1, out.3)
    };
    let (mut lo, mut hi) = (0.3, PI_TOP);
    let rlo = point(lo).0;
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if point(mid).0 * rlo > 0.0 { lo = mid } else { hi = mid }
    }
    let pi = 0.5 * (lo + hi);
    let p = point(pi);
    (pi, p.1, p.2)
}

#[test]
fn r5_a_hand_written_perfect_gas_solve_of_the_whole_point() {
    assert_eq!(cpg().h_c(1.0), 1004.0, "the hand solve assumes h = cp*T");
    let fl = fd39();
    let m38 = TwoSpoolMatcher::new(build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, real()), fl, 1.0);
    let m39 = mm39(cpg(), ComponentMap::flat(), ComponentMap::flat());
    m38.core().below_choke.set(true);
    let mut n = 0;
    let mut worst: f64 = 0.0;
    for tt4 in [500.0, 550.0, 600.0, 650.0, 700.0] {
        let r38 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m38.match_point(&fl, tt4).two()));
        let Ok(r38) = r38 else { continue };
        if r38.m9 >= 1.0 - 1e-9 { continue }
        let r39 = on(m39.core(), &fl, tt4).expect("rung 39 flat solves where rung 38 does");
        let (pi_lpt, pi_lpc, pi_hpc) = hand_cpg(m38.core(), &fl, tt4);
        for (k, x, y) in [("piLPT", r38.pi_lpt, pi_lpt), ("piLPC", r38.pi_lpc, pi_lpc), ("piHPC", r38.pi_hpc, pi_hpc),
                          ("39 piLPT", r39.base.pi_lpt, pi_lpt), ("39 piHPC", r39.base.pi_hpc, pi_hpc)] {
            let d = ((x - y) / y).abs();
            worst = worst.max(d);
            assert!(d <= 1e-9, "{tt4}: {k} shipped {x} hand {y}");
        }
        n += 1;
    }
    eprintln!("R5: {n} unchoked cells, worst {worst:.2e}");
    assert!(n >= 2, "only {n} unchoked cells solved");
}

// --------------------------------------------------------------------------------- R6, R7

#[test]
fn r6_just_below_the_boundary_the_branch_meets_the_pin_and_r7_mass_is_conserved() {
    let (mut worst6, mut worst7, mut n6, mut n7) = (0.0f64, 0.0f64, 0, 0);
    for (vn, c) in variants("steep") {
        for (_, fl) in [h2_flights()[0], h2_flights()[1]] {
            // R7 at every ON unchoked point of the H2 grid.
            for tt4 in h2_tt4() {
                let Ok(o) = on(&c, &fl, tt4) else { continue };
                if o.base.m9 >= 1.0 - 1e-9 { continue }
                let s5 = *o.base.station("5");
                let demand = c.base.a8 * o.base.p9 / (c.gas().r_t_at(s5.far) * o.base.t9) * o.base.v9;
                worst7 = worst7.max(((demand - s5.mdot) / s5.mdot).abs());
                n7 += 1;
            }
            // R6: bisect Tt4 onto the boundary between a refused and a choked throttle.
            let (mut lo, mut hi) = (575.0, 1200.0);
            if !unchoked(&off(&c, &fl, lo)) || off(&c, &fl, hi).is_err() { continue }
            for _ in 0..60 {
                let mid = 0.5 * (lo + hi);
                if unchoked(&off(&c, &fl, mid)) { lo = mid } else { hi = mid }
            }
            let (Ok(o), Ok(p)) = (on(&c, &fl, lo), pin(&c, &fl, lo)) else { panic!("{vn}: no point at the boundary") };
            worst6 = worst6.max((o.base.pi_lpt - p.base.pi_lpt).abs());
            n6 += 1;
        }
    }
    eprintln!("R6: {n6} boundaries, worst {worst6:.2e}; R7: {n7} points, worst {worst7:.2e}");
    assert!(n6 >= 6 && n7 >= 20);
    assert!(worst6 <= 1e-9, "R6 {worst6:e}");
    assert!(worst7 <= 1e-11, "R7 {worst7:e}");
}

// --------------------------------------------------------------------------------- H1, H2 (scored)

/// `Δπ_HPC` for `η_LPC` 0.90 → 0.91, through the full match, with the switch on or the pin kept.
fn d_pi_hpc(m: &mut TwoSpoolMapMatcher, fl: &FlightCondition, tt4: f64, branch: bool) -> Option<f64> {
    let run = |m: &TwoSpoolMapMatcher| if branch { on(m.core(), fl, tt4) } else { pin(m.core(), fl, tt4) };
    m.core_mut().base.eta_lpc = 0.90;
    let a = run(m).ok()?;
    m.core_mut().base.eta_lpc = 0.91;
    let b = run(m);
    m.core_mut().base.eta_lpc = 0.90;
    Some(b.ok()?.base.pi_hpc - a.base.pi_hpc)
}

#[test]
fn h1_below_choke_a_better_lp_compressor_lowers_the_hp_pressure_ratio() {
    let f = ComponentMap::flat();
    let shapes = [("flat", f, f),
                  ("mixed", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                            ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
                  ("steep", shape_maps("steep").unwrap().0, shape_maps("steep").unwrap().1)];
    let (mut n, mut misses) = (0, Vec::new());
    for (gn, gas) in [("CPG", cpg()), ("eq", Gas::reacting_equilibrium())] {
        for (sn, ml, mh) in shapes.iter() {
            let mut m = mm39(gas.clone(), *ml, *mh);
            for (fname, fl) in [("250K", fd39()), ("SL M0.50", flight(0.0, 0.0, 0.50))] {
                for tt4 in [450.0, 500.0, 550.0, 600.0, 650.0, 700.0, 750.0] {
                    let Some((o, _)) = scored(m.core(), &fl, tt4) else { continue };
                    let br = d_pi_hpc(&mut m, &fl, tt4, true).expect("the perturbed branch solves");
                    let pn = d_pi_hpc(&mut m, &fl, tt4, false).expect("the perturbed pin runs");
                    let hit = br < 0.0 && br.abs() >= 1e-6 && br.abs() >= 100.0 * pn.abs();
                    eprintln!("H1 {gn} {sn} {fname} {tt4:.0}: M9 {:.4} branch {br:+.3e} pin {pn:+.3e} {}",
                              o.base.m9, if hit { "HIT" } else { "MISS" });
                    if !hit { misses.push(format!("{gn} {sn} {fname} {tt4:.0}")) }
                    n += 1;
                }
            }
        }
    }
    assert_eq!(n, 38, "H1's frozen cell count");
    // SCORED 37 of 38. The one MISS sits 2e−4 below the choke boundary (M9 0.9998): sign right and 18 000× the pin, but
    // 2.7e−8 against the 1e−6 floor. The arrow VANISHES at the boundary — a just-sonic nozzle's flow function is flat
    // in its pressure ratio — which the anchor's derivation did not foresee (spec § 3). Pinned as measured.
    assert_eq!(misses, vec!["eq steep 250K 650".to_string()], "H1's frozen outcome moved");
}

#[test]
fn h2_the_lp_shaft_runs_slower_than_the_pin_said() {
    let mut n = 0;
    for shape in ["steep", "flat-eta"] {
        for (vn, c) in variants(shape) {
            for (fname, fl) in h2_flights() {
                for tt4 in h2_tt4() {
                    let Some((o, p)) = scored(&c, &fl, tt4) else { continue };
                    assert!(o.n_lp_ratio < p.n_lp_ratio,
                            "H2 MISS {shape} {vn} {fname} {tt4}: {} vs pin {}", o.n_lp_ratio, p.n_lp_ratio);
                    n += 1;
                }
            }
        }
    }
    assert_eq!(n, 174, "H2's frozen cell count");
}
