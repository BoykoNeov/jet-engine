//! RUNG 100 — BELOW CHOKE: rung 99's formula worked out AT THE READING — the jet's push from the pressure ratio, exact
//! at and below choke, and for the fan speed the LP shaft rebalanced against the flight's own air.
//!
//! Anchor (the device and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung100-anchor-unchoked-formula.md`. Spec: `docs/rung100-spec.md`.
//!
//! # The push
//!
//! On the Controls engine's perfect gas (one `γ`) the convergent nozzle's gross thrust depends on its total pressure
//! alone: `push = a8·[p9·(1 + γ·M9²) − p0]` — choked `p9 = p*`, `M9 = 1`; below choke `p9 = p0` and `M9` from `pt9/p0`.
//! `Tt9` cancels in both. So `F_true = push(π_n·EPR·pt2) − V0·ṁ_face` at every point (anchor G2, ≤ 6.5e−11 N per
//! kg/s): a pressure-ratio control that knows the nozzle's area and loss and the outside air reads the jet exactly,
//! and its whole error is the air it believes the engine swallows, `ṁ̂` — the carried table's.
//!
//! # The fan speed — the LP shaft rebalanced on the table's own gas-generator line
//!
//! A fan speed does not fix the jet: below choke the back-pressure takes work off the LP turbine (rung 97), so the
//! engine holding a fan speed burns hotter than the table's. Each design row `x` (its burner temperature) carries its
//! LP-turbine inlet and its LP power `w`. At `x` the LP turbine is re-solved against THIS flight's `p0`; its power,
//! scaled by the re-solved work over the choked work at the same inlet, is the LP load it can carry, and the table's
//! fan speed at that load is the speed it holds, `ν(x)`. The thrust believed at a fan speed is `F̂(x)` at the root of
//! `ν(x) = ν`. Choked, the re-solve IS the pin, the ratio is exactly 1, and this is rung 99's formula in algebra
//! (anchor G9). It cannot be exact: the row's compressors are the choked engine's (anchor G5).
//!
//! # Reduce
//!
//! `Tables::Formula { choked: true, .. }` (the request's `formula_choked`) never builds a [`Formula`]: rung 99's
//! column is read off the rows, bit for bit.

use crate::corrected_tables::Carry;
use crate::fuel_transient::FuelTransientCore;
use crate::gas::{powp, Abort};
use crate::inferred_thrust::{face, Reading, Table};
use crate::spool::{try_illinois, ILLINOIS_MAXIT};
use crate::two_spool_transient::Instant2;
use crate::unchoked_lp::try_lp_turbine;

/// What the control knows of the nozzle: its area (per design airflow) and loss, the gas's `γ`, the outside air.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Push {
    pub a8: f64,
    pub pi_n: f64,
    pub gamma: f64,
    pub p0: f64,
}

impl Push {
    /// The machine's nozzle against the air `p0`. The closed form needs one `γ` (anchor G1).
    pub fn new(ft: &FuelTransientCore, p0: f64) -> Push {
        let (base, gas) = (&ft.inner.inner.base, ft.inner.gas());
        let gamma = gas.gamma_t_at(300.0, 0.0);
        assert!(gas.gamma_t_at(2000.0, 0.05) == gamma, "rung 100's push is the perfect gas's closed form");
        Push { a8: base.a8, pi_n: base.pi_n, gamma, p0 }
    }

    /// The nozzle chokes: its sonic pressure is above the outside air's.
    pub fn choked(&self, pt5: f64) -> bool {
        let g = self.gamma;
        self.pi_n * pt5 * powp(2.0 / (g + 1.0), g / (g - 1.0)) > self.p0
    }

    /// The gross thrust at nozzle-inlet total pressure `pt5`, N per kg/s of design airflow.
    pub fn at(&self, pt5: f64) -> f64 {
        let (g, p0) = (self.gamma, self.p0);
        let pt9 = self.pi_n * pt5;
        let star = pt9 * powp(2.0 / (g + 1.0), g / (g - 1.0));
        if star > p0 {
            self.a8 * ((1.0 + g) * star - p0)
        } else {
            let m2 = 2.0 / (g - 1.0) * (powp(pt9 / p0, (g - 1.0) / g) - 1.0);
            self.a8 * g * p0 * m2
        }
    }
}

/// One design row as the fan-speed formula reads it, carried to this flight (`pt45` by `δ`, temperatures by `θ`,
/// the fan speed by `√θ`, the airflow by `δ/√θ`, each with its switch); `x` stays the design burner temperature.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GasGen {
    pub x: f64,
    pub nu: f64,
    pub pt45: f64,
    pub tt45: f64,
    pub f45: f64,
    pub eta: f64,
    /// The LP turbine's specific power in that steady state — the LP load the shaft carries there.
    pub w: f64,
    pub face: f64,
    /// At the build: the fan speed this row holds against the flight's air, and the thrust believed there.
    pub nu_x: f64,
    pub f_hat: f64,
}

/// The LP turbine's fuel/air ratio: the burner's, diluted by rung 91's cooling air where it rejoins.
pub fn f45(i: &Instant2) -> f64 {
    let c = &i.close;
    match c.cooling {
        Some(cool) => {
            let hpc = (1.0 - c.bleed.unwrap_or(0.0)) * c.mdot_face.expect("a rung-91 closure sets it");
            c.f * c.mdot_air / (c.mdot_air + cool.total() * hpc)
        }
        None => c.f,
    }
}

/// Linear in `x` between rows rising in it, the end segments carried on — rung 96's [`crate::inferred_thrust::lin`]
/// on the formula's own rows.
fn lin_by(rows: &[GasGen], x: impl Fn(&GasGen) -> f64, y: impl Fn(&GasGen) -> f64, v: f64) -> f64 {
    let n = rows.len();
    let k = if v <= x(&rows[0]) { 0 } else { rows.windows(2).position(|w| v <= x(&w[1])).unwrap_or(n - 2) };
    let (a, b) = (&rows[k], &rows[k + 1]);
    y(a) + (y(b) - y(a)) * (v - x(a)) / (x(b) - x(a))
}

/// The fan-speed formula at row parameter `x`: the fan speed held, the thrust believed, the LP turbine's re-solved
/// pressure ratio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtRow {
    pub nu: f64,
    pub f_hat: f64,
    pub pi_lpt: f64,
}

/// The pressure ratio's root tolerance (a pressure ratio near 2), and the row parameter's (K).
const TOL_EPR: f64 = 1e-14;
const TOL_X: f64 = 1e-10;

/// The formula's readers.
#[derive(Clone, Debug, PartialEq)]
pub enum Formula {
    /// The pressure ratio: `F̂(e) = push(π_n·e·pt2) − V0·ṁ̂(e)`; `rows` are `(e, F̂)` at the table's rows, rising.
    Epr { push: Push, rows: Vec<(f64, f64)> },
    /// The fan speed: the LP shaft rebalanced on the rows (rising in `x`); `k` turns `x` into this flight's burner
    /// temperature (rung 98's row translation).
    Fan { push: Push, rows: Vec<GasGen>, k: f64 },
}

impl Formula {
    /// The readers on the carried table `t` (rung 99's rows, this flight's `pt2`/`V0`), its design equilibria `eqs`.
    pub fn new(ft: &FuelTransientCore, c: &Carry, t: &Table, eqs: &[Instant2], p0: f64, r: Reading)
               -> Result<Formula, String> {
        let push = Push::new(ft, p0);
        let no_rise = |what: &str| format!(
            "The control's thrust formula at this flight speed: {what}, so a reading cannot be turned back into thrust.");
        match r {
            Reading::Epr => {
                let mut rows: Vec<(f64, f64)> = t.rows.iter().map(|w| (w.epr, epr_f_hat(&push, t, w.epr))).collect();
                rows.sort_by(|a, b| a.0.total_cmp(&b.0));
                if !rows.windows(2).all(|w| w[1].1 > w[0].1) {
                    return Err(no_rise("the jet's push less the intake's drag does not rise with the pressure ratio"));
                }
                Ok(Formula::Epr { push, rows })
            }
            Reading::LpSpeed => {
                assert!(eqs.iter().all(|i| i.close.wgas.is_none()), "rung 100's fan formula reads the machine's own gas");
                let t2 = c.t * c.t;
                let mut rows: Vec<GasGen> = eqs.iter().map(|i| GasGen {
                    x: i.tt4, nu: i.nu_lp * c.t, pt45: i.pi_hpt * i.close.pt4 * c.d, tt45: i.tt45 * t2, f45: f45(i),
                    eta: i.eta_lpt, w: i.pt_lp, face: face(i) * c.d / c.t, nu_x: f64::NAN, f_hat: f64::NAN,
                }).collect();
                rows.sort_by(|a, b| a.x.total_cmp(&b.x));
                if !rows.windows(2).all(|w| w[1].w > w[0].w && w[1].nu > w[0].nu) {
                    return Err(no_rise("the table's LP power and fan speed do not rise together"));
                }
                let mut f = Formula::Fan { push, rows: rows.clone(), k: c.k };
                for g in rows.iter_mut() {
                    let a = f.at_row(ft, t.v0, g.x).map_err(|e| f.fan_words(g.x, &e))?;
                    (g.nu_x, g.f_hat) = (a.nu, a.f_hat);
                }
                if !rows.windows(2).all(|w| w[1].nu_x > w[0].nu_x && w[1].f_hat > w[0].f_hat) {
                    return Err(no_rise("against this flight's air the table's engine does not rise in fan speed and \
                                        thrust together"));
                }
                if let Formula::Fan { rows: r, .. } = &mut f {
                    *r = rows;
                }
                Ok(f)
            }
        }
    }

    /// The fan-speed formula at row parameter `x` (anchor § 1).
    pub fn at_row(&self, ft: &FuelTransientCore, v0: f64, x: f64) -> Result<AtRow, Abort> {
        let Formula::Fan { push, rows, .. } = self else { unreachable!("the pressure ratio has no rows to rebalance") };
        let col = |y: fn(&GasGen) -> f64| lin_by(rows, |g| g.x, y, x);
        let (pt45, tt45, f45, eta, w) = (col(|g| g.pt45), col(|g| g.tt45), col(|g| g.f45), col(|g| g.eta), col(|g| g.w));
        let (base, gas) = (&ft.inner.inner.base, ft.inner.gas());
        let pin = base.try_solve_choked_turbine(gas, tt45, f45, base.a45, base.a8, base.pi_n, eta)?;
        let (pi, _, tt5) = try_lp_turbine(base, gas, tt45, f45, eta, pt45, push.p0)?;
        // The row's LP power at the re-solved work: exactly the row's own where the re-solve IS the pin.
        let h45 = gas.h_t(tt45, f45);
        let w_u = w * ((h45 - gas.h_t(tt5, f45)) / (h45 - gas.h_t(pin.2, f45)));
        Ok(AtRow { nu: lin_by(rows, |g| g.w, |g| g.nu, w_u), f_hat: push.at(pi * pt45) - v0 * col(|g| g.face), pi_lpt: pi })
    }

    /// The thrust the control believes at reading `v` (`t`: the table it was built on).
    pub fn try_inferred(&self, ft: &FuelTransientCore, t: &Table, r: Reading, v: f64) -> Result<f64, Abort> {
        match (self, r) {
            (Formula::Epr { push, .. }, Reading::Epr) => Ok(epr_f_hat(push, t, v)),
            (Formula::Fan { .. }, Reading::LpSpeed) => self.try_fan_root(ft, t, v).map(|(_, a)| a.f_hat),
            _ => unreachable!("a formula reads the reading it was built for"),
        }
    }

    /// The reading the control demands for thrust `f`.
    pub fn try_reading_at(&self, ft: &FuelTransientCore, t: &Table, r: Reading, f: f64) -> Result<f64, Abort> {
        match (self, r) {
            (Formula::Epr { push, rows }, Reading::Epr) => {
                let g = |e: f64| Ok(epr_f_hat(push, t, e) - f);
                let (e0, en) = (rows[0].0, rows[rows.len() - 1].0);
                let span = (en - e0) / (rows.len() - 1) as f64;
                let (a, b, ga, gb) = bracket(rows.iter().map(|&(e, h)| (e, h - f)).collect(), span, g, "pressure ratio")
                    .map_err(|m| Abort(m.replace("{asked}", &format!("{f:.1} N per kg/s"))))?;
                if ga == 0.0 { return Ok(a) }
                if gb == 0.0 { return Ok(b) }
                try_illinois(g, a, b, ga, gb, TOL_EPR, ILLINOIS_MAXIT)
            }
            (Formula::Fan { rows, .. }, Reading::LpSpeed) => {
                let g = |x: f64| self.at_row(ft, t.v0, x).map(|a| a.f_hat - f);
                let span = (rows[rows.len() - 1].x - rows[0].x) / (rows.len() - 1) as f64;
                let (a, b, ga, gb) = bracket(rows.iter().map(|q| (q.x, q.f_hat - f)).collect(), span, g, "row")
                    .map_err(|m| Abort(self.fan_root_words(&m.replace("{asked}", &format!("a thrust of {f:.1} N per kg/s")))))?;
                let x = if ga == 0.0 { a } else if gb == 0.0 { b } else { try_illinois(g, a, b, ga, gb, TOL_X, ILLINOIS_MAXIT)? };
                self.at_row(ft, t.v0, x).map(|a| a.nu)
            }
            _ => unreachable!("a formula reads the reading it was built for"),
        }
    }

    /// The fan-speed formula at fan speed `nu`: the row parameter it settles on and the formula there. Where the LP
    /// turbine stays choked at the table's own row for `nu`, that row (anchor G9).
    pub fn try_fan_root(&self, ft: &FuelTransientCore, t: &Table, nu: f64) -> Result<(f64, AtRow), Abort> {
        let Formula::Fan { rows, .. } = self else { unreachable!("a fan-speed root needs the fan formula") };
        let x_nu = lin_by(rows, |g| g.nu, |g| g.x, nu);
        let at = self.at_row(ft, t.v0, x_nu)?;
        if (at.nu - nu).abs() <= 1e-14 * nu.abs() {
            return Ok((x_nu, at));
        }
        let g = |x: f64| self.at_row(ft, t.v0, x).map(|a| a.nu - nu);
        let span = (rows[rows.len() - 1].x - rows[0].x) / (rows.len() - 1) as f64;
        let (a, b, ga, gb) = bracket(rows.iter().map(|q| (q.x, q.nu_x - nu)).collect(), span, g, "row")
            .map_err(|m| Abort(self.fan_root_words(&m.replace("{asked}", &format!("a fan speed of {:.2} %", 100.0 * nu)))))?;
        let x = if ga == 0.0 { a } else if gb == 0.0 { b } else { try_illinois(g, a, b, ga, gb, TOL_X, ILLINOIS_MAXIT)? };
        Ok((x, self.at_row(ft, t.v0, x)?))
    }

    /// This flight's burner temperature of row parameter `x` (rung 98's translation undone).
    pub fn flight_tt4(&self, x: f64) -> f64 {
        match self { Formula::Fan { k, .. } => x / k, Formula::Epr { .. } => f64::NAN }
    }

    /// Plain words for a row whose LP turbine has no working state against this flight's air (anchor § 1).
    fn fan_words(&self, x: f64, e: &Abort) -> String {
        let Formula::Fan { push, .. } = self else { unreachable!("the fan formula's words") };
        format!("The control's fan-speed formula: the table's engine at {:.0} K, with its exhaust pushing against this \
                 flight's air ({:.1} kPa), has no working state — the nozzle cannot pass its flow. The formula cannot \
                 cover this throttle; make the tables at this flight instead. (The model says: {})",
                self.flight_tt4(x), push.p0 / 1000.0, e.0)
    }

    fn fan_root_words(&self, m: &str) -> String {
        format!("The control's fan-speed formula: {m}; make the tables at this flight instead.")
    }
}

/// The pressure-ratio formula at reading `e` on table `t`.
pub fn epr_f_hat(push: &Push, t: &Table, e: f64) -> f64 {
    push.at(e * t.pt2) - t.v0 * t.face_at_epr(e)
}

/// A bracket of a residual rising with its parameter: from its values at the rows `(p, g)` (rising in `p`), the
/// segment holding its zero, or — beyond the rows — stepped out by `span` up to as many rows again. `{asked}` in the
/// error is filled by the caller.
fn bracket(rows: Vec<(f64, f64)>, span: f64, mut g: impl FnMut(f64) -> Result<f64, Abort>, what: &str)
           -> Result<(f64, f64, f64, f64), String> {
    let n = rows.len();
    if let Some(k) = rows.windows(2).position(|w| w[0].1 <= 0.0 && 0.0 <= w[1].1) {
        return Ok((rows[k].0, rows[k + 1].0, rows[k].1, rows[k + 1].1));
    }
    let (mut a, mut ga, dir) = if rows[0].1 > 0.0 { (rows[0].0, rows[0].1, -1.0) } else { (rows[n - 1].0, rows[n - 1].1, 1.0) };
    for _ in 0..n {
        let b = a + dir * span;
        let gb = g(b).map_err(|e| format!(
            "{{asked}} lies beyond the table's {what}s, where the table's engine, with its exhaust pushing against this \
             flight's air, has no working state (the model says: {})", e.0))?;
        if (gb > 0.0) != (ga > 0.0) || gb == 0.0 {
            return Ok(if dir > 0.0 { (a, b, ga, gb) } else { (b, a, gb, ga) });
        }
        (a, ga) = (b, gb);
    }
    Err(format!("{{asked}} is not reached within the table's {what}s and as many again beyond them"))
}
