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
//!   4. P8's identity — the tip-Mach reading at design equals the sizing's `M_rel,d` (V3).
//!   5. NO PANIC BEHIND A KNOB — out-of-range knobs and a choking design are `Err`, not a crash.
//!   6. THE EXPERIMENT — § 4.1 / § 4.5's grid, measured (`score_*`).

use turbojet::blade_speed::{build, reshape, size, BladeKnobs, Binding, Lever, Machine, SizingError,
                            SpoolDuty, Verdict};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
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
// GATE 4 — P8's IDENTITY (V3)
// ==========================================================================================

#[test]
fn the_tip_mach_reading_at_design_is_the_sizing_level() {
    for &(h, m) in &CELLS {
        for lambda in [0.0, 0.5, 1.0] {
            let z = machine("flow/press", h, m, lambda);
            for s in [&z.lp, &z.hp] {
                assert!((s.tip_rel_mach(1.0, 1.0, 0.0) - s.m_rel_d).abs() <= 1e-12);
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
// GATE 5 — NO PANIC BEHIND A KNOB
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
    // Φ_d = 1 at a high level with the blades at the wall: the front row's axial flow goes
    // sonic — reported, not a panic (P-C's regime).
    let hot = BladeKnobs { phi_d: 1.2, m_rel_lim: 1.6, lambda: 1.0, ..BladeKnobs::default() };
    assert!(matches!(size(duty, hot), Err(SizingError::Chokes { .. })));
}

// ==========================================================================================
// GATE 6 — THE EXPERIMENT (measured; scored in docs/rung85-spec.md)
// ==========================================================================================

fn show(tag: &str, z: &Machine, reads: &[turbojet::blade_speed::LeverRead]) {
    for r in reads {
        println!("{tag:<34} Tt4={:6.0} {:?} reached={:<5} v*={:.4}  N_L={:.4} (R {:.3}, {:?})  N_H={:.4} (R {:.3}, {:?})  bareN_L={:.4} bareN_H={:.4}  Mtip_L={:.3} Mtip_H={:.3}",
                 r.tt4, r.spool, r.reached, r.vsv_star, r.n_lp, z.lp.redline, r.verdict(z, Spool::Lp),
                 r.n_hp, z.hp.redline, r.verdict(z, Spool::Hp), r.n_lp_bare, r.n_hp_bare,
                 r.tip_mach_lp, r.tip_mach_hp);
    }
}

#[test]
#[ignore = "the scoring run — slow; run with --ignored --nocapture"]
fn score_the_grid() {
    let grid = [1500.0, 1300.0, 1100.0, 1000.0];
    let _ = Verdict::Void;
    // λ = 0, default shape: the three distinct (K_L, K_H) machines (§ 6.2's staircase) and the
    // lumped plant (identical in every cell — the maps do not move at λ = 0).
    for &(h, m) in &[(0.5, 1.3), (0.5, 1.4), (0.5, 1.5)] {
        let z = machine("flow/press", h, m, 0.0);
        let tag = format!("K=({},{}) h={h} M={m}", z.lp.k, z.hp.k);
        show(&format!("{tag} ALL LP"), &z, &z.schedule(Lever::AllRows, Spool::Lp, &grid));
        show(&format!("{tag} ALL HP"), &z, &z.schedule(Lever::AllRows, Spool::Hp, &grid));
        show(&format!("{tag} FRONT LP"), &z, &z.schedule(Lever::FrontRow, Spool::Lp, &grid));
        show(&format!("{tag} FRONT HP"), &z, &z.schedule(Lever::FrontRow, Spool::Hp, &grid));
    }
    let z = machine("flow/press", 0.5, 1.4, 0.0);
    show("lumped LP", &z, &z.schedule(Lever::Lumped, Spool::Lp, &grid));
    show("lumped HP", &z, &z.schedule(Lever::Lumped, Spool::Hp, &grid));
    // P3's K = 1 leg: the LP lumped, HP at 5 (the pair (1,5)).
    // (Lever::Lumped is (1,1); P3 needs the LP alone at 1 — read through AllRows on a K_L = 1 cell
    // is not producible by the walls, so P3's K = 1 point is the lumped LP schedule above.)
    // P7 — the other shapes, LP lumped at λ = 0 against each shape's own redlines.
    for shape in ["press/flow", "tilted", "flat-eta"] {
        for &(h, m) in &[(0.5, 1.5), (0.7, 1.5)] {
            let z = machine(shape, h, m, 0.0);
            show(&format!("{shape} h={h} M={m} lumped LP"), &z, &z.schedule(Lever::Lumped, Spool::Lp, &[1000.0]));
        }
    }
    // Q2/Q3 — λ ∈ {0.5, 1}, every LP cell, LP lumped at 1000.
    for lambda in [0.5, 1.0] {
        for &(h, m) in &CELLS {
            let z = machine("flow/press", h, m, lambda);
            let tag = format!("λ={lambda} h={h} M={m} r=({:.3},{:.3})", z.lp.r, z.hp.r);
            show(&format!("{tag} lumped LP"), &z, &z.schedule(Lever::Lumped, Spool::Lp, &[1000.0]));
        }
    }
}

#[test]
#[ignore = "diagnostic run — P8 in every cell, and the σ-vs-l split behind Q3; --ignored --nocapture"]
fn score_p8_and_the_q3_split() {
    for &(h, m) in &CELLS {
        let z = machine("flow/press", h, m, 0.0);
        for r in z.schedule(Lever::FrontRow, Spool::Lp, &[1300.0, 1100.0, 1000.0]) {
            println!("P8 h={h} M={m} K=({},{}) Tt4={:.0} reached={} N_L {:.4} -> {:.4}  Mtip_L {:.4} -> {:.4} ({:+.2} %)",
                     z.lp.k, z.hp.k, r.tt4, r.reached, r.n_lp_bare, r.n_lp, r.tip_mach_lp_bare, r.tip_mach_lp,
                     100.0 * (r.tip_mach_lp / r.tip_mach_lp_bare - 1.0));
        }
    }
    // Q3's split: the LP lumped bill at 1000 over (σ, l) on two η islands, the shipped λ = 0
    // path (maps passed as given; r = 1).
    let f = ComponentMap::flat();
    for (isl, base) in [("flow/press η", ComponentMap { a: 0.20, b: 0.05, ..f }),
                        ("tilted η", ComponentMap { a: 0.14, b: 0.10, c: 0.06, ..f })] {
        for (sigma, l) in [(0.1, 0.7), (0.1, 0.85), (0.2, 0.7), (0.2, 0.85), (0.1435, 1.446), (0.1, 1.446), (0.1435, 0.7)] {
            let ml = ComponentMap { sigma, l, ..base }.with_phi_surge(FLOOR);
            let (_, mh) = maps("flow/press");
            let z = build(&design(), flight(), ml, mh, knobs(0.5, 1.4, 0.0), knobs(0.5, 1.4, 0.0)).unwrap();
            let r = z.schedule(Lever::Lumped, Spool::Lp, &[1000.0])[0];
            println!("Q3 split {isl:<13} σ={sigma:<6} l={l:<6} reached={} v*={:.4} N_L={:.4} bare={:.4} bill={:+.2} %",
                     r.reached, r.vsv_star, r.n_lp, r.n_lp_bare, 100.0 * (r.n_lp / r.n_lp_bare - 1.0));
        }
    }
}

#[test]
#[ignore = "diagnostic — where along its scan the unreached LP all-rows schedule passes the redline; --ignored --nocapture"]
fn where_the_unreached_all_rows_schedule_crosses() {
    for &(h, m) in &[(0.5, 1.3), (0.5, 1.4)] {
        let z = machine("flow/press", h, m, 0.0);
        let plant = z.plant(Lever::AllRows, Spool::Lp);
        for tt4 in [1100.0, 1000.0] {
            let mut line = format!("K=({},{}) R={:.3} Tt4={tt4:.0}:", z.lp.k, z.hp.k, z.lp.redline);
            for v in [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0] {
                let o = plant.at_setting(v, 0.0).match_point(&flight(), tt4);
                line += &format!("  v={v}: N_L={:.3} φ={:.3}", o.n_lp_ratio, o.phi_lp);
            }
            println!("{line}");
        }
    }
}
