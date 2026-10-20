//! RUNG 105 — THE HP TURBINE'S VANE IS NOT CHOKED AT A NORMAL REACTION: a reaction law beside rungs 31/38's pin on `A4`,
//! a knob that defaults to the old pin.
//!
//! Anchor (bars frozen before any rung-105 code): `docs/plans/rung105-anchor-hp-vane.md` — gates § 2 (R2–R10; R1 is the
//! whole gate on the default, and R1b the gate built with `hp-law-default`), scored claims § 3 (H1–H4). Spec:
//! `docs/rung105-spec.md`.

use turbojet::atmosphere::standard;
use turbojet::blade_speed::shape_maps;
use turbojet::components::{choked_mfp, ram_recovery, try_choked_mfp, try_sonic_throat};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::hp_vane::{HpCapacity, HpVane};
use turbojet::map::ComponentMap;
use turbojet::matcher::{OffDesignMatcher, OffDesignResult};
use turbojet::sandbox::{fly, fly_solver, FlySettings, GasModel};
use turbojet::sandbox_blades::{rig_losses, BladeSettings};
use turbojet::sandbox_controls::ControlsSettings;
use turbojet::sandbox_cooling::CoolingSettings;
use turbojet::spool::SpoolTransient;
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolCore, TwoSpoolMapCore, TwoSpoolMapMatcher, TwoSpoolMapResult,
                          TwoSpoolMatcher};
use turbojet::visuals::Json;

// --------------------------------------------------------------------------------- engines, laws and cells

const R10: HpVane = HpVane::Rows { reaction: 0.10 };
const R30: HpVane = HpVane::Rows { reaction: 0.30 };
const R50: HpVane = HpVane::Rows { reaction: 0.50 };

fn flight(dt: f64, h: f64, m0: f64) -> FlightCondition {
    let (t, p) = standard(h);
    FlightCondition::new(t + dt, p, m0)
}
fn rel(a: f64, b: f64) -> f64 { a / b - 1.0 }
fn cpg() -> Gas {
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9, gamma_t: 1.3, cp_t: 1239.0,
                       r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() })
}
fn fd() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn full(m: TwoSpoolMapMatcher) -> TwoSpoolMapCore {
    match m { TwoSpoolMapMatcher::Full(c) => c, _ => unreachable!("both shafts") }
}

/// The Fly-it design (slice 1's opening knobs, the convergent nozzle) on `gas`, through rung 31/33's matcher.
fn one_shaft(gas: Gas) -> OffDesignMatcher {
    let d = turbojet::sandbox::Settings::defaults();
    let l = Losses { nozzle_convergent: true, p_exit: None, ..d.losses() };
    OffDesignMatcher::new(build_turbojet(gas, d.pi_c, d.tt4, fd().p0, l), fd(), 1.0)
}
/// Rung 39's rig (`π` 3/6, `Tt4` 1500 K at 250 K / 50 kPa / Mach 0.85) on `gas` and a map pair.
fn rig(gas: Gas, ml: ComponentMap, mh: ComponentMap) -> TwoSpoolMapCore {
    full(TwoSpoolMapMatcher::new(build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, 50_000.0, rig_losses()), fd(), 1.0, ml, mh))
}
/// The Controls engine (rung 85's rig at its defaults, the perfect gas) on one map shape.
fn controls_engine(shape: &str) -> TwoSpoolMapCore {
    let bs = BladeSettings::defaults();
    let (ml, mh) = shape_maps(shape).unwrap();
    full(TwoSpoolMapMatcher::new(bs.design(), bs.flight(), 1.0, ml, mh))
}
fn quiet<R>(f: impl FnOnce() -> R) -> Option<R> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).ok()
}

// S1 — the Fly-it page's own solve.
fn s1_flights() -> [(&'static str, FlightCondition); 3] {
    [("SL+20 M0.60", flight(20.0, 0.0, 0.60)), ("2km M0.50", flight(0.0, 2000.0, 0.50)), ("5km M0.70", flight(0.0, 5000.0, 0.70))]
}
fn s1_tt4() -> Vec<f64> { (0..6).map(|i| 900.0 + 100.0 * i as f64).collect() }
// S2 — rung 39's rig on the equilibrium gas.
const S2_SHAPES: [&str; 2] = ["steep", "flow/press"];
fn s2_flights() -> [(&'static str, FlightCondition); 2] {
    [("2km M0.80", flight(0.0, 2000.0, 0.80)), ("5km M0.70", flight(0.0, 5000.0, 0.70))]
}
fn s2_tt4() -> Vec<f64> { (0..7).map(|i| 800.0 + 100.0 * i as f64).collect() }
// S3 — the Controls engine below choke, rung 104's scored grid.
const S3_SHAPES: [&str; 3] = ["steep", "flat-eta", "flow/press"];
fn s3_flights() -> [(&'static str, FlightCondition); 3] {
    [("SL+20 M0.60", flight(20.0, 0.0, 0.60)), ("2km M0.50", flight(0.0, 2000.0, 0.50)), ("SL-10 M0.40", flight(-10.0, 0.0, 0.40))]
}
fn s3_tt4() -> Vec<f64> { (0..6).map(|i| 575.0 + 50.0 * i as f64).collect() }

/// A Fly-it point on the page's own solve.
struct FlyPt {
    pi_c: f64,
    mdot: f64,
    nu: f64,
    thrust: f64,
    choked: bool,
    /// `(Tt4, pt4, f, y = pt5/pt4, η_t)` — enough to read the vane back.
    vane: (f64, f64, f64, f64, f64),
}
fn fly_at(gas: GasModel, fl: &FlightCondition, tt4: f64, law: HpVane) -> Option<FlyPt> {
    let s = FlySettings { gas, tt4, t0: fl.t0, p0: fl.p0, m0: fl.m0, hp_vane: law, ..FlySettings::defaults() };
    let o = quiet(|| fly(&s))?.ok()?;
    let st = |k: &str| o.result.stations.iter().find(|(l, _)| *l == k).unwrap().1;
    let (s4, s5) = (st("4"), st("5"));
    Some(FlyPt { pi_c: o.point.pi_c, mdot: o.point.mdot_air, nu: o.point.nu, thrust: o.point.thrust,
                 choked: o.point.choked, vane: (s4.tt, s4.pt, s4.far, s5.pt / s4.pt, o.point.eta_t) })
}

/// Is the law's vane choked at a solved point? `cap` the engine's capture under `law`, `gas` its working gas there.
fn vane_choked(cap: &HpCapacity, gas: &Gas, y: f64, tt4: f64, f: f64, eta: f64) -> bool {
    cap.w4(gas, y, tt4, f, eta).unwrap().1
}

/// The S3 comparisons: the old model (both vanes pinned, pushed past choke), rung 104's model (HP pin, LP ellipse,
/// below choke on) and a law (LP ellipse, below choke on).
fn s3_run(c: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64, lp: TurbineLaw, hp: HpVane, old: bool)
          -> Option<TwoSpoolMapResult> {
    c.base.turbine_law.set(lp);
    c.base.set_hp_vane(hp);
    c.base.pin_past_choke.set(old);
    c.base.below_choke.set(!old);
    let r = quiet(|| c.try_match_point(fl, tt4)).and_then(|x| x.ok());
    c.base.pin_past_choke.set(false);
    c.base.below_choke.set(false);
    c.base.set_hp_vane(HpVane::Pin);
    r
}
struct S3Cell {
    name: String,
    fl: FlightCondition,
    tt4: f64,
    old: TwoSpoolMapResult,
    r104: TwoSpoolMapResult,
    law: TwoSpoolMapResult,
}
/// § 3's S3 cells of one map: the old model, rung 104's and the law all solve; the law's nozzle unchoked (`M9 ≤ 0.99`);
/// its thrust at least 50 N per kg/s of design air (no near-idle cells).
fn s3_cells(shape: &str, c: &TwoSpoolMapCore) -> Vec<S3Cell> {
    let mut out = Vec::new();
    for (fname, fl) in s3_flights() {
        for tt4 in s3_tt4() {
            let (Some(o), Some(q), Some(e)) = (s3_run(c, &fl, tt4, TurbineLaw::Pin, HpVane::Pin, true),
                                               s3_run(c, &fl, tt4, TurbineLaw::Ellipse, HpVane::Pin, false),
                                               s3_run(c, &fl, tt4, TurbineLaw::Ellipse, R30, false)) else { continue };
            if e.base.m9 <= 0.99 && e.base.thrust >= 50.0 * c.base.mdot_air_design {
                out.push(S3Cell { name: format!("{shape} {fname} {tt4}"), fl, tt4, old: o, r104: q, law: e });
            }
        }
    }
    out
}

// --------------------------------------------------------------------------------- R2, R3

/// R2 — the perfect gas, nozzle choked: the law's own search (shortcut off) is the pin to ≤ 1e−12; with the shortcut on,
/// bit for bit. Both engines, reactions 0.10/0.3/0.5, on § 3's flights and throttles.
#[test]
fn r2_perfect_gas_choked_every_law_is_the_pin() {
    let (mut n, mut worst) = (0, 0.0f64);
    let m = one_shaft(cpg());
    for (_, fl) in s1_flights() {
        for tt4 in s1_tt4() {
            m.set_hp_vane(HpVane::Pin);
            let p = m.match_point(&fl, tt4);
            if !p.nozzle_choked { continue }
            for law in [R10, R30, R50] {
                m.set_hp_vane(law);
                let q = m.match_point(&fl, tt4);
                for (x, y) in [(q.pi_t, p.pi_t), (q.pi_c, p.pi_c), (q.mdot_air, p.mdot_air), (q.thrust, p.thrust)] {
                    assert_eq!(x.to_bits(), y.to_bits(), "shortcut on: {tt4} {law:?}");
                }
                m.hp_shortcut.set(false);
                let q = m.match_point(&fl, tt4);
                m.hp_shortcut.set(true);
                for (x, y) in [(q.pi_t, p.pi_t), (q.pi_c, p.pi_c), (q.mdot_air, p.mdot_air), (q.thrust, p.thrust)] {
                    let d = rel(x, y).abs();
                    worst = worst.max(d);
                    assert!(d <= 1e-12, "one shaft {tt4} {law:?}: {x} against {y}");
                }
                n += 1;
            }
        }
    }
    for shape in S2_SHAPES {
        let (ml, mh) = shape_maps(shape).unwrap();
        let c = rig(cpg(), ml, mh);
        for (_, fl) in s2_flights() {
            for tt4 in s2_tt4() {
                c.base.set_hp_vane(HpVane::Pin);
                let Ok(p) = c.try_match_point(&fl, tt4) else { continue };
                if p.base.m9 < 1.0 - 1e-9 { continue }
                for law in [R10, R30, R50] {
                    c.base.set_hp_vane(law);
                    let q = c.try_match_point(&fl, tt4).expect("the pin solved here");
                    let pairs = |q: &TwoSpoolMapResult| [(q.base.pi_hpt, p.base.pi_hpt), (q.base.pi_lpt, p.base.pi_lpt),
                                                         (q.base.pi_lpc, p.base.pi_lpc), (q.base.pi_hpc, p.base.pi_hpc),
                                                         (q.n_lp_ratio, p.n_lp_ratio), (q.n_hp_ratio, p.n_hp_ratio),
                                                         (q.base.mdot_air, p.base.mdot_air), (q.base.thrust, p.base.thrust)];
                    for (x, y) in pairs(&q) {
                        assert_eq!(x.to_bits(), y.to_bits(), "shortcut on: {shape} {tt4} {law:?}");
                    }
                    c.base.law_shortcut.set(false);
                    let q = c.try_match_point(&fl, tt4).expect("the pin solved here");
                    c.base.law_shortcut.set(true);
                    for (x, y) in pairs(&q) {
                        let d = rel(x, y).abs();
                        worst = worst.max(d);
                        assert!(d <= 1e-12, "two shafts {shape} {tt4} {law:?}: {x} against {y}");
                    }
                    n += 1;
                }
                c.base.set_hp_vane(HpVane::Pin);
            }
        }
    }
    eprintln!("R2: {n} choked cells x laws, the law's own search within {worst:.2e} of the pin");
    assert!(n >= 100);
}

/// R3 — the design point: every reaction lands no further from the design run than 3x the pin's own distance, or 1e−12.
#[test]
fn r3_every_law_reproduces_the_design_run() {
    for (gn, gas) in [("perfect", cpg()), ("thermally perfect", Gas::thermally_perfect()), ("equilibrium", Gas::reacting_equilibrium())] {
        let m = one_shaft(gas.clone());
        let (s4, s5) = (*m.reference.station("4"), *m.reference.station("5"));
        let d = turbojet::sandbox::Settings::defaults();
        let dist = |r: &OffDesignResult| [rel(r.pi_t, s5.pt / s4.pt).abs(), rel(r.pi_c, d.pi_c).abs(), rel(r.mdot_air, 1.0).abs()];
        m.set_hp_vane(HpVane::Pin);
        let pin = dist(&m.match_point(&fd(), d.tt4));
        for law in [R10, R30, R50] {
            m.set_hp_vane(law);
            let q = dist(&m.match_point(&fd(), d.tt4));
            for k in 0..3 {
                assert!(q[k] <= (3.0 * pin[k]).max(1e-12), "one shaft {gn} {law:?}: {q:?} against the pin's {pin:?}");
            }
        }
        let c = rig(gas, ComponentMap::flat(), ComponentMap::flat());
        let (s4, s45) = (*c.base.reference.station("4"), *c.base.reference.station("45"));
        let dist = |r: &TwoSpoolMapResult| [rel(r.base.pi_hpt, s45.pt / s4.pt).abs(), rel(r.base.pi_hpc, 6.0).abs(),
                                            rel(r.base.pi_lpc, 3.0).abs(), rel(r.base.mdot_air, 1.0).abs()];
        c.base.set_hp_vane(HpVane::Pin);
        let pin = dist(&c.try_match_point(&fd(), 1500.0).unwrap());
        for law in [R10, R30, R50] {
            c.base.set_hp_vane(law);
            let q = dist(&c.try_match_point(&fd(), 1500.0).unwrap());
            for k in 0..4 {
                assert!(q[k] <= (3.0 * pin[k]).max(1e-12), "two shafts {gn} {law:?}: {q:?} against the pin's {pin:?}");
            }
        }
    }
}

// --------------------------------------------------------------------------------- R4 (and R6's perfect-gas half)

/// The reaction law written out on the perfect gas, independently of `hp_vane`: closed-form isentropic row fluxes capped
/// at the sonic ratio, the textbook-reaction split at design, the interstage pressure by bisection.
struct HandVane {
    gt: f64,
    rt: f64,
    cpt: f64,
    eta: f64,
    td: f64,
    w4d: f64,
    a1: f64,
    a2: f64,
}
impl HandVane {
    fn flux(&self, p: f64, t: f64, pb: f64) -> f64 {
        let (g, et) = (self.gt, (self.gt - 1.0) / self.gt);
        let r = (pb / p).max((2.0 / (g + 1.0)).powf(g / (g - 1.0)));
        let ts = t * r.powf(et);
        p * r / (self.rt * ts) * (2.0 * self.cpt * (t - ts)).sqrt()
    }
    fn t1(&self, tt4: f64, p1: f64) -> f64 { tt4 * (1.0 - self.eta * (1.0 - p1.powf((self.gt - 1.0) / self.gt))) }
    fn new(a4: f64, td: f64, yd: f64, eta: f64, reaction: f64) -> HandVane {
        let (gt, cpt) = (1.3f64, 1239.0f64);
        let rt = (gt - 1.0) / gt * cpt;
        let et = (gt - 1.0) / gt;
        let mfp = (gt / rt).sqrt() * (2.0 / (gt + 1.0)).powf((gt + 1.0) / (2.0 * (gt - 1.0)));
        let mut h = HandVane { gt, rt, cpt, eta, td, w4d: a4 * mfp, a1: 0.0, a2: 0.0 };
        let p1d = (1.0 - (1.0 - reaction) * eta * (1.0 - yd.powf(et))).powf(1.0 / et);
        h.a1 = 1.0 / h.flux(1.0, td, p1d);
        h.a2 = 1.0 / h.flux(p1d, h.t1(td, p1d), yd);
        h
    }
    fn w4(&self, y: f64, tt4: f64) -> f64 {
        let r = |p1: f64| self.a1 * self.flux(1.0, tt4, p1) - self.a2 * self.flux(p1, self.t1(tt4, p1), y);
        let p1 = bisect(y + 1e-12, 1.0 - 1e-12, r);
        self.w4d * self.a1 * self.flux(1.0, tt4, p1) * (tt4 / self.td).sqrt()
    }
}
fn bisect(mut lo: f64, mut hi: f64, f: impl Fn(f64) -> f64) -> f64 {
    let flo = f(lo);
    for _ in 0..200 {
        let m = 0.5 * (lo + hi);
        if f(m) * flo > 0.0 { lo = m } else { hi = m }
    }
    0.5 * (lo + hi)
}

/// The one-shaft point below choke by hand on the perfect gas, under reaction 0.3: for a trial `π_t` the shaft balance
/// and the compressor fix `pt4`; the vane's flow at `π_t` against the subsonic nozzle's — the first sign change from deep
/// expansion upward, bisected. Returns `(π_t, π_c)`.
fn hand_one_shaft(m: &OffDesignMatcher, fl: &FlightCondition, tt4: f64) -> Option<(f64, f64)> {
    let (gc, cpc, gt, cpt, hpr) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64, 42.8e6f64);
    let rt = (gt - 1.0) / gt * cpt;
    let (ec, et) = ((gc - 1.0) / gc, (gt - 1.0) / gt);
    let tt2 = fl.t0 * (1.0 + 0.5 * (gc - 1.0) * fl.m0 * fl.m0);
    let pt2 = m.pi_d_max * ram_recovery(fl.m0) * fl.p0 * (tt2 / fl.t0).powf(1.0 / ec);
    let (s4, s5) = (m.reference.station("4"), m.reference.station("5"));
    let v = HandVane::new(m.a4, s4.tt, s5.pt / s4.pt, m.eta_t, 0.3);
    let point = |y: f64| -> (f64, f64) {
        let tt5 = tt4 * (1.0 - m.eta_t * (1.0 - y.powf(et)));
        let (mut f, mut pi_c) = (m.f_design, 0.0);
        for _ in 0..400 {
            let tt3 = tt2 + m.eta_m * (1.0 + f) * cpt * (tt4 - tt5) / cpc;
            pi_c = (1.0 + m.eta_c * (tt3 / tt2 - 1.0)).powf(1.0 / ec);
            let fnew = (cpt * tt4 - cpc * tt3) / (m.eta_b * hpr - cpt * tt4);
            if (fnew - f).abs() <= 1e-16 { f = fnew; break }
            f = fnew;
        }
        let tt3 = tt2 + m.eta_m * (1.0 + f) * cpt * (tt4 - tt5) / cpc;
        pi_c = (1.0 + m.eta_c * (tt3 / tt2 - 1.0)).powf(1.0 / ec);
        let pt4 = m.pi_b * pi_c * pt2;
        let supply = v.w4(y, tt4) * pt4 / tt4.sqrt();
        let pt9 = m.pi_n * y * pt4;
        let m9 = (((pt9 / fl.p0).powf(et) - 1.0) / (0.5 * (gt - 1.0))).max(0.0).sqrt().min(1.0);
        let t9 = tt5 / (1.0 + 0.5 * (gt - 1.0) * m9 * m9);
        let p9 = if m9 < 1.0 { fl.p0 } else { pt9 / (1.0 + 0.5 * (gt - 1.0)).powf(1.0 / et) };
        let demand = m.a8 * p9 / (rt * t9) * m9 * (gt * rt * t9).sqrt();
        ((supply - demand) / supply, pi_c)
    };
    let grid: Vec<(f64, f64)> = (0..400).map(|i| 0.15 + 0.84 * (i as f64 + 0.5) / 400.0).map(|y| (y, point(y).0)).collect();
    let w = grid.windows(2).find(|w| w[0].1 * w[1].1 < 0.0)?;
    let s = w[0].1.signum();
    let y = bisect(w[0].0, w[1].0, |y| point(y).0 * s);
    Some((y, point(y).1))
}

/// The two-shaft point below choke by hand on the perfect gas, under reaction 0.3 and rung 104's ellipse (rung 104's R4
/// solve with the HP side's choked `A4` replaced by the law): for a trial `x = π_LPT` the HP ratio where the vane's flow
/// meets the LP vane's; the nozzle's mass residual bisected. Returns `(π_HPT, π_LPT, π_LPC, π_HPC)`.
fn hand_two_shaft(c: &TwoSpoolCore, fl: &FlightCondition, tt4: f64) -> Option<(f64, f64, f64, f64)> {
    let (gc, cpc, gt, cpt, hpr) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64, 42.8e6f64);
    let rt = (gt - 1.0) / gt * cpt;
    let (ec, et) = ((gc - 1.0) / gc, (gt - 1.0) / gt);
    let tt2 = fl.t0 * (1.0 + 0.5 * (gc - 1.0) * fl.m0 * fl.m0);
    let pt2 = c.pi_d_max * ram_recovery(fl.m0) * fl.p0 * (tt2 / fl.t0).powf(1.0 / ec);
    let tt_out = |tin: f64, pi: f64, eta: f64| tin * (1.0 - eta * (1.0 - pi.powf(et)));
    let d = &c.reference;
    let (s4, s45, s5) = (d.station("4"), d.station("45"), d.station("5"));
    let xd = s5.pt / s45.pt;
    let e = |x: f64| (1.0 - x * x).max(0.0).sqrt() / (1.0 - xd * xd).sqrt();
    let mfp = (gt / rt).sqrt() * (2.0 / (gt + 1.0)).powf((gt + 1.0) / (2.0 * (gt - 1.0)));
    let v = HandVane::new(c.a4, s4.tt, s45.pt / s4.pt, c.eta_hpt, 0.3);
    let point = |x: f64| -> Option<(f64, f64, f64, f64)> {
        let w45 = c.a45_pin() * mfp * e(x);
        let g = |p: f64| p * (tt4 / tt_out(tt4, p, c.eta_hpt)).sqrt() - v.w4(p, tt4) / w45;
        if !(g(0.02) < 0.0 && g(0.98) > 0.0) { return None }
        let ph = bisect(0.02, 0.98, g);
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
        let supply = w45 * pt45 / tt45.sqrt();
        let pt9 = c.pi_n * x * pt45;
        let m9 = (((pt9 / fl.p0).powf(et) - 1.0) / (0.5 * (gt - 1.0))).max(0.0).sqrt();
        let t9 = tt5 / (1.0 + 0.5 * (gt - 1.0) * m9 * m9);
        let demand = c.a8 * fl.p0 / (rt * t9) * m9 * (gt * rt * t9).sqrt();
        Some(((supply - demand) / supply, ph, out.0, out.1))
    };
    let grid: Vec<(f64, f64)> = (0..400).filter_map(|i| {
        let x = 0.3 + 0.69 * (i as f64 + 0.5) / 400.0;
        point(x).map(|p| (x, p.0))
    }).collect();
    let w = grid.windows(2).find(|w| w[0].1 * w[1].1 < 0.0)?;
    let s = w[0].1.signum();
    let x = bisect(w[0].0, w[1].0, |m| point(m).map_or(f64::NAN, |p| p.0) * s);
    let p = point(x)?;
    Some((p.1, x, p.2, p.3))
}

/// R4 — a hand-written perfect-gas solve below choke under reaction 0.3, both engines: ≤ 1e−10.
#[test]
fn r4_a_hand_written_perfect_gas_solve_below_choke() {
    assert_eq!(cpg().h_c(1.0), 1004.0, "the hand solves assume h = cp*T");
    let (mut n1, mut n2, mut worst) = (0, 0, 0.0f64);
    let m = one_shaft(cpg());
    m.set_hp_vane(R30);
    for fl in [FlightCondition::new(318.15, 101_325.0, 0.3), FlightCondition::new(288.15, 101_325.0, 0.3),
               FlightCondition::new(288.15, 101_325.0, 0.5)] {
        for i in 0..7 {
            let tt4 = 650.0 + 50.0 * i as f64;
            let Some(r) = quiet(|| m.match_point(&fl, tt4)) else { continue };
            if r.nozzle_choked { continue }
            let (pt, pc) = hand_one_shaft(&m, &fl, tt4).expect("the hand solve finds the point the matcher found");
            for (k, x, y) in [("pi_t", r.pi_t, pt), ("pi_c", r.pi_c, pc)] {
                let d = rel(x, y).abs();
                worst = worst.max(d);
                assert!(d <= 1e-10, "one shaft {} {tt4}: {k} shipped {x} hand {y}", fl.m0);
            }
            n1 += 1;
        }
    }
    let m38 = TwoSpoolMatcher::new(build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, rig_losses())
                                       .with_turbine_law(TurbineLaw::Ellipse).with_hp_vane(R30), fd(), 1.0);
    let c = m38.core();
    c.below_choke.set(true);
    let m39 = rig(cpg(), ComponentMap::flat(), ComponentMap::flat());
    m39.base.turbine_law.set(TurbineLaw::Ellipse);
    m39.base.set_hp_vane(R30);
    m39.base.below_choke.set(true);
    for fl in [flight(0.0, 0.0, 0.30), flight(40.0, 0.0, 0.85), fd()] {
        for i in 0..8 {
            let tt4 = 450.0 + 50.0 * i as f64;
            let Some(r) = quiet(|| c.match_point(&fl, tt4)) else { continue };
            if r.m9 >= 1.0 - 1e-9 { continue }
            let (ph, pl, plc, phc) = hand_two_shaft(c, &fl, tt4).expect("the hand solve finds the point the matcher found");
            let q = m39.try_match_point(&fl, tt4).expect("rung 39 flat solves where rung 38 does");
            for (k, x, y) in [("38 piHPT", r.pi_hpt, ph), ("38 piLPT", r.pi_lpt, pl), ("38 piLPC", r.pi_lpc, plc),
                              ("38 piHPC", r.pi_hpc, phc), ("39 piHPT", q.base.pi_hpt, ph), ("39 piLPT", q.base.pi_lpt, pl),
                              ("39 piHPC", q.base.pi_hpc, phc)] {
                let d = rel(x, y).abs();
                worst = worst.max(d);
                assert!(d <= 1e-10, "two shafts {tt4}: {k} shipped {x} hand {y}");
            }
            n2 += 1;
        }
    }
    eprintln!("R4: {n1} one-shaft and {n2} two-shaft unchoked cells, worst {worst:.2e}");
    assert!(n1 >= 6 && n2 >= 6, "{n1} / {n2} unchoked cells");
}

// --------------------------------------------------------------------------------- R5, R6 (the realistic half), R7

/// The isentropic flux through a convergent row from total `(p, t)` to `pb`, capped at the gas's own sonic point — written
/// out here with the gas's inversions, independently of the shipped `flux`.
fn row_flux(gas: &Gas, f: f64, p: f64, t: f64, pb: f64) -> f64 {
    let (ts_star, p_star, v_star) = try_sonic_throat(gas, t, p, f).unwrap();
    if pb <= p_star {
        return p_star / (gas.r_t_at(f) * ts_star) * v_star;
    }
    let ts = gas.try_t_from_pr_t(gas.pr_t(t, f) * pb / p, f).unwrap();
    pb / (gas.r_t_at(f) * ts) * (2.0 * (gas.h_t(t, f) - gas.h_t(ts, f))).max(0.0).sqrt()
}
/// The temperature after an expansion from `tt` to `pi` at efficiency `eta`, written out.
fn t_out(gas: &Gas, tt: f64, f: f64, pi: f64, eta: f64) -> f64 {
    let ts = gas.try_t_from_pr_t(gas.pr_t(tt, f) * pi, f).unwrap();
    gas.t_from_h_t(gas.h_t(tt, f) - eta * (gas.h_t(tt, f) - gas.h_t(ts, f)), f)
}
/// The reaction law re-derived from a capture's DESIGN data alone (not its stored areas): the textbook split, both rows'
/// areas, the interstage pressure by bisection, `W4` at `(y, Tt4)` on `gas`. `dgas` is the design run's gas.
fn w4_indep(dgas: &Gas, cap: &HpCapacity, reaction: f64, gas: &Gas, y: f64, tt4: f64, f: f64, eta: f64) -> f64 {
    let (fd, td, yd, etad) = (cap.fd, cap.td, cap.yd, cap.eta);
    let h4 = dgas.h_t(td, fd);
    let tys = dgas.try_t_from_pr_t(dgas.pr_t(td, fd) * yd, fd).unwrap();
    let hs1 = h4 - (1.0 - reaction) * etad * (h4 - dgas.h_t(tys, fd));
    let t1s = dgas.t_from_h_t(hs1, fd);
    let p1d = dgas.pr_t(t1s, fd) / dgas.pr_t(td, fd);
    let a1 = 1.0 / row_flux(dgas, fd, 1.0, td, p1d);
    let a2 = 1.0 / row_flux(dgas, fd, p1d, t_out(dgas, td, fd, p1d, etad), yd);
    let r = |p1: f64| a1 * row_flux(gas, f, 1.0, tt4, p1) - a2 * row_flux(gas, f, p1, t_out(gas, tt4, f, p1, eta), y);
    let p1 = bisect(y + 1e-12, 1.0 - 1e-12, r);
    cap.w4d * a1 * row_flux(gas, f, 1.0, tt4, p1) * (tt4 / td).sqrt()
}

/// R5 — one root: the shipped searches' residuals, outer state held, have one sign change on a 101-point grid over their
/// range, the root ≥ 0.02 from both ends — rung 31's (★) under the law on S1's cells (the thermally perfect gas), the
/// joint HP search on S2's and S3's, and the interstage search at every solved vane.
#[test]
fn r5_one_root_inside_every_search() {
    let one_change = |g: &[f64]| g.windows(2).filter(|w| w[0] * w[1] < 0.0).count();
    let (mut n, mut ni) = (0, 0);
    // The interstage residual at a solved vane, over its whole range (y, 1).
    let interstage = |cap: &HpCapacity, gas: &Gas, y: f64, tt4: f64, f: f64, eta: f64| -> usize {
        let [a1, a2] = cap.rows;
        let g: Vec<f64> = (1..=101).map(|i| {
            let p1 = y + (1.0 - y) * i as f64 / 102.0;
            a1 * row_flux(gas, f, 1.0, tt4, p1) - a2 * row_flux(gas, f, p1, t_out(gas, tt4, f, p1, eta), y)
        }).collect();
        one_change(&g)
    };
    let m = one_shaft(Gas::thermally_perfect());
    m.set_hp_vane(R30);
    let cap = m.hp_cap.get();
    for (fname, fl) in s1_flights() {
        for tt4 in s1_tt4() {
            let r = m.match_point(&fl, tt4);
            assert!(r.nozzle_choked);
            let s4 = r.station("4");
            let gas = m.gas();
            let g: Vec<f64> = (0..101).map(|i| {
                let y = 0.05 + (0.9995 - 0.05) * i as f64 / 100.0;
                let (tau, tt5) = m.tau_t_of_pi_t(gas, tt4, s4.far, y, None);
                cap.w4(gas, y, tt4, s4.far, m.eta_t).unwrap().0
                    - m.a8 * m.pi_n * choked_mfp(gas, tt5, s4.far) * y / tau.sqrt()
            }).collect();
            assert_eq!(one_change(&g), 1, "S1 {fname} {tt4}");
            assert!(r.pi_t >= 0.07 && r.pi_t <= 0.9795, "S1 {fname} {tt4}: root {} near an end", r.pi_t);
            assert_eq!(interstage(&cap, gas, r.pi_t, tt4, s4.far, m.eta_t), 1, "S1 interstage {fname} {tt4}");
            n += 1;
            ni += 1;
        }
    }
    // The joint residual at a two-shaft result's outer state: the vane's flow under the law against the LP vane's.
    let joint = |c: &TwoSpoolMapCore, r: &TwoSpoolMapResult, p0: Option<f64>, ph: f64| -> f64 {
        let b = &c.base;
        let s4 = r.base.station("4");
        let (f, pt4, tt4) = (s4.far, s4.pt, s4.tt);
        let owned = b.try_working_gas(f, tt4, pt4).unwrap();
        let gas = owned.as_ref().unwrap_or(b.gas());
        let (_, tt45) = b.try_tau_of(gas, tt4, f, ph, r.eta_hpt).unwrap();
        let hp = b.hp_cap.get().w4(gas, ph, tt4, f, r.eta_hpt).unwrap().0 * (tt45 / tt4).sqrt() / ph;
        let lp = match p0 {
            None => b.try_lp_choked(gas, tt45, f, r.eta_lpt).map(|x| x.0),
            Some(p0) => turbojet::unchoked_lp::try_lp_turbine(b, gas, tt45, f, r.eta_lpt, ph * pt4, p0).map(|x| x.0),
        };
        match lp {
            Ok(x) => hp - b.w45(gas, x, tt45, f, r.eta_lpt).unwrap(),
            Err(_) => hp,
        }
    };
    let grid = |c: &TwoSpoolMapCore, r: &TwoSpoolMapResult, p0: Option<f64>| -> Vec<f64> {
        (0..101).map(|i| joint(c, r, p0, 0.01 + 0.98 * i as f64 / 100.0)).collect()
    };
    for shape in S2_SHAPES {
        let (ml, mh) = shape_maps(shape).unwrap();
        let c = rig(Gas::reacting_equilibrium(), ml, mh);
        c.base.set_hp_vane(R30);
        for (fname, fl) in s2_flights() {
            for tt4 in s2_tt4() {
                let r = c.try_match_point(&fl, tt4).expect("pre-flighted");
                assert_eq!(one_change(&grid(&c, &r, None)), 1, "S2 {shape} {fname} {tt4}");
                assert!(r.base.pi_hpt >= 0.03 && r.base.pi_hpt <= 0.97, "S2 root near an end");
                let s4 = r.base.station("4");
                let owned = c.base.try_working_gas(s4.far, tt4, s4.pt).unwrap();
                let gas = owned.as_ref().unwrap_or(c.base.gas());
                assert_eq!(interstage(&c.base.hp_cap.get(), gas, r.base.pi_hpt, tt4, s4.far, r.eta_hpt), 1);
                n += 1;
                ni += 1;
            }
        }
    }
    for shape in S3_SHAPES {
        let c = controls_engine(shape);
        for cell in s3_cells(shape, &c) {
            c.base.turbine_law.set(TurbineLaw::Ellipse);
            c.base.set_hp_vane(R30);
            assert_eq!(one_change(&grid(&c, &cell.law, Some(cell.fl.p0))), 1, "S3 {}", cell.name);
            let ph = cell.law.base.pi_hpt;
            assert!((0.03..=0.97).contains(&ph), "S3 {}: root {ph} near an end", cell.name);
            let s4 = cell.law.base.station("4");
            assert_eq!(interstage(&c.base.hp_cap.get(), c.base.gas(), ph, cell.tt4, s4.far, cell.law.eta_hpt), 1);
            c.base.set_hp_vane(HpVane::Pin);
            n += 1;
            ni += 1;
        }
    }
    eprintln!("R5: {n} turbine searches and {ni} interstage searches, one root each");
    assert_eq!(n, 36 / 2 + 28 + 33, "S1 (one gas) + S2 + S3 as pre-flighted");
}

/// R6 — mass, read independently: the vane's flow re-derived from the RESULT's stations with an independent row law
/// (perfect gas: closed form; realistic gases: the gas's own sonic point and inversions, written out) equals the result's
/// station-4 flow to ≤ 1e−10.
#[test]
fn r6_mass_read_independently_of_the_search() {
    let (mut n, mut worst) = (0, 0.0f64);
    let mut check = |name: &str, w4_law: f64, s4: &turbojet::gas::FlowState| {
        let w4_result = s4.mdot * s4.tt.sqrt() / s4.pt;
        let d = rel(w4_law, w4_result).abs();
        worst = worst.max(d);
        assert!(d <= 1e-10, "{name}: the law gives W4 {w4_law}, the result passes {w4_result}");
        n += 1;
    };
    // The perfect gas, below choke (S3), closed form.
    for shape in S3_SHAPES {
        let c = controls_engine(shape);
        let (s4d, s45d) = (*c.base.reference.station("4"), *c.base.reference.station("45"));
        let hv = HandVane::new(c.base.a4, s4d.tt, s45d.pt / s4d.pt, c.base.eta_hpt, 0.3);
        for cell in s3_cells(shape, &c) {
            let (s4, s45) = (cell.law.base.station("4"), cell.law.base.station("45"));
            check(&cell.name, hv.w4(s45.pt / s4.pt, s4.tt), s4);
        }
    }
    // The equilibrium gas, choked (S2).
    for shape in S2_SHAPES {
        let (ml, mh) = shape_maps(shape).unwrap();
        let c = rig(Gas::reacting_equilibrium(), ml, mh);
        c.base.set_hp_vane(R30);
        let cap = c.base.hp_cap.get();
        for (fname, fl) in s2_flights() {
            for tt4 in s2_tt4() {
                let r = c.try_match_point(&fl, tt4).expect("pre-flighted");
                let (s4, s45) = (r.base.station("4"), r.base.station("45"));
                let owned = c.base.try_working_gas(s4.far, tt4, s4.pt).unwrap();
                let gas = owned.as_ref().unwrap_or(c.base.gas());
                let w = w4_indep(c.base.gas(), &cap, 0.3, gas, s45.pt / s4.pt, tt4, s4.far, r.eta_hpt);
                check(&format!("S2 {shape} {fname} {tt4}"), w, s4);
            }
        }
    }
    eprintln!("R6: {n} points, the vane's flow read back within {worst:.2e}");
    assert_eq!(n, 33 + 28);
}

/// R7 — the choke emerges: the reaction at which the capture's vane flips from choked to unchoked equals the closed form
/// `1 − (h(Tt4) − h*)/(η·(h(Tt4) − h_s(y_d)))` to ≤ 1e−9 (both engines, two gases); and below it the law returns
/// `A4·MFP*(Tt4)` bit for bit wherever the vane is choked.
#[test]
fn r7_the_choke_emerges_at_the_textbook_reaction() {
    let flip = |gas: &Gas, f: f64, tt4: f64, yd: f64, eta: f64, a4: f64| -> f64 {
        let (mut lo, mut hi) = (1e-6, 0.95);
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if HpCapacity::capture(gas, f, tt4, yd, eta, a4, HpVane::Rows { reaction: mid }).choked_d { lo = mid } else { hi = mid }
        }
        0.5 * (lo + hi)
    };
    let closed = |gas: &Gas, f: f64, tt4: f64, yd: f64, eta: f64| -> f64 {
        let (ts, _, _) = try_sonic_throat(gas, tt4, 1.0, f).unwrap();
        let tys = gas.try_t_from_pr_t(gas.pr_t(tt4, f) * yd, f).unwrap();
        let h4 = gas.h_t(tt4, f);
        1.0 - (h4 - gas.h_t(ts, f)) / (eta * (h4 - gas.h_t(tys, f)))
    };
    for (gn, gas) in [("perfect", cpg()), ("thermally perfect", Gas::thermally_perfect())] {
        let m = one_shaft(gas.clone());
        let (s4, s5) = (*m.reference.station("4"), *m.reference.station("5"));
        let (a, b) = (flip(m.gas(), s4.far, s4.tt, s5.pt / s4.pt, m.eta_t, m.a4), closed(m.gas(), s4.far, s4.tt, s5.pt / s4.pt, m.eta_t));
        assert!((a - b).abs() <= 1e-9, "one shaft {gn}: flips at {a}, the closed form says {b}");
        let c = TwoSpoolCore::new(build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, 50_000.0, rig_losses()), fd(), 1.0);
        let (s4, s45) = (*c.reference.station("4"), *c.reference.station("45"));
        let (a2, b2) = (flip(c.gas(), s4.far, s4.tt, s45.pt / s4.pt, c.eta_hpt, c.a4), closed(c.gas(), s4.far, s4.tt, s45.pt / s4.pt, c.eta_hpt));
        assert!((a2 - b2).abs() <= 1e-9, "two shafts {gn}: flips at {a2}, the closed form says {b2}");
        eprintln!("R7 {gn}: one shaft R* {a:.6} ({b:.6}), two shafts R* {a2:.6} ({b2:.6})");
    }
    // Below R*: choked, the law returns the pin's own expression.
    let m = one_shaft(Gas::thermally_perfect());
    m.set_hp_vane(R10);
    let cap = m.hp_cap.get();
    assert!(cap.choked_d);
    let mut n = 0;
    for (_, fl) in s1_flights() {
        for tt4 in s1_tt4() {
            let r = m.match_point(&fl, tt4);
            let s4 = r.station("4");
            let (w, choked) = cap.w4(m.gas(), r.pi_t, tt4, s4.far, m.eta_t).unwrap();
            if choked {
                assert_eq!(w.to_bits(), (m.a4 * choked_mfp(m.gas(), tt4, s4.far)).to_bits());
                n += 1;
            }
        }
    }
    assert!(n >= 12, "only {n} choked vanes at reaction 0.10");
}

// --------------------------------------------------------------------------------- R8, R10

/// R8 — the pages: the setting reaches each view's engine (a reply moves where the law moves the engine), round-trips,
/// is written only off its default, and is range-checked in words.
#[test]
fn r8_the_setting_reaches_each_views_engine() {
    // Fly it — below choke the point moves; the keys round-trip and appear only off the default.
    let s = FlySettings { tt4: 1000.0, t0: 318.15, p0: 101_325.0, m0: 0.3, ..FlySettings::defaults() };
    let (a, b) = (fly(&s).unwrap(), fly(&FlySettings { hp_vane: R30, ..s }).unwrap());
    assert!(!a.point.choked && a.point.pi_c != b.point.pi_c, "the law moves Fly it below choke");
    let j = FlySettings::defaults().to_json().dump_compact();
    assert!(!j.contains("hp_vane") && !j.contains("reaction"), "the default reply carries no rung-105 key");
    for law in [HpVane::Pin, R10, R30, R50] {
        let s = FlySettings { hp_vane: law, ..FlySettings::defaults() };
        assert_eq!(FlySettings::from_json(&s.to_json()).unwrap().hp_vane, law);
        let c = ControlsSettings { hp_vane: law, ..ControlsSettings::defaults() };
        assert_eq!(ControlsSettings::from_json(&c.to_json()).unwrap().hp_vane, law);
        let bl = BladeSettings { hp_vane: law, ..BladeSettings::defaults() };
        assert_eq!(BladeSettings::from_json(&bl.to_json()).unwrap().hp_vane, law);
        let k = CoolingSettings { hp_vane: law, ..CoolingSettings::defaults() };
        assert_eq!(CoolingSettings::from_json(&k.to_json()).unwrap().hp_vane, law);
    }
    for j in [ControlsSettings::defaults().to_json(), BladeSettings::defaults().to_json(), CoolingSettings::defaults().to_json()] {
        let t = j.dump_compact();
        assert!(!t.contains("hp_vane") && !t.contains("\"reaction\""), "a default reply carries no rung-105 key: {t}");
    }
    // Controls — the below-choke slam moves; Blades — its rig's engine carries the law; Cooling — a cooled point moves.
    let c = ControlsSettings { t0: 288.15 + 40.0, p0: 101_325.0, m0: 0.85, from: 700.0, to: 1400.0, ..ControlsSettings::defaults() };
    let (_, p) = turbojet::sandbox_controls::controls(&c).unwrap();
    let (_, q) = turbojet::sandbox_controls::controls(&ControlsSettings { hp_vane: R30, ..c }).unwrap();
    assert!(p.points.iter().zip(&q.points).any(|(x, y)| x.nu_hp != y.nu_hp), "the law moves the Controls slam below choke");
    assert_eq!(BladeSettings { hp_vane: R30, ..BladeSettings::defaults() }.design().hp_vane, R30);
    let k = CoolingSettings::defaults();
    let (x, y) = (turbojet::sandbox_cooling::cooling_point(&k, 1100.0).unwrap().dump_compact(),
                  turbojet::sandbox_cooling::cooling_point(&CoolingSettings { hp_vane: R30, ..k }, 1100.0).unwrap().dump_compact());
    assert_ne!(x, y, "the law moves a cooled point (the coolants move the HP turbine's expansion even choked)");
    // The range, in words.
    let e = fly(&FlySettings { hp_vane: HpVane::Rows { reaction: 0.9 }, ..FlySettings::defaults() }).err().expect("refused");
    assert!(e.0.contains("reaction must be between"), "{}", e.0);
    let e = fly(&FlySettings { hp_vane: R30, ab: Some(turbojet::afterburner::AbMode::Lit), ..FlySettings::defaults() })
        .err().expect("refused");
    assert!(e.0.contains("afterburner on Fly it runs with the HP turbine's vanes choked"), "{}", e.0);
}

/// R10 — the shortcut's edge (rung 104's lesson 4): a 1 K `Tt4` sweep through the unchoke boundary on the perfect gas —
/// rung 31/33's matcher, rung 39's, rung 34's instant and equilibrium, rung 40's instant — shortcut on against off within
/// 1e−10 (absolute below unit size), both classifying every step alike.
#[test]
fn r10_the_shortcut_at_the_unchoke_boundary() {
    let (mut n, mut worst) = (0, 0.0f64);
    let mut cmp = |what: &str, a: &[f64], b: &[f64], ca: bool, cb: bool| {
        assert_eq!(ca, cb, "{what}: the shortcut classifies the step differently");
        // Relative where the quantity is of order one or more; absolute where it is near zero (a shaft's power imbalance
        // at its equilibrium speed is round-off).
        for (x, y) in a.iter().zip(b) {
            let d = (x - y).abs() / y.abs().max(1.0);
            worst = worst.max(d);
            assert!(d <= 1e-10, "{what}: shortcut {x} without {y}");
        }
        n += 1;
    };
    // Where each engine unchokes at its flight: the first step of a coarse sweep whose nozzle is subsonic.
    let edge = |choked_at: &dyn Fn(f64) -> Option<bool>| -> f64 {
        let mut t = 1100.0;
        while t > 450.0 && choked_at(t).unwrap_or(true) { t -= 10.0 }
        t + 5.0
    };
    // One shaft, steady.
    let fl = FlightCondition::new(288.15, 101_325.0, 0.5);
    let m = one_shaft(cpg());
    m.set_hp_vane(R30);
    let t0 = edge(&|t| quiet(|| m.match_point(&fl, t)).map(|r| r.nozzle_choked));
    for i in -12..=12 {
        let tt4 = t0 + i as f64;
        let Some(a) = quiet(|| m.match_point(&fl, tt4)) else { continue };
        m.hp_shortcut.set(false);
        let b = quiet(|| m.match_point(&fl, tt4)).expect("the shortcut's point solves without it");
        m.hp_shortcut.set(true);
        cmp(&format!("rung 31/33 {tt4}"), &[a.pi_t, a.pi_c, a.mdot_air], &[b.pi_t, b.pi_c, b.mdot_air],
            a.nozzle_choked, b.nozzle_choked);
    }
    // One shaft, rung 34's instant (at the equilibrium speed of the edge) and equilibrium.
    let st = fly_solver(&FlySettings { gas: GasModel::Perfect, hp_vane: R30, ..FlySettings::defaults() });
    let inner = &st.inner.inner;
    let t1 = edge(&|t| quiet(|| st.equilibrium(&fl, t, None)).map(|e| e.branch == turbojet::matcher::Branch::Choked));
    let nu = st.equilibrium(&fl, t1, None).nu;
    for i in -12..=12 {
        let tt4 = t1 + i as f64;
        let Some(Ok(a)) = quiet(|| st.try_instant(&fl, nu, tt4, None)) else { continue };
        inner.hp_shortcut.set(false);
        let b = st.try_instant(&fl, nu, tt4, None).expect("the shortcut's instant runs without it");
        inner.hp_shortcut.set(true);
        cmp(&format!("rung 34 instant {tt4}"), &[a.pi_t, a.mdot_air, a.phi], &[b.pi_t, b.mdot_air, b.phi],
            a.branch == turbojet::matcher::Branch::Choked, b.branch == turbojet::matcher::Branch::Choked);
        let Some(a) = quiet(|| st.equilibrium(&fl, tt4, None)) else { continue };
        inner.hp_shortcut.set(false);
        let b = st.equilibrium(&fl, tt4, None);
        inner.hp_shortcut.set(true);
        cmp(&format!("rung 34 equilibrium {tt4}"), &[a.nu, a.pi_c, a.mdot_air], &[b.nu, b.pi_c, b.mdot_air],
            a.branch == turbojet::matcher::Branch::Choked, b.branch == turbojet::matcher::Branch::Choked);
    }
    // Two shafts, rung 39 below choke on, and rung 40's instant.
    let c = rig(cpg(), ComponentMap::flat(), ComponentMap::flat());
    c.base.set_hp_vane(R30);
    c.base.below_choke.set(true);
    let fl2 = flight(0.0, 0.0, 0.5);
    let t2 = edge(&|t| c.try_match_point(&fl2, t).ok().map(|r| r.base.m9 >= 1.0 - 1e-9));
    for i in -12..=12 {
        let tt4 = t2 + i as f64;
        let Ok(a) = c.try_match_point(&fl2, tt4) else { continue };
        c.base.law_shortcut.set(false);
        let b = c.try_match_point(&fl2, tt4).expect("the shortcut's point solves without it");
        c.base.law_shortcut.set(true);
        cmp(&format!("rung 39 {tt4}"), &[a.base.pi_hpt, a.base.pi_lpt, a.n_hp_ratio, a.n_lp_ratio],
            &[b.base.pi_hpt, b.base.pi_lpt, b.n_hp_ratio, b.n_lp_ratio], a.base.m9 >= 1.0 - 1e-9, b.base.m9 >= 1.0 - 1e-9);
    }
    let t = turbojet::two_spool_transient::TwoSpoolTransientCore::new(
        build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, rig_losses()).with_hp_vane(R30), fd(), 1.0,
        ComponentMap::flat(), ComponentMap::flat(), 1.0);
    let r = c.try_match_point(&fl2, t2).expect("the edge solves");
    for i in -12..=12 {
        let tt4 = t2 + i as f64;
        let Some(Ok(a)) = quiet(|| t.try_instant(&fl2, r.n_lp_ratio, r.n_hp_ratio, tt4)) else { continue };
        t.inner.base.law_shortcut.set(false);
        let b = t.try_instant(&fl2, r.n_lp_ratio, r.n_hp_ratio, tt4).expect("the shortcut's instant runs without it");
        t.inner.base.law_shortcut.set(true);
        cmp(&format!("rung 40 instant {tt4}"), &[a.pi_hpt, a.pi_lpt, a.phi_lp_dot, a.phi_hp_dot],
            &[b.pi_hpt, b.pi_lpt, b.phi_lp_dot, b.phi_hp_dot],
            a.branch == turbojet::matcher::Branch::Choked, b.branch == turbojet::matcher::Branch::Choked);
    }
    eprintln!("R10: {n} steps through the unchoke boundary, shortcut on against off within {worst:.2e}");
    assert!(n >= 80, "only {n} steps");
}

// --------------------------------------------------------------------------------- § 3 — SCORED (H1–H4)

/// S1 on both gases: `(gas, flight, Tt4, [pin, 0.10, 0.3, 0.5])` — all pre-flighted to solve, choked.
fn s1_points() -> Vec<(GasModel, &'static str, FlightCondition, f64, [FlyPt; 4])> {
    let mut out = Vec::new();
    for gm in [GasModel::ThermallyPerfect, GasModel::Equilibrium] {
        for (fname, fl) in s1_flights() {
            for tt4 in s1_tt4() {
                let p = [HpVane::Pin, R10, R30, R50].map(|law| fly_at(gm, &fl, tt4, law).expect("pre-flighted"));
                out.push((gm, fname, fl, tt4, p));
            }
        }
    }
    out
}

/// H1 + H2 on S1 (the one-shaft engine through the Fly-it page's own solve).
#[test]
fn h1_h2_s1_the_vane_is_never_choked_and_the_pin_was_nearly_right() {
    let (mut n, mut worst, mut misses) = (0, 0.0f64, Vec::new());
    for (gm, fname, fl, tt4, p) in s1_points() {
        let name = format!("S1 {} {fname} {tt4}", gm.key());
        assert!(p.iter().all(|x| x.choked), "{name}: pre-flighted choked");
        // H1: the solved vane at 0.3 is unchoked; at 0.10 choked (Tt4 >= 900).
        for (law, want) in [(R30, false), (R10, true)] {
            let st = fly_solver(&FlySettings { gas: gm, tt4, t0: fl.t0, p0: fl.p0, m0: fl.m0, hp_vane: law, ..FlySettings::defaults() });
            let mm = &st.inner.inner;
            let k = if law == R30 { 2 } else { 1 };
            let (t4, pt4, f, y, eta) = p[k].vane;
            let owned = mm.try_working_gas(f, t4, pt4).unwrap();
            let gas = owned.as_ref().unwrap_or(mm.gas());
            if vane_choked(&mm.hp_cap.get(), gas, y, t4, f, eta) != want {
                misses.push(format!("H1 {name} {law:?}: the vane's choked state is not {want}"));
            }
        }
        // H2: reaction 0.3 against the pin within 0.3 %.
        let (a, b) = (&p[2], &p[0]);
        for (q, x, y) in [("pi_c", a.pi_c, b.pi_c), ("mdot", a.mdot, b.mdot), ("N", a.nu, b.nu), ("F", a.thrust, b.thrust)] {
            let d = rel(x, y).abs();
            worst = worst.max(d);
            if d > 3e-3 { misses.push(format!("H2 {name}: {q} {:+.4} %", 100.0 * rel(x, y))); }
        }
        n += 1;
    }
    eprintln!("H1/H2 on S1: {n} cells, the largest change against the pin {:.4} %", 100.0 * worst);
    for m in &misses { eprintln!("  MISS {m}"); }
    assert_eq!(n, 36, "the pre-flight counted 36 S1 cells");
    assert!(misses.is_empty(), "{} misses", misses.len());
}

/// H1 + H2 + H4 on S2 (rung 39's rig on the equilibrium gas, choked).
#[test]
fn h1_h2_h4_s2_choked_two_shafts() {
    let (mut n, mut worst, mut misses) = (0, 0.0f64, Vec::new());
    for shape in S2_SHAPES {
        let (ml, mh) = shape_maps(shape).unwrap();
        let c = rig(Gas::reacting_equilibrium(), ml, mh);
        for (fname, fl) in s2_flights() {
            for tt4 in s2_tt4() {
                let name = format!("S2 {shape} {fname} {tt4}");
                let r: Vec<TwoSpoolMapResult> = [HpVane::Pin, R10, R30, R50].iter().map(|&law| {
                    c.base.set_hp_vane(law);
                    c.try_match_point(&fl, tt4).expect("pre-flighted")
                }).collect();
                assert!(r.iter().all(|x| x.base.m9 >= 1.0 - 1e-9), "{name}: pre-flighted choked");
                // H1: the solved vane at 0.3 unchoked; at 0.10 choked.
                for (k, law, want) in [(2usize, R30, false), (1, R10, true)] {
                    c.base.set_hp_vane(law);
                    let s4 = r[k].base.station("4");
                    let owned = c.base.try_working_gas(s4.far, tt4, s4.pt).unwrap();
                    let gas = owned.as_ref().unwrap_or(c.base.gas());
                    if vane_choked(&c.base.hp_cap.get(), gas, r[k].base.pi_hpt, tt4, s4.far, r[k].eta_hpt) != want {
                        misses.push(format!("H1 {name} {law:?}: the vane's choked state is not {want}"));
                    }
                }
                // H2: within 0.5 %; the work moves LP -> HP.
                let (a, b) = (&r[2], &r[0]);
                for (q, x, y) in [("pi_HPC", a.base.pi_hpc, b.base.pi_hpc), ("pi_LPC", a.base.pi_lpc, b.base.pi_lpc),
                                  ("N_H", a.n_hp_ratio, b.n_hp_ratio), ("N_L", a.n_lp_ratio, b.n_lp_ratio),
                                  ("F", a.base.thrust, b.base.thrust)] {
                    let d = rel(x, y).abs();
                    worst = worst.max(d);
                    if d > 5e-3 { misses.push(format!("H2 {name}: {q} {:+.4} %", 100.0 * rel(x, y))); }
                }
                if !(a.base.pi_hpc > b.base.pi_hpc && a.base.pi_lpc < b.base.pi_lpc) {
                    misses.push(format!("H2 {name}: the work does not move LP -> HP (dpiHPC {:+.2e}, dpiLPC {:+.2e})",
                                        rel(a.base.pi_hpc, b.base.pi_hpc), rel(a.base.pi_lpc, b.base.pi_lpc)));
                }
                // H4: |dpi_HPC| ordered 0.10 <= 0.3 < 0.5; 0.10 the pin where its vane is choked.
                let d: Vec<f64> = r[1..].iter().map(|x| rel(x.base.pi_hpc, b.base.pi_hpc).abs()).collect();
                if !(d[0] <= d[1] && d[1] < d[2]) { misses.push(format!("H4 {name}: {d:?}")); }
                if d[0] > 1e-12 { misses.push(format!("H4 {name}: 0.10 off the pin by {:.2e}", d[0])); }
                c.base.set_hp_vane(HpVane::Pin);
                n += 1;
            }
        }
    }
    eprintln!("H1/H2/H4 on S2: {n} cells, the largest change against the pin {:.4} %", 100.0 * worst);
    for m in &misses { eprintln!("  MISS {m}"); }
    assert_eq!(n, 28, "the pre-flight counted 28 S2 cells");
    assert!(misses.is_empty(), "{} misses", misses.len());
}

/// H1 + H3 + H4 on S3 (the Controls engine below choke, rung 104's scored grid).
#[test]
fn h1_h3_h4_s3_below_choke_the_hp_vane_hands_back_part_of_rung_104s_hp_slowdown() {
    let (mut n, mut excluded, mut misses) = (0, 0, Vec::new());
    let (mut s_min, mut s_max) = (f64::INFINITY, f64::NEG_INFINITY);
    for shape in S3_SHAPES {
        let c = controls_engine(shape);
        for cell in s3_cells(shape, &c) {
            let (o, q, e) = (&cell.old, &cell.r104, &cell.law);
            // H1: the solved vane at 0.3 unchoked.
            c.base.set_hp_vane(R30);
            let s4 = e.base.station("4");
            if vane_choked(&c.base.hp_cap.get(), c.base.gas(), e.base.pi_hpt, cell.tt4, s4.far, e.eta_hpt) {
                misses.push(format!("H1 {}: the vane is choked", cell.name));
            }
            c.base.set_hp_vane(HpVane::Pin);
            // H3 (second clause, every cell): rung 104's headline survives against the old model.
            let s = rel(e.n_hp_ratio, o.n_hp_ratio) / rel(e.n_lp_ratio, o.n_lp_ratio);
            s_min = s_min.min(s);
            s_max = s_max.max(s);
            if !(e.n_hp_ratio < o.n_hp_ratio && e.n_lp_ratio < o.n_lp_ratio && s >= 0.25) {
                misses.push(format!("H3 {}: against the old model share {s:.4}, N_H {:+.4} %, N_L {:+.4} %", cell.name,
                                    100.0 * rel(e.n_hp_ratio, o.n_hp_ratio), 100.0 * rel(e.n_lp_ratio, o.n_lp_ratio)));
            }
            // The boundary rule: where the law and rung 104's model barely differ, the first clause and H4 are not read.
            if rel(e.n_hp_ratio, q.n_hp_ratio).abs() < 1e-7 {
                excluded += 1;
                n += 1;
                continue;
            }
            // H3 (first clause): against rung 104's model, N_H higher, N_L lower, pi_HPC higher.
            if !(e.n_hp_ratio > q.n_hp_ratio && e.n_lp_ratio < q.n_lp_ratio && e.base.pi_hpc > q.base.pi_hpc) {
                misses.push(format!("H3 {}: against rung 104 N_H {:+.2e}, N_L {:+.2e}, pi_HPC {:+.2e}", cell.name,
                                    rel(e.n_hp_ratio, q.n_hp_ratio), rel(e.n_lp_ratio, q.n_lp_ratio),
                                    rel(e.base.pi_hpc, q.base.pi_hpc)));
            }
            // H4: dN_H against rung 104's model ordered 0.10 <= 0.3 < 0.5, 0.10's >= -1e-12.
            let at = |law: HpVane| s3_run(&c, &cell.fl, cell.tt4, TurbineLaw::Ellipse, law, false).expect("solves at 0.3");
            let d10 = rel(at(R10).n_hp_ratio, q.n_hp_ratio);
            let d30 = rel(e.n_hp_ratio, q.n_hp_ratio);
            let d50 = rel(at(R50).n_hp_ratio, q.n_hp_ratio);
            if !(d10 >= -1e-12 && d10 <= d30 && d30 < d50) {
                misses.push(format!("H4 {}: dN_H {d10:.3e} / {d30:.3e} / {d50:.3e}", cell.name));
            }
            n += 1;
        }
    }
    eprintln!("H1/H3/H4 on S3: {n} cells ({excluded} at the boundary, not read for H3's first clause and H4); rung 104's \
               share against the old model {s_min:.3}..{s_max:.3}");
    for m in &misses { eprintln!("  MISS {m}"); }
    assert_eq!(n, 33, "the pre-flight counted 33 S3 cells");
    assert!(misses.is_empty(), "{} misses", misses.len());
}
