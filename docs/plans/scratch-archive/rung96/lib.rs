//! Rung 96 pre-freeze probe — a PROTOTYPE inferred-thrust governor on the Controls engine, public pieces only.
//! Prints REACHABILITY only (monotonicity, clamps, which leg bound, identities) — never a quantity a claim is about.
use turbojet::fuel_transient::FuelInstant;
use turbojet::gas::Abort;
use turbojet::sandbox_controls::{ControlsSolver, DS};
use turbojet::two_spool_transient::Instant2;

pub fn inst(sv: &ControlsSolver, a: f64, b: f64, mf: f64) -> Result<FuelInstant, Abort> {
    sv.core.fuel.try_instant_fuel(&sv.flight, a, b, mf)
}

pub fn pt2(sv: &ControlsSolver) -> f64 { sv.core.fuel.inner.inlet(&sv.flight).1 }
pub fn v0(sv: &ControlsSolver) -> f64 { sv.core.fuel.inner.inlet(&sv.flight).2 }

/// Turbine-exit total pressure over inlet total pressure (pt5 = pi_lpt*pi_hpt*pt4, the tail's own s5).
pub fn epr(sv: &ControlsSolver, i: &Instant2) -> f64 { i.pi_lpt * i.pi_hpt * i.close.pt4 / pt2(sv) }

/// Gross thrust, same units as ControlsSolver::thrust (no bleed): net + ram drag.
pub fn face(i: &Instant2) -> f64 { i.close.mdot_face.unwrap_or(i.close.mdot_air) }
pub fn gross(sv: &ControlsSolver, i: &Instant2) -> f64 { ControlsSolver::thrust(i) + face(i) * v0(sv) }

fn edge(sv: &ControlsSolver, a: f64, b: f64, mf0: f64, k: f64) -> f64 {
    let (mut ok, mut bad) = (mf0, None);
    for _ in 0..400 {
        let w = ok * k;
        if inst(sv, a, b, w).is_ok() { ok = w } else { bad = Some(w); break }
    }
    let Some(mut bd) = bad else { return ok };
    for _ in 0..80 {
        if (bd - ok).abs() <= 1e-13 * bd { break }
        let m = 0.5 * (ok + bd);
        if inst(sv, a, b, m).is_ok() { ok = m } else { bd = m }
    }
    ok
}

pub fn range(sv: &ControlsSolver, a: f64, b: f64, guess: f64) -> Option<(f64, f64)> {
    let mut g = None;
    for x in [guess, sv.mf_lo, sv.mf_hi, 0.5 * sv.mf_lo, 2.0 * sv.mf_hi] {
        if inst(sv, a, b, x).is_ok() { g = Some(x); break }
    }
    let g = g?;
    Some((edge(sv, a, b, g, 0.95), edge(sv, a, b, g, 1.05)))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gov { Thrust, Epr, N1Rate(f64), N1Prop(f64) }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clamp { Root, Rich, Lean }

/// The steady table: (thrust, epr, nu_L, mf) rows over [start.tt4, end.tt4], ends exact, sorted by thrust.
pub fn table(sv: &ControlsSolver, n: usize) -> Vec<[f64; 6]> {
    let (t0, t1) = (sv.start.tt4, sv.end.tt4);
    let mut rows: Vec<[f64; 6]> = (0..n).map(|k| {
        let t = t0 + (t1 - t0) * k as f64 / (n - 1) as f64;
        let i = if k == 0 { sv.start.clone() } else if k == n - 1 { sv.end.clone() }
                else { sv.core.fuel.inner.try_equilibrium(&sv.flight, t, None).unwrap().0 };
        [ControlsSolver::thrust(&i), epr(sv, &i), i.nu_lp, i.close.f * i.close.mdot_air, gross(sv, &i), face(&i)]
    }).collect();
    rows.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
    rows
}

/// Linear interpolation of column `y` against column `x` (x rising), end rows beyond.
pub fn interp(rows: &[[f64; 6]], x: usize, y: usize, v: f64) -> f64 {
    let mut r: Vec<(f64, f64)> = rows.iter().map(|r| (r[x], r[y])).collect();
    r.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    if v <= r[0].0 { return r[0].1 }
    if v >= r[r.len() - 1].0 { return r[r.len() - 1].1 }
    let k = r.windows(2).position(|w| v <= w[1].0).unwrap();
    let (a, b) = (r[k], r[k + 1]);
    a.1 + (b.1 - a.1) * (v - a.0) / (b.0 - a.0)
}

pub struct Ctx<'a> { pub sv: &'a ControlsSolver, pub rows: Vec<[f64; 6]>, pub f_lo: f64, pub f_hi: f64 }

impl<'a> Ctx<'a> {
    pub fn new(sv: &'a ControlsSolver) -> Self {
        Ctx { sv, rows: table(sv, 13), f_lo: ControlsSolver::thrust(&sv.start), f_hi: ControlsSolver::thrust(&sv.end) }
    }
    pub fn w(&self, s: f64) -> f64 {
        let r = self.sv.settings.ramp;
        if s <= 0.0 { 0.0 } else if s >= r { 1.0 } else { s / r }
    }
    /// The thrust the throttle asks for at s (rung 95's straight line).
    pub fn f_dem(&self, s: f64) -> f64 { self.f_lo + (self.f_hi - self.f_lo) * self.w(s) }
    pub fn epr_dem(&self, s: f64) -> f64 { interp(&self.rows, 0, 1, self.f_dem(s)) }
    pub fn n1_dem(&self, s: f64) -> f64 { interp(&self.rows, 0, 2, self.f_dem(s)) }
}

pub fn gov_fuel(c: &Ctx, g: Gov, a: f64, b: f64, s: f64) -> Result<(f64, Clamp), Abort> {
    let sv = c.sv;
    let guess = sv.settings.schedule(sv.mf_lo, sv.mf_hi)(s);
    let (lo, hi) = range(sv, a, b, guess).ok_or_else(|| Abort("no solvable fuel at this state".into()))?;
    if let Gov::N1Prop(k) = g {
        let nd = c.n1_dem(s);
        let mf = interp(&c.rows, 2, 3, nd) * (1.0 + k * (nd - a));
        return Ok(if mf > hi { (hi, Clamp::Rich) } else if mf < lo { (lo, Clamp::Lean) } else { (mf, Clamp::Root) });
    }
    let rho = sv.core.fuel.rho();
    let (d, q): (f64, Box<dyn Fn(&Instant2) -> f64>) = match g {
        Gov::Thrust => (c.f_dem(s), Box::new(|i: &Instant2| ControlsSolver::thrust(i))),
        Gov::Epr => (c.epr_dem(s), Box::new(move |i: &Instant2| epr(sv, i))),
        Gov::N1Rate(tau) => ((c.n1_dem(s) - a) / tau, Box::new(move |i: &Instant2| i.phi_lp_dot / rho)),
        Gov::N1Prop(_) => unreachable!(),
    };
    let r = |mf: f64| inst(sv, a, b, mf).map(|i| q(&i.base) - d);
    let (gl, gh) = (r(lo)?, r(hi)?);
    if gh < 0.0 { return Ok((hi, Clamp::Rich)) }
    if gl > 0.0 { return Ok((lo, Clamp::Lean)) }
    let (mut x0, mut x1) = (lo, hi);
    for _ in 0..200 {
        if x1 - x0 <= 1e-14 * x1 { break }
        let m = 0.5 * (x0 + x1);
        if r(m)? < 0.0 { x0 = m } else { x1 = m }
    }
    Ok((0.5 * (x0 + x1), Clamp::Root))
}

/// Which leg bound: 0 none, 1 redline, 2 accel, 3 floor.
pub fn der(c: &Ctx, g: Gov, a: f64, b: f64, s: f64) -> Result<(f64, f64, Clamp, usize, FuelInstant), Abort> {
    let sv = c.sv;
    let ft = &sv.core.fuel;
    let fl = &sv.flight;
    let lim = sv.limiters();
    let floor = lim.floor();
    let (mf, clamp) = gov_fuel(c, g, a, b, s)?;
    let i = inst(sv, a, b, mf)?;
    let mut caps: Vec<(f64, usize)> = Vec::new();
    if let Some(t4) = lim.tt4_max {
        if i.base.tt4 > t4 { caps.push((ft.try_topping_fuel(fl, a, b, t4, mf)?, 1)) }
    }
    if let Some(acc) = lim.accel { caps.push((ft.try_sched_fuel(fl, a, b, mf, acc)?, 2)) }
    if let Some(f) = floor.as_ref() { caps.push((ft.try_surge_fuel(fl, a, b, mf, f)?, 3)) }
    caps.retain(|c| c.0 < mf);
    let (mfa, leg) = caps.iter().copied().fold((mf, 0), |m, c| if c.0 < m.0 { c } else { m });
    let i = if leg == 0 { i } else { inst(sv, a, b, mfa)? };
    Ok((i.base.phi_lp_dot / ft.rho(), i.base.phi_hp_dot, clamp, leg, i))
}

pub struct Rec { pub s: f64, pub a: f64, pub b: f64, pub clamp: Clamp, pub leg: usize, pub i: FuelInstant }

pub fn march(c: &Ctx, g: Gov, ds: f64) -> (Vec<Rec>, Option<String>) {
    let sv = c.sv;
    let (mut a, mut b) = (sv.start.nu_lp, sv.start.nu_hp);
    let mut s = 0.0f64;
    let n = (sv.settings.s_end() / ds).round_ties_even() as i64;
    let mut out = Vec::new();
    for _ in 0..=n {
        let (k1a, k1b, clamp, leg, i) = match der(c, g, a, b, s) { Ok(x) => x, Err(e) => return (out, Some(e.0)) };
        out.push(Rec { s, a, b, clamp, leg, i });
        let st = (|| -> Result<_, Abort> {
            let (k2a, k2b, ..) = der(c, g, a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, s + ds / 2.0)?;
            let (k3a, k3b, ..) = der(c, g, a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, s + ds / 2.0)?;
            let (k4a, k4b, ..) = der(c, g, a + ds * k3a, b + ds * k3b, s + ds)?;
            Ok((k2a, k2b, k3a, k3b, k4a, k4b))
        })();
        let (k2a, k2b, k3a, k3b, k4a, k4b) = match st { Ok(x) => x, Err(e) => return (out, Some(e.0)) };
        a += ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a);
        b += ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b);
        s += ds;
    }
    (out, None)
}

pub fn ds() -> f64 { DS }
