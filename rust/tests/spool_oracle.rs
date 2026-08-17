//! THE ORACLE, phase 6 slice P — `rung34/35/36`'s values held to PyPy's BITS.
//!
//! `slice_p_smoke.rs` was the go/no-go; **this is the gate.** It replays
//! `oracle/dump_spool.py`'s grid exactly and compares every key, and it is the only thing in the
//! slice that can settle § 5.13's predictions 1, 4, 5 and 6 — each of which is about something a
//! rung suite structurally cannot see.
//!
//! **WHY THE PREDICTIONS NEEDED A DUMP AND NOT A SUITE — MEASURED, NOT ARGUED.** Slice P swapped
//! rung 34's Illinois turbine solve for rung 31's bisection, the two agreeing only to ~9e-12, and
//! re-ran everything:
//!
//! | | gates failing |
//! |---|---|
//! | the 19 ported Python gates in `rung34.rs` + `rung35.rs` + `rung36.rs` | **0** |
//! | `slice_p_smoke.rs`'s bit comparisons | 6 of 9 |
//! | the two gates the PORT added | 2 |
//!
//! Every gate rung 34/35/36 ship is written at `1e-6`–`1e-9`, which is where their physics lives;
//! a defect four orders below that is invisible to all nineteen. That is the whole argument for
//! an oracle, and here it is a measurement rather than a principle.
//!
//! **THE CENSUS IS EMITTED AND COMPARED, NEVER RESTATED.** § 5.13's probe numbers (185 fallbacks,
//! 2 escalations, 16 508 `phi_max` calls) came off `probe_p.py`'s grid, which is NOT this one —
//! slice N step 4's lesson, that two censuses in one section can be measured on two grids and the
//! prose reads as though they share one. So the dump emits its own counts per section and this
//! file compares them; the probe numbers survive only as the reason the keys exist.

use std::collections::BTreeMap;

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::Gas;
use turbojet::map::{take_phi_max_arms, ComponentMap};
use turbojet::matcher::{take_r31_calls, Branch};
use turbojet::spool::{counters, SpoolTransient, TransientPoint};

const PI_C: f64 = 10.0;
const TT4: f64 = 1500.0;

fn flight() -> FlightCondition {
    FlightCondition { t0: 250.0, p0: 50_000.0, m0: 0.85 }
}

fn real() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, e_t: None,
        eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
    }
}

fn design() -> Engine {
    build_turbojet(Gas::thermally_perfect(), PI_C, TT4, 50_000.0, real())
}

fn st(cmap: ComponentMap) -> SpoolTransient {
    SpoolTransient::new(design(), flight(), 1.0, cmap)
}

fn shapes() -> [(&'static str, ComponentMap); 7] {
    [
        ("flat", ComponentMap::flat()),
        ("flow_dom", ComponentMap::flow_dominated()),
        ("press_dom", ComponentMap::pressure_dominated()),
        ("tilted", ComponentMap::tilted()),
        ("surge_flow", ComponentMap::surge_flow()),
        ("surge_pressure", ComponentMap::surge_pressure()),
        ("surge_tilted", ComponentMap::surge_tilted()),
    ]
}

fn shape_of(name: &str) -> ComponentMap {
    shapes().iter().find(|(n, _)| *n == name).expect("known shape").1
}

const THROTTLES: [f64; 6] = [1500.0, 1300.0, 1100.0, 900.0, 700.0, 520.0];

/// Python's `f"{x:.0f}"` for the throttle labels in the keys.
fn t0(x: f64) -> String {
    format!("{x:.0}")
}

// --------------------------------------------------------------------------- the recorder
/// Every value the Rust produces, keyed exactly as the dump keys it.
///
/// A `BTreeMap` rather than a `Vec` so a duplicate key is caught on insert — the same guard the
/// dump asserts on its own side. The two sides are then compared as SETS as well as values,
/// because *coverage is a name diff, never a count* (slice N FINDING 5).
#[derive(Default)]
struct Rec {
    v: BTreeMap<String, u64>,
}

impl Rec {
    fn put(&mut self, key: String, value: f64) {
        assert!(value.is_finite(), "{key} is not finite: {value}");
        assert!(self.v.insert(key.clone(), value.to_bits()).is_none(), "duplicate key {key}");
    }
    fn putd(&mut self, key: String, n: u64) {
        assert!(self.v.insert(key.clone(), n).is_none(), "duplicate key {key}");
    }
}

const EQ_KEYS: [&str; 24] = [
    "nu", "n", "pi_c", "tau_c", "mdot_air", "f", "pi_t", "tau_t", "Tt3", "Tt5", "flowcoef",
    "Phi", "sp_thrust", "M9", "pt9_over_p0", "eta_c", "eta_t", "nu_t", "p_net_spec", "m",
    "thrust", "Tt2", "pt2", "V0",
];

fn eq_value(i: &turbojet::spool::Instant, key: &str) -> f64 {
    match key {
        "nu" => i.nu, "n" => i.n, "pi_c" => i.pi_c, "tau_c" => i.tau_c,
        "mdot_air" => i.mdot_air, "f" => i.f, "pi_t" => i.pi_t, "tau_t" => i.tau_t,
        "Tt3" => i.tt3, "Tt5" => i.tt5, "flowcoef" => i.flowcoef, "Phi" => i.phi,
        "sp_thrust" => i.sp_thrust, "M9" => i.m9, "pt9_over_p0" => i.pt9_over_p0,
        "eta_c" => i.eta_c, "eta_t" => i.eta_t, "nu_t" => i.nu_t,
        "p_net_spec" => i.p_net_spec, "m" => i.m, "thrust" => i.thrust, "Tt2" => i.tt2,
        "pt2" => i.pt2, "V0" => i.v0,
        _ => unreachable!("unknown equilibrium key {key}"),
    }
}

const PT_KEYS: [&str; 12] = [
    "s", "nu", "Tt4", "pi_c", "tau_c", "mdot_air", "f", "tau_t", "Phi", "sp_thrust", "M9",
    "pt9_over_p0",
];

fn pt_value(p: &TransientPoint, key: &str) -> f64 {
    match key {
        "s" => p.s, "nu" => p.nu, "Tt4" => p.tt4, "pi_c" => p.pi_c, "tau_c" => p.tau_c,
        "mdot_air" => p.mdot_air, "f" => p.f, "tau_t" => p.tau_t, "Phi" => p.phi,
        "sp_thrust" => p.sp_thrust, "M9" => p.m9, "pt9_over_p0" => p.pt9_over_p0,
        _ => unreachable!("unknown point key {key}"),
    }
}

fn branch_index(b: Branch) -> u64 {
    match b {
        Branch::Choked => 0,
        Branch::Subsonic => 1,
    }
}

fn dump_traj(r: &mut Rec, prefix: &str, traj: &[TransientPoint]) {
    r.putd(format!("{prefix}/n_pts"), traj.len() as u64);
    let mut i = 0;
    while i < traj.len() {
        r.putd(format!("{prefix}/{i}/branch"), branch_index(traj[i].branch));
        for k in PT_KEYS {
            r.put(format!("{prefix}/{i}/{k}"), pt_value(&traj[i], k));
        }
        i += 7;
    }
    if let Some(p) = traj.last() {
        r.putd(format!("{prefix}/last/branch"), branch_index(p.branch));
        for k in PT_KEYS {
            r.put(format!("{prefix}/last/{k}"), pt_value(p, k));
        }
    }
}

/// Take the census and write it under `prefix/`, resetting every counter — the dump's
/// `emit_census`, key for key.
fn emit_census(r: &mut Rec, prefix: &str) {
    let c = counters::take();
    let arms = take_phi_max_arms();
    let _ = take_r31_calls(); // read so it cannot leak between sections; asserted separately
    for (name, n) in [
        ("illinois_calls", c.illinois_calls),
        ("illinois_evals", c.illinois_evals),
        ("illinois_exhausted", c.illinois_exhausted),
        ("phi_max_flat5", arms[0]),
        ("phi_max_linear", arms[2]),
        ("phi_max_quadratic", arms[1]),
        ("phi_max_swirled", arms[3]),
        ("r34_solve_turbine", c.r34_solve_turbine),
        ("subsonic_escalations", c.subsonic_escalations),
        // Python counts the RAISE out of `_turbine_subsonic`; the two Rust arms partition it,
        // since every failure is either absorbed by the `M9 > 0.985` guard or escalated by it.
        ("subsonic_raises", c.subsonic_fallbacks + c.subsonic_escalations),
    ] {
        r.putd(format!("census/{prefix}/{name}"), n);
    }
}

// ------------------------------------------------------------------------- the replay
fn rust_values() -> Rec {
    let mut r = Rec::default();
    let _ = counters::take();
    let _ = take_phi_max_arms();
    let _ = take_r31_calls();

    // --- section 1: equilibria ------------------------------------------------------------
    let (mut n_choked, mut n_subsonic) = (0u64, 0u64);
    for (name, cmap) in shapes() {
        let s = st(cmap);
        for tt4 in THROTTLES {
            let eq = s.equilibrium(&flight(), tt4, None);
            let tag = format!("eq/{name}/{}", t0(tt4));
            r.putd(format!("{tag}/branch"), branch_index(eq.branch));
            match eq.branch {
                Branch::Choked => n_choked += 1,
                Branch::Subsonic => n_subsonic += 1,
            }
            for k in EQ_KEYS {
                r.put(format!("{tag}/{k}"), eq_value(&eq, k));
            }
        }
    }
    emit_census(&mut r, "equilibria");
    r.putd("cells/choked".into(), n_choked);
    r.putd("cells/subsonic".into(), n_subsonic);

    // --- section 2: the Tt4 marches -------------------------------------------------------
    for name in ["surge_flow", "flow_dom", "flat"] {
        let cmap = shape_of(name);
        let s = st(cmap);
        for (rl, rv) in [("0.1", 0.1f64), ("1.0", 1.0), ("5.0", 5.0)] {
            let d = s.ramp_excursion(&flight(), 1100.0, 1450.0, rv, None, 8.0, 0.05);
            r.put(format!("ramp/{name}/{rl}/E"), d.e);
            r.put(format!("ramp/{name}/{rl}/nu0"), d.nu0);
            dump_traj(&mut r, &format!("ramp/{name}/{rl}"), &d.traj);
        }
        let nu0 = s.equilibrium(&flight(), 1100.0, None).nu;
        let traj = s.integrate(&flight(), |_| 600.0, nu0, 8.0, 0.05, None);
        dump_traj(&mut r, &format!("spooldown/{name}"), &traj);
        r.putd(
            format!("spooldown/{name}/nu_floor_hits"),
            traj.iter().filter(|p| p.nu == 0.2).count() as u64,
        );
        r.put(
            format!("const_speed/{name}"),
            s.constant_speed_excursion(&flight(), 1100.0, 1450.0, None),
        );
    }
    emit_census(&mut r, "tt4_marches");

    // --- section 3: rung 35, fuel control -------------------------------------------------
    for name in ["surge_flow", "surge_tilted", "flow_dom"] {
        let cmap = shape_of(name);
        let s = st(cmap);
        for tt4 in [1400.0f64, 1100.0] {
            let mf = s.fuel_for_tt4(&flight(), tt4, None);
            let tag = format!("fuel/{name}/{}", t0(tt4));
            r.put(format!("{tag}/mf"), mf);
            let eq = s.equilibrium_fuel(&flight(), mf, None);
            for k in EQ_KEYS {
                r.put(format!("{tag}/{k}"), eq_value(&eq, k));
            }
            r.put(format!("{tag}/Tt4_out"), eq.tt4);
        }
        let d = s.ramp_excursion_fuel(&flight(), 1250.0, 1450.0, 1.0, None, 6.0, 0.05);
        for (k, v) in [
            ("E_surge", d.e_surge), ("E_temp", d.e_temp), ("Tt4_peak", d.tt4_peak),
            ("nu0", d.nu0),
        ] {
            r.put(format!("fuelramp/{name}/{k}"), v);
        }
        dump_traj(&mut r, &format!("fuelramp/{name}"), &d.traj);
        let (s0, t0v, peak, target) =
            s.constant_speed_excursion_fuel(&flight(), 1250.0, 1450.0, None);
        for (k, v) in [
            ("E_surge0", s0), ("E_temp0", t0v), ("Tt4_peak", peak), ("Tt4_target", target),
        ] {
            r.put(format!("fuelstep/{name}/{k}"), v);
        }
        for (tt3l, tt3, fl, f) in [
            ("650", 650.0f64, "0.02", 0.020f64),
            ("700", 700.0, "0.025", 0.025),
            ("600", 600.0, "0.03", 0.030),
        ] {
            r.put(format!("tt4_from_f/{name}/{tt3l}/{fl}"), s.tt4_from_f(tt3, f));
        }
    }
    emit_census(&mut r, "fuel");

    // --- section 4: rung 36, the surge line -----------------------------------------------
    for name in ["surge_flow", "surge_pressure", "surge_tilted"] {
        let base = shape_of(name);
        for (pl, pv) in [("0.55", 0.55f64), ("0.65", 0.65), ("0.75", 0.75)] {
            let cm = base.with_phi_surge(pv);
            let s = st(base);
            let sched = s.surge_margin_schedule(
                &flight(), &[1500.0, 1300.0, 1100.0, 900.0, 800.0, 700.0], Some(&cm),
            );
            r.putd(format!("sm/{name}/{pl}/n_rows"), sched.len() as u64);
            for row in &sched {
                let tag = format!("sm/{name}/{pl}/{}", t0(row.tt4));
                for (k, v) in [
                    ("nu", row.nu), ("n", row.n), ("phi_op", row.phi_op),
                    ("phi_surge", row.phi_surge), ("pi_c", row.pi_c), ("SM_N", row.sm_n),
                    ("SM_flow", row.sm_flow),
                ] {
                    r.put(format!("{tag}/{k}"), v);
                }
            }
            for lo in [1400.0f64, 1000.0, 800.0, 700.0] {
                let b = s.acceleration_binding(&flight(), lo, 1500.0, Some(&cm));
                let tag = format!("ab/{name}/{pl}/{}", t0(lo));
                for (k, v) in [
                    ("nu0", b.nu0), ("E0", b.e0), ("SM_N", b.sm_n), ("ratio", b.ratio),
                    ("phi_step", b.phi_step), ("phi_surge", b.phi_surge),
                ] {
                    r.put(format!("{tag}/{k}"), v);
                }
                r.putd(format!("{tag}/reaches_surge"), b.reaches_surge as u64);
                r.putd(format!("{tag}/phi_step_le_surge"), b.phi_step_le_surge as u64);
            }
        }
    }
    emit_census(&mut r, "surge");

    // --- section 5: rung 41's channels (slice L's deferral) -------------------------------
    for name in ["surge_flow", "surge_tilted"] {
        let base = shape_of(name);
        let cm = base.with_phi_surge(0.65);
        let s = st(base);
        for tt4 in [1500.0f64, 1300.0, 1100.0, 900.0, 800.0] {
            let ch = s.surge_margin_channels(&flight(), tt4, Some(&cm), None);
            let tag = format!("ch/{name}/{}", t0(tt4));
            for (k, v) in [
                ("n", ch.n), ("phi_op", ch.phi_op), ("pi_c", ch.pi_c), ("SM_N", ch.sm_n),
                ("SM_phi_walk", ch.sm_phi_walk), ("SM_speed_line", ch.sm_speed_line),
                ("SM_ref", ch.sm_ref),
            ] {
                r.put(format!("{tag}/{k}"), v);
            }
        }
    }
    emit_census(&mut r, "channels");

    // --- section 6: phi_max, every arm driven DIRECTLY -------------------------------------
    let flat = ComponentMap::flat();
    let direct: [(&str, ComponentMap); 8] = [
        ("flat", ComponentMap::flat()),
        ("quad", ComponentMap::surge_flow()),
        ("quad2", ComponentMap { sigma: 0.2, l: 0.85, ..flat }),
        ("linear", ComponentMap { sigma: 0.0, l: 0.7, ..flat }),
        ("linear2", ComponentMap { sigma: 0.0, l: 1.4, ..flat }),
        ("swirl", ComponentMap { sigma: 0.1, l: 0.7, ..flat }.with_vsv(0.20)),
        ("swirl_lin", ComponentMap { sigma: 0.0, l: 0.7, ..flat }.with_vsv(0.10)),
        ("swirl_neg", ComponentMap { sigma: 0.1, l: 0.7, ..flat }.with_vsv(-0.15)),
    ];
    for (label, cm) in direct {
        for (fl, fv) in [("0.1", 0.1f64), ("0.2", 0.2), ("0.35", 0.35)] {
            r.put(format!("phi_max/{label}/{fl}"), cm.phi_max(fv));
        }
    }
    emit_census(&mut r, "phi_max_direct");

    // --- section 7: the map inverse -------------------------------------------------------
    for name in ["surge_flow", "surge_pressure", "surge_tilted", "flat"] {
        let cmap = shape_of(name);
        let s = st(cmap);
        for (nl, n) in [("0.6", 0.6f64), ("0.75", 0.75), ("0.9", 0.9), ("1.0", 1.0), ("1.1", 1.1)]
        {
            for (ml, m) in [("0.5", 0.5f64), ("0.8", 0.8), ("1.0", 1.0), ("1.2", 1.2)] {
                let tc = s.tau_c_forward(&cmap, n, m);
                r.put(format!("inv/{name}/{nl}/{ml}/tau_c"), tc);
                r.put(
                    format!("inv/{name}/{nl}/{ml}/n_back"),
                    cmap.solve_n(m, tc, s.inner.tau_c_d),
                );
            }
        }
    }
    r
}

fn load(path: &str) -> BTreeMap<String, u64> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{path}: {e} — regenerate with oracle/dump_spool.py"));
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        let mut it = line.split('\t');
        let (k, b) = (it.next().unwrap(), it.next().unwrap());
        assert!(m.insert(k.to_string(), b.parse::<u64>().unwrap()).is_none(), "dup {k}");
    }
    m
}

// ============================================================================== the gates
/// **§ 5.13 PREDICTION 1** — every value the Rust produces equals PyPy's, BIT FOR BIT.
///
/// The RK4 has no adaptive control, so it carries accumulation order and no stopping rule (§ 4.3's
/// precedent), and every solver under it is an Illinois already proven exact in slices I/J.
///
/// The key SETS are compared before the values, because a Rust that produced 200 fewer keys and
/// agreed on all of them would otherwise read as a pass — *coverage is a name diff, never a
/// count* (slice N FINDING 5, which found a flag that had disarmed itself and whose arm came back
/// 71 504 keys instead of 41 560, reading as "about the same").
#[test]
fn every_value_is_bit_exact_against_pypy() {
    let py = load("oracle/spool_pypy.tsv");
    let rs = rust_values();

    let pk: std::collections::BTreeSet<_> = py.keys().cloned().collect();
    let rk: std::collections::BTreeSet<_> = rs.v.keys().cloned().collect();
    let only_py: Vec<_> = pk.difference(&rk).take(8).collect();
    let only_rs: Vec<_> = rk.difference(&pk).take(8).collect();
    assert!(
        only_py.is_empty() && only_rs.is_empty(),
        "key SETS differ — only in PyPy: {only_py:?}; only in Rust: {only_rs:?}"
    );

    let mut bad: Vec<String> = Vec::new();
    for (k, &want) in &py {
        let got = rs.v[k];
        if got != want {
            bad.push(format!(
                "{k}: rust {got} ({}) != pypy {want} ({})",
                f64::from_bits(got),
                f64::from_bits(want)
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "{} of {} keys differ:\n  {}",
        bad.len(),
        py.len(),
        bad.iter().take(12).cloned().collect::<Vec<_>>().join("\n  ")
    );
    println!("spool_oracle: {} keys, 100% bit-exact against PyPy", py.len());
    assert!(py.len() > 7000, "the dump shrank — {} keys", py.len());
}

/// **§ 5.13 PREDICTION 4** — `phi_max`'s LINEAR arm is unreachable through a rung-34 march, and
/// the counter that says so is not vacuous.
///
/// Every marching section must show `phi_max_linear == 0` **and** a nonzero tally somewhere else
/// (or the counter is simply broken and reports zero for everything). Section 6 drives all four
/// arms directly, which is what makes the zeros above evidence rather than silence — the same
/// two-sidedness rung 55's `test_p6_verdicts_survive_the_work_split` needed.
#[test]
fn the_phi_max_linear_arm_is_dead_on_every_marched_section() {
    let py = load("oracle/spool_pypy.tsv");
    let get = |k: &str| *py.get(k).unwrap_or_else(|| panic!("missing census key {k}"));
    for sec in ["equilibria", "tt4_marches", "fuel", "surge", "channels"] {
        assert_eq!(get(&format!("census/{sec}/phi_max_linear")), 0, "{sec}: linear arm fired");
        assert_eq!(get(&format!("census/{sec}/phi_max_swirled")), 0, "{sec}: swirl arm fired");
    }
    let live: u64 = ["equilibria", "tt4_marches", "fuel", "surge", "channels"]
        .iter()
        .map(|s| get(&format!("census/{s}/phi_max_quadratic")) + get(&format!("census/{s}/phi_max_flat5")))
        .sum();
    assert!(live > 1000, "the phi_max counter reports {live} live calls — it is broken, not the \
                          arms being dead");
    // ...and the DIRECT section proves the two dead arms are reachable at all.
    assert!(get("census/phi_max_direct/phi_max_linear") > 0, "section 6 must drive the linear arm");
    assert!(get("census/phi_max_direct/phi_max_swirled") > 0, "section 6 must drive the swirl arm");
}

/// **§ 5.13 PREDICTION 5** — the two arms of the `M9 > 0.985` guard, both live and 90× apart.
///
/// The prediction registered 185 fallbacks against 2 escalations. Those were `probe_p.py`'s grid,
/// **not this one**, so the numbers are read off the dump rather than restated — slice N step 4's
/// rule. What is asserted is the STRUCTURE the prediction was about: on the marching section both
/// arms fire, and neither is a rounding artefact of the other.
#[test]
fn both_arms_of_the_subsonic_guard_are_live() {
    let py = load("oracle/spool_pypy.tsv");
    let get = |k: &str| *py.get(k).unwrap_or_else(|| panic!("missing census key {k}"));
    let raises = get("census/tt4_marches/subsonic_raises");
    let esc = get("census/tt4_marches/subsonic_escalations");
    assert!(raises > 100, "the subsonic bracket must FAIL often on the marches: {raises}");
    assert!(esc > 0, "the ESCALATION arm must fire — it is the branch the source insists must \
                      raise rather than hide under a `subsonic` label, and a port that turned it \
                      into a silent fallback would move no value key at all");
    assert!(raises > esc, "an escalation is a raise, so raises must exceed escalations");
    assert!(
        raises - esc > 50,
        "the FALLBACK arm must fire too, or the guard is a pure re-raise: {} fallbacks",
        raises - esc
    );
    // The exhaustion arm of `try_illinois` stays unreachable everywhere — the reason its `Ok(b)`
    // is invisible to every value gate, recorded so a future reader does not read 0 as untested.
    for sec in ["equilibria", "tt4_marches", "fuel", "surge", "channels"] {
        assert_eq!(get(&format!("census/{sec}/illinois_exhausted")), 0, "{sec}: maxit exhausted");
    }
}

/// **§ 5.13 PREDICTION 6** — a trajectory's LENGTH is an output, it varies by MAP SHAPE, and it
/// is bit-reproduced.
///
/// The value comparison above already pins every `n_pts` key. What this adds is that the lengths
/// are not all the same — otherwise the prediction is satisfied by a march that never breaks, and
/// the `except AssertionError` that makes length an output would be dead code the gate cannot see.
#[test]
fn the_spooldown_length_varies_by_map_shape() {
    let py = load("oracle/spool_pypy.tsv");
    let lens: Vec<u64> = ["surge_flow", "flow_dom", "flat"]
        .iter()
        .map(|s| py[&format!("spooldown/{s}/n_pts")])
        .collect();
    let full = 161u64; // (8.0 / 0.05) + 1
    assert!(
        lens.iter().any(|&l| l < full),
        "no spool-down terminated early — the `break` that makes LENGTH an output is dead on \
         this grid and prediction 6 is untested: {lens:?}"
    );
    assert!(
        lens.iter().any(|&l| l == full),
        "every spool-down terminated early — then the length is not SHAPE-dependent, it is just \
         short: {lens:?}"
    );
    assert!(
        lens.iter().collect::<std::collections::BTreeSet<_>>().len() > 1,
        "the lengths must DIFFER by shape: {lens:?}"
    );
    // The sub-idle floor is dead on this grid. Declared, not silently absent.
    for s in ["surge_flow", "flow_dom", "flat"] {
        assert_eq!(
            py[&format!("spooldown/{s}/nu_floor_hits")], 0,
            "the nu = max(0.2, .) floor fired on {s} — it has never fired before, so either the \
             grid moved or the march stopped breaking first"
        );
    }
}

/// **§ 5.13 PREDICTION 2, second clause** — rung 31's bisection never runs on a `SpoolTransient`.
///
/// `rung34.rs` asserts this on one equilibrium; here it is asserted over the WHOLE dump grid,
/// which is 7 300 keys of marches, fuel closures, surge schedules and channel decompositions.
#[test]
fn rung31s_bisection_never_runs_anywhere_on_the_grid() {
    // The replay's own `emit_census` reads and RESETS this counter per section, so running it
    // here would leave only the last section's tally. Drive a representative slice of the grid
    // directly instead, with the counter untouched from entry to assertion.
    let _ = take_r31_calls();
    let s = st(ComponentMap::surge_flow());
    for tt4 in THROTTLES {
        let _ = s.equilibrium(&flight(), tt4, None);
    }
    let nu0 = s.equilibrium(&flight(), 1100.0, None).nu;
    let _ = s.integrate(&flight(), |_| 600.0, nu0, 8.0, 0.05, None);
    let _ = s.ramp_excursion_fuel(&flight(), 1250.0, 1450.0, 1.0, None, 6.0, 0.05);
    let n = take_r31_calls();
    assert_eq!(
        n, 0,
        "rung 31's bisection ran {n} times under a rung-34 march. The two solvers agree only to \
         ~9e-12, and slice P measured that ALL 19 ported Python gates still pass when they are \
         swapped — this counter and the bit oracle above are the only things that can see it"
    );
}

/// **THE CPython ARM** — the port's standing sensitivity instrument, and slice P's is the
/// sharpest since slice G.
///
/// This gate compares Python against Python; the Rust reads it. The port's contract is
/// Rust ≡ PyPy bit-for-bit (above), and CPython answers a different question: *how much of this
/// agreement is arithmetic that could not have gone otherwise, and how much is a real
/// reproduction?* Slice K's 46.3 % and slice G's 8.0 % are the scale.
///
/// **MEASURED: 1 652 of 7 300 keys identical — 22.6 %.** So the bit-exactness above is not
/// something the arithmetic hands you: 83.8 % of the continuous keys move between two correct
/// implementations of the same language. The § 5.12 pre-flight had warned the opposite way — its
/// probe got **100 %** and recorded that as NOT coverage, because it ran a CPG gas whose
/// properties are closed-form. This grid runs the thermally-perfect gas, and the warning is
/// vindicated: the same instrument on the harder gas is a sharp detector.
///
/// **AND THE TIERING IS THE FINDING: EVERY DISCRETE *OUTPUT* IS INTERPRETER-INVARIANT.** Of 530
/// keys that are branch labels, trajectory lengths, schedule row counts, surge verdicts and cell
/// tallies, **zero** differ — while 22 of 60 census keys (solver iteration counts) do. The
/// physics' discrete answers are stable; only how many passes it took to get there moves. That is
/// the opposite of slice N's finding, where 520 discrete argmin keys flipped at the design
/// throttle because the quantities being compared had collapsed to 1–2 ULP.
///
/// **TWO STRUCTURAL-ZERO FAMILIES, NAMED SO A LATER READER DOES NOT RE-DERIVE THEM.**
/// * `Phi` and `p_net_spec` at an equilibrium ARE the residual driven to zero — 483 keys whose
///   worst *absolute* gap is 1.06e-5 and whose relative gap is meaningless.
/// * `E_surge` on a PEAKED map is `max(0, …)` sitting AT its clamp: rung 34's gate 4 says
///   `flow_dominated` run forward gives a NEGATIVE excursion, so the accumulator never leaves
///   0.0. PyPy returns **exactly 0.0** and CPython **1.61e-11** — a 100 % relative difference
///   from a 1e-11 drift. A `max`-accumulated quantity at its floor converts arbitrary small drift
///   into total relative disagreement, which is why the arm below is tiered on KIND and not on a
///   single tolerance.
#[test]
fn the_cpython_arm_is_a_detector_and_every_discrete_output_survives_it() {
    let py = load("oracle/spool_pypy.tsv");
    let cp = load("oracle/spool_cpython.tsv");
    assert_eq!(
        py.keys().collect::<Vec<_>>(),
        cp.keys().collect::<Vec<_>>(),
        "the two interpreter dumps must cover the SAME keys — coverage is a name diff, never a \
         count (slice N FINDING 5)"
    );

    let is_output_discrete = |k: &str| {
        k.ends_with("/n_pts") || k.ends_with("/branch") || k.ends_with("/n_rows")
            || k.ends_with("reaches_surge") || k.ends_with("phi_step_le_surge")
            || k.starts_with("cells/") || k.ends_with("nu_floor_hits")
    };

    let (mut n_out, mut bad_out) = (0usize, Vec::new());
    let (mut n_cen, mut d_cen) = (0usize, 0usize);
    let mut n_same = 0usize;
    for (k, &a) in &py {
        let b = cp[k];
        if a == b {
            n_same += 1;
        }
        if is_output_discrete(k) {
            n_out += 1;
            if a != b {
                bad_out.push(format!("{k}: pypy {a} vs cpython {b}"));
            }
        } else if k.starts_with("census/") {
            n_cen += 1;
            d_cen += (a != b) as usize;
        }
    }

    assert!(
        bad_out.is_empty(),
        "{} of {n_out} DISCRETE OUTPUT keys differ between interpreters — the branch labels, \
         trajectory lengths and surge verdicts are supposed to be arithmetic-independent:\n  {}",
        bad_out.len(),
        bad_out.iter().take(10).cloned().collect::<Vec<_>>().join("\n  ")
    );
    assert_eq!(n_out, 530, "the discrete-output family changed size — re-read the tiering");

    // The arm must be LIVE, or the paragraph above is a story about a dump that agrees with
    // itself. Both clauses: the census moves, and the file as a whole is far from identical.
    assert!(
        d_cen > 10,
        "only {d_cen} of {n_cen} census keys differ across interpreters — the solver iteration \
         counts are supposed to be the interpreter-sensitive half"
    );
    let pct = 100.0 * n_same as f64 / py.len() as f64;
    assert!(
        pct < 40.0,
        "the CPython arm agrees on {pct:.1}% of keys — that is not a detector, and the \
         bit-exactness gate above is measuring less than it appears to (§ 5.12's CPG-gas warning)"
    );
    println!(
        "cpython arm: {n_same}/{} identical ({pct:.1}%), {d_cen}/{n_cen} census keys differ, \
         {n_out}/{n_out} discrete outputs survive",
        py.len()
    );
}
