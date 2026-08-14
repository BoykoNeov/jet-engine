//! PHASE 5L GATE — every rung-41/42 value the Python oracle dumped, recomputed in Rust.
//!
//! WHAT THIS GATE IS BUILT TO CATCH:
//!
//! * **A DISPATCH THAT COMPILES AND RETURNS THE WRONG PHYSICS.** Rung 42 overrides a live virtual
//!   slot, and rung 41's three schedule methods reach it through `try_match_point`. Naming rung
//!   39's function in `R42`'s table produces numbers, not an error. § 4's block sweeps all three
//!   methods on a RUNG-42 core at `b > 0` — the narrowing § 5.8's step-4 line called out, since
//!   § 5.8.1's grid ran them on rung-39 matchers only and would have witnessed the dispatch
//!   through `surge_margin` alone.
//!
//! * **A NULL COLUMN CARRYING A PLAUSIBLE NUMBER.** `flow_coefficient_turn`'s `RAIL` branch nulls
//!   `pi_star`/`star_form` and OMITS `gamma_c`/`far`; `match` at `b = 0` returns rung 39's object,
//!   which has no booking at all. A port writing `0.0` there compares EQUAL to nothing and means
//!   something else, so both are dumped as a discriminant plus a declared impossible sentinel
//!   (`NULL = -1.0`), the null keys are PRESENT on both sides, and the branch COUNTS are gated.
//!
//! * **A CENSUS THAT MEASURES PHYSICS RATHER THAN PLUMBING.** Rung 42's UNCHOKED count rises
//!   23 → 23 → 24 → 25 with `b`: that is rung 42's own gate 6, *opening the valve shrinks the
//!   choked envelope*, expressed as a count. It is the counterexample to `docs`' *guessed census
//!   bars* entry — a count bar earns its place when the source has a claim it can refute.
//!
//! * **A JOINT LOOP RUNNING A DIFFERENT NUMBER OF PASSES** — and, as in slice K, that count is
//!   NOT interpreter-invariant on the integral gases. Bit-gated on PyPy, reported and
//!   cross-checked on CPython, never silently skipped.
//!
//! **THE GRID IS THE `mixed` SHAPE PAIR, AND THAT IS NOT A FREE CHOICE.** Every census number
//! § 5.8.1 (v) pre-registered was measured on `(LP_SHAPED, HP_SHAPED)`, and P7's census half is
//! that `b = 0` reproduces slice K's rung-39 row — which is slice K's `mixed` row. Swept on FLAT
//! maps instead, this dump read 68/68/68/67 matched with UNCHOKED flat at 23: numbers that are
//! perfectly correct, answer a different question, and would have looked like a REFUTATION of
//! rung 42's gate 6. Caught in the act, and it is § 5.7 (e)'s rule — a bar is measured on the
//! grid it will be gated on, never read off a neighbouring one.
//!
//! **WHAT IS DELIBERATELY ABSENT: the refinement count 33.** Python cannot instrument the shipped
//! body's two phases apart from outside, so its arm would be a transcription of the same loop and
//! the comparison would be self-confirming — rung 83's *identity round-trip sold as verification*.
//! It is gated in `rung41.rs` against the arithmetic instead.
//!
//! # THE DETECTOR WAS MEASURED, AND IT SAYS THE `value` BAR IS BLIND
//!
//! "25 458 / 25 458 bit-identical" is an observation until the check is calibrated — `docs`'
//! *slice J* entry is a 7 252-key bit-exact oracle that passed a deliberately mis-spelled square.
//! So one of the three `(1-b)` associations was flipped (`eta_m * (1+f) * (1-b)` for
//! `eta_m * (1-b) * (1+f)`: algebraically identical, a different double), the gate re-run, and the
//! defect reverted.
//!
//! **254 of 25 458 keys moved — 1.00 % — and exactly ONE of them exceeded the `value` bar. It was
//! not a value: it was an `n_pass`,** the joint loop taking 12 passes instead of 11. The worst
//! VALUE deviation over the whole sweep was **2.05e-9**, comfortably inside the 1e-8 relative bar
//! every value key is held to on the CPython arm.
//!
//! Two consequences, and both are why this file is built the way it is:
//!
//! * **The PyPy arm's bit-equality is not belt-and-braces, it is the detector.** Toleranced at
//!   1e-8 this gate would have caught the defect on 1 key of 25 458 rather than 254 — and that one
//!   key only exists because the pass count is dumped at all.
//! * **The edge cells earn their place.** The worst-moved value sits at `M0 = 1.60` on the
//!   thermally-perfect gas, which is precisely where § 5.8.3 (h) said this defect class surfaces.
//!   A sweep confined to the comfortable middle would have moved fewer rows still.
//!
//! # AND THE `value` BAR WAS COPIED, NOT MEASURED — WHAT THE CPYTHON ARM FOUND
//!
//! 1e-8 came from slice K's oracle, where it holds. On THIS dump the CPython arm failed on 34 of
//! 23 772 value keys, and the 34 turned out to be two disjoint populations with nothing else
//! between them — the accounting closes exactly, which is the reason to believe the split:
//!
//! **(A) THE LOCATION OF A FLAT EXTREMUM — 28 keys = 7 turn cells × 4 fields.** In the SAME cell,
//! out of the SAME golden section:
//!
//! ```text
//!   phi_star  (what the turn is WORTH)   worst  4.07e-11
//!   Tt4_star  (WHERE the turn is)        worst  7.39e-06
//!   far       (read AT that location)    worst  1.58e-05
//! ```
//!
//! Five to six orders apart, and the direction is not an accident: at an interior maximum the
//! objective's slope is zero, so noise in the objective — here the inner matcher's own
//! convergence, not machine epsilon — buys a first-order move in the abscissa and none in the
//! ordinate. The bracket is driven to 1e-5 K by a stopping rule the objective's noise floor
//! cannot support, so the extra refinements resolve nothing and the two interpreters settle
//! anywhere inside the ε-optimal set.
//!
//! **This INVERTS `docs`' *shape keys* entry** — *a peak's VALUE drifts between interpreters and
//! its LOCATION does not*. That was an argmax over a discrete GRID, which quantises the answer and
//! snaps both interpreters to the same node. This is an argmax over a CONTINUUM, where there is no
//! quantum to snap to. Same word, opposite conditioning, and the grid is what was doing the work.
//!
//! One thing deliberately NOT claimed: √(4.07e-11) = 6.4e-6 sits within 16 % of the worst
//! `Tt4_star`, which looks like the textbook √ε law. Per CELL the ratio runs 0.07 to 6.17 — a
//! spread of ~90 — so that agreement is two maxima meeting, not a law. Rung 66's *check where an
//! extremum sits before quoting it*. What is asserted is only the ORDERING, cell by cell.
//!
//! **(B) A PASS-COUNT FLIP COSTS ONE DECADE — 6 keys.** Value keys in cells whose joint loop ran a
//! different number of passes reach 1.55e-8; in cells that ran the same count they stop at
//! 1.03e-9. Both populations are now measured and BOTH bars are asserted, because the content is
//! the separation: if the unflipped half ever reaches the flipped half's bar, the flip is not what
//! is driving the drift and the section is measuring the wrong thing.
//!
//! **CPG is clean on both.** Zero calorically-perfect turn cells moved their location; zero
//! calorically-perfect pass counts flipped. Slice K established that invariant on the loop count
//! alone — here a second, unrelated phenomenon obeys it, for the same reason: a closed form has no
//! root-find under it for last-bit arithmetic to tip.
//!
//! # ONE PREDICTION THE DUMP DOES *NOT* WITNESS, STATED RATHER THAN LEFT IN A POOLED COUNT
//!
//! § 5.8.3 (g) reasoned that `flow_coefficient_turn`'s `MIN`/`RAIL` branch **can flip under
//! bleed**, since bleed moves `phi` and therefore the argmin index. On this grid it does not.
//! Reading the 16 rung-42-core turns cell by cell rather than as the pooled `10 MIN / 6 RAIL`:
//! `Hp` is **MIN in all 8**, `Lp` is **RAIL in all 6** at `M0 = 0.85` and **MIN in both** at
//! `M0 = 1.60` — and `kind` is IDENTICAL at `b = 0.00` and `b = 0.10` in every one of the eight
//! (gas, spool, flight) cells. So the branch driver here is the FLIGHT MACH, not the valve.
//!
//! The concern was still worth acting on — it is why `kind` is dumped per cell at all, so the
//! claim is gated rather than assumed. But a pooled `10 / 6` invites reading a bleed-driven flip
//! into a number that contains none, which is the shape of `docs`' *guessed census bars* entry.
//! Recorded so no later slice inherits the flip as established.
//!
//! Regenerate the oracle with:
//!     py -3                     rust/oracle/dump_slice_l.py rust/oracle/slice_l_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_slice_l.py rust/oracle/slice_l_pypy.tsv

use std::collections::HashMap;
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, counters, FlowTurn, Spool, TurnKind,
                          TwoSpoolLosses, TwoSpoolMapCore, TwoSpoolMapMatcher};

const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_l_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/slice_l_pypy.tsv");

fn load_oracle(text: &str) -> HashMap<&str, f64> {
    let mut out = HashMap::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let mut it = line.split('\t');
        let key = it.next().expect("key");
        let bits: u64 = it.next().expect("bits").parse().expect("u64 bits");
        out.insert(key, f64::from_bits(bits));
    }
    out
}

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

/// The declared sentinel — see the module note. Impossible for every column it stands in for.
const NULL: f64 = -1.0;

/// THE GRID, matching `dump_slice_l.py` line for line.
const M0S: &[f64] = &[0.0, 0.3, 0.5, 0.85, 1.2, 1.6, 2.0];
const TT4S: &[f64] = &[400.0, 500.0, 600.0, 650.0, 900.0, 1100.0, 1500.0];
const BLEEDS: &[f64] = &[0.00, 0.02, 0.05, 0.10];
const GASES: &[&str] = &["cpg", "tpg", "eq"];
const FLOORS: &[f64] = &[0.50, 0.55];
const GRID41: &[f64] = &[1500.0, 1300.0, 1100.0, 950.0, 900.0, 850.0, 800.0, 750.0, 700.0,
                         650.0, 600.0, 500.0, 400.0];

fn flight_design() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn flight_m16() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 1.60)
}

fn losses() -> TwoSpoolLosses {
    TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    }
}

fn cpg_with(gamma_c: f64, cp_c: f64, gamma_t: f64, cp_t: f64, hpr: f64) -> Gas {
    Gas::new(GasSpec {
        gamma_c, cp_c, r_c: (gamma_c - 1.0) / gamma_c * cp_c,
        gamma_t, cp_t, r_t: (gamma_t - 1.0) / gamma_t * cp_t,
        hpr, ..GasSpec::default()
    })
}

fn gas_for(tag: &str) -> Gas {
    match tag {
        "cpg" => cpg_with(1.4, 1004.0, 1.3, 1239.0, 42.8e6),
        "tpg" => Gas::thermally_perfect(),
        "eq" => Gas::reacting_equilibrium(),
        _ => unreachable!(),
    }
}

fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

/// Rung 41's OWN shape pairs, verbatim from `tests/test_rung41.py`.
fn shapes41() -> Vec<(&'static str, ComponentMap, ComponentMap)> {
    let f = ComponentMap::flat();
    vec![
        ("flow_press", lp_shaped(), hp_shaped()),
        ("press_flow", ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..f },
                       ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }),
        ("tilted", ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f },
                   ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f }),
        ("steep", ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f },
                  ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f }),
    ]
}

/// Abort codes — slice I's table as slice K appended to it, never renumbered. On this grid only
/// 0/2/3/4/5/6/7/8 fire; 9–14 are dumped as explicit zeros, which is § 5.6's P4 discipline and
/// also § 5.8's zero-firing verdict for rung 42's own asserts.
fn abort_code(msg: &str) -> f64 {
    for (tag, code) in [
        ("SUB-IDLE", 1.0),
        ("efficiency cascade", 2.0),
        ("inverse: root not bracketed", 3.0),
        ("equilibrium Newton", 4.0),
        ("off-design burner f did not converge", 5.0),
        ("nozzle back-pressure", 6.0),
        ("ram must not cool/depressurize", 7.0),
        ("UNCHOKED", 8.0),
        ("unphysical", 9.0),
        ("does not straddle", 10.0),
        ("efficiency secant did not converge", 11.0),
        ("turbine-efficiency loop did not converge", 12.0),
        ("speed-line bracket fails", 13.0),
        ("shaft does not close", 14.0),
    ] {
        if msg.contains(tag) {
            return code;
        }
    }
    panic!("UNCLASSIFIED abort, add it to abort_code: {}", &msg[..msg.len().min(140)]);
}

const ALL_CODES: &[f64] = &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0,
                            12.0, 13.0, 14.0];

fn bleed_matcher(tag: &str, ml: ComponentMap, mh: ComponentMap, b: f64) -> TwoSpoolBleedMatcher {
    let d = build_two_spool_turbojet(gas_for(tag), PI_LPC, PI_HPC, TT4, 50_000.0, losses());
    TwoSpoolBleedMatcher::new(d, flight_design(), 1.0, ml, mh, b)
}

fn map_matcher(tag: &str, ml: ComponentMap, mh: ComponentMap) -> TwoSpoolMapMatcher {
    let d = build_two_spool_turbojet(gas_for(tag), PI_LPC, PI_HPC, TT4, 50_000.0, losses());
    TwoSpoolMapMatcher::new(d, flight_design(), 1.0, ml, mh)
}

/// One `flow_coefficient_turn` result — the DISCRIMINANT first, then every field, with the four
/// `RAIL`-nulled ones written as the declared sentinel rather than OMITTED. Omitting them instead
/// would leave the key-count guard blind to a class absent from both sides (`docs`' *a documented
/// gate that doesn't exist*), which is exactly where P9 was written to bite.
fn push_turn(v: &mut Vec<(String, f64)>, tag: &str, t: &FlowTurn) {
    v.push((format!("{tag}/kind"), if t.kind == TurnKind::Min { 0.0 } else { 1.0 }));
    v.push((format!("{tag}/Tt4_star"), t.tt4_star));
    v.push((format!("{tag}/phi_star"), t.phi_star));
    v.push((format!("{tag}/closed_form"), t.closed_form));
    v.push((format!("{tag}/band_lo"), t.band.0));
    v.push((format!("{tag}/band_hi"), t.band.1));
    v.push((format!("{tag}/pi_star"), t.pi_star.unwrap_or(NULL)));
    v.push((format!("{tag}/star_form"), t.star_form.unwrap_or(NULL)));
    v.push((format!("{tag}/gamma_c"), t.gamma_c.unwrap_or(NULL)));
    v.push((format!("{tag}/far"), t.far.unwrap_or(NULL)));
}

/// `surge_margin_schedule` + `running_line_map`, EVERY field of EVERY row plus the LENGTHS.
///
/// The lengths ARE the skip census — the `except AssertionError: continue` is control flow, not
/// error handling. Every field is compared because `running_line_map`'s output feeds nothing
/// downstream: a transposed `x_lp`/`x_hp` or `pi_lpc`/`pi_hpc` pair would be revealed by no other
/// number in the port (§ 5.8.2 (c)).
fn push_schedules(v: &mut Vec<(String, f64)>, tag: &str, core: &TwoSpoolMapCore,
                  fl: &FlightCondition) {
    let sched = core.surge_margin_schedule(fl, GRID41);
    v.push((format!("{tag}/sched/n"), sched.len() as f64));
    for (i, r) in sched.iter().enumerate() {
        for (k, x) in [("Tt4", r.tt4), ("x_lp", r.x_lp), ("x_hp", r.x_hp),
                       ("phi_lp", r.phi_lp), ("phi_hp", r.phi_hp), ("n_lp", r.n_lp),
                       ("n_hp", r.n_hp), ("pi_lpc", r.pi_lpc), ("pi_hpc", r.pi_hpc),
                       ("slip", r.slip), ("SM_lp", r.sm_lp), ("SM_hp", r.sm_hp)] {
            v.push((format!("{tag}/sched/{i}/{k}"), x));
        }
        v.push((format!("{tag}/sched/{i}/binding"),
                if r.binding == Spool::Lp { 0.0 } else { 1.0 }));
    }
    let rl = core.running_line_map(fl, GRID41);
    v.push((format!("{tag}/runline/n"), rl.len() as f64));
    for (i, r) in rl.iter().enumerate() {
        for (k, x) in [("Tt4", r.tt4), ("x_lp", r.x_lp), ("x_hp", r.x_hp),
                       ("phi_lp", r.phi_lp), ("phi_hp", r.phi_hp), ("n_lp", r.n_lp),
                       ("n_hp", r.n_hp), ("pi_lpc", r.pi_lpc), ("pi_hpc", r.pi_hpc)] {
            v.push((format!("{tag}/runline/{i}/{k}"), x));
        }
    }
}

fn rust_values() -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = Vec::new();
    let flat = ComponentMap::flat();

    // === 1. rung 41's ZERO-NEW-CONSTANT closed form, and `phi_surge`'s arrival ==============
    for &g in GASES {
        v.push((format!("pistar/{g}"),
                map_matcher(g, flat, flat).core().critical_flow_turn_pi()));
    }
    for (i, gc) in [1.30, 1.35, 1.40, 1.45].into_iter().enumerate() {
        let d = build_two_spool_turbojet(cpg_with(gc, 1004.0, 1.3, 1239.0, 42.8e6),
                                         PI_LPC, PI_HPC, TT4, 50_000.0, losses());
        let m = TwoSpoolMapMatcher::new(d, flight_design(), 1.0, flat, flat);
        v.push((format!("pistar/gamma_c/{i}"), m.core().critical_flow_turn_pi()));
    }
    let bare = lp_shaped();
    let armed = ComponentMap { phi_surge: 0.55, ..bare };
    v.push(("phi_surge/carried".to_string(), armed.phi_surge));
    for (j, phi) in [0.55, 0.85, 1.0, 1.2].into_iter().enumerate() {
        v.push((format!("phi_surge/psi_bare/{j}"), bare.psi(phi)));
        v.push((format!("phi_surge/psi_armed/{j}"), armed.psi(phi)));
    }

    // === 2. rung 42's grid — 147 cells PER BLEED LEVEL, on the `mixed` pair =================
    for &b in BLEEDS {
        let mut census: HashMap<u64, usize> = HashMap::new();
        for &g in GASES {
            let m = bleed_matcher(g, lp_shaped(), hp_shaped(), b);
            for &m0 in M0S {
                let fl = FlightCondition::new(250.0, 50_000.0, m0);
                for &tt4 in TT4S {
                    let tag = format!("r42/{b:.2}/{g}/{m0:.2}/{tt4:.0}");
                    counters::reset();
                    let od = match m.try_match_point(&fl, tt4) {
                        Ok(od) => od,
                        Err(e) => {
                            let code = abort_code(&e.0);
                            v.push((format!("{tag}/abort"), code));
                            *census.entry(code.to_bits()).or_insert(0) += 1;
                            continue;
                        }
                    };
                    v.push((format!("{tag}/abort"), 0.0));
                    *census.entry(0.0f64.to_bits()).or_insert(0) += 1;
                    // THE PASS COUNT — bit-gated against PyPy only (§ 5.7 (c)).
                    v.push((format!("{tag}/n_pass"), counters::cascade_calls() as f64));
                    let base = &od.base.base;
                    let perf = &base.performance;
                    for (k, x) in [("pi_lpc", base.pi_lpc), ("pi_hpc", base.pi_hpc),
                                   ("eta_lpc", od.base.eta_lpc), ("eta_hpc", od.base.eta_hpc),
                                   ("eta_hpt", od.base.eta_hpt), ("eta_lpt", od.base.eta_lpt),
                                   ("n_lp", od.base.n_lp), ("n_hp", od.base.n_hp),
                                   ("N_lp_ratio", od.base.n_lp_ratio),
                                   ("N_hp_ratio", od.base.n_hp_ratio),
                                   ("slip", od.base.slip), ("phi_lp", od.base.phi_lp),
                                   ("phi_hp", od.base.phi_hp), ("nu_hpt", od.base.nu_hpt),
                                   ("nu_lpt", od.base.nu_lpt), ("tau_hpt", base.tau_hpt),
                                   ("tau_lpt", base.tau_lpt), ("tau_lpc", base.tau_lpc),
                                   ("tau_hpc", base.tau_hpc), ("pi_hpt", base.pi_hpt),
                                   ("pi_lpt", base.pi_lpt), ("mdot_air", base.mdot_air),
                                   ("mdot_ratio", base.mdot_ratio), ("thrust", base.thrust),
                                   ("V0", base.v0), ("V9", base.v9), ("M9", base.m9),
                                   ("T9", base.t9), ("p9", base.p9),
                                   ("F_over_mdot", perf.specific_thrust), ("tsfc", perf.tsfc),
                                   ("eta_th", perf.eta_thermal),
                                   ("eta_p", perf.eta_propulsive)] {
                        v.push((format!("{tag}/{k}"), x));
                    }
                    for st in ["2", "25", "3", "4", "45", "5", "9"] {
                        let s = base.station(st);
                        v.push((format!("{tag}/s{st}/Tt"), s.tt));
                        v.push((format!("{tag}/s{st}/pt"), s.pt));
                    }
                    // `mdot` IS the extraction's only visible trace — `try_score` never touches
                    // mass flow, so no downstream number would reveal a wrong split.
                    for st in ["2", "25", "3", "4"] {
                        v.push((format!("{tag}/s{st}/mdot"), base.station(st).mdot));
                    }
                    v.push((format!("{tag}/s4/far"), base.station("4").far));
                    // THE BOOKING, as a discriminant plus the declared sentinel.
                    v.push((format!("{tag}/booking_absent"),
                            if od.booking.is_none() { 1.0 } else { 0.0 }));
                    for (k, x) in [
                        ("bleed", od.booking.map(|k| k.bleed).unwrap_or(NULL)),
                        ("mdot_core", od.booking.map(|k| k.mdot_core).unwrap_or(NULL)),
                        ("st_inlet", od.booking.map(|k| k.st_inlet).unwrap_or(NULL)),
                        ("tsfc_inlet", od.booking.map(|k| k.tsfc_inlet).unwrap_or(NULL))] {
                        v.push((format!("{tag}/{k}"), x));
                    }
                }
            }
        }
        for &code in ALL_CODES {
            v.push((format!("census/r42/{b:.2}/abort_code/{code:.0}"),
                    *census.get(&code.to_bits()).unwrap_or(&0) as f64));
        }
    }

    // === 3. rung 41's schedules on RUNG-39 matchers ==========================================
    for &g in GASES {
        for (sname, ml, mh) in shapes41() {
            for &fl_v in FLOORS {
                let m = map_matcher(g, ComponentMap { phi_surge: fl_v, ..ml },
                                    ComponentMap { phi_surge: fl_v, ..mh });
                let tag = format!("r41/{g}/{sname}/{fl_v:.2}");
                push_schedules(&mut v, &tag, m.core(), &flight_design());
                // Gate 2's non-tautological reproduction, per cell at one throttle. Python
                // `continue`s when this match aborts, so the two keys are ABSENT there — the
                // Rust must skip on exactly the same condition or the key counts diverge.
                let Ok(od) = m.core().try_match_point(&flight_design(), 1100.0) else { continue };
                v.push((format!("{tag}/pi_shipped_lp"),
                        m.core().pi_c_spool_shipped(&od, Spool::Lp)));
                v.push((format!("{tag}/pi_shipped_hp"),
                        m.core().pi_c_spool_shipped(&od, Spool::Hp)));
            }
        }
    }

    // === 4. THE NARROWING FIX — all three rung-41 methods on a RUNG-42 core, at b > 0 ========
    //
    // The cells sit at the sweep's EDGES on purpose: § 5.8.3 (h) measured the check's sensitivity
    // to a mis-associated `(1-b)` at ~2 % of rows, and the rows that moved were at M0 = 1.60 or
    // on the equilibrium gas. A comfortable mid-band cell would have passed the defect.
    let mut n42 = (0usize, 0usize);
    let mut nflat = (0usize, 0usize);
    let mut nflat_lp_rail = 0usize;
    let mut n_ended_on_abort = 0usize;
    let mut n_turns = 0usize;
    for &g in GASES {
        for &b in [0.0, 0.10].iter() {
            for (flname, fl) in [("0.85", flight_design()), ("1.60", flight_m16())] {
                let m = bleed_matcher(g, ComponentMap { phi_surge: 0.55, ..lp_shaped() },
                                      ComponentMap { phi_surge: 0.55, ..hp_shaped() }, b);
                push_schedules(&mut v, &format!("r42sched/{g}/{b:.2}/{flname}"), &m.core, &fl);
            }
        }
        for &b in [0.0, 0.10].iter() {
            for spool in [Spool::Hp, Spool::Lp] {
                let m = bleed_matcher(g, lp_shaped(), hp_shaped(), b);
                let t = m.core.flow_coefficient_turn(&flight_design(), spool);
                let sp = if spool == Spool::Hp { "hp" } else { "lp" };
                push_turn(&mut v, &format!("r42turn/{g}/{b:.2}/{sp}"), &t);
                if t.kind == TurnKind::Min { n42.0 += 1 } else { n42.1 += 1 }
                n_turns += 1;
                if t.band.0 > 350.0 + 10.0 { n_ended_on_abort += 1 }
            }
        }
    }
    for &b in [0.0, 0.10].iter() {
        for spool in [Spool::Hp, Spool::Lp] {
            let m = bleed_matcher("cpg", lp_shaped(), hp_shaped(), b);
            let t = m.core.flow_coefficient_turn(&flight_m16(), spool);
            let sp = if spool == Spool::Hp { "hp" } else { "lp" };
            push_turn(&mut v, &format!("r42turn/cpg/M16/{b:.2}/{sp}"), &t);
            if t.kind == TurnKind::Min { n42.0 += 1 } else { n42.1 += 1 }
            n_turns += 1;
            if t.band.0 > 350.0 + 10.0 { n_ended_on_abort += 1 }
        }
    }

    // `bleed_trade` — the path where the dispatch was ALREADY witnessed (through `surge_margin`).
    // At `b = 0` the row reads the CORE numbers through the ABSENT booking, which is Python's
    // `getattr` fallback expressed as data.
    for &g in GASES {
        let mut m = bleed_matcher(g, ComponentMap { phi_surge: 0.55, ..lp_shaped() },
                                  ComponentMap { phi_surge: 0.55, ..hp_shaped() }, 0.0);
        for tt4 in [1500.0, 1300.0, 1100.0] {
            let rows = m.bleed_trade(&flight_design(), tt4, &[0.0, 0.05, 0.10]);
            v.push((format!("trade/{g}/{tt4:.0}/n"), rows.len() as f64));
            for (i, r) in rows.iter().enumerate() {
                for (k, x) in [("bleed", r.bleed), ("Tt4", r.tt4), ("phi_lp", r.phi_lp),
                               ("phi_hp", r.phi_hp), ("n_lp", r.n_lp), ("n_hp", r.n_hp),
                               ("pi_lpc", r.pi_lpc), ("pi_hpc", r.pi_hpc), ("Tt25", r.tt25),
                               ("slip", r.slip), ("mdot_air", r.mdot_air),
                               ("thrust", r.thrust), ("st_inlet", r.st_inlet),
                               ("tsfc", r.tsfc),
                               ("SM_lp", r.sm_lp.expect("both maps armed")),
                               ("SM_hp", r.sm_hp.expect("both maps armed"))] {
                    v.push((format!("trade/{g}/{tt4:.0}/{i}/{k}"), x));
                }
            }
            // the valve is RESTORED — a port that mutates and forgets leaves every later
            // reading on the wrong machine.
            v.push((format!("trade/{g}/{tt4:.0}/bleed_after"), m.bleed()));
        }
    }

    // === 5. `flow_coefficient_turn` on FLAT maps — gate 5's OWN 19 cases x both spools =======
    //
    // Gate 5 only ever calls the `hp` spool; the `lp` column is WIDER than any shipped gate,
    // deliberately, because P9's `RAIL` branch is where the LP spool normally LIVES (16 of 19).
    let mut cases: Vec<(String, Gas, f64, f64, TwoSpoolLosses, FlightCondition)> = vec![
        ("base".into(), gas_for("cpg"), PI_LPC, PI_HPC, losses(), flight_design()),
        ("split_4.5x4".into(), gas_for("cpg"), 4.5, 4.0, losses(), flight_design()),
        ("split_2.25x8".into(), gas_for("cpg"), 2.25, 8.0, losses(), flight_design()),
        ("eta_hpc_.80".into(), gas_for("cpg"), PI_LPC, PI_HPC,
         TwoSpoolLosses { eta_hpc: 0.80, ..losses() }, flight_design()),
        ("eta_hpc_.95".into(), gas_for("cpg"), PI_LPC, PI_HPC,
         TwoSpoolLosses { eta_hpc: 0.95, ..losses() }, flight_design()),
        ("eta_hpt_.85".into(), gas_for("cpg"), PI_LPC, PI_HPC,
         TwoSpoolLosses { eta_hpt: 0.85, ..losses() }, flight_design()),
        ("eta_lpc_.80".into(), gas_for("cpg"), PI_LPC, PI_HPC,
         TwoSpoolLosses { eta_lpc: 0.80, ..losses() }, flight_design()),
    ];
    for gc in [1.30, 1.35, 1.40, 1.45] {
        cases.push((format!("gamma_c_{gc}"), cpg_with(gc, 1004.0, 1.3, 1239.0, 42.8e6),
                    PI_LPC, PI_HPC, losses(), flight_design()));
    }
    cases.push(("gamma_t_1.25".into(), cpg_with(1.4, 1004.0, 1.25, 1239.0, 42.8e6),
                PI_LPC, PI_HPC, losses(), flight_design()));
    cases.push(("cp_t_1300".into(), cpg_with(1.4, 1004.0, 1.3, 1300.0, 42.8e6),
                PI_LPC, PI_HPC, losses(), flight_design()));
    for (i, hpr) in [4.28e8, 4.28e9, 4.28e10].into_iter().enumerate() {
        cases.push((format!("hPR_{i}"), cpg_with(1.4, 1004.0, 1.3, 1239.0, hpr),
                    PI_LPC, PI_HPC, losses(), flight_design()));
    }
    cases.push(("M0_1.60".into(), gas_for("cpg"), PI_LPC, PI_HPC, losses(), flight_m16()));
    cases.push(("tpg".into(), Gas::thermally_perfect(), PI_LPC, PI_HPC, losses(),
                flight_design()));
    cases.push(("tpg_M0_1.60".into(), Gas::thermally_perfect(), PI_LPC, PI_HPC, losses(),
                flight_m16()));

    let n_cases = cases.len();
    for (nm, gas, pl, ph, lo, fl) in cases {
        for spool in [Spool::Hp, Spool::Lp] {
            // NOTE the matcher is built at the DESIGN flight and matched at `fl` — gate 5's own
            // construction, copied rather than tidied.
            let d = build_two_spool_turbojet(gas.clone(), pl, ph, TT4, 50_000.0, lo);
            let m = TwoSpoolMapMatcher::new(d, flight_design(), 1.0, flat, flat);
            let t = m.core().flow_coefficient_turn(&fl, spool);
            let sp = if spool == Spool::Hp { "hp" } else { "lp" };
            push_turn(&mut v, &format!("turn/{nm}/{sp}"), &t);
            if t.kind == TurnKind::Min { nflat.0 += 1 } else { nflat.1 += 1 }
            if spool == Spool::Lp && t.kind == TurnKind::Rail { nflat_lp_rail += 1 }
            n_turns += 1;
            if t.band.0 > 350.0 + 10.0 { n_ended_on_abort += 1 }
        }
    }
    assert_eq!(n_cases, 19, "gate 5's own case set is 19");

    // P9's branch COUNTS, PER BLOCK — § 5.8.1 (viii)'s registered numbers are two different
    // populations ("60 MIN / 20 RAIL over the shaped grid", "16 of 19 lp RAIL on the flat one"),
    // and one pooled counter answers neither.
    v.push(("census/turn/n".into(), n_turns as f64));
    v.push(("census/turn42/MIN".into(), n42.0 as f64));
    v.push(("census/turn42/RAIL".into(), n42.1 as f64));
    v.push(("census/turnflat/MIN".into(), nflat.0 as f64));
    v.push(("census/turnflat/RAIL".into(), nflat.1 as f64));
    v.push(("census/turnflat/lp_RAIL".into(), nflat_lp_rail as f64));
    // P3's SECOND half, as data: the coarse scan always ends on the ABORT, never on `Tt4_lo`.
    v.push(("census/turn/ended_on_abort".into(), n_ended_on_abort as f64));

    // === 6. THE MEMO KEY SEQUENCE (P4) ======================================================
    for (tag, g, b, spool) in [("r39_hp", "cpg", 0.0, Spool::Hp),
                               ("r39_lp", "cpg", 0.0, Spool::Lp),
                               ("r42_hp", "cpg", 0.10, Spool::Hp),
                               ("r42_lp", "cpg", 0.10, Spool::Lp)] {
        let m = bleed_matcher(g, lp_shaped(), hp_shaped(), b);
        counters::reset();
        let t = m.core.flow_coefficient_turn(&flight_design(), spool);
        let keys = counters::memo_keys();
        v.push((format!("keys/{tag}/kind"), if t.kind == TurnKind::Min { 0.0 } else { 1.0 }));
        v.push((format!("keys/{tag}/n"), keys.len() as f64));
        for (i, k) in keys.iter().enumerate() {
            v.push((format!("keys/{tag}/{i}"), *k));
        }
    }

    v
}

/// A key's quantity class, which decides its bar.
fn quant_of(key: &str) -> &'static str {
    if key.ends_with("/n_pass") {
        return "loopcount";
    }
    // The memo keys are THROTTLES, not counts — but they are also `round(x, 6)` outputs, so they
    // are exact decimals and any drift is a CHANGED KEY rather than a last-bit one. Classed as a
    // value so the CPython arm can report a branch flip instead of failing the whole compare.
    if key.contains("/keys/") || key.starts_with("keys/") {
        return if key.ends_with("/n") || key.ends_with("/kind") { "discrete" } else { "value" };
    }
    if key.ends_with("/abort") || key.ends_with("/kind") || key.ends_with("/binding")
        || key.ends_with("/booking_absent") || key.ends_with("/sched/n")
        || key.ends_with("/runline/n") || key.ends_with("/n") || key.starts_with("census/") {
        return "discrete";
    }
    // THE LOCATION OF A FLAT EXTREMUM IS ITS OWN QUANTITY CLASS — see the header's § on the turn.
    // `flow_coefficient_turn` golden-sections to an interior MAXIMUM, so these four are read AT a
    // point where the objective's slope is zero and last-bit noise buys a first-order move in the
    // abscissa. `phi_star` — the objective's VALUE there — is deliberately NOT in this list: it is
    // the tight half of the pair, and keeping it at the `value` bar is what makes the split
    // measurable rather than asserted.
    if key.starts_with("turn/") || key.starts_with("r42turn/") {
        if key.ends_with("/Tt4_star") || key.ends_with("/far") || key.ends_with("/pi_star")
            || key.ends_with("/star_form") {
            return "location";
        }
    }
    "value"
}

/// The bars. **Measured on this dump, then written.**
fn bar_for(quant: &str, strict: bool) -> f64 {
    match quant {
        "discrete" => 0.0,
        // THE JOINT LOOP'S PASS COUNT IS NOT INTERPRETER-INVARIANT — § 5.7 (c). Bit-gated on
        // PyPy; on CPython it is reported and cross-checked below, never silently skipped.
        "loopcount" => if strict { 0.0 } else { f64::INFINITY },
        // MEASURED, NOT CHOSEN: worst 1.58e-5 over the 54 turn cells (7 of which move at all).
        // A decade of headroom, and the honest reading of that number is in the header — at 1e-4
        // this class is NOT meaningfully gated against CPython, and says so out loud. Its gate is
        // the PyPy bit-equality arm, which holds it to zero like everything else.
        "location" => if strict { 0.0 } else { 1.0e-4 },
        _ => 1.0e-8,
    }
}

fn compare_against(oracle_text: &str, label: &str, require_bit_exact: bool) {
    let oracle = load_oracle(oracle_text);
    let ours = rust_values();
    println!("\n=== Rust vs {label} ===");
    assert_eq!(ours.len(), oracle.len(),
               "key COUNT differs: rust {} vs oracle {} — the dump and the gate have drifted \
                apart. On THIS slice that most likely means a cell aborted on one side and \
                matched on the other, or a schedule SKIPPED a different number of throttles, \
                both of which are findings rather than bookkeeping slips.",
               ours.len(), oracle.len());

    let mut missing: Vec<&str> = Vec::new();
    let mut per: HashMap<&str, (usize, usize, f64, String)> = HashMap::new();
    let mut failures: Vec<String> = Vec::new();

    // The cells whose joint loop ran a different number of passes on this interpreter. Their value
    // keys answer to a bar one decade wider — and that decade is MEASURED, in the flip section
    // below, against the cells that did not flip. Hoisted above the main loop only so the two
    // populations are held to the same numbers in the per-key report and in the assertion.
    let flip_cells: Vec<String> = ours.iter()
        .filter(|(k, got)| quant_of(k) == "loopcount"
                && oracle.get(k.as_str()).map(|w| w.to_bits() != got.to_bits()).unwrap_or(false))
        .map(|(k, _)| k[..k.len() - "n_pass".len()].to_string())
        .collect();
    let bar_of = |key: &str, q: &str| -> f64 {
        if !require_bit_exact && q == "value"
            && flip_cells.iter().any(|c| key.starts_with(c.as_str())) {
            return 1.0e-7;
        }
        bar_for(q, require_bit_exact)
    };

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
        let dev = if scale == 0.0 { (got - want).abs() } else { (got - want).abs() / scale };
        if dev > e.2 {
            e.2 = dev;
            e.3 = key.clone();
        }
        if dev > bar_of(key, q) {
            failures.push(format!(
                "  {key:<56} rust {got:.17e}  oracle {want:.17e}  dev {dev:.2e}"));
        }
    }

    let mut rows: Vec<_> = per.iter().collect();
    rows.sort_by(|a, b| b.1 .2.partial_cmp(&a.1 .2).unwrap());
    println!("\n{:<12} {:>6} {:>11} {:>12} {:>12}",
             "quantity", "keys", "bit-exact", "worst dev", "base bar");
    for (q, (n, exact, worst, _)) in &rows {
        println!("{:<12} {:>6} {:>11} {:>12.2e} {:>12.0e}",
                 q, n, exact, worst, bar_for(q, require_bit_exact));
    }
    let total: usize = rows.iter().map(|r| r.1 .0).sum();
    let exact: usize = rows.iter().map(|r| r.1 .1).sum();
    println!("\n{exact} / {total} bit-identical to {label} ({:.2}%)",
             100.0 * exact as f64 / total as f64);
    for (q, (_, _, worst, key)) in &rows {
        if *worst > 0.0 {
            println!("  worst {q:<12} {worst:.2e}  at {key}");
        }
    }

    // THE LOOP-COUNT DISAGREEMENT, REPORTED RATHER THAN SKIPPED — and the invariant that survives
    // it, asserted. Slice K CORRECTED slice I's stated reason here: the flips are not confined to
    // the equilibrium gas but to gases that reach `cp` through an INTEGRAL and a root-find, so
    // the true invariant is that no CALORICALLY-PERFECT cell may flip.
    let flips: Vec<&String> = ours.iter()
        .filter(|(k, got)| quant_of(k) == "loopcount"
                && oracle.get(k.as_str()).map(|w| w.to_bits() != got.to_bits()).unwrap_or(false))
        .map(|(k, _)| k)
        .collect();
    let n_loopcount = ours.iter().filter(|(k, _)| quant_of(k) == "loopcount").count();
    println!("\nloop-count cells disagreeing with {label}: {} / {n_loopcount}", flips.len());
    let cpg_flips: Vec<&&String> = flips.iter().filter(|k| k.contains("/cpg/")).collect();
    assert!(cpg_flips.is_empty(),
            "a CALORICALLY-PERFECT cell flipped its pass count. Every property on that gas is a \
             closed form, so there is no root-find for last-bit arithmetic to tip — a flip here \
             is a different phenomenon entirely: {cpg_flips:?}");
    // WHAT A FLIP COSTS, MEASURED AGAINST THE CELLS THAT DID NOT FLIP. The first version of this
    // check held a flipped cell's values to the ordinary 1e-8 value bar and fired — which proved
    // only that 1e-8 had been COPIED from slice K rather than measured here. Both populations are
    // now measured, and the content is the SEPARATION between them: a flip costs one decade, and
    // a cell that did not flip stays a decade inside the bar. Either half moving is a finding.
    let unflipped_cells: Vec<String> = ours.iter()
        .filter(|(k, got)| quant_of(k) == "loopcount"
                && oracle.get(k.as_str()).map(|w| w.to_bits() == got.to_bits()).unwrap_or(false))
        .map(|(k, _)| k[..k.len() - "n_pass".len()].to_string())
        .collect();
    let worst_in = |cells: &[String]| -> (f64, String) {
        let mut worst = (0.0f64, String::new());
        for (vk, got) in ours.iter().filter(|(vk, _)| quant_of(vk) == "value") {
            if !cells.iter().any(|c| vk.starts_with(c.as_str())) {
                continue;
            }
            let want = oracle[vk.as_str()];
            let scale = got.abs().max(want.abs());
            let dev = if scale == 0.0 { (got - want).abs() } else { (got - want).abs() / scale };
            if dev > worst.0 {
                worst = (dev, vk.clone());
            }
        }
        worst
    };
    let (w_flip, k_flip) = worst_in(&flip_cells);
    let (w_keep, k_keep) = worst_in(&unflipped_cells);
    println!("  worst value in a FLIPPED cell     {w_flip:.2e}  at {k_flip}");
    println!("  worst value in an UNFLIPPED cell  {w_keep:.2e}  at {k_keep}");
    if !require_bit_exact {
        assert!(w_flip <= 1.0e-7,
                "a pass-count flip moved a value by {w_flip:.2e} at {k_flip} — measured at 1.55e-8 \
                 when this bar was written, so a decade of headroom is already spent");
        assert!(w_keep <= 1.0e-8,
                "a cell that ran the SAME number of passes still moved a value by {w_keep:.2e} at \
                 {k_keep} — measured at 1.03e-9. The flip is then not what is driving the drift, \
                 and this whole section is measuring the wrong thing");
        assert!(w_keep < w_flip,
                "the two populations no longer separate ({w_keep:.2e} unflipped vs {w_flip:.2e} \
                 flipped): a pass-count flip is supposed to COST something");
    }

    // THE FLAT EXTREMUM, BOTH HALVES, IN THE SAME CELLS. `Tt4_star` is WHERE the turn is and
    // `phi_star` is WHAT it is worth; they come out of one golden section, so any difference
    // between them is conditioning rather than arithmetic. Measured here: the location moves
    // 10^4-10^6 times as far as the value it locates.
    let dev_of = |k: &str| -> f64 {
        let got = ours.iter().find(|(kk, _)| kk == k).map(|(_, v)| *v).unwrap();
        let want = oracle[k];
        let scale = got.abs().max(want.abs());
        if scale == 0.0 { (got - want).abs() } else { (got - want).abs() / scale }
    };
    let turn_cells: Vec<String> = ours.iter()
        .filter(|(k, _)| k.ends_with("/Tt4_star"))
        .map(|(k, _)| k[..k.len() - "/Tt4_star".len()].to_string())
        .collect();
    let mut moved = 0usize;
    let mut worst_ratio = 0.0f64;
    for c in &turn_cells {
        let (dloc, dval) = (dev_of(&format!("{c}/Tt4_star")), dev_of(&format!("{c}/phi_star")));
        if dloc == 0.0 {
            continue;
        }
        moved += 1;
        // A CALORICALLY-PERFECT cell has no root-find under the objective, so there is no noise
        // for the flatness to amplify. Same invariant slice K found on the pass count, now on a
        // second, unrelated phenomenon.
        assert!(!c.contains("/cpg"),
                "a CALORICALLY-PERFECT turn moved its LOCATION at {c}: {dloc:.2e}");
        if !require_bit_exact {
            assert!(dloc > 1.0e3 * dval.max(1.0e-16),
                    "the turn at {c} moved its LOCATION by {dloc:.2e} and its VALUE by {dval:.2e} \
                     — under 10^3 apart, so the flat-extremum reading of this drift is wrong");
        }
        worst_ratio = worst_ratio.max(dloc / dval.max(1.0e-16));
    }
    println!("\nturn cells whose LOCATION moved: {moved} / {}  \
              (worst location:value conditioning {worst_ratio:.1e})", turn_cells.len());
    if !require_bit_exact {
        assert_eq!(moved, 7,
                   "the count of turn cells that disagree with CPython moved from 7 — the sweep, \
                    the solver tolerance, or the objective's flatness has changed");
    }

    assert!(missing.is_empty(), "keys computed by Rust but absent from the oracle: {missing:?}");
    assert!(failures.is_empty(), "{} value(s) outside the measured bar:\n{}",
            failures.len(), failures.join("\n"));
    if require_bit_exact {
        let drifted: Vec<&String> = rows.iter()
            .filter(|(_, (_, _, w, _))| *w > 0.0)
            .map(|(_, (_, _, _, k))| k)
            .collect();
        assert_eq!(exact, total,
                   "the PyPy arm is held to BIT-EQUALITY and {} value(s) drifted; \
                    worst keys: {:?}", total - exact, drifted);
    }
}

/// THE GATE. PyPy is the project's gate interpreter, so this arm is held to bit-equality.
#[test]
fn slice_l_matches_pypy_bit_for_bit() {
    compare_against(ORACLE_PYPY, "PyPy", true);
}

/// The CPython arm — a SANITY CHECK, not the gate.
#[test]
fn slice_l_matches_cpython_within_measured_bars() {
    compare_against(ORACLE_CPYTHON, "CPython", false);
}
