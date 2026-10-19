//! RUNG 101 — THE AIRFLOW OFF THE LINE: where rung 100's formula gets the air the engine swallows — rung 100's carried
//! table, the speed grid's tables, or an EXHAUST-TEMPERATURE SENSOR on the nozzle's own flow.
//!
//! Anchor (the device and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung101-anchor-airflow-source.md`. Spec: `docs/rung101-spec.md`.
//!
//! # The three sources
//!
//! Rung 100 made the jet's push exact from the pressure ratio; its whole remaining error is the air it believes the
//! engine swallows, `ṁ̂` — read off a table whose engine is on a different line below choke. The setting picks `ṁ̂`:
//! * `table` — rung 100's carried design table (the default, bit for bit);
//! * `grid` — the speed grid's thrust table at this flight's Mach, its airflow column only (anchor G1: the steady
//!   start to the floor of a table made at this flight, outside the 0.3–0.6 gap);
//! * `exhaust` — the sensor. On the perfect gas the convergent nozzle's flow is a closed form in its total pressure
//!   and temperature, `ṁ9 = a8·ρ9·V9` (choked `p9 = p*`, else `p9 = p0`); the control meters its own fuel and commands
//!   its own valve, so the air in is `(ṁ9 − W_f)/((1 − b)(1 − c))` — the plant's own mass bookkeeping, to 4e−14
//!   (anchor G2). On the pressure ratio the nozzle's total pressure is MEASURED (`EPR·pt2`); on the fan speed it is
//!   B's own guess at its row, so there the sensor is not exact (anchor G4).
//!
//! # The lag
//!
//! A real probe responds slowly: `dT_m/ds = (Tt5 − T_m)/τ`, `τ` in HP-shaft time constants, a THIRD STATE of the
//! governed march started at the steady start's `Tt5` ([`crate::governor`]). Off, `T_m` is the instant's `Tt5`.
//!
//! # Reduce
//!
//! [`AirflowFrom::Table`] builds nothing new: rung 100's readers, bit for bit.

use crate::gas::powp;
use crate::two_spool_transient::Instant2;
use crate::unchoked_formula::{f45, Push};

/// Where rung 100's formula gets the air swallowed (the user's three-way choice; `Table` the default).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AirflowFrom {
    /// Rung 100's carried design table.
    Table,
    /// The speed grid's thrust table at this flight's Mach.
    Grid,
    /// The exhaust-temperature sensor on the nozzle's flow.
    Exhaust,
}

impl AirflowFrom {
    pub const ALL: [AirflowFrom; 3] = [AirflowFrom::Table, AirflowFrom::Grid, AirflowFrom::Exhaust];
    pub fn key(self) -> &'static str {
        match self { AirflowFrom::Table => "table", AirflowFrom::Grid => "grid", AirflowFrom::Exhaust => "exhaust" }
    }
}

/// The sensor lag's range, HP-shaft time constants. RK4 on the linear decay is stable for `DS/τ < 2.78`, so with the
/// march's `DS` 0.02 the edge is 0.0072; the floor keeps the step a seventh of it (anchor § 1).
pub const TAU_MIN: f64 = 0.05;
pub const TAU_MAX: f64 = 5.0;

/// The sensor's lag on a governed run: its time constant (HP-shaft τ) and its reading at the start (the steady
/// start's `Tt5`, so the start's belief is the unlagged one).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensorLag {
    pub tau: f64,
    pub t0: f64,
}

/// A formula's airflow source, built.
#[derive(Clone, Debug, PartialEq)]
pub enum Air {
    /// Rung 100's: the carried table's own column.
    Table,
    /// The speed grid's table: `(reading, airflow)` rising in the reading, read linearly, end segments carried on.
    Grid(Vec<(f64, f64)>),
    /// The sensor: the hot gas's `R` (constant on the Controls engine, anchor G1 of rung 100).
    Exhaust { r: f64 },
}

impl Air {
    /// The grid's airflow at reading `v`.
    pub fn grid_at(rows: &[(f64, f64)], v: f64) -> f64 {
        let n = rows.len();
        let k = if v <= rows[0].0 { 0 } else { rows.windows(2).position(|w| v <= w[1].0).unwrap_or(n - 2) };
        let (a, b) = (rows[k], rows[k + 1]);
        a.1 + (b.1 - a.1) * (v - a.0) / (b.0 - a.0)
    }
}

/// What the sensor sees at an instant beside the reading: its temperature, the metered fuel, the share of the face air
/// that reaches the nozzle (`(1 − b)(1 − c)`: the valve's position, the customer's share), the gas's fuel/air ratio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sensed {
    pub tm: f64,
    pub wf: f64,
    pub through: f64,
    pub far: f64,
}

impl Sensed {
    /// The sensor at instant `i`: its reading `tm` when lagged, else the instant's own exhaust temperature.
    pub fn of(i: &Instant2, tm: Option<f64>) -> Sensed {
        let c = &i.close;
        Sensed {
            tm: tm.unwrap_or(i.tt5),
            wf: c.f * c.mdot_air,
            through: (1.0 - c.bleed.unwrap_or(0.0)) * (1.0 - c.customer.unwrap_or(0.0)),
            far: f45(i),
        }
    }
}

/// The convergent nozzle's flow at inlet total pressure `pt5` and temperature `tt5`, kg/s per kg/s of design airflow
/// (`a8` is per design airflow): `a8·ρ9·V9`, choked at `p*` and `M9 = 1`, else at `p0` and `M9` from `pt9/p0`.
pub fn nozzle_flow(push: &Push, r: f64, pt5: f64, tt5: f64) -> f64 {
    let g = push.gamma;
    let pt9 = push.pi_n * pt5;
    let star = pt9 * powp(2.0 / (g + 1.0), g / (g - 1.0));
    let (p9, m2) = if star > push.p0 {
        (star, 1.0)
    } else {
        (push.p0, 2.0 / (g - 1.0) * (powp(pt9 / push.p0, (g - 1.0) / g) - 1.0))
    };
    let t9 = tt5 / (1.0 + 0.5 * (g - 1.0) * m2);
    push.a8 * p9 / (r * t9) * powp(m2 * g * r * t9, 0.5)
}

/// The air the sensor credits at nozzle-inlet total pressure `pt5`.
pub fn sensed_air(push: &Push, r: f64, pt5: f64, s: &Sensed) -> f64 {
    (nozzle_flow(push, r, pt5, s.tm) - s.wf) / s.through
}
