//! RUNG 104 — THE LP TURBINE'S THROAT WAS NEVER CHOKED: a flow law replaces rung 38's pin on `A45`.
//!
//! Anchor: `docs/plans/rung104-anchor-turbine-law.md` (frozen as `0ddab63` before any rung-104 code). Spec:
//! `docs/rung104-spec.md`. Gates and scored claims at the anchor's own cells and bars:
//!
//!   R1b the pin reproduces the OLD model's record (`rust_owned/rung104_pin.txt`, captured from `33250a2`) bit for bit;
//!   R2  perfect gas, nozzle choked: every law IS the pin (≤ 1e−12);
//!   R3  the design point: every law reproduces the design run (≤ 1e−12), both gases;
//!   R4  a hand-written perfect-gas solve below choke under the ellipse (≤ 1e−10);
//!   R5  one root: the joint residual has one sign change on [0.2, 0.9] at every scored cell, the root inside;
//!   R6  mass, read independently of the search: the ellipse written out here on the result's stations; the nozzle;
//!   R7  sixteen equal rows reach the ellipse (≤ 1 %);
//!   R8  the vane is never choked (G1's bound ≤ 1);
//!   R10 the pages: the setting reaches each view's engine;
//!   H1  THE HEADLINE — below choke the back-pressure slows BOTH shafts (the share ≥ 0.25, ≥ 10 × rung 97's);
//!   H2  the laws are ordered (the premise of the sort's "re-check under two rows").
//! (R1a — the whole gate on the pin — is `test-all.ps1 -Features pin-default`, not a test; R9 is a scratch red check.)

use std::collections::HashMap;

use turbojet::afterburner::AbMode;
use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::burner_hold::Hold;
use turbojet::components::{choked_mfp, ram_recovery, try_choked_mfp, Nozzle};
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::gas::{FlowState, Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{controls, ControlsSettings, CustomerMode, GeneratorMode};
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolCore, TwoSpoolLosses, TwoSpoolMapCore, TwoSpoolMapMatcher,
                          TwoSpoolMapResult, TwoSpoolMatcher, TwoSpoolResult};
use turbojet::unchoked_lp::{try_lp_turbine, PI_TOP};

// --------------------------------------------------------------------------------- engines and cells

fn flight(dt: f64, h: f64, m0: f64) -> FlightCondition {
    let (t, p) = standard(h);
    FlightCondition::new(t + dt, p, m0)
}
fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
fn cpg() -> Gas {
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9, gamma_t: 1.3, cp_t: 1239.0,
                       r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() })
}
fn fd39() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn full(m: TwoSpoolMapMatcher) -> TwoSpoolMapCore {
    match m { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!("both shafts") }
}
/// Rung 39's engine (`π` 3/6, `Tt4` 1500 K at 250 K / 50 kPa / Mach 0.85) on `gas`, with `law` from the start.
fn r39(gas: Gas, ml: ComponentMap, mh: ComponentMap, law: TurbineLaw) -> TwoSpoolMapCore {
    full(TwoSpoolMapMatcher::new(build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, 50_000.0, real()).with_turbine_law(law),
                                 fd39(), 1.0, ml, mh))
}

/// The Controls engine's six steady variants (rung 103's H2 settings) on one map shape, every engine built on `law`.
fn variants(shape: &str, law: TurbineLaw) -> Vec<(&'static str, TwoSpoolMapCore)> {
    let bs = BladeSettings { turbine: law, ..BladeSettings::defaults() };
    let fd = bs.flight();
    let (ml, mh) = shape_maps(shape).unwrap();
    let d = || bs.design();
    vec![
        ("plain", full(TwoSpoolMapMatcher::new(d(), fd, 1.0, ml, mh))),
        ("bleed", TwoSpoolBleedMatcher::new(d(), fd, 1.0, ml, mh, 0.08).core),
        ("customer", CustomerBleedMatcher::new(d().with_customer_bleed(CustomerBleed::share(0.03)), fd, 1.0, ml, mh, 0.0).core),
        ("cooling", TurbineCoolingMatcher::new(d().with_turbine_cooling(TurbineCooling::new(0.02, 0.04)), fd, 1.0, ml, mh, 0.0).core),
        ("gen-HP", ShaftOfftakeMatcher::new(d().with_shaft_offtake(ShaftOfftake::hp(Demand::Power(0.02))), fd, 1.0, ml, mh, 0.0).core),
        ("gen-LP", ShaftOfftakeMatcher::new(d().with_shaft_offtake(ShaftOfftake::lp(Demand::Power(0.02))), fd, 1.0, ml, mh, 0.0).core),
    ]
}

/// The plain Controls engine alone on one map shape.
fn plain(shape: &str, law: TurbineLaw) -> TwoSpoolMapCore {
    let bs = BladeSettings { turbine: law, ..BladeSettings::defaults() };
    let (ml, mh) = shape_maps(shape).unwrap();
    full(TwoSpoolMapMatcher::new(bs.design(), bs.flight(), 1.0, ml, mh))
}

/// The three solves § 3 compares: the OLD model (the pin pushed past choke), rung 97/103's branch (the pin, the LP
/// turbine free), and the branch under `law`.
fn old(c: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64) -> Option<TwoSpoolMapResult> {
    c.base.turbine_law.set(TurbineLaw::Pin);
    c.base.pin_past_choke.set(true);
    let r = c.try_match_point(fl, tt4).ok();
    c.base.pin_past_choke.set(false);
    r
}
fn branch(c: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64, law: TurbineLaw) -> Option<TwoSpoolMapResult> {
    c.base.turbine_law.set(law);
    c.base.below_choke.set(true);
    let r = c.try_match_point(fl, tt4).ok();
    c.base.below_choke.set(false);
    r
}

const SCORED_SHAPES: [&str; 3] = ["steep", "flat-eta", "flow/press"];
fn scored_flights() -> [(&'static str, FlightCondition); 3] {
    [("SL+20 M0.60", flight(20.0, 0.0, 0.60)), ("2km M0.50", flight(0.0, 2000.0, 0.50)),
     ("SL-10 M0.40", flight(-10.0, 0.0, 0.40))]
}
fn scored_tt4() -> Vec<f64> { (0..6).map(|i| 575.0 + 50.0 * i as f64).collect() }

struct Cell {
    name: String,
    fl: FlightCondition,
    tt4: f64,
    old: TwoSpoolMapResult,
    r97: TwoSpoolMapResult,
    ell: TwoSpoolMapResult,
}
/// § 3's scored cells of one variant on one map: the old model, rung 97's branch and the ellipse all solve, and the
/// ellipse's nozzle is unchoked (`M9 ≤ 0.99`).
fn scored_cells(shape: &str, vn: &str, c: &TwoSpoolMapCore) -> Vec<Cell> {
    let mut out = Vec::new();
    for (fname, fl) in scored_flights() {
        for tt4 in scored_tt4() {
            let (Some(o), Some(q), Some(e)) = (old(c, &fl, tt4), branch(c, &fl, tt4, TurbineLaw::Pin),
                                               branch(c, &fl, tt4, TurbineLaw::Ellipse)) else { continue };
            if e.base.m9 <= 0.99 {
                out.push(Cell { name: format!("{shape} {vn} {fname} {tt4}"), fl, tt4, old: o, r97: q, ell: e });
            }
        }
    }
    out
}
fn rel(a: f64, b: f64) -> f64 { a / b - 1.0 }

// --------------------------------------------------------------------------------- R1b

type Keys = Vec<(String, Result<f64, String>)>;

fn map_point(out: &mut Keys, k: &str, c: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64) {
    match c.try_match_point(fl, tt4) {
        Ok(r) => for (n, v) in [("piHPT", r.base.pi_hpt), ("piLPT", r.base.pi_lpt), ("piLPC", r.base.pi_lpc),
                                ("piHPC", r.base.pi_hpc), ("NL", r.n_lp_ratio), ("NH", r.n_hp_ratio), ("F", r.base.thrust)] {
            out.push((format!("{k}.{n}"), Ok(v)));
        },
        Err(e) => out.push((k.to_string(), Err(e.0.chars().take(60).collect()))),
    }
}

/// The keys `probe/src/pin_golden.rs` wrote from the OLD model, computed now with every engine on the pin.
fn pin_keys() -> Keys {
    let mut out: Keys = Vec::new();
    let pin = TurbineLaw::Pin;
    for shape in ["tilted", "steep"] {
        for (vn, c) in variants(shape, pin) {
            c.base.below_choke.set(true);
            for (fname, fl) in [("SL+40M0.85", flight(40.0, 0.0, 0.85)), ("SLM0.30", flight(0.0, 0.0, 0.30)),
                                ("design", BladeSettings::defaults().flight())] {
                for i in 0..5 {
                    let tt4 = 600.0 + 100.0 * i as f64;
                    map_point(&mut out, &format!("steady.{shape}.{vn}.{fname}.{tt4}"), &c, &fl, tt4);
                }
            }
        }
    }
    let fd = fd39();
    for (mname, (ml, mh)) in [("flat", (ComponentMap::flat(), ComponentMap::flat())), ("steep", shape_maps("steep").unwrap())] {
        let m = r39(Gas::reacting_equilibrium(), ml, mh, pin);
        for i in 0..5 {
            let tt4 = 700.0 + 200.0 * i as f64;
            map_point(&mut out, &format!("r39eq.{mname}.{tt4}"), &m, &fd, tt4);
        }
    }
    let m38 = TwoSpoolMatcher::new(build_two_spool_turbojet(Gas::reacting_equilibrium(), 3.0, 6.0, 1500.0, 50_000.0, real())
                                       .with_turbine_law(pin), fd, 1.0);
    for i in 0..5 {
        let tt4 = 700.0 + 200.0 * i as f64;
        let r = m38.match_point(&fd, tt4).two();
        for (n, v) in [("piHPT", r.pi_hpt), ("piLPT", r.pi_lpt), ("piLPC", r.pi_lpc), ("piHPC", r.pi_hpc), ("F", r.thrust)] {
            out.push((format!("r38eq.{tt4}.{n}"), Ok(v)));
        }
    }
    let d = ControlsSettings { turbine: pin, ..ControlsSettings::defaults() };
    let lit = |s: ControlsSettings| ControlsSettings { ab: Some(AbMode::Lit), ..s };
    let runs: Vec<(&str, ControlsSettings)> = vec![
        ("default", d),
        ("unchoked-up", ControlsSettings { t0: 288.15 + 40.0, p0: 101_325.0, m0: 0.85, from: 700.0, to: 1400.0, ..d }),
        ("unchoked-down", ControlsSettings { t0: 288.15 + 40.0, p0: 101_325.0, m0: 0.85, from: 1400.0, to: 700.0, ..d }),
        ("governed-thrust", ControlsSettings { govern: true, hold: Hold::Thrust, ..d }),
        ("cooling", ControlsSettings { cool_vane: 0.05, cool_blade: 0.03, ..d }),
        ("gen-HP", ControlsSettings { generator: GeneratorMode::Hp, ..d }),
        ("customer", ControlsSettings { customer: CustomerMode::Share, ..d }),
        ("ab-lit", lit(d)),
        ("ab-lit-wide", lit(ControlsSettings { a8_ratio: 1.15, tt7: 1900.0, ..d })),
    ];
    for (rn, s) in runs {
        match controls(&s) {
            Ok((_, o)) => {
                out.push((format!("ctl.{rn}.points"), Ok(o.points.len() as f64)));
                for (i, p) in o.points.iter().enumerate() {
                    for (n, v) in [("nuL", p.nu_lp), ("nuH", p.nu_hp), ("Tt4", p.tt4), ("spF", p.sp_thrust), ("mdot", p.mdot_air)] {
                        out.push((format!("ctl.{rn}.{i}.{n}"), Ok(v)));
                    }
                }
            }
            Err(e) => out.push((format!("ctl.{rn}"), Err(format!("{e:?}").chars().take(60).collect()))),
        }
    }
    out
}

#[test]
fn r1b_the_pin_reproduces_the_old_models_record_bit_for_bit() {
    let text = std::fs::read_to_string("oracle/rust_owned/rung104_pin.txt").expect("the old model's record");
    let mut want: HashMap<String, String> = HashMap::new();
    for line in text.lines() {
        let (k, v) = line.split_once(' ').expect("key value");
        want.insert(k.to_string(), v.to_string());
    }
    let got = pin_keys();
    assert_eq!(got.len(), want.len(), "the record and the reproduction must hold the same keys");
    let mut bad = Vec::new();
    for (k, v) in &got {
        let w = want.get(k).unwrap_or_else(|| panic!("{k} is not in the record"));
        let same = match v {
            Ok(x) => w.parse::<f64>().map(|y| y.to_bits() == x.to_bits()).unwrap_or(false),
            Err(e) => w.strip_prefix("ERR ") == Some(e.as_str()),
        };
        if !same {
            bad.push(format!("{k}: record {w}, now {v:?}"));
        }
    }
    eprintln!("R1b: {} keys, {} differ", got.len(), bad.len());
    assert!(bad.is_empty(), "the pin is not the old model:\n{}", bad.iter().take(20).cloned().collect::<Vec<_>>().join("\n"));
}

// --------------------------------------------------------------------------------- R2, R3

#[test]
fn r2_perfect_gas_choked_every_law_is_the_pin() {
    let (mut n, mut worst) = (0, 0.0f64);
    for shape in SCORED_SHAPES {
        for law in [TurbineLaw::Ellipse, TurbineLaw::TwoRows, TurbineLaw::OneRow] {
            let pin = &plain(shape, TurbineLaw::Pin);
            let c = &plain(shape, law);
            for (_, fl) in scored_flights() {
                for i in 0..8 {
                    let tt4 = 875.0 + 75.0 * i as f64;
                    // The law's OWN search (the shortcut off): within 1e−12 of the pin.
                    c.base.law_shortcut.set(false);
                    let own = c.try_match_point(&fl, tt4);
                    c.base.law_shortcut.set(true);
                    let (Ok(a), Ok(b)) = (pin.try_match_point(&fl, tt4), own) else { continue };
                    if a.base.m9 < 1.0 - 1e-9 { continue }
                    for (k, x, y) in [("piHPT", b.base.pi_hpt, a.base.pi_hpt), ("piLPT", b.base.pi_lpt, a.base.pi_lpt),
                                      ("NL", b.n_lp_ratio, a.n_lp_ratio), ("NH", b.n_hp_ratio, a.n_hp_ratio),
                                      ("F", b.base.thrust, a.base.thrust)] {
                        let d = rel(x, y).abs();
                        worst = worst.max(d);
                        assert!(d <= 1e-12, "{shape} {law:?} {tt4}: {k} {x} vs the pin {y}");
                    }
                    // The shipped point (the shortcut on): the pin's, bit for bit.
                    let s = c.try_match_point(&fl, tt4).unwrap();
                    assert!(s.base.pi_hpt.to_bits() == a.base.pi_hpt.to_bits() && s.base.thrust.to_bits() == a.base.thrust.to_bits()
                            && s.n_lp_ratio.to_bits() == a.n_lp_ratio.to_bits(), "{shape} {law:?} {tt4}: the shortcut is not the pin");
                    n += 1;
                }
            }
        }
    }
    eprintln!("R2: {n} choked cells, worst {worst:.2e}");
    assert!(n >= 100, "only {n} choked cells");
}

#[test]
fn r3_every_law_reproduces_the_design_run() {
    // The anchor froze 1e−12 for both gases. On the equilibrium gas the OLD model itself lands 4.8e−12 from its design
    // run (the matcher's own floor there — measured only after the freeze): a MISS of the frozen bar, recorded in the
    // spec. Shipped: the perfect gas at the frozen bar; the equilibrium gas no further from the design than the pin.
    let mut misses = Vec::new();
    for (gname, gas) in [("perfect", cpg()), ("equilibrium", Gas::reacting_equilibrium())] {
        let err = |law: TurbineLaw| -> [f64; 2] {
            let c = r39(gas.clone(), ComponentMap::flat(), ComponentMap::flat(), law);
            let r = c.try_match_point(&fd39(), 1500.0).expect("the design point solves");
            let d = &c.base.reference;
            let (s4, s45, s5) = (d.station("4"), d.station("45"), d.station("5"));
            [rel(r.base.pi_hpt, s45.pt / s4.pt).abs(), rel(r.base.pi_lpt, s5.pt / s45.pt).abs()]
        };
        let pin = err(TurbineLaw::Pin);
        for law in [TurbineLaw::Ellipse, TurbineLaw::TwoRows, TurbineLaw::OneRow] {
            let e = err(law);
            eprintln!("R3 {gname} {law:?}: piHPT {:.2e} x {:.2e} (the pin: {:.2e} {:.2e})", e[0], e[1], pin[0], pin[1]);
            for i in 0..2 {
                let bar = if gname == "perfect" { 1e-12 } else { pin[i].max(1e-12) };
                if e[i] > bar { misses.push(format!("{gname} {law:?} [{i}]: {:.2e} over {bar:.2e}", e[i])); }
            }
        }
    }
    assert!(misses.is_empty(), "{misses:#?}");
}

// --------------------------------------------------------------------------------- R4

/// The whole steady point on the perfect gas, by hand, under the ELLIPSE: closed-form turbines, the ellipse formula, a
/// sonic `A4`, the subsonic nozzle; an outer bisection on `x = π_LPT` with the HP ratio from `A4 = A45` at each `x`.
/// Returns `(π_HPT, π_LPT, π_LPC, π_HPC)` at the first sign change of the nozzle's mass residual.
fn hand(c: &TwoSpoolCore, fl: &FlightCondition, tt4: f64) -> Option<(f64, f64, f64, f64)> {
    let (gc, cpc, gt, cpt, rt, hpr) = (1.4, 1004.0, 1.3, 1239.0, (1.3 - 1.0) / 1.3 * 1239.0, 42.8e6);
    let (ec, et) = ((gc - 1.0) / gc, (gt - 1.0) / gt);
    let tt2 = fl.t0 * (1.0 + 0.5 * (gc - 1.0) * fl.m0 * fl.m0);
    let pt2 = c.pi_d_max * ram_recovery(fl.m0) * fl.p0 * (tt2 / fl.t0).powf(1.0 / ec);
    let tt_out = |tin: f64, pi: f64, eta: f64| tin * (1.0 - eta * (1.0 - pi.powf(et)));
    let d = &c.reference;
    let xd = d.station("5").pt / d.station("45").pt;
    let e = |x: f64| (1.0 - x * x).max(0.0).sqrt() / (1.0 - xd * xd).sqrt();
    let k = c.a4 / c.a45_pin();
    let mfp = (gt / rt).sqrt() * (2.0 / (gt + 1.0)).powf((gt + 1.0) / (2.0 * (gt - 1.0)));
    let bis = |mut lo: f64, mut hi: f64, f: &dyn Fn(f64) -> f64| {
        let flo = f(lo);
        for _ in 0..200 {
            let m = 0.5 * (lo + hi);
            if f(m) * flo > 0.0 { lo = m } else { hi = m }
        }
        0.5 * (lo + hi)
    };
    let point = |x: f64| -> Option<(f64, f64, f64, f64)> {
        let tgt = k / e(x);
        if !(tgt < 1.0) { return None }
        let ph = bis(1e-4, 1.0, &|p| p * (tt4 / tt_out(tt4, p, c.eta_hpt)).sqrt() - tgt);
        let tt45 = tt_out(tt4, ph, c.eta_hpt);
        let tt5 = tt_out(tt45, x, c.eta_lpt);
        let (mut f, mut pt4, mut out) = (c.f_design, 0.0, (0.0, 0.0));
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
            out = (pi_lpc, pi_hpc);
            if done { break }
        }
        let pt45 = ph * pt4;
        let supply = c.a45_pin() * mfp * e(x) * pt45 / tt45.sqrt();
        let pt9 = c.pi_n * x * pt45;
        let m9 = (((pt9 / fl.p0).powf(et) - 1.0) / (0.5 * (gt - 1.0))).max(0.0).sqrt();
        let t9 = tt5 / (1.0 + 0.5 * (gt - 1.0) * m9 * m9);
        let demand = c.a8 * fl.p0 / (rt * t9) * m9 * (gt * rt * t9).sqrt();
        Some(((supply - demand) / supply, ph, out.0, out.1))
    };
    let xmax = (1.0 - k * k * (1.0 - xd * xd)).sqrt().min(PI_TOP);
    let n = 400;
    let grid: Vec<(f64, f64)> = (0..n)
        .filter_map(|i| {
            let x = 0.3 + (xmax - 0.3) * (i as f64 + 0.5) / n as f64;
            point(x).map(|p| (x, p.0))
        })
        .collect();
    let w = grid.windows(2).find(|w| w[0].1 * w[1].1 < 0.0)?;
    let rlo = w[0].1;
    let x = bis(w[0].0, w[1].0, &|m| point(m).map_or(f64::NAN, |p| p.0) * rlo.signum());
    let p = point(x)?;
    Some((p.1, x, p.2, p.3))
}

#[test]
fn r4_a_hand_written_perfect_gas_solve_below_choke_under_the_ellipse() {
    assert_eq!(cpg().h_c(1.0), 1004.0, "the hand solve assumes h = cp*T");
    let fd = fd39();
    let m38 = TwoSpoolMatcher::new(build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, real())
                                       .with_turbine_law(TurbineLaw::Ellipse), fd, 1.0);
    let c = m38.core();
    c.below_choke.set(true);
    let m39 = r39(cpg(), ComponentMap::flat(), ComponentMap::flat(), TurbineLaw::Ellipse);
    m39.base.below_choke.set(true);
    let (mut n, mut worst) = (0, 0.0f64);
    for fl in [flight(0.0, 0.0, 0.30), flight(40.0, 0.0, 0.85), fd] {
        for i in 0..8 {
            let tt4 = 450.0 + 50.0 * i as f64;
            let Ok(r) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.match_point(&fl, tt4))) else { continue };
            if r.m9 >= 1.0 - 1e-9 { continue }
            let (ph, pl, plc, phc) = hand(c, &fl, tt4).expect("the hand solve finds the point the matcher found");
            let q = m39.try_match_point(&fl, tt4).expect("rung 39 flat solves where rung 38 does");
            for (k, x, y) in [("38 piHPT", r.pi_hpt, ph), ("38 piLPT", r.pi_lpt, pl), ("38 piLPC", r.pi_lpc, plc),
                              ("38 piHPC", r.pi_hpc, phc), ("39 piHPT", q.base.pi_hpt, ph), ("39 piLPT", q.base.pi_lpt, pl),
                              ("39 piHPC", q.base.pi_hpc, phc)] {
                let d = rel(x, y).abs();
                worst = worst.max(d);
                assert!(d <= 1e-10, "{tt4}: {k} shipped {x} hand {y}");
            }
            n += 1;
        }
    }
    eprintln!("R4: {n} unchoked cells, worst {worst:.2e}");
    assert!(n >= 10, "only {n} unchoked cells");
}

// --------------------------------------------------------------------------------- R5, R6

/// The joint residual over the HP ratio at a result's own outer state (`pt4`, `f`, both efficiencies held): the HP
/// nozzle guide vane's flow against the law's `W45` at the LP turbine's own `x` (zero flow where the LP turbine cannot
/// pass its gas). Uncooled variants only (a cooled vane's residual mixes the coolants in).
fn joint_resid(b: &TwoSpoolCore, r: &TwoSpoolMapResult, fl: &FlightCondition, ph: f64) -> f64 {
    let gas = b.gas();
    let s4 = r.base.station("4");
    let (f, pt4, tt4) = (s4.far, s4.pt, s4.tt);
    let a4m = b.a4 * try_choked_mfp(gas, tt4, f).unwrap();
    let (_, tt45) = b.try_tau_of(gas, tt4, f, ph, r.eta_hpt).unwrap();
    let hp = a4m * (tt45 / tt4).sqrt() / ph;
    match try_lp_turbine(b, gas, tt45, f, r.eta_lpt, ph * pt4, fl.p0) {
        Ok((x, _, _)) => hp - b.w45(gas, x, tt45, f, r.eta_lpt).unwrap(),
        Err(_) => hp,
    }
}

#[test]
fn r5_one_root_inside_the_search_at_every_scored_cell() {
    let (mut n, mut skipped) = (0, 0);
    for shape in SCORED_SHAPES {
        for (vn, c) in variants(shape, TurbineLaw::Pin) {
            if vn == "cooling" {
                skipped += scored_cells(shape, vn, &c).len();
                continue;
            }
            for (fname, fl) in scored_flights() {
                for tt4 in scored_tt4() {
                    let Some(e) = branch(&c, &fl, tt4, TurbineLaw::Ellipse) else { continue };
                    if e.base.m9 > 0.99 { continue }
                    c.base.turbine_law.set(TurbineLaw::Ellipse);
                    let g: Vec<f64> = (0..71).map(|i| joint_resid(&c.base, &e, &fl, 0.2 + 0.01 * i as f64)).collect();
                    let changes = g.windows(2).filter(|w| w[0] * w[1] < 0.0).count();
                    let ph = e.base.pi_hpt;
                    assert_eq!(changes, 1, "{shape} {vn} {fname} {tt4}: {changes} sign changes");
                    assert!(ph >= 0.22 && ph <= 0.88, "{shape} {vn} {fname} {tt4}: root {ph} near an end");
                    n += 1;
                }
            }
        }
    }
    eprintln!("R5: {n} cells, one root each; the cooled variant's {skipped} cells are not scanned (its residual mixes \
               the coolants in)");
    assert!(n >= 150);
}

#[test]
fn r6_mass_read_independently_of_the_search() {
    let (mut worst45, mut worst9, mut n) = (0.0f64, 0.0f64, 0);
    for shape in SCORED_SHAPES {
        let c = &plain(shape, TurbineLaw::Ellipse);
        let b = &c.base;
        let gas = b.gas();
        let d = &b.reference;
        let (d45, d5) = (d.station("45"), d.station("5"));
        // The ellipse written out here: `W45_d` from the design run's station 45 and A45 through the gas's own MFP*.
        let w45d = b.a45_pin() * choked_mfp(gas, d45.tt, d45.far);
        let xd = d5.pt / d45.pt;
        for (_, fl) in scored_flights() {
            for i in 0..12 {
                let tt4 = 575.0 + 75.0 * i as f64;
                let Some(r) = branch(c, &fl, tt4, TurbineLaw::Ellipse) else { continue };
                let (s45, s5) = (r.base.station("45"), r.base.station("5"));
                let x = s5.pt / s45.pt;
                let w45 = w45d * (1.0 - x * x).sqrt() / (1.0 - xd * xd).sqrt();
                let e45 = rel(w45 * s45.pt / s45.tt.sqrt(), s45.mdot).abs();
                worst45 = worst45.max(e45);
                assert!(e45 <= 1e-11, "{shape} {tt4}: A45 {e45:e}");
                if r.base.m9 < 1.0 - 1e-9 {
                    let rho9 = r.base.p9 / (gas.r_t_at(s5.far) * r.base.t9);
                    let e9 = rel(b.a8 * rho9 * r.base.v9, s5.mdot).abs();
                    worst9 = worst9.max(e9);
                    assert!(e9 <= 1e-11, "{shape} {tt4}: nozzle {e9:e}");
                }
                n += 1;
            }
        }
    }
    eprintln!("R6: {n} points, A45 worst {worst45:.2e}, nozzle worst {worst9:.2e}");
}

// --------------------------------------------------------------------------------- R7, R8

/// The mass flux through a convergent row (the gas's own nozzle routine), from total `(p, t)` to `pb`.
fn row_flux(gas: &Gas, f: f64, p: f64, t: f64, pb: f64) -> f64 {
    let e = Nozzle::convergent(pb, 1.0).try_apply(&FlowState { tt: t, pt: p, mdot: 1.0, far: f }, gas).unwrap();
    e.p9 / (gas.r_t_at(f) * e.t9) * e.v9
}
fn bisect(mut lo: f64, mut hi: f64, f: impl Fn(f64) -> f64) -> f64 {
    let flo = f(lo);
    for _ in 0..200 {
        let m = 0.5 * (lo + hi);
        if f(m) * flo > 0.0 { lo = m } else { hi = m }
        if hi - lo < 1e-14 { break }
    }
    0.5 * (lo + hi)
}

#[test]
fn r7_sixteen_equal_rows_reach_the_ellipse() {
    let c = r39(Gas::reacting_equilibrium(), ComponentMap::flat(), ComponentMap::flat(), TurbineLaw::Ellipse);
    let b = &c.base;
    let gas = b.gas();
    let d = &b.reference;
    let (s4, s45, s5) = (d.station("4"), d.station("45"), d.station("5"));
    gas.freeze_equilibrium(s4.far, s4.tt, s4.pt);
    let (f, td, xd) = (s4.far, s45.tt, s5.pt / s45.pt);
    let n = 16;
    let r = xd.powf(1.0 / n as f64);
    let (mut p, mut t, mut areas) = (1.0, td, Vec::new());
    for _ in 0..n {
        areas.push(1.0 / row_flux(gas, f, p, t, p * r));
        t = b.try_tau_of(gas, t, f, r, b.eta_lpt).unwrap().1;
        p *= r;
    }
    let end = |fl: f64| -> f64 {
        let (mut p, mut t) = (1.0, td);
        for a in &areas {
            if fl >= a * row_flux(gas, f, p, t, 0.3 * p) { return -1.0 }
            let pb = bisect(0.3 * p, p * (1.0 - 1e-7), |pb| a * row_flux(gas, f, p, t, pb) - fl);
            t = b.try_tau_of(gas, t, f, pb / p, b.eta_lpt).unwrap().1;
            p = pb;
        }
        p
    };
    let mut worst = 0.0f64;
    for i in 0..10 {
        let x = 0.50 + 0.05 * i as f64;
        let rows = bisect(0.05, 50.0, |fl| end(fl) - x);
        let ell = (1.0 - x * x).sqrt() / (1.0 - xd * xd).sqrt();
        worst = worst.max(rel(rows, ell).abs());
    }
    eprintln!("R7: sixteen rows against the ellipse, worst {worst:.3e}");
    assert!(worst <= 0.01, "sixteen rows miss the ellipse by {worst}");
}

fn crit(g: f64) -> f64 { ((g + 1.0) / 2.0).powf(g / (g - 1.0)) }
fn stat(g: f64, m: f64) -> f64 { (1.0 + 0.5 * (g - 1.0) * m * m).powf(g / (g - 1.0)) }

#[test]
fn r8_the_vane_is_never_choked() {
    let mut worst = 0.0f64;
    let mut n = 0;
    for gas in [cpg(), Gas::reacting_equilibrium()] {
        let c = r39(gas, ComponentMap::flat(), ComponentMap::flat(), TurbineLaw::Ellipse);
        for fl in [fd39(), FlightCondition::new(288.15, 101_325.0, 0.0), FlightCondition::new(318.15, 101_325.0, 0.3),
                   FlightCondition::new(216.65, 22_632.0, 2.0)] {
            for i in 0..9 {
                let tt4 = 700.0 + 100.0 * i as f64;
                let Ok(r) = c.try_match_point(&fl, tt4) else { continue };
                let g = c.base.gas();
                let (s4, s45, s5) = (r.base.station("4"), r.base.station("45"), r.base.station("5"));
                if g.is_equilibrium() { g.freeze_equilibrium(s4.far, s4.tt, s4.pt); }
                // The vane's largest drop: pt45 down to the LP turbine's EXIT static at Mach 0.6, against the critical
                // ratio at the hottest (lowest-γ) end — a bound leaning toward choking.
                let bound = s45.pt / s5.pt * stat(g.gamma_t_at(s5.tt, s5.far), 0.6) / crit(g.gamma_t_at(s45.tt, s45.far));
                worst = worst.max(bound);
                n += 1;
            }
        }
    }
    eprintln!("R8: {n} points, worst bound/critical {worst:.4}");
    assert!(n >= 40 && worst < 1.0, "the vane's bound reaches {worst}");
}

// --------------------------------------------------------------------------------- R10

#[test]
fn r10_the_setting_reaches_each_views_engine() {
    // Controls: an unchoked move differs from the pin's; Blades and Cooling: the engine their settings build carries
    // the law.
    let s = ControlsSettings { t0: 288.15 + 40.0, p0: 101_325.0, m0: 0.85, from: 700.0, to: 1400.0,
                               turbine: TurbineLaw::Ellipse, ..ControlsSettings::defaults() };
    let (_, a) = controls(&s).unwrap();
    let (_, b) = controls(&ControlsSettings { turbine: TurbineLaw::Pin, ..s }).unwrap();
    assert!(a.points.iter().zip(&b.points).any(|(p, q)| p.nu_hp != q.nu_hp), "the Controls law did not reach the engine");
    for law in TurbineLaw::ALL {
        assert_eq!(BladeSettings { turbine: law, ..BladeSettings::defaults() }.design().turbine_law, law);
        let cs = turbojet::sandbox_cooling::CoolingSettings { turbine: law, ..turbojet::sandbox_cooling::CoolingSettings::defaults() };
        assert_eq!(cs.fixed().design.turbine_law, law, "Cooling");
        let back = ControlsSettings::from_json(&ControlsSettings { turbine: law, ..ControlsSettings::defaults() }.to_json()).unwrap();
        assert_eq!(back.turbine, law);
    }
    let shipped = if cfg!(feature = "pin-default") { TurbineLaw::Pin } else { TurbineLaw::Ellipse };
    assert_eq!(TurbineLaw::default(), shipped, "the default law (the pin only when built with `pin-default`)");
}

// --------------------------------------------------------------------------------- H1, H2

#[test]
fn h1_below_choke_the_back_pressure_slows_both_shafts() {
    let (mut n_a, mut n_b, mut excluded, mut r97_positive) = (0, 0, 0, 0);
    let (mut s_min, mut ratio_min) = (f64::INFINITY, f64::INFINITY);
    let mut misses = Vec::new();
    for shape in SCORED_SHAPES {
        for (vn, c) in variants(shape, TurbineLaw::Pin) {
            for cell in scored_cells(shape, vn, &c) {
                let (o, q, e) = (&cell.old, &cell.r97, &cell.ell);
                // H1a: both shafts slower than the old model; against rung 97's branch the HP slower, the LP faster.
                let a = e.n_hp_ratio < o.n_hp_ratio && e.n_lp_ratio < o.n_lp_ratio
                    && e.n_hp_ratio < q.n_hp_ratio && e.n_lp_ratio > q.n_lp_ratio;
                if !a { misses.push(format!("H1a {}", cell.name)); }
                n_a += 1;
                if vn != "plain" { continue }
                let dl = rel(e.n_lp_ratio, o.n_lp_ratio);
                if dl.abs() < 1e-6 {
                    excluded += 1;
                    continue;
                }
                let s = rel(e.n_hp_ratio, o.n_hp_ratio) / dl;
                let s97 = rel(q.n_hp_ratio, o.n_hp_ratio) / rel(q.n_lp_ratio, o.n_lp_ratio);
                if s97 < 0.0 { r97_positive += 1; }
                s_min = s_min.min(s);
                ratio_min = ratio_min.min(s / s97.abs());
                if !(s >= 0.25 && s >= 10.0 * s97.abs()) {
                    misses.push(format!("H1b {}: share {s:.4}, rung 97's {s97:.5}", cell.name));
                }
                n_b += 1;
            }
        }
    }
    eprintln!("H1: H1a {n_a} cells, H1b {n_b} cells (excluded at the boundary {excluded}); smallest share {s_min:.3}, \
               smallest share / |rung 97's| {ratio_min:.1}; rung 97's HP change of the other sign at {r97_positive}");
    for m in &misses { eprintln!("  MISS {m}"); }
    assert_eq!(n_a, 251, "the anchor's pre-flight counted 251 scored cells");
    assert!(misses.is_empty(), "{} misses", misses.len());
}

#[test]
fn h2_the_laws_are_ordered() {
    let mut misses = Vec::new();
    let mut n = 0;
    for shape in SCORED_SHAPES {
        let c = &plain(shape, TurbineLaw::Pin);
        for cell in scored_cells(shape, "plain", c) {
            let (fl, tt4) = (cell.fl, cell.tt4);
            let (Some(one), Some(two)) = (branch(c, &fl, tt4, TurbineLaw::OneRow), branch(c, &fl, tt4, TurbineLaw::TwoRows))
                else { misses.push(format!("{} a row law refused", cell.name)); continue };
            let o = &cell.old;
            let hp = |r: &TwoSpoolMapResult| -rel(r.n_hp_ratio, o.n_hp_ratio);
            let lp = |r: &TwoSpoolMapResult| -rel(r.n_lp_ratio, o.n_lp_ratio);
            let ordered = hp(&cell.r97) < hp(&one) && hp(&one) < hp(&two) && hp(&two) < hp(&cell.ell)
                && lp(&cell.r97) > lp(&one) && lp(&one) > lp(&two) && lp(&two) > lp(&cell.ell);
            if !ordered { misses.push(format!("{}", cell.name)); }
            n += 1;
        }
    }
    // On the equilibrium gas, choked: |Δπ_HPT| against the pin ordered one row < two rows < ellipse, all > 0.
    let mut m = 0;
    for (mname, (ml, mh)) in [("flat", (ComponentMap::flat(), ComponentMap::flat())), ("steep", shape_maps("steep").unwrap())] {
        let laws = [TurbineLaw::Pin, TurbineLaw::OneRow, TurbineLaw::TwoRows, TurbineLaw::Ellipse].map(|l| r39(Gas::reacting_equilibrium(), ml, mh, l));
        for (fname, fl) in [("design", fd39()), ("2km M0.80", flight(0.0, 2000.0, 0.80))] {
            for i in 0..7 {
                let tt4 = 800.0 + 100.0 * i as f64;
                let r: Vec<TwoSpoolResult> = laws.iter().map(|c| c.try_match_point(&fl, tt4).expect("pre-flighted").base).collect();
                let d: Vec<f64> = r[1..].iter().map(|x| rel(x.pi_hpt, r[0].pi_hpt).abs()).collect();
                if !(0.0 < d[0] && d[0] < d[1] && d[1] < d[2]) { misses.push(format!("eq {mname} {fname} {tt4}: {d:?}")); }
                m += 1;
            }
        }
    }
    eprintln!("H2: {n} perfect-gas cells below choke, {m} equilibrium-gas cells choked");
    for x in &misses { eprintln!("  MISS {x}"); }
    assert_eq!(m, 28);
    assert!(misses.is_empty(), "{} misses", misses.len());
}

// --------------------------------------------------------------------------------- the sort's corrections (spec § 5)

/// RUNG 103's HEADLINE, RE-CHECKED (category 2 — a correction). Rung 103: below choke a better LP compressor LOWERS the
/// HP pressure ratio. Its own instrument (`Δπ_HPC` for `η_LPC` 0.90 → 0.91 through the full match, its branch on) on its
/// own grid: on the pin every below-choke cell is negative, as rung 103 found; under EVERY law — one row and two rows
/// included, so the flip is not the extreme law's alone — every one is POSITIVE. The arrow from the LP compressor to the
/// HP spool survives below choke; its direction reverses (spec § 5).
#[test]
fn s103_below_choke_a_better_lp_compressor_raises_the_hp_pressure_ratio_under_every_law() {
    let fd = fd39();
    let f = ComponentMap::flat();
    let shapes = [("flat", f, f),
                  ("mixed", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                            ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
                  ("steep", shape_maps("steep").unwrap().0, shape_maps("steep").unwrap().1)];
    let laws = [TurbineLaw::Pin, TurbineLaw::OneRow, TurbineLaw::TwoRows, TurbineLaw::Ellipse];
    let mut tally = [[0usize; 2]; 4];
    for gas in [cpg(), Gas::reacting_equilibrium()] {
        for (_, ml, mh) in shapes.iter() {
            for (k, law) in laws.iter().enumerate() {
                let mut m = TwoSpoolMapMatcher::new(build_two_spool_turbojet(gas.clone(), 3.0, 6.0, 1500.0, 50_000.0, real())
                                                        .with_turbine_law(*law), fd, 1.0, *ml, *mh);
                for fl in [fd, flight(0.0, 0.0, 0.50)] {
                    for i in 0..7 {
                        let tt4 = 450.0 + 50.0 * i as f64;
                        let mut at = |eta: f64| {
                            m.core_mut().base.eta_lpc = eta;
                            m.core().base.below_choke.set(true);
                            let r = m.core().try_match_point(&fl, tt4).ok();
                            m.core().base.below_choke.set(false);
                            r
                        };
                        let (a, b) = (at(0.90), at(0.91));
                        m.core_mut().base.eta_lpc = 0.90;
                        let (Some(a), Some(b)) = (a, b) else { continue };
                        if a.base.m9 >= 1.0 - 1e-9 { continue }
                        tally[k][(b.base.pi_hpc > a.base.pi_hpc) as usize] += 1;
                    }
                }
            }
        }
    }
    eprintln!("S103: below-choke cells with dπ_HPC < 0 / > 0 — pin {:?}, one row {:?}, two rows {:?}, ellipse {:?}",
              tally[0], tally[1], tally[2], tally[3]);
    assert!(tally[0][0] >= 30 && tally[0][1] == 0, "on the pin rung 103's arrow points DOWN");
    for k in 1..4 {
        assert!(tally[k][1] >= 40 && tally[k][0] == 0, "{:?}: the arrow points UP at every cell", laws[k]);
    }
}
