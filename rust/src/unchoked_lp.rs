//! RUNG 97 — THE UNCHOKED NOZZLE ON TWO SHAFTS: below the unchoke boundary the LP turbine's expansion is the unknown.
//!
//! Anchor (the device and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung97-anchor-unchoked-lp.md`. Spec: `docs/rung97-spec.md`.
//!
//! # The equation
//!
//! Rung 38 pinned the LP turbine by two choked throats, `A45` and the nozzle's `A8` — pure geometry. Below the nozzle's
//! unchoke boundary only `A45` stays choked; the convergent nozzle expands fully (`p9 = p0`, `M9 < 1`) and passes
//! `A8·ρ9·V9` at the ACTUAL `pt9/p0`. So `π_LPT` becomes the unknown that matches the two:
//!
//! ```text
//! resid(π_LPT) = A45·pt45·MFP*(Tt45, f45)/√Tt45 − A8·ρ9·V9 = 0,   pt9 = π_n·π_LPT·pt45
//! ```
//!
//! rung 33's `(★★)` one throat downstream. At an instant the compressors, the HP nozzle guide vane and the HP turbine
//! do not read the LP turbine (rung 38's triangular cascade), so the branch changes the LP shaft's power only.
//!
//! # Reduce
//!
//! The pin is solved first, exactly as before; while its nozzle is choked (the tails' own test, `p9 > p0 + 1e−6`) it is
//! returned untouched — every choked instant bit for bit.

use crate::components::{try_choked_mfp, Nozzle};
use crate::gas::{powp, Abort, FlowState, Gas};
use crate::two_spool::TwoSpoolCore;

/// The top of the bracket: the LP turbine nearly unexpanded (rung 33's high march starts here too).
pub const PI_TOP: f64 = 0.9995;

/// The LP turbine at an instant: `(π_LPT, τ_LPT, Tt5)` — the pin while the nozzle stays choked, else the subsonic root.
/// `pt45` is the LP turbine's inlet total pressure (`π_HPT·pt4`), `p0` the air the nozzle pushes against.
#[allow(clippy::too_many_arguments)]
pub fn try_lp_turbine(
    base: &TwoSpoolCore, gas: &Gas, tt45: f64, f45: f64, eta_lpt: f64, pt45: f64, p0: f64,
) -> Result<(f64, f64, f64), Abort> {
    let pin = base.try_solve_choked_turbine(gas, tt45, f45, base.a45, base.a8, base.pi_n, eta_lpt)?;
    if base.lp_pinned.get() {
        return Ok(pin);
    }
    let nozzle = Nozzle::convergent(p0, base.pi_n);
    let at = |pi: f64, tt5: f64| nozzle.try_apply(&FlowState { tt: tt5, pt: pi * pt45, mdot: 1.0, far: f45 }, gas);
    if matches!(at(pin.0, pin.2), Ok(e) if e.p9 > p0 + 1e-6) {
        return Ok(pin);
    }
    let supply = base.a45 * pt45 * try_choked_mfp(gas, tt45, f45)? / powp(tt45, 0.5);
    let resid = |pi: f64| -> Result<(f64, f64, f64), Abort> {
        let (tau, tt5) = base.try_tau_of(gas, tt45, f45, pi, eta_lpt)?;
        let demand = match at(pi, tt5) {
            Ok(e) => base.a8 * (e.p9 / (gas.r_t_at(f45) * e.t9)) * e.v9,
            Err(_) => 0.0,   // the nozzle cannot push the gas out at all
        };
        Ok((supply - demand, tau, tt5))
    };
    if resid(PI_TOP)?.0 >= 0.0 {
        return Err(Abort(format!(
            "rung-97: the nozzle cannot pass the LP turbine's flow against {p0:.0} Pa even with the turbine nearly \
             unexpanded (pi_LPT {PI_TOP}) — the engine has no operating point at this instant")));
    }
    let (mut lo, mut hi) = (pin.0, PI_TOP);
    for _ in 0..TwoSpoolCore::MAX {
        let mid = 0.5 * (lo + hi);
        if resid(mid)?.0 > 0.0 { lo = mid } else { hi = mid }
        if hi - lo <= TwoSpoolCore::TOL { break }
    }
    let pi = 0.5 * (lo + hi);
    let (_, tau, tt5) = resid(pi)?;
    Ok((pi, tau, tt5))
}
