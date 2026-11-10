//! PROTOTYPE (rung 107 probe): rung 95's governor as the demand of rung 52's asymmetric (fast-attack / slow-release)
//! lag on the acceleration schedule and the stall floor. Built from public pieces only, in `integrate_fuel_asym`'s
//! order: applied = G − g (the open path's 1e-9 floor); the INSTANT redline on top reading the clipped fuel; on the
//! governed path rung 95's metal leg the same way and the lean wall held; the instant there; `required` off the DEMAND
//! through `try_leg_below` (the solvable-ceiling fallback); `dg = (req − g)/lag.tau(req, g)`.
use std::cell::OnceCell;
use turbojet::fuel_transient::AsymmetricLag;
use turbojet::gas::Abort;
use turbojet::governor::{self as gv, Clamp};
use turbojet::sandbox_controls::ControlsSolver;

#[derive(Clone, Copy, Debug)]
pub struct Rec {
    pub s: f64,
    pub a: f64,
    pub b: f64,
    pub g: f64,
    pub req: f64,
    pub clamp: Clamp,
    pub lean_applied: bool,
    /// 0 none, 1 lag, 2 redline, 3 metal
    pub leg: u8,
    pub mf: f64,
    pub mf_dem: f64,
    /// any evaluation up to and including this point's step had req > 0 (rung 95 would have cut)
    pub engaged: bool,
    pub phi_lp: f64,
    pub phi_hp: f64,
}

pub struct Der {
    pub da: f64,
    pub db: f64,
    pub dg: f64,
    pub rec: Rec,
}

pub fn der(sv: &ControlsSolver, lag: AsymmetricLag, open: bool, a: f64, b: f64, g: f64, s: f64) -> Result<Der, Abort> {
    let ft = &sv.core.fuel;
    let fl = &sv.flight;
    ft.inner.inner.base.light_at(s);
    let lim = sv.limiters();
    let floor = lim.floor();
    let (mf_dem, clamp) = if open {
        (sv.settings.schedule(sv.mf_lo, sv.mf_hi)(s), Clamp::Root)
    } else {
        gv::try_governor_fuel(ft, fl, sv.governor.as_ref().expect("governed"), a, b, s)?
    };
    let mut mf = mf_dem - g;
    let mut leg = if g != 0.0 { 1u8 } else { 0 };
    let mut lean_applied = false;
    if open {
        if 1e-9 > mf {
            mf = 1e-9;
        }
    } else if ft.try_instant_fuel(fl, a, b, mf).is_err() && mf < mf_dem {
        ft.try_instant_fuel(fl, a, b, mf_dem)?;
        mf = gv::wall(ft, fl, a, b, mf_dem, mf.max(0.0));
        lean_applied = true;
    }
    if let Some(t4) = lim.tt4_max {
        if ft.try_instant_fuel(fl, a, b, mf)?.base.tt4 > t4 {
            let c = ft.try_topping_fuel(fl, a, b, t4, mf)?;
            if c < mf {
                mf = c;
                leg = 2;
            }
        }
    }
    if !open && sv.settings.metal_leg() {
        let over = |x: &turbojet::two_spool_transient::Instant2| sv.metal_over(x);
        if over(&ft.try_instant_fuel(fl, a, b, mf)?.base) > 0.0 {
            let c = gv::try_metal_fuel(ft, fl, a, b, mf, &over)?;
            if c < mf {
                mf = c;
                leg = 3;
            }
        }
    }
    let i = ft.try_instant_fuel(fl, a, b, mf)?;
    let top = OnceCell::new();
    let mut undecided = false;
    let mut caps: Vec<f64> = Vec::new();
    if let Some(acc) = lim.accel {
        let solve = |x: f64| ft.try_sched_fuel(fl, a, b, x, acc);
        match ft.try_leg_below(fl, a, b, mf_dem, &top, &solve)? {
            Some(c) => caps.push(c),
            None => undecided = true,
        }
    }
    if let Some(f) = floor.as_ref() {
        let solve = |x: f64| ft.try_surge_fuel(fl, a, b, x, f);
        match ft.try_leg_below(fl, a, b, mf_dem, &top, &solve)? {
            Some(c) => caps.push(c),
            None => undecided = true,
        }
    }
    if undecided && caps.is_empty() {
        return Err(ft.try_instant_fuel(fl, a, b, mf_dem).err().expect("undecided ⇒ the demand fails"));
    }
    let req = if caps.is_empty() {
        0.0
    } else {
        let mut m = caps[0];
        for &c in &caps[1..] {
            if c < m {
                m = c;
            }
        }
        0.0f64.max(mf_dem - m)
    };
    let dg = (req - g) / lag.tau(req, g);
    let rec = Rec { s, a, b, g, req, clamp, lean_applied, leg, mf, mf_dem, engaged: req > 0.0,
                    phi_lp: i.base.close.phi_lp, phi_hp: i.base.close.phi_hp };
    Ok(Der { da: i.base.phi_lp_dot / ft.rho(), db: i.base.phi_hp_dot, dg, rec })
}

/// The march; returns the records and the stop message (None: ran to the end).
pub fn march(sv: &ControlsSolver, lag: AsymmetricLag, open: bool, ds: f64) -> (Vec<Rec>, Option<String>) {
    let (mut a, mut b, mut g) = (sv.start.nu_lp, sv.start.nu_hp, 0.0f64);
    let mut s = 0.0f64;
    let n = (sv.settings.s_end() / ds).round_ties_even() as i64;
    let mut recs = Vec::new();
    let mut eng = false;
    for _ in 0..=n {
        let d = match der(sv, lag, open, a, b, g, s) {
            Ok(d) => d,
            Err(e) => return (recs, Some(format!("k1 s={s:.3}: {}", e.0))),
        };
        let st = (|| -> Result<_, Abort> {
            let d2 = der(sv, lag, open, a + ds / 2.0 * d.da, b + ds / 2.0 * d.db, g + ds / 2.0 * d.dg, s + ds / 2.0)?;
            let d3 = der(sv, lag, open, a + ds / 2.0 * d2.da, b + ds / 2.0 * d2.db, g + ds / 2.0 * d2.dg, s + ds / 2.0)?;
            let d4 = der(sv, lag, open, a + ds * d3.da, b + ds * d3.db, g + ds * d3.dg, s + ds)?;
            Ok((d2, d3, d4))
        })();
        let mut rec = d.rec;
        eng |= rec.engaged;
        rec.engaged = eng;
        recs.push(rec);
        let (d2, d3, d4) = match st {
            Ok(x) => x,
            Err(e) => return (recs, Some(format!("rk s={s:.3}: {}", e.0))),
        };
        eng |= d2.rec.engaged || d3.rec.engaged || d4.rec.engaged;
        a += ds / 6.0 * (d.da + 2.0 * d2.da + 2.0 * d3.da + d4.da);
        b += ds / 6.0 * (d.db + 2.0 * d2.db + 2.0 * d3.db + d4.db);
        g += ds / 6.0 * (d.dg + 2.0 * d2.dg + 2.0 * d3.dg + d4.dg);
        s += ds;
    }
    sv.core.fuel.inner.inner.base.ab_w.set(0.0);
    (recs, None)
}

/// Sign changes of `req − g` over recorded points (exact zeros skipped): (attack→release, release→attack).
pub fn crossings(recs: &[Rec]) -> (usize, usize) {
    let (mut rel, mut reatt) = (0, 0);
    let mut last = 0i8;
    for r in recs {
        let d = r.req - r.g;
        let sg = if d > 0.0 { 1 } else if d < 0.0 { -1 } else { 0 };
        if sg == 0 {
            continue;
        }
        if last == 1 && sg == -1 {
            rel += 1;
        }
        if last == -1 && sg == 1 {
            reatt += 1;
        }
        last = sg;
    }
    (rel, reatt)
}
