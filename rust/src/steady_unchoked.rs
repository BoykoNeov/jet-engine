//! RUNG 103 — THE STEADY TWO-SHAFT MATCHERS BELOW CHOKE: the refusal becomes a solve.
//!
//! Anchor (the device and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung103-anchor-steady-unchoked.md`. Spec: `docs/rung103-spec.md`.
//!
//! # The equation
//!
//! Rung 97's, on the steady matchers. Below the nozzle's unchoke boundary only `A45` stays choked, so the LP turbine's
//! `π_LPT` is the root of rung 33's `(★★)` one throat downstream — [`crate::unchoked_lp::try_lp_turbine`] — at
//! `pt45 = π_HPT·pt4` and the flight's `p0`. The steady matchers (rungs 38/39/42/90/91/92) close `pt4` in their joint
//! `(f, pt4)` loop, so each pass hands the LP turbine the `pt4` it started with: the root and the loop converge
//! together.
//!
//! # The switch and the dispatch
//!
//! [`TwoSpoolCore::below_choke`] is `false` by default (the user's decision: off in the old code, on in the pages).
//! Each matcher runs its PINNED solve and rebuild unchanged; only if that fails — at the unchoked guard or earlier — and
//! the switch is on does it re-solve once with the LP turbine free. A point whose pinned path succeeds never enters
//! the branch, so every choked point is the switch-off point bit for bit (anchor R2). Rung 38's matcher is infallible:
//! it re-solves at its guard only, and its branch's failures panic in their own words.
//!
//! # Below idle
//!
//! Inside the branch a point whose core specific thrust or net thrust is not positive is refused in words (rung 33's
//! precedent) — before the shared scoring, which cannot take a negative thrust. With [`TwoSpoolCore::idle_rule`] at
//! `Refuse` (sandbox plan § 38 — the Blades and Cooling views) a CHOKED point with no positive net thrust is refused
//! too, so those views hold one below-idle rule.

use crate::gas::{Abort, Gas};
use crate::two_spool::TwoSpoolCore;

/// How a steady cascade solves its LP turbine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SteadyLp {
    /// Rung 38's two-choked-throat pin — every shipped path.
    Pin,
    /// The below-choke re-solve: rung 97's root at `pt45 = π_HPT·pt4`, `pt4` the joint loop's current pass.
    Free { pt4: f64, p0: f64 },
    /// The LP turbine held at this `π_LPT` — the independent-root gate's outer search only (anchor R3). The
    /// unchoked guard passes and nothing is refused; never set on a page or the CLI.
    Held(f64),
}

/// Clears the re-solve state on every exit, panics included (anchor R8).
struct Restore<'a> {
    core: &'a TwoSpoolCore,
    lp: SteadyLp,
    dispatching: bool,
}

impl Drop for Restore<'_> {
    fn drop(&mut self) {
        self.core.steady_lp.set(self.lp);
        self.core.dispatching.set(self.dispatching);
    }
}

/// The LP turbine in a steady cascade: `(π_LPT, τ_LPT, Tt5)`. `pi_hpt` is `pt45/pt4` (cooled or not).
pub fn try_lp_steady(
    core: &TwoSpoolCore, gas: &Gas, tt45: f64, f45: f64, eta_lpt: f64, pi_hpt: f64,
) -> Result<(f64, f64, f64), Abort> {
    match core.steady_lp.get() {
        SteadyLp::Pin => core.try_solve_choked_turbine(gas, tt45, f45, core.a45, core.a8, core.pi_n, eta_lpt),
        SteadyLp::Free { pt4, p0 } => crate::unchoked_lp::try_lp_turbine(core, gas, tt45, f45, eta_lpt, pi_hpt * pt4, p0),
        SteadyLp::Held(pi) => {
            let (tau, tt5) = core.try_tau_of(gas, tt45, f45, pi, eta_lpt)?;
            Ok((pi, tau, tt5))
        }
    }
}

/// A joint-loop pass records the `pt4` it starts from while a re-solve runs.
pub fn track_pt4(core: &TwoSpoolCore, pt4: f64) {
    if let SteadyLp::Free { p0, .. } = core.steady_lp.get() {
        core.steady_lp.set(SteadyLp::Free { pt4, p0 });
    }
}

/// At a matcher's unchoked guard: `Ok(true)` — refuse with the matcher's old words (the pin); `Ok(false)` — carry on
/// to the rebuild's scoring; `Err` — below idle, inside the re-solve. `sp_core` is the core's specific thrust at the
/// rebuilt exit (the unchoked nozzle has no pressure term).
pub fn refuse_unchoked(core: &TwoSpoolCore, tt4: f64, sp_core: f64) -> Result<bool, Abort> {
    match core.steady_lp.get() {
        SteadyLp::Pin => Ok(!core.pin_past_choke.get()),
        SteadyLp::Free { .. } if !(sp_core > 0.0) => Err(below_idle(tt4, sp_core)),
        _ => Ok(false),
    }
}

/// A matcher's body behind the switch: the pinned body; if it fails, the switch is on, and no outer call is already
/// dispatching (a body that forwards to another matcher's), the body again with the LP turbine free. `thrust` reads
/// the result's net thrust for the below-idle check.
pub fn dispatch<T>(
    core: &TwoSpoolCore, p0: f64, tt4: f64, body: impl Fn() -> Result<T, Abort>, thrust: impl Fn(&T) -> f64,
) -> Result<T, Abort> {
    // Plan § 38: with the below-idle rule at `Refuse`, the OUTER call refuses any point it returns with no positive net
    // thrust — the choked ones the branch below never judges. A nested call (inside a pinned body, or inside the
    // re-solve) is left to its caller, which judges the whole engine's thrust.
    let outer = !core.dispatching.get() && core.steady_lp.get() == SteadyLp::Pin;
    let x = dispatch_free(core, p0, tt4, body, &thrust)?;
    if outer && core.idle_rule.get() == crate::two_spool::IdleRule::Refuse {
        let t = thrust(&x);
        if !(t > 0.0) {
            return Err(below_idle_choked(tt4, t));
        }
    }
    Ok(x)
}

/// [`dispatch`] before the below-idle floor: rung 103's switch alone.
fn dispatch_free<T>(
    core: &TwoSpoolCore, p0: f64, tt4: f64, body: impl Fn() -> Result<T, Abort>, thrust: &impl Fn(&T) -> f64,
) -> Result<T, Abort> {
    if !core.below_choke.get() || core.dispatching.get() || core.steady_lp.get() != SteadyLp::Pin {
        return body();
    }
    let pinned = {
        let _r = Restore { core, lp: SteadyLp::Pin, dispatching: false };
        core.dispatching.set(true);
        body()
    };
    if pinned.is_ok() {
        return pinned;
    }
    let _r = Restore { core, lp: SteadyLp::Pin, dispatching: false };
    core.steady_lp.set(SteadyLp::Free { pt4: f64::NAN, p0 });
    let x = body()?;
    let t = thrust(&x);
    if t > 0.0 { Ok(x) } else { Err(below_idle(tt4, t)) }
}

/// Rung 38's infallible matcher at its guard: the branch's point, or a panic in its own words.
pub fn rung38_below_choke<T>(
    core: &TwoSpoolCore, p0: f64, tt4: f64, body: impl Fn() -> T, thrust: impl Fn(&T) -> f64,
) -> T {
    let _r = Restore { core, lp: SteadyLp::Pin, dispatching: false };
    core.steady_lp.set(SteadyLp::Free { pt4: f64::NAN, p0 });
    let x = body();
    let t = thrust(&x);
    if !(t > 0.0) {
        panic!("{}", below_idle(tt4, t).0);
    }
    x
}

/// Rung 33's sub-idle refusal, on two shafts.
pub fn below_idle(tt4: f64, sp: f64) -> Abort {
    Abort(format!(
        "rung-103: below idle at Tt4={tt4:.0}: with the nozzle unchoked the engine makes more drag than thrust here \
         (net {sp:.1} N per kg/s) — below this throttle it does not run usefully"))
}

/// The below-idle floor's refusal (sandbox plan § 38) — a point the pinned path solved, nozzle choked. The same
/// lead words as [`below_idle`], so every reader that sorts on them sorts it the same way.
pub fn below_idle_choked(tt4: f64, thrust: f64) -> Abort {
    Abort(format!(
        "rung-103: below idle at Tt4={tt4:.0}: the engine makes more drag than thrust here (net {thrust:.1} N) — \
         below this throttle it does not run usefully"))
}
