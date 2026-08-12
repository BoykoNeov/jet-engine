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
use crate::nox::{
    equilibrium_no_fraction, expand_nozzle, k_zeldovich, mix_entropy_molar, mix_h_abs_b,
    mix_mass_per_air, ZonedNoxOpts, T_EXIT_FLOOR,
};

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
    /// **This pair carries the one thing the oracle structurally cannot say: whether the loop
    /// CONVERGED.** Slice E classified its own `Expansion::iters` as a NAMING key, because `t9`
    /// is gated at bit-equality and so a mis-shaped loop is already caught by the value. That
    /// remains true — but `used == 200` means the bracket never met its stopping rule, and *that*
    /// is invisible in the result: `0.5*(lo+hi)` off an unconverged bracket is a perfectly
    /// plausible temperature. A Python↔Rust dump cannot see it either, since both sides would
    /// agree on the same unconverged number.
    ///
    /// Measured 36–37 across all 70 marches of § 4.11 probe 1, and gated in
    /// `tests/rung25.rs::the_energy_bisection_converges_far_inside_its_cap`. An earlier draft of
    /// this slice computed both fields and read them nowhere, while the oracle's header described
    /// an `iters/` key class the dump never emitted.
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

// ------------------------------------------------------------------------------------------- //
// Rung 27 — NO FREEZE-OUT: is the frozen-NO assumption every NO number carries EARNED?
// ------------------------------------------------------------------------------------------- //

/// Anchored super-equilibrium NO-destruction clock (rung 27) — the relaxation time of
/// super-equilibrium exhaust NO back toward its LOCAL equilibrium, on the extended-Zeldovich
/// REVERSE reactions:
///
/// ```text
/// τ_NO = 1 / ( 2 ( k2r·[O] + k3r·[H] ) ) = 1 / ( 2 c_tot ( k2r·x_O + k3r·x_H ) )
/// ```
///
/// Built from rung 7's OWN Hanson & Salimian constants — zero new constants, as rung 26. This is
/// the `a >> 1` limit of the full rung-7 rate, and it is `[NO]_e`- AND `a`-INDEPENDENT, so the
/// freeze answer does not depend on which frozen NO level is fed in.
///
/// **CONTRAST rung 26's clock, which is the whole point of the rung.** That one has `Ea = 0` (so
/// it ACCELERATES on cooling) and is termolecular (`c_tot^2`); this one is Arrhenius (so it
/// CRATERS on cooling) and bimolecular (`c_tot^1`). Both its factors AGREE — both drive freezing —
/// so the kill test INVERTS rung 26's, where density won DESPITE an opposing rate constant.
///
/// Evaluated on the FROZEN (radical-rich) pool, which is the FASTEST possible relaxation, so
/// `Da_NO` comes out an UPPER bound — the same bounding logic rung 26 used for `x_OH`. Returns
/// `+inf` when `[O] = [H] = 0`. `kill_t` pins T in the rate constants only; `kill_c` pins the total
/// concentration in `[O]`,`[H]` only. Each leaves the OTHER live.
///
/// **The premise rung 27 gave for the `a >> 1` form is FALSE at the nozzle entry** — NO arrives
/// SUB-equilibrium there and initially tries to FORM. See [`tau_no_exact`] for the repair, which
/// is what actually justifies this surrogate.
pub fn tau_no_destroy(
    comp: &[(&str, f64)],
    t: f64,
    p: f64,
    kill_t: Option<f64>,
    kill_c: Option<f64>,
) -> f64 {
    let ntot: f64 = comp.iter().map(|&(_, n)| n).sum();
    if ntot <= 0.0 {
        return f64::INFINITY;
    }
    let c_tot = p / (RU * t); // mol/m^3 (SI — `k_zeldovich` returns SI)
    let c_use = kill_c.unwrap_or(c_tot);
    let tk = kill_t.unwrap_or(t);
    let c_o = get_or_zero(comp, "O") / ntot * c_use;
    let c_h = get_or_zero(comp, "H") / ntot * c_use;
    let denom = 2.0 * (k_zeldovich("2r", tk) * c_o + k_zeldovich("3r", tk) * c_h);
    if denom > 0.0 {
        1.0 / denom
    } else {
        f64::INFINITY
    }
}

/// The EXACT local linearised NO relaxation time at the actual local `a` (rung 28) — the check
/// that certifies rung 27's `a >> 1` surrogate is a genuine bound. Returns `(tau_exact, beta, a)`.
///
/// ```text
/// d[NO]/dt = 2R1(1-a^2)/(1+beta*a),   a = [NO]/[NO]_e,   beta = R1/(R2+R3)
/// => tau_exact = [NO]_e (1+beta*a)^2 / ( 2 R1 (2a + beta*a^2 + beta) )
/// ```
///
/// Its limits: `a → inf` gives exactly [`tau_no_destroy`]; `a → 0` gives that surrogate divided by
/// `beta^2`. In between, with `u = beta*a`, the ratio is `(1+u)^2/[(1+u)^2 − (1−beta^2)] > 1` for
/// every `a >= 0` whenever `beta < 1`. So the surrogate is the fast asymptote approached from
/// ABOVE: a uniform LOWER bound on tau in BOTH regimes, which is what rung 27's bound claim needed
/// and what its "arrives super-equilibrium" premise did not supply.
///
/// **`beta < 1` is EMPIRICAL, not a theorem**, and remains the honest weak point. Measured over the
/// path × the design ladder it spans 0.0022 … 0.5429, and the source's own sweep of the whole
/// runnable `(Tt4, pi_c)` plane maxes at 0.5444; beta is exactly pressure-invariant and both of
/// `pi_c`'s indirect channels push it DOWN.
///
/// **`(1.0 + beta*a) ** 2` is an INTEGER exponent, so it spells as a PRODUCT** — the exact inverse
/// of [`N_HOHM`] earlier in this module, which is a float constant and must reach libm `pow`. Both
/// rules live here, which is why each site restates which one applies.
///
/// Returns `(inf, 0, 0)` when the rate degenerates (`[NO]_e`, `R1` or `R2+R3` <= 0). That branch is
/// DORMANT at every shipped condition — 0 of 55 sampled cells — so `tests/rung28.rs` reaches it by
/// forcing the radicals to zero rather than gating it only from the accepting side.
pub fn tau_no_exact(comp: &[(&str, f64)], t: f64, p: f64, x_no: f64) -> (f64, f64, f64) {
    let ntot: f64 = comp.iter().map(|&(_, n)| n).sum();
    if ntot <= 0.0 {
        return (f64::INFINITY, 0.0, 0.0);
    }
    let c_tot = p / (RU * t);
    let c_o = get_or_zero(comp, "O") / ntot * c_tot;
    let c_h = get_or_zero(comp, "H") / ntot * c_tot;
    let c_n2 = get_or_zero(comp, "N2") / ntot * c_tot;
    let c_noe = equilibrium_no_fraction(comp, t) * c_tot;
    if c_noe <= 0.0 {
        return (f64::INFINITY, 0.0, 0.0);
    }
    let r1 = k_zeldovich("1f", t) * c_o * c_n2;
    let r2 = k_zeldovich("2r", t) * c_noe * c_o;
    let r3 = k_zeldovich("3r", t) * c_noe * c_h;
    if r1 <= 0.0 || (r2 + r3) <= 0.0 {
        return (f64::INFINITY, 0.0, 0.0);
    }
    let beta = r1 / (r2 + r3);
    let a = x_no * c_tot / c_noe;
    let u = 1.0 + beta * a; // `** 2` on an INT exponent => a product, not `powp`
    let tau = c_noe * (u * u) / (2.0 * r1 * (2.0 * a + beta * a * a + beta));
    (tau, beta, a)
}

/// The frozen-composition isentropic temperature at pressure `p` — byte-identical to
/// `expand_nozzle(shifting=false)`'s bisection (same bracket, same `1e-13` tolerance, same loop
/// shape), so an exit temperature from here matches `nozzle_flow` bit-for-bit.
///
/// That is the reduce hinge for rungs 27 and 28, and § 4.13 prediction 1 measured it holding at
/// all five design points. It is a COPY of the loop rather than a second route to the same number,
/// which is why the port predicted it would survive before measuring.
///
/// **THIS ONE MERGE IS SAFE, AND THE DISTINCTION FROM THE RUNG-25/26 DUPLICATION IS THE POINT.**
/// The Python carries two textual copies of this bisection, one inside `_no_freeze_out_expand` and
/// one inside `_frozen_no_trajectory`, and the port merges them. That is the opposite of the
/// module header's "do not factor" rule, so it needs a reason rather than a preference: **what the
/// rung-28 reduce tests is not this helper.** The reduce compares two independently written NO
/// marches — their loop order, their `max_a` tracking, their relaxation expression, their clock
/// calls — and merging a shared temperature lookup leaves every one of those still separately
/// spelled. The rung-25/26 case is different in kind: there the ENTIRE loop is the thing under
/// comparison, so merging it would leave the gate comparing a function to itself. The test is
/// always "would the merge make the gate trivial", never "is duplication virtuous".
fn frozen_t_at(comp_entry: &[(&str, f64)], s_entry: f64, tt9: f64, p: f64) -> f64 {
    let (mut lo, mut hi) = (T_EXIT_FLOOR, tt9);
    for _ in 0..200 {
        let tm = 0.5 * (lo + hi);
        if mix_entropy_molar(comp_entry, tm, p) > s_entry {
            hi = tm;
        } else {
            lo = tm;
        }
        if hi - lo <= 1e-13 * tm {
            break;
        }
    }
    0.5 * (lo + hi)
}

/// One NO FREEZE-OUT march (rung 27) — a SINGLE scalar (the NO mole fraction) relaxed along
/// rung-14's FROZEN isentropic nozzle path toward the LOCAL equilibrium NO:
///
/// ```text
/// x_no <- x_no + (1 - exp(-Da_NO_local*ds))*(x_NO_e(T) - x_no)
/// ```
///
/// Unlike rung 26's march this carries no energy spine — NO is a trace diagnostic riding on the
/// frozen major pool. `Da_NO_local -> 0` leaves NO frozen at entry (the reduce, and the clamp then
/// equals rung 14/17's bit-for-bit); `-> inf` makes NO track `x_NO_e` down and the clamp go
/// dormant. The FINDING is that the anchored `Da_NO << 1` everywhere, so the first branch is the
/// physical one and the frozen-NO assumption carried since rung 7 is DERIVED, not assumed.
///
/// `max_a` is tracked over the WHOLE trajectory. The source hedges that "a relaxed one may peak
/// earlier" than the cold exit — measured over 5 design points × 4 rate scales spanning `1e-12` to
/// `1e12`, it never does: the peak is at the exit in all 20 cells, including where NO is 97 %
/// relaxed. Returns `(t9, x_no_exit, x_no_e_exit, max_a, da_entry, da_exit)`.
pub fn no_freeze_out_expand(
    comp_entry: &[(&'static str, f64)],
    tt9: f64,
    pt9: f64,
    p9: f64,
    x_no_entry: f64,
    da_no_fn: DaLocalFn,
    nstep: usize,
) -> (f64, f64, f64, f64, f64, f64) {
    let s_entry = mix_entropy_molar(comp_entry, tt9, pt9);
    let lnr = (p9 / pt9).ln();
    let ds = 1.0 / nstep as f64;

    let mut x_no = x_no_entry;
    let da_entry = da_no_fn(comp_entry, tt9, pt9);
    let mut max_a = 0.0f64;
    for k in 0..nstep {
        let p0 = pt9 * (lnr * k as f64 * ds).exp();
        let t0 = if k == 0 { tt9 } else { frozen_t_at(comp_entry, s_entry, tt9, p0) };
        let x_no_e0 = equilibrium_no_fraction(comp_entry, t0);
        if x_no_e0 > 0.0 {
            max_a = max_a.max(x_no / x_no_e0); // trajectory a, BEFORE this step's relax
        }
        let da_local = da_no_fn(comp_entry, t0, p0);
        let relax = 1.0 - (-da_local * ds).exp();
        x_no += relax * (x_no_e0 - x_no);
    }
    let t9 = frozen_t_at(comp_entry, s_entry, tt9, p9);
    let x_no_e_exit = equilibrium_no_fraction(comp_entry, t9);
    if x_no_e_exit > 0.0 {
        max_a = max_a.max(x_no / x_no_e_exit); // the cold exit — where a frozen NO peaks
    }
    let da_exit = da_no_fn(comp_entry, t9, p9);
    (t9, x_no, x_no_e_exit, max_a, da_entry, da_exit)
}

// ------------------------------------------------------------------------------------------- //
// Rung 28 — THE COUPLED NO MARCH: rung 27's clock on rung 26's RELAXING pool
// ------------------------------------------------------------------------------------------- //

/// Rung-27's FROZEN nozzle path as an explicit trajectory (rung 28).
///
/// Same pressure grid as [`freeze_out_expand`] / [`no_freeze_out_expand`] and the same entropy
/// bisection — and, as in rung 27, the `k = 0` temperature is `tt9` EXACTLY rather than a bisection
/// at `pt9`. Composition is `comp_entry` at every station, frozen by definition.
///
/// Feeding this to [`coupled_no_march`] reproduces [`no_freeze_out_expand`] BIT-FOR-BIT — the
/// rung-28 reduce, structural rather than numerical. § 4.13 prediction 2 measured it at 10/10, and
/// it holds *despite* rung 27 computing `equilibrium_no_fraction` once per step where rung 28
/// computes it twice: same function, same arguments, same bits. **A COPY is about the arithmetic
/// performed, not the syntax.**
pub fn frozen_no_trajectory(
    comp_entry: &[(&'static str, f64)],
    tt9: f64,
    pt9: f64,
    p9: f64,
    nstep: usize,
) -> Vec<MarchStation> {
    let s_entry = mix_entropy_molar(comp_entry, tt9, pt9);
    let lnr = (p9 / pt9).ln();
    let ds = 1.0 / nstep as f64;

    let mut traj = Vec::with_capacity(nstep + 1);
    for k in 0..nstep {
        let p0 = pt9 * (lnr * k as f64 * ds).exp();
        let t = if k == 0 { tt9 } else { frozen_t_at(comp_entry, s_entry, tt9, p0) };
        traj.push(MarchStation { s: k as f64 * ds, p: p0, t, comp: comp_entry.to_vec() });
    }
    let t_exit = frozen_t_at(comp_entry, s_entry, tt9, p9);
    traj.push(MarchStation { s: 1.0, p: p9, t: t_exit, comp: comp_entry.to_vec() });
    traj
}

/// The rung-28 COUPLED NO march — rung 27's trace-NO relaxation reading its clock off a SUPPLIED
/// trajectory instead of the hard-wired frozen station-4 pool.
///
/// Rung 27 deferred this with the note that coupling to rung 26's relaxing pool "can ONLY slow NO
/// further (radical-poorer => larger tau_NO)". **That is ONE-SIDED**: coupling to rung 26 couples
/// to ALL of rung 26, INCLUDING its exothermic heat release, which lifts `T(s)` above the frozen
/// isentrope — and because this clock is Arrhenius, that SPEEDS NO destruction. Two opposing
/// channels:
///
/// 1. **radical depletion** — `[O]`,`[H]` recombine => `tau_NO` rises => DEEPER frozen;
/// 2. **heat release** — `T(s)` above frozen => Arrhenius `k` rises => LESS frozen.
///
/// Taking `clock_traj` as a PARAMETER is what makes the decomposition first-class rather than a
/// probe artefact: pass the frozen trajectory for rung 27, the freeze-out one for the coupled
/// march, and the two hybrids to isolate each channel. Coupling only the composition would
/// structurally exclude channel 2 and thereby manufacture rung 27's tidy prediction.
///
/// `ref_traj` supplies the clamp DENOMINATOR path and is held on the FROZEN nozzle deliberately:
/// the coupled exit is warmer, so an equilibrium NO read there would move `max_a` for a purely
/// THERMODYNAMIC reason and entangle it with the kinetic finding.
///
/// ONE-WAY by construction: NO is a trace species and is never fed back into the pool.
pub fn coupled_no_march(
    clock_traj: &[MarchStation],
    ref_traj: &[MarchStation],
    x_no_entry: f64,
    da_no_fn: DaLocalFn,
) -> (f64, f64, f64, f64, f64, f64) {
    assert_eq!(
        ref_traj.len(),
        clock_traj.len(),
        "coupled_no_march: trajectory length mismatch ({} vs {})",
        clock_traj.len(),
        ref_traj.len()
    );
    let nstep = clock_traj.len() - 1;
    let ds = 1.0 / nstep as f64;
    let mut x_no = x_no_entry;
    let mut max_a = 0.0f64;
    let da_entry = da_no_fn(&clock_traj[0].comp, clock_traj[0].t, clock_traj[0].p);
    for k in 0..nstep {
        let (p0, t_clock) = (clock_traj[k].p, clock_traj[k].t);
        let comp_clock = &clock_traj[k].comp;
        let x_no_e_ref = equilibrium_no_fraction(&ref_traj[k].comp, ref_traj[k].t);
        if x_no_e_ref > 0.0 {
            max_a = max_a.max(x_no / x_no_e_ref); // trajectory a, BEFORE this step's relax
        }
        let da_local = da_no_fn(comp_clock, t_clock, p0);
        let relax = 1.0 - (-da_local * ds).exp();
        x_no += relax * (equilibrium_no_fraction(comp_clock, t_clock) - x_no);
    }
    let last = &clock_traj[nstep];
    let ref_last = &ref_traj[nstep];
    let x_no_e_ref_exit = equilibrium_no_fraction(&ref_last.comp, ref_last.t);
    if x_no_e_ref_exit > 0.0 {
        max_a = max_a.max(x_no / x_no_e_ref_exit); // the cold exit — where a frozen NO peaks
    }
    let da_exit = da_no_fn(&last.comp, last.t, last.p);
    (ref_last.t, x_no, x_no_e_ref_exit, max_a, da_entry, da_exit)
}

// ------------------------------------------------------------------------------------------- //
// The rung-27 / rung-28 configs, states and `Gas` entry points
// ------------------------------------------------------------------------------------------- //

/// Rung-27 NO-freeze-out config: rung 26's anchored-clock machinery on a NO-destruction clock.
#[derive(Debug, Clone, Copy)]
pub struct NoFreezeOut {
    /// residence length, m — sets `tau_res`, as rung 26
    pub l: f64,
    /// NO-relaxation march resolution
    pub nstep: usize,
    /// dimensionless `Da_NO` multiplier for the limit gates (1.0 = anchored)
    pub rate_scale: f64,
}

impl Default for NoFreezeOut {
    fn default() -> Self {
        Self { l: 0.5, nstep: 400, rate_scale: 1.0 }
    }
}

impl NoFreezeOut {
    /// Python's `__post_init__`.
    pub fn validate(&self) {
        assert!(self.l > 0.0, "NOFreezeOut.L={} must be positive", self.l);
        assert!(self.nstep >= 100, "NOFreezeOut.nstep={} too coarse (need >= 100)", self.nstep);
        assert!(
            self.rate_scale > 0.0,
            "NOFreezeOut.rate_scale={} must be positive",
            self.rate_scale
        );
    }
}

/// Rung-27 NO-freeze-out diagnostic. A pure diagnostic BESIDE the cycle.
#[derive(Debug, Clone)]
pub struct NoFreezeOutNozzleState {
    /// frozen nozzle exit static T (== `nozzle_flow`'s `t9_frozen`), K
    pub t9_frozen: f64,
    /// `Da_NO` at the nozzle entry — `<< 1` means frozen from entry (the finding)
    pub da_entry: f64,
    /// `Da_NO` at the exit (falls further — the kill test's both-factors-agree)
    pub da_exit: f64,
    /// entry exhaust NO fed through (the rung-8 zoned mole fraction)
    pub x_no_frozen: f64,
    /// exit NO after the anchored march (it barely moves)
    pub x_no_relaxed: f64,
    /// equilibrium NO at the entry
    pub x_no_e_entry: f64,
    /// equilibrium NO at the exit (collapsed — the clamp denominator)
    pub x_no_e_exit: f64,
    /// max `[NO]/[NO]_e` over the RELAXED march (the real clamp margin)
    pub max_a: f64,
    /// max `[NO]/[NO]_e` if NO were fully frozen (== rung 14/17's number)
    pub max_a_frozen: f64,
}

impl NoFreezeOutNozzleState {
    /// True when `Da_NO < 1` at the entry — NO never relaxes. TRUE at every `Tt4`, which is the
    /// rung: unlike the major pool, NO is frozen from entry everywhere.
    pub fn frozen_from_entry(&self) -> bool {
        self.da_entry < 1.0
    }

    /// Does the dropped clamp fire (super-equilibrium NO at the exit)?
    pub fn clamp_fires(&self) -> bool {
        self.max_a > 1.0
    }

    /// How far exhaust NO relaxed toward equilibrium: 0 = fully frozen (the anchored finding),
    /// 1 = fully equilibrated (`rate_scale` → ∞).
    ///
    /// It can come out slightly NEGATIVE at the hot anchored points, and that is physics rather
    /// than noise: `a < 1` at the entry, so NO arrives SUB-equilibrium and initially FORMS. That
    /// is rung 28's erratum showing up in rung 27's own output.
    pub fn relaxed_fraction(&self) -> f64 {
        let num = self.x_no_frozen - self.x_no_relaxed;
        let den = self.x_no_frozen - self.x_no_e_exit;
        if den.abs() > 0.0 {
            num / den
        } else {
            0.0
        }
    }
}

/// Rung-28 coupled NO-freeze-out config.
#[derive(Debug, Clone, Copy)]
pub struct CoupledNoFreezeOut {
    /// residence length, m
    pub l: f64,
    /// shared march resolution (both trajectories on one pressure grid)
    pub nstep: usize,
    /// NO-clock `Da_NO` multiplier (1.0 = anchored; → 0 gives the reduce)
    pub rate_scale: f64,
    /// rung-26 recombination-clock multiplier (1.0 = anchored; → ∞ is the structural gate)
    pub pool_rate_scale: f64,
}

impl Default for CoupledNoFreezeOut {
    fn default() -> Self {
        Self { l: 0.5, nstep: 400, rate_scale: 1.0, pool_rate_scale: 1.0 }
    }
}

impl CoupledNoFreezeOut {
    /// Python's `__post_init__`.
    pub fn validate(&self) {
        assert!(self.l > 0.0, "CoupledNOFreezeOut.L={} must be positive", self.l);
        assert!(
            self.nstep >= 100,
            "CoupledNOFreezeOut.nstep={} too coarse (need >= 100)",
            self.nstep
        );
        assert!(
            self.rate_scale > 0.0,
            "CoupledNOFreezeOut.rate_scale={} must be positive",
            self.rate_scale
        );
        assert!(
            self.pool_rate_scale > 0.0,
            "CoupledNOFreezeOut.pool_rate_scale={} must be positive (an UNRELAXED pool is \
             rung 27 — reach it with couple=false, not with pool_rate_scale=0)",
            self.pool_rate_scale
        );
    }
}

/// Rung-28 coupled NO-freeze-out diagnostic.
///
/// **READ THE RATIOS AS THE CLOCK'S DEPTH, NOT NO's MOTION** — NO does not move
/// (`relaxed_fraction` is ~0).
#[derive(Debug, Clone)]
pub struct CoupledNoFreezeOutState {
    /// frozen nozzle exit static T (== `nozzle_flow`'s), K
    pub t9_frozen: f64,
    /// rung-26 freeze-out exit static T, K — WARMER, because of the heat release
    pub t9_pool: f64,
    /// rung-26's pool freeze point — the INTERLOCK that gates the coupling
    pub s_freeze_pool: f64,
    /// `Da_NO` at entry — path-INDEPENDENT, so bit-for-bit rung 27's
    pub da_entry: f64,
    /// rung-27 baseline: frozen T + frozen composition
    pub da_exit_frozen: f64,
    /// channel 1 alone: frozen T + coupled composition
    pub da_exit_depletion: f64,
    /// channel 2 alone: coupled T + frozen composition
    pub da_exit_heat: f64,
    /// the rung-28 march: coupled T + coupled composition
    pub da_exit_coupled: f64,
    /// `(x_O + x_H)` at the nozzle entry (frozen, radical-rich)
    pub x_radical_entry: f64,
    /// `(x_O + x_H)` at the exit after rung-26 relaxation (depleted)
    pub x_radical_exit_pool: f64,
    /// entry exhaust NO fed through
    pub x_no_frozen: f64,
    /// exit NO after the COUPLED march
    pub x_no_relaxed: f64,
    /// equilibrium NO at the frozen exit (the clamp denominator)
    pub x_no_e_exit: f64,
    /// max `[NO]/[NO]_e` over the coupled march
    pub max_a: f64,
    /// max `[NO]/[NO]_e` if NO were fully frozen (== rung 14/17/27's number)
    pub max_a_frozen: f64,
    /// `[NO]/[NO]_e` at the nozzle ENTRY — SUB-equilibrium (< 1) for `Tt4` ≥ 1800 K
    pub a_entry: f64,
    /// `[NO]/[NO]_e` at the exit — super-equilibrium, where the clamp is read
    pub a_exit: f64,
    /// max `β = R1/(R2+R3)` over the path; `< 1` means the surrogate bounds the rate
    pub beta_max: f64,
    /// min `τ_exact/τ_surrogate` over the path; `≥ 1` means the bound holds pointwise
    pub tau_ratio_min: f64,
}

impl CoupledNoFreezeOutState {
    /// Channel 1: how much the radical depletion slows the clock.
    pub fn depletion_factor(&self) -> f64 {
        if self.da_exit_frozen > 0.0 {
            self.da_exit_depletion / self.da_exit_frozen
        } else {
            1.0
        }
    }

    /// Channel 2: how much the heat release speeds it back up.
    pub fn heat_release_factor(&self) -> f64 {
        if self.da_exit_frozen > 0.0 {
            self.da_exit_heat / self.da_exit_frozen
        } else {
            1.0
        }
    }

    /// The two channels together.
    pub fn net_factor(&self) -> f64 {
        if self.da_exit_frozen > 0.0 {
            self.da_exit_coupled / self.da_exit_frozen
        } else {
            1.0
        }
    }

    /// `|ln(ch2)/ln(ch1)|` — how much of the depletion effect the heat release cancels, in the log
    /// space where the two channels compose. Rises MONOTONICALLY with `Tt4`, which is the
    /// certified trend; the NET's non-monotone turnaround is NOT claimed.
    pub fn channel_ratio(&self) -> f64 {
        let (d, h) = (self.depletion_factor(), self.heat_release_factor());
        if d <= 0.0 || d == 1.0 || h <= 0.0 {
            return 0.0;
        }
        (h.ln() / d.ln()).abs()
    }

    /// Does the coupling push the clock DEEPER below the freeze threshold (rung 27's conclusion)?
    pub fn deeper_frozen(&self) -> bool {
        self.net_factor() < 1.0
    }

    /// Is rung 27's `a ≫ 1` surrogate a genuine bound on the rate along this path?
    pub fn surrogate_bounds_rate(&self) -> bool {
        self.beta_max < 1.0 && self.tau_ratio_min >= 1.0
    }

    /// Does NO arrive SUB-equilibrium at the nozzle entry? (The erratum's point.)
    pub fn sub_equilibrium_entry(&self) -> bool {
        self.a_entry < 1.0
    }

    /// True when `Da_NO < 1` at the entry. The entry state is PATH-INDEPENDENT, so this is rung
    /// 27's answer bit-for-bit.
    pub fn frozen_from_entry(&self) -> bool {
        self.da_entry < 1.0
    }

    /// Does the dropped clamp fire?
    pub fn clamp_fires(&self) -> bool {
        self.max_a > 1.0
    }

    /// How far exhaust NO relaxed toward equilibrium over the coupled march (~0).
    pub fn relaxed_fraction(&self) -> f64 {
        let num = self.x_no_frozen - self.x_no_relaxed;
        let den = self.x_no_frozen - self.x_no_e_exit;
        if den.abs() > 0.0 {
            num / den
        } else {
            0.0
        }
    }
}

/// `(x_O + x_H)` — the radical fraction whose depletion is rung 28's channel 1.
fn radical_fraction(comp: &[(&str, f64)]) -> f64 {
    let n: f64 = comp.iter().map(|&(_, v)| v).sum();
    if n > 0.0 {
        (get_or_zero(comp, "O") + get_or_zero(comp, "H")) / n
    } else {
        0.0
    }
}

impl Gas {
    /// NO-freeze-out nozzle diagnostic (rung 27).
    ///
    /// Every NO number since rung 7 ASSUMES the station-4 exhaust NO freezes through the nozzle,
    /// and the rung-14/17 dropped-clamp corollary reads `max_a` OFF that assumption. Rung 26 then
    /// showed the MAJOR pool freezes only partway down. This asks the same of NO — and finds the
    /// assumption is EARNED: `Da_NO ≪ 1` from entry at EVERY `Tt4`, so the clamp firing is derived,
    /// on an upper bound (the frozen radical-rich pool is the fastest possible relaxation).
    ///
    /// NO LOCATION / moving-freeze-point is claimed — rung 26's headline has no analogue here,
    /// because NO is frozen from entry everywhere.
    #[allow(clippy::too_many_arguments)]
    pub fn no_freeze_out_nozzle(
        &self,
        far: f64,
        tt3: f64,
        tt4: f64,
        pt4: f64,
        tt9: f64,
        pt9: f64,
        p9: f64,
        phi_primary: f64,
        no_freeze_out: NoFreezeOut,
    ) -> NoFreezeOutNozzleState {
        assert!(
            self.is_equilibrium(),
            "no_freeze_out_nozzle: needs the rung-6 equilibrium gas (Gas::reacting_equilibrium())"
        );
        assert!(
            p9 <= pt9 * (1.0 + 1e-12),
            "no_freeze_out_nozzle: back-pressure p9={p9:.0} Pa exceeds pt9={pt9:.0} Pa \
             (cannot expand to it)"
        );
        no_freeze_out.validate();
        let comp_entry = equilibrium_composition(far, tt4, pt4);

        // The clamp-relevant frozen exhaust NO: the rung-8 zoned mole fraction — what rungs 14/17
        // carry through the nozzle, and what arrives SUPER-equilibrium at the cold exit.
        let zn = self.zoned_nox(far, tt3, tt4, pt4, phi_primary, ZonedNoxOpts::default());
        let x_no_frozen = zn.x_no_mix;

        // The rung-14 reference. `nozzle_flow` is UNTOUCHED — this only reads it.
        let nf = self.nozzle_flow(far, tt4, pt4, tt9, pt9, p9, Some(x_no_frozen));

        let tau_res = no_freeze_out.l / (0.6 * nf.v9_frozen); // pinned to FROZEN V9, as rung 26
        let rs = no_freeze_out.rate_scale;
        let da_no = move |comp: &[(&'static str, f64)], t: f64, p: f64| {
            rs * tau_res / tau_no_destroy(comp, t, p, None, None)
        };

        let (_t9, x_no_relaxed, _x_no_e_exit, max_a, da_entry, da_exit) = no_freeze_out_expand(
            &comp_entry,
            tt9,
            pt9,
            p9,
            x_no_frozen,
            &da_no,
            no_freeze_out.nstep,
        );

        NoFreezeOutNozzleState {
            t9_frozen: nf.t9_frozen,
            da_entry,
            da_exit,
            x_no_frozen,
            x_no_relaxed,
            x_no_e_entry: nf.x_no_e_entry,
            x_no_e_exit: nf.x_no_e_exit,
            max_a,
            max_a_frozen: nf.max_a.expect("nozzle_flow was given x_no_frozen"),
        }
    }

    /// Coupled NO-freeze-out nozzle diagnostic (rung 28).
    ///
    /// Rung 27 read its NO clock on the FROZEN station-4 pool and deferred the coupled march with
    /// the note that it "can ONLY slow NO further". Rung 28 builds it and finds that "only" was
    /// ONE-SIDED — see [`coupled_no_march`] for the two channels.
    ///
    /// THE VERDICT is a CONFIRMATION with a MECHANISTIC CORRECTION: rung 27's conclusion holds
    /// (`net_factor < 1` everywhere in band), its mechanism was incomplete (`heat_release_factor`
    /// exceeds 1 everywhere, cancelling nearly half the depletion at the hot edge), the win is
    /// STRUCTURAL rather than incidental (channel 1 is unbounded while channel 2 saturates), and
    /// the HEADLINE IS UNTOUCHED — the nozzle-entry state is path-independent, so `da_entry` is
    /// rung 27's bit-for-bit and NO stays frozen from entry.
    ///
    /// `couple = false` runs the NO clock on the FROZEN trajectory — i.e. rung 27 exactly, and the
    /// reduce is STRUCTURAL (the same expression sequence, not merely the same answer to a
    /// tolerance).
    #[allow(clippy::too_many_arguments)]
    pub fn coupled_no_freeze_out_nozzle(
        &self,
        far: f64,
        tt3: f64,
        tt4: f64,
        pt4: f64,
        tt9: f64,
        pt9: f64,
        p9: f64,
        phi_primary: f64,
        coupled: CoupledNoFreezeOut,
        couple: bool,
    ) -> CoupledNoFreezeOutState {
        assert!(
            self.is_equilibrium(),
            "coupled_no_freeze_out_nozzle: needs the rung-6 equilibrium gas \
             (Gas::reacting_equilibrium())"
        );
        assert!(
            p9 <= pt9 * (1.0 + 1e-12),
            "coupled_no_freeze_out_nozzle: back-pressure p9={p9:.0} Pa exceeds pt9={pt9:.0} Pa \
             (cannot expand to it)"
        );
        coupled.validate();
        let comp_entry = equilibrium_composition(far, tt4, pt4);
        let nstep = coupled.nstep;

        let zn = self.zoned_nox(far, tt3, tt4, pt4, phi_primary, ZonedNoxOpts::default());
        let x_no_frozen = zn.x_no_mix;
        let nf = self.nozzle_flow(far, tt4, pt4, tt9, pt9, p9, Some(x_no_frozen));

        let tau_res = coupled.l / (0.6 * nf.v9_frozen);
        let rate_scale = coupled.rate_scale;
        let pool_rate_scale = coupled.pool_rate_scale;
        let da_no = move |comp: &[(&'static str, f64)], t: f64, p: f64| {
            rate_scale * tau_res / tau_no_destroy(comp, t, p, None, None)
        };
        let da_pool = move |comp: &[(&'static str, f64)], t: f64, p: f64| {
            pool_rate_scale * tau_res / tau_chem_recomb(comp, t, p, None, None)
        };

        // (a) rung-14/27's FROZEN trajectory — the clamp-denominator reference and rung 27's clock.
        let frozen_traj = frozen_no_trajectory(&comp_entry, tt9, pt9, p9, nstep);

        // (b) rung-26's RELAXING trajectory, recorded by a PURE OBSERVER, so rung 26 stays
        //     bit-for-bit with and without the recording (gated in `tests/rung26.rs`).
        let mut pool_traj: Vec<MarchStation> = Vec::new();
        let (_pool, s_freeze_pool, _dae, _dax) = freeze_out_expand(
            &comp_entry,
            far,
            tt9,
            pt9,
            p9,
            &da_pool,
            nstep,
            Some(&mut pool_traj),
        );

        let clock_traj = if couple { &pool_traj } else { &frozen_traj };
        let (_t9_ref, x_no_relaxed, x_no_e_exit, max_a, da_entry, da_exit_coupled) =
            coupled_no_march(clock_traj, &frozen_traj, x_no_frozen, &da_no);

        // The channel decomposition, read at the exit on the two HYBRID paths. Coupling only the
        // composition would exclude channel 2 by construction, so BOTH hybrids are reported.
        let pool_ex = &pool_traj[nstep];
        let froz_ex = &frozen_traj[nstep];
        let da_exit_frozen = da_no(&comp_entry, froz_ex.t, pool_ex.p); // rung-27 baseline
        let da_exit_depl = da_no(&pool_ex.comp, froz_ex.t, pool_ex.p); // channel 1 alone
        let da_exit_heat = da_no(&comp_entry, pool_ex.t, pool_ex.p); // channel 2 alone

        // The β repair: certify that rung-27's surrogate really does bound the rate, on the frozen
        // reference path where the freeze verdict is read.
        let (mut beta_max, mut tau_ratio_min) = (0.0f64, f64::INFINITY);
        let (mut a_entry, mut a_exit) = (0.0, 0.0);
        for i in 0..11 {
            let st = &frozen_traj[(i * nstep / 10).min(nstep)];
            let (tau_e, beta_i, a_i) = tau_no_exact(&st.comp, st.t, st.p, x_no_frozen);
            let tau_s = tau_no_destroy(&st.comp, st.t, st.p, None, None);
            beta_max = beta_max.max(beta_i);
            if tau_s > 0.0 && tau_e.is_finite() {
                tau_ratio_min = tau_ratio_min.min(tau_e / tau_s);
            }
            if i == 0 {
                a_entry = a_i;
            }
            a_exit = a_i;
        }
        if !tau_ratio_min.is_finite() {
            // DORMANT at every shipped condition — 0 of 55 sampled cells reach it. Gated from the
            // REFUSING side in `tests/rung28.rs` rather than only from the accepting one.
            tau_ratio_min = 1.0;
        }

        CoupledNoFreezeOutState {
            t9_frozen: nf.t9_frozen,
            t9_pool: pool_ex.t,
            s_freeze_pool,
            da_entry,
            da_exit_frozen,
            da_exit_depletion: da_exit_depl,
            da_exit_heat,
            da_exit_coupled,
            x_radical_entry: radical_fraction(&comp_entry),
            x_radical_exit_pool: radical_fraction(&pool_ex.comp),
            x_no_frozen,
            x_no_relaxed,
            x_no_e_exit,
            max_a,
            max_a_frozen: nf.max_a.expect("nozzle_flow was given x_no_frozen"),
            a_entry,
            a_exit,
            beta_max,
            tau_ratio_min,
        }
    }
}
