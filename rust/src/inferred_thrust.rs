//! RUNG 96 — A GOVERNOR THAT INFERS THRUST: the throttle holds a READING — the engine pressure ratio or the LP shaft
//! speed — that the control turns into thrust through a steady table, as a real control does.
//!
//! Anchor (the device and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung96-anchor-inferred-thrust.md`. Spec: `docs/rung96-spec.md`.
//!
//! # The reading and the table
//!
//! Rung 95's governor read thrust itself. An engine in flight cannot: it reads a pressure ratio (`EPR`, turbine-exit
//! over inlet total pressure) or its fan speed (the LP shaft speed `N1`) and turns that into thrust through a SCHEDULE
//! — here a table of 13 steady points (rung 48's count) between the two ends, read linearly between rows. The throttle
//! still asks for rung 95's straight line in thrust; the table turns it into the reading's demand, and the reading at
//! each instant into the thrust the control BELIEVES (`F̂`).
//!
//! # What the pressure ratio sees, exactly
//!
//! Both nozzles choked on a perfect gas: the convergent nozzle's gross thrust is `A9·[(1+γ)·p9 − p0]` with `p9` a fixed
//! fraction of `pt5` — `Tt9` cancels. So a pressure-ratio control reads GROSS thrust exactly (anchor G2, to ~1e−15), and
//! gross thrust is affine in `EPR` (G3), so the table's linear rows carry it with no error. The whole gap between the
//! thrust it infers and the true thrust is the ram drag it cannot see: `F_true − F̂ = V0·(ṁ̂_face − ṁ_face)`, the
//! flight speed times the table's airflow at that reading less the air actually swallowed (gated, `tests/rung96.rs`).
//!
//! # Reduce
//!
//! With thrust read directly the governor is rung 95's, by dispatch; the table is built only for an inferred reading.

use crate::burner_hold::HoldRefusal;
use crate::engine::FlightCondition;
use crate::fuel_transient::FuelTransientCore;
use crate::gas::Abort;
use crate::governor::{thrust, TABLE_ROWS};
use crate::two_spool_transient::Instant2;
use crate::unchoked_formula::Formula;

/// The engine pressure ratio at an instant: the nozzle inlet's total pressure — every transient tail builds it as
/// `π_LPT·π_HPT·pt4` (anchor G5) — over the inlet's, `pt2`.
pub fn epr(i: &Instant2, pt2: f64) -> f64 { i.pi_lpt * i.pi_hpt * i.close.pt4 / pt2 }

/// The air the inlet swallows, kg/s of design airflow: the face flow when the model books one (a bleed, the
/// customer's air, cooling), else the core's.
pub fn face(i: &Instant2) -> f64 { i.close.mdot_face.unwrap_or(i.close.mdot_air) }

/// What a control reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    /// The engine pressure ratio.
    Epr,
    /// The LP shaft speed (`N1`).
    LpSpeed,
}

/// One steady point of the control's table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Row {
    /// Thrust, N per kg/s of design airflow, with the bleed booking ([`thrust`]).
    pub thrust: f64,
    pub epr: f64,
    pub nu_lp: f64,
    /// Fuel flow, kg/s of design airflow — the LP proportional law's steady fuel.
    pub fuel: f64,
    /// The air swallowed — only to split the pressure-ratio gap into its ram drag.
    pub face: f64,
}

impl Row {
    pub fn of(i: &Instant2, pt2: f64) -> Row {
        Row { thrust: thrust(i), epr: epr(i, pt2), nu_lp: i.nu_lp, fuel: i.close.f * i.close.mdot_air, face: face(i) }
    }
}

/// The control's table: steady points rising in thrust (and, anchor G4, in `EPR`, LP speed and fuel), the run's
/// inlet pressure and flight speed.
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    pub rows: Vec<Row>,
    pub pt2: f64,
    pub v0: f64,
    /// RUNG 100: the formula's readers, when the thrust is worked out at the reading rather than read off the rows —
    /// read through [`Table::try_inferred`] / [`Table::try_reading_at`].
    pub formula: Option<Formula>,
}

/// Linear in `x` between the rows, the END SEGMENTS carried on beyond (a reading outside the table — a leg cutting, a
/// slowdown undershooting — still gets the straight-line value, which for gross thrust is exact, G3).
pub fn lin(rows: &[Row], x: impl Fn(&Row) -> f64, y: impl Fn(&Row) -> f64, v: f64) -> f64 {
    let n = rows.len();
    let k = if v <= x(&rows[0]) { 0 } else { rows.windows(2).position(|w| v <= x(&w[1])).unwrap_or(n - 2) };
    let (a, b) = (&rows[k], &rows[k + 1]);
    y(a) + (y(b) - y(a)) * (v - x(a)) / (x(b) - x(a))
}

impl Table {
    /// The reading the table gives at thrust `f` — the reading's demand. A table with a formula is read through
    /// [`Table::try_reading_at`].
    pub fn reading_at(&self, r: Reading, f: f64) -> f64 {
        assert!(self.formula.is_none(), "a table with rung 100's formula is read through try_reading_at");
        match r {
            Reading::Epr => lin(&self.rows, |w| w.thrust, |w| w.epr, f),
            Reading::LpSpeed => lin(&self.rows, |w| w.thrust, |w| w.nu_lp, f),
        }
    }

    /// The thrust the control BELIEVES at reading `v` — `F̂`. A table with a formula is read through
    /// [`Table::try_inferred`].
    pub fn inferred(&self, r: Reading, v: f64) -> f64 {
        assert!(self.formula.is_none(), "a table with rung 100's formula is read through try_inferred");
        match r {
            Reading::Epr => lin(&self.rows, |w| w.epr, |w| w.thrust, v),
            Reading::LpSpeed => lin(&self.rows, |w| w.nu_lp, |w| w.thrust, v),
        }
    }

    /// [`Table::reading_at`], or RUNG 100's formula worked out at the reading (`ft` is the machine whose LP turbine
    /// the fan-speed formula re-solves).
    pub fn try_reading_at(&self, ft: &FuelTransientCore, r: Reading, f: f64) -> Result<f64, Abort> {
        match &self.formula {
            None => Ok(self.reading_at(r, f)),
            Some(m) => m.try_reading_at(ft, self, r, f),
        }
    }

    /// [`Table::inferred`], or RUNG 100's formula worked out at the reading.
    pub fn try_inferred(&self, ft: &FuelTransientCore, r: Reading, v: f64) -> Result<f64, Abort> {
        match &self.formula {
            None => Ok(self.inferred(r, v)),
            Some(m) => m.try_inferred(ft, self, r, v),
        }
    }

    /// The reading at an instant.
    pub fn read(&self, r: Reading, i: &Instant2) -> f64 {
        match r { Reading::Epr => epr(i, self.pt2), Reading::LpSpeed => i.nu_lp }
    }

    /// The LP proportional law's steady fuel at LP speed `nu`.
    pub fn fuel_at_speed(&self, nu: f64) -> f64 { lin(&self.rows, |w| w.nu_lp, |w| w.fuel, nu) }

    /// The air the table expects at pressure ratio `e` — the ram-drag half of the decomposition.
    pub fn face_at_epr(&self, e: f64) -> f64 { lin(&self.rows, |w| w.epr, |w| w.face, e) }
}

/// The reading in words.
pub fn reading_words(r: Reading) -> &'static str {
    match r { Reading::Epr => "pressure ratio", Reading::LpSpeed => "fan (LP shaft) speed" }
}

/// Plain words for a calibrated end that cannot land (rung 94's refusals, in the reading's units).
pub fn refusal_words(e: &HoldRefusal, r: Reading, want: f64) -> String {
    let (what, scale, unit) = match r { Reading::Epr => ("pressure ratio", 1.0, ""), Reading::LpSpeed => ("fan speed", 100.0, " % of design") };
    match e {
        HoldRefusal::NoSolve => "with the generator on the engine has no steady point anywhere in the range searched.".into(),
        HoldRefusal::BelowFloor { t_floor, q_floor } => format!(
            "the {what} asked for ({:.4}{unit}) is below the least this loaded engine allows: it needs at least \
             {t_floor:.0} K to carry the generator, and reads {:.4}{unit} there.", want * scale, q_floor * scale),
        HoldRefusal::AboveRange { t_top, q_top } => format!(
            "the {what} asked for ({:.4}{unit}) is more than this loaded engine reads even at {t_top:.0} K ({:.4}{unit}).",
            want * scale, q_top * scale),
        HoldRefusal::Gap { t, message } => format!(
            "the search met a burner temperature ({t:.0} K) where the engine has no steady point. The model says: {message}"),
    }
}

/// RUNG 96. The control's table on `ft`'s own equilibria between `start` and `end` (both on that machine), the ends
/// themselves as the end rows — so at each end the table's thrust IS that end's — the burner temperatures between
/// spaced evenly.
pub fn try_table(
    ft: &FuelTransientCore, flight: &FlightCondition, start: &Instant2, end: &Instant2,
) -> Result<Table, Abort> {
    let (_, pt2, v0) = ft.inner.inlet(flight);
    let n = TABLE_ROWS;
    let mut rows = Vec::with_capacity(n);
    for k in 0..n {
        rows.push(if k == 0 {
            Row::of(start, pt2)
        } else if k == n - 1 {
            Row::of(end, pt2)
        } else {
            let t = start.tt4 + (end.tt4 - start.tt4) * k as f64 / (n - 1) as f64;
            Row::of(&ft.inner.try_equilibrium(flight, t, None)?.0, pt2)
        });
    }
    rows.sort_by(|a, b| a.thrust.total_cmp(&b.thrust));
    for w in rows.windows(2) {
        if !(w[1].thrust > w[0].thrust && w[1].epr > w[0].epr && w[1].nu_lp > w[0].nu_lp && w[1].fuel > w[0].fuel) {
            return Err(Abort("rung-96: the control's table does not rise in thrust, pressure ratio, LP speed and fuel \
                              together, so a reading cannot be turned back into thrust".into()));
        }
    }
    Ok(Table { rows, pt2, v0, formula: None })
}
