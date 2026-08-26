//! RUNG 64 — the `phi`-REFERENCED BLEED LIMITER: the first CLOSED LOOP on an airflow lever.
//!
//! Slice X. Python's `BleedLimiter` + `LimitedBleedTransient` (`turbojet/engine.py`, 505 lines),
//! ported onto [`crate::bleed_transient`]'s shape.
//!
//! **HEADLINE (the rung's, not the port's): a limiter's LAW cannot buy PROTECTION, only its
//! PRICE.** The ceiling on the protected coordinate is `min phi` over the FULLY-OPEN march, which
//! is a property of `b_max` — the lever's AUTHORITY, i.e. hardware — and `b = b_max` is itself an
//! OPEN-LOOP law. What feedback buys is the BILL.
//!
//! # What slice X adds to the table
//!
//! **ONE cell** — [`b_at_point`](crate::bleed_transient::LeverHooks::b_at_point), whose two ladder
//! call sites are rung 64's own `_bill_cell` and rung 65's override — and **eight swaps**. § 5.22
//! (iii); it is the first row of § 5.19 (x)'s cell column an emitter confirms.
//!
//! # The two dead branches, with their counts — measured, not read
//!
//! § 5.22 (vi), on one floored march at `ds = 0.02`:
//!
//! * `b_of`'s **fall-through to rung 62** is taken **0 of 1 705 times** on any ARMED machine. It
//!   is live only where rung 64's cell and rung 62's are behaviourally identical, which is
//!   [[rust-port-slice-u-step3]]'s *a function exercised only on cells chosen for INERTNESS*.
//! * `b_of`'s **`b_state` override** is taken **0 of 1 705 times** at rung 64 ENTIRELY — it is
//!   rung 65's lagged valve, declared at 64. A port that drops it passes every slice-X gate and
//!   breaks at slice Y.
//!
//! Both are gated by MANUFACTURED bugs at step 5, never by a value key, because no value key can
//! reach them.

use crate::map::ComponentMap;

// ---------------------------------------------------------------------------------------------
// THE DEVICE
// ---------------------------------------------------------------------------------------------

/// RUNG 64's control law: **the smallest valve position in `[0, b_max]` that holds
/// `phi_lp >= phi_lim`.**
///
/// Every arming of this valve from rung 42 to 63 was OPEN LOOP — a constant position (42) or a
/// schedule `b(n_L)` read off the state (62). This one watches the **protected variable**, so it
/// is to rung 62 exactly what rung 49's `SurgeLimiter` is to rung 48's feedforward schedule.
///
/// **THE THREE CLAMPS ARE THE THREE REGIMES, and they are the rung** — see [`Regime`].
///
/// **WATCHES THE LP AND ONLY THE LP**, disclosed rather than parameterised: rung 42 established
/// the valve is a degree of freedom on the LP spool and not the HP, and the outer solve needs
/// `phi` MONOTONE in `b`, which the choked-`A4` argument gives for the LP face flow (it carries
/// `1/(1-b)`) and does not give for the HP.
///
/// `Copy` because [`LeverArming`](crate::bleed_transient::LeverArming) is, and that is what keeps
/// § 5.21 (iii)'s "the signature is never re-opened" true when this field is added to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BleedLimiter {
    /// The floor, in the map's own flow-coefficient units.
    pub phi_lim: f64,
    /// The valve's AUTHORITY — hardware, not a control setting.
    pub b_max: f64,
    /// RUNG 65: the valve's BANDWIDTH — hardware too. `None` IS rung 64's instantaneous valve,
    /// **not** `Some(0.0)`; Python's own assert says so and rung 65's finding is that the
    /// difference does not vanish as `tau -> 0`.
    pub tau: Option<f64>,
}

impl BleedLimiter {
    /// Python's `__init__` + `__post_init__` — all three asserts, in Python's order.
    pub fn new(phi_lim: f64, b_max: f64) -> Self {
        Self::with_tau(phi_lim, b_max, None)
    }

    /// The full constructor. **`b_max = 0` is REFUSED, not silently reduced**: a limiter that
    /// cannot act is a DIFFERENT object from an absent one (that is `bleed_lim = None`), and the
    /// distinction is the whole rung — the ceiling belongs to `b_max`.
    pub fn with_tau(phi_lim: f64, b_max: f64, tau: Option<f64>) -> Self {
        assert!(phi_lim > 0.0, "rung-64 phi floor is a flow coefficient");
        assert!(b_max > 0.0 && b_max < 0.5,
                "rung-64 needs a valve with AUTHORITY: b_max = 0 is a limiter that cannot act, \
                 which is a DIFFERENT object from an absent one (that is `bleed_lim=None`), and \
                 b >= 0.5 is rung 42's own starved-core bound; got b_max = {b_max}");
        assert!(tau.is_none_or(|t| t > 0.0),
                "rung-65 tau is a time constant on the march coordinate; the INSTANTANEOUS valve \
                 is rung 64 (tau=None), not tau=0. The two are different objects and rung 65's \
                 finding is that the difference does not vanish as tau -> 0; got tau = {tau:?}");
        BleedLimiter { phi_lim, b_max, tau }
    }

    /// `phi_lim = (1+sm) * phi_surge` off the map's OWN imposed surge line — rung 49's
    /// `from_margin`, so the two floors are set in identical units and rung 63 § 3's band edges
    /// are directly comparable set points.
    pub fn from_margin(cmap: &ComponentMap, b_max: f64, sm: f64) -> Self {
        Self::from_margin_tau(cmap, b_max, sm, None)
    }

    /// [`from_margin`](Self::from_margin) with rung 65's bandwidth.
    pub fn from_margin_tau(cmap: &ComponentMap, b_max: f64, sm: f64, tau: Option<f64>) -> Self {
        assert!(cmap.phi_surge > 0.0,
                "rung-64 from_margin needs a surge line: build the map with .with_phi_surge(.)");
        assert!(sm >= 0.0, "the rung-64 floor sits AT or ABOVE the surge line");
        Self::with_tau((1.0 + sm) * cmap.phi_surge, b_max, tau)
    }

    /// RUNG 65. The SAME control law on a valve with finite bandwidth — the only difference
    /// between rung 64's object and rung 65's, so every comparison between them holds `phi_lim`
    /// and `b_max` fixed by construction.
    pub fn lagged(&self, tau: f64) -> Self {
        Self::with_tau(self.phi_lim, self.b_max, Some(tau))
    }
}

/// Which clamp `_solve_b` landed on — **reported, never inferred by a reader comparing floats**,
/// which is Python's own sentence.
///
/// The distribution is not uniform and a gate that runs one machine tests two branches of three
/// (§ 5.22 (vi)/P3): on the rung's headline machine (`phi_lim = 0.80`, reachable) the split is
/// **257 dormant / 135 riding / 0 saturated**; only `authority_ceiling`'s deliberately over-set
/// floor saturates, at **167 / 74 / 151**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regime {
    /// `b = 0` — `phi` already clears the floor, and the closure dispatches to rung 63's parent
    /// **bit-for-bit**, not to a `0.0` position.
    Dormant,
    /// `0 < b < b_max` — rung 60's tautology pins `min phi_lp == phi_lim` EXACTLY.
    Riding,
    /// `b = b_max` — the floor is VIOLATED. The first law in this family that cannot deliver its
    /// own set point, and the regime that proves the CEILING belongs to `b_max` and not to the
    /// law.
    Saturated,
}
