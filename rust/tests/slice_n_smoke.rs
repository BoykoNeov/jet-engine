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

use turbojet::map::ComponentMap;
use turbojet::stage::{take_census, CapProfile, Split, StageStack, StageStackSpec};

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
                      ("E", cell_uniform(8)), ("F", cell_uniform(4))] {
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
    // 3. Python's `_M_of_nu` range guard is LATENT-ONLY (worst `nu^2` on § 5.10's grid is 2.7 %
    //    of the limit) and lives in `map.rs`, shipped at step 1. Its `#[should_panic]` is owed by
    //    step 5 on a hand-built profile, not reachable from any stack this file builds.
    // 4. Rung 55's `lp_disabled` refusal — no such parameter exists in the Rust, so
    //    `assert not (lp_disabled and K > 1)` has nothing to witness (§ 5.10 P10).
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
