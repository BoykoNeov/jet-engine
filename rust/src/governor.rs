//! RUNG 95 — A GOVERNOR IN THE SLAM: the throttle holds thrust, HP speed or burner temperature DURING the move, the
//! fuel limiters cutting what it asks for.
//!
//! Anchor (the device and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung95-anchor-governor.md`. Spec: `docs/rung95-spec.md`.
//!
//! # The governor is the TOP of the min-select
//!
//! Rungs 43–52 march an OPEN-LOOP fuel ramp and let the limiters cut it. Here the ramp's place is taken by a governor:
//! the quantity rung 94's throttle holds follows a straight line from the start end's target to the final end's over
//! the ramp, and the governor's fuel — the DEMAND — is what every armed leg (rung 46's redline, 48's schedule, 49/60's
//! floor, and the new METAL leg) is bracketed from and cuts. A leg can only cut, so the hold could never be one: on an
//! acceleration short of fuel it would hold nothing (the anchor's correction to rung 94's seam sentence).
//!
//! # The laws — a root in FUEL at the instant's frozen shaft speeds
//!
//! * burner temperature and thrust: INSTANT inversion `Q(mf; ν) = Q_dem(s)` — fuel moves both at once, no constant;
//! * HP speed: a shaft cannot be set instantly, so a law with a constant (the user's toggle) — the COMMANDED RATE,
//!   `dν_H/ds = (ν_dem − ν_H)/τ_N`, or the PROPORTIONAL law `mf = W(ν_dem)·[1 + K·(ν_dem − ν_H)]` on a steady table
//!   `W` of the machine's own equilibria.
//!
//! The anchor's G1 measured thrust, `Tt4` and the HP acceleration rising strictly with fuel at frozen speeds (four
//! maps), so each inversion has one root. A demand beyond the richest runnable fuel takes that fuel, below the
//! leanest the leanest — CLAMPED, and recorded (the user's choice); the walls are the fuel closure's fuel-air walls
//! (G2). The governor holds no state, so a leg's release hands it authority straight back: nothing winds up.
//!
//! # Reduce
//!
//! With the governor off the Controls run is rung 43's march, by dispatch. Governing the burner temperature IS rung
//! 40's temperature-commanded march on the same machine — two code paths (`tests/rung95.rs`).

use crate::airflow_source::SensorLag;
use crate::engine::FlightCondition;
use crate::fuel_transient::{point, FuelInstant, FuelLimiters, FuelPoint, FuelTransientCore, PointExtra};
use crate::gas::Abort;
use crate::inferred_thrust::{epr, Reading, Table};
use crate::spool::{try_illinois, ILLINOIS_MAXIT};
use crate::two_spool_transient::Instant2;

/// What the governor holds, and its law.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Law {
    /// The burner temperature, by instant inversion.
    BurnerTemp,
    /// The thrust (with the bleed booking, [`thrust`]), by instant inversion.
    Thrust,
    /// The HP speed, by a commanded rate: the shaft is asked to close its gap in `tau` (HP spool time constants).
    SpeedRate { tau: f64 },
    /// The HP speed, by a proportional law on the steady table: `gain` is the fuel fraction added per unit HP speed
    /// short.
    SpeedProp { gain: f64 },
    /// RUNG 96: thrust INFERRED from the engine pressure ratio, by instant inversion on the reading's demand.
    Epr,
    /// RUNG 96: thrust inferred from the LP speed, by a commanded rate on the LP shaft (`tau` in HP spool time
    /// constants, as rung 95's).
    LpRate { tau: f64 },
    /// RUNG 96: thrust inferred from the LP speed, by the proportional law on the control's table.
    LpProp { gain: f64 },
}

impl Law {
    /// RUNG 96: what the law reads, when it infers thrust (`None` for rung 95's laws).
    pub fn reading(self) -> Option<Reading> {
        match self {
            Law::Epr => Some(Reading::Epr),
            Law::LpRate { .. } | Law::LpProp { .. } => Some(Reading::LpSpeed),
            _ => None,
        }
    }
}

/// Where the governor's fuel sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clamp {
    /// On its root (or, for the proportional law, inside the runnable range).
    Root,
    /// Asking for more than the richest runnable fuel gives — clamped there.
    Rich,
    /// Asking for less than the leanest gives — clamped there.
    Lean,
}

impl Clamp {
    pub fn key(self) -> &'static str {
        match self { Clamp::Root => "root", Clamp::Rich => "rich", Clamp::Lean => "lean" }
    }
}

/// Which leg cut the governor's fuel at a point (`None`: the applied fuel IS the governor's).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leg {
    None,
    Redline,
    Accel,
    Floor,
    Metal,
}

/// The governor of one run.
#[derive(Clone, Debug)]
pub struct Governor {
    pub law: Law,
    /// The held quantity's targets at the two ends (K, N, or HP speed); the demand runs straight between them.
    pub lo: f64,
    pub hi: f64,
    pub ramp: f64,
    /// The open-loop ramp's two fuel flows: the bracket search starts from the ramp's fuel at `s`.
    pub mf_lo: f64,
    pub mf_hi: f64,
    /// The proportional law's steady table, `(ν_H, fuel)` rising in `ν_H` (empty for the other laws).
    pub table: Vec<(f64, f64)>,
    /// RUNG 96: the control's table, when the law infers thrust from a reading (`lo`/`hi` are then thrusts).
    pub infer: Option<Table>,
    /// RUNG 101: the exhaust sensor's lag, when the table reads a lagged sensor — the march then carries its reading as
    /// a third state.
    pub lag: Option<SensorLag>,
}

/// The step factors the bracket search walks out by, up and down.
pub const STEP_UP: f64 = 1.25;
pub const STEP_DOWN: f64 = 0.8;
/// How many steps out before the search gives up.
pub const MAX_STEPS: usize = 200;
/// A wall's bisection, relative.
pub const WALL_TOL: f64 = 1e-13;
/// How many log-spaced fuels the fallback scan for a runnable start tries (two decades and the ends' span).
pub const SCAN_POINTS: usize = 80;
/// The proportional law's table rows — rung 48's count.
pub const TABLE_ROWS: usize = 13;

impl Governor {
    /// The straight line's weight at `s`: 0 before, 1 after the ramp.
    pub fn weight(&self, s: f64) -> f64 {
        if s <= 0.0 { 0.0 } else if s >= self.ramp { 1.0 } else { s / self.ramp }
    }

    /// The demanded held quantity at `s` (for the speed laws, the demanded HP speed; for rung 96's, the READING the
    /// table gives at the demanded thrust).
    pub fn demand(&self, s: f64) -> f64 {
        match (&self.infer, self.law.reading()) {
            (Some(t), Some(r)) => t.reading_at(r, self.thrust_demand(s)),
            _ => self.lo + (self.hi - self.lo) * self.weight(s),
        }
    }

    /// [`Governor::demand`] through RUNG 100's formula when the table carries one (`ft` the machine its fan-speed
    /// formula re-solves) — every call inside a run goes through here.
    pub fn try_demand(&self, ft: &FuelTransientCore, s: f64) -> Result<f64, Abort> {
        match (&self.infer, self.law.reading()) {
            (Some(t), Some(r)) => t.try_reading_at(ft, r, self.thrust_demand(s)),
            _ => Ok(self.demand(s)),
        }
    }

    /// RUNG 96: the thrust the throttle asks for at `s` — rung 95's straight line between the ends' targets.
    pub fn thrust_demand(&self, s: f64) -> f64 { self.lo + (self.hi - self.lo) * self.weight(s) }

    /// The open-loop ramp's fuel at `s` — where the bracket search starts.
    pub fn guess(&self, s: f64) -> f64 { self.mf_lo + (self.mf_hi - self.mf_lo) * self.weight(s) }

    /// The proportional law's steady fuel at HP speed `nu`, linear between rows, the end rows beyond.
    pub fn steady_fuel(&self, nu: f64) -> f64 {
        let t = &self.table;
        assert!(t.len() >= 2, "the proportional law needs its steady table");
        if nu <= t[0].0 {
            return t[0].1;
        }
        if nu >= t[t.len() - 1].0 {
            return t[t.len() - 1].1;
        }
        let k = t.windows(2).position(|w| nu <= w[1].0).expect("inside the table");
        let (a, b) = (t[k], t[k + 1]);
        a.1 + (b.1 - a.1) * (nu - a.0) / (b.0 - a.0)
    }
}

/// Thrust, N per kg/s of design airflow — with the bleed open, the honest inlet figure (rung 42's booking: the dumped
/// air pays full ram drag and returns nothing). The Controls view's reading.
pub fn thrust(i: &Instant2) -> f64 {
    match (i.close.bleed, i.sp_thrust_inlet, i.close.mdot_face) {
        (Some(_), Some(spi), Some(face)) => spi * face,
        _ => i.sp_thrust * i.close.mdot_air,
    }
}

/// The quantity an instant inversion solves for: `Tt4`, thrust, or the HP acceleration `dν_H/ds`.
pub fn held(law: Law, i: &Instant2) -> f64 {
    match law {
        Law::BurnerTemp => i.tt4,
        Law::Thrust => thrust(i),
        Law::SpeedRate { .. } | Law::SpeedProp { .. } => i.phi_hp_dot,
        Law::Epr | Law::LpRate { .. } | Law::LpProp { .. } =>
            unreachable!("rung 96's readings need the run's inlet and spool ratio: use `held_by`"),
    }
}

/// The quantity an instant inversion solves for, every law: rung 95's [`held`], and RUNG 96's pressure ratio (on the
/// control's table's inlet) and LP acceleration `dν_L/ds` (the march's own `Φ_L/ρ`).
pub fn held_by(gov: &Governor, ft: &FuelTransientCore, i: &Instant2) -> f64 {
    match gov.law {
        Law::Epr => epr(i, gov.infer.as_ref().expect("an inferred law carries its table").pt2),
        Law::LpRate { .. } | Law::LpProp { .. } => i.phi_lp_dot / ft.rho(),
        law => held(law, i),
    }
}

/// The proportional law's steady table on `ft`'s own equilibria between the two ends' burner temperatures, the ends
/// themselves as the end rows (so the law's fixed point at each end is exact).
pub fn try_steady_table(
    ft: &FuelTransientCore, flight: &FlightCondition, start: &Instant2, end: &Instant2,
) -> Result<Vec<(f64, f64)>, Abort> {
    let fuel = |i: &Instant2| i.close.f * i.close.mdot_air;
    let n = TABLE_ROWS;
    let mut rows = Vec::with_capacity(n);
    for k in 0..n {
        rows.push(if k == 0 {
            (start.nu_hp, fuel(start))
        } else if k == n - 1 {
            (end.nu_hp, fuel(end))
        } else {
            let t = start.tt4 + (end.tt4 - start.tt4) * k as f64 / (n - 1) as f64;
            let (i, _, _) = ft.inner.try_equilibrium(flight, t, None)?;
            (i.nu_hp, fuel(&i))
        });
    }
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    Ok(rows)
}

/// The largest (`up`) or smallest runnable fuel between `ok` (runs) and `bad` (does not), bisected.
fn wall(ft: &FuelTransientCore, flight: &FlightCondition, a: f64, b: f64, mut ok: f64, mut bad: f64) -> f64 {
    for _ in 0..200 {
        if (bad - ok).abs() <= WALL_TOL * ok.abs().max(bad.abs()) {
            break;
        }
        let m = 0.5 * (ok + bad);
        if ft.try_instant_fuel(flight, a, b, m).is_ok() { ok = m } else { bad = m }
    }
    ok
}

/// A runnable fuel near the ramp's at `s` — the ramp's own, else either end's.
fn runnable(ft: &FuelTransientCore, flight: &FlightCondition, gov: &Governor, a: f64, b: f64, s: f64)
            -> Result<f64, Abort> {
    let mut first = None;
    for x in [gov.guess(s), gov.mf_lo, gov.mf_hi] {
        match ft.try_instant_fuel(flight, a, b, x) {
            Ok(_) => return Ok(x),
            Err(e) => { first.get_or_insert(e); }
        }
    }
    // None of the three runs — on a deep slowdown the ramp's fuel is too lean for shafts still spinning fast and the
    // start's too rich (the crash map's 3 stops, `docs/rung95-spec.md`). Scan out from the ramp's fuel, nearest first, on
    // a log grid spanning a decade either side of the two ends. Reached only where the three fail, so every run that
    // found one is unchanged.
    let g = gov.guess(s);
    let (lo, hi) = (0.1 * gov.mf_lo.min(gov.mf_hi), 10.0 * gov.mf_lo.max(gov.mf_hi));
    let mut grid: Vec<f64> = (0..=SCAN_POINTS).map(|k| lo * (hi / lo).powf(k as f64 / SCAN_POINTS as f64)).collect();
    grid.sort_by(|x, y| (x / g).ln().abs().total_cmp(&(y / g).ln().abs()));
    if let Some(x) = grid.into_iter().find(|&x| ft.try_instant_fuel(flight, a, b, x).is_ok()) {
        return Ok(x);
    }
    Err(Abort(format!("rung-95: no fuel runs at this state, scanning two decades around the ramp ({})",
                      first.expect("three tries").0)))
}

/// The root of an increasing `r(mf)` from the runnable `x0`, or the wall it is clamped at.
fn root_or_clamp(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, b: f64, x0: f64,
    r: &dyn Fn(f64) -> Result<f64, Abort>,
) -> Result<(f64, Clamp), Abort> {
    let r0 = r(x0)?;
    if r0 == 0.0 {
        return Ok((x0, Clamp::Root));
    }
    let (k, clamp) = if r0 < 0.0 { (STEP_UP, Clamp::Rich) } else { (STEP_DOWN, Clamp::Lean) };
    let (mut x, mut rx) = (x0, r0);
    for _ in 0..MAX_STEPS {
        let y = x * k;
        let (y, ry) = if ft.try_instant_fuel(flight, a, b, y).is_ok() {
            (y, r(y)?)
        } else {
            let w = wall(ft, flight, a, b, x, y);
            let rw = r(w)?;
            if (rw < 0.0) == (r0 < 0.0) && rw != 0.0 {
                return Ok((w, clamp));
            }
            (w, rw)
        };
        if ry == 0.0 {
            return Ok((y, Clamp::Root));
        }
        if (ry < 0.0) != (rx < 0.0) {
            let (lo, hi, glo, ghi) = if x < y { (x, y, rx, ry) } else { (y, x, ry, rx) };
            return Ok((try_illinois(r, lo, hi, glo, ghi, FuelTransientCore::LEG_TOL, ILLINOIS_MAXIT)?, Clamp::Root));
        }
        x = y;
        rx = ry;
    }
    Err(Abort("rung-95: the governor's bracket search walked out of steps".into()))
}

/// RUNG 95. The governor's fuel at a state: its law's root (or the proportional law's fuel), clamped to the runnable
/// range.
pub fn try_governor_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, gov: &Governor, a: f64, b: f64, s: f64,
) -> Result<(f64, Clamp), Abort> {
    try_governor_fuel_at(ft, flight, gov, a, b, s, None)
}

/// [`try_governor_fuel`] with the exhaust sensor's lagged reading `tm` (RUNG 101; `None`: unlagged, or no sensor). A
/// table that reads the sensor believes a thrust that depends on the instant — its exhaust temperature, its fuel, its
/// valve — so the demand cannot be worked out from `s` alone: each law's root is taken on the belief AT the trial
/// fuel (anchor § 1; G6 measured each residual rising with fuel, one root). Every other table is rung 95/96's, by
/// dispatch.
pub fn try_governor_fuel_at(
    ft: &FuelTransientCore, flight: &FlightCondition, gov: &Governor, a: f64, b: f64, s: f64, tm: Option<f64>,
) -> Result<(f64, Clamp), Abort> {
    if let Some(t) = gov.infer.as_ref().filter(|t| t.sensed()) {
        let x0 = runnable(ft, flight, gov, a, b, s)?;
        let f_dem = gov.thrust_demand(s);
        let at = |mf: f64| ft.try_instant_fuel(flight, a, b, mf).map(|i| i.base);
        return match gov.law {
            Law::Epr => {
                let r = |mf: f64| at(mf).and_then(|i| Ok(t.try_belief(ft, Reading::Epr, &i, tm)? - f_dem));
                root_or_clamp(ft, flight, a, b, x0, &r)
            }
            Law::LpRate { tau } => {
                let r = |mf: f64| at(mf).and_then(|i| {
                    let nd = t.try_demand_at(ft, Reading::LpSpeed, f_dem, &i, tm)?;
                    Ok(i.phi_lp_dot / ft.rho() - (nd - a) / tau)
                });
                root_or_clamp(ft, flight, a, b, x0, &r)
            }
            // The proportional law's fuel is its own fixed point here: the demanded fan speed falls with fuel (more
            // fuel, more thrust believed at every row), so `mf − W(ν_dem)·[1 + K·(ν_dem − ν_L)]` rises strictly.
            Law::LpProp { gain } => {
                let r = |mf: f64| at(mf).and_then(|i| {
                    let nd = t.try_demand_at(ft, Reading::LpSpeed, f_dem, &i, tm)?;
                    Ok(mf - t.fuel_at_speed(nd) * (1.0 + gain * (nd - a)))
                });
                root_or_clamp(ft, flight, a, b, x0, &r)
            }
            law => unreachable!("only an inferred law reads the sensor: {law:?}"),
        };
    }
    let x0 = runnable(ft, flight, gov, a, b, s)?;
    // The proportional laws: rung 95's on the HP speed and its steady table, rung 96's on the LP speed and the
    // control's table.
    let prop = match gov.law {
        Law::SpeedProp { gain } => Some((gain, b, None)),
        Law::LpProp { gain } => Some((gain, a, gov.infer.as_ref())),
        _ => None,
    };
    if let Some((gain, nu, table)) = prop {
        let nd = gov.try_demand(ft, s)?;
        let w = match table { None => gov.steady_fuel(nd), Some(t) => t.fuel_at_speed(nd) };
        let mf = w * (1.0 + gain * (nd - nu));
        if mf > 0.0 && ft.try_instant_fuel(flight, a, b, mf).is_ok() {
            return Ok((mf, Clamp::Root));
        }
        return Ok(if mf > x0 {
            // the first failing fuel above x0 on the way to mf
            let mut x = x0;
            loop {
                let y = (x * STEP_UP).min(mf);
                if ft.try_instant_fuel(flight, a, b, y).is_err() {
                    break (wall(ft, flight, a, b, x, y), Clamp::Rich);
                }
                x = y;
            }
        } else {
            let mut x = x0;
            loop {
                let y = (x * STEP_DOWN).max(mf.max(0.0));
                if y <= 0.0 || ft.try_instant_fuel(flight, a, b, y).is_err() {
                    break (wall(ft, flight, a, b, x, y), Clamp::Lean);
                }
                x = y;
            }
        });
    }
    let target = match gov.law {
        Law::SpeedRate { tau } => (gov.try_demand(ft, s)? - b) / tau,
        Law::LpRate { tau } => (gov.try_demand(ft, s)? - a) / tau,
        _ => gov.try_demand(ft, s)?,
    };
    let r = |mf: f64| ft.try_instant_fuel(flight, a, b, mf).map(|i| held_by(gov, ft, &i.base) - target);
    root_or_clamp(ft, flight, a, b, x0, &r)
}

/// RUNG 95. The metal leg's set point: the fuel below `mf_over` (whose metal is over its limit) at which the hotter
/// metal reaches it, at frozen speeds. `over` reads the metal's excess (> 0 over the limit) on an instant.
pub fn try_metal_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, b: f64, mf_over: f64,
    over: &dyn Fn(&Instant2) -> f64,
) -> Result<f64, Abort> {
    let r = |mf: f64| ft.try_instant_fuel(flight, a, b, mf).map(|i| over(&i.base));
    match root_or_clamp(ft, flight, a, b, mf_over, &r)? {
        (x, Clamp::Root) => Ok(x),
        (_, _) => Err(Abort(format!(
            "rung-95: the turbine metal is over its limit even at the leanest fuel that runs at nu=({a:.4},{b:.4}) — \
             the metal cap leaves nothing to burn"))),
    }
}

/// Which call inside one derivative evaluation failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GovCall {
    /// The governor's own solve.
    Governor,
    /// The engine at the governor's fuel.
    AtDemand,
    Redline,
    Accel,
    Floor,
    Metal,
    /// The engine at the applied (cut) fuel.
    AtApplied,
}

/// A failed derivative evaluation: the model's message, the call, the fuel handed to it.
#[derive(Clone, Debug, PartialEq)]
pub struct GovFailure {
    pub message: String,
    pub call: GovCall,
    pub mf: f64,
}

/// One derivative evaluation.
#[derive(Clone, Debug)]
pub struct GovDer {
    pub da: f64,
    pub db: f64,
    /// The governor's fuel (the demand the legs cut) and where it sits.
    pub mf_gov: f64,
    pub clamp: Clamp,
    /// The applied fuel and the leg that set it.
    pub mf: f64,
    pub leg: Leg,
    pub inst: FuelInstant,
}

/// RUNG 95. One derivative evaluation of the governed march: the governor's fuel, the instant there, each armed leg
/// solved off it (the redline only when the instant is over it, the metal leg likewise), the minimum, the instant
/// re-made there. The first of equal caps wins, in the order redline, schedule, floor, metal.
pub fn try_der(
    ft: &FuelTransientCore, flight: &FlightCondition, gov: &Governor, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, a: f64, b: f64, s: f64,
) -> Result<GovDer, GovFailure> {
    try_der_at(ft, flight, gov, lim, metal, a, b, s, None)
}

/// [`try_der`] with the exhaust sensor's lagged reading `tm` (RUNG 101).
#[allow(clippy::too_many_arguments)]
pub fn try_der_at(
    ft: &FuelTransientCore, flight: &FlightCondition, gov: &Governor, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, a: f64, b: f64, s: f64, tm: Option<f64>,
) -> Result<GovDer, GovFailure> {
    assert!(lim.freeze.is_none() && lim.tau_gov.is_none() && lim.lag.is_none() && lim.s_off.is_none()
                && lim.tau_rel.is_none(),
            "rung-95 governs rung 43's PLAIN min-select: no freeze, no lagged governor, no release lag or forced release");
    let fail = |call: GovCall, mf: f64| move |e: Abort| GovFailure { message: e.0, call, mf };
    let (mf_gov, clamp) = try_governor_fuel_at(ft, flight, gov, a, b, s, tm).map_err(fail(GovCall::Governor, f64::NAN))?;
    let i = ft.try_instant_fuel(flight, a, b, mf_gov).map_err(fail(GovCall::AtDemand, mf_gov))?;
    let floor = lim.floor();
    let mut caps: Vec<(f64, Leg)> = Vec::new();
    if let Some(t4) = lim.tt4_max {
        if i.base.tt4 > t4 {
            caps.push((ft.try_topping_fuel(flight, a, b, t4, mf_gov).map_err(fail(GovCall::Redline, mf_gov))?, Leg::Redline));
        }
    }
    if let Some(acc) = lim.accel {
        caps.push((ft.try_sched_fuel(flight, a, b, mf_gov, acc).map_err(fail(GovCall::Accel, mf_gov))?, Leg::Accel));
    }
    if let Some(f) = floor.as_ref() {
        caps.push((ft.try_surge_fuel(flight, a, b, mf_gov, f).map_err(fail(GovCall::Floor, mf_gov))?, Leg::Floor));
    }
    if let Some(over) = metal {
        if over(&i.base) > 0.0 {
            caps.push((try_metal_fuel(ft, flight, a, b, mf_gov, over).map_err(fail(GovCall::Metal, mf_gov))?, Leg::Metal));
        }
    }
    let (mut mf, mut leg) = (mf_gov, Leg::None);
    for &(c, l) in &caps {
        if c < mf {
            mf = c;
            leg = l;
        }
    }
    let inst = if leg == Leg::None { i } else { ft.try_instant_fuel(flight, a, b, mf).map_err(fail(GovCall::AtApplied, mf))? };
    Ok(GovDer { da: inst.base.phi_lp_dot / ft.rho(), db: inst.base.phi_hp_dot, mf_gov, clamp, mf, leg, inst })
}

/// RUNG 95. The rest of one RK4 step from `(a, b)` at `s`, given its opening slope `k1` — the march's own stages
/// (`k2`/`k3` at the half step, `k4` at the full one) and arithmetic.
#[allow(clippy::too_many_arguments)]
pub fn try_rk_step(
    ft: &FuelTransientCore, flight: &FlightCondition, gov: &Governor, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, a: f64, b: f64, s: f64, ds: f64, k1: (f64, f64),
) -> Result<(f64, f64), GovFailure> {
    let d = |a: f64, b: f64, s: f64| try_der(ft, flight, gov, lim, metal, a, b, s).map(|d| (d.da, d.db));
    let (k1a, k1b) = k1;
    let (k2a, k2b) = d(a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, s + ds / 2.0)?;
    let (k3a, k3b) = d(a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, s + ds / 2.0)?;
    let (k4a, k4b) = d(a + ds * k3a, b + ds * k3b, s + ds)?;
    Ok((a + ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a), b + ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b)))
}

/// RUNG 101. The rest of one RK4 step of the three-state march — the shafts and the lagged sensor's reading `m`
/// (`dm/ds = (Tt5 − m)/τ` at each stage's APPLIED instant) — from its opening slope `k1`, the march's own stages and
/// arithmetic.
#[allow(clippy::too_many_arguments)]
pub fn try_rk_step_sensed(
    ft: &FuelTransientCore, flight: &FlightCondition, gov: &Governor, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, a: f64, b: f64, m: f64, s: f64, ds: f64, k1: (f64, f64, f64),
) -> Result<(f64, f64, f64), GovFailure> {
    let tau = gov.lag.expect("the three-state march is the lagged sensor's").tau;
    let d = |a: f64, b: f64, m: f64, s: f64| try_der_at(ft, flight, gov, lim, metal, a, b, s, Some(m))
        .map(|d| (d.da, d.db, (d.inst.base.tt5 - m) / tau));
    let (k1a, k1b, k1m) = k1;
    let (k2a, k2b, k2m) = d(a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, m + ds / 2.0 * k1m, s + ds / 2.0)?;
    let (k3a, k3b, k3m) = d(a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, m + ds / 2.0 * k2m, s + ds / 2.0)?;
    let (k4a, k4b, k4m) = d(a + ds * k3a, b + ds * k3b, m + ds * k3m, s + ds)?;
    Ok((a + ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a), b + ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b),
        m + ds / 6.0 * (k1m + 2.0 * k2m + 2.0 * k3m + k4m)))
}

/// What the governed march records at a point beyond the fuel march's fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GovMark {
    pub clamp: Clamp,
    pub leg: Leg,
}

/// RUNG 95. The governed march — rung 43's plain march with the governor in each derivative evaluation (the point's
/// `mf_sched` is the governor's fuel, `mf` the applied). The coordinate is accumulated (`s += ds`) and the step count
/// is `round_ties_even(s_end/ds)`, as every fuel march. A failed evaluation ends the run (a RESULT; the Controls view
/// re-runs it for the reason).
#[allow(clippy::too_many_arguments)]
pub fn march(
    ft: &FuelTransientCore, flight: &FlightCondition, gov: &Governor, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, nu0: (f64, f64), s_end: f64, ds: f64,
) -> (Vec<FuelPoint>, Vec<GovMark>) {
    if let Some(lag) = gov.lag {
        return march_sensed(ft, flight, gov, lim, metal, nu0, lag, s_end, ds);
    }
    let (mut pts, mut marks) = (Vec::new(), Vec::new());
    let (mut a, mut b) = nu0;
    let mut s = 0.0f64;
    let n_steps = (s_end / ds).round_ties_even() as i64;
    for _ in 0..=n_steps {
        let Ok(d) = try_der(ft, flight, gov, lim, metal, a, b, s) else { break };
        pts.push(point(s, a, b, &d.inst, d.mf, d.mf_gov, PointExtra::None));
        marks.push(GovMark { clamp: d.clamp, leg: d.leg });
        let Ok((a1, b1)) = try_rk_step(ft, flight, gov, lim, metal, a, b, s, ds, (d.da, d.db)) else { break };
        a = a1;
        b = b1;
        s += ds;
    }
    (pts, marks)
}

/// RUNG 101. The governed march with the lagged exhaust sensor: [`march`]'s loop on three states, the sensor's
/// reading started at the steady start's `Tt5` and recorded at every point ([`PointExtra::Sensor`]).
#[allow(clippy::too_many_arguments)]
fn march_sensed(
    ft: &FuelTransientCore, flight: &FlightCondition, gov: &Governor, lim: &FuelLimiters<'_>,
    metal: Option<&dyn Fn(&Instant2) -> f64>, nu0: (f64, f64), lag: SensorLag, s_end: f64, ds: f64,
) -> (Vec<FuelPoint>, Vec<GovMark>) {
    let (mut pts, mut marks) = (Vec::new(), Vec::new());
    let ((mut a, mut b), mut m) = (nu0, lag.t0);
    let mut s = 0.0f64;
    let n_steps = (s_end / ds).round_ties_even() as i64;
    for _ in 0..=n_steps {
        let Ok(d) = try_der_at(ft, flight, gov, lim, metal, a, b, s, Some(m)) else { break };
        pts.push(point(s, a, b, &d.inst, d.mf, d.mf_gov, PointExtra::Sensor { tm: m }));
        marks.push(GovMark { clamp: d.clamp, leg: d.leg });
        let k1 = (d.da, d.db, (d.inst.base.tt5 - m) / lag.tau);
        let Ok((a1, b1, m1)) = try_rk_step_sensed(ft, flight, gov, lim, metal, a, b, m, s, ds, k1) else { break };
        (a, b, m) = (a1, b1, m1);
        s += ds;
    }
    (pts, marks)
}
