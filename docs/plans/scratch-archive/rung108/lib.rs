//! Rung 108 probe: rung 106's march with rung 52's two-speed lag on the redline clip (a prototype — a copy of
//! `governor_lag::try_der` with `dg = (r - g)/lag.tau(r, g)`).
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{AsymmetricLag, FuelLimiters, FuelTransientCore};
use turbojet::governor::{self as gv, Leg};
use turbojet::governor_lag::{self as gl, Demand};
use turbojet::sandbox_controls::{ControlsSettings, ControlsSolver};
use turbojet::two_spool_transient::Instant2;

#[derive(Clone, Debug, PartialEq)]
pub struct P {
    pub s: f64, pub nu_lp: f64, pub nu_hp: f64, pub phi_lp: f64, pub phi_hp: f64, pub tt4: f64, pub thrust: f64,
    pub mf: f64, pub mf_dem: f64, pub g: f64, pub req: f64, pub leg: Leg,
}

pub struct D { pub da: f64, pub db: f64, pub dg: f64, pub p: P }

#[allow(clippy::too_many_arguments)]
pub fn der(ft: &FuelTransientCore, flight: &FlightCondition, dem: Demand<'_>, tt4_max: f64, lag: AsymmetricLag,
           lim: &FuelLimiters<'_>, metal: Option<&dyn Fn(&Instant2) -> f64>, a: f64, b: f64, g: f64, s: f64)
           -> Result<D, String> {
    ft.inner.inner.base.light_at(s);
    let e = |x: turbojet::gas::Abort| x.0;
    let (mf_dem, _clamp) = match dem {
        Demand::Governor(gov) => gv::try_governor_fuel(ft, flight, gov, a, b, s).map_err(e)?,
        Demand::Ramp(f) => (f(s), gv::Clamp::Root),
    };
    let i = ft.try_instant_fuel(flight, a, b, mf_dem).map_err(e)?;
    let required = if i.base.tt4 > tt4_max { mf_dem - ft.try_topping_fuel(flight, a, b, tt4_max, mf_dem).map_err(e)? } else { 0.0 };
    let (mut mf, mut leg) = (mf_dem - g, if g != 0.0 { Leg::Lag } else { Leg::None });
    let mut lowest = mf_dem;
    let floor = lim.floor();
    let mut caps: Vec<(f64, Leg)> = Vec::new();
    if let Some(acc) = lim.accel { caps.push((ft.try_sched_fuel(flight, a, b, mf_dem, acc).map_err(e)?, Leg::Accel)); }
    if let Some(f) = floor.as_ref() { caps.push((ft.try_surge_fuel(flight, a, b, mf_dem, f).map_err(e)?, Leg::Floor)); }
    if let Some(over) = metal {
        if over(&i.base) > 0.0 { caps.push((gv::try_metal_fuel(ft, flight, a, b, mf_dem, over).map_err(e)?, Leg::Metal)); }
    }
    for &(c, l) in &caps { lowest = lowest.min(c); if c < mf { mf = c; leg = l; } }
    let inst = match leg {
        Leg::None => i,
        _ => match (mf > 0.0).then(|| ft.try_instant_fuel(flight, a, b, mf)) {
            Some(Ok(x)) => x,
            _ => { mf = gv::wall(ft, flight, a, b, lowest, mf.max(0.0)); leg = Leg::MinFuel;
                   ft.try_instant_fuel(flight, a, b, mf).map_err(e)? }
        },
    };
    let c = &inst.base.close;
    Ok(D { da: inst.base.phi_lp_dot / ft.rho(), db: inst.base.phi_hp_dot, dg: (required - g) / lag.tau(required, g),
           p: P { s, nu_lp: a, nu_hp: b, phi_lp: c.phi_lp, phi_hp: c.phi_hp, tt4: inst.base.tt4, thrust: inst.base.sp_thrust,
                  mf, mf_dem, g, req: required, leg } })
}

#[allow(clippy::too_many_arguments)]
pub fn march_raw(ft: &FuelTransientCore, flight: &FlightCondition, dem: Demand<'_>, tt4_max: f64, lag: AsymmetricLag,
                 lim: &FuelLimiters<'_>, metal: Option<&dyn Fn(&Instant2) -> f64>, nu0: (f64, f64), s_end: f64, ds: f64)
                 -> Vec<P> {
    let mut pts = Vec::new();
    let ((mut a, mut b), mut g) = (nu0, 0.0f64);
    let mut s = 0.0f64;
    let n = (s_end / ds).round_ties_even() as i64;
    let d = |a, b, g, s| der(ft, flight, dem, tt4_max, lag, lim, metal, a, b, g, s);
    for _ in 0..=n {
        let Ok(k1) = d(a, b, g, s) else { break };
        pts.push(k1.p.clone());
        let step = || -> Result<(f64, f64, f64), String> {
            let k2 = d(a + ds / 2.0 * k1.da, b + ds / 2.0 * k1.db, g + ds / 2.0 * k1.dg, s + ds / 2.0)?;
            let k3 = d(a + ds / 2.0 * k2.da, b + ds / 2.0 * k2.db, g + ds / 2.0 * k2.dg, s + ds / 2.0)?;
            let k4 = d(a + ds * k3.da, b + ds * k3.db, g + ds * k3.dg, s + ds)?;
            Ok((a + ds / 6.0 * (k1.da + 2.0 * k2.da + 2.0 * k3.da + k4.da), b + ds / 6.0 * (k1.db + 2.0 * k2.db + 2.0 * k3.db + k4.db),
                g + ds / 6.0 * (k1.dg + 2.0 * k2.dg + 2.0 * k3.dg + k4.dg)))
        };
        let Ok((a1, b1, g1)) = step() else { break };
        (a, b, g) = (a1, b1, g1);
        s += ds;
    }
    pts
}

/// The prototype on a page solver: governed if it has a governor, else the open ramp; the clip at (att, rel).
pub fn march(sv: &ControlsSolver, att: f64, rel: f64, ds: f64) -> Vec<P> {
    let s: &ControlsSettings = &sv.settings;
    let over = |i: &Instant2| sv.metal_over(i);
    let metal: Option<&dyn Fn(&Instant2) -> f64> = if s.metal_leg() { Some(&over) } else { None };
    let ramp = s.schedule(sv.mf_lo, sv.mf_hi);
    let dem = match &sv.governor { Some(g) => Demand::Governor(g), None => Demand::Ramp(&ramp) };
    let metal = if sv.governor.is_some() { metal } else { None };
    march_raw(&sv.core.fuel, &sv.flight, dem, s.redline, AsymmetricLag::new(att, rel), &sv.lag_limiters(), metal,
              (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), ds)
}

/// Rung 106's own march (governed) or rung 47's page march (open), for the reduce check, as (s, phi_lp, phi_hp, tt4, mf).
pub fn reference(sv: &ControlsSolver) -> Vec<(f64, f64, f64, f64, f64)> {
    let pts = sv.march();
    pts.iter().map(|p| (p.s, p.phi_lp, p.phi_hp, p.tt4, p.mf)).collect()
}

pub fn same_as_reference(sv: &ControlsSolver, mine: &[P]) -> bool {
    let r = reference(sv);
    r.len() == mine.len() && r.iter().zip(mine).all(|(x, p)| x.0 == p.s && x.1 == p.phi_lp && x.2 == p.phi_hp && x.3 == p.tt4 && x.4 == p.mf)
}

/// Largest flow-coefficient gap between run x (step DS/kx) and y (DS/ky), on the DS grid.
pub fn gap(x: &[P], kx: usize, y: &[P], ky: usize) -> f64 {
    if x.is_empty() || y.is_empty() { return f64::NAN; }
    let n = ((x.len() - 1) / kx).min((y.len() - 1) / ky) + 1;
    (0..n).map(|i| (x[kx * i].phi_lp - y[ky * i].phi_lp).abs().max((x[kx * i].phi_hp - y[ky * i].phi_hp).abs())).fold(0.0, f64::max)
}

/// How many times the clip's target falls from above the clip to at/below it (the release edge engages).
pub fn releases(p: &[P]) -> usize {
    p.windows(2).filter(|w| w[0].req > w[0].g && w[1].req <= w[1].g).count()
}
pub fn attacks(p: &[P]) -> usize {
    p.windows(2).filter(|w| w[0].req <= w[0].g && w[1].req > w[1].g).count() + p.first().map_or(0, |x| (x.req > x.g) as usize)
}
pub fn minfuel(p: &[P]) -> usize { p.iter().filter(|x| x.leg == Leg::MinFuel).count() }
