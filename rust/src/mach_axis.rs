//! RUNG 99 — THE SPEED AXIS: the control's three tables made at EIGHT flight speeds and blended at the one flown —
//! or the thrust table's thrust recomputed for it — because rung 98's pressure and temperature corrections cannot see a
//! change of flight speed.
//!
//! Anchor (the device and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung99-anchor-mach-axis.md`. Spec: `docs/rung99-spec.md`.
//!
//! # The grid
//!
//! A SLICE is rung 98's design-flight table with the slice's flight in the design's place: built in the design
//! altitude's air at Mach `M_j` ([`GRID`]) on rows at the run's throttles × `Tt2_slice/Tt2`, carried by that slice's own
//! `δ_j`, `θ_j`. The Mach-0.85 slice IS the design flight ([`slice_flight`]), so a flight at Mach 0.85 reads rung 98's
//! own table. A flight at Mach `M` reads the two slices around it (outside the grid the two end slices, the straight
//! line carried on — the user's choice) with the weight `w = (M − M_a)/(M_b − M_a)` on the second.
//!
//! # The blend — at equal reading, never row by row
//!
//! The two slices' rows lie at different throttles, so each table is blended on its own abscissa (the reading for the
//! thrust table, HP speed for the steady table, corrected HP speed for the schedule): the blended rows sit at the union
//! of both slices' abscissae, every other column `(1 − w)·y_a(x) + w·y_b(x)`, each slice read the way its own reader
//! reads it (rung 96's table carries its end segments on, rung 95's steady table and rung 48's schedule hold their end
//! values). Inside both slices' rows that IS "read both tables at the same reading and blend the answers".
//!
//! # The formula — the thrust table only
//!
//! Rung 98's carried design table with its thrust recomputed for this flight: `F̂ = d·F_d + d·V0_d·ṁ_d − V0·ṁ̂ +
//! a8·(d·p0_d − p0)`, `ṁ̂ = ṁ_d·d/t`. Every term is linear in the row's own columns, so reading between rows gives the
//! formula at any reading. Wherever the point and its rows are choked, `F_true − F̂ = V0·(ṁ̂ − ṁ_face)` exactly (anchor
//! G3) — only the table's guess of the air swallowed is left.
//!
//! # Reduce
//!
//! `SpeedCorr::None` is rung 98's carry; so is a grid flight at Mach 0.85 (its one slice is the design flight).
//!
//! # Rung 100
//!
//! Two slow slices (Mach 0.05, 0.15) before the eight — `low` off is rung 99's grid ([`GRID99`]) — and the formula
//! read at the reading by [`crate::unchoked_formula`] — `choked` on is rung 99's column alone.

use crate::corrected_tables::{self as ct, Carry, RowFailure};
use crate::engine::FlightCondition;
use crate::fuel_transient::{AccelSchedule, FuelTransientCore};
use crate::inferred_thrust::{epr, face, lin, Reading, Row, Table};
use crate::two_spool_transient::Instant2;
use crate::unchoked_formula as uf;

/// How the control corrects for flight speed (rung 99's setting; acts only on tables made at the design flight).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeedCorr {
    /// Tables at the eight grid speeds, blended at the one flown — the default (the user's choice).
    Grid,
    /// The thrust table's thrust recomputed for this flight (the other two tables are rung 98's).
    Formula,
    /// No speed correction — rung 98.
    None,
}

impl SpeedCorr {
    pub const ALL: [SpeedCorr; 3] = [SpeedCorr::Grid, SpeedCorr::Formula, SpeedCorr::None];
    pub fn key(self) -> &'static str {
        match self { SpeedCorr::Grid => "grid", SpeedCorr::Formula => "formula", SpeedCorr::None => "none" }
    }
}

/// The grid's flight speeds. Rung 99's eight (the user's, after its reach probe: to Mach 3 so a fast flight's slices
/// can be built) and RUNG 100's two slow slices before them — the user's, after `docs/rung100-spec.md`'s G6: below
/// Mach 0.3 rung 99 carried a straight line on, and where the nozzle unchokes that line is far off.
pub const GRID: [f64; 10] = [0.05, 0.15, 0.3, 0.6, 0.85, 1.2, 1.6, 2.0, 2.5, 3.0];

/// Rung 99's grid — rung 100's reduce setting (`low_grid` off).
pub const GRID99: [f64; 8] = [0.3, 0.6, 0.85, 1.2, 1.6, 2.0, 2.5, 3.0];

/// The slice that IS the design flight, by its Mach.
pub const DESIGN_MACH: f64 = 0.85;

/// The grid a run reads: rung 100's with the slow slices, else rung 99's.
pub fn grid(low: bool) -> &'static [f64] {
    if low { &GRID } else { &GRID99 }
}

/// The slices a flight at Mach `m` reads in grid `g`: the first, the second (none at a grid speed), and the weight on
/// the second (below 0 or above 1 outside the grid).
pub fn bracket(g: &[f64], m: f64) -> (usize, Option<usize>, f64) {
    if let Some(j) = g.iter().position(|&x| x == m) {
        return (j, None, 0.0);
    }
    let n = g.len();
    let a = if m < g[0] { 0 } else if m > g[n - 1] { n - 2 } else { g.iter().rposition(|&x| x < m).expect("inside") };
    (a, Some(a + 1), (m - g[a]) / (g[a + 1] - g[a]))
}

/// The flight of the slice at Mach `m`: the design flight ITSELF for the design slice (so it is rung 98's table, bit
/// for bit), else the design altitude's air at that Mach.
pub fn slice_flight(design: &FlightCondition, m: f64) -> FlightCondition {
    if m == DESIGN_MACH { *design } else { FlightCondition::new(design.t0, design.p0, m) }
}

/// A flight in the design altitude's air at a speed of grid `g` — where a slice IS a table made at this flight.
pub fn at_slice_flight(g: &[f64], design: &FlightCondition, flight: &FlightCondition) -> bool {
    flight.t0 == design.t0 && flight.p0 == design.p0 && g.contains(&flight.m0)
}

/// One slice: its Mach and the carry from its flight to the flight flown.
#[derive(Clone, Copy, Debug)]
pub struct Slice {
    pub m: f64,
    pub carry: Carry,
}

/// Where the control's tables come from under a design-flight table.
#[derive(Clone, Copy, Debug)]
pub enum Tables {
    /// Rung 98: the design-flight table carried (no speed correction, or a grid flight at Mach 0.85).
    Design(Carry),
    /// Two grid slices and the weight on the second (one slice at a grid speed: `b` none).
    Grid { a: Slice, b: Option<Slice>, w: f64 },
    /// Rung 98's carry, the thrust table's thrust recomputed for this flight: the model's nozzle area `a8` (per design
    /// airflow) and both flights' ambient pressure. `choked` is rung 99's column (rung 100's reduce setting); else
    /// RUNG 100's readers ([`crate::unchoked_formula`]) on the same carry.
    Formula { carry: Carry, a8: f64, p0: f64, p0_d: f64, choked: bool },
}

impl Tables {
    /// `low` is rung 100's grid with the slow slices; `choked` rung 99's formula column.
    #[allow(clippy::too_many_arguments)]
    pub fn new(ft: &FuelTransientCore, design: &FlightCondition, flight: &FlightCondition, corr: SpeedCorr,
               pressure: bool, temperature: bool, low: bool, choked: bool) -> Tables {
        let carry = |from: &FlightCondition| Carry::new(ft, from, flight, pressure, temperature);
        let g = grid(low);
        match corr {
            SpeedCorr::None => Tables::Design(carry(design)),
            SpeedCorr::Formula => Tables::Formula { carry: carry(design), a8: ft.inner.inner.base.a8, p0: flight.p0,
                                                    p0_d: design.p0, choked },
            SpeedCorr::Grid => match bracket(g, flight.m0) {
                (a, None, _) if g[a] == DESIGN_MACH => Tables::Design(carry(design)),
                (a, b, w) => {
                    let slice = |j: usize| Slice { m: g[j], carry: carry(&slice_flight(design, g[j])) };
                    Tables::Grid { a: slice(a), b: b.map(slice), w }
                }
            },
        }
    }

    /// Rung 98's design-flight carry, where these tables are built on it (none for the grid).
    pub fn design_carry(&self) -> Option<&Carry> {
        match self { Tables::Design(c) | Tables::Formula { carry: c, .. } => Some(c), Tables::Grid { .. } => None }
    }

    /// The thrust table the control reads, blended or recomputed, and — for rung 98's split — the design-flight table
    /// it was carried from (none for the grid). `r` is the reading the control uses: the grid blends on it.
    pub fn thrust_table(&self, ft: &FuelTransientCore, lo: f64, hi: f64, r: Reading)
                        -> Result<(Table, Option<Table>), String> {
        let what = "thrust table";
        match self {
            Tables::Design(c) => {
                let dt = ct::try_design_table(ft, c, lo, hi).map_err(|f| ct::row_words(what, c, lo, hi, &f))?;
                Ok((c.thrust_table(&dt), Some(dt)))
            }
            Tables::Formula { carry: c, a8, p0, p0_d, choked } => {
                let (dt, rows) = ct::try_design_table_rows(ft, c, lo, hi).map_err(|f| ct::row_words(what, c, lo, hi, &f))?;
                let mut t = formula_table(c, &dt, *a8, *p0, *p0_d);
                check_rise(&t.rows).map_err(|m| format!("The control's thrust table, recomputed for this flight speed: {m}"))?;
                // RUNG 100: the formula read AT the reading — the jet's push from the pressure ratio, or the LP shaft
                // rebalanced against this flight's air for the fan speed. Rung 99's column stays as the rows.
                if !choked {
                    t.formula = Some(uf::Formula::new(ft, c, &t, &rows, *p0, r)?);
                }
                Ok((t, Some(dt)))
            }
            Tables::Grid { a, b, w } => {
                let one = |s: &Slice| ct::try_design_table(ft, &s.carry, lo, hi).map(|dt| s.carry.thrust_table(&dt))
                    .map_err(|f| self.slice_words(what, s, lo, hi, &f));
                let ta = one(a)?;
                let Some(b) = b else { return Ok((ta, None)) };
                let t = blend_thrust(&ta, &one(b)?, *w, r);
                check_rise(&t.rows).map_err(|m| format!("The control's thrust table, blended between Mach {} and {}: {m}",
                                                        a.m, b.m))?;
                Ok((t, None))
            }
        }
    }

    /// Rung 95's steady `(ν_H, fuel)` table, carried (and blended under the grid).
    pub fn steady_table(&self, ft: &FuelTransientCore, lo: f64, hi: f64) -> Result<Vec<(f64, f64)>, String> {
        let what = "speed governor's steady table";
        let one = |c: &Carry| ct::try_design_steady_table(ft, c, lo, hi).map(|t| c.steady_table(&t));
        match self {
            Tables::Design(c) | Tables::Formula { carry: c, .. } => one(c).map_err(|f| ct::row_words(what, c, lo, hi, &f)),
            Tables::Grid { a, b, w } => {
                let ta = one(&a.carry).map_err(|f| self.slice_words(what, a, lo, hi, &f))?;
                let Some(b) = b else { return Ok(ta) };
                let tb = one(&b.carry).map_err(|f| self.slice_words(what, b, lo, hi, &f))?;
                Ok(blend_pairs(&ta, &tb, *w))
            }
        }
    }

    /// Rung 48's schedule, carried (and blended under the grid).
    pub fn accel(&self, ft: &FuelTransientCore, lo: f64, hi: f64, n: usize, margin: f64) -> Result<AccelSchedule, String> {
        let what = "acceleration schedule";
        let one = |c: &Carry| ct::try_design_accel(ft, c, lo, hi, n, margin).map(|t| c.accel(&t));
        match self {
            Tables::Design(c) | Tables::Formula { carry: c, .. } => one(c).map_err(|f| ct::row_words(what, c, lo, hi, &f)),
            Tables::Grid { a, b, w } => {
                let ta = one(&a.carry).map_err(|f| self.slice_words(what, a, lo, hi, &f))?;
                let Some(b) = b else { return Ok(ta) };
                let tb = one(&b.carry).map_err(|f| self.slice_words(what, b, lo, hi, &f))?;
                let pa: Vec<(f64, f64)> = ta.n_h.iter().copied().zip(ta.kappa.iter().copied()).collect();
                let pb: Vec<(f64, f64)> = tb.n_h.iter().copied().zip(tb.kappa.iter().copied()).collect();
                let rows = blend_pairs(&pa, &pb, *w);
                Ok(AccelSchedule { margin, n_h: rows.iter().map(|r| r.0).collect(), kappa: rows.iter().map(|r| r.1).collect() })
            }
        }
    }

    /// Plain words for a grid slice that cannot be built (anchor § 1).
    fn slice_words(&self, what: &str, s: &Slice, lo: f64, hi: f64, f: &RowFailure) -> String {
        let Tables::Grid { a, b, .. } = self else { unreachable!("slice words are the grid's") };
        let d = &s.carry.design;
        let made = match b {
            Some(b) => format!("Mach {} and {}", a.m, b.m),
            None => format!("Mach {}", a.m),
        };
        let at = if f.tt4.is_finite() { format!(", and at {:.0} K the engine has no steady point there", f.tt4) } else { String::new() };
        format!("The control's {what} is made at {made} in the design altitude's air ({:.0} K, {:.0} kPa). At Mach {} this \
                 run's throttles are {:.0}–{:.0} K{at}: tables made there cannot cover this flight. Make the tables at this \
                 flight instead. (The model says: {})",
                d.t0, d.p0 / 1000.0, s.m, lo.min(hi) * s.carry.k, lo.max(hi) * s.carry.k, f.message)
    }

    /// The pressure-ratio control's gap split under the formula (anchor § 1): the AIRFLOW GUESS `V0·(ṁ̂ − ṁ_face)`,
    /// with `ṁ̂` the table's airflow at the reading, and THE REST — zero while the nozzle is choked.
    pub fn formula_split(&self, table: &Table, i: &Instant2, f_true: f64, f_hat: f64) -> Option<(f64, f64)> {
        let Tables::Formula { carry, .. } = self else { return None };
        let guess = carry.v0 * (table.face_at_epr(epr(i, table.pt2)) - face(i));
        Some((guess, f_true - f_hat - guess))
    }
}

/// Rung 98's carried table with the thrust recomputed for the flight flown (module docs).
pub fn formula_table(c: &Carry, design: &Table, a8: f64, p0: f64, p0_d: f64) -> Table {
    let mut t = c.thrust_table(design);
    let push = a8 * (c.d * p0_d - p0);
    for (w, d) in t.rows.iter_mut().zip(&design.rows) {
        w.thrust = c.d * d.thrust + c.d * design.v0 * d.face - c.v0 * w.face + push;
    }
    t
}

/// Rung 96's check: a table rises in thrust, pressure ratio, LP speed and fuel together.
fn check_rise(rows: &[Row]) -> Result<(), String> {
    match rows.windows(2).all(|w| w[1].thrust > w[0].thrust && w[1].epr > w[0].epr && w[1].nu_lp > w[0].nu_lp
                                  && w[1].fuel > w[0].fuel) {
        true => Ok(()),
        false => Err("it does not rise in thrust, pressure ratio, LP speed and fuel together, so a reading cannot be \
                      turned back into thrust".into()),
    }
}

/// Two carried thrust tables blended at equal reading `r` (module docs), rows at the union of both readings.
pub fn blend_thrust(ta: &Table, tb: &Table, w: f64, r: Reading) -> Table {
    let x = |row: &Row| match r { Reading::Epr => row.epr, Reading::LpSpeed => row.nu_lp };
    let mut xs: Vec<f64> = ta.rows.iter().chain(&tb.rows).map(x).collect();
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    let at = |t: &Table, v: f64, y: fn(&Row) -> f64| lin(&t.rows, x, y, v);
    let mix = |v: f64, y: fn(&Row) -> f64| (1.0 - w) * at(ta, v, y) + w * at(tb, v, y);
    let mut rows: Vec<Row> = xs.iter().map(|&v| Row {
        thrust: mix(v, |q| q.thrust),
        epr: if r == Reading::Epr { v } else { mix(v, |q| q.epr) },
        nu_lp: if r == Reading::LpSpeed { v } else { mix(v, |q| q.nu_lp) },
        fuel: mix(v, |q| q.fuel),
        face: mix(v, |q| q.face),
    }).collect();
    rows.sort_by(|a, b| a.thrust.total_cmp(&b.thrust));
    Table { rows, pt2: ta.pt2, v0: ta.v0, formula: None }
}

/// Linear between rows rising in `x`, the END VALUES held beyond — rung 95's `steady_fuel` and rung 48's `cap`.
pub fn held_lin(t: &[(f64, f64)], v: f64) -> f64 {
    if v <= t[0].0 {
        return t[0].1;
    }
    if v >= t[t.len() - 1].0 {
        return t[t.len() - 1].1;
    }
    let k = t.windows(2).position(|p| v <= p[1].0).expect("inside the table");
    let (a, b) = (t[k], t[k + 1]);
    a.1 + (b.1 - a.1) * (v - a.0) / (b.0 - a.0)
}

/// Two `(x, y)` tables blended at equal `x`, each read with its end values held, rows at the union of both `x`.
pub fn blend_pairs(ta: &[(f64, f64)], tb: &[(f64, f64)], w: f64) -> Vec<(f64, f64)> {
    let mut xs: Vec<f64> = ta.iter().chain(tb).map(|p| p.0).collect();
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    xs.iter().map(|&v| (v, (1.0 - w) * held_lin(ta, v) + w * held_lin(tb, v))).collect()
}
