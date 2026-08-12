//! PHASE 3B GATE — every rung-10/11/12/20 finite-quench value the Python oracle dumped,
//! recomputed in Rust.
//!
//! The fourth in the family (`gas_oracle.rs` → `cycle_oracle.rs` → `nox_oracle.rs` → here),
//! and a separate file from slice A's for the same reason the dump is: a trajectory is `ngrid`
//! mix-out bisections each re-solving the 8-species Newton, so each gate's cost stays
//! proportional to what it certifies.
//!
//! WHAT IS NEW, and therefore what this gate is built to catch:
//!
//! * `quench_trajectory` — a THIRD loop around slice A's deepest nesting. Every point is a
//!   distinct mix-out root at its own `far_local`, so the trajectory rows are 5·33 = 165
//!   distinct bisections-over-a-Newton and [`the_trajectory_rests_on_enough_distinct_roots`]
//!   asserts that count rather than trusting the row total.
//! * `quench_no` — clamp-free RK4 in REAL time indexed on a β schedule. No stopping rule, so
//!   what it measures is accumulation order.
//! * `JetMixing::schedule` — a float-ATTRIBUTE exponent, which PyPy does NOT rewrite into a
//!   multiply, so the Rust must reach libm `pow`. The `sched/` keys answer that in isolation.
//! * `JetMixing::tau_q` / `Unmixedness::c` — `math.sqrt`, the sqrt instruction, NOT `powp`.
//!   The inverse of phase 2's trap, and the `tauq/`/`holdeman/` keys pin both directions.
//!
//! The bars are not invented. The project ships on two interpreters, so whatever THEY disagree
//! by is a deviation it ALREADY tolerates. Measured on this dump: **58.32 %** of the 2507
//! values are bit-identical between CPython and PyPy — the same ~58–64 % band the cycle and
//! NOx oracles found, so "Rust IS PyPy" stays a stronger statement than "Python is Python".
//!
//! Regenerate the oracle with:
//!     C:\Python314\python.exe rust/oracle/dump_quench.py rust/oracle/quench_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_quench.py rust/oracle/quench_pypy.tsv

use std::collections::HashMap;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{self, equilibrium_composition, Gas};
use turbojet::nox::{
    primary_aft, quench_no, quench_trajectory, thermal_no, JetMixing, QuenchOpts, QuenchPoint,
    Unmixedness, ZonedNoxOpts,
};

const ORACLE_CPYTHON: &str = include_str!("../oracle/quench_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/quench_pypy.tsv");

/// Python's `repr(float)` for every value this dump keys on.
///
/// `nox_oracle.rs` gets away with "append a bare `.0`" because every grid value there sits in
/// `[1e-4, 1e16)`. **This slice's τ_q sweep goes two decades below that** — `repr(1e-05)` is
/// `'1e-05'` in Python and `0.00001` in Rust — so the rule has to be the real one.
///
/// CPython formats a float as shortest-round-trip digits plus a decimal-point position
/// `decpt`, and switches to exponential when `decpt <= -4 || decpt > 16`; the exponent then
/// carries a sign and AT LEAST two digits. Rust's `Display` never uses exponential and its
/// `LowerExp` never pads, so both halves need doing by hand.
///
/// A wrong answer here cannot pass silently — a mis-keyed value lands in `missing` and the
/// gate names it — but [`py_repr_matches_cpython`] pins the cases anyway, because "the gate
/// caught it" is a worse place to learn this than "the unit test did".
fn py_repr(v: f64) -> String {
    if v == 0.0 {
        return if v.is_sign_negative() { "-0.0".into() } else { "0.0".into() };
    }
    let sci = format!("{v:e}"); // shortest round-trip mantissa + "e" + exponent
    let (mantissa, exp) = sci.split_once('e').expect("LowerExp always emits an 'e'");
    let exp: i32 = exp.parse().expect("integer exponent");
    let decpt = exp + 1;
    if decpt <= -4 || decpt > 16 {
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{mantissa}e{sign}{:02}", exp.abs())
    } else {
        let s = format!("{v}");
        if s.contains('.') {
            s
        } else {
            format!("{s}.0")
        }
    }
}

/// The formatting rule above, pinned against values CPython's `repr` was actually asked for.
///
/// Every expectation is a literal transcription of what `repr()` returns, including the two
/// that broke the first run of this gate (`1e-05`, `3e-05`) and the neighbours on the other
/// side of the `decpt <= -4` switch, which is exactly where an off-by-one would hide.
#[test]
fn py_repr_matches_cpython() {
    for (v, want) in [
        (1e-5, "1e-05"),
        (3e-5, "3e-05"),
        (1e-4, "0.0001"),   // decpt = -3: still FIXED, the boundary the switch turns on
        (3e-4, "0.0003"),
        (1e-3, "0.001"),
        (3e-3, "0.003"),
        (1e-2, "0.01"),
        (0.0, "0.0"),
        (0.7, "0.7"),
        (0.0625, "0.0625"),
        (1.0, "1.0"),
        (2.5, "2.5"),
        (16.0, "16.0"),
        (128.0, "128.0"),
        (0.3333333333333333, "0.3333333333333333"),
        (7.716049382716049, "7.716049382716049"),
    ] {
        assert_eq!(py_repr(v), want, "py_repr({v}) must match CPython's repr");
    }
}

fn load_oracle(text: &str) -> HashMap<String, f64> {
    let mut m = HashMap::new();
    for line in text.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let mut it = line.split('\t');
        let key = it.next().expect("key");
        let bits: u64 = it.next().expect("bits").parse().expect("u64 bits");
        m.insert(key.to_string(), f64::from_bits(bits));
    }
    m
}

// --- the grids, transcribed from `dump_quench.py` ----------------------------------------
const NGRID: usize = 33;
const NSTEPS: usize = 800;
const NSTEPS_DEEP: usize = 2000;
const TAU: f64 = 3e-3;
const SHAPE_N: &[f64] = &[1.0, 1.5, 2.0, 2.5, 3.0, 4.0];
const TFRAC: &[f64] =
    &[0.0, 0.125, 0.25, 0.3333333333333333, 0.5, 0.625, 0.75, 0.875, 0.9, 1.0];
const J_GRID: &[f64] = &[1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0];
const TAU_Q: &[f64] = &[1e-5, 3e-5, 1e-4, 3e-4, 1e-3, 3e-3, 1e-2];
const SHIFT_S: &[f64] = &[0.05, 0.0625, 0.08, 0.09, 0.125];
const TRAJ_CASES: &[(&str, f64)] =
    &[("dp1", 0.8), ("dp1", 1.0), ("dp1", 1.5), ("dp1", 2.0), ("dp4", 1.5)];

/// `JetMixing(J=j)` — Python's dataclass with only `J` given, i.e. every other field default.
fn jm(j: f64) -> JetMixing {
    JetMixing { j, ..JetMixing::default() }
}

/// The two design points, derived from REAL equilibrium-engine runs exactly as the oracle
/// derives them. Returns `(name, Tt3, Tt4, far, pt4)`.
fn design_points() -> Vec<(&'static str, f64, f64, f64, f64)> {
    let sub = FlightCondition::new(250.0, 50_000.0, 0.85);
    let sup = FlightCondition::new(216.7, 18_750.0, 2.0);
    let losses = Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    };
    [("dp1", &sub, 10.0, 1500.0, 50.0), ("dp4", &sup, 12.0, 1800.0, 50.0)]
        .iter()
        .map(|&(name, flight, pi_c, tt4, mdot)| {
            let r = build_turbojet(Gas::reacting_equilibrium(), pi_c, tt4, flight.p0, losses)
                .run(flight, mdot);
            (name, r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt)
        })
        .collect()
}

/// One prebuilt trajectory and everything `quench_no` needs to run against it — the oracle's
/// `trajectory()`, and the reason the whole sweep is affordable: the path takes no `tau_q`, no
/// J and no schedule, so ONE build serves every τ_q, every J, and rung 12's bulk/core pair.
struct Traj {
    comp: Vec<(&'static str, f64)>,
    t_p: f64,
    alpha: f64,
    n0: f64,
    tab: Vec<QuenchPoint>,
    ei9: f64,
    tt3: f64,
    far: f64,
    p: f64,
}

fn build_traj(dp: &(&'static str, f64, f64, f64, f64), phi_p: f64) -> Traj {
    let (_, tt3, _tt4, far, p) = *dp;
    let far_p = phi_p * gas::f_stoich();
    let alpha = far / far_p;
    let t_p = primary_aft(far_p, p, tt3, gas::hf_fuel_default());
    let comp = equilibrium_composition(far_p, t_p, p);
    let nox = thermal_no(&comp, t_p, p, TAU, far_p, 4000, 1.0);
    let ntot: f64 = comp.iter().map(|&(_, v)| v).sum();
    let n0 = alpha * nox.x_no * ntot;
    let tab = quench_trajectory(&comp, t_p, alpha, far, tt3, p, NGRID);
    Traj { comp, t_p, alpha, n0, tab, ei9: nox.ei_no, tt3, far, p }
}

impl Traj {
    /// The oracle's `dump_quench` helper — one clamp-free integration against the prebuilt path.
    fn quench(
        &self,
        tau_q: f64,
        schedule: Option<&dyn Fn(f64) -> f64>,
        super_eq_o: bool,
        nsteps: usize,
    ) -> turbojet::nox::QuenchResult {
        quench_no(
            &self.comp,
            self.t_p,
            self.alpha,
            self.far,
            self.tt3,
            self.p,
            self.n0,
            tau_q,
            QuenchOpts { nsteps, ngrid: NGRID, tab: Some(&self.tab), schedule, super_eq_o },
        )
    }
}

/// Recompute every key the oracle dumped. The ORDER of the sections mirrors `dump_quench.py`
/// so the two can be read side by side.
fn rust_values() -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = Vec::new();
    let mut put = |k: String, v: f64| out.push((k, v));

    // --- SECTION 1: the mixing ALGEBRA ---------------------------------------------------
    for &n in SHAPE_N {
        let m = JetMixing { j: 16.0, shape_n: n, ..JetMixing::default() };
        for &x in TFRAC {
            put(format!("sched/{}/{}", py_repr(n), py_repr(x)), m.schedule(x));
        }
    }
    for &j in J_GRID {
        put(format!("tauq/J/{}", py_repr(j)), jm(j).tau_q());
    }
    for &h in &[0.05f64, 0.10, 0.20] {
        put(format!("tauq/H/{}", py_repr(h)), JetMixing { j: 16.0, h, ..JetMixing::default() }.tau_q());
    }
    for &c_e in &[0.10f64, 0.15, 0.25] {
        put(format!("tauq/Ce/{}", py_repr(c_e)), JetMixing { j: 16.0, c_e, ..JetMixing::default() }.tau_q());
    }
    for &u_c in &[50.0f64, 75.0, 120.0] {
        put(format!("tauq/Uc/{}", py_repr(u_c)), JetMixing { j: 16.0, u_c, ..JetMixing::default() }.tau_q());
    }
    for &s in &[0.0625f64, 0.125] {
        let um = Unmixedness { s, ..Unmixedness::default() };
        for &j in J_GRID {
            let c = um.c(&jm(j));
            let tag = format!("holdeman/{}/{}", py_repr(s), py_repr(j));
            put(format!("{tag}/C"), c);
            put(format!("{tag}/u"), um.u(c));
            put(format!("{tag}/w"), um.core_fraction(c));
            put(format!("{tag}/tcore"), um.core_dwell(c));
        }
    }
    for &(k_u, w_max) in &[(0.0f64, 0.7f64), (2.5, 0.2), (5.0, 0.7)] {
        let um = Unmixedness { k_u, w_max, ..Unmixedness::default() };
        for &j in &[1.0f64, 16.0, 128.0] {
            put(
                format!("holdeman/ku{}w{}/{}/w", py_repr(k_u), py_repr(w_max), py_repr(j)),
                um.core_fraction(um.c(&jm(j))),
            );
        }
    }

    // --- SECTION 2: the design points ----------------------------------------------------
    let dps = design_points();
    for &(name, tt3, tt4, far, pt4) in &dps {
        put(format!("dp/{name}/Tt3"), tt3);
        put(format!("dp/{name}/Tt4"), tt4);
        put(format!("dp/{name}/far"), far);
        put(format!("dp/{name}/pt4"), pt4);
    }
    let find = |n: &str| *dps.iter().find(|d| d.0 == n).expect("design point");

    // --- SECTION 3: the TRAJECTORIES -----------------------------------------------------
    let trajs: Vec<((&str, f64), Traj)> =
        TRAJ_CASES.iter().map(|&(dp, phi)| ((dp, phi), build_traj(&find(dp), phi))).collect();
    for ((dp, phi_p), t) in &trajs {
        let tag = format!("traj/{dp}/{}", py_repr(*phi_p));
        put(format!("{tag}/T_p"), t.t_p);
        put(format!("{tag}/alpha"), t.alpha);
        put(format!("{tag}/n0"), t.n0);
        put(format!("{tag}/ei9"), t.ei9);
        for (i, row) in t.tab.iter().enumerate() {
            put(format!("{tag}/{i}/a"), row.a);
            put(format!("{tag}/{i}/T"), row.t);
            put(format!("{tag}/{i}/cO"), row.c_o);
            put(format!("{tag}/{i}/cN2"), row.c_n2);
            put(format!("{tag}/{i}/cH"), row.c_h);
            put(format!("{tag}/{i}/cNOe"), row.c_noe);
            put(format!("{tag}/{i}/ntot_local"), row.ntot_local);
            put(format!("{tag}/{i}/V"), row.v);
        }
        // Python's `max(range(n), key=...)` keeps the FIRST maximum on ties, and so does this.
        let mut best = 0usize;
        for (i, row) in t.tab.iter().enumerate() {
            if row.t > t.tab[best].t {
                best = i;
            }
        }
        put(format!("{tag}/argmax_i"), best as f64);
        put(format!("{tag}/T_peak"), t.tab[best].t);
        put(format!("{tag}/T_end"), t.tab[t.tab.len() - 1].t);
    }
    let traj_of = |dp: &str, phi: f64| {
        &trajs.iter().find(|((d, p), _)| *d == dp && *p == phi).expect("trajectory").1
    };

    // --- SECTION 4: RUNG 10 — the τ_q sweep ----------------------------------------------
    let dump = |out: &mut Vec<(String, f64)>, tag: String, q: turbojet::nox::QuenchResult| {
        out.push((format!("{tag}/ei"), q.ei));
        out.push((format!("{tag}/x_no_mix"), q.x_no_mix));
        out.push((format!("{tag}/n_no"), q.n_no));
        out.push((format!("{tag}/T_peak"), q.t_peak));
        out.push((format!("{tag}/max_a"), q.max_a));
    };
    for &(dp, phi_p) in TRAJ_CASES {
        let t = traj_of(dp, phi_p);
        for &tau_q in TAU_Q {
            let tag = format!("r10/{dp}/{}/{}", py_repr(phi_p), py_repr(tau_q));
            dump(&mut out, tag, t.quench(tau_q, None, false, NSTEPS));
        }
    }
    let t15 = traj_of("dp1", 1.5);
    for &tau_q in &[1e-4f64, 1e-3, 3e-3] {
        let tag = format!("r10deep/dp1/1.5/{}", py_repr(tau_q));
        dump(&mut out, tag, t15.quench(tau_q, None, false, NSTEPS_DEEP));
    }

    // --- SECTION 5: RUNG 11 — the J sweep ------------------------------------------------
    for &(dp, phi_p) in &[("dp1", 1.5f64), ("dp1", 1.0)] {
        let t = traj_of(dp, phi_p);
        for &j in J_GRID {
            let m = jm(j);
            let tag = format!("r11/{dp}/{}/{}", py_repr(phi_p), py_repr(j));
            out.push((format!("{tag}/tau_q"), m.tau_q()));
            let sched = |x: f64| m.schedule(x);
            dump(&mut out, tag, t.quench(m.tau_q(), Some(&sched), false, NSTEPS));
        }
    }
    let jm1 = JetMixing { j: 16.0, shape_n: 1.0, ..JetMixing::default() };
    let s1 = |x: f64| jm1.schedule(x);
    dump(&mut out, "r11/reduce/sched1".into(), t15.quench(jm1.tau_q(), Some(&s1), false, NSTEPS));
    dump(&mut out, "r11/reduce/linear".into(), t15.quench(jm1.tau_q(), None, false, NSTEPS));
    for &n in &[1.5f64, 2.0, 3.0] {
        let m = JetMixing { j: 16.0, shape_n: n, ..JetMixing::default() };
        let sched = |x: f64| m.schedule(x);
        dump(&mut out, format!("r11/shape/{}", py_repr(n)),
             t15.quench(m.tau_q(), Some(&sched), false, NSTEPS));
    }

    // --- SECTION 6: RUNG 12 — the two-stream split, and WHERE its minimum sits ------------
    for &s in &[0.0625f64] {
        let um = Unmixedness { s, ..Unmixedness::default() };
        let (mut best_j, mut best_ei) = (0.0f64, f64::INFINITY);
        for &j in J_GRID {
            let m = jm(j);
            let c = um.c(&m);
            let w = um.core_fraction(c);
            let tag = format!("r12/{}/{}", py_repr(s), py_repr(j));
            out.push((format!("{tag}/C"), c));
            out.push((format!("{tag}/w"), w));
            let sched = |x: f64| m.schedule(x);
            let qb = t15.quench(m.tau_q(), Some(&sched), false, NSTEPS);
            let qc = t15.quench(um.core_dwell(c), Some(&sched), false, NSTEPS);
            dump(&mut out, format!("{tag}/bulk"), qb);
            dump(&mut out, format!("{tag}/core"), qc);
            let ei = (1.0 - w) * qb.ei + w * qc.ei;
            out.push((format!("{tag}/ei_unmixed"), ei));
            if ei < best_ei {
                best_j = j;
                best_ei = ei;
            }
        }
        out.push((format!("r12/{}/argmin_J", py_repr(s)), best_j));
        out.push((format!("r12/{}/min_ei", py_repr(s)), best_ei));
    }
    for &s in SHIFT_S {
        let um = Unmixedness { s, ..Unmixedness::default() };
        let j_opt = (um.c_opt * JetMixing::default().h / um.s).powi(2);
        out.push((format!("r12shift/{}/J_opt", py_repr(s)), j_opt));
        let js = [j_opt / 4.0, j_opt / 2.0, j_opt, 2.0 * j_opt, 4.0 * j_opt];
        let mut eis = Vec::new();
        for (k, &j) in js.iter().enumerate() {
            let m = jm(j);
            let c = um.c(&m);
            let w = um.core_fraction(c);
            let tag = format!("r12shift/{}/{k}", py_repr(s));
            out.push((format!("{tag}/J"), j));
            out.push((format!("{tag}/C"), c));
            out.push((format!("{tag}/w"), w));
            let sched = |x: f64| m.schedule(x);
            let qb = t15.quench(m.tau_q(), Some(&sched), false, NSTEPS);
            let qc = t15.quench(um.core_dwell(c), Some(&sched), false, NSTEPS);
            dump(&mut out, format!("{tag}/bulk"), qb);
            dump(&mut out, format!("{tag}/core"), qc);
            let ei = (1.0 - w) * qb.ei + w * qc.ei;
            out.push((format!("{tag}/ei_unmixed"), ei));
            eis.push(ei);
        }
        let mut imin = 0usize;
        for (i, &e) in eis.iter().enumerate() {
            if e < eis[imin] {
                imin = i;
            }
        }
        out.push((format!("r12shift/{}/argmin_i", py_repr(s)), imin as f64));
        let m_opt = jm(j_opt);
        let sched = |x: f64| m_opt.schedule(x);
        let q_m = t15.quench(m_opt.tau_q(), Some(&sched), false, NSTEPS);
        let q_c = t15.quench(um.tau_res, Some(&sched), false, NSTEPS);
        dump(&mut out, format!("r12shift/{}/pin_Em", py_repr(s)), q_m);
        dump(&mut out, format!("r12shift/{}/pin_Ec", py_repr(s)), q_c);
        out.push((format!("r12shift/{}/tau_mean_opt", py_repr(s)), m_opt.tau_q()));
        out.push((format!("r12shift/{}/pin_lhs", py_repr(s)), um.k_u * (q_c.ei - q_m.ei)));
        out.push((format!("r12shift/{}/pin_rhs", py_repr(s)), q_m.ei));
    }
    let um0 = Unmixedness { k_u: 0.0, ..Unmixedness::default() };
    let jm16 = jm(16.0);
    let c0 = um0.c(&jm16);
    let s16 = |x: f64| jm16.schedule(x);
    out.push(("r12/ku0/C".into(), c0));
    out.push(("r12/ku0/w".into(), um0.core_fraction(c0)));
    let qb0 = t15.quench(jm16.tau_q(), Some(&s16), false, NSTEPS);
    let qc0 = t15.quench(um0.core_dwell(c0), Some(&s16), false, NSTEPS);
    dump(&mut out, "r12/ku0/bulk".into(), qb0);
    dump(&mut out, "r12/ku0/core".into(), qc0);
    let w0 = um0.core_fraction(c0);
    out.push(("r12/ku0/ei_unmixed".into(), (1.0 - w0) * qb0.ei + w0 * qc0.ei));

    // --- SECTION 7: RUNG 20 — the super-eq O lift THROUGH the quench ----------------------
    for &(dp, phi_p) in &[("dp1", 1.0f64), ("dp1", 1.5), ("dp4", 1.5)] {
        let t = traj_of(dp, phi_p);
        for &tau_q in &[1e-4f64, 1e-3, 3e-3] {
            let tag = format!("r20/{dp}/{}/{}", py_repr(phi_p), py_repr(tau_q));
            dump(&mut out, tag, t.quench(tau_q, None, true, NSTEPS));
        }
    }
    dump(&mut out, "r20/jet/J16".into(), t15.quench(jm16.tau_q(), Some(&s16), true, NSTEPS));
    let um = Unmixedness::default();
    dump(&mut out, "r20/core/S0625".into(),
         t15.quench(um.core_dwell(um.c(&jm16)), Some(&s16), true, NSTEPS));

    // --- SECTION 8: the PUBLIC entry point ------------------------------------------------
    let g = Gas::reacting_equilibrium();
    let (_, tt3_1, tt4_1, far_1, pt4_1) = find("dp1");
    let base = ZonedNoxOpts {
        tau: TAU,
        quench_ngrid: NGRID,
        quench_nsteps: NSTEPS,
        ..ZonedNoxOpts::default()
    };
    let cases: [(&str, f64, ZonedNoxOpts); 8] = [
        ("r10/tq1e-3", 1.5, ZonedNoxOpts { tau_q: Some(1e-3), ..base }),
        ("r10/tq3e-3", 1.5, ZonedNoxOpts { tau_q: Some(3e-3), ..base }),
        ("r10/lean", 0.9, ZonedNoxOpts { tau_q: Some(1e-3), ..base }),
        ("r11/J16", 1.5, ZonedNoxOpts { mixing: Some(jm(16.0)), ..base }),
        ("r11/J64", 1.5, ZonedNoxOpts { mixing: Some(jm(64.0)), ..base }),
        ("r12/J16", 1.5, ZonedNoxOpts {
            mixing: Some(jm(16.0)), unmixedness: Some(Unmixedness::default()), ..base }),
        ("r12/J128", 1.5, ZonedNoxOpts {
            mixing: Some(jm(128.0)), unmixedness: Some(Unmixedness::default()), ..base }),
        ("r20/J16", 1.5, ZonedNoxOpts {
            mixing: Some(jm(16.0)), unmixedness: Some(Unmixedness::default()),
            super_eq_o: true, ..base }),
    ];
    for (tag, phi, opts) in cases {
        let z = g.zoned_nox(far_1, tt3_1, tt4_1, pt4_1, phi, opts);
        out.push((format!("zn/{tag}/ei_no"), z.ei_no()));
        out.push((format!("zn/{tag}/x_no_mix"), z.x_no_mix));
        out.push((format!("zn/{tag}/T_primary"), z.t_primary));
        out.push((format!("zn/{tag}/T_mix"), z.t_mix));
        out.push((format!("zn/{tag}/tau_q"), z.tau_q.expect("finite quench")));
        out.push((format!("zn/{tag}/ei_quenched"), z.ei_no_quenched.expect("finite quench")));
        out.push((format!("zn/{tag}/x_quenched"), z.x_no_quenched.expect("finite quench")));
        out.push((format!("zn/{tag}/T_peak"), z.t_peak.expect("finite quench")));
        out.push((format!("zn/{tag}/max_a"), z.max_a_quench.expect("finite quench")));
        if let Some(ei_unmixed) = z.ei_no_unmixed {
            out.push((format!("zn/{tag}/C_holdeman"), z.c_holdeman.expect("rung 12")));
            out.push((format!("zn/{tag}/w_core"), z.w_core.expect("rung 12")));
            out.push((format!("zn/{tag}/ei_core"), z.ei_no_core.expect("rung 12")));
            out.push((format!("zn/{tag}/ei_unmixed"), ei_unmixed));
        }
    }
    out
}

/// Which class a key belongs to — the same split `nox_oracle.rs` uses, extended for the two
/// things this slice adds: the pure mixing algebra (no solver at all) and the LOCATION keys,
/// which are small integers and must therefore be EXACT in every arm, CPython included.
fn quant_of(key: &str) -> &'static str {
    let head = key.split('/').next().unwrap_or("");
    let last = key.rsplit('/').next().unwrap_or("");
    if last == "argmax_i" || last == "argmin_i" || last == "argmin_J" || last == "J_opt" {
        return "shape_location";
    }
    match head {
        // Closed forms: no solver, no integrator, no composition.
        "sched" | "tauq" | "holdeman" => "mixing_algebra",
        "dp" => "design_point",
        _ => match last {
            // A trajectory point's T IS a mix-out bisection root; `a` and `V` ride on it.
            "T" | "T_p" | "T_peak" | "T_end" | "T_primary" | "T_mix" => "bisection_root",
            "cNOe" | "ntot_local" => "equilibrium",
            "C" | "w" | "tcore" | "tau_q" | "tau_mean_opt" | "J" => "mixing_algebra",
            _ => "kinetic",
        },
    }
}

/// The bar for each class — CPYTHON arm only; the PyPy arm is held to bit-equality.
///
/// Every number is a MEASUREMENT of the CPython↔PyPy spread on this dump (the deviation the
/// project already tolerates, since it ships on both), with headroom — not a guess. Since the
/// Rust reproduces PyPy exactly, the CPython arm's own "worst rel" column IS that spread:
///
/// ```text
///   mixing_algebra  0.00e0   <- EXACTLY equal, all 272 keys: closed forms, no iterate
///   bisection_root  0.00e0   <- ditto, all 352: a bisection lands on a bracket MIDPOINT
///   shape_location  0.00e0   <- all 16 locations identical (the VALUES at them are not)
///   design_point    1.36e-15
///   equilibrium     2.32e-15
///   kinetic         3.77e-15
/// ```
///
/// The SPLIT is the finding and it reproduces slice A's exactly: the three classes that are
/// EXACTLY equal across interpreters are the three with no accumulated iterate. Note the
/// trajectory temperatures land in `bisection_root` and are bit-identical across interpreters
/// on all 352 — 33 points × 5 cases of bisection-over-a-Newton, agreeing to the last bit on
/// both, which is a stronger statement than slice A could make with 22 roots.
///
/// **`design_point` was initially set to 1e-15 here and that was a GUESS, not a measurement —
/// it failed on one key at 1.36e-15.** Slice A puts the same keys at 1e-12; this now does too.
/// The failure is recorded rather than quietly overwritten because an invented bar is exactly
/// what § 4.2 of the port plan says let a real defect ride for a whole phase.
///
/// `shape_location` gets ZERO, and that is the point of dumping locations at all: they are
/// small integers read off a deliberately coarse grid, so ANY movement is a real relocation
/// rather than last-bit noise. Slice A measured the two interpreters disagreeing on an
/// extremum's VALUE while agreeing on its POSITION exactly; this bar makes that enforceable.
fn bar_for(quant: &str) -> f64 {
    match quant {
        "shape_location" => 0.0,
        "mixing_algebra" | "bisection_root" => 1.0e-15,
        _ => 1.0e-12,
    }
}

fn compare_against(oracle_text: &str, label: &str, require_bit_exact: bool) {
    let oracle = load_oracle(oracle_text);
    let ours = rust_values();
    println!("\n=== Rust vs {label} ===");

    assert_eq!(
        ours.len(),
        oracle.len(),
        "key COUNT differs: rust {} vs oracle {} — the dump and the test have drifted apart, \
         so a missing key would otherwise read as a pass",
        ours.len(),
        oracle.len()
    );

    let mut missing: Vec<&str> = Vec::new();
    let mut per: HashMap<&str, (usize, usize, f64, String)> = HashMap::new();
    let mut failures: Vec<String> = Vec::new();

    for (key, got) in &ours {
        let Some(&want) = oracle.get(key.as_str()) else {
            missing.push(key);
            continue;
        };
        let q = quant_of(key);
        let e = per.entry(q).or_insert((0, 0, 0.0, String::new()));
        e.0 += 1;
        if got.to_bits() == want.to_bits() {
            e.1 += 1;
            continue;
        }
        let scale = got.abs().max(want.abs());
        let rel = if scale > 0.0 { (got - want).abs() / scale } else { (got - want).abs() };
        if rel > e.2 {
            e.2 = rel;
            e.3 = key.clone();
        }
        if rel > bar_for(q) {
            failures.push(format!(
                "  {key:<52} rust {got:.17e}  oracle {want:.17e}  rel {rel:.2e}"
            ));
        }
    }

    let mut rows: Vec<_> = per.iter().collect();
    rows.sort_by(|a, b| b.1 .2.partial_cmp(&a.1 .2).unwrap());
    println!("\n{:<16} {:>6} {:>11} {:>12} {:>12}", "quantity", "keys", "bit-exact", "worst rel", "bar");
    println!("{}", "-".repeat(62));
    for (q, (n, exact, worst, _)) in &rows {
        println!("{:<16} {:>6} {:>11} {:>12.2e} {:>12.0e}", q, n, exact, worst, bar_for(q));
    }
    let total: usize = rows.iter().map(|r| r.1 .0).sum();
    let exact: usize = rows.iter().map(|r| r.1 .1).sum();
    println!("\n{exact} / {total} bit-identical to {label} ({:.2}%)",
             100.0 * exact as f64 / total as f64);
    for (q, (_, _, worst, key)) in &rows {
        if *worst > 0.0 {
            println!("  worst {q:<16} {worst:.2e}  at {key}");
        }
    }

    assert!(missing.is_empty(), "keys computed by Rust but absent from the oracle: {missing:?}");
    assert!(failures.is_empty(),
            "{} value(s) outside the measured bar:\n{}", failures.len(), failures.join("\n"));
    if require_bit_exact {
        let drifted: Vec<&String> =
            rows.iter().filter(|(_, (_, _, w, _))| *w > 0.0).map(|(_, (_, _, _, k))| k).collect();
        assert_eq!(exact, total,
                   "phase 3B measured {total}/{total} BIT-IDENTICAL to {label}; this run got \
                    {exact}. A drop is either a real arithmetic regression or a toolchain/libm \
                    change — find out WHICH before loosening this to a tolerance. Phase 1 ran \
                    its own arm at 98.89 % and the missing 1.11 % was a transcription bug in a \
                    polynomial's power spelling. First drifted keys: {drifted:?}");
    }
}

#[test]
fn quench_matches_the_cpython_oracle() {
    compare_against(ORACLE_CPYTHON, "CPython 3.14.3", false);
}

/// The same comparison against the interpreter the test gate actually runs on — and here the
/// bar is BIT-EQUALITY, not a tolerance.
///
/// Not redundant with the CPython arm; it is the DISCRIMINATOR. Either Rust has its own drift
/// that coincidentally matches PyPy's, or Rust and PyPy are computing the same function. The
/// CPython arm's ~42 % disagreement is what makes the coincidence implausible.
#[test]
fn quench_matches_the_pypy_oracle_to_the_bit() {
    compare_against(ORACLE_PYPY, "PyPy 3.11.15", true);
}

/// The trajectory claim, sized by DISTINCT ROOTS rather than by row count.
///
/// "The trajectory reproduces 1320/1320" would be worthless if the 33 points per case were one
/// root repeated. Every point sits at its own `far_local = far_ov/a(β)`, so every point is a
/// separate mix-out bisection over the 8-species Newton — and this asserts that count so it
/// cannot silently collapse. It also asserts the trajectory's DIRECTION, which is what a
/// wrong-way `a(β)` would break while leaving the root count intact.
#[test]
fn the_trajectory_rests_on_enough_distinct_roots() {
    let dps = design_points();
    let find = |n: &str| *dps.iter().find(|d| d.0 == n).expect("design point");
    let mut roots: Vec<u64> = Vec::new();
    for &(dp, phi) in TRAJ_CASES {
        let t = build_traj(&find(dp), phi);
        // `a(β)` runs from α (all primary) to exactly 1 (all the air), monotonically.
        assert_eq!(t.tab[0].a.to_bits(), t.alpha.to_bits(),
                   "{dp}/{phi}: the trajectory must START at the primary split α");
        assert!((t.tab[t.tab.len() - 1].a - 1.0).abs() < 1e-12,
                "{dp}/{phi}: the trajectory must END on the full total-air basis a=1");
        for w in t.tab.windows(2) {
            assert!(w[1].a > w[0].a, "{dp}/{phi}: a(β) must increase along the path");
        }
        roots.extend(t.tab.iter().map(|r| r.t.to_bits()));
    }
    let n_rows = roots.len();
    roots.sort_unstable();
    roots.dedup();
    println!("distinct mix-out roots on the trajectories: {} of {} points", roots.len(), n_rows);
    assert_eq!(n_rows, TRAJ_CASES.len() * NGRID);
    assert!(roots.len() >= 160,
            "the trajectory sweep collapsed to {} distinct mix-out roots out of {} points — \
             each point should sit at its own far_local, so the solver claim is thinner than \
             it reads", roots.len(), n_rows);
}

/// RUNG 10's smoking gun, as a LOCATION rather than a value.
///
/// A RICH primary's local mixture sweeps UP through stoichiometric as the dilution air comes
/// in, so its temperature PEAKS several β steps along the path — not at the start. A LEAN one
/// starts at or above the peak and only cools, so its maximum is index 0 exactly. A port that
/// runs `a(β)` backwards, or interpolates on the wrong axis, breaks this while leaving every
/// individual temperature a plausible number.
#[test]
fn the_temperature_peak_sits_where_the_mixture_crosses_stoichiometric() {
    let dps = design_points();
    let find = |n: &str| *dps.iter().find(|d| d.0 == n).expect("design point");
    for &(dp, phi, want_interior) in &[("dp1", 0.8, false), ("dp1", 1.0, false),
                                       ("dp1", 1.5, true), ("dp1", 2.0, true),
                                       ("dp4", 1.5, true)] {
        let t = build_traj(&find(dp), phi);
        let mut best = 0usize;
        for (i, row) in t.tab.iter().enumerate() {
            if row.t > t.tab[best].t {
                best = i;
            }
        }
        if want_interior {
            assert!(best > 0 && best < NGRID - 1,
                    "{dp}/φ={phi}: a RICH primary must peak INSIDE the path (the stoich \
                     crossing), got index {best} of {NGRID}");
            assert!(t.tab[best].t > t.t_p + 100.0,
                    "{dp}/φ={phi}: the peak {:.1} K must rise well above the primary AFT \
                     {:.1} K — that rise IS the re-making", t.tab[best].t, t.t_p);
        } else {
            assert_eq!(best, 0,
                       "{dp}/φ={phi}: a LEAN/stoich primary only COOLS, so the maximum must be \
                        the first point; got index {best}");
            for w in t.tab.windows(2) {
                assert!(w[1].t <= w[0].t + 1e-9,
                        "{dp}/φ={phi}: a lean trajectory must not rise along β");
            }
        }
    }
}
