//! RUNG 104 — THE LP TURBINE'S THROAT WAS NEVER CHOKED: a flow law replaces rung 38's pin on `A45`.
//!
//! PROTOTYPE (throwaway worktree) — the anchor is written from what this measures.
//!
//! # The equation
//!
//! Rung 38 pinned the LP turbine's flow parameter at its nozzle guide vane's CHOKED value, `W45 = ṁ√Tt45/pt45 =
//! A45·MFP*(Tt45)`, independent of how far the LP turbine expands. The whole LP turbine expands only ~1.45 (probe G1:
//! even taking the whole drop to the exit static at Mach 0.6 reaches 0.990–0.996 of the critical ratio), so the vane
//! never chokes. The law gives `W45` as a function of the LP turbine's own `x = pt5/pt45`, captured at the design point
//! (`W45(x_d, Tt45_d) = A45·MFP*(Tt45_d)`), so no new constant enters except the two-row split:
//!
//! * `Ellipse` — Stodola's ellipse (Cooke 1985): `W45 = W45_d·√(1−x²)/√(1−x_d²)` — the many-row limit.
//! * `OneRow` — the vane takes the whole drop: one convergent row (the gas's own isentropic flux, capped at its sonic
//!   point) from `pt45` to `pt5`.
//! * `TwoRows` — vane and rotor as two such rows in series (velocity dumped between them), sized at design to an
//!   EQUAL PRESSURE RATIO each — the guessed split.
//! * `Pin` — rung 38's choked vane: the OLD CODE, called literally.
//!
//! With the nozzle choked, the HP turbine's ratio is no longer pinned by `(A4, A45)` alone: its residual reads `W45` at
//! the LP turbine's own `x`, so the two turbines are one joint solve ([`TwoSpoolCore::try_turbine_pair`]).

use crate::components::{try_choked_mfp, try_sonic_throat};
use crate::gas::{powp, Abort, Gas};
use crate::two_spool::TwoSpoolCore;

/// The COST instrument (anchor § COST; C3 was diagnosed by it): (joint searches, joint residual evaluations, `w45` calls,
/// interstage residual evaluations), per thread. Read by the probes, not by a gate.
pub mod census {
    use std::cell::Cell;
    thread_local! { pub static C: Cell<[u64; 4]> = const { Cell::new([0; 4]) }; }
    pub fn bump(i: usize) { C.with(|c| { let mut v = c.get(); v[i] += 1; c.set(v) }) }
    pub fn take() -> [u64; 4] { C.with(|c| c.replace([0; 4])) }
}

/// What sets the LP turbine's flow parameter `W45 = ṁ√Tt45/pt45`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TurbineLaw {
    /// Rung 38's choked vane — the old code, literally.
    Pin,
    /// Stodola's ellipse — the default.
    Ellipse,
    /// One convergent row takes the whole drop.
    OneRow,
    /// Vane + rotor, equal pressure ratio each at design.
    TwoRows,
}

impl TurbineLaw {
    pub const ALL: [TurbineLaw; 4] = [TurbineLaw::Ellipse, TurbineLaw::TwoRows, TurbineLaw::OneRow, TurbineLaw::Pin];

    /// The pages' request key `turbine`.
    pub fn key(self) -> &'static str {
        match self {
            TurbineLaw::Ellipse => "ellipse",
            TurbineLaw::TwoRows => "two",
            TurbineLaw::OneRow => "one",
            TurbineLaw::Pin => "pin",
        }
    }

    pub fn from_key(k: &str) -> Result<Self, String> {
        Self::ALL.into_iter().find(|l| l.key() == k)
            .ok_or_else(|| format!("unknown turbine law {k:?} (\"ellipse\", \"two\", \"one\" or \"pin\")"))
    }

    /// Writes `turbine` into a view's settings object only off its default, so every default reply keeps its shape.
    pub fn push_json(self, j: &mut crate::visuals::Json) {
        if let crate::visuals::Json::Obj(kv) = j {
            if self != TurbineLaw::default() {
                kv.push(("turbine".into(), crate::visuals::Json::from(self.key())));
            }
        }
    }
}

/// The ellipse (the user's choice, 2026-10-20) — or, built with the `pin-default` feature, rung 38's pin: the whole
/// gate on the old model (gate R1a).
impl Default for TurbineLaw {
    fn default() -> Self {
        if cfg!(feature = "pin-default") { TurbineLaw::Pin } else { TurbineLaw::Ellipse }
    }
}

/// The law's design capture.
#[derive(Clone, Copy, Debug)]
pub struct LpCapacity {
    /// `pt5/pt45` at design.
    pub xd: f64,
    /// `pt45/pt4` at design — where the joint search starts.
    pub phd: f64,
    /// `Tt45` at design.
    pub td: f64,
    /// `W45` at design — `A45·MFP*(Tt45_d)`.
    pub w45d: f64,
    /// The one row's area, normalised so the design flow (at `pt45 = 1`) is 1.
    pub one: f64,
    /// The two rows' areas, normalised the same way.
    pub two: [f64; 2],
    /// The LP turbine's design efficiency — one of the shortcut's exact conditions ([`TwoSpoolCore::law_is_pin`]).
    pub eta: f64,
}

/// The temperature at a pressure-ratio function value, POLISHED: the gas's own inversion stops after a Newton step under
/// 1e−11 of `T`, which leaves a jitter of that size — harmless in one number, but in a flow DIFFERENCE it reaches
/// ~1e−10 and the outer loops cannot settle through it (anchor C3: the row laws were rough at 1e−10, the ellipse at
/// 1e−16). Two more Newton steps on `ln pr` (`d ln pr/dT = cp/(R·T)`) take it to the last digits.
fn t_from_pr_fine(gas: &Gas, pr: f64, f: f64) -> Result<f64, Abort> {
    let mut t = gas.try_t_from_pr_t(pr, f)?;
    for _ in 0..2 {
        t -= (gas.pr_t(t, f) / pr).ln() * gas.r_t_at(f) * t / gas.cp_t_at(t, f);
    }
    Ok(t)
}

/// The temperature at an enthalpy, polished the same way (`dh/dT = cp`).
fn t_from_h_fine(gas: &Gas, h: f64, f: f64) -> f64 {
    let mut t = gas.t_from_h_t(h, f);
    for _ in 0..2 {
        t -= (gas.h_t(t, f) - h) / gas.cp_t_at(t, f);
    }
    t
}

/// `Tt_out` at a trial ratio — [`TwoSpoolCore::try_tau_of`]'s body without its census counter, polished.
fn tau_free(gas: &Gas, tt_in: f64, f: f64, pi_t: f64, eta: f64) -> Result<f64, Abort> {
    let tt_outs = t_from_pr_fine(gas, gas.pr_t(tt_in, f) * pi_t, f)?;
    let dh_ideal = gas.h_t(tt_in, f) - gas.h_t(tt_outs, f);
    Ok(t_from_h_fine(gas, gas.h_t(tt_in, f) - eta * dh_ideal, f))
}

/// The mass flux `ρV` through a convergent row from total `(p, t)` to back-pressure `pb`: isentropic, capped at the
/// gas's own sonic point. Above `pb/p = 0.6` no gas with `γ ≥ 1.1` is sonic, so the sonic search is skipped there.
fn flux(gas: &Gas, f: f64, p: f64, t: f64, pb: f64) -> Result<f64, Abort> {
    let r = gas.r_t_at(f);
    if pb < 0.6 * p {
        let (ts_star, p_star, v_star) = try_sonic_throat(gas, t, p, f)?;
        if pb <= p_star {
            return Ok(p_star / (r * ts_star) * v_star);
        }
    }
    let ts = t_from_pr_fine(gas, gas.pr_t(t, f) * pb / p, f)?;
    let v = (2.0 * (gas.h_t(t, f) - gas.h_t(ts, f))).max(0.0).sqrt();
    Ok(pb / (r * ts) * v)
}

/// The LP turbine's two refusals that mean "its inlet pressure is too low for the gas to leave": rung 97's unchoked
/// nozzle and rung 102's lit afterburner (its lean wall). Every other failure is a real one and is passed on.
fn passes_nothing(e: &Abort) -> bool {
    e.0.starts_with("rung-97: the nozzle cannot pass") || e.0.contains("the hot gas cannot get out")
}

/// A bracketed Illinois search with no census counter (the crate's shared one is counted by older gates), started at `x0` (a design value, never a previous call's root, so a point does not depend on what
/// ran before it). Every residual here is positive at its lower end and falls through its root, so `f(x0)`'s sign says
/// which way to step; the step doubles until the sign changes or the end `[lo, hi]` is reached.
fn illinois_from(f: impl Fn(f64) -> Result<f64, Abort>, x0: f64, step: f64, lo: f64, hi: f64, tol: f64) -> Result<f64, Abort> {
    let x0 = x0.clamp(lo, hi);
    let (mut a, mut fa) = (x0, f(x0)?);
    if fa == 0.0 {
        return Ok(a);
    }
    let up = fa > 0.0;
    let mut s = step;
    loop {
        let b = if up { (a + s).min(hi) } else { (a - s).max(lo) };
        let fb = f(b)?;
        if fa * fb <= 0.0 {
            return illinois_known(f, a, fa, b, fb, tol);
        }
        if b == hi || b == lo {
            return Err(Abort(format!("rung-104: the turbine flow law does not bracket from {x0} on [{lo}, {hi}]")));
        }
        a = b;
        fa = fb;
        s *= 2.0;
    }
}

/// The search on a known bracket `[a, b]` with its two values.
pub(crate) fn illinois_known(f: impl Fn(f64) -> Result<f64, Abort>, mut a: f64, mut fa: f64, mut b: f64, mut fb: f64, tol: f64)
    -> Result<f64, Abort> {
    if fb == 0.0 {
        return Ok(b);
    }
    if fa == 0.0 {
        return Ok(a);
    }
    if fa * fb > 0.0 {
        return Err(Abort(format!("rung-104: the turbine flow law has no sign change on [{a}, {b}] ({fa:e}, {fb:e})")));
    }
    for _ in 0..200 {
        let c = (a * fb - b * fa) / (fb - fa);
        let fc = f(c)?;
        // To the last digits: a root found only to the outer loops' own tolerance leaves them a jitter of that size and
        // they never settle (anchor C3: rung 38's loop ran to its cap of 200 under two rows).
        let tiny = 4.0 * f64::EPSILON * a.abs().max(b.abs());
        if (b - a).abs() <= tol.max(tiny) || fc == 0.0 || c == a || c == b {
            return Ok(c);
        }
        if fc * fb < 0.0 {
            a = b;
            fa = fb;
        } else {
            fa *= 0.5;
        }
        b = c;
        fb = fc;
    }
    Ok(b)
}

impl LpCapacity {
    /// Capture at the design point: `(Tt45, pt45)`, `pt5`, the design mixture and LP efficiency, and `A45`.
    #[allow(clippy::too_many_arguments)]
    pub fn capture(gas: &Gas, f: f64, pt4: f64, tt45: f64, pt45: f64, pt5: f64, eta_lpt: f64, a45: f64) -> LpCapacity {
        let xd = pt5 / pt45;
        let w45d = a45 * try_choked_mfp(gas, tt45, f).expect("design MFP*");
        let one = 1.0 / flux(gas, f, 1.0, tt45, xd).expect("design one-row flux");
        let r = xd.sqrt();
        let a1 = 1.0 / flux(gas, f, 1.0, tt45, r).expect("design row-1 flux");
        let t1 = tau_free(gas, tt45, f, r, eta_lpt).expect("design row-1 expansion");
        let a2 = 1.0 / flux(gas, f, r, t1, xd).expect("design row-2 flux");
        LpCapacity { xd, phd: pt45 / pt4, td: tt45, w45d, one, two: [a1, a2], eta: eta_lpt }
    }
}

impl TwoSpoolCore {
    /// The LP turbine's flow parameter `W45 = ṁ√Tt45/pt45` at its expansion `x = pt5/pt45`, under the core's law.
    pub fn w45(&self, gas: &Gas, x: f64, tt45: f64, f45: f64, eta_lpt: f64) -> Result<f64, Abort> {
        let c = self.lp_capacity;
        census::bump(2);
        match self.turbine_law.get() {
            TurbineLaw::Pin => Ok(self.a45_pin() * try_choked_mfp(gas, tt45, f45)?),
            TurbineLaw::Ellipse => Ok(c.w45d * (1.0 - x * x).max(0.0).sqrt() / (1.0 - c.xd * c.xd).sqrt()),
            TurbineLaw::OneRow => Ok(c.w45d * c.one * flux(gas, f45, 1.0, tt45, x)? * powp(tt45 / c.td, 0.5)),
            TurbineLaw::TwoRows => {
                let [a1, a2] = c.two;
                // The interstage pressure where the two rows pass the same flow (pt45 = 1).
                let resid = |p1: f64| -> Result<f64, Abort> {
                    census::bump(3);
                    let t1 = tau_free(gas, tt45, f45, p1, eta_lpt)?;
                    Ok(a1 * flux(gas, f45, 1.0, tt45, p1)? - a2 * flux(gas, f45, p1, t1, x)?)
                };
                // Started where the design put the interstage pressure, as a share of this drop.
                let sd = (c.xd.sqrt() - c.xd) / (1.0 - c.xd);
                let (lo, hi) = (x + 1e-9 * (1.0 - x), 1.0 - 1e-9 * (1.0 - x));
                let p1 = illinois_from(resid, x + sd * (1.0 - x), 0.02 * (1.0 - x), lo, hi, 0.0)?;
                Ok(c.w45d * a1 * flux(gas, f45, 1.0, tt45, p1)? * powp(tt45 / c.td, 0.5))
            }
        }
    }

    /// THE SHORTCUT — where a law provably IS the pin, the pin's own solve is used. On the perfect gas (`MFP*` a
    /// constant) with the LP turbine at its design efficiency, the choked nozzle fixes the LP turbine's ratio at its
    /// design value whatever the vane does (anchor G2), so every law's `W45` there is the design one and both turbines
    /// sit exactly where the pin puts them. The law's own search lands within ~1e−14 of that point; the shortcut lands ON
    /// it, so such points are the old model bit for bit and readings built on exact ties or kinks (rungs 58, 66, 68, 69,
    /// 72–75) read the same. Exact conditions only, no tolerance: the hot section calorically perfect, `η_LPT` equal to
    /// the design value, no afterburner lit — and, for a pair, the LP step's result equal to the pin's choked root (the
    /// nozzle choked). Off with `law_shortcut` (gate R2 compares the law's own search with the pin).
    /// A lit afterburner behind a HELD throat — the one case where the LP turbine reads station 7, not the dry nozzle
    /// (`unchoked_lp::try_lp_turbine`'s own dispatch; matched, it keeps its unlit place).
    pub fn lit_held(&self) -> bool { self.lit_ab().is_some_and(|x| x.s_now().is_some()) }

    pub fn law_is_pin(&self, gas: &Gas, eta_lpt: f64) -> bool {
        self.law_shortcut.get() && gas.hot_is_cpg() && eta_lpt == self.lp_capacity.eta && !self.lit_held()
    }

    /// The LP turbine with the nozzle CHOKED: `(π_LPT, τ_LPT, Tt5)`. `Pin` is rung 38's (★-LP), called literally.
    pub fn try_lp_choked(&self, gas: &Gas, tt45: f64, f45: f64, eta: f64) -> Result<(f64, f64, f64), Abort> {
        if self.turbine_law.get() == TurbineLaw::Pin || self.law_is_pin(gas, eta) {
            return self.try_solve_choked_turbine(gas, tt45, f45, self.a45_pin(), self.a8, self.pi_n, eta);
        }
        let resid = |x: f64| -> Result<f64, Abort> {
            let tt5 = tau_free(gas, tt45, f45, x, eta)?;
            let tau = tt5 / tt45;
            Ok(self.w45(gas, x, tt45, f45, eta)? - self.a8 * self.pi_n * try_choked_mfp(gas, tt5, f45)? * x / powp(tau, 0.5))
        };
        let x = illinois_from(resid, self.lp_capacity.xd, 0.02, 0.05, 0.9995, 0.0)?;
        let tt5 = tau_free(gas, tt45, f45, x, eta)?;
        Ok((x, tt5 / tt45, tt5))
    }

    /// BOTH turbines: `((π_HPT, τ_HPT, Tt45), (π_LPT, τ_LPT, Tt5))`. `lp(Tt45, π_HPT)` is the variant's own LP step and
    /// returns its result with the LP efficiency it used. `Pin`: rung 38's (★-HP) and then `lp`, literally the old
    /// order. A law: the HP residual `A4·MFP*(Tt4)·√τ_HPT/π_HPT = W45(x, Tt45)` reads the LP turbine's own `x`.
    #[allow(clippy::type_complexity)]
    pub fn try_turbine_pair(
        &self, gas: &Gas, tt4: f64, f: f64, eta_hpt: f64,
        lp: impl Fn(f64, f64) -> Result<((f64, f64, f64), f64), Abort>,
    ) -> Result<((f64, f64, f64), (f64, f64, f64)), Abort> {
        if self.turbine_law.get() == TurbineLaw::Pin {
            let hp = self.try_solve_choked_turbine(gas, tt4, f, self.a4, self.a45_pin(), 1.0, eta_hpt)?;
            let (l, _) = lp(hp.2, hp.0)?;
            return Ok((hp, l));
        }
        // The shortcut: the pin's pair, kept only if the LP step landed on the pin's own choked root.
        if self.law_shortcut.get() && gas.hot_is_cpg() && !self.lit_held() {
            let hp = self.try_solve_choked_turbine(gas, tt4, f, self.a4, self.a45_pin(), 1.0, eta_hpt)?;
            if let Ok((l, eta_l)) = lp(hp.2, hp.0) {
                if self.law_is_pin(gas, eta_l) {
                    let c = self.try_solve_choked_turbine(gas, hp.2, f, self.a45_pin(), self.a8, self.pi_n, eta_l)?;
                    if c.0.to_bits() == l.0.to_bits() {
                        return Ok((hp, l));
                    }
                }
            }
        }
        let a4m = self.a4 * try_choked_mfp(gas, tt4, f)?;
        let ph = self.try_joint_hp(gas, a4m, tt4, |ph| {
            let tt45 = tau_free(gas, tt4, f, ph, eta_hpt)?;
            Ok((tt45, lp(tt45, ph).map(|((x, _, _), eta_l)| (x, eta_l)), f))
        })?;
        let (tau_h, tt45) = self.try_tau_of(gas, tt4, f, ph, eta_hpt)?;
        let (l, _) = lp(tt45, ph)?;
        Ok(((ph, tau_h, tt45), l))
    }

    /// The joint HP search under a law: the `π_HPT` at which the vane's `W45(x, Tt45)` passes what the HP nozzle
    /// guide vane delivers, `k·√(Tt45/Tt4)/π` (`k = A4·MFP*(Tt4)`, times the coolants' share when cooled). `at(π)` gives
    /// `Tt45`, the LP step's `(x, η_LPT)` or its refusal, and the LP mixture.
    ///
    /// A trial HP expansion the LP turbine cannot follow (its inlet pressure too low for the gas to leave —
    /// [`passes_nothing`]) passes nothing, every law's `W45` at `x → 1`, so the residual there is the HP side alone,
    /// positive. The search can then stop on that region's EDGE rather than a root, so the root is checked at the end.
    #[allow(clippy::type_complexity)]
    pub fn try_joint_hp(
        &self, gas: &Gas, k: f64, tt4: f64, at: impl Fn(f64) -> Result<(f64, Result<(f64, f64), Abort>, f64), Abort>,
    ) -> Result<f64, Abort> {
        let refused: std::cell::RefCell<Option<Abort>> = std::cell::RefCell::new(None);
        census::bump(0);
        let resid = |ph: f64| -> Result<f64, Abort> {
            census::bump(1);
            let (tt45, lp, f45) = at(ph)?;
            let hp = k * powp(tt45 / tt4, 0.5) / ph;
            match lp {
                Ok((x, eta_l)) => Ok(hp - self.w45(gas, x, tt45, f45, eta_l)?),
                Err(e) if passes_nothing(&e) => {
                    *refused.borrow_mut() = Some(e);
                    Ok(hp)
                }
                Err(e) => Err(e),
            }
        };
        // No sign change at all with the LP turbine refusing somewhere: that refusal is the reason, in its own words.
        let ph = match illinois_from(&resid, self.lp_capacity.phd, 0.02, 0.01, 0.99, 0.0) {
            Ok(ph) => ph,
            Err(e) => return Err(refused.into_inner().unwrap_or(e)),
        };
        let (tt45, lp, f45) = at(ph)?;
        let (x, eta_l) = lp?;
        let r = k * powp(tt45 / tt4, 0.5) / ph - self.w45(gas, x, tt45, f45, eta_l)?;
        if r.abs() > 1e-8 * k {
            return Err(refused.into_inner().unwrap_or_else(|| {
                Abort(format!("rung-104: the two turbines have no joint root at Tt4={tt4:.1} (residual {r:e})"))
            }));
        }
        Ok(ph)
    }
}
