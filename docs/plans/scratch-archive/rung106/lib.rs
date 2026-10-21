//! PROTOTYPE (rung 106 probe): rung 95's governor as the demand of rung 47's LAGGED redline clip. Built from public
//! pieces only. The clip amount `g` is a third state: `dg/ds = (required âˆ’ g)/Ï„`, `required = max(0, gov âˆ’ topping)`
//! (armed only where the governor's instant is over the redline â€” rung 46/47's guard), applied `= min(gov âˆ’ g, caps)`,
//! the other legs (schedule, floor, metal) instant and solved off the governor's fuel (rung 47's pattern). An applied
//! fuel below the leanest runnable one is CLAMPED there and marked (rung 95's lean rule, extended).
use turbojet::gas::Abort;
use turbojet::governor::{self as gv, Clamp};
use turbojet::sandbox_controls::ControlsSolver;

#[derive(Clone, Copy, Debug)]
pub struct Rec {
    pub s: f64,
    pub a: f64,
    pub b: f64,
    pub g: f64,
    pub clamp: Clamp,
    pub lean_applied: bool,
    /// 0 none, 1 redline-lag, 2 sched, 3 floor, 4 metal
    pub leg: u8,
    pub mf: f64,
    pub mf_gov: f64,
    pub tt4: f64,
}

pub struct Der {
    pub da: f64,
    pub db: f64,
    pub dg: f64,
    pub rec: Rec,
}

fn wall(sv: &ControlsSolver, a: f64, b: f64, mut ok: f64, mut bad: f64) -> f64 {
    let ft = &sv.core.fuel;
    for _ in 0..200 {
        if (bad - ok).abs() <= 1e-13 * ok.abs().max(bad.abs()) { break }
        let m = 0.5 * (ok + bad);
        if m > 0.0 && ft.try_instant_fuel(&sv.flight, a, b, m).is_ok() { ok = m } else { bad = m }
    }
    ok
}

/// `open`: the demand is the open-loop ramp (rung 47's own march — the prototype's check), not the governor.
pub static OPEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn der(sv: &ControlsSolver, t4max: f64, tau: f64, a: f64, b: f64, g: f64, s: f64) -> Result<Der, Abort> {
    let ft = &sv.core.fuel;
    let fl = &sv.flight;
    ft.inner.inner.base.light_at(s);
    let lim = sv.limiters();
    let (mf_gov, clamp) = if OPEN.load(std::sync::atomic::Ordering::Relaxed) {
        (sv.settings.schedule(sv.mf_lo, sv.mf_hi)(s), Clamp::Root)
    } else {
        gv::try_governor_fuel(ft, fl, sv.governor.as_ref().expect("governed"), a, b, s)?
    };
    let i = ft.try_instant_fuel(fl, a, b, mf_gov)?;
    let required = if i.base.tt4 > t4max { mf_gov - ft.try_topping_fuel(fl, a, b, t4max, mf_gov)? } else { 0.0 };
    let (mut mf, mut leg) = (mf_gov - g, if g != 0.0 { 1u8 } else { 0 });
    let mut lowest_ok = mf_gov;
    if let Some(acc) = lim.accel {
        let c = ft.try_sched_fuel(fl, a, b, mf_gov, acc)?;
        lowest_ok = lowest_ok.min(c);
        if c < mf { mf = c; leg = 2 }
    }
    if let Some(f) = lim.floor().as_ref() {
        let c = ft.try_surge_fuel(fl, a, b, mf_gov, f)?;
        lowest_ok = lowest_ok.min(c);
        if c < mf { mf = c; leg = 3 }
    }
    if sv.settings.metal_leg() {
        let over = |x: &turbojet::two_spool_transient::Instant2| sv.metal_over(x);
        if over(&i.base) > 0.0 {
            let c = gv::try_metal_fuel(ft, fl, a, b, mf_gov, &over)?;
            lowest_ok = lowest_ok.min(c);
            if c < mf { mf = c; leg = 4 }
        }
    }
    let mut lean_applied = false;
    let inst = if leg == 0 { i } else {
        match ft.try_instant_fuel(fl, a, b, mf) {
            Ok(x) if mf > 0.0 => x,
            _ => {
                // below the leanest runnable fuel: clamp there
                let w = wall(sv, a, b, lowest_ok, mf.min(lowest_ok * 0.999));
                lean_applied = true;
                mf = w;
                ft.try_instant_fuel(fl, a, b, mf)?
            }
        }
    };
    let rec = Rec { s, a, b, g, clamp, lean_applied, leg, mf, mf_gov, tt4: inst.base.tt4 };
    Ok(Der { da: inst.base.phi_lp_dot / ft.rho(), db: inst.base.phi_hp_dot, dg: (required - g) / tau, rec })
}

/// The lagged governed march; returns the records and the stop message (None: ran to the end).
pub fn march(sv: &ControlsSolver, t4max: f64, tau: f64, ds: f64) -> (Vec<Rec>, Option<String>) {
    let (mut a, mut b, mut g) = (sv.start.nu_lp, sv.start.nu_hp, 0.0f64);
    let mut s = 0.0f64;
    let n = (sv.settings.s_end() / ds).round_ties_even() as i64;
    let mut recs = Vec::new();
    for _ in 0..=n {
        let d = match der(sv, t4max, tau, a, b, g, s) { Ok(d) => d, Err(e) => return (recs, Some(format!("k1 s={s:.3}: {}", e.0))) };
        recs.push(d.rec);
        let st = (|| -> Result<_, Abort> {
            let d2 = der(sv, t4max, tau, a + ds / 2.0 * d.da, b + ds / 2.0 * d.db, g + ds / 2.0 * d.dg, s + ds / 2.0)?;
            let d3 = der(sv, t4max, tau, a + ds / 2.0 * d2.da, b + ds / 2.0 * d2.db, g + ds / 2.0 * d2.dg, s + ds / 2.0)?;
            let d4 = der(sv, t4max, tau, a + ds * d3.da, b + ds * d3.db, g + ds * d3.dg, s + ds)?;
            Ok((d2, d3, d4))
        })();
        let (d2, d3, d4) = match st { Ok(x) => x, Err(e) => return (recs, Some(format!("rk s={s:.3}: {}", e.0))) };
        a += ds / 6.0 * (d.da + 2.0 * d2.da + 2.0 * d3.da + d4.da);
        b += ds / 6.0 * (d.db + 2.0 * d2.db + 2.0 * d3.db + d4.db);
        g += ds / 6.0 * (d.dg + 2.0 * d2.dg + 2.0 * d3.dg + d4.dg);
        s += ds;
    }
    sv.core.fuel.inner.inner.base.ab_w.set(0.0);
    (recs, None)
}
