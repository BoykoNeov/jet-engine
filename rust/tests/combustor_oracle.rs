//! SLICE Q step 3 — the rung-37 ORACLE. Every value `oracle/dump_combustor.py` produces,
//! recomputed here and compared BIT for bit.
//!
//! # Why an oracle, when ten gates already pass
//!
//! Step 2 measured it rather than argued it. Three defects were injected into the shipped port
//! and run against the seven ported Python gates plus the three the port adds:
//!
//! | injected defect | smoke values (of 517) | Python gates (of 7) | added gates (of 3) |
//! |---|---|---|---|
//! | the Illinois exhaustion arm returns `a` | 97 | **0** | **0** |
//! | the plenum power block copied from `_instant_tail` | 60 | **0** | **0** |
//! | `equilibrium_soak`'s two loops UNIFIED | 29 | **0** | **0** |
//!
//! Every gate rung 37 ships is written at `1e-6`–`1e-9`, where its physics lives, and all three
//! defects sit three or more orders below that. The middle row is the sharpest: the difference
//! between the honest two-mass-flow power and a per-unit-air copy is
//! `eta_m*dh_t*(mdot_ngv - mdot_c*(1+f))`, and `Phi` is read at exactly ONE site — the residual of
//! the very condition `mdot_c + mdot_fuel = mdot_ngv` that makes the bracket vanish.
//!
//! **It vanishes to the mass balance's own RESIDUAL, not to zero, and that distinction is
//! measured rather than asserted.** The equilibrium closes the balance to ~1e-12 relative, so the
//! injection moves the plenum equilibrium by **5.4e-12 in `nu`** and **9.1e-12 in `pi_c`** —
//! three orders below gate 2's `1e-9` bar, which is why it fails no gate, and 104 section-C keys
//! plus 30 section-B ones, which is why it fails the dump. The first draft of this paragraph said
//! the two formulas "cannot differ" where `Phi` is read; they differ by the residual, and stating
//! the size is both stronger and true.
//!
//! # The three census families, and what each is for
//!
//! * `illinois_exhausted` — slice P shipped `try_illinois`'s `Ok(b)` arm with ZERO firings and
//!   could only close the blind spot with a counter. Here `_plenum_pt4_at` passes `N_TOL = 1e-12`
//!   as an ABSOLUTE bracket width on a `pt4` of order 1e5 Pa, and the arm becomes the path most
//!   of that site's calls take.
//! * the per-call-site counts — rung 37's three marches have no `try`, so a failing stage aborts
//!   the whole call rather than shortening it. An evaluation count that reproduces exactly is
//!   therefore the certificate that every step ran, which is the only form the reachability
//!   claim can take when nothing fails.
//! * `subsonic_raises` / `subsonic_escalations` — slice P's two rarest branches, dumped here to
//!   show they are STRUCTURALLY unreachable from the plenum path rather than merely absent.
//!
//! Regenerate with:
//!     .venv\Scripts\python.exe rust\oracle\dump_combustor.py rust\oracle\combustor_pypy.tsv
//!     C:\Python314\python.exe  rust\oracle\dump_combustor.py rust\oracle\combustor_cpython.tsv

use std::collections::BTreeMap;

use turbojet::combustor::{counters as ccount, CombustorTransient, Theta0};
use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::Gas;
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::spool::{counters as scount, Instant, SpoolTransient};

// ------------------------------------------------------------------------------ the grid
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

fn build(cmap: ComponentMap, r_v: f64, g: f64, r_m: f64) -> CombustorTransient {
    CombustorTransient::new(design(), flight(), 1.0, cmap, r_v, g, r_m)
}

fn shapes() -> [(&'static str, ComponentMap); 3] {
    [
        ("flow", ComponentMap::surge_flow()),
        ("press", ComponentMap::surge_pressure()),
        ("tilt", ComponentMap::surge_tilted()),
    ]
}

/// Every numeric label is a LITERAL string, never a formatted `f64`. Python's `f"{1.0}"` is
/// `"1.0"` and Rust's is `"1"`, so formatting the value would silently produce a disjoint key set
/// — which the key-set diff would catch, but as a coverage failure rather than as what it is.
const NUS: [(&str, f64); 4] = [("0.7", 0.7), ("0.85", 0.85), ("1.0", 1.0), ("1.3", 1.3)];
const RVS: [(&str, f64); 2] = [("0.03", 0.03), ("0.1", 0.1)];
const GAINS: [(&str, f64); 2] = [("0.05", 0.05), ("0.15", 0.15)];
const RMS: [(&str, f64); 2] = [("1.0", 1.0), ("5.0", 5.0)];

// ------------------------------------------------------------------------------ the harness
fn load(path: &str) -> BTreeMap<String, u64> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{path}: {e} — regenerate with oracle/dump_combustor.py"));
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, b) = (it.next().unwrap(), it.next().unwrap());
        assert!(m.insert(k.to_string(), b.parse::<u64>().unwrap()).is_none(), "dup {k}");
    }
    m
}

#[derive(Default)]
struct Dump {
    v: BTreeMap<String, u64>,
}

impl Dump {
    fn put(&mut self, key: String, value: f64) {
        assert!(value.is_finite(), "{key} is not finite: {value}");
        assert!(self.v.insert(key.clone(), value.to_bits()).is_none(), "dup {key}");
    }
    fn putd(&mut self, key: String, n: u64) {
        assert!(self.v.insert(key.clone(), n).is_none(), "dup {key}");
    }
    fn instant(&mut self, prefix: &str, i: &Instant) {
        for (k, v) in [
            ("nu", i.nu), ("Tt4", i.tt4), ("pi_c", i.pi_c), ("tau_c", i.tau_c),
            ("eta_c", i.eta_c), ("eta_t", i.eta_t), ("m", i.m), ("n", i.n),
            ("flowcoef", i.flowcoef), ("mdot_air", i.mdot_air), ("f", i.f), ("pi_t", i.pi_t),
            ("tau_t", i.tau_t), ("Tt3", i.tt3), ("Tt5", i.tt5), ("nu_t", i.nu_t),
            ("p_net_spec", i.p_net_spec), ("Phi", i.phi), ("sp_thrust", i.sp_thrust),
            ("thrust", i.thrust), ("M9", i.m9), ("pt9_over_p0", i.pt9_over_p0),
        ] {
            self.put(format!("{prefix}{k}"), v);
        }
    }
    fn census(&mut self, prefix: &str) {
        let s = scount::take();
        let c = ccount::take();
        let arms = turbojet::map::take_phi_max_arms(); // [flat5, quadratic, linear, swirled]
        for (k, v) in [
            ("phi_max_flat5", arms[0]),
            ("phi_max_quadratic", arms[1]),
            ("phi_max_linear", arms[2]),
            ("phi_max_swirled", arms[3]),
            ("illinois_calls", s.illinois_calls),
            ("illinois_evals", s.illinois_evals),
            ("illinois_exhausted", s.illinois_exhausted),
            ("r34_solve_turbine", s.r34_solve_turbine),
            ("subsonic_raises", s.subsonic_fallbacks + s.subsonic_escalations),
            ("subsonic_escalations", s.subsonic_escalations),
            ("backpressure_calls", c.backpressure_calls),
            ("backpressure_bracket_fails", c.backpressure_bracket_fails),
            ("pt4_at_calls", c.pt4_at_calls),
            ("pt4_at_bracket_fails", c.pt4_at_bracket_fails),
            ("pt4_at_floor_fails", c.pt4_at_floor_fails),
            ("soak_close_calls", c.soak_close_calls),
            ("soak_close_bracket_fails", c.soak_close_bracket_fails),
            ("plenum_state_calls", c.plenum_state_calls),
            ("instant_soak_calls", c.instant_soak_calls),
        ] {
            self.putd(format!("census/{prefix}/{k}"), v);
        }
    }
}

/// Recompute the whole dump. ONE function, because the census counters are thread-locals that
/// `take()` resets, so two tests building sections concurrently would steal each other's tallies.
fn rust_values() -> Dump {
    let fl = flight();
    let mut d = Dump::default();
    let _ = scount::take();
    let _ = ccount::take();

    // ============================ A: the speed line, and BOTH arms of the flow ceiling
    for (sh, cmap) in shapes() {
        let ct = build(cmap, 0.05, 0.0, 0.0);
        for (nulab, nu) in NUS {
            let (tt2, _, n, _) = ct.face(&fl, nu);
            let band = ct.pic_band(&cmap, n, tt2);
            let tag = format!("A/{sh}/{nulab}");
            d.put(format!("{tag}/m_lo"), band.m_lo);
            d.put(format!("{tag}/pic_max"), band.pic_max);
            d.put(format!("{tag}/m_hi"), band.m_hi);
            d.put(format!("{tag}/pic_min"), band.pic_min);
            d.putd(
                format!("{tag}/ceiling_is_the_literal"),
                u64::from(2.5 <= cmap.phi_max(0.1) * n),
            );
            for (im, frac) in [0.0f64, 0.25, 0.5, 0.75, 1.0].into_iter().enumerate() {
                let m = band.m_lo + frac * (band.m_hi - band.m_lo);
                let p = ct.pic_of_m(&cmap, n, tt2, m);
                d.put(format!("{tag}/{im}/pi_c"), p.pi_c);
                d.put(format!("{tag}/{im}/flowcoef"), p.flowcoef);
                d.put(format!("{tag}/{im}/tau_c"), p.tau_c);
                d.put(format!("{tag}/{im}/tt3"), p.tt3);
                d.put(format!("{tag}/{im}/eta_c"), p.eta_c);
            }
        }
    }
    d.census("A");

    // ============================ B: the DECOUPLED instant — the only place `Phi` is observable
    for (sh, cmap) in shapes() {
        let ct = build(cmap, 0.05, 0.0, 0.0);
        for tt4 in [1400.0f64, 1100.0] {
            let mf = ct.inner.fuel_for_tt4(&fl, tt4, Some(&cmap));
            let nu0 = ct.inner.equilibrium_fuel(&fl, mf, Some(&cmap)).nu;
            let pt4_s = ct.plenum_pt4_at(&fl, nu0, mf, &cmap);
            let tag = format!("B/{sh}/{tt4:.0}");
            d.put(format!("{tag}/nu0"), nu0);
            d.put(format!("{tag}/pt4_steady"), pt4_s);
            for (ip, scale) in [0.94f64, 0.97, 1.0, 1.03, 1.06].into_iter().enumerate() {
                let s = ct.plenum_state(&fl, nu0, pt4_s * scale, mf, &cmap);
                for (k, v) in [
                    ("nu", s.nu), ("pt4", s.pt4), ("Tt4", s.tt4), ("pi_c", s.pi_c),
                    ("phi", s.flowcoef), ("f", s.f), ("mdot_c", s.mdot_c),
                    ("mdot_ngv", s.mdot_ngv), ("Phi", s.phi), ("dpt4_ds", s.dpt4_ds),
                    ("tau_t", s.tau_t), ("Tt3", s.tt3),
                ] {
                    d.put(format!("{tag}/{ip}/{k}"), v);
                }
                d.put(
                    format!("{tag}/{ip}/split"),
                    (s.mdot_c + mf - s.mdot_ngv) / s.mdot_ngv,
                );
            }
            let (tt2, pt2, n, _) = ct.face(&fl, nu0);
            let c = ct
                .try_compressor_from_backpressure(&cmap, n, tt2, pt2, pt4_s)
                .expect("the invert brackets at the steady pressure");
            d.put(format!("{tag}/bp_m"), c.m);
            d.put(format!("{tag}/bp_phi"), c.flowcoef);
            d.put(format!("{tag}/bp_tau_c"), c.tau_c);
            d.put(format!("{tag}/bp_Tt3"), c.tt3);
            d.put(format!("{tag}/bp_eta_c"), c.eta_c);
            d.put(format!("{tag}/bp_pi_c"), c.pi_c);
        }
    }
    d.census("B");

    // ============================ C: the plenum EQUILIBRIUM
    for (sh, cmap) in shapes() {
        let ct = build(cmap, 0.05, 0.0, 0.0);
        for tt4 in [1400.0f64, 1100.0, 900.0] {
            let mf = ct.inner.fuel_for_tt4(&fl, tt4, Some(&cmap));
            let a = ct.equilibrium_plenum(&fl, mf, Some(&cmap));
            let b = ct.inner.equilibrium_fuel(&fl, mf, Some(&cmap));
            let tag = format!("C/{sh}/{tt4:.0}");
            for (k, v) in [
                ("nu", a.nu), ("pt4", a.pt4), ("Tt4", a.tt4), ("pi_c", a.pi_c),
                ("phi", a.flowcoef), ("f", a.f), ("mdot_c", a.mdot_c),
                ("mdot_ngv", a.mdot_ngv), ("Phi", a.phi), ("dpt4_ds", a.dpt4_ds),
                ("tau_t", a.tau_t), ("Tt3", a.tt3),
            ] {
                d.put(format!("{tag}/{k}"), v);
            }
            d.put(format!("{tag}/rung35_nu"), b.nu);
            d.put(format!("{tag}/rung35_pi_c"), b.pi_c);
            d.put(format!("{tag}/rung35_tau_t"), b.tau_t);
            d.put(format!("{tag}/massbal_rel"), (a.mdot_c + mf - a.mdot_ngv) / a.mdot_ngv);
            d.put(format!("{tag}/mf"), mf);
        }
    }
    d.census("C");

    // ============================ D: the plenum MARCH
    for (sh, cmap) in shapes() {
        for (rvlab, r_v) in RVS {
            let ct = build(cmap, r_v, 0.0, 0.0);
            let r = ct.plenum_frozen_peak(&fl, 1100.0, 1400.0, Some(&cmap), 1.0 / 15.0);
            let tag = format!("D/{sh}/{rvlab}");
            d.put(format!("{tag}/E0"), r.e0);
            d.put(format!("{tag}/peak"), r.peak);
            d.put(format!("{tag}/peak_minus_E0"), r.peak_minus_e0);
            d.put(format!("{tag}/split_max"), r.split_max);
            d.put(format!("{tag}/nu0"), r.nu0);
            d.put(format!("{tag}/r_v"), r.r_v);
        }
    }
    d.census("D");

    // ============================ E: the soak CLOSURE and INSTANT, driven directly
    for (sh, cmap) in shapes() {
        let ct = build(cmap, 0.0, 0.1, 3.0);
        let mf = ct.inner.fuel_for_tt4(&fl, 1400.0, Some(&cmap));
        let nu = ct.inner.equilibrium_fuel(&fl, mf, Some(&cmap)).nu;
        let (tt2, pt2, n, _) = ct.face(&fl, nu);
        d.put(format!("E/{sh}/nu"), nu);
        d.put(format!("E/{sh}/tt2"), tt2);
        d.put(format!("E/{sh}/pt2"), pt2);
        d.put(format!("E/{sh}/n"), n);
        for (im, tm) in [1000.0f64, 1250.0, 1450.0, 1600.0].into_iter().enumerate() {
            let c = ct
                .try_close_compressor_fuel_soak(tt2, pt2, &cmap, n, mf, tm)
                .expect("the soak closure brackets on the running line");
            let tag = format!("E/{sh}/{im}");
            for (k, v) in [
                ("m", c.comp.m), ("m_imp", c.comp.m_imp), ("phi", c.comp.phi),
                ("tau_c", c.comp.tau_c), ("eta_c", c.comp.eta_c), ("Tt3", c.comp.tt3),
                ("Tt4_b", c.tt4_b), ("Tt4_t", c.tt4_t), ("pi_c", c.comp.pi_c),
                ("pt4", c.comp.pt4), ("f", c.comp.f), ("mdot4", c.comp.mdot4),
                ("mdot_air", c.comp.mdot_air),
            ] {
                d.put(format!("{tag}/{k}"), v);
            }
            let i = ct.instant_soak(&fl, nu, mf, tm, Some(&cmap));
            d.instant(&format!("{tag}/i_"), &i.inst);
            d.put(format!("{tag}/Tt4_burner"), i.tt4_burner);
            d.put(format!("{tag}/dTm_ds"), i.dtm_ds);
            d.putd(
                format!("{tag}/branch_choked"),
                u64::from(i.inst.branch == Branch::Choked),
            );
        }
    }
    d.census("E");

    // ============================ F: the soak EQUILIBRIUM
    for (sh, cmap) in shapes() {
        let ct = build(cmap, 0.0, 0.1, 3.0);
        for tt4 in [1400.0f64, 1100.0] {
            let mf = ct.inner.fuel_for_tt4(&fl, tt4, Some(&cmap));
            let a = ct.equilibrium_soak(&fl, mf, Some(&cmap));
            let b = ct.inner.equilibrium_fuel(&fl, mf, Some(&cmap));
            let tag = format!("F/{sh}/{tt4:.0}");
            d.instant(&format!("{tag}/"), &a.inst);
            d.put(format!("{tag}/Tt4_burner"), a.tt4_burner);
            d.put(format!("{tag}/dTm_ds"), a.dtm_ds);
            d.put(format!("{tag}/rung35_nu"), b.nu);
            d.put(format!("{tag}/rung35_pi_c"), b.pi_c);
            d.put(format!("{tag}/rung35_tau_t"), b.tau_t);
        }
    }
    d.census("F");

    // ============================ G: the two-state MARCH
    for (sh, cmap) in shapes() {
        for (glab, g) in GAINS {
            for (rlab, r_m) in RMS {
                let ct = build(cmap, 0.0, g, r_m);
                let tag = format!("G/{sh}/{glab}/{rlab}");
                let runs = [
                    ("cold", ct.soak_excursion(
                        &fl, 1100.0, 1400.0, Theta0::Cold, Some(&cmap), 0.05, 6.0)),
                    ("hot", ct.soak_excursion(
                        &fl, 1100.0, 1400.0, Theta0::Hot, Some(&cmap), 0.05, 6.0)),
                    ("adiab", ct.adiabatic_excursion(
                        &fl, 1100.0, 1400.0, Some(&cmap), 0.05, 6.0)),
                ];
                for (name, r) in runs {
                    d.put(format!("{tag}/{name}/e_surge"), r.e_surge);
                    d.put(format!("{tag}/{name}/nu0"), r.nu0);
                    d.put(format!("{tag}/{name}/nu_final"), r.nu_final);
                    d.putd(
                        format!("{tag}/{name}/t_accel_is_none"),
                        u64::from(r.t_accel.is_none()),
                    );
                    d.put(format!("{tag}/{name}/t_accel"), r.t_accel.unwrap_or(0.0));
                }
                d.putd(
                    format!("{tag}/ordering_holds"),
                    u64::from(
                        runs[0].1.e_surge < runs[1].1.e_surge
                            && runs[1].1.e_surge < runs[2].1.e_surge,
                    ),
                );
            }
        }
    }
    d.census("G");

    // ============================ H: the accel LAG
    let cmap = ComponentMap::surge_flow();
    for (glab, g) in GAINS {
        let ct = build(cmap, 0.0, g, 3.0);
        for (name, r) in [
            ("adiab", ct.adiabatic_excursion(&fl, 1100.0, 1400.0, Some(&cmap), 0.05, 12.0)),
            ("cold", ct.soak_excursion(
                &fl, 1100.0, 1400.0, Theta0::Cold, Some(&cmap), 0.05, 12.0)),
            ("hot", ct.soak_excursion(
                &fl, 1100.0, 1400.0, Theta0::Hot, Some(&cmap), 0.05, 12.0)),
        ] {
            d.put(format!("H/{glab}/{name}/e_surge"), r.e_surge);
            d.putd(format!("H/{glab}/{name}/t_accel_is_none"), u64::from(r.t_accel.is_none()));
            d.put(format!("H/{glab}/{name}/t_accel"), r.t_accel.unwrap_or(0.0));
        }
    }
    d.census("H");

    // ============================ I: the both-OFF REDUCE
    let ct_off = build(ComponentMap::surge_flow(), 0.0, 0.0, 0.0);
    let st = SpoolTransient::new(design(), fl, 1.0, ComponentMap::surge_flow());
    d.put("I/plenum_K".into(), ct_off.plenum_k);
    d.put("I/pt4_d".into(), ct_off.pt4_d);
    d.put("I/mdot4_d".into(), ct_off.mdot4_d);
    for tt4 in [1500.0f64, 1300.0, 1200.0, 1000.0, 900.0] {
        let mf = st.fuel_for_tt4(&fl, tt4, None);
        let a = ct_off.inner.equilibrium_fuel(&fl, mf, None);
        let b = st.equilibrium_fuel(&fl, mf, None);
        let mut identical = true;
        for (k, av, bv) in [
            ("nu", a.nu, b.nu), ("pi_c", a.pi_c, b.pi_c), ("tau_t", a.tau_t, b.tau_t),
            ("Tt4", a.tt4, b.tt4), ("mdot_air", a.mdot_air, b.mdot_air),
        ] {
            d.put(format!("I/{tt4:.0}/ct_{k}"), av);
            d.put(format!("I/{tt4:.0}/st_{k}"), bv);
            identical &= av == bv;
        }
        d.put(format!("I/{tt4:.0}/ct_Phi"), a.phi);
        d.put(format!("I/{tt4:.0}/st_Phi"), b.phi);
        d.putd(format!("I/{tt4:.0}/bit_identical"), u64::from(identical));
    }
    d.census("I");

    d
}

// ============================================================================== the gates

/// **§ 5.14 PREDICTION 1** — every value the Rust produces equals PyPy's, BIT FOR BIT.
///
/// The key SETS are compared before the values: a Rust that produced 200 fewer keys and agreed on
/// all of them would otherwise read as a pass. *Coverage is a name diff, never a count.*
#[test]
fn every_value_is_bit_exact_against_pypy() {
    let py = load("oracle/combustor_pypy.tsv");
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
    println!("combustor_oracle: {} keys, 100% bit-exact against PyPy", py.len());
    assert!(py.len() > 2000, "the dump shrank — {} keys", py.len());
}

/// **§ 5.14 PREDICTION 3, the value half.** The Illinois exhaustion arm is not merely reached —
/// it is the path most of one site's calls take, and no other site takes at all.
///
/// Slice P measured this arm at ZERO firings across its whole grid, listed the `a`-vs-`b` return
/// as one of two defects invisible to 132 bit-exact values, and closed it with a counter rather
/// than deleting the claim. The counter now has a population, and it is confined to the site whose
/// tolerance is an ABSOLUTE `1e-12` on a quantity of order `1e5`.
#[test]
fn the_exhaustion_arm_is_confined_to_the_pressure_solve() {
    let py = load("oracle/combustor_pypy.tsv");
    let get = |k: &str| *py.get(k).unwrap_or_else(|| panic!("missing census key {k}"));

    // E, F, G and H root-find hard — 219 to 50 976 Illinois calls apiece — and touch
    // `_plenum_pt4_at` zero times. Their zero exhaustion is therefore evidence.
    //
    // **Section A is deliberately NOT in this list, and the first draft of this gate put it
    // there.** A is pure arithmetic — `pic_band` and `pic_of_m`, no bracket anywhere — so it
    // makes exactly **0** Illinois calls, and "A exhausts nothing" is the emptiest sentence in
    // the file. The `illinois_calls > 50` clause is what caught it, which is the clause's whole
    // purpose: an exhaustion count of zero means nothing without a call count beside it.
    for sec in ["E", "F", "G", "H"] {
        assert_eq!(get(&format!("census/{sec}/pt4_at_calls")), 0, "{sec} calls the solve");
        assert_eq!(
            get(&format!("census/{sec}/illinois_exhausted")), 0,
            "section {sec} exhausts `maxit` without ever calling `_plenum_pt4_at` — the arm is \
             not supposed to be reachable from anywhere else"
        );
        assert!(
            get(&format!("census/{sec}/illinois_calls")) > 50,
            "section {sec} barely root-finds, so its zero above is silence, not evidence"
        );
    }
    assert_eq!(
        get("census/A/illinois_calls"), 0,
        "section A is pure arithmetic and must root-find NOTHING — if it acquires a bracket, it \
         belongs in the loop above rather than here"
    );
    // C runs the nested plenum equilibrium and exhausts heavily.
    //
    // **THE DENOMINATOR IS NOT THE CALL COUNT, AND THE FIRST VERSION OF THIS BAR USED IT.** Of
    // section C's 225 `_plenum_pt4_at` calls, **116 fail the bracket test** and never reach the
    // Illinois at all — `equilibrium_soak`'s outer root search probes speeds off the operable map.
    // The population the exhaustion rate is about is `calls - bracket_fails`, and against that
    // denominator it is 103 of 109 — the 94.5 % § 5.14 probe 2 measured, reproduced exactly.
    // Against the raw call count it is 46 %, which is a different number about a different set.
    let calls = get("census/C/pt4_at_calls");
    let failed = get("census/C/pt4_at_bracket_fails");
    let ex = get("census/C/illinois_exhausted");
    let reached = calls - failed;
    assert!(calls > 50 && ex > 50, "C: {calls} pressure solves, {ex} exhaustions");
    assert!(
        ex * 10 >= reached * 9,
        "at least 90 % of the `_plenum_pt4_at` calls that REACH the Illinois must run out of \
         iterations — {ex} of {reached} ({calls} calls less {failed} that never bracketed). \
         Fewer would mean the tolerance was ported as a RELATIVE bar."
    );
}

/// **§ 5.14 PREDICTION 7, and the limit of what a per-SECTION census can say about it.**
///
/// The prediction is that `_plenum_state` reaches rung 34's hook once per call and the nozzle
/// dispatch never — it solves the choked `(★)` geometry and stops, with no `Nozzle`, no subsonic
/// re-solve and no `M9 > 0.985` escalation.
///
/// **THE FIRST VERSION OF THIS GATE ASSERTED `subsonic_raises == 0` IN THE PLENUM SECTIONS AND
/// FAILED, CORRECTLY.** Section B tallies **1** and section D **4** — not from any plenum instant,
/// but from the rung-35 calls each section makes to *reach* one (`fuel_for_tt4`,
/// `equilibrium_fuel`, `constant_speed_excursion_fuel`), which all go through `_instant_tail`. A
/// per-section census answers *"what happened while this section ran"*; the prediction is about
/// *"what happens inside this function"*, and those are different questions. **It is the same
/// conflation `probe_q2.py`'s scoped-vs-global instrument made**, arriving in a gate rather than
/// in a probe — where it would have shipped as a false claim had the tallies happened to be zero.
///
/// The scoped form lives in `rung37.rs::the_plenum_instant_reaches_the_hook_and_never_the_nozzle`,
/// which brackets the counters around `try_plenum_state` calls **alone** and gets zero. What the
/// dump can support is asserted here instead: the escalation guard is dead across the WHOLE rung,
/// the hook fires at least once per plenum instant, and the raises are three orders too few to be
/// one-per-instant.
#[test]
fn the_plenum_path_never_reaches_the_nozzle_dispatch() {
    let py = load("oracle/combustor_pypy.tsv");
    let get = |k: &str| *py.get(k).unwrap_or_else(|| panic!("missing census key {k}"));

    // Slice P's rarest branch — the `M9 > 0.985` escalation — never fires anywhere in rung 37.
    for sec in ["A", "B", "C", "D", "E", "F", "G", "H", "I"] {
        assert_eq!(
            get(&format!("census/{sec}/subsonic_escalations")), 0,
            "{sec}: rung 37 never reaches the escalation guard, so slice P's count for it is not \
             coverage that transfers here"
        );
    }
    for sec in ["B", "C", "D"] {
        let (states, hook, raises) = (
            get(&format!("census/{sec}/plenum_state_calls")),
            get(&format!("census/{sec}/r34_solve_turbine")),
            get(&format!("census/{sec}/subsonic_raises")),
        );
        assert!(states > 100, "{sec} barely runs the plenum instant: {states}");
        assert!(
            hook >= states,
            "{sec}: the hook must fire at least once per plenum instant ({hook} vs {states})"
        );
        assert!(
            raises * 100 < states,
            "{sec}: {raises} subsonic raises against {states} plenum instants. A raise per instant \
             would mean the plenum was dispatching on the nozzle; a handful means they came from \
             the rung-35 calls that reach the plenum, which is what the scoped gate in rung37.rs \
             is for"
        );
    }
}

/// **§ 5.14 PREDICTION 5** — the two dead bracket failures, gated against zero WITH live siblings.
///
/// `_compressor_from_backpressure`'s bracket and `_plenum_pt4_at`'s `m_min < m_max` floor never
/// fire on any grid measured. That is only evidence because `_plenum_pt4_at`'s OWN bracket and the
/// soak closure's DO fire in the same dump — a zero on its own is silence.
#[test]
fn the_two_dead_brackets_are_dead_and_the_live_ones_are_live() {
    let py = load("oracle/combustor_pypy.tsv");
    let get = |k: &str| *py.get(k).unwrap_or_else(|| panic!("missing census key {k}"));
    let secs = ["A", "B", "C", "D", "E", "F", "G", "H", "I"];
    let sum = |name: &str| -> u64 {
        secs.iter().map(|s| get(&format!("census/{s}/{name}"))).sum()
    };
    assert!(sum("backpressure_calls") > 1000, "the invert barely ran");
    assert_eq!(sum("backpressure_bracket_fails"), 0, "the invert's bracket is supposed to be dead");
    assert_eq!(sum("pt4_at_floor_fails"), 0, "the flow-floor guard is supposed to be dead");
    assert!(
        sum("pt4_at_bracket_fails") > 0,
        "`_plenum_pt4_at`'s bracket MUST fail during the equilibrium march-in — without a live \
         sibling the two zeros above are silence"
    );
    assert!(
        sum("soak_close_bracket_fails") > 0,
        "the soak closure's bracket MUST fail during `equilibrium_soak`'s march-in"
    );
    // ...and it must NOT fail inside the marches, which have no `try` (§ 5.14 prediction 4).
    for sec in ["G", "H"] {
        assert!(get(&format!("census/{sec}/soak_close_calls")) > 1000, "{sec} barely marched");
        assert_eq!(
            get(&format!("census/{sec}/soak_close_bracket_fails")), 0,
            "{sec}: a bracket failure inside a rung-37 march would ABORT it — Python has no `try` \
             there, so this must stay at zero for the marches to mean anything"
        );
    }
}

/// **§ 5.14 PREDICTION 9's shape, moved to the right function.** The flow ceiling's two arms.
///
/// `min(2.5, phi_max*n)` takes the map arm in every cell any GATE reaches; § 5.14 probe 3
/// measured 15 of 15. Section A drives `nu` up to 1.3 to put the other side on the dump.
///
/// **AND THE ARM IS SHAPE-DEPENDENT, WHICH THIS GATE PREDICTED WRONG.** The first version
/// asserted that all three shapes take the literal arm at `nu = 1.3`; **one** does. `surge_flow`'s
/// `phi_max` is large enough that `phi_max*n` clears 2.5 there, and `surge_pressure`'s and
/// `surge_tilted`'s are not — so the literal is reachable on ONE of the three maps and unreachable
/// on the other two even a third of the way past the design speed. The gate now names the cell.
#[test]
fn both_arms_of_the_flow_ceiling_are_present() {
    let py = load("oracle/combustor_pypy.tsv");
    let keys: Vec<_> =
        py.keys().filter(|k| k.ends_with("/ceiling_is_the_literal")).collect();
    assert_eq!(keys.len(), 12, "section A's ceiling census changed size");
    let lit: Vec<_> = keys.iter().filter(|k| py[k.as_str()] == 1).collect();
    assert_eq!(
        lit.len(), 1,
        "exactly ONE cell should take the LITERAL 2.5 arm; got {lit:?}"
    );
    assert_eq!(
        lit[0].as_str(), "A/flow/1.3/ceiling_is_the_literal",
        "the literal arm is reachable on `surge_flow` at nu = 1.3 and nowhere else on this grid"
    );
    for k in &keys {
        if k.as_str() != "A/flow/1.3/ceiling_is_the_literal" {
            assert_eq!(py[*k], 0, "{k}: takes the map arm");
        }
    }
}

/// **§ 5.14 PREDICTION 9** — rung 37 reaches exactly ONE arm of `phi_max`, and it is not the same
/// census slice P measured.
///
/// **THIS GATE EXISTS BECAUSE AN ENUMERATION OVER THE REGISTERED PREDICTIONS FOUND IT MISSING.**
/// The first version of this oracle dumped no arm tallies at all, so prediction 9 was carried by
/// nothing — slice P's step 3 recorded exactly that failure mode (*"a step's write-up is a claim
/// about what is gated, and it is only as good as an enumeration over the registered list"*), and
/// the enumeration was run before the write-up this time rather than after.
///
/// Rung 37's grid is all surge shapes, so only `quadratic` fires. Slice P's had flat maps too and
/// measured `flat5` at 5 258 — a DIFFERENT census, never to be merged with this one. That the
/// other three arms are reachable AT ALL is `spool_oracle.rs`'s gate, whose direct section drives
/// all four; re-driving them here would gate the same fact twice.
#[test]
fn rung37_reaches_only_the_quadratic_phi_max_arm() {
    let py = load("oracle/combustor_pypy.tsv");
    let get = |k: &str| *py.get(k).unwrap_or_else(|| panic!("missing census key {k}"));
    let mut live = 0u64;
    for sec in ["A", "B", "C", "D", "E", "F", "G", "H", "I"] {
        for dead in ["phi_max_flat5", "phi_max_linear", "phi_max_swirled"] {
            assert_eq!(
                get(&format!("census/{sec}/{dead}")), 0,
                "{sec}: {dead} fired, but rung 37's grid is all surge shapes"
            );
        }
        live += get(&format!("census/{sec}/phi_max_quadratic"));
    }
    assert!(
        live > 1000,
        "the arm counter reports {live} live calls across the whole dump — it is broken, not the \
         other three arms being dead"
    );
}

/// **THE VERDICT KEYS.** Rung 37's two findings, dumped as discrete answers rather than inferred
/// from the values: `cold < hot < adiabatic` on all twelve heat-soak cells, and the both-OFF
/// reduce bit-identical on all five.
#[test]
fn the_two_findings_are_dumped_as_discrete_verdicts() {
    let py = load("oracle/combustor_pypy.tsv");
    let ord: Vec<_> = py.keys().filter(|k| k.ends_with("/ordering_holds")).collect();
    assert_eq!(ord.len(), 12, "gate 5's grid changed size");
    for k in &ord {
        assert_eq!(py[*k], 1, "{k}: cold < hot < adiabatic must hold");
    }
    let red: Vec<_> = py.keys().filter(|k| k.ends_with("/bit_identical")).collect();
    assert_eq!(red.len(), 5);
    for k in &red {
        assert_eq!(py[*k], 1, "{k}: the both-OFF reduce must be BIT-identical to rung 35");
    }
    // `t_accel` is an Option and BOTH arms are reachable — 4 of 42 `None` on this grid.
    let ta: Vec<_> = py.keys().filter(|k| k.ends_with("/t_accel_is_none")).collect();
    assert_eq!(ta.len(), 42);
    let none = ta.iter().filter(|k| py[k.as_str()] == 1).count();
    assert!(
        none > 0 && none < ta.len(),
        "both arms of `t_accel` must be present on the dump grid; got {none} of {}",
        ta.len()
    );
}

/// **THE CPython ARM.** The same dump under CPython 3.14, as a detector for how much of the
/// bit-exactness above is real agreement and how much is arithmetic that could not disagree.
///
/// § 5.12's pre-flight probe got **100 %** on a CPG gas and recorded it as NOT coverage. On the
/// thermally-perfect gas the same arm agrees on **20.9 %** — slice P measured 22.6 % on its own
/// grid, so this is the second independent confirmation that the arm is a sharp instrument rather
/// than a formality.
///
/// **The tiering is the finding, and it reproduces slice P's exactly.** The DISCRETE OUTPUT keys —
/// branch labels, the `cold < hot < adiabatic` verdicts, the `t_accel` presence flags, the reduce
/// flags, which arm of the flow ceiling binds — are **interpreter-invariant to the last one**,
/// while four fifths of the continuous keys move between two correct implementations of one
/// language. What the physics DECIDES does not depend on the interpreter; what it computes does.
#[test]
fn the_cpython_arm_is_a_detector_and_every_discrete_output_survives_it() {
    let py = load("oracle/combustor_pypy.tsv");
    let cp = load("oracle/combustor_cpython.tsv");
    assert_eq!(
        py.keys().collect::<Vec<_>>(),
        cp.keys().collect::<Vec<_>>(),
        "the two interpreter dumps must cover the SAME keys"
    );

    let is_output_discrete = |k: &str| {
        k.ends_with("/branch_choked")
            || k.ends_with("/ordering_holds")
            || k.ends_with("/t_accel_is_none")
            || k.ends_with("/bit_identical")
            || k.ends_with("/ceiling_is_the_literal")
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
            d_cen += usize::from(a != b);
        }
    }

    assert!(
        bad_out.is_empty(),
        "{} of {n_out} DISCRETE OUTPUT keys differ between interpreters — rung 37's verdicts are \
         supposed to be arithmetic-independent:\n  {}",
        bad_out.len(),
        bad_out.iter().take(10).cloned().collect::<Vec<_>>().join("\n  ")
    );
    assert_eq!(n_out, 83, "the discrete-output family changed size — re-read the tiering");

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
