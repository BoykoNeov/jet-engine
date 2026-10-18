//! RUNG 98 — THE CORRECTED TABLES: the control's three tables made ONCE at the design flight, and flown elsewhere
//! through the air's pressure (`δ`) and temperature (`θ`) — as a real schedule is.
//!
//! Anchor (the device and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung98-anchor-corrected-tables.md`. Spec: `docs/rung98-spec.md`.
//!
//! # The three tables and their corrections
//!
//! Rungs 48, 95 and 96 built their tables at the flight flown, so a flight correction had nothing to do. Here each
//! table is built at the DESIGN flight, on rows at the run's own throttles translated to that flight
//! (`Tt4·Tt2_d/Tt2`, the corrected-equivalent throttle), and CARRIED to the flight flown with the inlet-total ratios
//! `δ = pt2/pt2_d`, `θ = Tt2/Tt2_d` (each switchable; off, the factor is 1):
//!
//! * rung 96's thrust table: thrust `×δ`, `EPR` as it is, LP speed `×√θ`, fuel `×δ√θ`, face airflow `×δ/√θ`;
//! * rung 95's steady HP-speed table: speed `×√θ`, fuel `×δ√θ`;
//! * rung 48's acceleration schedule: `Wf/pt3 ×√θ` — its abscissa is ALREADY the model's corrected HP speed and its
//!   ordinate already divides out pressure, so `δ` has nothing to correct there (a ratio unit carries its own).
//!
//! Carrying the table is correcting each reading the other way; every downstream read is unchanged.
//!
//! # What corrects exactly, and what cannot
//!
//! The pressure correction is EXACT on this model (anchor G1, ≤ 3.6e−11: nothing in it knows absolute pressure). The
//! temperature correction is not (G2: the fuel's fixed heating value). And at fixed inlet totals a Mach change moves
//! only the thrust (G3), which no correction of the readings can see.
//!
//! # Reduce
//!
//! Tables made at THIS flight (`TableAt::Flight`), or flown at the design flight, are rungs 48/95/96's own tables —
//! the Controls view dispatches to their code, bit for bit.

use crate::engine::FlightCondition;
use crate::fuel_transient::{AccelSchedule, FuelTransientCore};
use crate::governor::TABLE_ROWS;
use crate::inferred_thrust::{epr, face, Row, Table};
use crate::two_spool_transient::Instant2;

/// Where the control's tables are made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableAt {
    /// Once, at the design flight, and carried to the flight flown — rung 98.
    Design,
    /// At the flight flown — rungs 48/95/96.
    Flight,
}

impl TableAt {
    pub const ALL: [TableAt; 2] = [TableAt::Design, TableAt::Flight];
    pub fn key(self) -> &'static str {
        match self { TableAt::Design => "design", TableAt::Flight => "flight" }
    }
}

/// The carry from the design flight to the flight flown.
#[derive(Clone, Copy, Debug)]
pub struct Carry {
    pub design: FlightCondition,
    /// The inlet-total ratios, `θ = Tt2/Tt2_d`, `δ = pt2/pt2_d`.
    pub theta: f64,
    pub delta: f64,
    /// The factors APPLIED: `δ` and `√θ` with their switch on, else 1.
    pub d: f64,
    pub t: f64,
    /// The row translation `Tt2_d/Tt2` — applied whatever the switches (where the rows lie is the table's coverage).
    pub k: f64,
    /// Both flights' inlet total pressure and flight speed.
    pub pt2: f64,
    pub v0: f64,
    pub pt2_d: f64,
    pub v0_d: f64,
}

impl Carry {
    /// Both flights through the core's own `inlet()`, so each ratio is exactly 1 when they are the same flight.
    pub fn new(ft: &FuelTransientCore, design: &FlightCondition, flight: &FlightCondition, pressure: bool,
               temperature: bool) -> Carry {
        let (tt2_d, pt2_d, v0_d) = ft.inner.inlet(design);
        let (tt2, pt2, v0) = ft.inner.inlet(flight);
        let (theta, delta) = (tt2 / tt2_d, pt2 / pt2_d);
        Carry { design: *design, theta, delta, d: if pressure { delta } else { 1.0 },
                t: if temperature { theta.sqrt() } else { 1.0 }, k: tt2_d / tt2, pt2, v0, pt2_d, v0_d }
    }

    /// The design-flight burner temperature of row `j` of `n` between the run's throttles `lo` and `hi`.
    pub fn row_tt4(&self, lo: f64, hi: f64, j: usize, n: usize) -> f64 {
        (lo + (hi - lo) * j as f64 / (n - 1) as f64) * self.k
    }

    /// Rung 96's table carried to the flight flown (read there with the flight's own `pt2` and `V0`).
    pub fn thrust_table(&self, design: &Table) -> Table {
        let (d, t) = (self.d, self.t);
        Table {
            rows: design.rows.iter().map(|w| Row {
                thrust: w.thrust * d, epr: w.epr, nu_lp: w.nu_lp * t, fuel: w.fuel * d * t, face: w.face * d / t,
            }).collect(),
            pt2: self.pt2,
            v0: self.v0,
        }
    }

    /// Rung 95's steady `(ν_H, fuel)` table carried to the flight flown.
    pub fn steady_table(&self, design: &[(f64, f64)]) -> Vec<(f64, f64)> {
        design.iter().map(|&(nu, w)| (nu * self.t, w * self.d * self.t)).collect()
    }

    /// Rung 48's schedule carried to the flight flown: `Wf/pt3 × √θ` (no `δ`, by construction).
    pub fn accel(&self, design: &AccelSchedule) -> AccelSchedule {
        AccelSchedule { margin: design.margin, n_h: design.n_h.clone(), kappa: design.kappa.iter().map(|k| k * self.t).collect() }
    }

    /// The NOZZLE part of a pressure-ratio control's gap at a choked instant (anchor § 1, G6): what is left of
    /// `F_true − F̂` after the ram-drag part `δ·V0_d·ṁ_d(e) − V0·ṁ_face`, with `ṁ_d(e)` the DESIGN table's airflow at the
    /// reading. A constant through a run, `A9·(δ·p0_d − p0)`, whatever the reading. Meaningful only with `δ` applied.
    pub fn nozzle_part(&self, design: &Table, i: &Instant2, f_true: f64, f_hat: f64) -> f64 {
        f_true - f_hat - self.ram_part(design, i)
    }

    /// The RAM-DRAG part of the same gap: `δ·V0_d·ṁ_d(e) − V0·ṁ_face`.
    pub fn ram_part(&self, design: &Table, i: &Instant2) -> f64 {
        let e = epr(i, self.pt2);
        self.delta * self.v0_d * design.face_at_epr(e) - self.v0 * face(i)
    }
}

/// A design-flight row with no steady point: its burner temperature there and the model's words.
#[derive(Clone, Debug, PartialEq)]
pub struct RowFailure {
    pub tt4: f64,
    pub message: String,
}

/// The design-flight equilibria of `n` rows between the run's throttles.
fn try_rows(ft: &FuelTransientCore, c: &Carry, lo: f64, hi: f64, n: usize) -> Result<Vec<Instant2>, RowFailure> {
    (0..n).map(|j| {
        let t = c.row_tt4(lo, hi, j, n);
        ft.inner.try_equilibrium(&c.design, t, None).map(|x| x.0).map_err(|e| RowFailure { tt4: t, message: e.0 })
    }).collect()
}

/// RUNG 98. Rung 96's thrust table at the DESIGN flight (13 rows, `lo`..`hi` translated), checked to rise as rung
/// 96's is. Not yet carried: [`Carry::thrust_table`].
pub fn try_design_table(ft: &FuelTransientCore, c: &Carry, lo: f64, hi: f64) -> Result<Table, RowFailure> {
    let mut rows: Vec<Row> = try_rows(ft, c, lo, hi, TABLE_ROWS)?.iter().map(|i| Row::of(i, c.pt2_d)).collect();
    rows.sort_by(|a, b| a.thrust.total_cmp(&b.thrust));
    for w in rows.windows(2) {
        if !(w[1].thrust > w[0].thrust && w[1].epr > w[0].epr && w[1].nu_lp > w[0].nu_lp && w[1].fuel > w[0].fuel) {
            return Err(RowFailure { tt4: f64::NAN, message: "rung-98: the control's design-flight table does not rise in \
                thrust, pressure ratio, LP speed and fuel together, so a reading cannot be turned back into thrust".into() });
        }
    }
    Ok(Table { rows, pt2: c.pt2_d, v0: c.v0_d })
}

/// RUNG 98. Rung 95's steady `(ν_H, fuel)` table at the DESIGN flight, rising in `ν_H`. Not yet carried.
pub fn try_design_steady_table(ft: &FuelTransientCore, c: &Carry, lo: f64, hi: f64) -> Result<Vec<(f64, f64)>, RowFailure> {
    let mut rows: Vec<(f64, f64)> = try_rows(ft, c, lo, hi, TABLE_ROWS)?.iter()
        .map(|i| (i.nu_hp, i.close.f * i.close.mdot_air)).collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    Ok(rows)
}

/// RUNG 98. Rung 48's schedule at the DESIGN flight — the Controls view's `accel_table` arithmetic on the translated
/// rows. Not yet carried.
pub fn try_design_accel(ft: &FuelTransientCore, c: &Carry, lo: f64, hi: f64, n: usize, margin: f64)
                        -> Result<AccelSchedule, RowFailure> {
    let pi_b = ft.inner.inner.base.pi_b;
    let mut rows: Vec<(f64, f64)> = try_rows(ft, c, lo, hi, n)?.iter()
        .map(|eq| (eq.close.n_hp, eq.close.f * eq.close.mdot_air / (eq.close.pt4 / pi_b))).collect();
    rows.sort_by(|a, b| a.partial_cmp(b).expect("running-line rows are finite"));
    Ok(AccelSchedule { margin, n_h: rows.iter().map(|&(a, _)| a).collect(), kappa: rows.iter().map(|&(_, b)| b).collect() })
}

/// Plain words for a design-flight table that cannot be built (anchor § 1).
pub fn row_words(what: &str, c: &Carry, lo: f64, hi: f64, f: &RowFailure) -> String {
    let d = &c.design;
    let at = if f.tt4.is_finite() { format!(", and at {:.0} K the engine has no steady point there", f.tt4) } else { String::new() };
    format!("The control's {what} is made at the design flight ({:.0} K, {:.0} kPa, Mach {}). In the design flight's terms \
             this run's throttles are {:.0}–{:.0} K{at}: a table made at the design flight cannot cover this flight. Make the \
             tables at this flight instead. (The model says: {})",
            d.t0, d.p0 / 1000.0, d.m0, lo.min(hi) * c.k, lo.max(hi) * c.k, f.message)
}
