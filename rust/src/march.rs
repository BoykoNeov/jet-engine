//! **Phase 4 — the nozzle & turbine MARCHES (rungs 25–30).** Slice F is rungs 25/26.
//!
//! Rung 14 (slice E, in [`crate::nox`]) bracketed the frozen production nozzle against a
//! reversible shifting-equilibrium expansion. These rungs resolve the REAL flow *between* those
//! bounds — first with a normalised Damköhler knob that slides the whole expansion uniformly
//! (rung 25), then with a LOCAL, anchored recombination clock that lets the relaxation shut off
//! partway down the nozzle (rung 26, freeze-out).
//!
//! # Why this is a module and not more of `nox.rs`
//!
//! Slice E's port decision 1 put rung 17 in `nox.rs` and recorded explicitly that phase 4's
//! marches "are where that module decision belongs — not pre-built for a phase that has not been
//! scoped". It is scoped now, and the decision goes the other way: `nox.rs` is already 4,349
//! lines, and the dependency here is strictly ONE-WAY — the marches consume
//! [`mix_entropy_molar`] / [`mix_mass_per_air`] / [`mix_h_abs_b`] / [`expand_nozzle`], and
//! nothing in rungs 7–24 consumes a march. There is no circular dependency to buy, which was
//! slice E's stated reason against splitting.
//!
//! # THE DUPLICATION IS DELIBERATE — DO NOT FACTOR IT
//!
//! [`freeze_out_expand`] is [`finite_rate_expand`]'s loop copied line for line, with the scalar
//! `da` promoted to a per-step `da_local_fn(comp, T, p)`. The Python does exactly this and says
//! why: it keeps rung 25 literally untouched, and rung 26's reduce — a constant `da_local_fn`
//! reproducing `finite_rate_expand(da)` **to the ULP** — is then a live tripwire on the copy.
//!
//! **Factoring the two into one generic body would compile, pass the oracle, and silently make
//! that gate a self-comparison.** That is not hypothetical: slice D's retrospective (memory:
//! *a ported test can go VACUOUS*) is the same mechanism — a better factorisation dissolving the
//! source's real pin. The oracle cannot see it either, because a Python↔Rust dump compares
//! values and a tautological gate still produces correct ones. So the copy stays a copy, and
//! `tests/rung26.rs` gates the two Rust functions against each other.
//!
//! # What the probes measured before this was written
//!
//! `todo-rust-port.md` § 4.11, on PyPy, over a sweep wider than the source's own gates:
//!
//! * The energy bisection's upper bracket `T + 50` is sound — largest single-step temperature
//!   RISE 12.956 K (hottest/fastest/coarsest corner, first step), never closer than 37.04 K to
//!   the top. It is transcribed literally: it sets the iterate sequence, so narrowing it moves
//!   every bit downstream.
//! * [`equilibrate_hp`]'s bracket `[tt9 − 100, tt9 + 800]` is over-wide and asymmetric in the
//!   UNUSED direction — the root is always ABOVE `tt9`, by 0.02 K cold to 21.40 K hot. Also
//!   transcribed literally, for the same reason.
//! * [`DS_FLOOR`]'s justifying comment is CONFIRMED in both its numbers. `ds` legitimately lands
//!   NEGATIVE in 13 of 70 sweep cells at the shipped `nstep` — a near-total cancellation whose
//!   sign is not fixed, which makes it the most drift-sensitive quantity here and the arm the
//!   oracle leads with.
//! * The composition-ORDER hazard filed against the two hand-built relaxation dictionaries is NOT
//!   live: `equilibrium_composition` returns `[CO2, H2O, CO, H2, OH, O, H, O2, N2, Ar]` for 112
//!   of 112 probed states, so the `.get(sp, 0.0)` zero-fill has no reachable instance. The ORDER
//!   is still load-bearing (both accumulations run in it), so the oracle dumps it as data.
//!
//! # Arithmetic spellings that are NOT free here
//!
//! * `k(T) = A·T^n` with `n = -2.0` — a float CONSTANT, not an integer literal, so PyPy's
//!   `x ** 2` → multiply rewrite does not apply and Python reaches libm `pow`. Hence [`powp`],
//!   by the same reasoning `JetMixing::schedule` carries.
//! * `V9 = math.sqrt(...)` is the sqrt instruction — `.sqrt()`, never `powp(_, 0.5)`.
//! * Every accumulation runs in the composition's own order; `f64` addition is not associative.

use crate::gas::{equilibrium_composition, powp, Gas, RU};
use crate::nox::{expand_nozzle, mix_entropy_molar, mix_h_abs_b, mix_mass_per_air, T_EXIT_FLOOR};

// ------------------------------------------------------------------------------------------- //
// Rung 25 — FINITE-RATE nozzle chemistry (the Damköhler flow BETWEEN rung-14's bounds)
// ------------------------------------------------------------------------------------------- //

/// 2nd-law tolerance, J/(mol·air·K).
///
/// The marched entropy production is a difference of two molar entropies and lands slightly
/// NEGATIVE from trapezoid truncation. The source's comment justifies the value with two
/// numbers, and § 4.11 probe 3 confirmed both: at the config minimum `nstep = 100` the worst
/// truncation is −5.366e-04 (measured at the FROZEN limit, `da` = 0.03, and concentrated at cold
/// `Tt4` — exactly as the comment predicts), while `nstep = 10` gives −3.2e-02 … −7.3e-02 and
/// fires. The assert does not fire at `nstep = 20`, so the config's own `nstep ≥ 100` guard
/// carries 5× margin over where this floor actually bites.
pub const DS_FLOOR: f64 = -5e-3;

/// One marched nozzle expansion — the shared result of rungs 25 and 26.
#[derive(Debug, Clone)]
pub struct March {
    /// exit static temperature, K
    pub t9: f64,
    /// exhaust velocity, m/s (on ABSOLUTE enthalpy, so recombination energy appears)
    pub v9: f64,
    /// exit composition (mole numbers per mol dry air), in the entry composition's order
    pub comp9: Vec<(&'static str, f64)>,
    /// entropy production `S_exit − S_entry`, J/(mol·air·K) — ≥ 0 physically, see [`DS_FLOOR`]
    pub ds: f64,
    /// fewest halvings any one step's energy bisection took
    ///
    /// A NAMING key, not an independent discriminator, and recorded as one — the same
    /// classification slice E gave `Expansion::iters`. `t9` is already gated at bit-equality, so
    /// a mis-shaped loop is caught by the VALUE; what the count adds is that the failure reads
    /// "41 halvings instead of 37". Measured 36–37 across all 70 probe marches.
    pub iters_min: usize,
    /// most halvings any one step's energy bisection took (see [`March::iters_min`])
    pub iters_max: usize,
}

/// The per-step Damköhler schedule rung 26 promotes rung 25's scalar `Da` into: a function of the
/// running `(composition, T, p)`.
///
/// That single promotion is the whole structural difference between the two marches, so it gets a
/// name — and the name is what lets [`freeze_out_expand`] keep a signature that reads as
/// "[`finite_rate_expand`], with the rate made local".
pub type DaLocalFn<'a> = &'a dyn Fn(&[(&'static str, f64)], f64, f64) -> f64;

/// One station of a march, recorded for rung 28's coupled NO clock.
#[derive(Debug, Clone)]
pub struct MarchStation {
    /// progress coordinate ∈ [0, 1] (linear in ln p)
    pub s: f64,
    /// static pressure, Pa
    pub p: f64,
    /// static temperature, K
    pub t: f64,
    /// composition at this station (mole numbers per mol dry air)
    pub comp: Vec<(&'static str, f64)>,
}

/// The `dict.get(sp, 0.0)` of the two hand-built relaxation dictionaries.
///
/// § 4.11 probe 5 measured the zero-fill branch UNREACHABLE from these entry points (112 of 112
/// probed states return the same ten species in the same order). It is transcribed anyway,
/// because the Python does it and a port that assumes the key sets match is asserting something
/// the source does not.
fn get_or_zero(comp: &[(&str, f64)], sp: &str) -> f64 {
    comp.iter().find(|&&(s, _)| s == sp).map_or(0.0, |&(_, n)| n)
}

/// `CO/(CO+CO2)` — the dissociation content, and rung 26's load-bearing observable.
fn co_fraction(comp: &[(&str, f64)]) -> f64 {
    let a = get_or_zero(comp, "CO");
    let b = get_or_zero(comp, "CO2");
    if a + b > 0.0 {
        a / (a + b)
    } else {
        0.0
    }
}

/// One FINITE-RATE nozzle expansion (rung 25) — the Damköhler flow BETWEEN rung-14's frozen
/// (`Da→0`) and irreversible-fast (`Da→∞`) limits.
///
/// Marches the exact `dh = v·dp` relation (energy + momentum, valid for ANY adiabatic
/// frictionless flow, reversible or not) down a GEOMETRIC pressure schedule
/// `p(s) = pt9·(p9/pt9)^s`, `s ∈ [0,1]`:
///
/// * **composition** — exact linear relaxation over the step,
///   `Δn = (1 − e^{−Da·ds})·(n_eq(T,p) − n)` toward local equilibrium. Unconditionally stable for
///   the stiff relaxation (a raw explicit step blows up at large `Da`) and conserves atoms
///   exactly, each element count being a linear invariant shared by `n` and `n_eq`.
/// * **temperature** — `dh = v·dp` integrated IMPLICITLY per step by bisecting `T1` on
///   `H_abs(comp1,T1) − H_abs(comp0,T0) = ½(v0+v1)·dp`, with `v = n_tot·Ru·T/p` per mol air.
///   Carries BOTH the pressure-work cooling and the recombination reheat, the composition change
///   being inside `H_abs`.
///
/// NEVER called at `da = 0` or `da = ∞` — those dispatch to the exact (F)/(I) references.
///
/// **LOOP SHAPE IS LITERAL**, and it is this slice's second of three bisection tolerances
/// (`1e-11·Tm` here, `1e-10·T` in [`equilibrate_hp`], `1e-13·Tm` in slice E's `expand_nozzle`):
/// counted loop, midpoint at the TOP, bracket updated, break tested on THIS iteration's
/// PRE-update midpoint with `<=`, and the result recomputed from the FINAL bracket after the
/// loop. Transcribing the three uniformly is the silent one-bit defect.
pub fn finite_rate_expand(
    comp_entry: &[(&'static str, f64)],
    far: f64,
    tt9: f64,
    pt9: f64,
    p9: f64,
    da: f64,
    nstep: usize,
) -> March {
    let s_entry = mix_entropy_molar(comp_entry, tt9, pt9);
    let h_entry = mix_h_abs_b(comp_entry, tt9);
    let m = mix_mass_per_air(comp_entry);
    let mut comp: Vec<(&'static str, f64)> = comp_entry.to_vec();
    let mut t = tt9;
    let lnr = (p9 / pt9).ln(); // < 0
    let ds = 1.0 / nstep as f64;
    let relax = 1.0 - (-da * ds).exp(); // exact linear-relaxation fraction per step
    let mut iters_min = usize::MAX;
    let mut iters_max = 0usize;

    for k in 0..nstep {
        let p0 = pt9 * (lnr * k as f64 * ds).exp();
        let p1 = pt9 * (lnr * (k + 1) as f64 * ds).exp();
        let dp = p1 - p0; // < 0 (expanding)
        let h0 = mix_h_abs_b(&comp, t);
        let ntot0: f64 = comp.iter().map(|&(_, n)| n).sum();
        let v0 = ntot0 * RU * t / p0;
        let base = h0 + 0.5 * v0 * dp;
        let n_eq = equilibrium_composition(far, t, p0); // target at the step-START state
        let comp1: Vec<(&'static str, f64)> = comp
            .iter()
            .map(|&(sp, n)| (sp, n + relax * (get_or_zero(&n_eq, sp) - n)))
            .collect();
        let ntot1: f64 = comp1.iter().map(|&(_, n)| n).sum();

        // hi headroom covers a step where recombination reheat outruns the pressure-work
        // cooling. § 4.11 probe 1 measured the largest such rise at 12.956 K against this
        // 50 K bracket, and the entry re-equilibration that bounds it at 21.40 K.
        let (mut lo, mut hi) = (T_EXIT_FLOOR, t + 50.0);
        let mut used = 0usize;
        for _ in 0..200 {
            let tm = 0.5 * (lo + hi);
            used += 1;
            let resid = mix_h_abs_b(&comp1, tm) - base - 0.5 * (ntot1 * RU * tm / p1) * dp;
            if resid > 0.0 {
                hi = tm;
            } else {
                lo = tm;
            }
            if hi - lo <= 1e-11 * tm {
                break;
            }
        }
        t = 0.5 * (lo + hi);
        assert!(
            t > T_EXIT_FLOOR + 1.0,
            "finite-rate exit T={t:.1} K pinned at the {T_EXIT_FLOOR:.0} K floor \
             (Da={da}, far={far:.4})"
        );
        iters_min = iters_min.min(used);
        iters_max = iters_max.max(used);
        comp = comp1;
    }

    let h_exit = mix_h_abs_b(&comp, t);
    // `math.sqrt`, so `.sqrt()` — NOT `powp(_, 0.5)`.
    let v9 = (2.0 * (h_entry - h_exit) / m).sqrt();
    let ds_prod = mix_entropy_molar(&comp, t, p9) - s_entry;
    // 2nd law, as a conservation assert that runs on every execution (project contract).
    assert!(
        ds_prod > DS_FLOOR,
        "finite-rate dS={ds_prod:.3e} < 0 (2nd law violated) — nstep={nstep} too coarse \
         (trapezoid truncation); increase nstep (≥ 100 is well-resolved here)."
    );
    March { t9: t, v9, comp9: comp, ds: ds_prod, iters_min, iters_max }
}

/// Constant-`(H, p)` adiabatic equilibration (rung 25): `(comp*, T*)` with
/// `comp* = eq(far, T*, p)` and `H_abs(comp*, T*) = H_target`.
///
/// This is the entry RE-EQUILIBRATION of the super-equilibrium frozen mixture — recombination
/// reheats, so `T* >` the frozen-entry `T`. § 4.11 probe 2 measured that rise at 0.02 K (cold)
/// to 21.40 K (hot), against a bracket that reaches 800 K above and 100 K below: the lower half
/// is never entered. The bracket is still passed and transcribed literally, because it sets the
/// bisection's iterate sequence.
///
/// Bisection tolerance here is `1e-10·T` — the slice's third, and different from both others.
pub fn equilibrate_hp(
    far: f64,
    h_target: f64,
    p: f64,
    t_lo: f64,
    t_hi: f64,
) -> (Vec<(&'static str, f64)>, f64) {
    let (mut lo, mut hi) = (t_lo, t_hi);
    for _ in 0..200 {
        let t = 0.5 * (lo + hi);
        let comp = equilibrium_composition(far, t, p);
        if mix_h_abs_b(&comp, t) > h_target {
            hi = t;
        } else {
            lo = t;
        }
        if hi - lo <= 1e-10 * t {
            break;
        }
    }
    let t = 0.5 * (lo + hi);
    assert!(
        t > T_EXIT_FLOOR + 1.0,
        "const-(H,p) equilibration T={t:.1} K pinned at the {T_EXIT_FLOOR:.0} K floor \
         (far={far:.4})"
    );
    (equilibrium_composition(far, t, p), t)
}

/// The IRREVERSIBLE-FAST ceiling (I) — the `Da→∞` finite-rate limit, in CLOSED FORM (rung 25).
///
/// Rate-law-INDEPENDENT: re-equilibrate the frozen super-equilibrium entry at constant
/// `(H, pt9)` → `(comp*, T*)`, then expand reversibly-shifting from `(comp*, T*, pt9)` to `p9`
/// on rung-14's machinery. Because the const-`H` step conserves enthalpy, `V9` is measured from
/// the SAME stagnation enthalpy as (F) and (R), so it sits STRICTLY BELOW rung-14's reversible
/// bound: the entry re-equilibration is an entropy source no rate can remove.
///
/// Returns `(t9, v9, comp9, t_star)`. The KEYSTONE of rung 25 is that the marching integrator's
/// `Da→∞` asymptote converges to THIS closed form.
pub fn irreversible_fast_expand(
    comp_entry: &[(&'static str, f64)],
    far: f64,
    tt9: f64,
    pt9: f64,
    p9: f64,
) -> (f64, f64, Vec<(&'static str, f64)>, f64) {
    let h_entry = mix_h_abs_b(comp_entry, tt9);
    let (comp_star, t_star) = equilibrate_hp(far, h_entry, pt9, tt9 - 100.0, tt9 + 800.0);
    let ex = expand_nozzle(&comp_star, far, t_star, pt9, p9, true);
    (ex.t9, ex.v9, ex.comp9, t_star)
}

// ------------------------------------------------------------------------------------------- //
// Rung 26 — FREEZE-OUT: an ANCHORED recombination clock over rung-25's exact integrator
// ------------------------------------------------------------------------------------------- //

/// GRI-Mech 3.0, verbatim: `H+OH+M <=> H2O+M`, `A = 2.200e22` cm⁶/(mol²·s).
///
/// The dominant three-body radical sink — the SAME mechanism the dissociation species' NASA
/// polynomials already cite, so ZERO new unanchored constants.
pub const K_HOHM_A: f64 = 2.200e22;
/// Temperature exponent `n` of the recombination rate. `Ea = 0` EXACTLY ⇒ `k(T) = A·T^n` with NO
/// Arrhenius exponential; `n < 0` ⇒ `k` ACCELERATES as `T` falls, OPPOSING freeze-out (which is
/// density-driven — that inversion is rung 26's finding).
pub const N_HOHM: f64 = -2.0;

/// Anchored recombination clock (rung 26) — the OH-consumption relaxation time of the GRI-Mech
/// termolecular sink:
///
/// ```text
/// τ_chem = 1 / ( k(T)·[OH]·[M] ),  k(T) = A·T^n (Ea=0),  [OH] = x_OH·c_tot,  [M] = c_tot
/// c_tot  = p/(Ru·T), converted to mol/cm³ to match the CHEMKIN cm⁶/mol²/s units.
/// ```
///
/// A three-body rate is `k[X][Y][M]`, so this carries a DENSITY² law (`c_tot² ∝ (p/T)²`). That
/// collapse — AGAINST a `k(T)` that RISES on cooling — is what freezes the flow. Returns `+∞`
/// when `x_OH ≤ 0` (no radical to recombine ⇒ infinitely slow ⇒ frozen).
///
/// `kill_t` pins `T` in `k(T)` only (density alone drives); `kill_m` pins the density in the
/// `[OH]·[M]` term only (temperature alone). Each leaves the OTHER dependence live — the
/// mechanism certification. Run on a STANDALONE clock with `x_OH` held fixed; the marched
/// integrator moves `x_OH`, so it does not isolate the two.
///
/// **`powp`, not a product.** `N_HOHM` is a float CONSTANT, so Python reaches libm `pow` here —
/// the same reasoning `JetMixing::schedule` carries for its float exponent field.
pub fn tau_chem_recomb(
    comp: &[(&str, f64)],
    t: f64,
    p: f64,
    kill_t: Option<f64>,
    kill_m: Option<f64>,
) -> f64 {
    let ntot: f64 = comp.iter().map(|&(_, n)| n).sum();
    let x_oh = if ntot > 0.0 { get_or_zero(comp, "OH") / ntot } else { 0.0 };
    if x_oh <= 0.0 {
        return f64::INFINITY;
    }
    let c_tot = p / (RU * t) / 1.0e6; // mol/m³ → mol/cm³ (CHEMKIN units)
    let tk = kill_t.unwrap_or(t); // k(T) temperature   (kill_t pins it)
    let c_m = kill_m.unwrap_or(c_tot); // density in [OH]·[M] (kill_m pins it)
    let k = K_HOHM_A * powp(tk, N_HOHM); // Ea=0 ⇒ pure power law, no exp
    1.0 / (k * x_oh * c_m * c_m) // [OH]·[M] = x_oh·c_M·c_M (density²)
}

/// One FREEZE-OUT nozzle expansion (rung 26) — [`finite_rate_expand`]'s loop DUPLICATED VERBATIM,
/// with the scalar `da` promoted to a per-step `da_local_fn(comp, T, p)`.
///
/// That single change lets the relaxation SHUT OFF partway down the nozzle (freeze-out). When
/// `da_local_fn` returns a CONSTANT this reproduces `finite_rate_expand(da)` **to the ULP** —
/// § 4.11 probe 4 measured that on the Python at 40/40 bit-exact (the first exactness claim in
/// this lineage to survive, after three consecutive corrections), and `tests/rung26.rs` gates the
/// two RUST functions against each other. **See the module header for why the duplication must
/// not be factored away.**
///
/// `record`, when supplied, is APPENDED the `(s, p, T, comp)` state at every step start plus the
/// exit. It is a PURE OBSERVER — it copies already-computed values and feeds nothing back, so
/// every returned number is bit-for-bit identical with and without it. Rung 28 (slice G) needs
/// this trajectory to read its NO clock on the RELAXING pool instead of the frozen one.
///
/// Returns the march plus `(s_freeze, da_entry, da_exit)`. `s_freeze ∈ [0,1]` is where
/// `da_local` first crosses below 1 — the freeze point; 0 means frozen from entry, 1 means
/// relaxing throughout.
#[allow(clippy::too_many_arguments)]
pub fn freeze_out_expand(
    comp_entry: &[(&'static str, f64)],
    far: f64,
    tt9: f64,
    pt9: f64,
    p9: f64,
    da_local_fn: DaLocalFn,
    nstep: usize,
    mut record: Option<&mut Vec<MarchStation>>,
) -> (March, f64, f64, f64) {
    let s_entry = mix_entropy_molar(comp_entry, tt9, pt9);
    let h_entry = mix_h_abs_b(comp_entry, tt9);
    let m = mix_mass_per_air(comp_entry);
    let mut comp: Vec<(&'static str, f64)> = comp_entry.to_vec();
    let mut t = tt9;
    let lnr = (p9 / pt9).ln(); // < 0
    let ds = 1.0 / nstep as f64;
    let mut s_freeze = 1.0; // default: never crosses ⇒ relaxes throughout
    let mut frozen = false;
    let da_entry = da_local_fn(&comp, t, pt9);
    let mut iters_min = usize::MAX;
    let mut iters_max = 0usize;

    for k in 0..nstep {
        let p0 = pt9 * (lnr * k as f64 * ds).exp();
        let p1 = pt9 * (lnr * (k + 1) as f64 * ds).exp();
        let dp = p1 - p0; // < 0 (expanding)
        if let Some(rec) = record.as_deref_mut() {
            // rung-28 observer (no feedback — bit-for-bit)
            rec.push(MarchStation { s: k as f64 * ds, p: p0, t, comp: comp.clone() });
        }
        let da_local = da_local_fn(&comp, t, p0); // <-- THE only change vs rung 25
        if !frozen && da_local < 1.0 {
            // first sub-unity crossing = the freeze point
            s_freeze = k as f64 * ds;
            frozen = true;
        }
        let relax = 1.0 - (-da_local * ds).exp();
        let h0 = mix_h_abs_b(&comp, t);
        let ntot0: f64 = comp.iter().map(|&(_, n)| n).sum();
        let v0 = ntot0 * RU * t / p0;
        let base = h0 + 0.5 * v0 * dp;
        let n_eq = equilibrium_composition(far, t, p0);
        let comp1: Vec<(&'static str, f64)> = comp
            .iter()
            .map(|&(sp, n)| (sp, n + relax * (get_or_zero(&n_eq, sp) - n)))
            .collect();
        let ntot1: f64 = comp1.iter().map(|&(_, n)| n).sum();

        let (mut lo, mut hi) = (T_EXIT_FLOOR, t + 50.0);
        let mut used = 0usize;
        for _ in 0..200 {
            let tm = 0.5 * (lo + hi);
            used += 1;
            let resid = mix_h_abs_b(&comp1, tm) - base - 0.5 * (ntot1 * RU * tm / p1) * dp;
            if resid > 0.0 {
                hi = tm;
            } else {
                lo = tm;
            }
            if hi - lo <= 1e-11 * tm {
                break;
            }
        }
        t = 0.5 * (lo + hi);
        assert!(
            t > T_EXIT_FLOOR + 1.0,
            "freeze-out exit T={t:.1} K pinned at the {T_EXIT_FLOOR:.0} K floor (far={far:.4})"
        );
        iters_min = iters_min.min(used);
        iters_max = iters_max.max(used);
        comp = comp1;
    }

    if let Some(rec) = record {
        // rung-28 observer: the exit state
        rec.push(MarchStation { s: 1.0, p: p9, t, comp: comp.clone() });
    }
    let h_exit = mix_h_abs_b(&comp, t);
    let v9 = (2.0 * (h_entry - h_exit) / m).sqrt();
    let ds_prod = mix_entropy_molar(&comp, t, p9) - s_entry;
    assert!(
        ds_prod > DS_FLOOR,
        "freeze-out dS={ds_prod:.3e} < 0 (2nd law violated) — nstep={nstep} too coarse \
         (trapezoid truncation); increase nstep (≥ 100 is well-resolved here)."
    );
    let da_exit = da_local_fn(&comp, t, p9);
    (
        March { t9: t, v9, comp9: comp, ds: ds_prod, iters_min, iters_max },
        s_freeze,
        da_entry,
        da_exit,
    )
}

// ------------------------------------------------------------------------------------------- //
// The configs and the diagnostic states
// ------------------------------------------------------------------------------------------- //

/// Rung-25 finite-rate nozzle config: the Damköhler flow BETWEEN rung-14's bounds.
///
/// `da` is a NORMALISED-SCHEDULE Damköhler number (`τ_flow/τ_chem` over the whole expansion), NOT
/// an Arrhenius-anchored chemical time — a CARTOON knob. `Da→0` is frozen, `Da→∞` the
/// irreversible-fast ceiling. A CONSTANT `da` interpolates the bracket but CANNOT show
/// freeze-out; that is what rung 26 is for.
#[derive(Debug, Clone, Copy)]
pub struct FiniteRate {
    /// normalised Damköhler number (THE knob); interior `0 < da < ∞`
    pub da: f64,
    /// pressure-march resolution (≥ 100)
    ///
    /// The exp-relaxation step is UNCONDITIONALLY stable in `da` (`relax ∈ [0,1]`); this sets
    /// only the trapezoid-energy accuracy, which is 2nd-order.
    pub nstep: usize,
}

impl Default for FiniteRate {
    /// Python's dataclass defaults. `Da` has none there (it is the required knob), so the value
    /// here is a placeholder every construction site is expected to overwrite.
    fn default() -> Self {
        Self { da: 1.0, nstep: 400 }
    }
}

impl FiniteRate {
    /// Python's `__post_init__`.
    pub fn validate(&self) {
        assert!(
            self.da > 0.0,
            "FiniteRate.Da={} must be positive (Da=0 and Da=∞ are the dispatched F/I bounds)",
            self.da
        );
        assert!(
            self.nstep >= 100,
            "FiniteRate.nstep={} too coarse (need ≥ 100 — below it the trapezoid truncation \
             gives a non-physical 2nd-law violation)",
            self.nstep
        );
    }
}

/// Rung-26 freeze-out config: an ANCHORED recombination clock over rung-25's exact integrator.
///
/// `l` is the ONE geometric knob (`τ_res = L/(0.6·V9_frozen)`); the chemistry is anchored to
/// GRI-Mech 3.0 with zero new constants. `rate_scale` (1.0 = anchored) scales `da_local` to drive
/// the limit gates: `→0` gives frozen (F), `→∞` the irreversible-fast ceiling (I). It does NOT
/// give the bit-for-bit rung-25 reduce — it scales `da_local` but the schedule still varies with
/// `T,p`; that reduce needs a CONSTANT `da_local`.
#[derive(Debug, Clone, Copy)]
pub struct FreezeOut {
    /// residence length, m — THE geometric knob; sets `τ_res`, hence the freeze LOCATION
    pub l: f64,
    /// pressure-march resolution (≥ 100), as rung 25
    pub nstep: usize,
    /// dimensionless `da_local` multiplier for the limit gates (1.0 = anchored)
    pub rate_scale: f64,
}

impl Default for FreezeOut {
    fn default() -> Self {
        Self { l: 0.5, nstep: 400, rate_scale: 1.0 }
    }
}

impl FreezeOut {
    /// Python's `__post_init__`.
    pub fn validate(&self) {
        assert!(self.l > 0.0, "FreezeOut.L={} must be positive", self.l);
        assert!(
            self.nstep >= 100,
            "FreezeOut.nstep={} too coarse (need ≥ 100 — below it the trapezoid truncation \
             gives a non-physical 2nd-law violation)",
            self.nstep
        );
        assert!(
            self.rate_scale > 0.0,
            "FreezeOut.rate_scale={} must be positive",
            self.rate_scale
        );
    }
}

/// Rung-25 finite-rate nozzle diagnostic. A pure diagnostic BESIDE the cycle — the production
/// nozzle stays FROZEN, so the cycle is bit-for-bit rung 6.
///
/// THE THREE-STATE PICTURE — `V9_frozen ≤ V9_finite ≤ V9_irrev_fast ≤ V9_reversible`:
///
/// * **(F)** `v9_frozen` — `Da→0`, rung-14's lower bound and the production nozzle (the EXACT
///   reduce).
/// * **(I)** `v9_irrev_fast` — `Da→∞`, the ATTAINABLE ceiling. The super-equilibrium frozen entry
///   re-equilibrates IRREVERSIBLY, an entropy loss no rate removes, so it sits STRICTLY BELOW …
/// * **(R)** `v9_reversible` — … rung-14's reversible upper bound, an UNREACHABLE ceiling.
#[derive(Debug, Clone)]
pub struct FiniteRateNozzleState {
    /// the config's Damköhler number
    pub da: f64,
    /// (F) frozen exit static T (== production nozzle), K
    pub t9_frozen: f64,
    /// (F) frozen exhaust velocity, m/s
    pub v9_frozen: f64,
    /// (I) irreversible-fast exit static T, K
    pub t9_irrev_fast: f64,
    /// (I) attainable-ceiling exhaust velocity, m/s
    pub v9_irrev_fast: f64,
    /// reheated entry T after const-`(H,pt9)` re-equilibration, K (> `tt9`)
    pub t_star_entry: f64,
    /// (R) reversible-shift exit static T, K
    pub t9_reversible: f64,
    /// (R) unreachable-ceiling exhaust velocity, m/s
    pub v9_reversible: f64,
    /// finite-rate exit static T, K
    pub t9_finite: f64,
    /// finite-rate exhaust velocity, m/s (between F and I)
    pub v9_finite: f64,
    /// finite-rate entropy production, J/(mol·air·K) ≥ 0
    pub ds_finite: f64,
    /// `CO/(CO+CO2)` at the nozzle entry (the dissociation content)
    pub co_fraction_entry: f64,
    /// `CO/(CO+CO2)` at the finite-rate exit (recombination burnout)
    pub co_fraction_finite_exit: f64,
}

impl FiniteRateNozzleState {
    /// `(I−F)`: the exhaust velocity a fast REAL nozzle can recover, m/s.
    pub fn attainable_gap(&self) -> f64 {
        self.v9_irrev_fast - self.v9_frozen
    }

    /// `(R−I)`: the reversible bound's UNREACHABLE margin — the entry re-equilibration
    /// irreversibility, m/s. The "sliver" rung 14 set aside; > 0 whenever the entry is
    /// super-equilibrium. Its existence and sign are thermodynamically robust; the magnitude is
    /// not certified.
    pub fn unreachable_gap(&self) -> f64 {
        self.v9_reversible - self.v9_irrev_fast
    }

    /// Fraction of the ATTAINABLE bracket `[F, I]` the finite-rate flow reaches at this `da`.
    pub fn finite_filled(&self) -> f64 {
        let g = self.v9_irrev_fast - self.v9_frozen;
        if g > 0.0 {
            (self.v9_finite - self.v9_frozen) / g
        } else {
            0.0
        }
    }
}

/// Rung-26 freeze-out diagnostic. A pure diagnostic BESIDE the cycle.
///
/// Adds NO new bound: the freeze-out flow lands inside rung-25's `[v9_frozen, v9_irrev_fast]`.
/// What it resolves is WHERE the relaxation shuts off, and that point MOVES with `Tt4`. The
/// freeze is certified in COMPOSITION space (`s_freeze` and the frozen-in exit `CO/(CO+CO2)`);
/// the `V9` bracket is sub-percent hot, so `v9_freeze` is a tiny wiggle inside `[F, I]` — its
/// ordering holds but its magnitude is not the finding.
#[derive(Debug, Clone)]
pub struct FreezeOutNozzleState {
    /// (F) frozen exit static T (== production nozzle), K
    pub t9_frozen: f64,
    /// (F) frozen exhaust velocity, m/s
    pub v9_frozen: f64,
    /// (I) irreversible-fast exit static T, K
    pub t9_irrev_fast: f64,
    /// (I) attainable-ceiling exhaust velocity, m/s
    pub v9_irrev_fast: f64,
    /// freeze-out exit static T, K
    pub t9_freeze: f64,
    /// freeze-out exhaust velocity, m/s (inside `[F, I]`)
    pub v9_freeze: f64,
    /// freeze-out entropy production, J/(mol·air·K) ≥ 0
    pub ds_freeze: f64,
    /// progress coordinate where `da_local` crosses 1 (0 = frozen from entry)
    pub s_freeze: f64,
    /// `da_local` at the nozzle entry
    pub da_entry: f64,
    /// `da_local` at the exit
    pub da_exit: f64,
    /// `CO/(CO+CO2)` at the nozzle entry (the dissociation content)
    pub co_fraction_entry: f64,
    /// `CO/(CO+CO2)` frozen in at the freeze-out exit — the load-bearing observable, not `V9`
    pub co_fraction_freeze_exit: f64,
}

impl FreezeOutNozzleState {
    /// True when `da_local < 1` at the entry — the flow never switches on (dormant lean).
    pub fn frozen_from_entry(&self) -> bool {
        self.da_entry < 1.0
    }

    /// Fraction of the attainable bracket `[F, I]` the freeze-out flow reaches (sub-percent hot).
    pub fn bracket_filled(&self) -> f64 {
        let g = self.v9_irrev_fast - self.v9_frozen;
        if g > 0.0 {
            (self.v9_freeze - self.v9_frozen) / g
        } else {
            0.0
        }
    }
}

// ------------------------------------------------------------------------------------------- //
// The `Gas` entry points
// ------------------------------------------------------------------------------------------- //

impl Gas {
    /// Finite-rate nozzle-flow diagnostic (rung 25).
    ///
    /// Rung 14 bracketed the frozen production nozzle against a reversible shifting-equilibrium
    /// expansion. This resolves the REAL flow BETWEEN them at a finite Damköhler number — and
    /// finds the bracket is really a THREE-state picture (see [`FiniteRateNozzleState`]).
    ///
    /// Reduces to rung-14 FROZEN exactly, and DELIBERATELY does not reduce to equilibrium: the
    /// `(R−I)` entry-irreversibility gap is the finding — dormant lean, ~7 % of the bracket hot.
    /// Pass the run's `tt9 = Tt5`, `pt9 = π_n·pt5`, `p9 = p_exit`.
    ///
    /// A pure diagnostic: reads only `(far, tt4, pt4, tt9, pt9, p9)` and touches no cycle path,
    /// so the cycle stays bit-for-bit rung 6. Requires the equilibrium (rung-6) gas.
    #[allow(clippy::too_many_arguments)]
    pub fn finite_rate_nozzle(
        &self,
        far: f64,
        tt4: f64,
        pt4: f64,
        tt9: f64,
        pt9: f64,
        p9: f64,
        finite_rate: FiniteRate,
    ) -> FiniteRateNozzleState {
        assert!(
            self.is_equilibrium(),
            "finite_rate_nozzle: needs the rung-6 equilibrium gas (Gas::reacting_equilibrium())"
        );
        assert!(
            p9 <= pt9 * (1.0 + 1e-12),
            "finite_rate_nozzle: back-pressure p9={p9:.0} Pa exceeds pt9={pt9:.0} Pa \
             (cannot expand to it)"
        );
        finite_rate.validate();
        let comp_entry = equilibrium_composition(far, tt4, pt4); // the FROZEN station-4 mixture

        let f = expand_nozzle(&comp_entry, far, tt9, pt9, p9, false); // (F)
        let r = expand_nozzle(&comp_entry, far, tt9, pt9, p9, true); // (R)
        let (t9i, v9i, _, t_star) = irreversible_fast_expand(&comp_entry, far, tt9, pt9, p9); // (I)
        let d = finite_rate_expand(
            &comp_entry,
            far,
            tt9,
            pt9,
            p9,
            finite_rate.da,
            finite_rate.nstep,
        );

        FiniteRateNozzleState {
            da: finite_rate.da,
            t9_frozen: f.t9,
            v9_frozen: f.v9,
            t9_irrev_fast: t9i,
            v9_irrev_fast: v9i,
            t_star_entry: t_star,
            t9_reversible: r.t9,
            v9_reversible: r.v9,
            t9_finite: d.t9,
            v9_finite: d.v9,
            ds_finite: d.ds,
            co_fraction_entry: co_fraction(&comp_entry),
            co_fraction_finite_exit: co_fraction(&d.comp9),
        }
    }

    /// Freeze-out nozzle-flow diagnostic (rung 26).
    ///
    /// Rung 25 resolved the finite-rate flow with a single normalised `Da`, a cartoon that slides
    /// the whole expansion uniformly and CANNOT show freeze-out. This replaces it with a LOCAL
    /// `Da(T,p) = τ_res/τ_chem(T,p)` from an ANCHORED clock (GRI-Mech's `H+OH+M` sink; zero new
    /// constants), so the relaxation SHUTS OFF partway down the nozzle — and the shut-off point
    /// MOVES with `Tt4`: lean it never switches on, hot it crosses mid-expansion and later as
    /// `Tt4` climbs.
    ///
    /// `τ_res` is pinned to the FROZEN/cycle `V9`, not to the freeze-out output — that would be a
    /// fixed-point coupling. The freeze EXISTS / is ABSENT / MOVES are the certified claims; the
    /// LOCATION rides on the geometric knob `l` and is disclaimed.
    #[allow(clippy::too_many_arguments)]
    pub fn freeze_out_nozzle(
        &self,
        far: f64,
        tt4: f64,
        pt4: f64,
        tt9: f64,
        pt9: f64,
        p9: f64,
        freeze_out: FreezeOut,
    ) -> FreezeOutNozzleState {
        assert!(
            self.is_equilibrium(),
            "freeze_out_nozzle: needs the rung-6 equilibrium gas (Gas::reacting_equilibrium())"
        );
        assert!(
            p9 <= pt9 * (1.0 + 1e-12),
            "freeze_out_nozzle: back-pressure p9={p9:.0} Pa exceeds pt9={pt9:.0} Pa \
             (cannot expand to it)"
        );
        freeze_out.validate();
        let comp_entry = equilibrium_composition(far, tt4, pt4); // the FROZEN station-4 mixture

        let f = expand_nozzle(&comp_entry, far, tt9, pt9, p9, false); // (F)
        let (t9i, v9i, _, _) = irreversible_fast_expand(&comp_entry, far, tt9, pt9, p9); // (I)

        let tau_res = freeze_out.l / (0.6 * f.v9); // pinned to FROZEN V9 (no fixed-point coupling)
        let rs = freeze_out.rate_scale;
        // Da_local = rate_scale · τ_res / τ_chem(T,p; comp). τ_chem→∞ (x_OH≤0) ⇒ Da_local→0.
        let da_local = move |comp: &[(&'static str, f64)], t: f64, p: f64| {
            rs * tau_res / tau_chem_recomb(comp, t, p, None, None)
        };

        let (d, s_freeze, da_entry, da_exit) = freeze_out_expand(
            &comp_entry,
            far,
            tt9,
            pt9,
            p9,
            &da_local,
            freeze_out.nstep,
            None,
        );

        FreezeOutNozzleState {
            t9_frozen: f.t9,
            v9_frozen: f.v9,
            t9_irrev_fast: t9i,
            v9_irrev_fast: v9i,
            t9_freeze: d.t9,
            v9_freeze: d.v9,
            ds_freeze: d.ds,
            s_freeze,
            da_entry,
            da_exit,
            co_fraction_entry: co_fraction(&comp_entry),
            co_fraction_freeze_exit: co_fraction(&d.comp9),
        }
    }
}
