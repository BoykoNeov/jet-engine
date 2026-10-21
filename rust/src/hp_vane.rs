//! RUNG 105 — THE HP TURBINE'S VANE IS NOT CHOKED AT A NORMAL REACTION: a reaction law beside rungs 31/38's pin on `A4`.
//!
//! Anchor (the device and every pre-registered bar, committed before any of this code existed):
//! `docs/plans/rung105-anchor-hp-vane.md`. Spec: `docs/rung105-spec.md`.
//!
//! # The equation
//!
//! Rung 31 pinned the turbine's flow parameter at its nozzle guide vane's CHOKED value, `W4 = ṁ√Tt4/pt4 = A4·MFP*(Tt4)`,
//! and rung 38 kept it on two shafts — the pin every off-design solver takes the engine's airflow from. The vane is choked
//! only if its exit kinetic energy reaches the sonic one: with a repeating stage of reaction `R`, `c1²/2 = (1 − R)·Δh0`,
//! so it chokes iff `R ≤ R* = 1 − (c*²/2)/Δh0` — 0.16–0.20 on these engines (anchor G1), where published single stages
//! run 0.25–0.5. The law gives `W4` as a function of the HP turbine's own expansion `y = pt_out/pt4`:
//!
//! * `Rows { reaction }` — vane and rotor as two convergent rows in series (rung 104's two-row model: the gas's own
//!   isentropic flux, capped at its sonic point, the velocity dumped between them), the design split set by the TEXTBOOK
//!   reaction — the vane isentropic, its exit kinetic energy `(1 − R)` of the stage's actual drop — so the vane is choked
//!   at design exactly when `R ≤ R*`. Where a vane choked at design is choked here, the pin's own expression is returned:
//!   at a low enough reaction the law IS the pin wherever the vane stays choked.
//! * `Pin` — the choked vane: the OLD CODE, called literally. **The default** (the user's choice, once the law's cost was
//!   measured: anchor G5); the cargo feature `hp-law-default` makes the law at 0.3 the default instead (gate R1b).
//!
//! # Where it enters
//!
//! Every reader's `A4` becomes the EFFECTIVE `A4 = W4(y)/MFP*(Tt4)` ([`HpCapacity::a4_at`]), which is `A4` itself, bit
//! for bit, on the pin, on a choked-at-design vane still choked, and on the shortcut — so the old arithmetic runs
//! unchanged there. The one-shaft solves live in `matcher.rs` / `spool.rs`; the two-shaft pair in `turbine_law.rs`.

use crate::components::{try_choked_mfp, try_sonic_throat};
use crate::gas::{powp, Abort, Gas};
use crate::turbine_law::{flux, illinois_from, t_from_h_fine, t_from_pr_fine, tau_free};

/// The reaction the law runs at when switched on (the user's choice, 2026-10-20): the middle of the published range for
/// one high-work stage.
pub const REACTION_DEFAULT: f64 = 0.3;

/// The pages' reaction range.
pub const REACTION_MIN: f64 = 0.05;
pub const REACTION_MAX: f64 = 0.6;

/// What sets the HP turbine's flow parameter `W4 = ṁ√Tt4/pt4`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HpVane {
    /// The choked vane — the old code, literally.
    Pin,
    /// Vane and rotor in series, the vane's exit kinetic energy `(1 − reaction)` of the stage's drop at design.
    Rows { reaction: f64 },
}

impl HpVane {
    /// The law at the default reaction.
    pub fn law() -> Self { HpVane::Rows { reaction: REACTION_DEFAULT } }

    /// The pages' request keys: `hp_vane` ("pin" / "rows") and, for the law, `reaction` — written only off their
    /// defaults, so every default reply keeps its shape.
    pub fn push_json(self, j: &mut crate::visuals::Json) {
        if let crate::visuals::Json::Obj(kv) = j {
            if self == HpVane::default() {
                return;
            }
            match self {
                HpVane::Pin => kv.push(("hp_vane".into(), crate::visuals::Json::from("pin"))),
                HpVane::Rows { reaction } => {
                    kv.push(("hp_vane".into(), crate::visuals::Json::from("rows")));
                    kv.push(("reaction".into(), crate::visuals::Json::from(reaction)));
                }
            }
        }
    }

    /// Read the keys back; a missing one keeps `base`.
    pub fn from_json(j: &crate::visuals::Json, base: HpVane) -> Result<HpVane, String> {
        use crate::visuals::Json;
        let r = match j.get("reaction") {
            None => None,
            Some(Json::Float(x)) => Some(*x),
            Some(Json::Int(n)) => Some(*n as f64),
            Some(other) => return Err(format!("setting \"reaction\" must be a number, got {other:?}")),
        };
        let law = match j.get("hp_vane") {
            None => base,
            Some(Json::Str(s)) if s == "pin" => HpVane::Pin,
            Some(Json::Str(s)) if s == "rows" => HpVane::law(),
            Some(other) => return Err(format!("unknown hp_vane {other:?} (\"pin\" or \"rows\")")),
        };
        Ok(match (law, r) {
            (HpVane::Rows { .. }, Some(reaction)) => HpVane::Rows { reaction },
            (l, _) => l,
        })
    }

    /// The pages' range, in words.
    pub fn check(self) -> Result<(), String> {
        match self {
            HpVane::Rows { reaction } if !(REACTION_MIN..=REACTION_MAX).contains(&reaction) => Err(format!(
                "The HP turbine stage's reaction must be between {REACTION_MIN} and {REACTION_MAX} (the share of the \
                 stage's drop its rotor takes; published single stages run 0.25–0.5)")),
            _ => Ok(()),
        }
    }
}

/// The old pin — or, built with `hp-law-default`, the law at 0.3 (the whole gate under the law: gate R1b).
impl Default for HpVane {
    fn default() -> Self {
        if cfg!(feature = "hp-law-default") { HpVane::law() } else { HpVane::Pin }
    }
}

/// The law's design capture (zero new constants but the reaction).
#[derive(Clone, Copy, Debug)]
pub struct HpCapacity {
    pub law: HpVane,
    /// The design mixture, `Tt4`, HP-turbine expansion `pt_out/pt4` and efficiency.
    pub fd: f64,
    pub td: f64,
    pub yd: f64,
    pub eta: f64,
    /// `A4`, and `W4` at design, `A4·MFP*(Tt4_d)`.
    pub a4: f64,
    pub w4d: f64,
    /// The interstage pressure at design (`pt4 = 1`) and the two rows' areas, normalised so the design flow is 1.
    pub p1d: f64,
    pub rows: [f64; 2],
    /// The vane choked at design: then wherever it stays choked the law IS the pin, and returns the pin's expression.
    pub choked_d: bool,
}

/// The vane's sonic point at total `(1, Tt4)`, found ONCE for a search over the turbine's expansion — the gas's sonic
/// search is the dearest call in the law (rung 104's `flux` repeats it at every call below `0.6` of its inlet pressure).
///
/// It also carries the WARM START of the interstage search: within one search over `y` at one `(Tt4, f)` each
/// interstage solve starts where the last one landed (as a share of its drop), not at the design share — anchor C2's
/// count found ~11 interstage residuals a vane flow from cold. The sequence of calls inside one search is fixed by its
/// inputs, so a point still does not depend on anything run before it.
#[derive(Clone, Debug)]
pub struct Vane {
    pub p_star: f64,
    pub flux_star: f64,
    last_share: std::cell::Cell<f64>,
}

impl Vane {
    pub fn at(gas: &Gas, tt4: f64, f: f64) -> Result<Vane, Abort> {
        census::bump(0);
        let (ts, ps, vs) = try_sonic_throat(gas, tt4, 1.0, f)?;
        Ok(Vane { p_star: ps, flux_star: ps / (gas.r_t_at(f) * ts) * vs, last_share: std::cell::Cell::new(f64::NAN) })
    }

    /// The vane's isentropic flux from total `(1, Tt4)` to `p1`, capped at its sonic point.
    pub fn flux(&self, gas: &Gas, f: f64, tt4: f64, p1: f64) -> Result<f64, Abort> {
        if p1 <= self.p_star {
            return Ok(self.flux_star);
        }
        let ts = t_from_pr_fine(gas, gas.pr_t(tt4, f) * p1, f)?;
        let v = (2.0 * (gas.h_t(tt4, f) - gas.h_t(ts, f))).max(0.0).sqrt();
        Ok(p1 / (gas.r_t_at(f) * ts) * v)
    }
}

impl HpCapacity {
    /// Capture at the design point.
    #[allow(clippy::too_many_arguments)]
    pub fn capture(gas: &Gas, f: f64, tt4: f64, yd: f64, eta: f64, a4: f64, law: HpVane) -> HpCapacity {
        let w4d = a4 * try_choked_mfp(gas, tt4, f).expect("design MFP*");
        let mut c = HpCapacity { law, fd: f, td: tt4, yd, eta, a4, w4d, p1d: f64::NAN, rows: [f64::NAN; 2],
                                 choked_d: true };
        if let HpVane::Rows { reaction } = law {
            assert!((0.0..1.0).contains(&reaction), "rung-105: the reaction must lie in [0, 1), got {reaction}");
            // THE TEXTBOOK REACTION: the vane's exit kinetic energy is (1 − R) of the stage's ACTUAL drop Δh0 (a
            // repeating stage, the vane isentropic) — so the vane chokes exactly where G1's `R*` says.
            let h4 = gas.h_t(tt4, f);
            let tys = t_from_pr_fine(gas, gas.pr_t(tt4, f) * yd, f).expect("design expansion");
            let hs1 = h4 - (1.0 - reaction) * eta * (h4 - gas.h_t(tys, f));
            let t1s = t_from_h_fine(gas, hs1, f);
            let p1 = gas.pr_t(t1s, f) / gas.pr_t(tt4, f);
            let a1 = 1.0 / flux(gas, f, 1.0, tt4, p1).expect("design vane flux");
            let t1 = tau_free(gas, tt4, f, p1, eta).expect("design vane expansion");
            let a2 = 1.0 / flux(gas, f, p1, t1, yd).expect("design rotor flux");
            c.p1d = p1;
            c.rows = [a1, a2];
            c.choked_d = p1 <= Vane::at(gas, tt4, f).expect("design sonic point").p_star;
        }
        c
    }

    /// The same design, another law.
    pub fn with_law(&self, gas: &Gas, law: HpVane) -> HpCapacity {
        HpCapacity::capture(gas, self.fd, self.td, self.yd, self.eta, self.a4, law)
    }

    /// `(W4, the vane choked here)` at the HP turbine's expansion `y = pt_out/pt4`.
    pub fn w4(&self, gas: &Gas, y: f64, tt4: f64, f: f64, eta: f64) -> Result<(f64, bool), Abort> {
        if self.law == HpVane::Pin {
            return Ok((self.a4 * try_choked_mfp(gas, tt4, f)?, true));
        }
        self.w4_with(&Vane::at(gas, tt4, f)?, gas, y, tt4, f, eta)
    }

    /// The interstage pressure (`pt4 = 1`) where the two rows pass the same flow at the turbine's expansion `y`.
    pub fn p1_at(&self, v: &Vane, gas: &Gas, y: f64, tt4: f64, f: f64, eta: f64) -> Result<f64, Abort> {
        let [a1, a2] = self.rows;
        // Positive at `y` (the rotor passes nothing), falling through the root.
        let resid = |p1: f64| -> Result<f64, Abort> {
            census::bump(2);
            let t1 = tau_free(gas, tt4, f, p1, eta)?;
            Ok(a1 * v.flux(gas, f, tt4, p1)? - a2 * flux(gas, f, p1, t1, y)?)
        };
        // Started where the last solve of this search landed — else where the design put the interstage pressure — as a
        // share of this drop.
        let (s0, step) = match v.last_share.get() {
            s if s.is_nan() => ((1.0 - self.p1d) / (1.0 - self.yd), 0.02),
            s => (s, 0.001),
        };
        let (lo, hi) = (y + 1e-9 * (1.0 - y), 1.0 - 1e-9 * (1.0 - y));
        let p1 = illinois_from(resid, 1.0 - s0 * (1.0 - y), step * (1.0 - y), lo, hi, 0.0)?;
        v.last_share.set((1.0 - p1) / (1.0 - y));
        Ok(p1)
    }

    /// [`w4`](Self::w4) with the vane's sonic point found once (`Vane::at`) — for a search over `y` at one `(Tt4, f)`.
    pub fn w4_with(&self, v: &Vane, gas: &Gas, y: f64, tt4: f64, f: f64, eta: f64) -> Result<(f64, bool), Abort> {
        census::bump(1);
        if self.law == HpVane::Pin {
            return Ok((self.a4 * try_choked_mfp(gas, tt4, f)?, true));
        }
        let p1 = self.p1_at(v, gas, y, tt4, f, eta)?;
        if self.choked_d && p1 <= v.p_star {
            return Ok((self.a4 * try_choked_mfp(gas, tt4, f)?, true));
        }
        Ok((self.w4d * self.rows[0] * v.flux(gas, f, tt4, p1)? * powp(tt4 / self.td, 0.5), p1 <= v.p_star))
    }

    /// The EFFECTIVE `A4` at `y`: `W4/MFP*(Tt4)` — `A4` itself, bit for bit, on the pin and wherever a vane choked at
    /// design is choked here, so every reader's old arithmetic runs unchanged there.
    pub fn a4_at(&self, gas: &Gas, y: f64, tt4: f64, f: f64, eta: f64) -> Result<f64, Abort> {
        if self.law == HpVane::Pin {
            return Ok(self.a4);
        }
        self.a4_with(&Vane::at(gas, tt4, f)?, gas, y, tt4, f, eta)
    }

    /// [`a4_at`](Self::a4_at) with the vane's sonic point found once.
    pub fn a4_with(&self, v: &Vane, gas: &Gas, y: f64, tt4: f64, f: f64, eta: f64) -> Result<f64, Abort> {
        if self.law == HpVane::Pin {
            return Ok(self.a4);
        }
        match self.w4_with(v, gas, y, tt4, f, eta)? {
            (_, true) if self.choked_d => Ok(self.a4),
            (w, _) => Ok(w / try_choked_mfp(gas, tt4, f)?),
        }
    }
}

/// A transient closure reached under the law without the flight it runs at (the turbine below choke needs the flight's
/// ambient pressure), or — on two shafts — fed a compressor face other than the one its engine's inlet step last read
/// (the stored pressure would be another flight's) — refused in words rather than silently run on the pin.
pub fn no_flight(who: &str) -> Abort {
    Abort(format!("rung-105: {who} reached the HP vane's reaction law without its flight's ambient pressure — call it \
                   through an instant (which reads the flight's inlet first), not on its own"))
}

/// Rung 102's one-shaft afterburner under the law: its turbine is solved against the nozzle BEHIND the afterburner
/// (station 7: the duct's loss, the throat `A8`, the heat added lit), which the vane's closure ([`crate::spool`]'s
/// `a4_flow`, the dry nozzle at station 5) does not have — refused at every entry, before any speed search can skip it.
pub fn afterburner_refused() -> Abort {
    Abort("rung-105: rung 102's afterburner runs with the HP turbine's vanes choked (the old model) only: under the \
           reaction law the vanes' flow depends on how far the turbine expands, and behind an afterburner that is set by \
           the afterburner's own nozzle — a solve not built yet".into())
}

/// The COST instrument (anchor § COST): (vane sonic points, vane flows, interstage residuals, closure turbine
/// residuals), per thread. Read by the probes and the cost gate, never by a value gate.
pub mod census {
    use std::cell::Cell;
    thread_local! { pub static C: Cell<[u64; 4]> = const { Cell::new([0; 4]) }; }
    pub fn bump(i: usize) { C.with(|c| { let mut v = c.get(); v[i] += 1; c.set(v) }) }
    pub fn take() -> [u64; 4] { C.with(|c| c.replace([0; 4])) }
}
