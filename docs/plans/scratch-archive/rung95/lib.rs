//! Rung 95 pre-freeze probe — a PROTOTYPE governor on the Controls engine, built from public pieces only.
//! Prints REACHABILITY only (clamps, which leg bound, monotonicity) — never a quantity a claim is about.
use turbojet::fuel_transient::FuelInstant;
use turbojet::gas::Abort;
use turbojet::sandbox_controls::{ControlsSolver, DS};

pub fn inst(sv: &ControlsSolver, a: f64, b: f64, mf: f64) -> Result<FuelInstant, Abort> {
    sv.core.fuel.try_instant_fuel(&sv.flight, a, b, mf)
}

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

/// The solvable fuel range at frozen speeds, from a solvable guess.
pub fn range(sv: &ControlsSolver, a: f64, b: f64, guess: f64) -> Option<(f64, f64)> {
    let mut g = None;
    for x in [guess, sv.mf_lo, sv.mf_hi, 0.5 * sv.mf_lo, 2.0 * sv.mf_hi] {
        if inst(sv, a, b, x).is_ok() { g = Some(x); break }
    }
    let g = g?;
    Some((edge(sv, a, b, g, 0.95), edge(sv, a, b, g, 1.05)))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gov { Tt4, Thrust, SpeedRate(f64) }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clamp { Root, Rich, Lean }

pub fn q(g: Gov, i: &FuelInstant) -> f64 {
    match g {
        Gov::Tt4 => i.base.tt4,
        Gov::Thrust => ControlsSolver::thrust(&i.base),
        Gov::SpeedRate(_) => i.base.phi_hp_dot,
    }
}

pub fn demand(sv: &ControlsSolver, g: Gov, s: f64, b: f64) -> f64 {
    let r = sv.settings.ramp;
    let w = if s <= 0.0 { 0.0 } else if s >= r { 1.0 } else { s / r };
    let lin = |x0: f64, x1: f64| x0 + (x1 - x0) * w;
    match g {
        Gov::Tt4 => lin(sv.start.tt4, sv.end.tt4),
        Gov::Thrust => lin(ControlsSolver::thrust(&sv.start), ControlsSolver::thrust(&sv.end)),
        Gov::SpeedRate(tau) => (lin(sv.start.nu_hp, sv.end.nu_hp) - b) / tau,
    }
}

/// The governor's fuel at a state: the root of q(mf) = demand inside the solvable range, or a clamp.
pub fn gov_fuel(sv: &ControlsSolver, g: Gov, a: f64, b: f64, s: f64) -> Result<(f64, Clamp), Abort> {
    let guess = sv.settings.schedule(sv.mf_lo, sv.mf_hi)(s);
    let (lo, hi) = range(sv, a, b, guess).ok_or_else(|| Abort("no solvable fuel at this state".into()))?;
    let d = demand(sv, g, s, b);
    let r = |mf: f64| inst(sv, a, b, mf).map(|i| q(g, &i) - d);
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
pub fn der(sv: &ControlsSolver, g: Gov, a: f64, b: f64, s: f64) -> Result<(f64, f64, Clamp, usize, FuelInstant), Abort> {
    let ft = &sv.core.fuel;
    let fl = &sv.flight;
    let lim = sv.limiters();
    let floor = lim.floor();
    let (mf, clamp) = gov_fuel(sv, g, a, b, s)?;
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

pub fn march(sv: &ControlsSolver, g: Gov, ds: f64) -> (Vec<Rec>, Option<String>) {
    let (mut a, mut b) = (sv.start.nu_lp, sv.start.nu_hp);
    let mut s = 0.0f64;
    let n = (sv.settings.s_end() / ds).round_ties_even() as i64;
    let mut out = Vec::new();
    for _ in 0..=n {
        let (k1a, k1b, clamp, leg, i) = match der(sv, g, a, b, s) { Ok(x) => x, Err(e) => return (out, Some(e.0)) };
        out.push(Rec { s, a, b, clamp, leg, i });
        let st = (|| -> Result<_, Abort> {
            let (k2a, k2b, ..) = der(sv, g, a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, s + ds / 2.0)?;
            let (k3a, k3b, ..) = der(sv, g, a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, s + ds / 2.0)?;
            let (k4a, k4b, ..) = der(sv, g, a + ds * k3a, b + ds * k3b, s + ds)?;
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

/// The proportional law's steady table on the march's machine: (nu_H, mf) rows over [start.tt4, end.tt4], ends exact.
pub fn table(sv: &ControlsSolver, n: usize) -> Vec<(f64, f64)> {
    let (t0, t1) = (sv.start.tt4, sv.end.tt4);
    let mut rows: Vec<(f64, f64)> = (0..n).map(|k| {
        let t = t0 + (t1 - t0) * k as f64 / (n - 1) as f64;
        let i = if k == 0 { sv.start.clone() } else if k == n - 1 { sv.end.clone() }
                else { sv.core.fuel.inner.try_equilibrium(&sv.flight, t, None).unwrap().0 };
        (i.nu_hp, i.close.f * i.close.mdot_air)
    }).collect();
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    rows
}
pub fn interp(rows: &[(f64, f64)], x: f64) -> f64 {
    if x <= rows[0].0 { return rows[0].1 }
    if x >= rows[rows.len() - 1].0 { return rows[rows.len() - 1].1 }
    let k = rows.windows(2).position(|w| x <= w[1].0).unwrap();
    let (a, b) = (rows[k], rows[k + 1]);
    a.1 + (b.1 - a.1) * (x - a.0) / (b.0 - a.0)
}
/// P-law fuel at a state, clamped to the solvable range.
pub fn p_fuel(sv: &ControlsSolver, rows: &[(f64, f64)], k: f64, a: f64, b: f64, s: f64) -> Result<(f64, Clamp), Abort> {
    let r = sv.settings.ramp;
    let w = if s <= 0.0 { 0.0 } else if s >= r { 1.0 } else { s / r };
    let nd = sv.start.nu_hp + (sv.end.nu_hp - sv.start.nu_hp) * w;
    let ff = interp(rows, nd);
    let mf = ff * (1.0 + k * (nd - b));
    let (lo, hi) = range(sv, a, b, ff).ok_or_else(|| Abort("no solvable fuel".into()))?;
    Ok(if mf > hi { (hi, Clamp::Rich) } else if mf < lo { (lo, Clamp::Lean) } else { (mf, Clamp::Root) })
}
pub fn p_march(sv: &ControlsSolver, rows: &[(f64, f64)], k: f64, ds: f64) -> (Vec<(f64, Clamp)>, Option<String>) {
    let ft = &sv.core.fuel;
    let der = |a: f64, b: f64, s: f64| -> Result<(f64, f64, Clamp), Abort> {
        let (mf, c) = p_fuel(sv, rows, k, a, b, s)?;
        let i = inst(sv, a, b, mf)?;
        Ok((i.base.phi_lp_dot / ft.rho(), i.base.phi_hp_dot, c))
    };
    let (mut a, mut b) = (sv.start.nu_lp, sv.start.nu_hp);
    let mut s = 0.0f64;
    let n = (sv.settings.s_end() / ds).round_ties_even() as i64;
    let mut out = Vec::new();
    for _ in 0..=n {
        let (k1a, k1b, c) = match der(a, b, s) { Ok(x) => x, Err(e) => return (out, Some(e.0)) };
        out.push((s, c));
        let st = (|| -> Result<_, Abort> {
            let (k2a, k2b, _) = der(a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, s + ds / 2.0)?;
            let (k3a, k3b, _) = der(a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, s + ds / 2.0)?;
            let (k4a, k4b, _) = der(a + ds * k3a, b + ds * k3b, s + ds)?;
            Ok((k2a, k2b, k3a, k3b, k4a, k4b))
        })();
        let Ok((k2a, k2b, k3a, k3b, k4a, k4b)) = st else { return (out, Some("rk stage".into())) };
        a += ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a);
        b += ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b);
        s += ds;
    }
    (out, None)
}
