//! SLICE N step 2 — the smoke check for [`StageStack`], against a Python dump of the SAME cells.
//!
//! Not the slice's oracle (that is step 4, on the 640-cell / 160-row grid, through the matcher).
//! This exists to catch a structural mistake — a swapped ladder, a `r ** k` spelled as a repeated
//! multiply, a floor applied after the value it guards, a capacity profile that round-trips row 0
//! through the bisection — BEFORE rung 55's matcher is written on top of it at step 3.
//!
//! # It enumerates the methods step 2 ships, and the enumeration found a hole
//!
//! Slice L step 3's smoke reached 1 of the 3 methods its own headline named; slice M's needed
//! `currency_split` twice because at `v = 0` it could not discriminate the sibling constructor.
//! The specific trap here is that **a stack built with `vsv_stages = None` never touches
//! `cmap_axial`**, so the second branch of both [`StageStack::psi_at`] and
//! [`StageStack::vsv_at`] is dead. Cells B and C exist to reach it, with a MOVED stator so the
//! two maps differ.
//!
//! The four cells cover: both [`Split`]s, both [`CapProfile`]s, the lumped and front-row levers,
//! a closed stator (`v > 0`) and an opened one (`v < 0`), `K` of 8/8/4/1, and
//! [`StageStack::solve_n`] on **both** the `K = 1` dispatch and the `K > 1` bisection.
//!
//! # And the floors: the branch a face grid does not reach, and a DERIVED threshold
//!
//! The first grid clamped NOTHING — including at `march(0.9, 0.1)`, which was picked on the
//! guess that `solve_n`'s LOW bracket end is the non-physical one. It is the HIGH `m/n` end:
//! `n_k^2` is tiny at `n = 0.1`, so `tau_k` stays near 1, while `march(8, 2)` drives **7 of 8**
//! stages to `_T_FLOOR`. Measured, and the guess was wrong.
//!
//! `_P_FLOOR` is then dead for a **derived** reason and not a grid one. `tau_k` is floored
//! BEFORE `base = 1 + e*(tau_k - 1)` is formed, so `base >= 1 - e*(1 - T_FLOOR)`, and
//! `base < P_FLOOR` requires
//!
//! ```text
//!     e > (1 - P_FLOOR)/(1 - T_FLOOR) = (1 - 1e-6)/(1 - 1e-3) = 1.001 EXACTLY
//! ```
//!
//! — a threshold in the two floor constants **alone**: independent of the map, the split, `K`
//! and the design point. That is a strictly stronger statement than § 5.10 (iii)'s measured
//! *0 firings in 521 649 marches*, which is a claim about a grid. Since `e = e_d*(eta_live/eta_d)`
//! and `e_d > eta_d`, the threshold sits just above any physical live efficiency — which is WHY
//! the count is zero. Pushed past it, both floors fire on the SAME stages and the shared counter
//! doubles exactly (7 → 14, 4 → 8), which is § 5.10 (iii)'s *the two can never be conflated*
//! shown rather than asserted.
//!
//! Regenerate with:
//!     .venv\Scripts\python.exe rust\oracle\dump_slice_n_smoke.py > oracle\slice_n_smoke_pypy.tsv

use std::collections::HashMap;

use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stage::{take_census, CapProfile, Split, StageStack, StageStackCore,
                      StageStackCoreSpec, StageStackSpec};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE: &str = include_str!("../oracle/slice_n_smoke_pypy.tsv");

fn load() -> HashMap<&'static str, f64> {
    let mut out = HashMap::new();
    for line in ORACLE.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let mut it = line.split('\t');
        let key = it.next().expect("key");
        let hex = it.next().expect("hex");
        out.insert(key, parse_hex(hex));
    }
    out
}

/// Python's `float.hex()` — `[-]0x1.<mantissa>p<exp>` — parsed exactly, so the comparison is on
/// BITS and not on a decimal round-trip.
fn parse_hex(s: &str) -> f64 {
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let s = s.strip_prefix("0x").expect("0x prefix");
    let (mant, exp) = s.split_once('p').expect("p exponent");
    let exp: i32 = exp.parse().expect("exponent");
    let (int_part, frac) = match mant.split_once('.') {
        Some((i, f)) => (i, f),
        None => (mant, ""),
    };
    let mut v: f64 = int_part.parse::<u64>().expect("int part") as f64;
    let mut scale = 1.0f64 / 16.0;
    for ch in frac.chars() {
        v += (ch.to_digit(16).expect("hex digit") as f64) * scale;
        scale /= 16.0;
    }
    let out = v * (2.0f64).powi(exp);
    if neg { -out } else { out }
}

// ----------------------------------------------------------------------------- the anchor
//
// The design point cells A–D's `(tau_d, pi_d, eta_d, kc)` were taken from — a two-spool
// equilibrium design at `pi_lpc = 3`, `pi_hpc = 6`, `Tt4 = 1500`. They are LITERALS here rather
// than built through a matcher, because step 2 is `StageStack` and nothing else; the dump emits
// them as `anchor/*` rows so these copies are CHECKED against the machine that produced them
// rather than trusted (slice M's `slice_m_deferrals` discipline).

const TAU_LP: f64 = f64::from_bits(0x3FF68785F977C32E);
const PI_LP: f64 = 3.0;
const ETA_LP: f64 = 0.90;
const TAU_HP: f64 = f64::from_bits(0x3FFBA6C8B9D6AFB8);
const PI_HP: f64 = 6.0;
const ETA_HP: f64 = 0.88;
/// `gamma_c/(gamma_c - 1.0)` off the equilibrium gas's cold section — NOT the dataclass default
/// `3.5`, which is what `StageStackMatcher` hands the stack.
const KC: f64 = f64::from_bits(0x400C000000000001);

fn cell_a() -> StageStack {
    let cmap = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, vsv: 0.25, capacity: 0.80,
                              ..ComponentMap::flat() };
    StageStack::new(StageStackSpec { kc: KC, ..StageStackSpec::new(8, cmap, TAU_LP, PI_LP, ETA_LP) })
}

fn cell_b() -> StageStack {
    let cmap = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, vsv: 0.25, capacity: 0.80,
                              ..ComponentMap::flat() };
    StageStack::new(StageStackSpec {
        kc: KC, split: Split::Tau, vsv_stages: Some(1),
        ..StageStackSpec::new(8, cmap, TAU_LP, PI_LP, ETA_LP) })
}

fn cell_c() -> StageStack {
    let cmap = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, vsv: -0.15, capacity: 0.85,
                              ..ComponentMap::flat() };
    StageStack::new(StageStackSpec {
        kc: KC, vsv_stages: Some(2), cap_profile: CapProfile::Uniform,
        ..StageStackSpec::new(4, cmap, TAU_HP, PI_HP, ETA_HP) })
}

fn cell_d() -> StageStack {
    let cmap = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, capacity: 0.80,
                              ..ComponentMap::flat() };
    StageStack::new(StageStackSpec { kc: KC, ..StageStackSpec::new(1, cmap, TAU_LP, PI_LP, ETA_LP) })
}

/// THE POWER-SPELLING DISCRIMINATOR, and it exists because the cell that was meant to do this
/// job was nearly blind.
///
/// [`StageStack`]'s `ladder_t` under [`Split::Tau`] is the ONE place in rungs 55/56 where Python
/// raises to a **variable integer** exponent (`r ** k`) — every other `**` in the two rungs is
/// `0.5` or `kc`, neither of which has an alternative spelling. So it is the file's only genuine
/// power-spelling choice, between `powp(r, k as f64)`, a running product, and the tempting
/// "simplify the two powers into one" `powp(tau, k as f64 / k_total)`.
///
/// **Measured over 109 650 `(tau, K, k)` cells: the spellings differ on 34.8 % and 65.5 % of
/// them.** But at cell B's own `(tau_lp, K = 8)` only rows 7–8 separate `pow` from the product,
/// by ONE bit — and at `(tau_lp, K = 4)` **nothing separates them at all**. `K = 16` separates on
/// 8 rows against the product and 14 against the single power, so here the rule is *pinned*
/// rather than incidentally satisfied. Slice J's lesson, third instance: *exactness bounds the
/// CELLS visited, not the RULES discriminated* — and the detector has to be measured, not assumed
/// from the fact that an arm exists.
///
/// **THE DETECTOR WAS THEN MEASURED RATHER THAN TRUSTED**, which is this project's standing rule
/// and the one `rung32.rs`'s square gate was written under. `ladder_t` was deliberately
/// re-spelled as a running product and the dump re-run: it fails at `cellB/theta_d/7`,
/// `0x3ff595ff5c0b5e4d` against `…4e` — one bit, on exactly the row the scan predicted would be
/// the first to separate.
fn cell_g() -> StageStack {
    let cmap = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, vsv: 0.10, capacity: 0.80,
                              ..ComponentMap::flat() };
    StageStack::new(StageStackSpec {
        kc: KC, split: Split::Tau, vsv_stages: Some(3),
        ..StageStackSpec::new(16, cmap, TAU_LP, PI_LP, ETA_LP) })
}

/// The DESIGN SETTING on the UNIFORM profile — where § 5.10 (iv)'s degenerate argmin lives.
/// Nothing is moved, so at `(m, n) = (1, 1)` every row sits at `phi_k = 1` and the per-row
/// margins collapse onto ONE value to the bit, except where the march's own `th *= tau_k`
/// accumulation has drifted from the ladder `theta_d` it divides by. `K = 8` and `K = 4` land on
/// OPPOSITE sides of that drift — see the tie-break gate.
fn cell_uniform(k: usize) -> StageStack {
    let cmap = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, capacity: 0.80,
                              ..ComponentMap::flat() };
    StageStack::new(StageStackSpec {
        kc: KC, cap_profile: CapProfile::Uniform,
        ..StageStackSpec::new(k, cmap, TAU_LP, PI_LP, ETA_LP) })
}

/// Python's `("design", 1.0, 1.0), ("part", 0.82, 0.90), ("deep", 0.55, 0.72)`.
const FACES: [(&str, f64, f64); 3] =
    [("design", 1.0, 1.0), ("part", 0.82, 0.90), ("deep", 0.55, 0.72)];

#[test]
fn stage_stack_readings_match_pypy_bit_for_bit() {
    let want = load();
    let mut seen = 0usize;
    let mut check = |key: String, got: f64| {
        let exp = *want.get(key.as_str())
            .unwrap_or_else(|| panic!("smoke key absent from the Python dump: {key}"));
        assert_eq!(got.to_bits(), exp.to_bits(),
                   "{key}: rust {got:e} ({:#x}) vs python {exp:e} ({:#x})",
                   got.to_bits(), exp.to_bits());
        seen += 1;
    };

    // 0. THE ANCHOR — the six literals above, checked against the design run that produced them.
    for (name, got) in [("tau_lpc_d", TAU_LP), ("pi_lpc_design", PI_LP), ("eta_lpc", ETA_LP),
                        ("tau_hpc_d", TAU_HP), ("pi_hpc_design", PI_HP), ("eta_hpc", ETA_HP),
                        ("kc", KC)] {
        check(format!("anchor/{name}"), got);
    }

    for (tag, st) in [("A", cell_a()), ("B", cell_b()), ("C", cell_c()), ("D", cell_d()),
                      ("E", cell_uniform(8)), ("F", cell_uniform(4)), ("G", cell_g())] {
        let k = st.k;
        let p = format!("cell{tag}");

        // 1. THE DESIGN LADDER — `_ladder_T` on both splits, `_stage_eta`'s inversion, and
        //    `_ladder_p` built on top of it.
        check(format!("{p}/e_d"), st.e_d);
        check(format!("{p}/vsv_stages"), st.vsv_stages as f64);
        check(format!("{p}/cmap_vsv"), st.cmap.vsv);
        check(format!("{p}/cmap_axial_vsv"), st.cmap_axial.vsv);
        for i in 0..=k {
            check(format!("{p}/theta_d/{i}"), st.theta_d[i]);
            check(format!("{p}/varpi_d/{i}"), st.varpi_d[i]);
        }

        // 2. THE PER-ROW READS — `psi_at`/`vsv_at` on BOTH branches (cells B and C), rung 56's
        //    lazy capacity cache on both profiles, and the three throat reads.
        for i in 0..k {
            check(format!("{p}/psi_at/{i}"), st.psi_at(i, 0.93));
            check(format!("{p}/vsv_at/{i}"), st.vsv_at(i));
            check(format!("{p}/throat_ratio/{i}"), st.stage_throat_ratio(i));
            check(format!("{p}/capacity/{i}"), st.stage_capacity(i));
            check(format!("{p}/throat_loading/{i}"), st.stage_throat_loading(i, 0.97));
            check(format!("{p}/capacity_margin/{i}"), st.stage_capacity_margin(i, 0.97));
        }

        // 3. THE MARCH, at three faces — every stage's own coordinates, not just the total.
        for (fname, m, n) in FACES {
            let q = format!("{p}/{fname}");
            let r = st.march(m, n, st.eta_d);
            check(format!("{q}/tau"), r.tau);
            check(format!("{q}/pi_internal"), r.pi_internal);
            check(format!("{q}/e"), r.e);
            check(format!("{q}/clamped"), r.clamped as f64);
            for i in 0..k {
                check(format!("{q}/phi/{i}"), r.phis[i]);
                check(format!("{q}/n_k/{i}"), r.n_ks[i]);
                check(format!("{q}/tau_k/{i}"), r.taus[i]);
            }
            check(format!("{q}/tau_of"), st.tau_of(m, n, st.eta_d));
            check(format!("{q}/lumped_tau"), st.lumped_tau(m, n));
            check(format!("{q}/m_k_last"), r.phis[k - 1] * r.n_ks[k - 1]);
            // Rung 56's per-row margin at the MARCHED flow, and the ARGMIN over it. The index is
            // checked beside the values because where they agree to the bit the index is the only
            // thing that discriminates — § 5.10 (iv)'s *a value oracle would be blind to it
            // because the values agree to the bit while the INDEX flips*.
            let margins: Vec<f64> =
                (0..k).map(|i| st.stage_capacity_margin(i, r.phis[i] * r.n_ks[i])).collect();
            for i in 0..k {
                check(format!("{q}/row_margin/{i}"), margins[i]);
            }
            let argmin = (0..k).min_by(|&a, &b| margins[a].total_cmp(&margins[b])).expect("rows");
            check(format!("{q}/row_argmin"), argmin as f64);
        }

        // 4. THE FLOORS. `bracket_lo` is the arm that clamps NOTHING — kept because the first
        //    grid consisted only of it and read as coverage. `clamp_T` fires `_T_FLOOR` alone;
        //    `clamp_TP` pushes `e` past the derived 1.001 threshold so `_P_FLOOR` fires too.
        let lo = st.march(0.9, 0.1, st.eta_d);
        check(format!("{p}/bracket_lo/tau"), lo.tau);
        check(format!("{p}/bracket_lo/clamped"), lo.clamped as f64);
        let t_only = st.march(8.0, 2.0, st.eta_d);
        check(format!("{p}/clamp_T/tau"), t_only.tau);
        check(format!("{p}/clamp_T/clamped"), t_only.clamped as f64);
        for i in 0..k {
            check(format!("{p}/clamp_T/tau_k/{i}"), t_only.taus[i]);
        }
        let eta_hi = 0.99 * st.eta_d / 0.90;
        let both = st.march(8.0, 2.0, eta_hi);
        check(format!("{p}/clamp_TP/eta_live"), eta_hi);
        check(format!("{p}/clamp_TP/e"), both.e);
        check(format!("{p}/clamp_TP/tau"), both.tau);
        check(format!("{p}/clamp_TP/pi_internal"), both.pi_internal);
        check(format!("{p}/clamp_TP/clamped"), both.clamped as f64);

        // 5. THE SPEED-LINE INVERSION — the `K > 1` bisection on A/B/C, and on D the DISPATCH to
        //    rung 32's own `ComponentMap::solve_n`, which is what makes § 5.10 P5's reduce
        //    bit-for-bit rather than merely tight.
        for (lbl, m, tau_c) in [("at_design", 1.0, st.tau_d),
                                ("throttled", 0.86, 1.0 + 0.80 * (st.tau_d - 1.0))] {
            check(format!("{p}/solve_n/{lbl}"), st.solve_n(m, tau_c, st.eta_d));
        }

        // 6. A MOVED live efficiency, so `e = e_d*(eta_live/eta_d)` is not the identity and the
        //    parenthesisation is exercised.
        check(format!("{p}/march_eta_lo/tau"), st.march(0.95, 0.96, st.eta_d * 0.97).tau);
        check(format!("{p}/march_eta_lo/e"), st.march(0.95, 0.96, st.eta_d * 0.97).e);
    }

    assert_eq!(seen, want.len(),
               "the Rust read {seen} of the dump's {} keys — a key the port never reads is a \
                method the smoke does not witness", want.len());
}

// =========================================================================================
// The claims the value dump is BLIND to
// =========================================================================================

/// § 5.10 (iii)'s two LIVE tolerances, and the two DEAD caps beside them.
///
/// **The 48 is predictable from the arithmetic and not merely measured**, which is why it is
/// gated here rather than left to the oracle: both breaks are ABSOLUTE (`hi - lo <= 1e-14`) over
/// a FIXED bracket, so the count is `ceil(log2(width/1e-14))` and cannot depend on the data —
/// `ceil(log2(1.95/1e-14)) = 48` for `stage_eta`'s `[0.05, 2.0]` and `ceil(log2(1.9/1e-14)) = 48`
/// for `solve_n`'s `[0.1, 2.0]`. `map.rs`'s `solve_n` carries the same 48 for the same reason.
///
/// A count that differs means the arithmetic diverged somewhere a value gate still passes.
#[test]
fn the_two_tolerances_are_live_and_the_two_caps_are_dead() {
    let _ = take_census();
    let st = cell_a();
    let c = take_census();
    assert_eq!(c.stacks_built, 1);
    assert_eq!(c.eta_passes, 48,
               "stage_eta's break is _E_TOL and not the 300 cap: {} passes", c.eta_passes);
    assert!(c.eta_passes < StageStack::E_MAX as u64, "the 300 cap is DEAD");

    let _ = st.solve_n(0.86, 1.0 + 0.80 * (st.tau_d - 1.0), st.eta_d);
    let c = take_census();
    assert_eq!(c.solve_n_calls, 1);
    assert_eq!(c.solve_n_passes, 48,
               "solve_n's break is _N_TOL and not the 200 cap: {} passes", c.solve_n_passes);
    assert!(c.solve_n_passes < StageStack::N_MAX as u64, "the 200 cap is DEAD");
    // 2 bracket endpoints + 48 bisection residuals + the ONE extra march the clamped-root check
    // costs. § 5.10 (vi)'s 6 464 is built out of exactly this.
    assert_eq!(c.marches, 51, "the clamped-root check is a 51st march, not a free read");
}

/// § 5.10 P8, DERIVED rather than counted — see the module note.
///
/// The measured claim is *`_P_FLOOR` fires 0 times in 521 649 marches*, which is a statement
/// about a grid. The threshold `e > (1 - P_FLOOR)/(1 - T_FLOOR) = 1.001` is a statement about the
/// two constants, and it holds on any map, split, `K` and design point — so this gates the
/// mechanism and not the sweep.
///
/// It also shows the conflation § 5.10 (iii) warns about: past the threshold both floors fire on
/// the SAME stages, and Python's shared counter reads exactly double. A gate on `clamped` alone
/// would see 14 and be unable to say which guard produced it.
#[test]
fn the_p_floor_is_dead_by_a_derived_threshold_not_by_the_grid() {
    let threshold = (1.0 - StageStack::P_FLOOR) / (1.0 - StageStack::T_FLOOR);
    assert_eq!(threshold, 1.001, "the threshold is exactly 1.001 in the two constants");

    let st = cell_a();
    // Below the threshold: `_T_FLOOR` alone, on 7 of 8 stages.
    let _ = take_census();
    let below = st.march(8.0, 2.0, st.eta_d);
    let c = take_census();
    assert!(st.e_d * (st.eta_d / st.eta_d) < threshold, "the design run sits BELOW the threshold");
    assert_eq!((c.t_floor_fires, c.p_floor_fires), (7, 0));
    assert_eq!(below.clamped, 7);

    // Above it: both, on the same stages, and the shared counter doubles.
    let eta_hi = 0.99;
    let above = st.march(8.0, 2.0, eta_hi);
    let c = take_census();
    assert!(above.e > threshold, "e = {} must clear {threshold}", above.e);
    assert_eq!((c.t_floor_fires, c.p_floor_fires), (7, 7),
               "past the threshold both floors fire on the SAME stages");
    assert_eq!(above.clamped, 14, "Python's shared counter reads the SUM and nothing else");

    // And the reachable side is not an artefact of one stack: `march(0.9, 0.1)` — the LOW end of
    // `solve_n`'s bracket, which the first grid assumed was the non-physical one — clamps
    // nothing at all.
    let _ = take_census();
    let _ = st.march(0.9, 0.1, st.eta_d);
    let c = take_census();
    assert_eq!((c.t_floor_fires, c.p_floor_fires), (0, 0),
               "it is the HIGH m/n end that is non-physical, not the low one");
}

/// § 5.10's lazy-cache decision, gated where `probe_n4.py` measured it must be.
///
/// 80 of the slice's own 160 schedule rows are built on maps with `capacity == 0.0`, so an eager
/// build in the constructor would panic where Python is silent. The construction must succeed and
/// the FIRST READ must raise.
///
/// **And the call is DIRECT, not through a matcher.** `probe_n4.py` found a second assert one
/// level up carrying the same sentence, which always fires first — so a gate driven through rung
/// 55's matcher would check the outer guard while reading as though it checked this one.
#[test]
fn the_capacity_cache_is_lazy_and_the_construction_is_silent() {
    let cmap = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() };
    assert_eq!(cmap.capacity, 0.0);
    let st = StageStack::new(StageStackSpec {
        kc: KC, ..StageStackSpec::new(8, cmap, TAU_LP, PI_LP, ETA_LP) });
    // Everything that does NOT need rung 54's throat model still works, exactly as in Python.
    let _ = st.march(1.0, 1.0, st.eta_d);
    let _ = st.tau_of(0.9, 0.95, st.eta_d);
    let _ = st.solve_n(0.9, 1.0 + 0.9 * (st.tau_d - 1.0), st.eta_d);
    let _ = st.lumped_tau(0.9, 0.95);
    // Rung 56's AREA half needs no `C` either — the split is per method, not per rung.
    let _ = st.stage_throat_ratio(3);
    let _ = st.stage_throat_loading(3, 0.97);
}

#[test]
#[should_panic(expected = "rung-56 per-row capacity needs rung 54's throat model")]
fn the_capacity_read_raises_from_the_stack_and_not_from_the_map() {
    let cmap = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() };
    let st = StageStack::new(StageStackSpec {
        kc: KC, ..StageStackSpec::new(8, cmap, TAU_LP, PI_LP, ETA_LP) });
    let _ = st.capacities();
}

/// The cache is a cache — built once, then hit.
///
/// § 5.10 (vi) measured 120 builds against 4 360 hits, which is the number that makes rebuilding
/// the stack on demand (the rejected `Copy` [`turbojet::stator::Descendant`]) vacuous.
#[test]
fn the_capacity_cache_is_built_once_and_hit_thereafter() {
    let st = cell_a();
    let _ = take_census();
    let first = st.capacities().to_vec();
    let c = take_census();
    assert_eq!((c.capacities_built, c.capacities_hits), (1, 0));

    for k in 0..st.k {
        let _ = st.stage_capacity(k);
    }
    let c = take_census();
    assert_eq!((c.capacities_built, c.capacities_hits), (0, st.k as u64));
    assert_eq!(st.capacities(), first.as_slice());
}

/// § 5.10 (iv)'s ARGMIN TIE-BREAK — the degeneracy is REAL, and it is not where I first put it.
///
/// **The first version of this gate asserted the tie on cell C and was refuted by its own
/// numbers** (`[0.140, 0.243, 0.331, 0.348]` — no tie at all). Cell C carries a MOVED stator, so
/// at `(m, n) = (1, 1)` the march is not at the stack's design point: `psi(1) != 1` once `vsv`
/// is nonzero, and the rows separate by whole percent. The degeneracy needs the DESIGN setting.
///
/// Measured there ([`cell_uniform`], and the same numbers are in the dump so they are compared
/// against PyPy rather than restated here):
///
/// | | rows 0..K-2 | last row | argmin |
/// |---|---|---|---|
/// | `K = 8` | **bit-identical, all 7** | LOWER by one step in the last place | **7** |
/// | `K = 4` | **bit-identical, all 3** | HIGHER by one step | **0** |
///
/// So the margins agree to ~1e-16 and the argmin lands on OPPOSITE ends of the stack depending on
/// which way the march's own `th *= tau_k` accumulation drifted from the ladder `theta_d` it
/// divides by. That is § 5.10 (iv) exactly: *not physics but a tie-break*, and *a value oracle
/// would be blind to it because the values agree to the bit while the INDEX flips*.
///
/// **The tie-break rule itself is pinned on a CONSTRUCTED tie**, not on these cells. On measured
/// data the rule would be *incidentally* satisfied — cell F's argmin is 0 because row 0 is
/// genuinely tied-and-first, and cell E's is 7 because row 7 is genuinely smaller; neither run
/// discriminates *first-of-equals* from *last-of-equals*. That is the self-comparison failure
/// mode (§ 5.10 P6), so the rule gets its own vector.
#[test]
fn the_design_row_margins_tie_and_the_argmin_is_decided_by_the_last_bit() {
    for (k, tied, expect_argmin) in [(8usize, 7usize, 7usize), (4, 3, 0)] {
        let st = cell_uniform(k);
        let r = st.march(1.0, 1.0, st.eta_d);
        let margins: Vec<f64> =
            (0..k).map(|i| st.stage_capacity_margin(i, r.phis[i] * r.n_ks[i])).collect();
        for i in 1..tied {
            assert_eq!(margins[i].to_bits(), margins[0].to_bits(),
                       "K={k} row {i} must tie row 0 exactly: {margins:?}");
        }
        assert_ne!(margins[k - 1].to_bits(), margins[0].to_bits(),
                   "K={k}: the last row is the one the accumulation moved");
        let spread = margins.iter().cloned().fold(f64::MIN, f64::max)
                   - margins.iter().cloned().fold(f64::MAX, f64::min);
        assert!(spread < 1e-15, "K={k}: the whole spread is {spread:e}, i.e. a last-bit effect");
        let argmin = (0..k).min_by(|&a, &b| margins[a].total_cmp(&margins[b])).expect("rows");
        assert_eq!(argmin, expect_argmin, "K={k} margins {margins:?}");
    }
}

/// § 5.10 P1's FALLIBLE TWIN — the arm that makes it a twin at all.
///
/// `try_solve_n` ships with **two** `Err` returns, and P1's entire content is that rungs 55/56
/// need exactly those two and nothing else. Shipping them with neither one executed would be
/// *slice L step 3* again — a headline naming machinery no test reaches.
///
/// The bracket arm is reachable from numbers already measured for the floors: at `m = 8` the
/// stack cannot do the design work at EITHER bracket end, so `flo` and `fhi` are both negative
/// and the sign change the bisection needs does not exist.
///
/// The **clamped-root** arm is deliberately NOT reached here — § 5.10 (i) measured it at 1 firing
/// in 40, and manufacturing it needs a root that exists whose march still clamps. It was booked in
/// [`slice_n_deferrals_so_far`] and step 4's oracle DISCHARGED it, from the firing's own
/// arguments: `slice_n_oracle.rs::the_clamped_root_arm_is_reached_from_the_dump_grid`. Written
/// down rather than left silent, which is *a documented gate that doesn't exist* read the other
/// way round.
#[test]
fn the_speed_line_bracket_arm_returns_err_and_the_panicking_half_agrees() {
    let st = cell_a();
    let err = st.try_solve_n(8.0, st.tau_d, st.eta_d)
        .expect_err("a work the stack cannot reach at either bracket end must Err, not solve");
    assert!(err.0.contains("rung-55 stack speed-line bracket fails"),
            "the abort names ITS OWN frame, not `ComponentMap`'s: {}", err.0);

    // The two halves are one string produced in one place — a twin whose halves said different
    // things would make the caught edge and the uncaught crash look like different failures.
    let panicked = std::panic::catch_unwind(|| {
        let st = cell_a();
        st.solve_n(8.0, st.tau_d, st.eta_d)
    });
    let msg = panicked.expect_err("the panicking half must panic here");
    let msg = msg.downcast_ref::<String>().expect("panic payload is the Abort's own string");
    assert_eq!(msg, &err.0, "the fallible and panicking halves must say the SAME thing");

    // And the arm is not vacuous: the same solve at a reachable work returns Ok.
    assert!(st.try_solve_n(1.0, st.tau_d, st.eta_d).is_ok());
}

/// What step 2 does NOT port, booked at its gate rather than silently dropped.
///
/// Step 5's `rung55.rs`/`rung56.rs` take this ledger over; it lives here so that between step 2
/// and step 5 the omissions are written down somewhere a reader will hit.
#[test]
fn slice_n_deferrals_so_far() {
    // 1. Python's `assert split in ("dT", "tau")` and `assert cap_profile in ("derived",
    //    "uniform")` — UNREPRESENTABLE, not owed. [`Split`] and [`CapProfile`] are enums, so
    //    there is no invalid value to reject and no runtime check to port. The type-level
    //    refusal is strictly stronger (rung 53's `lp_disabled` precedent, § 5.10 P10).
    //    Witnessed by the fact that this file constructs every variant of both and no third one
    //    can be written.
    let _ = (Split::DT, Split::Tau, CapProfile::Derived, CapProfile::Uniform);

    // 2. Python's `assert 0 <= vsv_stages` — half UNREPRESENTABLE. `usize` cannot be negative,
    //    so only the `<= K` half survives as a runtime assert. It is live and gated below.
    // 2b. `try_solve_n`'s CLAMPED-ROOT `Err` arm — the 1-in-40 of § 5.10 (i). **DISCHARGED BY
    //    STEP 4**, in `slice_n_oracle.rs::the_clamped_root_arm_is_reached_from_the_dump_grid`:
    //    the dump classifies each caught firing AT THE RAISE (the schedule's `except
    //    AssertionError: break` swallows which arm it was) and carries the firing's
    //    `(m, tau_c, eta_live)` plus its cell, so the gate rebuilds that stack — checked against
    //    its own `tau_d`/`e_d` bits — and re-enters the arm directly. Both arms of P1's twin are
    //    now executed. Left in the ledger with its outcome rather than struck, so a reader
    //    following the BRACKET arm's note above finds where the other half went.
    // 3. Python's `_M_of_nu` range guard is LATENT-ONLY (worst `nu^2` on § 5.10's grid is 2.7 %
    //    of the limit) and lives in `map.rs`, shipped at step 1. Its `#[should_panic]` is owed by
    //    step 5 on a hand-built profile, not reachable from any stack this file builds.
    // 4. Rung 55's `lp_disabled` refusal — no such parameter exists in the Rust, so
    //    `assert not (lp_disabled and K > 1)` has nothing to witness (§ 5.10 P10).

    // --- STEP 3's own additions -------------------------------------------------------------
    // 5. `assert spool in self._SPOOLS` — THREE instances (`_stack_of`, `throat_walk`,
    //    `stage_incidence_schedule`), all UNREPRESENTABLE: `Spool` is a two-variant enum, so
    //    there is no third value to reject. Same shape as item 1, and counted separately because
    //    a name → parameter-set diff at step 5 will otherwise read three gates as missing.
    let _ = (Spool::Lp, Spool::Hp);
    // 6. Rung 55's `test_cycle_untouched_transient_ladder_is_bit_for_bit_unstacked` — OWED TO
    //    PHASE 6, not to this slice. It runs a rung-43 fuel transient twice on the same hardware
    //    (once before a stack is live, once after) and demands the two point lists compare `==`;
    //    `TwoSpoolFuelTransient` does not exist in Rust yet. Booked under THAT reason and not
    //    under `phi_max`, which is where the first draft filed it — *a deferral filed against the
    //    wrong cause is a deferral nobody can discharge.*
}

#[test]
#[should_panic(expected = "rung-55 vsv_stages must be in [0, K=4]")]
fn the_surviving_half_of_the_vsv_stages_guard_is_live() {
    let cmap = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() };
    let _ = StageStack::new(StageStackSpec {
        kc: KC, vsv_stages: Some(5), ..StageStackSpec::new(4, cmap, TAU_LP, PI_LP, ETA_LP) });
}

/// § 5.10 P6's CONSTRUCTED tie — the rule, not an instance of it.
///
/// Python's `min(range(n), key=…)` returns the FIRST minimum. Rust's `Iterator::min_by` also
/// returns the first of equal elements, so the idiomatic spelling agrees — but a `fold` with
/// `<=`, or a `max_by` anywhere near it, would not, and no value gate can see the difference.
#[test]
fn the_argmin_returns_the_first_of_several_bit_identical_minima() {
    let v: [f64; 5] = [0.5, 0.25, 0.25, 0.25, 0.75];
    let argmin = (0..v.len()).min_by(|&a, &b| v[a].total_cmp(&v[b])).expect("rows");
    assert_eq!(argmin, 1, "first of the three exact minima, not the last");
    // The refuted spelling, kept so the gate is two-sided rather than a restatement of itself.
    let mut last = 0usize;
    for i in 1..v.len() {
        if v[i] <= v[last] { last = i; }
    }
    assert_eq!(last, 3, "a `<=` fold picks the LAST — which is the defect this gate exists for");
    assert_ne!(argmin, last);
}

// =========================================================================================
// STEP 3 — REACHABILITY, and the reduce identities that are STRUCTURAL rather than measured
// =========================================================================================
//
// Step 3 ships six public reading methods and two hook bodies, and until step 4's oracle exists
// NONE of them is executed by anything. That is slice L step 3's shape exactly — *my smoke check
// witnessed 1 of the 3 methods the slice's own headline names* — so this gate enumerates the six
// and calls every one.
//
// It deliberately asserts only what is STRUCTURAL: row counts, and the three `K = 1` identities
// that hold by which BRANCH runs rather than by arithmetic agreeing. § 5.10's P4 (rung 56's face
// read field-for-field against rung 54's) and P5 (`solve_n`'s dispatch) are step 5's, and stating
// them here as approximate versions of themselves would be the *ported test goes VACUOUS* trap in
// reverse — a weaker gate standing where the strong one is owed.

fn eng_design() -> TwoSpoolEngine {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    let gas = Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    });
    build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, 50_000.0, TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    })
}

fn eng_flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// Armed with BOTH walls: rung 36's floor (the incidence anchor) and rung 54's capacity.
fn armed_maps() -> (ComponentMap, ComponentMap) {
    (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
         .with_phi_surge(0.55).with_capacity(0.80),
     ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
         .with_phi_surge(0.55).with_capacity(0.80))
}

fn matcher(k_lp: usize, k_hp: usize, vsv_stages_lp: Option<usize>) -> StageStackCore {
    let (ml, mh) = armed_maps();
    StageStackCore::new(StageStackCoreSpec {
        k_lp, k_hp, vsv_stages_lp,
        ..StageStackCoreSpec::new(eng_design(), eng_flight(), 1.0, ml, mh)
    })
}

/// The six methods, each reached at least once, on a STACKED matcher and on its `K = 1` control.
#[test]
fn every_step3_reader_is_reached_and_the_k1_branches_are_exact() {
    let f = eng_flight();
    let stacked = matcher(8, 8, Some(1));
    let lumped = matcher(1, 1, None);

    // --- (1) stage_margin: K rows per spool, and ONE row where no object was built.
    let sm = stacked.stage_margin(&f, 1200.0);
    assert_eq!(sm.lp.stages.len(), 8);
    assert_eq!(sm.hp.stages.len(), 8);
    assert!(sm.lp.worst < 8 && sm.hp.worst < 8);
    let sm1 = lumped.stage_margin(&f, 1200.0);
    assert_eq!(sm1.lp.stages.len(), 1);
    // One row means front IS rear, so `rear_excess` is a STRUCTURAL zero — the sign of a lumped
    // read, and the quantity rung 55 exists to make non-zero.
    assert_eq!(sm1.lp.rear_excess.to_bits(), 0.0f64.to_bits());
    assert!(sm.lp.rear_excess != 0.0, "a resolved stack must separate front from rear");

    // --- (2) stage_throat_margin: at K = 1 the FACE read IS the row read, by branch.
    let tm1 = lumped.stage_throat_margin(&f, 1200.0);
    for r in [&tm1.lp, &tm1.hp] {
        assert_eq!(r.stages.len(), 1);
        assert_eq!(r.m_c_worst.to_bits(), r.m_c_face.to_bits());
        assert_eq!(r.x_worst.to_bits(), r.x_face.to_bits());
        assert_eq!(r.amplification.to_bits(), 1.0f64.to_bits(),
                   "at K = 1 the binding row IS the face, exactly");
        assert_eq!(r.binds, 0);
        assert!(r.front_binds && r.rear_binds);
    }
    let tm = stacked.stage_throat_margin(&f, 1200.0);
    assert_eq!(tm.lp.stages.len(), 8);
    assert!(tm.lp.binds < 8 && tm.lp.inc_worst < 8);

    // --- (3) throat_walk: one row per throttle, on one spool.
    let grid = [1500.0, 1200.0, 1000.0];
    let walk = stacked.throat_walk(&f, &grid, Spool::Lp);
    assert_eq!(walk.len(), 3);
    for (row, &tt4) in walk.iter().zip(&grid) {
        assert_eq!(row.tt4.to_bits(), tt4.to_bits());
        assert_eq!(row.capacities.len(), 8);
        assert_eq!(row.margins.len(), 8);
    }

    // --- (4) work_gap: EXACTLY zero at K = 1 (the `None` branch assigns the lumped value to the
    //         marched one), and non-zero once resolved. That is the non-tautology, in-repo.
    let wg1 = lumped.work_gap(&f, 1200.0);
    assert_eq!(wg1.lp.gap.to_bits(), 0.0f64.to_bits());
    assert_eq!(wg1.hp.gap.to_bits(), 0.0f64.to_bits());
    assert_eq!(wg1.lp.tau_marched.to_bits(), wg1.lp.tau_lumped.to_bits());
    let wg = stacked.work_gap(&f, 1200.0);
    assert!(wg.lp.gap != 0.0 && wg.hp.gap != 0.0,
            "a marched stack must do different work from the lumped law it replaces");

    // --- (5) running_line_shift: against `at_stages(1, 1)`. On a matcher that IS K = 1 the
    //         baseline is the same machine, so every delta is a STRUCTURAL exact zero — which is
    //         also the only cheap witness that `at_stages` rebuilds the same hardware.
    let s1 = lumped.running_line_shift(&f, &grid);
    assert_eq!(s1.len(), 3);
    for row in &s1 {
        assert_eq!(row.lp.d_n.to_bits(), 0.0f64.to_bits());
        assert_eq!(row.hp.d_phi.to_bits(), 0.0f64.to_bits());
        assert_eq!(row.d_thrust.to_bits(), 0.0f64.to_bits());
    }
    let sh = stacked.running_line_shift(&f, &grid);
    assert!(sh.iter().any(|r| r.lp.d_n != 0.0),
            "the stacked efficiency loops must move the running line");

    // --- (6) stage_incidence_schedule: the FRONT-ROW lever, which § 5.10 (ii) measured reaching
    //         on all 80 of its rows. One off-design throttle is enough to reach the scan, the
    //         bracket and the bisection.
    let rows = stacked.stage_incidence_schedule(&f, &[1000.0], Spool::Lp, 0, 4.0);
    assert_eq!(rows.len(), 1);
    let r = &rows[0];
    assert!(r.reached, "the front-row schedule must EXIST at Tt4 = 1000");
    assert!(r.vsv_star > 0.0, "closing the stator is what buys the design incidence back");
    assert!(r.residual.abs() <= StageStackCore::INC_TOL,
            "a reached root sits on the residual tolerance, not merely near it");
    assert_eq!(r.k, 8);
    assert_eq!(r.vsv_stages, Some(1));
    assert_eq!(r.spool, Spool::Lp);
}

/// **THE ONE-SIDED STACK IS A CONTROLLED EXPERIMENT — the reachability half of § 5.10 P9.**
///
/// Where a stack is built on ONE spool only, the other spool's efficiency loop is literally rung
/// 39's, reached through the `None` arm of rung 55's hook body. This gate witnesses that arm
/// exists and is taken; P9's measured claim — that stacking the LP spool leaves all four HP
/// fields BIT-IDENTICAL while the reverse does not — is step 5's, on its 40-point grid.
#[test]
fn a_one_sided_stack_leaves_the_other_spools_loop_inherited() {
    let f = eng_flight();
    let one_sided = matcher(8, 1, None);
    assert!(one_sided.stack_of(Spool::Lp).is_some());
    assert!(one_sided.stack_of(Spool::Hp).is_none(),
            "K_hp = 1 builds no object, so its loop is rung 39's own");
    let sm = one_sided.stage_margin(&f, 1200.0);
    assert_eq!(sm.lp.stages.len(), 8);
    assert_eq!(sm.hp.stages.len(), 1);
    assert_eq!(sm.hp.rear_excess.to_bits(), 0.0f64.to_bits());
}

/// **THE PER-ROW SURGE FLOOR, WHICH IS THE ONE SPELLING IN STEP 3 THAT COULD DIVERGE SILENTLY.**
///
/// Python computes each row's floor as `cmap.phi_surge / (1 + v_k*cmap.phi_surge)` — off the map's
/// `phi_surge` **FIELD**, which by rung 53's rule means the DESIGN-setting anchor — and NOT via
/// `ComponentMap::phi_surge_at`, which reads the MAP's own `vsv`. On a lumped lever the two agree
/// for every row and the choice is invisible. On the FRONT-ROW lever at a moved setting they do
/// not: the rear rows carry `v_k = 0` while `cmap.vsv != 0`, so the map-level reader would put the
/// rear rows' floor at the front rows' value.
///
/// The reachability gate above reads `m_i` / `worst` / `rear_excess` and would never see it —
/// step 2's own lesson (a smoke grid on the default lever leaves `cmap_axial`'s branch dead) one
/// method further along.
#[test]
fn the_per_row_surge_floor_is_read_off_the_design_anchor_not_the_moved_map() {
    let f = eng_flight();
    let moved = matcher(8, 8, Some(1)).at_setting(0.20, 0.0);
    let (ml, _) = armed_maps();
    let lp = moved.stage_margin(&f, 1200.0);
    let rows = &lp.lp.stages;
    assert_eq!(rows.len(), 8);
    assert_eq!(rows[0].vsv.to_bits(), 0.20f64.to_bits(), "row 0 carries the stator");
    assert_eq!(rows[7].vsv.to_bits(), 0.0f64.to_bits(), "row 7 does not");

    // The rear rows sit at the DESIGN anchor, to the bit — `v_k = 0` makes the formula the
    // identity, which is what "the field means the design-setting floor" cashes out to.
    assert_eq!(rows[7].phi_surge.to_bits(), ml.phi_surge.to_bits());
    // ...and the front row does NOT, which is what makes the previous line a claim.
    assert!(rows[0].phi_surge != rows[7].phi_surge,
            "a moved front row must sit on a moved floor");
    // The discriminator against the wrong spelling: the MAP-level reader returns the front row's
    // number for every row, so a port written on `phi_surge_at()` would fail the line above.
    let moved_map = moved.core.core.map_lp;
    assert_eq!(rows[0].phi_surge.to_bits(), moved_map.phi_surge_at().to_bits());
    assert!(moved_map.phi_surge_at() != ml.phi_surge,
            "...and the two readers genuinely disagree here, or this gate is vacuous");

    // `m_phi` is the same claim in the currency rungs 36–52 spend, so it splits the same way.
    assert_eq!(rows[7].m_phi.to_bits(), (rows[7].phi - ml.phi_surge).to_bits());
}
