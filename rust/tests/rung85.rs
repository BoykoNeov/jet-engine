//! RUNG 85 — THE BLADE-SPEED WALLS. `docs/rung85-spec.md`; anchor
//! `docs/plans/rung85-anchor-blade-speed.md` (§ refs are to it).
//!
//! Gate groups:
//!
//!   1. THE TABLE — the sizing reproduces § 6.2's 12 default-shape cells and § 6.3's per-shape
//!      `K`/`R`, against the PRINTED digits typed in here. Those were computed by a throwaway
//!      script outside the repo before this code existed (§ 4.4 A7), so this is the one gate in
//!      the file whose expected values this code did not produce.
//!   2. Q0 — at `λ = 1` the redline is § 6.2's `R at K*` column (no credit: an identity; the typed
//!      digits are what make it a check).
//!   3. REDUCE — `λ = 0` hands the plants the SHIPPED map objects, bit for bit (§ 4.5 A10).
//!   4. P8's identity — the PLANT's design point read through the tip-Mach reading equals the
//!      sizing's `M_rel,d` (V3).
//!   5. NO PANIC BEHIND A KNOB — out-of-range knobs and a choking design are `Err`, not a crash;
//!      the strength-bound branch.
//!   6. THE EXPERIMENT — § 4.1 / § 4.5's grid, measured and PINNED, each gate naming its
//!      HIT/MISS; plus the finding (the slope cancels on the lumped plant) and rung 53's
//!      published schedule reproduced (the independent check on the lumped route).

use turbojet::blade_speed::{build, reshape, size, BladeKnobs, Binding, Lever, Machine, SizingError,
                            SpoolDuty, Verdict};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stage::{StageStackCore, StageStackCoreSpec};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const FLOOR: f64 = 0.55;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// Rung 55's rig, verbatim: CPG, `π_LPC` 3, `π_HPC` 6, `Tt4` 1500 — the design § 6.2 sized.
fn design() -> TwoSpoolEngine {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    let gas = Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    });
    let losses = TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    };
    build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, 50_000.0, losses)
}

/// Rung 55's five disclosed shapes (`tests/rung55.rs` `maps`), verbatim.
fn maps(name: &str) -> (ComponentMap, ComponentMap) {
    let f = ComponentMap::flat();
    let (l, h) = match name {
        "flow/press" => (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                         ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
        "press/flow" => (ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..f },
                         ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }),
        "tilted"     => (ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f },
                         ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f }),
        "steep"      => (ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f },
                         ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f }),
        "flat-eta"   => (ComponentMap { sigma: 0.1, l: 0.7, ..f },
                         ComponentMap { sigma: 0.1, l: 1.0, ..f }),
        other => panic!("unknown shape {other}"),
    };
    (l.with_phi_surge(FLOOR), h.with_phi_surge(FLOOR))
}

/// § 6.2's six (`h`, `M_rel,lim`) cells, in its row order.
const CELLS: [(f64, f64); 6] = [(0.5, 1.3), (0.5, 1.4), (0.5, 1.5), (0.7, 1.3), (0.7, 1.4), (0.7, 1.5)];

fn knobs(h: f64, m: f64, lambda: f64) -> BladeKnobs {
    BladeKnobs { h, m_rel_lim: m, lambda, ..BladeKnobs::default() }
}

fn machine(shape: &str, h: f64, m: f64, lambda: f64) -> Machine {
    let (ml, mh) = maps(shape);
    let k = knobs(h, m, lambda);
    build(&design(), flight(), ml, mh, k, k).unwrap_or_else(|e| panic!("{shape} {h} {m} {lambda}: {e}"))
}

fn near(got: f64, want: f64, half_ulp: f64, what: &str) {
    assert!((got - want).abs() <= half_ulp * 1.0001,
            "{what}: {got} is not the printed {want} (± {half_ulp})");
}

// ==========================================================================================
// GATE 1 — THE TABLE, against the anchor's printed digits
// ==========================================================================================

/// § 6.2, typed: `(K*, K, U_tip wall, U_tip,d, C at K, M_rel,d, R at K, R at K*)` per cell, LP
/// then HP. Typed from the anchor, never recomputed.
const TABLE_6_2: [[(f64, usize, f64, f64, f64, f64, f64, f64); 6]; 2] = [
    [(2.23, 3, 399.5, 344.3, 0.649, 1.114, 1.708, 1.472),
     (1.94, 2, 428.6, 421.7, 0.761, 1.376, 1.395, 1.372),
     (1.70, 2, 457.5, 421.7, 0.761, 1.376, 1.395, 1.286),
     (1.82, 2, 389.7, 372.1, 0.761, 1.238, 1.917, 1.830),
     (1.59, 2, 417.8, 372.1, 0.761, 1.238, 1.917, 1.707),
     (1.39, 2, 445.6, 372.1, 0.761, 1.238, 1.917, 1.601)],
    [(4.86, 5, 474.3, 467.7, 0.723, 1.281, 1.258, 1.240),
     (4.22, 5, 508.9, 467.7, 0.723, 1.281, 1.258, 1.156),
     (3.71, 4, 543.2, 522.9, 0.785, 1.441, 1.125, 1.083),
     (3.98, 4, 462.7, 461.4, 0.785, 1.296, 1.546, 1.542),
     (3.46, 4, 496.1, 461.4, 0.785, 1.296, 1.546, 1.438),
     (3.04, 4, 529.1, 461.4, 0.785, 1.296, 1.546, 1.348)],
];

#[test]
fn the_sizing_reproduces_the_anchor_table() {
    for (c, &(h, m)) in CELLS.iter().enumerate() {
        let at0 = machine("flow/press", h, m, 0.0);
        let at1 = machine("flow/press", h, m, 1.0);
        for (s, (z0, z1)) in [(&at0.lp, &at1.lp), (&at0.hp, &at1.hp)].into_iter().enumerate() {
            let (ks, k, uw, ud, cap, mrel, r_k, r_ks) = TABLE_6_2[s][c];
            let w = format!("{} h={h} M={m}", ["LP", "HP"][s]);
            assert_eq!(z0.binding, Binding::Airflow, "{w}: airflow binds every cell (§ 0 P-B)");
            near(z0.k_star, ks, 0.005, &format!("{w} K*"));
            assert_eq!(z0.k, k, "{w} K");
            near(2.0 / (1.0 + h) * z0.u_wall, uw, 0.05, &format!("{w} U_tip wall"));
            near(z0.u_tip, ud, 0.05, &format!("{w} U_tip,d"));
            near(z0.capacity, cap, 0.0005, &format!("{w} C"));
            near(z0.m_rel_d, mrel, 0.0005, &format!("{w} M_rel,d"));
            near(z0.redline, r_k, 0.0005, &format!("{w} R at K"));
            // GATE 2 — Q0: λ = 1 puts the blades at the wall, so the redline IS the K* column.
            near(z1.redline, r_ks, 0.0005, &format!("{w} R at λ=1 (Q0, the K* column)"));
            assert_eq!(z1.k, z0.k, "{w}: K does not depend on λ");
        }
    }
}

/// § 6.3, typed: `K/R` per cell for the three shapes whose slopes differ from the default.
#[test]
fn the_per_shape_table_reproduces_section_6_3() {
    let rows: [(&str, [(usize, f64); 6], [(usize, f64); 6]); 3] = [
        ("press/flow",
         [(3, 1.575), (3, 1.575), (2, 1.286), (3, 2.164), (2, 1.767), (2, 1.767)],
         [(5, 1.364), (4, 1.220), (4, 1.220), (4, 1.677), (3, 1.452), (3, 1.452)]),
        ("tilted",
         [(3, 1.637), (3, 1.637), (2, 1.337), (2, 1.837), (2, 1.837), (2, 1.837)],
         [(5, 1.308), (4, 1.169), (4, 1.169), (4, 1.607), (4, 1.607), (3, 1.392)]),
        ("steep",
         [(3, 1.502), (3, 1.502), (3, 1.502), (3, 2.064), (3, 2.064), (2, 1.685)],
         [(6, 1.313), (5, 1.199), (5, 1.199), (5, 1.648), (4, 1.474), (4, 1.474)]),
    ];
    for (shape, lp, hp) in rows {
        for (c, &(h, m)) in CELLS.iter().enumerate() {
            let z = machine(shape, h, m, 0.0);
            for (s, (got, want)) in [(&z.lp, lp[c]), (&z.hp, hp[c])].into_iter().enumerate() {
                let w = format!("{shape} {} h={h} M={m}", ["LP", "HP"][s]);
                assert_eq!(got.k, want.0, "{w} K");
                near(got.redline, want.1, 0.0005, &format!("{w} R"));
            }
        }
    }
}

// ==========================================================================================
// GATE 3 — REDUCE
// ==========================================================================================

fn map_bits(m: &ComponentMap) -> [u64; 8] {
    [m.a, m.b, m.c, m.sigma, m.a_t, m.l, m.phi_surge, m.vsv].map(f64::to_bits)
}

/// `λ = 0` ⇒ `r == 1.0` exactly ⇒ the plants get the SHIPPED map objects, in every cell and
/// shape — so every lever read at `λ = 0` is rung 53/55's own plant (A10).
#[test]
fn lambda_zero_hands_the_plants_the_shipped_maps() {
    for shape in ["flow/press", "press/flow", "tilted", "steep", "flat-eta"] {
        let (ml, mh) = maps(shape);
        for &(h, m) in &CELLS {
            let z = machine(shape, h, m, 0.0);
            assert_eq!(z.lp.r, 1.0);
            assert_eq!(z.hp.r, 1.0);
            assert_eq!(z.lp.v_d, 0.0);
            assert_eq!(map_bits(&z.map_lp), map_bits(&ml), "{shape}: LP map moved at λ = 0");
            assert_eq!(map_bits(&z.map_hp), map_bits(&mh), "{shape}: HP map moved at λ = 0");
        }
    }
    // …and `reshape` itself is the identity at r = 1 for a slope where (1+l)/1 − 1 would not be.
    let odd = ComponentMap { l: 0.1, ..ComponentMap::flat() };
    assert_ne!(((1.0 + odd.l) / 1.0 - 1.0).to_bits(), odd.l.to_bits(), "pick an l the formula moves");
    assert_eq!(map_bits(&reshape(odd, 1.0)), map_bits(&odd));
}

/// The reshaped map IS the shipped one with a design pre-swirl (A9): at every `(φ, v')`,
/// `ψ'(φ, v') = ψ(φ, v_d + v')/r`.
#[test]
fn the_reshaped_map_is_the_preswirled_map_renormalised() {
    let (ml, _) = maps("flow/press");
    for r in [0.95, 0.85, 0.695, 0.4] {
        let v_d = (1.0 - r) / (1.0 + ml.l);
        for phi in [0.6, 0.8, 1.0, 1.2] {
            for v in [-0.1, 0.0, 0.1, 0.3] {
                let got = reshape(ml, r).with_vsv(v).psi(phi);
                let want = ml.with_vsv(v_d + v).psi(phi) / r;
                assert!((got - want).abs() <= 1e-12, "r={r} φ={phi} v={v}: {got} vs {want}");
            }
        }
    }
}

// ==========================================================================================
// GATE 4 — P8's IDENTITY (V3), on the PLANT
// ==========================================================================================

/// V3, as A5 registered it: the PLANT's own design point — matched at `Tt4` = 1500 with the lever
/// at design — read through the tip-Mach reading, equals the sizing's `M_rel,d` to `1e-12`. That
/// is what guards the matcher's `n`/`φ` being normalised the way the reading assumes; feeding the
/// reading `(1, 1)` by hand would compare `front_row_mach` with itself. Measured 2026-10-06: every
/// residual ≤ 1.4e-14, lumped and stacked, both spools, `λ` ∈ {0, 1}.
#[test]
fn the_plants_design_point_reads_the_sizing_tip_mach() {
    for &(h, m) in &CELLS {
        for lambda in [0.0, 1.0] {
            let z = machine("flow/press", h, m, lambda);
            for lever in [Lever::Lumped, Lever::AllRows] {
                let o = z.plant(lever, Spool::Lp).match_point(&flight(), 1500.0);
                let dl = z.lp.tip_rel_mach(o.n_lp, o.phi_lp, 0.0) - z.lp.m_rel_d;
                let dh = z.hp.tip_rel_mach(o.n_hp, o.phi_hp, 0.0) - z.hp.m_rel_d;
                assert!(dl.abs() <= 1e-12 && dh.abs() <= 1e-12, "h={h} M={m} λ={lambda} {lever:?}: {dl:e} {dh:e}");
            }
            for s in [&z.lp, &z.hp] {
                if lambda == 0.0 {
                    assert!(s.m_rel_d <= m + 1e-12, "λ = 0 sits at or below the level (rounding slows the blade)");
                } else {
                    assert!(s.m_rel_d < m, "the design pre-swirl puts the tip BELOW the level (A11)");
                }
            }
        }
    }
}

// ==========================================================================================
// GATE 5 — NO PANIC BEHIND A KNOB; the strength-bound branch
// ==========================================================================================

#[test]
fn a_bad_knob_is_an_error_not_a_crash() {
    let duty = SpoolDuty { tt: 286.125, dh: 117_700.0, l: 0.7, gamma: 1.4, cp: 1004.0 };
    for (k, name) in [
        (BladeKnobs { h: 1.0, ..BladeKnobs::default() }, "h"),
        (BladeKnobs { lambda: 1.5, ..BladeKnobs::default() }, "lambda"),
        (BladeKnobs { phi_d: 0.0, ..BladeKnobs::default() }, "phi_d"),
        (BladeKnobs { overspeed: 0.5, ..BladeKnobs::default() }, "overspeed"),
    ] {
        assert!(matches!(size(duty, k), Err(SizingError::Knob(n, _)) if n == name), "{name}");
    }
    // Φ_d = 1.2 at a high level: the front row's axial flow goes sonic — reported, not a panic.
    let hot = BladeKnobs { phi_d: 1.2, m_rel_lim: 1.6, lambda: 1.0, ..BladeKnobs::default() };
    assert!(matches!(size(duty, hot), Err(SizingError::Chokes { .. })));
}

/// D3's vacuity condition, reachable from the material knob: a weak enough material makes
/// STRENGTH set the design, and with the blades at the wall (`λ` = 1) the design sits exactly AT
/// its own redline — every overspeed crosses by construction.
#[test]
fn a_weak_material_makes_strength_set_the_design() {
    let duty = SpoolDuty { tt: 286.125, dh: 117_700.0, l: 0.7, gamma: 1.4, cp: 1004.0 };
    let weak = BladeKnobs { sigma_over_rho: 3.0e4, ..BladeKnobs::default() };
    let z0 = size(duty, weak).unwrap();
    let z1 = size(duty, BladeKnobs { lambda: 1.0, ..weak }).unwrap();
    assert_eq!(z0.binding, Binding::Strength);
    assert!((z1.redline - 1.0).abs() <= 1e-12, "λ = 1 on the strength wall: R = 1, got {}", z1.redline);
    assert!(z0.redline >= 1.0, "rounding K up can only slow the blade below the wall");
    assert_eq!(size(duty, BladeKnobs::default()).unwrap().binding, Binding::Airflow);
}

// ==========================================================================================
// GATE 6 — THE EXPERIMENT, measured and pinned (scored in docs/rung85-spec.md)
// ==========================================================================================

const THROTTLE: [f64; 4] = [1500.0, 1300.0, 1100.0, 1000.0];

/// Rung 53's and rung 55's PUBLISHED schedule numbers, through the one-row stack (`Lever::Lumped`)
/// at `λ` = 0. These came from outside this code — rung 55's spec, "Shape robustness" table, and
/// § 4.0's `N_L(v*)` = 1.26006 — so this is the independent check that the lumped route IS rung
/// 53's plant (scan-bracketed rather than rung 53's doubling ladder: same root, to `INC_TOL`).
#[test]
fn the_lumped_route_reproduces_rung_53s_published_schedule() {
    let z = machine("flow/press", 0.5, 1.4, 0.0);
    let r = z.schedule(Lever::Lumped, Spool::Lp, &[1000.0])[0];
    assert!(r.reached);
    near(r.vsv_star, 1.2436, 0.00005, "v* flow/press");
    near(r.n_lp, 1.26006, 0.000005, "N_L(v*) flow/press");
    for (shape, v) in [("press/flow", 1.0499), ("tilted", 1.4883), ("flat-eta", 0.9620)] {
        let z = machine(shape, 0.5, 1.4, 0.0);
        near(z.schedule(Lever::Lumped, Spool::Lp, &[1000.0])[0].vsv_star, v, 0.00005, &format!("v* {shape}"));
    }
}

/// THE SLOPE CANCELS (a finding, scoped to the LUMPED plant). Holding the design incidence pins
/// `1/φ − v = 1`, and then `ψ = 1 − σu² − l·u − v(1+l)φ` collapses to `1 + u − σu²`: the held
/// machine's work, hence its speed and setting, carry NO `l`. Only the BARE speed does — so a
/// bill quoted against bare is `l`-dependent through its denominator alone. On a stack only stage
/// 0 is held and the rows behind keep their `l`, so nothing here claims it there. Measured
/// 2026-10-06: `|ΔN|` ≤ 1.2e-11 (the incidence bisection's `INC_TOL` = 1e-12, propagated).
#[test]
fn holding_design_incidence_cancels_the_map_slope_on_the_lumped_plant() {
    let f = ComponentMap::flat();
    let (_, mh) = maps("flow/press");
    for tt4 in [1300.0, 1100.0, 1000.0] {
        let reads: Vec<_> = [0.3, 0.7, 1.0, 1.446, 2.0].iter().map(|&l| {
            let ml = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l, ..f }.with_phi_surge(FLOOR);
            let z = build(&design(), flight(), ml, mh, knobs(0.5, 1.4, 0.0), knobs(0.5, 1.4, 0.0)).unwrap();
            z.schedule(Lever::Lumped, Spool::Lp, &[tt4])[0]
        }).collect();
        for r in &reads {
            assert!(r.reached);
            assert!((r.n_lp - reads[1].n_lp).abs() <= 1e-10, "Tt4={tt4}: the held speed moved with l");
            assert!((r.vsv_star - reads[1].vsv_star).abs() <= 1e-9, "Tt4={tt4}: the held setting moved with l");
        }
        // Not vacuous: the BARE speed does move with l.
        assert!(reads[0].n_lp_bare - reads[4].n_lp_bare > 0.04, "Tt4={tt4}: bare must depend on l");
    }
}

/// P4 — HIT. The HP lumped schedule: max physical `N_H/N_H,d` over the grid ≤ 1.10, point ≈ 1.0.
/// P6 — HIT. The HP front-row schedule: max ≤ 1.00. (Both maxima sit at `Tt4` = 1500, `v*` = 0.)
#[test]
fn p4_p6_the_hp_lumped_and_front_row_levers_stay_at_or_below_design_speed() {
    let z = machine("flow/press", 0.5, 1.5, 0.0);
    let lumped = z.schedule(Lever::Lumped, Spool::Hp, &THROTTLE);
    let front = z.schedule(Lever::FrontRow, Spool::Hp, &THROTTLE);
    for (name, reads, bar) in [("P4 lumped", &lumped, 1.10), ("P6 front", &front, 1.00)] {
        let max = reads.iter().map(|r| r.n_hp).fold(f64::MIN, f64::max);
        assert!(reads.iter().all(|r| r.reached), "{name}: every point reached");
        assert!(max <= bar + 1e-12, "{name}: max N_H {max} over {bar}");
        assert!(reads.iter().all(|r| r.verdict(&z, Spool::Hp) == Verdict::Under), "{name}: no crossing");
    }
    near(lumped[3].n_hp, 0.8848, 0.00005, "P4 lumped N_H at 1000");
}

/// P5 — MISS (by 1.6 %). The HP ALL-ROWS schedule at the walls' `K_H` = 4 reaches `N_H/N_H,d` =
/// 1.1073 at `Tt4` = 1000 — UNDER the tightest HP cell's 1.125, so "crosses the tightest cell" is
/// refuted; "under the `h` = 0.7 cells' 1.546" holds. At `K_H` = 5 (cells `h` = 0.5, 1.3/1.4)
/// it reaches 1.1442 against 1.258: under. No HP cell crosses on the default shape at `λ` = 0.
#[test]
fn p5_the_hp_all_rows_schedule_stays_just_under_the_tightest_hp_redline() {
    let z4 = machine("flow/press", 0.5, 1.5, 0.0);
    let z5 = machine("flow/press", 0.5, 1.4, 0.0);
    assert_eq!((z4.hp.k, z5.hp.k), (4, 5));
    let r4 = z4.schedule(Lever::AllRows, Spool::Hp, &[1000.0])[0];
    let r5 = z5.schedule(Lever::AllRows, Spool::Hp, &[1000.0])[0];
    assert!(r4.reached && r5.reached);
    near(r4.n_hp, 1.1073, 0.00005, "P5 K_H=4");
    near(r5.n_hp, 1.1442, 0.00005, "K_H=5");
    assert!(r4.n_hp < z4.hp.redline && z4.hp.redline < 1.126, "the miss: under the tightest 1.125");
    assert!(r5.n_hp < z5.hp.redline);
}

/// P1/P2 — MISS on existence; "crosses before target" under V1. The LP ALL-ROWS schedule reaches
/// its target at 1500/1300 only (P1/P2 said all four, at `K` = 2 and 3): at 1100/1000 the scan
/// runs to the map edge (`v` 2.1/2.4) without the incidence coming down, and passes the redline
/// on the way (at travel ≈ 1.2–1.4) — so every LP cell CROSSES BEFORE TARGET there. Both bands
/// (P1 [1.30, 1.60], P2 [1.35, 1.70]) were for a reached `v*` and have nothing to score.
#[test]
fn p1_p2_the_lp_all_rows_schedule_ends_unreached_and_crosses_before_target() {
    for &(h, m) in &[(0.5, 1.3), (0.5, 1.4), (0.5, 1.5)] {
        let z = machine("flow/press", h, m, 0.0);
        let reads = z.schedule(Lever::AllRows, Spool::Lp, &THROTTLE);
        assert!(reads[0].reached && reads[1].reached, "K_L={}: reached at 1500/1300", z.lp.k);
        for r in &reads[2..] {
            assert!(!r.reached, "K_L={} Tt4={}: unreached", z.lp.k, r.tt4);
            assert_eq!(r.verdict(&z, Spool::Lp), Verdict::Crosses, "V1: crosses before target");
            assert_eq!(r.verdict(&z, Spool::Hp), Verdict::Under, "the unmoved spool is a reading, never Void");
        }
        assert!(reads[1].n_lp < z.lp.redline, "reached at 1300: under");
    }
}

/// P3 — HIT at its one scorable point. Built so ONLY `K_L` moves — `(1, 5)`, `(2, 5)`, `(3, 5)` —
/// against the anchor's wording; `K_L` = 1 is not a wall-produced machine (V4), disclosed. At
/// `Tt4` = 1300 all three reach and `N(1) < N(2) < N(3)`; at 1100/1000 `K_L` = 2, 3 are unreached
/// (P1/P2), so nothing more is scorable.
#[test]
fn p3_the_all_rows_bill_rises_with_the_lp_row_count() {
    let z = machine("flow/press", 0.5, 1.4, 0.0);
    let n_at = |k_lp: usize| {
        let p = StageStackCore::new(StageStackCoreSpec {
            k_lp, k_hp: 5, ..StageStackCoreSpec::new(design(), flight(), 1.0, z.map_lp, z.map_hp)
        });
        let r = p.stage_incidence_schedule(&flight(), &[1300.0], Spool::Lp, 0, 4.0)[0];
        assert!(r.reached, "K_L={k_lp} at 1300");
        p.at_setting(r.vsv_star, 0.0).match_point(&flight(), 1300.0).n_lp_ratio
    };
    let (n1, n2, n3) = (n_at(1), n_at(2), n_at(3));
    assert!(n1 < n2 && n2 < n3, "{n1} {n2} {n3}");
}

/// P7 — HIT, all four clauses. LP lumped at 1000, each shape against ITS OWN redlines (§ 6.3):
/// `tilted` crosses its tightest LP cell (1.337) and not its `h` = 0.7 cells; `press/flow` and
/// `flat-eta` cross none. (Estimates were 1.43 / 1.22 / 1.16; measured 1.4123 / 1.1948 / 1.1648.)
#[test]
fn p7_shape_decides_the_lp_lumped_verdict() {
    for (shape, crosses_tight, n) in [("tilted", true, 1.4123), ("press/flow", false, 1.1948), ("flat-eta", false, 1.1648)] {
        for &(h, m) in &CELLS {
            let z = machine(shape, h, m, 0.0);
            let r = z.schedule(Lever::Lumped, Spool::Lp, &[1000.0])[0];
            assert!(r.reached);
            near(r.n_lp, n, 0.00005, shape);
            let tight = (h, m) == (0.5, 1.5);
            let want = if crosses_tight && tight { Verdict::Crosses } else { Verdict::Under };
            assert_eq!(r.verdict(&z, Spool::Lp), want, "{shape} h={h} M={m} R={}", z.lp.redline);
        }
    }
}

/// P8 — HIT. The front-row LP lever vs bare, at the walls' `K`, `Tt4` ∈ {1300, 1100, 1000}: the
/// front row's relative tip Mach FALLS (by 13–16 % at 1000, bar ≥ 3 %) in every cell while the
/// physical `N_L` RISES.
#[test]
fn p8_the_front_row_lever_lowers_tip_mach_while_raising_shaft_speed() {
    for &(h, m) in &CELLS {
        let z = machine("flow/press", h, m, 0.0);
        for r in z.schedule(Lever::FrontRow, Spool::Lp, &[1300.0, 1100.0, 1000.0]) {
            assert!(r.reached);
            assert!(r.n_lp > r.n_lp_bare, "h={h} M={m} Tt4={}: N_L must rise", r.tt4);
            assert!(r.tip_mach_lp < r.tip_mach_lp_bare, "h={h} M={m} Tt4={}: tip Mach must fall", r.tt4);
            if r.tt4 == 1000.0 {
                assert!(r.tip_mach_lp <= 0.97 * r.tip_mach_lp_bare, "h={h} M={m}: by at least 3 %");
            }
        }
    }
}

/// Q2 — HIT, for a reason NOT the one registered. The LP lumped bill rises strictly over
/// `λ` ∈ {0, 0.5, 1} in all six cells, every point reached (the void count is met). But the
/// registered mechanism — "the reshaped map is steeper" — is REFUTED by the slope cancellation:
/// the held point carries no `l`. The rise is the CURVATURE `σ' = σ/r`, i.e. A9's disclosed
/// "curvature fixed in absolute work units". Fixed relative to design work instead, `λ` would
/// leave every scheduled speed unchanged.
/// Q3 — MISS. The far cell (`h` = 0.7, `M_rel,lim` = 1.5) at `λ` = 1: `N_L` = 1.3392, far below
/// the band [1.60, 2.60] and under its 1.601 — the extrapolation was in `l`, which does not enter.
/// Old-Q3 (no credit): the `h` = 0.5, 1.5 cell at `λ` = 1 does cross its redline — by 0.09 %.
#[test]
fn q2_q3_the_rounding_knob_raises_the_lumped_bill_through_curvature() {
    let base = machine("flow/press", 0.5, 1.4, 0.0).schedule(Lever::Lumped, Spool::Lp, &[1000.0])[0];
    for &(h, m) in &CELLS {
        let half = machine("flow/press", h, m, 0.5).schedule(Lever::Lumped, Spool::Lp, &[1000.0])[0];
        let z1 = machine("flow/press", h, m, 1.0);
        let one = z1.schedule(Lever::Lumped, Spool::Lp, &[1000.0])[0];
        assert!(base.reached && half.reached && one.reached, "Q2's void count: all reached");
        assert!(base.n_lp < half.n_lp && half.n_lp < one.n_lp, "h={h} M={m}: strictly rising in λ");
        if (h, m) == (0.7, 1.5) {
            near(one.n_lp, 1.3392, 0.00005, "Q3 far cell");
            assert!(one.n_lp < 1.60 && one.n_lp < z1.lp.redline, "Q3: outside its band, under 1.601");
        }
        if (h, m) == (0.5, 1.5) {
            let margin = one.n_lp / z1.lp.redline - 1.0;
            assert!(margin > 0.0 && margin < 0.002, "old-Q3: crosses, by {margin:e}");
        }
    }
}
