//! PHASE 5N GATE — every rung-55/56 value the Python oracle dumped, recomputed in Rust.
//!
//! # What this gate is built to catch
//!
//! * **A MARCH THAT IS RIGHT ONLY WHERE IT IS READ.** Every per-spool aggregate in rungs 55/56 is
//!   an argmin or a face read, so a stack whose row 3 is wrong at `K = 8` can still produce the
//!   right `binds`, the right `m_c_worst` and the right `amplification`. Both argmin currencies
//!   (`m_c`, `m_i`) are therefore dumped for **every row of every half-row**, not for the winning
//!   row and not on a subgrid.
//!
//! * **THE ARGMIN TIE-BREAK, WHICH THE VALUES CANNOT SHOW.** § 5.10 (iv) measured 13 half-rows
//!   where the per-row margins agree to 1–2 ULP and several rows are BIT-IDENTICAL. There the
//!   argmin is a tie-break, not physics: `min(range(n), key=…)` returns the FIRST minimum, and a
//!   `fold` with `<=` would return the last **while every value key still passes**. `binds` and
//!   `inc_worst` are dumped as indices on all 1 280 half-rows, and the rule itself is pinned on a
//!   constructed tie in `slice_n_smoke.rs`.
//!
//! * **A CENSUS THAT IS COMPARED RATHER THAN RESTATED.** § 5.10 (iii)'s `3 204 / 521 649` and
//!   (vi)'s `120 / 4 360` were measured on the PROBES' grids — `probe_n1` sweeps `K` in
//!   {2, 4, 8}, three values, with no `cap_profile` axis, i.e. 240 cells and 120 stacks against
//!   this dump's 640 and 320. Copying them here would be *slice L step 4*'s copied bar. So Python
//!   COUNTS on this grid and [`take_census`] is compared against those counts, key by key.
//!
//! * **THE ONE CAUGHT SCOPE, BY REASON RATHER THAN BY COUNT.** `stage_incidence_schedule`'s
//!   `except AssertionError: break` swallows WHICH arm of `try_solve_n` fired, so reproducing the
//!   firings does not gate the arms. The dump classifies each firing at the raise and carries the
//!   `(m, tau_c, eta_live)` triple of the first CLAMPED-ROOT one, which is what lets
//!   [`the_clamped_root_arm_is_reached_from_the_dump_grid`] re-enter that arm directly — the
//!   deferral `slice_n_smoke.rs::slice_n_deferrals_so_far` item 2b, discharged.
//!
//! * **A PROFILE THAT LEAKS INTO THE MARCH.** `cap_profile` is read ONLY by
//!   `StageStack::capacities`, so rung 55's whole reading set — the matched point included — must
//!   be bit-identical across it. The dump carries `stage_margin`/`work_gap` on the `derived`
//!   profile alone and the invariance is gated here instead, which is a stronger statement than
//!   dumping the duplicate would have been.
//!
//! Regenerate the oracle with:
//!     .venv\Scripts\python.exe rust/oracle/dump_slice_n.py fast  rust/oracle/slice_n_pypy.tsv
//!     .venv\Scripts\python.exe rust/oracle/dump_slice_n.py equil rust/oracle/slice_n_eq_pypy.tsv
//!     py -3                    rust/oracle/dump_slice_n.py lean  rust/oracle/slice_n_cpython.tsv

use std::collections::HashMap;

use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stage::{take_census, CapProfile, Split, StackCensus, StageStackCore,
                      StageStackCoreSpec};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_n_pypy.tsv");
const ORACLE_EQ: &str = include_str!("../oracle/slice_n_eq_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_n_cpython.tsv");

/// MEASURED over the CPython arm's 41 560 keys, never guessed — slice M's arm was drafted as a
/// BIT bar and a third of its dump refuted it. See `slice_n_matches_cpython` for the tiers.
const REL_BAR: f64 = 1e-7;
const ABS_BAR: f64 = 1e-10;
/// The MEASURED worst (2.889e-9, a `d_n` finite difference) with ~40 % headroom, not
/// 17x. A pin whose job is *a regression cannot hide under `REL_BAR`* has to be tight
/// enough to do it — slice M's was 5e-8 against a measured 2.38e-8, i.e. 2x.
const WORST_BAR: f64 = 4e-9;

/// A projection key — `.../r<k>/<field>`. The CPython arm does not carry them.
fn is_per_row(key: &str) -> bool {
    key.split('/').any(|s| s.len() > 1 && s.starts_with('r')
                       && s[1..].chars().all(|c| c.is_ascii_digit()))
}

/// The key families that are DISCRETE — an index, a count, a flag, or a branch verdict. Held to
/// BIT equality on every interpreter, because they are not the output of an iteration: a
/// difference here is a different BRANCH, never drift.
///
/// **`n` IS THE SHAFT SPEED EVERYWHERE IN THIS DUMP AND IS NOT IN THIS LIST** — slice M's
/// tail-only classifier collided a step COUNT with a speed and held four speeds to a bit bar.
/// The counts here are spelled `n_rows`/`n_caps` for exactly that reason.
fn is_discrete(key: &str) -> bool {
    let tail = key.rsplit('/').next().unwrap_or(key);
    matches!(tail, "binds" | "inc_worst" | "worst" | "n_rows" | "n_caps" | "chokes" | "index"
                 | "rear_binds" | "front_binds" | "reached" | "K" | "has_vsv_stages"
                 | "vsv_stages")
}

/// An ARGMIN-DERIVED key — an index, or a flag computed from one.
fn is_argmin(key: &str) -> bool {
    let tail = key.rsplit('/').next().unwrap_or(key);
    matches!(tail, "binds" | "inc_worst" | "worst" | "rear_binds" | "front_binds")
}

/// **AT THE DESIGN THROTTLE THE ARGMIN IS A COIN FLIP, AND THIS IS THE ONE TIER SLICE M's
/// CLASSIFIER WOULD HAVE GOT WRONG HERE.** Slice M's rule was *discrete → bits on every
/// interpreter, because a difference is a BRANCH and not drift*. Measured on this dump, **520
/// discrete keys differ between CPython and PyPy and every single one is at `Tt4 = 1500`** —
/// where every `phi_k = 1`, the per-row margins collapse to within 1–2 ULP, and which row wins
/// is decided by the last bit of the march's own accumulation. That is drift wearing a branch's
/// clothes, and it is § 5.10 (iv)'s *degenerate argmin, not a third physical class* showing up
/// across interpreters instead of across rows.
///
/// So the argmin keys are held to bits OFF design — **4 680 of them, ZERO differing** — and
/// counted at design (1 560 keys, 520 flips). The other 3 681 discrete keys — `reached`,
/// `chokes`, `n_rows`, `n_caps`, `index`, `K`, `vsv_stages` — are bit-exact everywhere; none is
/// an argmin and none differs.
///
/// **THE FIRST DRAFT OF THOSE TWO COUNTS WAS 7 481 / 2 280 AND BOTH WERE WRONG**, derived over
/// ALL discrete keys rather than the argmin subset. The value tiers passed on the first run and
/// only the counts failed — *a count derived from a superset is a guess*, which is this port's
/// own `guessed census bars` lesson arriving on a population rather than on a magnitude.
/// **The segment match is verified, not assumed.** It reads like a substring rule; it is not —
/// `1500` occupies exactly ONE position in each family that has it (`thr`/`mar`/`gap` 6, `walk` 7,
/// `sched` 5) and that position's only possible values are the four throttles. No shape name, no
/// `K`-token and no field name can equal it. `shift` carries no throttle segment at all and no
/// argmin key, so nothing escapes the classification.
fn at_design(key: &str) -> bool {
    key.split('/').any(|s| s == "1500")
}

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
/// § 5.10's pre-registered capacity, and **its stated provenance is wrong.** `probe_n1` calls it
/// *"rung 54's disclosed capacity constant, as the rung-56 tests carry it"*; those tests carry
/// `CAP = 0.90` (`tests/test_rung56.py:48`) and `0.60` appears nowhere in rungs 53–56 as a
/// capacity. It is kept because the (iv) census was MEASURED at 0.60 — moving it would re-point
/// every one of those bars at cells nobody has looked at (§ 5.7 (e)).
const CAP: f64 = 0.60;

const K_GRID: [usize; 4] = [2, 4, 8, 16];
const SPLITS: [(&str, Split); 2] = [("dT", Split::DT), ("tau", Split::Tau)];
const PROFILES: [(&str, CapProfile); 2] =
    [("derived", CapProfile::Derived), ("uniform", CapProfile::Uniform)];
const THROTTLE_FAST: [f64; 4] = [1500.0, 1200.0, 1000.0, 800.0];
const K_EQ: [usize; 2] = [2, 8];
const THROTTLE_EQ: [f64; 2] = [1500.0, 1200.0];
const SPOOLS: [(&str, Spool); 2] = [("lp", Spool::Lp), ("hp", Spool::Hp)];

fn load(text: &str) -> HashMap<&str, f64> {
    let mut out = HashMap::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let mut it = line.split('\t');
        let key = it.next().expect("key");
        out.insert(key, parse_hex(it.next().expect("hex")));
    }
    out
}

/// Python's `float.hex()`, parsed exactly — the comparison is on BITS, not a decimal round-trip.
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
    let mut v = int_part.parse::<u64>().expect("int part") as f64;
    let mut scale = 1.0f64 / 16.0;
    for ch in frac.chars() {
        v += (ch.to_digit(16).expect("hex digit") as f64) * scale;
        scale /= 16.0;
    }
    let out = v * (2.0f64).powi(exp);
    if neg { -out } else { out }
}

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    }
}

fn cpg_gas() -> Gas {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn gas_by_name(name: &str) -> Gas {
    match name {
        "cpg" => cpg_gas(),
        "tpg" => Gas::thermally_perfect(),
        "eq" => Gas::reacting_equilibrium(),
        other => panic!("unknown gas {other}"),
    }
}

fn design(gname: &str) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas_by_name(gname), PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

/// Rung 53's OWN five disclosed shapes, verbatim from `tests/test_rung53.py::SHAPES` — the set
/// `probe_n1`/`probe_n3` swept, hence the set every § 5.10 census was measured over.
fn shapes() -> Vec<(&'static str, ComponentMap, ComponentMap)> {
    let f = ComponentMap::flat();
    vec![
        ("flow_press", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                       ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
        ("press_flow", ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..f },
                       ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }),
        ("tilted",     ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f },
                       ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f }),
        ("steep",      ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f },
                       ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f }),
        ("flat_eta",   ComponentMap { sigma: 0.1, l: 0.7, ..f },
                       ComponentMap { sigma: 0.1, l: 1.0, ..f }),
    ]
}

/// The dump's `maps(..., capacity)` — the floor always, rung 54's throat only when asked.
fn armed(ml: ComponentMap, mh: ComponentMap, capacity: bool) -> (ComponentMap, ComponentMap) {
    let (a, b) = (ml.with_phi_surge(FLOOR), mh.with_phi_surge(FLOOR));
    if capacity { (a.with_capacity(CAP), b.with_capacity(CAP)) } else { (a, b) }
}

fn stacked(d: TwoSpoolEngine, ml: ComponentMap, mh: ComponentMap, k: usize, split: Split,
           cap_profile: CapProfile) -> StageStackCore {
    StageStackCore::new(StageStackCoreSpec {
        k_lp: k, k_hp: k, split, cap_profile,
        ..StageStackCoreSpec::new(d, flight(), 1.0, ml, mh)
    })
}

/// What the census cannot count for itself: how many cells and half-rows the sweep visited, and
/// the two argmin populations § 5.10 (iv) splits into.
#[derive(Default)]
struct Tally {
    cells: usize,
    half_rows: usize,
    /// `binds` position — [front, rear, interior] — indexed by profile.
    binds: [[usize; 3]; 2],
    /// `inc_worst` position, both profiles pooled (it does not see the profile).
    inc: [usize; 3],
    /// The interior readings — `(cell tag, spread of the currency that IS interior)`, so
    /// § 5.10 (iv)'s "ONE population, decided by the LAST BIT" claim is testable rather than
    /// restated. The spread must be of the currency whose argmin landed inside; the OTHER
    /// currency's spread is a whole percent there and would make the bar vacuous.
    interior: Vec<(String, f64)>,
    sched_rows: usize,
    sched_reached: usize,
    sched_reached_frontrow: usize,
    sched_frontrow: usize,
    sched_missed_at_design: usize,
}

fn place(i: usize, n: usize) -> usize {
    if i == 0 { 0 } else if i == n - 1 { 1 } else { 2 }
}

const THR_KEYS: [&str; 14] = ["vsv", "m", "n", "capacity_front", "tan_b1_crit", "binds",
                              "m_c_worst", "x_worst", "c_min_worst", "m_c_face", "x_face",
                              "amplification", "inc_worst", "m_i_worst"];
const MAR_KEYS: [&str; 11] = ["vsv", "phi_face", "n", "m", "tan_b1_crit", "worst", "m_i_worst",
                              "m_i_face", "rear_excess", "phi_front", "phi_rear"];
const GAP_KEYS: [&str; 6] = ["m", "n", "tau_lumped", "tau_marched", "gap", "gap_frac"];
const WALK_KEYS: [&str; 10] = ["binds", "m_c_worst", "m_c_face", "amplification", "inc_worst",
                               "m_i_worst", "c_min_worst", "m", "n", "vsv"];
const SHIFT_KEYS: [&str; 9] = ["n_lumped", "n_stacked", "d_n", "phi_lumped", "phi_stacked",
                               "d_phi", "pi_lumped", "pi_stacked", "d_pi"];
const WIDE_KEYS: [&str; 8] = ["phi", "n", "vsv", "m_k", "capacity", "area", "throat_loading",
                              "c_min"];
const SCHED_KEYS: [&str; 14] = ["vsv_star", "residual", "tan_b1", "tan_b1_design", "phi_stage",
                                "phi_stage_bare", "m_i", "m_i_bare", "m_i_worst", "worst", "n",
                                "n_bare", "d_n", "rear_excess"];

/// The throat/margin sweep — a transcription of `dump_slice_n.py::sweep_fast`, in its order.
///
/// The ORDER matters for one reason only and it is the census: `take_census` is a total over the
/// calls made, so a reader that visits the same cells by a different route still agrees, but one
/// that visits a different SET does not. That is the coverage half of this gate.
fn sweep_fast(
    gases: &[&str], ks: &[usize], splits: &[(&str, Split)], profiles: &[(&str, CapProfile)],
    throttles: &[f64], subgrids: bool, mut check: impl FnMut(String, f64),
) -> Tally {
    let mut t = Tally::default();
    let f = flight();
    for gname in gases {
        for (sname, ml0, mh0) in shapes() {
            let (a_l, a_h) = armed(ml0, mh0, true);
            for &k in ks {
                for &(spname, split) in splits {
                    for &(pfname, prof) in profiles {
                        let cfg = format!("{gname}/{sname}/K{k}/{spname}/{pfname}");
                        let m = stacked(design(gname), a_l, a_h, k, split, prof);
                        for &tt4 in throttles {
                            t.cells += 1;
                            let cell = format!("{cfg}/{tt4:.0}");
                            let r = m.stage_throat_margin(&f, tt4);
                            for &(spn, spool) in SPOOLS.iter() {
                                let s = r.spool(spool);
                                let p = format!("thr/{cell}/{spn}");
                                let vals = [s.vsv, s.m, s.n, s.capacity_front, s.tan_b1_crit,
                                            s.binds as f64, s.m_c_worst, s.x_worst,
                                            s.c_min_worst, s.m_c_face, s.x_face,
                                            s.amplification, s.inc_worst as f64, s.m_i_worst];
                                for (key, v) in THR_KEYS.iter().zip(vals) {
                                    check(format!("{p}/{key}"), v);
                                }
                                check(format!("{p}/chokes"), b(s.chokes));
                                check(format!("{p}/rear_binds"), b(s.rear_binds));
                                check(format!("{p}/front_binds"), b(s.front_binds));
                                check(format!("{p}/n_rows"), s.stages.len() as f64);
                                for st in &s.stages {
                                    let q = format!("{p}/r{}", st.stage);
                                    check(format!("{q}/m_c"), st.m_c);
                                    check(format!("{q}/m_i"), st.m_i);
                                }
                                if k == 8 && tt4 == 1500.0 {
                                    for st in &s.stages {
                                        let q = format!("{p}/r{}", st.stage);
                                        let w = [st.phi, st.n, st.vsv, st.m_k, st.capacity,
                                                 st.area, st.throat_loading, st.c_min];
                                        for (key, v) in WIDE_KEYS.iter().zip(w) {
                                            check(format!("{q}/{key}"), v);
                                        }
                                        check(format!("{q}/chokes"), b(st.chokes));
                                    }
                                }
                                t.half_rows += 1;
                                let n = s.stages.len();
                                t.binds[if prof == CapProfile::Derived { 0 } else { 1 }]
                                    [place(s.binds, n)] += 1;
                                t.inc[place(s.inc_worst, n)] += 1;
                                let spread = |f: fn(&turbojet::stage::StageThroatRow) -> f64| {
                                    s.stages.iter().map(f).fold(f64::MIN, f64::max)
                                        - s.stages.iter().map(f).fold(f64::MAX, f64::min)
                                };
                                if place(s.binds, n) == 2 {
                                    t.interior.push((format!("binds/{cell}/{spn}"),
                                                     spread(|x| x.m_c)));
                                }
                                if place(s.inc_worst, n) == 2 {
                                    t.interior.push((format!("inc/{cell}/{spn}"),
                                                     spread(|x| x.m_i)));
                                }
                            }

                            if prof != CapProfile::Derived {
                                continue;
                            }
                            let a = m.stage_margin(&f, tt4);
                            for &(spn, spool) in SPOOLS.iter() {
                                let s = a.spool(spool);
                                let p = format!("mar/{cell}/{spn}");
                                let vals = [s.vsv, s.phi_face, s.n, s.m, s.tan_b1_crit,
                                            s.worst as f64, s.m_i_worst, s.m_i_face,
                                            s.rear_excess, s.phi_front, s.phi_rear];
                                for (key, v) in MAR_KEYS.iter().zip(vals) {
                                    check(format!("{p}/{key}"), v);
                                }
                                for st in &s.stages {
                                    check(format!("{p}/r{}/m_phi", st.stage), st.m_phi);
                                    if tt4 == 1500.0 {
                                        check(format!("{p}/r{}/phi_surge", st.stage),
                                              st.phi_surge);
                                    }
                                }
                            }
                            let w = m.work_gap(&f, tt4);
                            for &(spn, spool) in SPOOLS.iter() {
                                let g = w.spool(spool);
                                let vals = [g.m, g.n, g.tau_lumped, g.tau_marched, g.gap,
                                            g.gap_frac];
                                for (key, v) in GAP_KEYS.iter().zip(vals) {
                                    check(format!("gap/{cell}/{spn}/{key}"), v);
                                }
                            }
                        }

                        if !(subgrids && k == 8 && prof == CapProfile::Derived) {
                            continue;
                        }
                        for &(spn, spool) in SPOOLS.iter() {
                            for (i, row) in m.throat_walk(&f, throttles, spool).iter().enumerate()
                            {
                                let p = format!("walk/{cfg}/{spn}/{:.0}", row.tt4);
                                check(format!("{p}/index"), i as f64);
                                let vals = [row.binds as f64, row.m_c_worst, row.m_c_face,
                                            row.amplification, row.inc_worst as f64,
                                            row.m_i_worst, row.c_min_worst, row.m, row.n,
                                            row.vsv];
                                for (key, v) in WALK_KEYS.iter().zip(vals) {
                                    check(format!("{p}/{key}"), v);
                                }
                                check(format!("{p}/chokes"), b(row.chokes));
                                check(format!("{p}/n_caps"), row.capacities.len() as f64);
                                for (key, col) in [("capacities", &row.capacities),
                                                   ("throat_loadings", &row.throat_loadings),
                                                   ("margins", &row.margins)] {
                                    check(format!("{p}/{key}_first"), col[0]);
                                    check(format!("{p}/{key}_last"), col[col.len() - 1]);
                                }
                            }
                        }
                        for (i, row) in m.running_line_shift(&f, throttles).iter().enumerate() {
                            let p = format!("shift/{cfg}/{i}");
                            for &(spn, spool) in SPOOLS.iter() {
                                let s = row.spool(spool);
                                let vals = [s.n_lumped, s.n_stacked, s.d_n, s.phi_lumped,
                                            s.phi_stacked, s.d_phi, s.pi_lumped, s.pi_stacked,
                                            s.d_pi];
                                for (key, v) in SHIFT_KEYS.iter().zip(vals) {
                                    check(format!("{p}/{spn}/{key}"), v);
                                }
                            }
                            check(format!("{p}/thrust_lumped"), row.thrust_lumped);
                            check(format!("{p}/thrust_stacked"), row.thrust_stacked);
                            check(format!("{p}/d_thrust"), row.d_thrust);
                        }
                    }
                }
            }
        }
    }
    t
}

fn b(v: bool) -> f64 {
    if v { 1.0 } else { 0.0 }
}

/// § 5.10 (i)/(ii)'s arm — the ONE caught scope, on CAPACITY-FREE maps (all 160 rows, which is
/// `probe_n3.probe_scan_cells` verbatim; see the dump's note on § 5.10's "80 of the 160").
fn sweep_schedule(
    t: &mut Tally, gases: &[&str], throttles: &[f64], mut check: impl FnMut(String, f64),
) {
    let f = flight();
    for gname in gases {
        for (sname, ml0, mh0) in shapes() {
            let (a_l, a_h) = armed(ml0, mh0, false);
            for &(spn, spool) in SPOOLS.iter() {
                for vs in [None, Some(1usize)] {
                    let (vl, vh) = match spool {
                        Spool::Lp => (vs, None),
                        Spool::Hp => (None, vs),
                    };
                    let m = StageStackCore::new(StageStackCoreSpec {
                        k_lp: 8, k_hp: 8, vsv_stages_lp: vl, vsv_stages_hp: vh,
                        ..StageStackCoreSpec::new(design(gname), flight(), 1.0, a_l, a_h)
                    });
                    let vtag = match vs { None => "N".to_string(), Some(v) => v.to_string() };
                    let tag = format!("{gname}/{sname}/{spn}/vs{vtag}");
                    for row in m.stage_incidence_schedule(&f, throttles, spool, 0, 4.0) {
                        t.sched_rows += 1;
                        if vs.is_some() {
                            t.sched_frontrow += 1;
                            if row.reached { t.sched_reached_frontrow += 1; }
                        }
                        if row.reached { t.sched_reached += 1; }
                        else if row.tt4 == 1500.0 { t.sched_missed_at_design += 1; }
                        let p = format!("sched/{tag}/{:.0}", row.tt4);
                        check(format!("{p}/reached"), b(row.reached));
                        check(format!("{p}/K"), row.k as f64);
                        check(format!("{p}/has_vsv_stages"), b(row.vsv_stages.is_some()));
                        check(format!("{p}/vsv_stages"),
                              row.vsv_stages.map_or(-1.0, |v| v as f64));
                        let vals = [row.vsv_star, row.residual, row.tan_b1, row.tan_b1_design,
                                    row.phi_stage, row.phi_stage_bare, row.m_i, row.m_i_bare,
                                    row.m_i_worst, row.worst as f64, row.n, row.n_bare,
                                    row.d_n, row.rear_excess];
                        for (key, v) in SCHED_KEYS.iter().zip(vals) {
                            check(format!("{p}/{key}"), v);
                        }
                    }
                }
            }
        }
    }
}

/// Compare a census against the dump's own counts. The two floors are ONE counter in Python
/// (`march` adds both into `clamped`), which is § 5.10 (iii)'s point, so the sum is what can be
/// compared and the split is asserted beside it.
///
/// `bracket_aborts` is subtracted from the pass-count identity for a reason the first draft got
/// wrong: a solve that fails the BRACKET returns before the loop and runs **zero** passes, while
/// the clamped-root one runs the full 48 and aborts after it. On the schedule arm that is 39
/// calls × 48 passes of difference — a bar that would have been off by 1 872 and read as a port
/// defect.
fn check_census(c: &StackCensus, want: &HashMap<&str, f64>, tag: &str, bracket_aborts: u64,
                read: &mut std::collections::HashSet<String>, bad: &mut Vec<String>) {
    let mut g = |k: &str| -> f64 {
        let key = format!("census/{tag}/{k}");
        let v = *want.get(key.as_str())
            .unwrap_or_else(|| panic!("dump has no {key}"));
        read.insert(key);
        v
    };
    for (k, got) in [("stacks_built", c.stacks_built), ("marches", c.marches),
                     ("solve_n_calls", c.solve_n_calls),
                     ("capacities_built", c.capacities_built),
                     ("capacities_hits", c.capacities_hits)] {
        let exp = g(k) as u64;
        if got != exp {
            bad.push(format!("census/{tag}/{k}: rust {got} vs python {exp}"));
        }
    }
    let clamped = c.t_floor_fires + c.p_floor_fires;
    if clamped != g("clamped_total") as u64 {
        bad.push(format!("census/{tag}/clamped_total: rust {} (t {} + p {}) vs python {}",
                         clamped, c.t_floor_fires, c.p_floor_fires, g("clamped_total") as u64));
    }
    if c.p_floor_fires != 0 {
        bad.push(format!("census/{tag}: _P_FLOOR fired {} times — § 5.10 P8 says never, and \
                          Python's shared counter cannot see the difference",
                         c.p_floor_fires));
    }
    // Both bisections break on an ABSOLUTE width over a FIXED bracket, so the pass count is
    // `ceil(log2(width/1e-14)) = 48` and cannot depend on the data. Gated as a product rather
    // than dumped: Python cannot report it without re-implementing the loop bodies.
    let k1 = g("solve_n_k1") as u64;
    if c.eta_passes != 48 * c.stacks_built {
        bad.push(format!("census/{tag}/eta_passes: {} != 48 x {} stacks",
                         c.eta_passes, c.stacks_built));
    }
    let looped = c.solve_n_calls - k1 - bracket_aborts;
    if c.solve_n_passes != 48 * looped {
        bad.push(format!("census/{tag}/solve_n_passes: {} != 48 x ({} calls - {} K=1 - {} \
                          bracket aborts)", c.solve_n_passes, c.solve_n_calls, k1,
                         bracket_aborts));
    }
    // § 5.10 P5's `K == 1` DISPATCH IS UNREACHABLE THROUGH THE MATCHER, and this measured it.
    // `StageStackCore` builds a stack only where `K > 1`, so at `K = 1` the efficiency loop is
    // rung 39's own and calls `ComponentMap::solve_n` directly — `StageStack::solve_n`'s own
    // `if k == 1` branch is reached ONLY from a hand-built stack (step 2's smoke cell D). The
    // dispatch is still the right spelling; the count says no matcher-driven path exercises it,
    // which is a different fact from the branch being wrong.
    if k1 != 0 {
        bad.push(format!("census/{tag}/solve_n_k1 = {k1} — a matcher reached the K=1 dispatch, \
                          which the whole grid says is unreachable"));
    }

    // --- THE TWO KEYS RUST CANNOT REPRODUCE, READ RATHER THAN LEFT ORPHANED ---------------
    //
    // A `seen == want.len()` accounting that counts census keys by PREFIX certifies them as
    // covered whether or not anyone looked, which is *a documented gate that doesn't exist* with
    // its own alibi. So both are read here, each with the bar it can actually carry.
    //
    // `marches_clamped` counts MARCHES that clamped; `StackCensus` counts STAGE firings, and
    // adding a per-march flag would be a step-2 edit at step 4 (§ 5.9 (a)'s ripple). What it is
    // kept FOR is the shape of `clamped_total`: 49 173 stage firings over 22 565 marches is ~2.2
    // stages each, not one pathological march doing all of it — so the RELATION, not the count,
    // is the gate. *Make a dead key earn its place* rather than delete it.
    let mc = g("marches_clamped") as u64;
    if !(mc > 0 && mc <= clamped) {
        bad.push(format!("census/{tag}: {mc} clamped marches against {clamped} stage firings —                           each clamped march clamps at least one stage, so mc <= total"));
    }
    // `map_solve_n_calls` is § 5.10 (i)'s THIRD ROW, and the reason P1 could not inherit slice
    // M's answer: with both spools stacked, `ComponentMap::solve_n` is never called at all.
    // `map.rs` exposes `psi_calls` but no solve counter, and adding one is a step-1 file edit —
    // so this is a bar on the DUMP, stated as such, and it is the pre-registered claim verbatim.
    let msn = g("map_solve_n_calls");
    let expect_zero = tag != "fast";
    if (msn == 0.0) != expect_zero {
        bad.push(format!("census/{tag}/map_solve_n_calls = {msn}: the schedule and equilibrium                           arms are fully stacked so it must be ZERO, and the fast arm's                           `running_line_shift` builds a K=1 baseline so it must not be"));
    }
}

// ==========================================================================================
// THE PYPY ARM — bit-for-bit
// ==========================================================================================

#[test]
fn slice_n_matches_pypy_bit_for_bit() {
    let want = load(ORACLE_PYPY);
    let mut seen = 0usize;
    let mut bad: Vec<String> = Vec::new();
    let mut t;
    let (fast_census, sched_census);
    let mut read: std::collections::HashSet<String> = std::collections::HashSet::new();
    let _ = take_census();
    {
        let mut check = |key: String, got: f64| match want.get(key.as_str()) {
            None => bad.push(format!("{key}: absent from the dump")),
            Some(&exp) => {
                if got.to_bits() != exp.to_bits() {
                    bad.push(format!("{key}: rust {got:e} vs python {exp:e}"));
                }
                seen += 1;
            }
        };
        t = sweep_fast(&["cpg", "tpg"], &K_GRID, &SPLITS, &PROFILES, &THROTTLE_FAST, true,
                       &mut check);
        fast_census = take_census();
        sweep_schedule(&mut t, &["cpg", "tpg"], &THROTTLE_FAST, &mut check);
        sched_census = take_census();
    }
    check_census(&fast_census, &want, "fast", want["fire/fast/bracket"] as u64, &mut read,
                 &mut bad);
    check_census(&sched_census, &want, "sched", want["fire/sched/bracket"] as u64, &mut read,
                 &mut bad);
    assert!(bad.is_empty(), "{} of {} keys differ:\n{}", bad.len(), want.len(),
            bad.iter().take(12).cloned().collect::<Vec<_>>().join("\n"));
    // --- THE ACCOUNTING, AND IT IS NOT A PREFIX COUNT --------------------------------------
    // Counting census/fire keys by PREFIX would certify them as covered whether or not any gate
    // looked at them — the accounting supplying its own alibi, which is *a documented gate that
    // doesn't exist* read from the other side. `read` is the set of keys a gate actually
    // consumed, and the assert below is that it IS the census/fire key set.
    let mut fire = |key: &str| -> f64 {
        read.insert(key.to_string());
        *want.get(key).unwrap_or_else(|| panic!("dump has no {key}"))
    };
    let (f_br, f_cl) = (fire("fire/sched/bracket"), fire("fire/sched/clamped_root"));
    for k in ["fire/fast/bracket", "fire/fast/clamped_root", "fire/fast/other",
              "fire/fast/map_bracket"] {
        let v = fire(k);
        assert_eq!(v, 0.0, "{k} = {v} — the fast arm has no caught scope, so a firing there is a                             raise the sweep survived by accident");
    }
    assert_eq!(fire("fire/sched/other"), 0.0,
               "§ 5.10 P1's refutation clause: an abort reaching the scan from a THIRD frame");
    assert_eq!(fire("fire/sched/map_bracket"), 0.0,
               "with both spools stacked, ComponentMap::solve_n is never called — slice M's                 frame is absent, which is why P1 was measured rather than inherited");
    assert_eq!((f_br, f_cl), (39.0, 1.0), "§ 5.10 (i)'s frame census, to the firing");
    assert_eq!(fire("fire/has_clamped_sample"), 1.0);
    // RECORDED HERE, CHECKED THERE. `is_finite` is not a bar — `at_gas`/`at_spool`/`at_vs` are
    // small integers and nothing realistic fails it. The loop exists so these keys enter `read`
    // and `owed` stays honest; the values are consumed as a rebuild recipe by
    // `the_clamped_root_arm_is_reached_from_the_dump_grid`, which is where they are checked (its
    // `tau_d`/`e_d` bit asserts) and where a wrong one fails.
    for k in ["m", "tau_c", "eta_live", "K", "vsv", "tau_d", "pi_d", "eta_d", "e_d",
              "at_gas", "at_shape", "at_spool", "at_vs"] {
        assert!(fire(&format!("fire/clamped/{k}")).is_finite(),
                "the clamped-root sample's {k}");
    }

    // § 5.10 (ii)'s 40 AND § 5.10 (i)'s 40 ARE THE SAME 40, and that is not automatic: a row can
    // also fail to reach by walking the scan to `v_hi` without a sign change. The equality is
    // what says EVERY non-reached row is a RAISE.
    assert_eq!(f_br + f_cl, (t.sched_rows - t.sched_reached) as f64,
               "every non-reached schedule row must be a raise, not an exhausted scan");

    let owed: Vec<&str> = want.keys()
        .filter(|k| (k.starts_with("census/") || k.starts_with("fire/")) && !read.contains(**k))
        .copied().collect();
    assert!(owed.is_empty(), "{} census/fire keys are in the dump and read by NO gate: {:?}",
            owed.len(), owed);
    assert_eq!(seen + read.len(), want.len(),
               "read {seen} value keys + {} census/fire keys of the dump's {}", read.len(),
               want.len());

    // --- THE GRID IS THE PRE-REGISTERED ONE ---------------------------------------------
    assert_eq!(t.cells, 640, "§ 5.10's grid: 2 gases x 5 shapes x K{{2,4,8,16}} x 2 splits \
                              x 2 profiles x 4 throttles");
    assert_eq!(t.half_rows, 1280, "640 cells x 2 spools");

    // --- § 5.10 (iv): TWO POPULATIONS, AND THE ACCOUNTING CLOSES ------------------------
    // The bar is NOT "binds is front-or-rear": it is front-or-rear everywhere except a handful
    // of DEGENERATE-ARGMIN readings, which are a tie-break and not a third physical class. The
    // interior count is asserted with its spread beside it so it can never read as physics.
    assert_eq!(t.binds[0], [240, 400, 0], "binds on the DERIVED profile (front, rear, interior)");
    assert_eq!(t.binds[1], [50, 587, 3], "binds on the UNIFORM profile");
    assert_eq!(t.inc, [1182, 88, 10], "inc_worst, either profile — the profile cannot see it");
    assert_eq!(t.binds[0].iter().sum::<usize>(), 640, "derived-profile half-rows");
    assert_eq!(t.binds[1].iter().sum::<usize>(), 640, "uniform-profile half-rows");
    // THE 13 INTERIOR READINGS ARE ONE POPULATION, AND THAT IS THE BAR — not "binds is
    // front-or-rear". Each is at the DESIGN throttle, on the HP spool, on the CPG gas, where
    // every row sits at `phi_k = 1` and the per-row margins collapse to within a couple of ULP.
    // Measured worst spread 4.441e-16 on values of 0.4 and 0.818.
    assert_eq!(t.interior.len(), 13, "§ 5.10 (iv)'s 3 binds + 10 inc_worst");
    for (name, spread) in &t.interior {
        assert!(*spread < 1e-15,
                "an INTERIOR argmin at {name} has margin spread {spread:e} — that is a third \
                 physical class, not the last-bit tie § 5.10 (iv) measured");
        assert!(name.contains("/cpg/") && name.contains("/1500/") && name.ends_with("/hp"),
                "the interior readings are ONE population — HP, CPG, design throttle — and \
                 {name} is not in it");
    }

    // --- § 5.10 (ii)/P7: THE vsv_stages SPLIT, AS A COUNT --------------------------------
    assert_eq!(t.sched_rows, 160, "2 gases x 5 shapes x 2 spools x vsv_stages{{None,1}} x 4");
    assert_eq!(t.sched_frontrow, 80);
    assert_eq!(t.sched_reached_frontrow, 80,
               "EVERY front-row-lever row reaches — rung 55's P3 showing up as map validity");
    assert_eq!(t.sched_missed_at_design, 0, "no miss is at the design throttle");
    assert_eq!(t.sched_reached, 120, "§ 5.10 (ii): 120 of 160 reached, the 40 misses all lumped");

}

// ==========================================================================================
// THE EQUILIBRIUM ARM — its own file, because it is its own process
// ==========================================================================================

/// The equilibrium gas on `stage_throat_margin` only — § 5.10 (v)'s sized arm.
///
/// **IT IS WHERE P1's REFUTATION CLAUSE WOULD SHOW.** Step 3 recorded one divergence and did not
/// repair it: rung 55's efficiency-loop non-convergence is an `AssertionError` in Python, which
/// `stage_incidence_schedule` would CATCH, and a `panic!` in Rust, which nothing can. On the
/// equilibrium gas `_equil_solve`'s Newton and the burner's `_solve_f` are live inside the same
/// scope. This arm carries no schedule (one row costs 36.9 s — § 5.10 (v)), so what it can show
/// is a value or a row-count divergence, and a THIRD raising frame would appear as the sweep
/// dying rather than as a mismatch.
#[test]
fn slice_n_equilibrium_matches_pypy_bit_for_bit() {
    let want = load(ORACLE_EQ);
    let mut seen = 0usize;
    let mut bad: Vec<String> = Vec::new();
    let _ = take_census();
    let (t, c);
    {
        let check = |key: String, got: f64| match want.get(key.as_str()) {
            None => bad.push(format!("{key}: absent from the dump")),
            Some(&exp) => {
                if got.to_bits() != exp.to_bits() {
                    bad.push(format!("{key}: rust {got:e} vs python {exp:e}"));
                }
                seen += 1;
            }
        };
        t = sweep_fast(&["eq"], &K_EQ, &SPLITS, &PROFILES[..1], &THROTTLE_EQ, false, check);
        c = take_census();
    }
    // No caught scope on this arm, so a bracket abort would have killed the dump rather than
    // been counted — zero by the fact that the dump completed.
    let mut read: std::collections::HashSet<String> = std::collections::HashSet::new();
    check_census(&c, &want, "equil", 0, &mut read, &mut bad);
    assert!(bad.is_empty(), "{} of {} keys differ:\n{}", bad.len(), want.len(),
            bad.iter().take(12).cloned().collect::<Vec<_>>().join("\n"));
    let owed: Vec<&str> = want.keys()
        .filter(|k| k.starts_with("census/") && !read.contains(**k)).copied().collect();
    assert!(owed.is_empty(), "census keys read by no gate: {owed:?}");
    assert_eq!(seen + read.len(), want.len());
    assert_eq!(t.cells, 40, "5 shapes x K{{2,8}} x 2 splits x derived x 2 throttles");
}

// ==========================================================================================
// THE TWO GATES THE VALUE DUMP CANNOT CARRY
// ==========================================================================================

/// `slice_n_deferrals_so_far` item 2b, DISCHARGED — `try_solve_n`'s CLAMPED-ROOT arm.
///
/// § 5.10 (i) measured it at **1 firing in 40**, three frames down inside an efficiency loop and
/// behind `except AssertionError: break`. So reproducing the schedule does NOT gate it: the catch
/// swallows which arm fired, and step 2's smoke could only reach the BRACKET arm because a
/// hand-built cell has no way to produce a root that exists whose march still clamps.
///
/// The dump records the `(m, tau_c, eta_live)` triple at the raise together with the cell it
/// happened in, so this re-enters the arm DIRECTLY, on the stack that produced it.
#[test]
fn the_clamped_root_arm_is_reached_from_the_dump_grid() {
    let want = load(ORACLE_PYPY);
    assert_eq!(want["fire/has_clamped_sample"], 1.0,
               "the dump grid produced no clamped-root firing, so item 2b is still OPEN — say \
                so in the ledger rather than letting this gate read as covering it");
    let g = |k: &str| want[format!("fire/clamped/{k}").as_str()];
    let gases = ["cpg", "tpg"];
    let gname = gases[g("at_gas") as usize];
    let (sname, ml0, mh0) = shapes().swap_remove(g("at_shape") as usize);
    let (spn, spool) = SPOOLS[g("at_spool") as usize];
    let vs = match g("at_vs") { v if v < 0.0 => None, v => Some(v as usize) };
    let (vl, vh) = match spool { Spool::Lp => (vs, None), Spool::Hp => (None, vs) };
    let (a_l, a_h) = armed(ml0, mh0, false);
    let m = StageStackCore::new(StageStackCoreSpec {
        k_lp: 8, k_hp: 8, vsv_stages_lp: vl, vsv_stages_hp: vh,
        ..StageStackCoreSpec::new(design(gname), flight(), 1.0, a_l, a_h)
    });
    // The scan reads the residual on a SIBLING at the scanned setting, so the stack that raised
    // carries that setting on its own map — which is what the dumped `vsv` is.
    let moved = match spool {
        Spool::Lp => m.at_setting(g("vsv"), 0.0),
        Spool::Hp => m.at_setting(0.0, g("vsv")),
    };
    let st = moved.stack_of(spool).expect("K = 8 on both spools");
    assert_eq!(st.tau_d.to_bits(), g("tau_d").to_bits(),
               "the rebuilt stack must be the one that raised ({gname}/{sname}/{spn})");
    assert_eq!(st.e_d.to_bits(), g("e_d").to_bits());

    let err = st.try_solve_n(g("m"), g("tau_c"), g("eta_live"))
                .expect_err("this triple is the one Python's clamped-root assert fired on");
    assert!(err.0.contains("sits in the clamped (non-physical) region"),
            "the CLAMPED-ROOT arm, not the bracket one: {}", err.0);
    // And it is a genuinely different arm, not the bracket arm relabelled: a root DID exist.
    assert!(!err.0.contains("bracket fails"));
}

/// `cap_profile` is read ONLY by `StageStack::capacities`, so it cannot touch the march, the
/// matched point, or any rung-55 reading. Gated as a BIT identity rather than dumped twice.
///
/// This is what licenses the dump carrying `stage_margin`/`work_gap` on the derived profile
/// alone. Dumping the duplicate would have doubled the file to restate a claim; asserting the
/// identity here is the stronger statement, and it would catch a port that let the profile leak
/// into `march` — which no value key in the dump could see, because both profiles would then be
/// wrong together on the capacity side and right together on the incidence side.
#[test]
fn the_capacity_profile_cannot_reach_any_rung_55_reading() {
    let f = flight();
    let mut moved = 0usize;
    for gname in ["cpg", "tpg"] {
        for (sname, ml0, mh0) in shapes() {
            let (a_l, a_h) = armed(ml0, mh0, true);
            for k in [2usize, 8] {
                for &(_, split) in SPLITS.iter() {
                    let d = stacked(design(gname), a_l, a_h, k, split, CapProfile::Derived);
                    let u = stacked(design(gname), a_l, a_h, k, split, CapProfile::Uniform);
                    for tt4 in [1500.0f64, 1000.0] {
                        let (sd, su) = (d.stage_margin(&f, tt4), u.stage_margin(&f, tt4));
                        let (gd, gu) = (d.work_gap(&f, tt4), u.work_gap(&f, tt4));
                        for &(_, spool) in SPOOLS.iter() {
                            let (a, b) = (sd.spool(spool), su.spool(spool));
                            assert_eq!(a.m_i_worst.to_bits(), b.m_i_worst.to_bits(),
                                       "{gname}/{sname}/K{k}/{tt4}: the profile moved an \
                                        INCIDENCE reading");
                            assert_eq!(a.rear_excess.to_bits(), b.rear_excess.to_bits());
                            assert_eq!(a.worst, b.worst);
                            for (x, y) in a.stages.iter().zip(&b.stages) {
                                assert_eq!(x.phi.to_bits(), y.phi.to_bits());
                                assert_eq!(x.m_i.to_bits(), y.m_i.to_bits());
                            }
                            let (x, y) = (gd.spool(spool), gu.spool(spool));
                            assert_eq!(x.tau_marched.to_bits(), y.tau_marched.to_bits(),
                                       "the profile moved the MARCHED WORK, i.e. the solve");
                            assert_eq!(x.n.to_bits(), y.n.to_bits());
                        }
                        // …and it DOES move the capacity currency, or the gate above is a
                        // comparison of two identical objects.
                        let (td, tu) = (d.stage_throat_margin(&f, tt4),
                                        u.stage_throat_margin(&f, tt4));
                        for &(_, spool) in SPOOLS.iter() {
                            if td.spool(spool).m_c_worst.to_bits()
                                != tu.spool(spool).m_c_worst.to_bits() {
                                moved += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(moved > 0, "the two profiles produced identical CAPACITY margins everywhere, so \
                        the invariance above is a self-comparison");
}

// ==========================================================================================
// THE CPYTHON ARM — the SAME 640 cells and 160 schedule rows, WITHOUT the per-row projections
// ==========================================================================================

/// CPython's `lean` dump: **41 560 keys, 2.73 MB** against the PyPy arm's 72 520 / 4.86 MB.
///
/// The omission is the per-row keys and it is deliberate, with its size beside it. Those exist
/// for COVERAGE against PyPy — measured, an interior-row defect of 2 ULP moves **7 040 per-row
/// keys and only 6 aggregates**, so they are the difference between catching it and not. This arm
/// answers a different question — how much of the dump is interpreter-STABLE, i.e. how strong the
/// PyPy bit claim is — and an argmin over all `K` rows already moves when a row drifts far
/// enough to matter. Its bar is over exactly what it dumped.
///
/// **THE FLAG THAT PRODUCES THE LEAN ARM WAS BROKEN AND A COUNT COULD NOT SEE IT.** The dump's
/// `rows` parameter was shadowed by `rows = m.throat_walk(...)` in the same function, so it
/// disarmed itself after the first subgrid cell: 71 504 keys instead of 41 560, which next to
/// `fast`'s 72 520 reads as "about the same". Only a key-SET diff found the 856 missing, all in
/// the first shape — this project's *coverage is a name → parameter-set diff, never a count*,
/// arriving on a dump script rather than on a test list.
#[test]
fn slice_n_matches_cpython() {
    let want = load(ORACLE_CPYTHON);
    let mut seen = 0usize;
    let mut bad: Vec<String> = Vec::new();
    let mut worst = (0.0f64, String::new());
    let mut smallest = (f64::INFINITY, String::new());
    let (mut t_design_argmin, mut t_design_flips, mut t_offdesign_argmin) = (0usize, 0, 0);
    let mut t;
    {
        let mut check = |key: String, got: f64| {
            if is_per_row(&key) {
                return;                 // not in this arm's dump — see the note above
            }
            let Some(&exp) = want.get(key.as_str()) else {
                bad.push(format!("{key}: absent from the dump"));
                return;
            };
            seen += 1;
            if is_discrete(&key) {
                if is_argmin(&key) && at_design(&key) {
                    // The degenerate half of the argmin — counted, not held. See `at_design`.
                    t_design_argmin += 1;
                    if got.to_bits() != exp.to_bits() {
                        t_design_flips += 1;
                    }
                    return;
                }
                if got.to_bits() != exp.to_bits() {
                    bad.push(format!("{key}: DISCRETE, rust {got:e} vs cpython {exp:e}"));
                }
                if is_argmin(&key) {
                    t_offdesign_argmin += 1;
                }
                return;
            }
            if exp != 0.0 && exp.abs() < smallest.0 {
                smallest = (exp.abs(), key.clone());
            }
            let d = (got - exp).abs();
            if d <= ABS_BAR {
                return;
            }
            let rel = d / exp.abs();
            if rel > worst.0 {
                worst = (rel, key.clone());
            }
            if rel > REL_BAR {
                bad.push(format!("{key}: rust {got:e} vs cpython {exp:e} (rel {rel:e})"));
            }
        };
        t = sweep_fast(&["cpg", "tpg"], &K_GRID, &SPLITS, &PROFILES, &THROTTLE_FAST, true,
                       &mut check);
        let _ = take_census();
        sweep_schedule(&mut t, &["cpg", "tpg"], &THROTTLE_FAST, &mut check);
        let _ = take_census();
    }
    assert!(bad.is_empty(), "{} of {} keys differ from CPython:\n{}", bad.len(), want.len(),
            bad.iter().take(12).cloned().collect::<Vec<_>>().join("\n"));
    let census_keys = want.keys().filter(|k| k.starts_with("census/") || k.starts_with("fire/"))
                          .count();
    assert_eq!(seen + census_keys, want.len(),
               "read {seen} value keys + {census_keys} census/fire of the dump's {}", want.len());
    assert!(worst.0 < WORST_BAR, "worst CPython deviation {:e} at {}", worst.0, worst.1);
    // The census counts are NOT re-asserted here — slice K's P2 established that iteration
    // counts are not interpreter-invariant, and the schedule's scan LENGTH is data-dependent.
    // The branch verdicts are, because a verdict is a comparison and not an iteration.
    assert_eq!(t.binds[0], [240, 400, 0], "the binds census is interpreter-invariant");
    assert_eq!(t.binds[1], [50, 587, 3]);
    assert_eq!(t.inc, [1182, 88, 10]);
    assert_eq!((t.sched_reached, t.sched_rows), (120, 160));

    // --- THE ARGMIN TIER, WITH BOTH HALVES COUNTED --------------------------------------
    // The off-design half is a real bit bar and it is not vacuous: 7 481 keys carry it. The
    // design half is where the flips live, and the count is pinned so a port that started
    // flipping OFF design could not hide inside it.
    assert_eq!(t_offdesign_argmin, 4680, "argmin keys held to BITS off design, ZERO differing");
    assert_eq!(t_design_argmin, 1560, "argmin keys at the design throttle");
    assert_eq!(t_design_flips, 520,
               "the MEASURED interpreter flips — all of them at design, where the per-row                 margins are within 2 ULP. A change here is a change in how many rows tie,                 which is a statement about the arithmetic and not about the tie-break rule.");
}
