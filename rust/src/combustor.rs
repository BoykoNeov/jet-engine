//! RUNG 37 — THE TWO INTERNAL CLOCKS: volume-filling CONFIRMS, heat-soak CORRECTS.
//!
//! Rungs 34–36 made the shaft the ONLY dynamic element and filed everything below the rotor as one
//! bundled concession — *"no combustor volume-filling, no heat soak … faster clocks below
//! `tau_spool`, they do not change the `r` framing"*. `CombustorTransient` (`engine.py:2012–2396`)
//! tests both halves and they SPLIT:
//!
//! * a **PLENUM** (`tau_fill << tau_spool`) CONFIRMS the concession — the `r -> 0` peak excursion
//!   still lands on rung 35's algebraic `E0` — while exposing the one thing the rigid coupling hid:
//!   this is the **first rung where compressor mass flow differs from NGV mass flow**, the plenum
//!   storing the difference;
//! * a **HEAT-SOAK** metal temperature (`tau_soak ~ tau_spool`) CORRECTS it — a genuine second
//!   STATE carrying thermal memory, so the transient is `E(r, theta0)` and not a function of `r`
//!   alone.
//!
//! **COMPOSITION, NOT A VIRTUAL SET** — § 5.12's census ran § 5.3's inheritance sweep in the
//! opposite direction and found six names crossing out of phase 6, every one of them on the
//! two-spool chain. `CombustorTransient` is `SpoolTransient`'s only subclass, it has none of its
//! own, and it overrides nothing rung 34 dispatches through. So this module reaches the parent
//! through a plain [`SpoolTransient`] field and needs no `Hooks` table — slice P's module note
//! predicted exactly this, and slice Q is the confirmation.
//!
//! **THE THREE MARCHES HERE CARRY NO `try`, AND `spool.rs::march` DOES.** Rung 34's marcher
//! `break`s out when any RK sub-stage leaves the valid region, which makes trajectory LENGTH an
//! output. All three of rung 37's marches ([`plenum_frozen_peak`], [`soak_excursion`],
//! [`adiabatic_excursion`]) run their step count unconditionally and let a failing stage
//! propagate. § 5.14 probe 3 measured **0 stage failures over 30 marches**, so the difference is
//! LATENT — which is exactly why the marches are written out here instead of routed through
//! `march`: fusing them would silently convert a raise into a truncation that no value gate could
//! ever see. It also supplies a second, independent reason for the decision slice P booked to
//! slice R, and from the opposite direction: rung 37's marches are not `march` with a different
//! closure, they are `march` without its most load-bearing line.
//!
//! **FALLIBILITY IS PER CALL SITE — AND BOTH ARMS ARE LIVE IN THIS ONE FILE.**
//! [`try_close_compressor_fuel_soak`] fires its bracket failure when it is reached from
//! [`equilibrium_soak`]'s march-in and never when it is reached from [`soak_excursion`]'s RK
//! stages. Same function, opposite treatment: the first caller must absorb (it is inside a bracket
//! search), the second must die (Python has no `try` there). Slice L step 1's rule with both
//! halves of it in one module for the first time.
//!
//! **Each count carries its own grid**, because there are two grids and they are not the same:
//! § 5.14 probe 3 measured **208 of 1 373** and **0 of 11 544** on `probe_q.py`'s (which runs
//! `equilibrium_soak` twice per cell, by construction), and `oracle/combustor_pypy.tsv` records
//! **104 of 666** in section F and **0 of 11 544** in section G on its own. The RATIO is the
//! claim; the totals belong to whichever grid produced them.
//!
//! [`plenum_frozen_peak`]: CombustorTransient::plenum_frozen_peak
//! [`soak_excursion`]: CombustorTransient::soak_excursion
//! [`adiabatic_excursion`]: CombustorTransient::adiabatic_excursion
//! [`try_close_compressor_fuel_soak`]: CombustorTransient::try_close_compressor_fuel_soak
//! [`equilibrium_soak`]: CombustorTransient::equilibrium_soak

use crate::components::{choked_mfp, ram_recovery};
use crate::engine::FlightCondition;
use crate::gas::{powp, Abort, Gas};
use crate::map::ComponentMap;
use crate::matcher::OffDesignMatcher;
use crate::spool::{
    try_illinois, CompState, Instant, SpoolTransient, ILLINOIS_MAXIT,
};

// ---------------------------------------------------------------------------------------------
// Instrumentation
// ---------------------------------------------------------------------------------------------

thread_local! {
    static BACKPRESSURE_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static BACKPRESSURE_BRACKET_FAILS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static PT4_AT_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static PT4_AT_BRACKET_FAILS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static PT4_AT_FLOOR_FAILS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static SOAK_CLOSE_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static SOAK_CLOSE_BRACKET_FAILS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static PLENUM_STATE_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static INSTANT_SOAK_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Census counters for rung 37's own call sites.
///
/// **Two of the five failure counters are expected DEAD and are gated against zero rather than
/// left absent** — `_compressor_from_backpressure`'s bracket and `_plenum_pt4_at`'s
/// `m_min < m_max` floor never fire, against `_plenum_pt4_at`'s bracket at **116 of 225** and the
/// soak closure's at **104 of 666** (`oracle/combustor_pypy.tsv`, sections C and F). A zero with a
/// live sibling in the same table is evidence; a zero on its own is silence.
///
/// The call counters do a second job the failure ones cannot: because rung 37's marches have no
/// `try`, a stage that failed would ABORT the whole march, so an evaluation count that reproduces
/// exactly is the certificate that every step ran. That is the whole of prediction 4's gate.
pub mod counters {
    use super::*;

    pub fn backpressure_calls() -> u64 { BACKPRESSURE_CALLS.with(|c| c.get()) }
    pub fn backpressure_bracket_fails() -> u64 { BACKPRESSURE_BRACKET_FAILS.with(|c| c.get()) }
    pub fn pt4_at_calls() -> u64 { PT4_AT_CALLS.with(|c| c.get()) }
    pub fn pt4_at_bracket_fails() -> u64 { PT4_AT_BRACKET_FAILS.with(|c| c.get()) }
    pub fn pt4_at_floor_fails() -> u64 { PT4_AT_FLOOR_FAILS.with(|c| c.get()) }
    pub fn soak_close_calls() -> u64 { SOAK_CLOSE_CALLS.with(|c| c.get()) }
    pub fn soak_close_bracket_fails() -> u64 { SOAK_CLOSE_BRACKET_FAILS.with(|c| c.get()) }
    pub fn plenum_state_calls() -> u64 { PLENUM_STATE_CALLS.with(|c| c.get()) }
    pub fn instant_soak_calls() -> u64 { INSTANT_SOAK_CALLS.with(|c| c.get()) }

    /// Read and RESET every counter. Same single-consumer caveat as `spool::counters::take`.
    pub fn take() -> Census {
        let c = Census {
            backpressure_calls: backpressure_calls(),
            backpressure_bracket_fails: backpressure_bracket_fails(),
            pt4_at_calls: pt4_at_calls(),
            pt4_at_bracket_fails: pt4_at_bracket_fails(),
            pt4_at_floor_fails: pt4_at_floor_fails(),
            soak_close_calls: soak_close_calls(),
            soak_close_bracket_fails: soak_close_bracket_fails(),
            plenum_state_calls: plenum_state_calls(),
            instant_soak_calls: instant_soak_calls(),
        };
        BACKPRESSURE_CALLS.with(|x| x.set(0));
        BACKPRESSURE_BRACKET_FAILS.with(|x| x.set(0));
        PT4_AT_CALLS.with(|x| x.set(0));
        PT4_AT_BRACKET_FAILS.with(|x| x.set(0));
        PT4_AT_FLOOR_FAILS.with(|x| x.set(0));
        SOAK_CLOSE_CALLS.with(|x| x.set(0));
        SOAK_CLOSE_BRACKET_FAILS.with(|x| x.set(0));
        PLENUM_STATE_CALLS.with(|x| x.set(0));
        INSTANT_SOAK_CALLS.with(|x| x.set(0));
        c
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Census {
        pub backpressure_calls: u64,
        pub backpressure_bracket_fails: u64,
        pub pt4_at_calls: u64,
        pub pt4_at_bracket_fails: u64,
        pub pt4_at_floor_fails: u64,
        pub soak_close_calls: u64,
        pub soak_close_bracket_fails: u64,
        pub plenum_state_calls: u64,
        pub instant_soak_calls: u64,
    }
}

// ---------------------------------------------------------------------------------------------
// Records
// ---------------------------------------------------------------------------------------------

/// One point on the forward speed line read as `pi_c(m)` — `_pic_of_m`'s 5-tuple.
///
/// **NOT [`SpoolTransient::pi_c_map`], which looks like the same arithmetic and is not.** That one
/// is rung 36's: it takes the flow coefficient `phi` and carries a `tau_c > 1.0` guard that
/// returns an error below the positive-work edge. This one takes the corrected flow `m` and has no
/// guard at all. Reusing rung 36's would add an error path Python does not have, on a function
/// that sits inside a bracket search.
#[derive(Clone, Copy, Debug)]
pub struct PicPoint {
    pub pi_c: f64,
    /// `m / n` — the flow coefficient.
    pub flowcoef: f64,
    pub tau_c: f64,
    pub tt3: f64,
    pub eta_c: f64,
}

/// The achievable `pi_c` band on the STABLE branch at one corrected speed — `_pic_band`'s 4-tuple.
#[derive(Clone, Copy, Debug)]
pub struct PicBand {
    /// Flow at the `phi`-floor; `pi_c` is DECREASING in `m` here, so this end is the maximum.
    pub m_lo: f64,
    pub pic_max: f64,
    /// Flow at the positive-work ceiling.
    pub m_hi: f64,
    pub pic_min: f64,
}

/// The compressor run from the plenum BACK-PRESSURE — `_compressor_from_backpressure`'s dict.
///
/// `pi_c` is the REQUIRED ratio `pt4/(pi_b*pt2)`, not the value recomputed at the root: Python
/// returns `pi_c=pi_c_req`, and the two differ in the last bits by however far the Illinois
/// stopped from the exact root.
#[derive(Clone, Copy, Debug)]
pub struct BackPressureComp {
    pub m: f64,
    pub flowcoef: f64,
    pub tau_c: f64,
    pub tt3: f64,
    pub eta_c: f64,
    pub pi_c: f64,
}

/// The decoupled plenum instant at `(nu, pt4, mdot_fuel)` — `_plenum_state`'s dict.
///
/// **This is neither an [`Instant`] nor a [`CompState`]**: there is no thrust, no `M9`, no nozzle
/// at all (see [`CombustorTransient::try_plenum_state`] for why), and there is a `dpt4_ds` neither
/// of them has. Its own struct, registered as a port decision in § 5.14.
#[derive(Clone, Copy, Debug)]
pub struct PlenumState {
    pub nu: f64,
    /// THE SECOND STATE — burner-exit total pressure.
    pub pt4: f64,
    pub tt4: f64,
    pub pi_c: f64,
    /// `m / n`.
    pub flowcoef: f64,
    pub f: f64,
    /// Compressor AIR delivery, kg/s.
    pub mdot_c: f64,
    /// NGV TOTAL drain (air + fuel), kg/s. **Differs from `mdot_c*(1+f)` off equilibrium — the
    /// first rung in the project where the two mass flows are not the same number.**
    pub mdot_ngv: f64,
    /// `dnu/ds`.
    pub phi: f64,
    /// `dpt4/ds = K*(mdot_c + mdot_fuel - mdot_ngv)`.
    pub dpt4_ds: f64,
    pub tau_t: f64,
    pub tt3: f64,
}

/// The heat-soak compressor closure — `_close_compressor_fuel_soak`'s dict.
///
/// The [`CompState`] carries `Tt4_turb` in its `tt4` field, because that is the temperature handed
/// to the NGV choke, to the turbine, and to [`SpoolTransient::try_instant_tail`] as its `tt4`
/// argument. `tt4_b` is the burner's own output, read only by `dTm/ds` and by the metal fixed
/// point. Registered as a port decision: Python's dict carries both under distinct keys and there
/// is exactly one `tt4` slot here.
pub struct SoakClose {
    pub comp: CompState,
    /// `Tt4_burner` — before the metal sink.
    pub tt4_b: f64,
    /// `Tt4_turb = Tt4_burner - G*(Tt4_burner - Tm)` — what the turbine sees.
    pub tt4_t: f64,
}

/// One heat-soak instant — rung 34's [`Instant`] plus the two things the metal state adds.
#[derive(Clone, Copy, Debug)]
pub struct SoakInstant {
    pub inst: Instant,
    pub tt4_burner: f64,
    /// `dTm/ds = (Tt4_burner - Tm)/r_m`.
    pub dtm_ds: f64,
}

/// RUNG 37's PLENUM FINDING.
#[derive(Clone, Copy, Debug)]
pub struct PlenumPeak {
    /// Rung 35's algebraic frozen-spool excursion.
    pub e0: f64,
    /// The finite-plenum peak — expected to land ON `e0`, independent of the fill clock.
    pub peak: f64,
    pub peak_minus_e0: f64,
    /// `max |mdot_c + mdot_fuel - mdot_ngv| / mdot_ngv` — the DECOUPLING, and the load-bearing
    /// half: a confirmation whose content is that the two flows came apart at all.
    pub split_max: f64,
    pub nu0: f64,
    pub r_v: f64,
}

/// The initial metal state of a heat-soak acceleration.
///
/// **PORT DECISION, and it NARROWS the domain.** Python takes a string and writes
/// `Tm = Tt4_lo if theta0 == "cold" else Tt4_hi`, so every value that is not `"cold"` — including
/// a typo — is hot. An enum makes that unrepresentable, which is a deliberate divergence rather
/// than an accident. [`Adiabatic`](Self::Adiabatic) is unreachable from
/// [`soak_excursion`](CombustorTransient::soak_excursion): it is the LABEL
/// [`adiabatic_excursion`](CombustorTransient::adiabatic_excursion) returns, never an input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theta0 {
    /// Metal at `Tt4_lo` — a first acceleration from cold. Heat sink ACTIVE.
    Cold,
    /// Metal at `Tt4_hi` — a re-acceleration from hot (the reslam / bodie). Little sink.
    Hot,
    /// `G = 0`: no metal at all. A return label, not an input.
    Adiabatic,
}

impl Theta0 {
    pub fn label(self) -> &'static str {
        match self {
            Theta0::Cold => "cold",
            Theta0::Hot => "hot",
            Theta0::Adiabatic => "adiabatic",
        }
    }
}

/// RUNG 37's HEAT-SOAK FINDING, and the `G = 0` reference it is measured against.
#[derive(Clone, Copy, Debug)]
pub struct SoakExcursion {
    pub theta0: Theta0,
    /// Peak running-line-referenced surge excursion.
    pub e_surge: f64,
    /// Nondimensional time to 99 % of the speed rise. `None` when the march ends first — which
    /// § 5.14 probe 3 measured REACHABLE on gate 5's `s_end = 6` grid (4 of 24 cells) and
    /// unreachable on gate 6's default `s_end = 12`.
    pub t_accel: Option<f64>,
    pub nu0: f64,
    pub nu_final: f64,
}

// ---------------------------------------------------------------------------------------------
// The transient
// ---------------------------------------------------------------------------------------------

/// RUNG 37. The two internal clocks, each modeled SEPARATELY and each DEFAULT OFF.
///
/// Both effects reduce to rung 35 by **exact dispatch, not by a stiff limit**: with `plenum_ratio`
/// and `soak_gain` at zero the extra state is never built and the inherited
/// [`equilibrium_fuel`](SpoolTransient::equilibrium_fuel) /
/// [`integrate_fuel`](SpoolTransient::integrate_fuel) are literally rung 34/35's. In Rust that is
/// the `inner` field being untouched, which is why gate 1 is a BIT comparison rather than a
/// tolerance.
pub struct CombustorTransient {
    /// The rung-34/35/36 transient this is built on. `pub` because gate 1 needs the SAME captured
    /// hardware on both sides, and because every inherited method is reached through it.
    pub inner: SpoolTransient,
    /// `r_v = tau_fill/tau_spool` at design. `0` ⇒ the plenum is OFF.
    pub plenum_ratio: f64,
    /// `G = hA/(mdot4*cp)`, the heat-extraction gain. `0` ⇒ heat-soak is OFF.
    pub soak_gain: f64,
    /// `r_m = tau_soak/tau_spool`, the metal clock.
    pub soak_ratio: f64,
    pub pt4_d: f64,
    pub mdot4_d: f64,
    /// `dpt4/ds = K*(mdot_c + mdot_fuel - mdot_ngv)`, fixed at the design station-4 state so the
    /// linearised drain rate is `1/r_v` at design and `tau_fill` rides off-design as a real fixed
    /// volume would.
    ///
    /// **`0.0` is a live value of this field and a dead one of the physics.** It is what the
    /// plenum-OFF construction gives, and gates 1 and 7 build exactly that — but no plenum method
    /// can be called on such an object (they all assert `plenum_ratio > 0`), so the zero is never
    /// multiplied by anything. Spelled because slice N step 3's rule says a constant measured DEAD
    /// still has to be spelled right.
    pub plenum_k: f64,
}

impl CombustorTransient {
    /// The stable-branch flow-coefficient floor — Python's `_PHI_FLOOR` class constant.
    ///
    /// Below it `pi_c(m)` turns back UP (the stalled branch past the efficiency-island peak at
    /// `phi ~ 0.2`), so the back-pressure invert would not be monotone. `0.3` clears the peak and
    /// still covers the deep-throttle near-surge points (`phi ~ 0.45`) the low-speed balance can
    /// need. A per-cell constant on § 5.12's rule, though nothing here shadows it.
    pub const PHI_FLOOR: f64 = 0.3;

    /// The metal fixed point's initial guess — a bare literal in Python, at BOTH of
    /// [`equilibrium_soak`](Self::equilibrium_soak)'s two loops.
    ///
    /// Not `Tt4_lo`, not the design `Tt4`, not the operating point: `1500.0` regardless of where
    /// the engine is running. § 5.14 probe 1 measured the worst pass count at **8** against the
    /// `range(60)` cap, so the guess is far enough in — but it is a constant of the source and is
    /// spelled as one.
    pub const TM_GUESS: f64 = 1500.0;

    /// The metal fixed point's iteration cap and its relative tolerance.
    pub const TM_MAX: usize = 60;
    pub const TM_TOL: f64 = 1e-10;

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        design_engine: crate::engine::Engine, flight_design: FlightCondition, mdot_design: f64,
        comp_map: ComponentMap, plenum_ratio: f64, soak_gain: f64, soak_ratio: f64,
    ) -> Self {
        assert!(
            plenum_ratio >= 0.0 && soak_gain >= 0.0 && soak_ratio >= 0.0,
            "rung-37 clock ratios / gain must be non-negative"
        );
        assert!(
            soak_gain == 0.0 || soak_ratio > 0.0,
            "heat-soak (soak_gain>0) needs soak_ratio>0"
        );
        let inner = SpoolTransient::new(design_engine, flight_design, mdot_design, comp_map);
        let s4 = inner.inner.inner.reference.station("4");
        let (pt4_d, far4) = (s4.pt, s4.far);
        let mdot4_d = inner.inner.inner.mdot_air_design * (1.0 + far4);
        let plenum_k =
            if plenum_ratio > 0.0 { pt4_d / (plenum_ratio * mdot4_d) } else { 0.0 };
        Self { inner, plenum_ratio, soak_gain, soak_ratio, pt4_d, mdot4_d, plenum_k }
    }

    fn mm(&self) -> &OffDesignMatcher { &self.inner.inner.inner }

    /// Python's `cmap if cmap is not None else self.comp_map`.
    fn cmap(&self, cmap: Option<&ComponentMap>) -> ComponentMap {
        *cmap.unwrap_or(&self.inner.inner.comp_map)
    }

    /// The freestream-derived `(Tt2, pt2, n)` every entry point of this rung opens with.
    ///
    /// Three Python bodies repeat these four lines verbatim (`_plenum_state` 2119–2122,
    /// `_plenum_pt4_at` 2155–2158, `_instant_soak` 2278–2281) and none of them argues for the
    /// repetition, so — as with `spool.rs::march` — the crate's *do not factor a deliberate
    /// duplication away* rule does not bite: it targets a duplication the SOURCE defends.
    ///
    /// `pub` because the smoke and oracle dumps drive [`pic_of_m`](Self::pic_of_m) and
    /// [`pic_band`](Self::pic_band) DIRECTLY, and they take `(n, Tt2)` rather than a `nu`. The
    /// alternative — re-deriving those four lines in the test — is the *a copy gates the copy*
    /// failure one level out.
    pub fn face(&self, flight: &FlightCondition, nu: f64) -> (f64, f64, f64, f64) {
        let mm = self.mm();
        let pi_d = mm.pi_d_max * ram_recovery(flight.m0);
        let (state0, v0) = mm.freestream_for(flight);
        let (tt2, pt2) = (state0.tt, pi_d * state0.pt);
        let n = nu * powp(self.inner.inner.tt2_d / tt2, 0.5);
        (tt2, pt2, n, v0)
    }

    // =========================================================================================
    // EFFECT 1 — the combustor PLENUM. `pt4` becomes a STATE and the compressor unlocks from
    // the NGV: `mdot_c != mdot_NGV`, the plenum storing the difference.
    // =========================================================================================

    /// The forward speed line's `pi_c` (and `phi`, `tau_c`, `Tt3`, `eta_c`) at corrected flow `m`
    /// and speed `n` — the arithmetic [`SpoolTransient::try_close_compressor`] uses, read as
    /// `pi_c(m)`.
    ///
    /// See [`PicPoint`] for why this is NOT [`SpoolTransient::pi_c_map`].
    pub fn pic_of_m(&self, cmap: &ComponentMap, n: f64, tt2: f64, m: f64) -> PicPoint {
        let mm = self.mm();
        let gas = mm.gas();
        let flowcoef = m / n;
        let tau_c = self.inner.tau_c_forward(cmap, n, m);
        let tt3 = tt2 * tau_c;
        let eta_c = cmap.eta_c_at(mm.eta_c, flowcoef, n);
        let (h2, h3) = (gas.h_c(tt2), gas.h_c(tt3));
        let tt3s = gas.t_from_h_c(h2 + eta_c * (h3 - h2));
        PicPoint { pi_c: gas.pr_c(tt3s) / gas.pr_c(tt2), flowcoef, tau_c, tt3, eta_c }
    }

    /// The achievable `pi_c` band on the STABLE branch at speed `n`.
    ///
    /// `pi_c` is monotone-DECREASING in `m` above the island peak, so a back-pressure whose
    /// required `pi_c` sits inside `(pic_min, pic_max)` has a unique operating flow. § 5.14 probe 3
    /// measured which arm of the ceiling binds: **`phi_max*n` in 15 of 15 cells**, the literal
    /// `2.5` never, and the floor never above the ceiling.
    pub fn pic_band(&self, cmap: &ComponentMap, n: f64, tt2: f64) -> PicBand {
        let (m_lo, m_hi) = (Self::PHI_FLOOR * n, 2.5f64.min(cmap.phi_max(0.1) * n));
        PicBand {
            m_lo,
            pic_max: self.pic_of_m(cmap, n, tt2, m_lo).pi_c,
            m_hi,
            pic_min: self.pic_of_m(cmap, n, tt2, m_hi).pi_c,
        }
    }

    /// Run the compressor from the plenum BACK-PRESSURE: invert the forward speed line
    /// `pi_c(n, m)` for the corrected flow `m` at the required `pi_c = pt4/(pi_b*pt2)`.
    ///
    /// **A THIRD use of the map** — not forward (rung 34), not inverted-for-`n` (rung 32), but
    /// inverted-for-`m` at a given `pi_c`.
    ///
    /// **The bracket failure is DEAD on every grid measured — 0 of 15 136 calls** (§ 5.14 probe 3)
    /// — and it is fallible anyway, because the site is reachable from inside
    /// [`try_plenum_pt4_at`](Self::try_plenum_pt4_at)'s Illinois, where Python's `assert` would
    /// abort a bracket search rather than an operation. Slice I's rule reads the REACHABILITY, not
    /// the firing count; the count is what the census gates against zero.
    pub fn try_compressor_from_backpressure(
        &self, cmap: &ComponentMap, n: f64, tt2: f64, pt2: f64, pt4: f64,
    ) -> Result<BackPressureComp, Abort> {
        BACKPRESSURE_CALLS.with(|c| c.set(c.get() + 1));
        let pi_c_req = pt4 / (self.mm().pi_b * pt2);
        let band = self.pic_band(cmap, n, tt2);
        let rlo = band.pic_max - pi_c_req;
        let rhi = band.pic_min - pi_c_req;
        if !(rlo > 0.0 && 0.0 > rhi) {
            BACKPRESSURE_BRACKET_FAILS.with(|c| c.set(c.get() + 1));
            return Err(Abort(format!(
                "rung-37 plenum back-pressure invert does not bracket at n={n:.4}, \
                 pt4={pt4:.0} (pi_c_req={pi_c_req:.4} outside band [{:.3},{:.3}]).",
                band.pic_min, band.pic_max
            )));
        }
        let m = try_illinois(
            |mm| Ok(self.pic_of_m(cmap, n, tt2, mm).pi_c - pi_c_req),
            band.m_lo,
            band.m_hi,
            rlo,
            rhi,
            SpoolTransient::HOT_TOL,
            ILLINOIS_MAXIT,
        )?;
        let p = self.pic_of_m(cmap, n, tt2, m);
        // `pi_c` is the REQUIRED ratio, not `p.pi_c` — Python returns `pi_c=pi_c_req`.
        Ok(BackPressureComp {
            m,
            flowcoef: p.flowcoef,
            tau_c: p.tau_c,
            tt3: p.tt3,
            eta_c: p.eta_c,
            pi_c: pi_c_req,
        })
    }

    /// The decoupled instant at `(nu, pt4, mdot_fuel)`: the two DISTINCT mass flows, the power
    /// imbalance, and `dpt4/ds`.
    ///
    /// **THE POWER BLOCK IS NOT [`SpoolTransient::try_instant_tail`]'s, AND THE DIFFERENCE IS THE
    /// WHOLE RUNG.** The tail computes a specific power `eta_m*(1+f)*(h_t4 - h_t5)` per unit AIR,
    /// which silently assumes the turbine passes exactly what the compressor delivered. Here the
    /// turbine passes `mdot_ngv` and the compressor `mdot_c`, so the powers are formed on the
    /// ABSOLUTE flows and divided by `P_ref*nu` at the end. The two agree only when
    /// `mdot_ngv == mdot_c*(1+f)` — which is precisely the coupling the plenum exists to break.
    ///
    /// **NO NOZZLE, SO NO DISPATCH.** This uses the choked turbine solve unconditionally (the
    /// plenum findings are all choked), which means `_instant_tail`'s subsonic re-solve and its
    /// `M9 > 0.985` escalation guard are **structurally unreachable** from the plenum path.
    /// Measured rather than read off slice P's totals: 2 272 calls made 2 272 `r34_solve_turbine`
    /// calls, 0 `_instant_tail` calls and 0 `_turbine_subsonic` calls (§ 5.14 probe 3).
    pub fn try_plenum_state(
        &self, flight: &FlightCondition, nu: f64, pt4: f64, mdot_fuel: f64, cmap: &ComponentMap,
    ) -> Result<PlenumState, Abort> {
        PLENUM_STATE_CALLS.with(|c| c.set(c.get() + 1));
        let mm = self.mm();
        let (tt2, pt2, n, _) = self.face(flight, nu);
        let c = self.try_compressor_from_backpressure(cmap, n, tt2, pt2, pt4)?;
        let (tt3, pi_c, flowcoef) = (c.tt3, c.pi_c, c.flowcoef);
        let mdot_c = c.m * self.inner.inner.mdot_corr_d * pt2 / powp(tt2, 0.5); // compressor AIR
        let f = mdot_fuel / mdot_c;
        let tt4 = self.inner.tt4_from_f(tt3, f);
        let wgas = mm.try_working_gas(f, tt4, pt4)?;
        let wg: &Gas = wgas.as_ref().unwrap_or_else(|| mm.gas());
        let mdot_ngv = mm.a4 * pt4 * choked_mfp(wg, tt4, f) / powp(tt4, 0.5); // NGV TOTAL drain
        let nu_t = nu * powp(self.inner.inner.tt4_d / tt4, 0.5);
        let eta_t = cmap.eta_t_at(mm.eta_t, nu_t);
        let (_pi_t, tau_t, tt5) = mm.solve_turbine(wg, tt4, f, Some(eta_t));
        let p_t = mm.eta_m * mdot_ngv * (wg.h_t(tt4, f) - wg.h_t(tt5, f));
        let p_c = mdot_c * (wg.h_c(tt3) - wg.h_c(tt2));
        let phi = (p_t - p_c) / (self.inner.p_ref * nu);
        let dpt4_ds = self.plenum_k * (mdot_c + mdot_fuel - mdot_ngv);
        Ok(PlenumState {
            nu, pt4, tt4, pi_c, flowcoef, f, mdot_c, mdot_ngv, phi, dpt4_ds, tau_t, tt3,
        })
    }

    /// [`try_plenum_state`](Self::try_plenum_state) for a caller that cannot fail — the RK stages
    /// of [`plenum_frozen_peak`](Self::plenum_frozen_peak), where Python has no `try` either.
    pub fn plenum_state(
        &self, flight: &FlightCondition, nu: f64, pt4: f64, mdot_fuel: f64, cmap: &ComponentMap,
    ) -> PlenumState {
        self.try_plenum_state(flight, nu, pt4, mdot_fuel, cmap)
            .unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The steady plenum pressure at fixed `(nu, mdot_fuel)`: `dpt4/ds = 0`, i.e.
    /// `mdot_c + mdot_fuel = mdot_NGV`.
    ///
    /// `mdot_NGV` rises ~linearly in `pt4` while `mdot_c` FALLS as the back-pressure loads the
    /// compressor, so the residual is monotone-decreasing and brackets cleanly.
    ///
    /// **THE TOLERANCE IS THE SLICE'S LEADING FINDING.** Python passes `tol=self._N_TOL = 1e-12`
    /// — an ABSOLUTE bracket width — on a `pt4` of order `1e5` Pa, seventeen decades below the
    /// values. § 5.14 probe 2 attributed the consequence: **103 of this site's 109 Illinois calls
    /// exhaust `maxit` and return `b`**, against **0** at every other call site in the port. Slice
    /// P step 1 had listed *"exhausting `maxit` returns `a` instead of `b`"* as one of two injected
    /// defects invisible to 132 bit-exact values and closed the blind spot with
    /// `counters::illinois_exhausted`; this is where that counter acquires a population. Injecting
    /// `return a` moves the answer by 3.5e-12 — still four orders below the 1e-9 bar rung 37's own
    /// gates are written at, so the COUNT remains the only thing that separates the two returns.
    ///
    /// **Both asserts are fallible and only one of them fires.** The bracket failure is reached
    /// **116 of 225** times from [`equilibrium_plenum`](Self::equilibrium_plenum)'s march-in; the
    /// `m_min < m_max` floor failure **0**. Gated against zero with a live sibling beside it.
    pub fn try_plenum_pt4_at(
        &self, flight: &FlightCondition, nu: f64, mdot_fuel: f64, cmap: &ComponentMap,
    ) -> Result<f64, Abort> {
        PT4_AT_CALLS.with(|c| c.set(c.get() + 1));
        let mm = self.mm();
        let bal = |pt4: f64| -> Result<f64, Abort> {
            let s = self.try_plenum_state(flight, nu, pt4, mdot_fuel, cmap)?;
            Ok(s.mdot_c + mdot_fuel - s.mdot_ngv)
        };
        // Bracket `pt4` by the compressor FLOW band, bounded like rung 35 so `f <= f_cap` (below
        // it the low-flow endpoint sends `f` huge and the burner inverse fails). THIRD copy of the
        // `0.05` literal (rung 35's fuel closure and the soak closure carry the other two);
        // Python keeps three and so does this.
        let f_cap = 0.05;
        let (tt2, pt2, n, _) = self.face(flight, nu);
        let m_fcap = mdot_fuel * powp(tt2, 0.5) / (f_cap * self.inner.inner.mdot_corr_d * pt2);
        let m_min = (Self::PHI_FLOOR * n).max(m_fcap);
        let m_max = 2.5f64.min(cmap.phi_max(0.1) * n);
        // `!(a < b)`, not `a >= b`: Python's `assert m_min < m_max` FAILS on a NaN and `>=` would
        // pass it. The negated form is the one that reproduces the source's NaN behaviour.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(m_min < m_max) {
            PT4_AT_FLOOR_FAILS.with(|c| c.set(c.get() + 1));
            return Err(Abort(format!(
                "rung-37 plenum: flow floor above the map ceiling at nu={nu:.4}"
            )));
        }
        // Nudge the endpoints strictly INSIDE the band so the invert never lands on the band edge,
        // where a last-bit rounding of `pi_c_req` against the recomputed edge trips its bracket.
        let lo = self.pic_of_m(cmap, n, tt2, m_max).pi_c * mm.pi_b * pt2 * (1.0 + 1e-9);
        let hi = self.pic_of_m(cmap, n, tt2, m_min).pi_c * mm.pi_b * pt2 * (1.0 - 1e-9);
        let (blo, bhi) = (bal(lo)?, bal(hi)?);
        if !(blo > 0.0 && 0.0 > bhi) {
            PT4_AT_BRACKET_FAILS.with(|c| c.set(c.get() + 1));
            return Err(Abort(format!(
                "rung-37 plenum mass balance does not bracket at nu={nu:.4}: \
                 b[lo]={blo:.3e}, b[hi]={bhi:.3e}"
            )));
        }
        try_illinois(bal, lo, hi, blo, bhi, SpoolTransient::N_TOL, ILLINOIS_MAXIT)
    }

    /// [`try_plenum_pt4_at`](Self::try_plenum_pt4_at) for a caller that cannot fail.
    pub fn plenum_pt4_at(
        &self, flight: &FlightCondition, nu: f64, mdot_fuel: f64, cmap: &ComponentMap,
    ) -> f64 {
        self.try_plenum_pt4_at(flight, nu, mdot_fuel, cmap).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The plenum EQUILIBRIUM — `dnu/ds = 0` AND `dpt4/ds = 0` at fixed FUEL.
    ///
    /// **THE NON-TAUTOLOGICAL REDUCE.** It reproduces rung 35's
    /// [`equilibrium_fuel`](SpoolTransient::equilibrium_fuel) through the BACK-PRESSURE closure —
    /// a genuinely different code path from rung 35's NGV-continuity root find. Two closures, one
    /// operating point. Nested: for each `nu` the inner solve closes the mass balance on `pt4`,
    /// and the outer one finds the `nu` where the power balances.
    pub fn equilibrium_plenum(
        &self, flight: &FlightCondition, mdot_fuel: f64, cmap: Option<&ComponentMap>,
    ) -> PlenumState {
        let cmap = self.cmap(cmap);
        assert!(
            self.plenum_ratio > 0.0,
            "equilibrium_plenum needs a plenum: plenum_ratio>0."
        );
        let nu = self.inner.find_equilibrium_nu(|nu| {
            let pt4 = self.try_plenum_pt4_at(flight, nu, mdot_fuel, &cmap)?;
            Ok(self.try_plenum_state(flight, nu, pt4, mdot_fuel, &cmap)?.phi)
        });
        let pt4 = self.plenum_pt4_at(flight, nu, mdot_fuel, &cmap);
        self.plenum_state(flight, nu, pt4, mdot_fuel, &cmap)
    }

    /// THE PLENUM FINDING. At `r -> 0` (a fuel step at a frozen spool `nu0`) the plenum fills; the
    /// PEAK surge excursion still lands on rung 35's algebraic `E0`, INDEPENDENT of the fill clock
    /// `r_v` — the CONFIRMATION — and the structural content is the mass-flow SPLIT stored during
    /// the fill.
    ///
    /// **NO `try` ANYWHERE IN THE MARCH.** Python runs `n_steps + 1` iterations unconditionally
    /// and a failing RK stage propagates out of the whole call. That is the opposite of
    /// `spool.rs::march`, and it is why this body is written out rather than routed through it —
    /// see the module note. § 5.14 probe 3 measured 0 stage failures over 30 marches, so the
    /// difference is latent and the dump gates it by reproducing the evaluation COUNT.
    ///
    /// `ds_frac` is Python's default `1/15`. `n_steps` uses `round_ties_even` on `march`'s
    /// precedent; the quotient is exactly `150.0` at every `r_v` swept here (§ 5.14 probe 3), so
    /// the tie is measured unreachable rather than assumed to be.
    pub fn plenum_frozen_peak(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, cmap: Option<&ComponentMap>,
        ds_frac: f64,
    ) -> PlenumPeak {
        let cmap = self.cmap(cmap);
        assert!(
            self.plenum_ratio > 0.0,
            "plenum_frozen_peak needs a plenum: plenum_ratio>0."
        );
        let mf_lo = self.inner.fuel_for_tt4(flight, tt4_lo, Some(&cmap));
        let mf_hi = self.inner.fuel_for_tt4(flight, tt4_hi, Some(&cmap));
        let eq_lo = self.inner.equilibrium_fuel(flight, mf_lo, Some(&cmap)); // rung-35 start
        let (nu0, pc_lo) = (eq_lo.nu, eq_lo.pi_c);
        let e0 = self.inner.constant_speed_excursion_fuel(flight, tt4_lo, tt4_hi, Some(&cmap)).0;
        let mut pt4 = self.plenum_pt4_at(flight, nu0, mf_lo, &cmap); // steady plenum at the start

        let dpt4 =
            |pt4v: f64| -> f64 { self.plenum_state(flight, nu0, pt4v, mf_hi, &cmap).dpt4_ds };

        let r_v = self.plenum_ratio;
        let ds = r_v * ds_frac;
        let n_steps = (10.0 * r_v / ds).round_ties_even() as i64;
        let (mut e_peak, mut split_max) = (0.0f64, 0.0f64);
        for i in 0..=n_steps {
            let s = self.plenum_state(flight, nu0, pt4, mf_hi, &cmap);
            e_peak = e_peak.max(s.pi_c / pc_lo - 1.0);
            split_max =
                split_max.max(((s.mdot_c + mf_hi - s.mdot_ngv) / s.mdot_ngv).abs());
            if i == n_steps {
                break;
            }
            let k1 = s.dpt4_ds;
            let k2 = dpt4(pt4 + 0.5 * ds * k1);
            let k3 = dpt4(pt4 + 0.5 * ds * k2);
            let k4 = dpt4(pt4 + ds * k3);
            pt4 += ds / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
        }
        PlenumPeak { e0, peak: e_peak, peak_minus_e0: e_peak - e0, split_max, nu0, r_v }
    }

    // =========================================================================================
    // EFFECT 2 — HEAT-SOAK. A metal state `Tm` between burner-exit and turbine-inlet:
    //   Tt4_turb = Tt4_burner - G*(Tt4_burner - Tm),   dTm/ds = (Tt4_burner - Tm)/r_m.
    // Mass flows stay COUPLED (the NGV-continuity closure holds); only the TEMPERATURE lags.
    // =========================================================================================

    /// Rung 35's fuel closure with the metal heat sink between burner-exit and the NGV.
    ///
    /// `Tt4_turb = Tt4_burner - G*(Tt4_burner - Tm)` feeds the choke and the turbine; the root
    /// find is the same NGV-continuity residual, so only the temperature is depressed.
    ///
    /// **THE BRACKET FAILURE IS LIVE FROM ONE CALLER AND DEAD FROM THE OTHER —
    /// 104 of 666 from [`equilibrium_soak`](Self::equilibrium_soak)'s march-in, 0 of 11 544 from
    /// [`soak_excursion`](Self::soak_excursion)'s RK stages**, both off the oracle's own grid
    /// (`census/F/*` and `census/G/*`). That is slice L step 1's *fallibility is per call site,
    /// not per function* with both arms of it in one module: the first caller is a bracket search
    /// and must absorb, the second has no `try` in Python and must die.
    pub fn try_close_compressor_fuel_soak(
        &self, tt2: f64, pt2: f64, cmap: &ComponentMap, n: f64, mdot_fuel: f64, tm: f64,
    ) -> Result<SoakClose, Abort> {
        SOAK_CLOSE_CALLS.with(|c| c.set(c.get() + 1));
        let mm = self.mm();
        let gas = mm.gas();
        let g_gain = self.soak_gain;

        let eval_m = |m: f64| -> Result<SoakClose, Abort> {
            let flowcoef = m / n;
            let tau_c = self.inner.tau_c_forward(cmap, n, m);
            let tt3 = tt2 * tau_c;
            let eta_c = cmap.eta_c_at(mm.eta_c, flowcoef, n);
            let (h2, h3) = (gas.h_c(tt2), gas.h_c(tt3));
            let tt3s = gas.t_from_h_c(h2 + eta_c * (h3 - h2));
            let pi_c = gas.pr_c(tt3s) / gas.pr_c(tt2);
            let pt4 = mm.pi_b * pi_c * pt2;
            let mdot_air = m * self.inner.inner.mdot_corr_d * pt2 / powp(tt2, 0.5);
            let f = mdot_fuel / mdot_air;
            let tt4_b = self.inner.tt4_from_f(tt3, f);
            let tt4_t = tt4_b - g_gain * (tt4_b - tm); // metal heat sink
            let wgas = mm.try_working_gas(f, tt4_t, pt4)?;
            let wg: &Gas = wgas.as_ref().unwrap_or(gas);
            let mdot4 = mm.a4 * pt4 * choked_mfp(wg, tt4_t, f) / powp(tt4_t, 0.5);
            let mdot_air_ngv = mdot4 / (1.0 + f);
            let m_imp = (mdot_air_ngv * powp(tt2, 0.5) / pt2) / self.inner.inner.mdot_corr_d;
            Ok(SoakClose {
                comp: CompState {
                    m, m_imp, phi: flowcoef, tau_c, eta_c, tt3, tt4: tt4_t, pi_c, pt4, f, wgas,
                    mdot4, mdot_air,
                },
                tt4_b,
                tt4_t,
            })
        };

        // SECOND copy of the `f_cap` literal (rung 35's fuel closure carries the first, the plenum
        // pressure solve the third). Python keeps three.
        let f_cap = 0.05;
        let lo = mdot_fuel * powp(tt2, 0.5) / (f_cap * self.inner.inner.mdot_corr_d * pt2);
        let hi = 2.5f64.min(cmap.phi_max(0.1) * n);
        let g = |m: f64| -> Result<f64, Abort> { Ok(m - eval_m(m)?.comp.m_imp) };
        let (glo, ghi) = (g(lo)?, g(hi)?);
        if !(glo < 0.0 && 0.0 < ghi) {
            SOAK_CLOSE_BRACKET_FAILS.with(|c| c.set(c.get() + 1));
            return Err(Abort(format!(
                "rung-37 heat-soak closure does not bracket at n={n:.4}, mdot_fuel={mdot_fuel:.5} \
                 (g[{lo:.3}]={glo:.3e}, g[{hi:.3}]={ghi:.3e})."
            )));
        }
        let root = try_illinois(g, lo, hi, glo, ghi, SpoolTransient::HOT_TOL, ILLINOIS_MAXIT)?;
        eval_m(root)
    }

    /// The heat-soak instant at `(nu, mdot_fuel, Tm)`.
    ///
    /// The turbine, the power imbalance and the thrust reuse rung 34's
    /// [`try_instant_tail`](SpoolTransient::try_instant_tail) unchanged — mass flows stay coupled
    /// here, so only `Tt4` is depressed and everything below the closure is the same arithmetic.
    /// That is the OPPOSITE of the plenum, whose power block could not reuse it (see
    /// [`try_plenum_state`](Self::try_plenum_state)).
    pub fn try_instant_soak(
        &self, flight: &FlightCondition, nu: f64, mdot_fuel: f64, tm: f64,
        cmap: Option<&ComponentMap>,
    ) -> Result<SoakInstant, Abort> {
        INSTANT_SOAK_CALLS.with(|c| c.set(c.get() + 1));
        let cmap = self.cmap(cmap);
        let (tt2, pt2, n, v0) = self.face(flight, nu);
        let comp = self.try_close_compressor_fuel_soak(tt2, pt2, &cmap, n, mdot_fuel, tm)?;
        let inst = self.inner.try_instant_tail(
            flight, nu, comp.tt4_t, &comp.comp, n, tt2, pt2, v0, &cmap,
        )?;
        Ok(SoakInstant {
            inst,
            tt4_burner: comp.tt4_b,
            dtm_ds: (comp.tt4_b - tm) / self.soak_ratio,
        })
    }

    /// [`try_instant_soak`](Self::try_instant_soak) for a caller that cannot fail — the RK stages
    /// of [`soak_excursion`](Self::soak_excursion), where Python has no `try`.
    pub fn instant_soak(
        &self, flight: &FlightCondition, nu: f64, mdot_fuel: f64, tm: f64,
        cmap: Option<&ComponentMap>,
    ) -> SoakInstant {
        self.try_instant_soak(flight, nu, mdot_fuel, tm, cmap)
            .unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The heat-soak EQUILIBRIUM at fixed FUEL.
    ///
    /// **THE REDUCE:** at steady state `dTm/ds = 0` ⇒ `Tm = Tt4_burner` ⇒ `Q = 0` ⇒
    /// `Tt4_turb = Tt4_burner`, so this reproduces rung 35's `equilibrium_fuel` EXACTLY. Heat-soak
    /// is a purely TRANSIENT effect and never moves the running line — a structural reduce, not a
    /// knob-to-zero limit.
    ///
    /// **THE SAME FIXED-POINT LOOP APPEARS TWICE AND THE TWO COPIES DO NOT AGREE, WHICH IS WHY
    /// THEY ARE WRITTEN OUT.** Python's inner loop (inside the residual) assigns
    /// `Tm = Tt4_burner` on the pass that CONVERGES, before breaking; the outer loop breaks
    /// WITHOUT that assignment. So the residual the root find sees is evaluated one fixed-point
    /// update ahead of the instant this method returns. § 5.14 probe 1 measured what unifying them
    /// costs: `nu` **bit-identical** (the outer loop is downstream of the root find and cannot
    /// move it), `pi_c` **3.098e-12**, `Tt4` **9.767e-12**, against a gate written at `1e-9`. The
    /// crate's *do not factor a deliberate duplication away* rule usually turns on whether the
    /// source argues for the copy; here the two copies are not even the same code, so factoring
    /// them would have to CHOOSE one semantics and would silently choose the wrong one.
    pub fn equilibrium_soak(
        &self, flight: &FlightCondition, mdot_fuel: f64, cmap: Option<&ComponentMap>,
    ) -> SoakInstant {
        let cmap = self.cmap(cmap);
        assert!(self.soak_gain > 0.0, "equilibrium_soak needs heat-soak: soak_gain>0.");

        let nu = self.inner.find_equilibrium_nu(|nu| {
            // metal in equilibrium with the gas: at fixed nu, Q=0 <=> Tm = Tt4_burner. Iterate.
            let mut tm = Self::TM_GUESS;
            for _ in 0..Self::TM_MAX {
                let inst = self.try_instant_soak(flight, nu, mdot_fuel, tm, Some(&cmap))?;
                if (inst.tt4_burner - tm).abs() <= Self::TM_TOL * tm {
                    tm = inst.tt4_burner; // <-- the line the OUTER loop below does not have
                    break;
                }
                tm = inst.tt4_burner;
            }
            Ok(self.try_instant_soak(flight, nu, mdot_fuel, tm, Some(&cmap))?.inst.phi)
        });

        let mut tm = Self::TM_GUESS;
        for _ in 0..Self::TM_MAX {
            let inst = self.instant_soak(flight, nu, mdot_fuel, tm, Some(&cmap));
            if (inst.tt4_burner - tm).abs() <= Self::TM_TOL * tm {
                break; // <-- and NOT `tm = inst.tt4_burner;` first. See the doc comment.
            }
            tm = inst.tt4_burner;
        }
        self.instant_soak(flight, nu, mdot_fuel, tm, Some(&cmap))
    }

    /// THE HEAT-SOAK FINDING. March the two-state `(nu, Tm)` transient for a fuel step
    /// `mf(Tt4_lo) -> mf(Tt4_hi)` from an initial metal state.
    ///
    /// * [`Cold`](Theta0::Cold) — metal at `Tt4_lo` (a first acceleration from cold): the heat
    ///   sink is ACTIVE, `Tt4_turb` is depressed, the colder sonic throat passes MORE corrected
    ///   flow, so the operating point moves AWAY from surge — and the acceleration is SLOW,
    ///   because the metal is stealing turbine work.
    /// * [`Hot`](Theta0::Hot) — metal at `Tt4_hi` (a re-acceleration from hot, the reslam): little
    ///   sink, so the excursion sits NEAR the adiabatic worst case and the accel is ~fast.
    ///
    /// The load-bearing claim is the ORDERING `cold < hot < adiabatic`, not the magnitudes.
    ///
    /// **The march has no `try`** — see [`plenum_frozen_peak`](Self::plenum_frozen_peak) and the
    /// module note.
    #[allow(clippy::too_many_arguments)]
    pub fn soak_excursion(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, theta0: Theta0,
        cmap: Option<&ComponentMap>, ds: f64, s_end: f64,
    ) -> SoakExcursion {
        let cmap = self.cmap(cmap);
        assert!(self.soak_gain > 0.0, "soak_excursion needs heat-soak: soak_gain>0.");
        let grid: Vec<f64> =
            (0..9).map(|k| tt4_lo + (tt4_hi - tt4_lo) * k as f64 / 8.0).collect();
        let rl = self.inner.running_line(flight, &grid, Some(&cmap));
        let nus: Vec<f64> = rl.iter().map(|p| p.0).collect();
        let pcs: Vec<f64> = rl.iter().map(|p| p.1).collect();
        let nu0 = self.inner.equilibrium(flight, tt4_lo, Some(&cmap)).nu;
        let nu_final = self.inner.equilibrium(flight, tt4_hi, Some(&cmap)).nu;
        let mf_hi = self.inner.fuel_for_tt4(flight, tt4_hi, Some(&cmap));
        // Python: `Tm = Tt4_lo if theta0 == "cold" else Tt4_hi` — every non-"cold" string is hot.
        let mut tm = if theta0 == Theta0::Cold { tt4_lo } else { tt4_hi };

        let deriv = |nu_: f64, tm_: f64| -> SoakInstant {
            self.instant_soak(flight, nu_, mf_hi, tm_, Some(&cmap))
        };

        let (mut nu, mut s) = (nu0, 0.0f64);
        let (mut e_surge, mut t_accel) = (0.0f64, None);
        let n_steps = (s_end / ds).round_ties_even() as i64;
        for i in 0..=n_steps {
            let a = deriv(nu, tm);
            let (k1n, k1m) = (a.inst.phi, a.dtm_ds);
            e_surge = e_surge.max(a.inst.pi_c / SpoolTransient::interp(&nus, &pcs, nu) - 1.0);
            if t_accel.is_none() && nu >= nu0 + 0.99 * (nu_final - nu0) {
                t_accel = Some(s);
            }
            if i == n_steps {
                break;
            }
            let b = deriv(nu + 0.5 * ds * k1n, tm + 0.5 * ds * k1m);
            let (k2n, k2m) = (b.inst.phi, b.dtm_ds);
            let c = deriv(nu + 0.5 * ds * k2n, tm + 0.5 * ds * k2m);
            let (k3n, k3m) = (c.inst.phi, c.dtm_ds);
            let d = deriv(nu + ds * k3n, tm + ds * k3m);
            let (k4n, k4m) = (d.inst.phi, d.dtm_ds);
            nu += ds / 6.0 * (k1n + 2.0 * k2n + 2.0 * k3n + k4n);
            tm += ds / 6.0 * (k1m + 2.0 * k2m + 2.0 * k3m + k4m);
            s += ds;
        }
        SoakExcursion { theta0, e_surge, t_accel, nu0, nu_final }
    }

    /// The `G = 0` (adiabatic) reference for [`soak_excursion`](Self::soak_excursion): the rung-35
    /// fuel-control step response with no metal at all.
    ///
    /// `E_surge` here is rung 35's `E_surge0` — the peak occurs at the frozen-spool instant.
    ///
    /// **It does NOT assert `soak_gain > 0`,** so it is callable on a soak-configured object and
    /// on a bare one alike; the object's `G` is simply never read, which is what makes it a
    /// reference rather than a limit. Note the shaft ODE is marched with rung 35's
    /// [`instant_fuel`](SpoolTransient::instant_fuel) — the PANICKING twin, because Python has no
    /// `try` here either.
    pub fn adiabatic_excursion(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, cmap: Option<&ComponentMap>,
        ds: f64, s_end: f64,
    ) -> SoakExcursion {
        let cmap = self.cmap(cmap);
        let grid: Vec<f64> =
            (0..9).map(|k| tt4_lo + (tt4_hi - tt4_lo) * k as f64 / 8.0).collect();
        let rl = self.inner.running_line(flight, &grid, Some(&cmap));
        let nus: Vec<f64> = rl.iter().map(|p| p.0).collect();
        let pcs: Vec<f64> = rl.iter().map(|p| p.1).collect();
        let nu0 = self.inner.equilibrium(flight, tt4_lo, Some(&cmap)).nu;
        let nu_final = self.inner.equilibrium(flight, tt4_hi, Some(&cmap)).nu;
        let mf_hi = self.inner.fuel_for_tt4(flight, tt4_hi, Some(&cmap));

        let phi_at =
            |nu_: f64| -> f64 { self.inner.instant_fuel(flight, nu_, mf_hi, Some(&cmap)).phi };

        let (mut nu, mut s) = (nu0, 0.0f64);
        let (mut e_surge, mut t_accel) = (0.0f64, None);
        let n_steps = (s_end / ds).round_ties_even() as i64;
        for i in 0..=n_steps {
            let inst = self.inner.instant_fuel(flight, nu, mf_hi, Some(&cmap));
            e_surge = e_surge.max(inst.pi_c / SpoolTransient::interp(&nus, &pcs, nu) - 1.0);
            if t_accel.is_none() && nu >= nu0 + 0.99 * (nu_final - nu0) {
                t_accel = Some(s);
            }
            if i == n_steps {
                break;
            }
            let k1 = inst.phi;
            let k2 = phi_at(nu + 0.5 * ds * k1);
            let k3 = phi_at(nu + 0.5 * ds * k2);
            let k4 = phi_at(nu + ds * k3);
            nu += ds / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
            s += ds;
        }
        SoakExcursion { theta0: Theta0::Adiabatic, e_surge, t_accel, nu0, nu_final }
    }
}
