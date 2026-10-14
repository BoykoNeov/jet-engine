//! RUNG 86 — **FUEL + BLEED + STATOR on one plant: the CUBE.**
//!
//! `docs/rung86-spec.md`; the anchor, with every derivation and pre-registered bar, is
//! `docs/plans/rung86-anchor-three-levers.md` (§ refs below are to it).
//!
//! # What it adds — an INSTRUMENT, not plant code (§ 0 P1)
//!
//! Rung 63 held to two levers, *"since a third lever breaks the second difference's isolation"*.
//! The plant for three was already built — rung 62's machine carries the stator schedule and the
//! bleed schedule together, and rung 57's march takes a fuel limiter — so the rung is the reading:
//! three switches `S` (LP stator schedule), `B` (bleed schedule), `F` (ONE fuel limiter), eight
//! cells, each the LP incidence-margin minimum `M_i` (rung 58's currency — its wall is the metal,
//! one number in every cell). Their alternating sum is the THREE-WAY term
//!
//! ```text
//!     T = I_SB|F=1 − I_SB|F=0 = I_BF|S=1 − I_BF|S=0 = I_SF|B=1 − I_SF|B=0
//! ```
//!
//! — what exists only when all three are on. Its two-lever faces are earlier rungs' instruments,
//! and are gated equal to them bit for bit: `I_BF|S=0` is rung 63's `lever_composite`, `I_SF|B=0`
//! rung 58's `composite_credit`, `I_SB|F=0` rung 62's leg-free pair.
//!
//! # Every fuel limiter (the user's choice), as ONE [`Limiter`]
//!
//! The acceleration schedule (48) and its realistic release lag (52); the turbine limiter (46) and
//! its lag (47); the stall floor as a flow coefficient (49) and as an incidence (60). The two lags
//! are not [`StatorLeg`] fields, so the cube marches through [`ramp_march`] — rung 57's own march
//! body — with the whole limiter set, after [`reset_built_maps`] exactly as `stator_march_scoped`
//! does. With a plain limiter that march IS `stator_march`, bit for bit (`tests/rung86.rs`).
//!
//! # Disciplines (§ 1)
//!
//! Every cell a FRESH `at_lever` sibling with its arming ASSERTED (rung 63 § 0: an inherited
//! reader can hand back an identical sibling and report a perfect zero); ONE leg object, derived
//! once on the BARE sibling, for all four legged cells (rung 58); every cell's point count kept,
//! because `M_i` is a minimum over the run and any SUM compares only between equal-length runs.

use crate::bleed_transient::LeverArm;
use crate::engine::FlightCondition;
use crate::fuel_transient::{AccelSchedule, AsymmetricLag, Floor, FuelLimiters, FuelPoint, SurgeLimiter};
use crate::stator_transient::{
    ramp_march, reset_built_maps, CellRead, CreditProfile, IncidenceLimiter, Ramp, ScheduledStatorCore, StatorArm,
    StatorLeg,
};
use crate::two_spool::Spool;

/// The rows rung 48's derived schedule reads off the running line — rung 63's `leg_of`.
pub const ACCEL_ROWS: usize = 13;

/// ONE fuel limiter — the `F` switch of the cube.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Limiter {
    /// Rung 48's `Wf/pt3` schedule, `margin` above the steady line.
    Accel { margin: f64 },
    /// The same, behind rung 52's fast-attack / slow-release lag.
    AccelLag { margin: f64, lag: AsymmetricLag },
    /// Rung 46's turbine-temperature limiter at `tt4_max`.
    Topping { tt4_max: f64 },
    /// The same, behind rung 47's first-order lag `tau_gov`.
    ToppingLag { tt4_max: f64, tau_gov: f64 },
    /// Rung 60's incidence floor, at `sm` above the design stall line (in `φ`, at the design
    /// setting) — its coordinate IS the cube's currency.
    IncidenceFloor { sm: f64 },
    /// Rung 49's `φ` floor at `sm` above the design stall line.
    PhiFloor { sm: f64 },
}

impl Limiter {
    pub fn key(&self) -> &'static str {
        match self {
            Limiter::Accel { .. } => "accel",
            Limiter::AccelLag { .. } => "accel+lag",
            Limiter::Topping { .. } => "topping",
            Limiter::ToppingLag { .. } => "topping+lag",
            Limiter::IncidenceFloor { .. } => "incidence floor",
            Limiter::PhiFloor { .. } => "phi floor",
        }
    }
}

/// A [`Limiter`] made concrete on one machine: the leg object every legged cell shares.
#[derive(Clone, Debug)]
pub struct ArmedLimiter {
    pub limiter: Limiter,
    pub accel: Option<AccelSchedule>,
    pub floor: Option<Floor>,
}

impl ArmedLimiter {
    /// Derive the leg ONCE, on `bare` (the unarmed sibling) — rung 58's discipline.
    pub fn on(bare: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, spool: Spool,
              limiter: Limiter) -> Self {
        let cmap = bare.design_map(spool);
        let (accel, floor) = match limiter {
            Limiter::Accel { margin } | Limiter::AccelLag { margin, .. } => (
                Some(bare.fuel.accel_schedule(flight, ramp.tt4_lo, ramp.tt4_hi, margin, ACCEL_ROWS)),
                None,
            ),
            Limiter::Topping { .. } | Limiter::ToppingLag { .. } => (None, None),
            Limiter::IncidenceFloor { sm } => {
                (None, Some(Floor::Incidence(IncidenceLimiter::from_margin(&cmap, spool, sm))))
            }
            Limiter::PhiFloor { sm } => (None, Some(Floor::Phi(SurgeLimiter::from_margin(&cmap, spool, sm)))),
        };
        ArmedLimiter { limiter, accel, floor }
    }

    /// The instantaneous leg — what `stator_march` would take, and what `s_eng` reads.
    pub fn leg(&self) -> StatorLeg<'_> {
        let tt4_max = match self.limiter {
            Limiter::Topping { tt4_max } | Limiter::ToppingLag { tt4_max, .. } => Some(tt4_max),
            _ => None,
        };
        StatorLeg { accel: self.accel.as_ref(), surge: self.floor, tt4_max }
    }

    /// The whole limiter set the march takes — the leg, plus a lag if this limiter has one.
    pub fn limiters(&self, below_ceiling: bool) -> FuelLimiters<'_> {
        let (surge, incidence) = match self.floor {
            Some(Floor::Phi(s)) => (Some(s), None),
            Some(Floor::Incidence(i)) => (None, Some(i)),
            None => (None, None),
        };
        let leg = self.leg();
        FuelLimiters {
            tt4_max: leg.tt4_max,
            tau_gov: match self.limiter { Limiter::ToppingLag { tau_gov, .. } => Some(tau_gov), _ => None },
            accel: leg.accel,
            surge,
            incidence,
            lag: match self.limiter { Limiter::AccelLag { lag, .. } => Some(lag), _ => None },
            below_ceiling,
            ..Default::default()
        }
    }

    /// The set point in the cube's currency, for the floors: the incidence floor's `m_lim`, or the
    /// `φ` floor's `φ_lim`.
    pub fn set_point(&self) -> Option<f64> {
        match self.floor {
            Some(Floor::Incidence(i)) => Some(i.m_lim),
            Some(Floor::Phi(s)) => Some(s.phi_lim),
            None => None,
        }
    }
}

/// The march of one cell: rung 57's ramp with `lim` (or none), from the map reset, the ceiling
/// switch read off THIS machine's carrier as rung 57's march reads it.
pub fn march(m: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, lim: Option<&ArmedLimiter>)
    -> (Vec<FuelPoint>, (f64, f64)) {
    let below = m.fuel.inner.below_ceiling.get();
    reset_built_maps(&m.fuel.inner);
    let set = match lim {
        Some(l) => l.limiters(below),
        None => FuelLimiters { below_ceiling: below, ..Default::default() },
    };
    ramp_march(&m.fuel, flight, ramp, None, &set)
}

/// One cell of the cube, with the extras the bars read.
#[derive(Clone, Debug)]
pub struct Cell {
    pub read: CellRead,
    /// The cell's arming, asserted at construction: (stator scheduled, valve armed, leg on).
    pub arming: (bool, bool, bool),
    /// `φ` at the `M_i` grid minimum — C2 (a): a pinned minimum reads `φ_lim` here.
    pub phi_at_min: f64,
    /// The highest `Tt4` up to and including the LAST point of the `M_i` stencil (the grid
    /// minimum's later neighbour) — B1's reach test, read on a LEG-FREE run.
    pub tt4_to_stencil: f64,
    /// Total scheduled fuel over the run (trapezoid) — C3's denominator.
    pub fuel_scheduled: f64,
}

fn phi_of(p: &FuelPoint, spool: Spool) -> f64 {
    match spool {
        Spool::Lp => p.phi_lp,
        Spool::Hp => p.phi_hp,
    }
}

fn cell(m: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, spool: Spool,
        lim: Option<&ArmedLimiter>, arming: (bool, bool, bool)) -> Cell {
    assert!(m.arming().is_scheduled() == arming.0 && m.armed_bleed() == arming.1
                && lim.is_some() == arming.2,
            "rung-86 cell {arming:?}: the sibling reports stator={} valve={} leg={} -- a cube \
             cell that is not the machine it names makes the alternating sum isolate nothing \
             (rung 63 § 0).", m.arming().is_scheduled(), m.armed_bleed(), lim.is_some());
    let (traj, nu0) = march(m, flight, ramp, lim);
    if traj.is_empty() {
        // A floor above the START can refuse the very first point: no fuel holds it there, and
        // the march records nothing. That is a RESULT — the run does not exist — never a crash.
        let nan = f64::NAN;
        let read = CellRead {
            m_i: nan, m_i_grid: nan, m_phi: nan, s: nan, prof: Vec::new(), v: nan, s_grid: nan,
            min_phi: nan, nu0: nu0.0, nu_lp_end: nan, nu_hp_end: nan, tt4_peak: nan,
            fuel_removed: nan, s_eng: nan, npts: 0,
        };
        return Cell { read, arming, phi_at_min: nan, tt4_to_stencil: nan, fuel_scheduled: nan };
    }
    let free = StatorLeg::default();
    let leg = lim.map(|l| l.leg());
    let read = m.cell_of(flight, &traj, nu0, spool, leg.as_ref().unwrap_or(&free));
    let j = traj.iter().position(|p| p.s == read.s_grid)
        .expect("the grid minimum is a point of the march it was read on");
    let last = (j + 1).min(traj.len() - 1);
    let tt4_to_stencil = traj[..=last].iter().map(|p| p.tt4).fold(f64::NEG_INFINITY, f64::max);
    let mut fuel_scheduled = 0.0;
    for i in 1..traj.len() {
        fuel_scheduled += 0.5 * (traj[i].s - traj[i - 1].s) * (traj[i - 1].mf_sched + traj[i].mf_sched);
    }
    Cell { phi_at_min: phi_of(&traj[j], spool), tt4_to_stencil, fuel_scheduled, read, arming }
}

/// The eight cells and everything differenced from them. Cell index `4·S + 2·B + F`.
#[derive(Clone, Debug)]
pub struct Cube {
    pub limiter: Limiter,
    pub spool: Spool,
    pub r: f64,
    pub ds: f64,
    pub set_point: Option<f64>,
    /// `T_c` of the currency's wall (the design map's critical `tan β1`).
    pub t_c: f64,
    pub cells: Vec<Cell>,
    /// The pair interactions on each face of the third switch: `[off, on]`.
    pub i_sb: [f64; 2],
    pub i_bf: [f64; 2],
    pub i_sf: [f64; 2],
    /// THE THREE-WAY TERM, in its three spellings (equal to arithmetic precision).
    pub t: f64,
    pub t_bf: f64,
    pub t_sf: f64,
    /// Per lever cell `xy` (index `2·S + B`): does the leg move `M_i` or its time at all?
    pub reached: [bool; 4],
    /// Rung 58's predictor across the stator faces (§ 3 A3): each face's leg-free BLEED credit
    /// profile read at its legged-both vs leg-free-neither minimum, differenced across faces.
    pub predicted: f64,
}

impl Cube {
    pub fn at(&self, s: usize, b: usize, f: usize) -> &Cell {
        &self.cells[4 * s + 2 * b + f]
    }

    pub fn m(&self, s: usize, b: usize, f: usize) -> f64 {
        self.at(s, b, f).read.m_i
    }

    /// Every cell ran to the end of the march.
    pub fn full_length(&self) -> bool {
        let n = self.cells[0].read.npts;
        self.cells.iter().all(|c| c.read.npts == n)
    }

    /// § 3 C1's ledger: the legged face predicted from the four leg-free cells and the set point
    /// alone — `max(c_xy0, m_lim)` — and the `T` it implies. Incidence floor only.
    pub fn incidence_ledger(&self) -> Option<([f64; 4], f64)> {
        let m_lim = match self.limiter { Limiter::IncidenceFloor { .. } => self.set_point?, _ => return None };
        let mut led = [0.0; 4];
        for (k, (s, b)) in [(0, 0), (0, 1), (1, 0), (1, 1)].into_iter().enumerate() {
            led[k] = self.m(s, b, 0).max(m_lim);
        }
        // T = I_SB|F=1 − I_SB|F=0, with the legged face replaced by the ledger.
        let i_led = led[3] - led[1] - led[2] + led[0];
        Some((led, i_led - self.i_sb[0]))
    }

    /// Withheld fuel as a fraction of the cell's own scheduled fuel (§ 3 C3).
    pub fn withheld_fraction(&self, s: usize, b: usize) -> f64 {
        let c = self.at(s, b, 1);
        c.read.fuel_removed / c.fuel_scheduled
    }
}

/// THE RUNG (86). Build the cube on `m` (the BARE machine) for one limiter.
///
/// `stator` and `bleed` are the two levers as `at_lever` keywords; each cell is a fresh sibling.
#[allow(clippy::too_many_arguments)]
pub fn cube(
    m: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, spool: Spool, stator: &LeverArm,
    bleed: &LeverArm, limiter: Limiter,
) -> Cube {
    assert!(!m.arming().is_armed() && !m.armed_bleed(),
            "rung-86 builds every cell from the BARE machine: pass the unarmed one.");
    assert!(!stator.arms_valve() && bleed.arms_valve() && bleed.stator == StatorArm::default(),
            "rung-86: `stator` must carry no valve and `bleed` no stator -- each switch is ONE lever.");
    let sibs = [
        m.at_lever(&LeverArm::default()),
        m.at_lever(bleed),
        m.at_lever(stator),
        m.at_lever(&LeverArm::merged(stator, bleed)),
    ];
    let lim = ArmedLimiter::on(&sibs[0], flight, ramp, spool, limiter);
    let mut cells = Vec::with_capacity(8);
    for s in 0..2 {
        for b in 0..2 {
            let sib = &sibs[2 * s + b];
            for f in 0..2 {
                let l = if f == 1 { Some(&lim) } else { None };
                cells.push(cell(sib, flight, ramp, spool, l, (s == 1, b == 1, f == 1)));
            }
        }
    }
    let c = |s: usize, b: usize, f: usize| cells[4 * s + 2 * b + f].read.m_i;
    let i_sb = [0, 1].map(|f| c(1, 1, f) - c(0, 1, f) - c(1, 0, f) + c(0, 0, f));
    let i_bf = [0, 1].map(|s| c(s, 1, 1) - c(s, 0, 1) - c(s, 1, 0) + c(s, 0, 0));
    let i_sf = [0, 1].map(|b| c(1, b, 1) - c(0, b, 1) - c(1, b, 0) + c(0, b, 0));
    let mut reached = [false; 4];
    for (k, (s, b)) in [(0, 0), (0, 1), (1, 0), (1, 1)].into_iter().enumerate() {
        let (free, legged) = (&cells[4 * s + 2 * b].read, &cells[4 * s + 2 * b + 1].read);
        reached[k] = free.m_i != legged.m_i || free.s != legged.s;
    }
    let pred_face = |s: usize| {
        let (neither, lever) = (&cells[4 * s].read, &cells[4 * s + 2].read);
        let prof = CreditProfile::new(&neither.prof, &lever.prof);
        prof.at(cells[4 * s + 3].read.s) - prof.at(neither.s)
    };
    // A short march cannot be differenced on one grid (CreditProfile asserts); the predictor is
    // then NaN, and the cube says so through `full_length`.
    let same_len = cells.iter().all(|x| x.read.npts == cells[0].read.npts);
    let predicted = if same_len { pred_face(1) - pred_face(0) } else { f64::NAN };
    let t_c = sibs[0].design_map(spool).tan_beta1_crit();
    Cube {
        limiter,
        spool,
        r: ramp.r,
        ds: ramp.ds,
        set_point: lim.set_point(),
        t_c,
        t: i_sb[1] - i_sb[0],
        t_bf: i_bf[1] - i_bf[0],
        t_sf: i_sf[1] - i_sf[0],
        i_sb,
        i_bf,
        i_sf,
        reached,
        predicted,
        cells,
    }
}
