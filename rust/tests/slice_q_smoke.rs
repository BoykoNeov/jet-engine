//! SLICE Q step 1 — the smoke check for [`CombustorTransient`], against a Python dump of the
//! SAME cells.
//!
//! Not the slice's oracle (that is step 3, on the seven gates' own grid). This exists to catch a
//! structural mistake before the Python gates are ported on top of it — and § 5.14 named four in
//! advance, each of which the shipped code deliberately does NOT do:
//!
//! 1. the plenum's power block copied from `_instant_tail` (per unit AIR) instead of written on
//!    the ABSOLUTE flows — equal only where the plenum is not storing, which is everywhere except
//!    the transient the rung is about;
//! 2. `_compressor_from_backpressure` returning the `pi_c` recomputed at the root instead of the
//!    REQUIRED back-pressure Python returns;
//! 3. `equilibrium_soak`'s two fixed-point loops unified — they differ by one line and the
//!    difference is 3e-12, four orders below the gate that is supposed to catch it;
//! 4. a march routed through `spool.rs::march`, which would convert a raise into a truncation.
//!
//! # The census is half the check, because two of the four are COUNT properties
//!
//! Every section emits `spool.rs`'s counters beside this module's. The one that matters most is
//! `illinois_exhausted`: § 5.14 probe 2 measured `_plenum_pt4_at`'s Illinois running out of
//! iterations on **94.5 %** of its calls, because it passes `_N_TOL = 1e-12` as an ABSOLUTE
//! bracket width on a `pt4` of order 1e5 Pa. Slice P had shipped that same arm with **zero**
//! firings and had to close the blind spot with a counter; here the counter has a population, and
//! whether the arm returns `b` or `a` is worth 3.5e-12 — still invisible to every value gate rung
//! 37 ships.
//!
//! Regenerate the goldens with:
//!     .venv\Scripts\python.exe rust\oracle\dump_slice_q_smoke.py > rust\oracle\slice_q_smoke_pypy.tsv

use std::collections::BTreeMap;

use turbojet::combustor::{counters as ccount, CombustorTransient, Theta0};
use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::Gas;
use turbojet::map::ComponentMap;
use turbojet::spool::{counters as scount, SpoolTransient};

const ORACLE: &str = include_str!("../oracle/slice_q_smoke_pypy.tsv");

fn load() -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in ORACLE.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, b) = (it.next().expect("key"), it.next().expect("bits"));
        assert!(m.insert(k.to_string(), b.parse::<u64>().expect("u64")).is_none(), "dup {k}");
    }
    m
}

/// Accumulates `(key, got_bits, want_bits)` so ONE run reports every disagreement, not the first.
struct Cmp {
    py: BTreeMap<String, u64>,
    bad: Vec<String>,
    seen: usize,
}

impl Cmp {
    fn new() -> Self {
        Cmp { py: load(), bad: Vec::new(), seen: 0 }
    }
    fn f(&mut self, key: &str, got: f64) {
        let want = *self.py.get(key).unwrap_or_else(|| panic!("no golden for {key}"));
        self.seen += 1;
        if got.to_bits() != want {
            self.bad.push(format!(
                "{key}: rust {got:.17e} ({:016x}) != py {:.17e} ({want:016x})",
                got.to_bits(),
                f64::from_bits(want)
            ));
        }
    }
    fn d(&mut self, key: &str, got: u64) {
        let want = *self.py.get(key).unwrap_or_else(|| panic!("no golden for {key}"));
        self.seen += 1;
        if got != want {
            self.bad.push(format!("{key}: rust {got} != py {want}"));
        }
    }
    fn finish(self, what: &str) {
        assert!(
            self.bad.is_empty(),
            "{} of {} {what} values differ:\n  {}",
            self.bad.len(),
            self.seen,
            self.bad.join("\n  ")
        );
        println!("slice_q_smoke/{what}: {} values bit-exact against PyPy", self.seen);
    }
}

// ---------------------------------------------------------------------------- the grid
fn flight() -> FlightCondition {
    FlightCondition { t0: 250.0, p0: 50_000.0, m0: 0.85 }
}

fn engine() -> Engine {
    build_turbojet(
        Gas::thermally_perfect(),
        10.0,
        1500.0,
        50_000.0,
        Losses {
            pi_d: 0.97, eta_c: 0.88, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, e_t: None,
            eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
        },
    )
}

fn build(cmap: ComponentMap, r_v: f64, g: f64, r_m: f64) -> CombustorTransient {
    CombustorTransient::new(engine(), flight(), 1.0, cmap, r_v, g, r_m)
}

fn census(c: &mut Cmp, prefix: &str) {
    let s = scount::take();
    let k = ccount::take();
    c.d(&format!("census/{prefix}/illinois_calls"), s.illinois_calls);
    c.d(&format!("census/{prefix}/illinois_evals"), s.illinois_evals);
    c.d(&format!("census/{prefix}/illinois_exhausted"), s.illinois_exhausted);
    c.d(&format!("census/{prefix}/backpressure_calls"), k.backpressure_calls);
    c.d(&format!("census/{prefix}/backpressure_bracket_fails"), k.backpressure_bracket_fails);
    c.d(&format!("census/{prefix}/pt4_at_calls"), k.pt4_at_calls);
    c.d(&format!("census/{prefix}/pt4_at_bracket_fails"), k.pt4_at_bracket_fails);
    c.d(&format!("census/{prefix}/pt4_at_floor_fails"), k.pt4_at_floor_fails);
    c.d(&format!("census/{prefix}/soak_close_calls"), k.soak_close_calls);
    c.d(&format!("census/{prefix}/soak_close_bracket_fails"), k.soak_close_bracket_fails);
    c.d(&format!("census/{prefix}/plenum_state_calls"), k.plenum_state_calls);
    c.d(&format!("census/{prefix}/instant_soak_calls"), k.instant_soak_calls);
}

/// The whole smoke, in ONE test.
///
/// The counters are thread-locals and `take()` RESETS them, so a second `#[test]` in this binary
/// would run concurrently and steal the other's tallies — the failure would then read as a
/// physics disagreement rather than a harness one. `stage.rs`'s `take_census` note, applied.
#[test]
fn slice_q_smoke_is_bit_exact_against_pypy() {
    let fl = flight();
    let flow = ComponentMap::surge_flow();
    let press = ComponentMap::surge_pressure();
    let mut c = Cmp::new();

    // reset anything the process did before this point
    let _ = scount::take();
    let _ = ccount::take();

    // ------------------------------------------------- A: the speed line read as pi_c(m)
    let ctp = build(flow, 0.05, 0.0, 0.0);
    for (iu, nu) in [0.85f64, 1.0].into_iter().enumerate() {
        let (tt2, _pt2, n, _v0) = ctp.face(&fl, nu);
        let band = ctp.pic_band(&flow, n, tt2);
        c.f(&format!("A/band/{iu}/m_lo"), band.m_lo);
        c.f(&format!("A/band/{iu}/pic_max"), band.pic_max);
        c.f(&format!("A/band/{iu}/m_hi"), band.m_hi);
        c.f(&format!("A/band/{iu}/pic_min"), band.pic_min);
        for (im, m) in [band.m_lo, 0.5 * (band.m_lo + band.m_hi), band.m_hi]
            .into_iter()
            .enumerate()
        {
            let p = ctp.pic_of_m(&flow, n, tt2, m);
            c.f(&format!("A/pic/{iu}/{im}/pi_c"), p.pi_c);
            c.f(&format!("A/pic/{iu}/{im}/flowcoef"), p.flowcoef);
            c.f(&format!("A/pic/{iu}/{im}/tau_c"), p.tau_c);
            c.f(&format!("A/pic/{iu}/{im}/tt3"), p.tt3);
            c.f(&format!("A/pic/{iu}/{im}/eta_c"), p.eta_c);
        }
    }
    census(&mut c, "A");

    // ------------------------------------------------- B: the BACK-PRESSURE invert
    for (iu, nu) in [0.85f64, 1.0].into_iter().enumerate() {
        let (tt2, pt2, n, _) = ctp.face(&fl, nu);
        let band = ctp.pic_band(&flow, n, tt2);
        let m_mid = 0.5 * (band.m_lo + band.m_hi);
        let pt4 = ctp.pic_of_m(&flow, n, tt2, m_mid).pi_c * ctp.inner.inner.inner.pi_b * pt2;
        let b = ctp
            .try_compressor_from_backpressure(&flow, n, tt2, pt2, pt4)
            .expect("the invert brackets at a pt4 built from an interior flow");
        c.f(&format!("B/{iu}/m"), b.m);
        c.f(&format!("B/{iu}/phi"), b.flowcoef);
        c.f(&format!("B/{iu}/tau_c"), b.tau_c);
        c.f(&format!("B/{iu}/Tt3"), b.tt3);
        c.f(&format!("B/{iu}/eta_c"), b.eta_c);
        c.f(&format!("B/{iu}/pi_c"), b.pi_c);
        c.f(&format!("B/{iu}/m_target"), m_mid);
    }
    census(&mut c, "B");

    // ------------------------------------------------- C: the DECOUPLED instant
    let mf14 = ctp.inner.fuel_for_tt4(&fl, 1400.0, Some(&flow));
    let nu0 = ctp.inner.equilibrium_fuel(&fl, mf14, Some(&flow)).nu;
    let pt4_s = ctp.plenum_pt4_at(&fl, nu0, mf14, &flow);
    c.f("C/nu0", nu0);
    c.f("C/pt4_steady", pt4_s);
    for (ip, scale) in [0.97f64, 1.0, 1.03].into_iter().enumerate() {
        let s = ctp.plenum_state(&fl, nu0, pt4_s * scale, mf14, &flow);
        c.f(&format!("C/{ip}/nu"), s.nu);
        c.f(&format!("C/{ip}/pt4"), s.pt4);
        c.f(&format!("C/{ip}/Tt4"), s.tt4);
        c.f(&format!("C/{ip}/pi_c"), s.pi_c);
        c.f(&format!("C/{ip}/phi"), s.flowcoef);
        c.f(&format!("C/{ip}/f"), s.f);
        c.f(&format!("C/{ip}/mdot_c"), s.mdot_c);
        c.f(&format!("C/{ip}/mdot_ngv"), s.mdot_ngv);
        c.f(&format!("C/{ip}/Phi"), s.phi);
        c.f(&format!("C/{ip}/dpt4_ds"), s.dpt4_ds);
        c.f(&format!("C/{ip}/tau_t"), s.tau_t);
        c.f(&format!("C/{ip}/Tt3"), s.tt3);
        c.f(&format!("C/{ip}/split"), (s.mdot_c + mf14 - s.mdot_ngv) / s.mdot_ngv);
    }
    census(&mut c, "C");

    // ------------------------------------------------- D: the EXHAUSTING root find
    for (iu, nu) in [0.85f64, 1.0].into_iter().enumerate() {
        c.f(&format!("D/{iu}/pt4"), ctp.plenum_pt4_at(&fl, nu, mf14, &flow));
    }
    census(&mut c, "D");

    // ------------------------------------------------- E: the non-tautological reduce
    for (sh, cmap) in [("flow", flow), ("press", press)] {
        let ct = build(cmap, 0.05, 0.0, 0.0);
        for (it, tt4) in [1400.0f64, 1100.0].into_iter().enumerate() {
            let mf = ct.inner.fuel_for_tt4(&fl, tt4, Some(&cmap));
            let a = ct.equilibrium_plenum(&fl, mf, Some(&cmap));
            let b = ct.inner.equilibrium_fuel(&fl, mf, Some(&cmap));
            c.f(&format!("E/{sh}/{it}/nu"), a.nu);
            c.f(&format!("E/{sh}/{it}/pt4"), a.pt4);
            c.f(&format!("E/{sh}/{it}/Tt4"), a.tt4);
            c.f(&format!("E/{sh}/{it}/pi_c"), a.pi_c);
            c.f(&format!("E/{sh}/{it}/phi"), a.flowcoef);
            c.f(&format!("E/{sh}/{it}/f"), a.f);
            c.f(&format!("E/{sh}/{it}/mdot_c"), a.mdot_c);
            c.f(&format!("E/{sh}/{it}/mdot_ngv"), a.mdot_ngv);
            c.f(&format!("E/{sh}/{it}/Phi"), a.phi);
            c.f(&format!("E/{sh}/{it}/dpt4_ds"), a.dpt4_ds);
            c.f(&format!("E/{sh}/{it}/tau_t"), a.tau_t);
            c.f(&format!("E/{sh}/{it}/Tt3"), a.tt3);
            c.f(&format!("E/{sh}/{it}/rung35_nu"), b.nu);
            c.f(&format!("E/{sh}/{it}/rung35_pi_c"), b.pi_c);
            c.f(
                &format!("E/{sh}/{it}/massbal_rel"),
                (a.mdot_c + mf - a.mdot_ngv) / a.mdot_ngv,
            );
        }
    }
    census(&mut c, "E");

    // ------------------------------------------------- F: the plenum march
    for (sh, cmap) in [("flow", flow), ("press", press)] {
        for (iv, r_v) in [0.03f64, 0.1].into_iter().enumerate() {
            let ct = build(cmap, r_v, 0.0, 0.0);
            let r = ct.plenum_frozen_peak(&fl, 1100.0, 1400.0, Some(&cmap), 1.0 / 15.0);
            c.f(&format!("F/{sh}/{iv}/E0"), r.e0);
            c.f(&format!("F/{sh}/{iv}/peak"), r.peak);
            c.f(&format!("F/{sh}/{iv}/peak_minus_E0"), r.peak_minus_e0);
            c.f(&format!("F/{sh}/{iv}/split_max"), r.split_max);
            c.f(&format!("F/{sh}/{iv}/nu0"), r.nu0);
            c.f(&format!("F/{sh}/{iv}/r_v"), r.r_v);
        }
    }
    census(&mut c, "F");

    // ------------------------------------------------- G: the soak closure
    let cts = build(flow, 0.0, 0.1, 3.0);
    let mfs = cts.inner.fuel_for_tt4(&fl, 1400.0, Some(&flow));
    let nus = cts.inner.equilibrium_fuel(&fl, mfs, Some(&flow)).nu;
    c.f("G/nu", nus);
    let (tt2, pt2, n, _) = cts.face(&fl, nus);
    c.f("G/tt2", tt2);
    c.f("G/pt2", pt2);
    c.f("G/n", n);
    // 1600 K is ABOVE the burner exit, so the sink runs BACKWARDS (the reslam sign) — the branch
    // a cold-only grid would leave untouched.
    for (im, tm) in [1100.0f64, 1400.0, 1600.0].into_iter().enumerate() {
        let s = cts
            .try_close_compressor_fuel_soak(tt2, pt2, &flow, n, mfs, tm)
            .expect("the soak closure brackets at the running-line speed");
        c.f(&format!("G/{im}/m"), s.comp.m);
        c.f(&format!("G/{im}/m_imp"), s.comp.m_imp);
        c.f(&format!("G/{im}/phi"), s.comp.phi);
        c.f(&format!("G/{im}/tau_c"), s.comp.tau_c);
        c.f(&format!("G/{im}/eta_c"), s.comp.eta_c);
        c.f(&format!("G/{im}/Tt3"), s.comp.tt3);
        c.f(&format!("G/{im}/Tt4_b"), s.tt4_b);
        c.f(&format!("G/{im}/Tt4_t"), s.tt4_t);
        c.f(&format!("G/{im}/pi_c"), s.comp.pi_c);
        c.f(&format!("G/{im}/pt4"), s.comp.pt4);
        c.f(&format!("G/{im}/f"), s.comp.f);
        c.f(&format!("G/{im}/mdot4"), s.comp.mdot4);
        c.f(&format!("G/{im}/mdot_air"), s.comp.mdot_air);
    }
    census(&mut c, "G");

    // ------------------------------------------------- H: the soak instant
    for (im, tm) in [1100.0f64, 1400.0, 1600.0].into_iter().enumerate() {
        let s = cts.instant_soak(&fl, nus, mfs, tm, Some(&flow));
        let i = &s.inst;
        c.f(&format!("H/{im}/nu"), i.nu);
        c.f(&format!("H/{im}/Tt4"), i.tt4);
        c.f(&format!("H/{im}/pi_c"), i.pi_c);
        c.f(&format!("H/{im}/tau_c"), i.tau_c);
        c.f(&format!("H/{im}/eta_c"), i.eta_c);
        c.f(&format!("H/{im}/eta_t"), i.eta_t);
        c.f(&format!("H/{im}/m"), i.m);
        c.f(&format!("H/{im}/n"), i.n);
        c.f(&format!("H/{im}/flowcoef"), i.flowcoef);
        c.f(&format!("H/{im}/mdot_air"), i.mdot_air);
        c.f(&format!("H/{im}/f"), i.f);
        c.f(&format!("H/{im}/pi_t"), i.pi_t);
        c.f(&format!("H/{im}/tau_t"), i.tau_t);
        c.f(&format!("H/{im}/Tt3"), i.tt3);
        c.f(&format!("H/{im}/Tt5"), i.tt5);
        c.f(&format!("H/{im}/nu_t"), i.nu_t);
        c.f(&format!("H/{im}/p_net_spec"), i.p_net_spec);
        c.f(&format!("H/{im}/Phi"), i.phi);
        c.f(&format!("H/{im}/sp_thrust"), i.sp_thrust);
        c.f(&format!("H/{im}/thrust"), i.thrust);
        c.f(&format!("H/{im}/M9"), i.m9);
        c.f(&format!("H/{im}/pt9_over_p0"), i.pt9_over_p0);
        c.f(&format!("H/{im}/Tt4_burner"), s.tt4_burner);
        c.f(&format!("H/{im}/dTm_ds"), s.dtm_ds);
        c.d(
            &format!("H/{im}/branch_choked"),
            u64::from(i.branch == turbojet::matcher::Branch::Choked),
        );
    }
    census(&mut c, "H");

    // ------------------------------------------------- I: the TWO-LOOP equilibrium
    for (sh, cmap) in [("flow", flow), ("press", press)] {
        let ct = build(cmap, 0.0, 0.1, 3.0);
        for (it, tt4) in [1400.0f64, 1100.0].into_iter().enumerate() {
            let mf = ct.inner.fuel_for_tt4(&fl, tt4, Some(&cmap));
            let a = ct.equilibrium_soak(&fl, mf, Some(&cmap));
            let b = ct.inner.equilibrium_fuel(&fl, mf, Some(&cmap));
            c.f(&format!("I/{sh}/{it}/nu"), a.inst.nu);
            c.f(&format!("I/{sh}/{it}/Tt4"), a.inst.tt4);
            c.f(&format!("I/{sh}/{it}/pi_c"), a.inst.pi_c);
            c.f(&format!("I/{sh}/{it}/Phi"), a.inst.phi);
            c.f(&format!("I/{sh}/{it}/tau_t"), a.inst.tau_t);
            c.f(&format!("I/{sh}/{it}/Tt4_burner"), a.tt4_burner);
            c.f(&format!("I/{sh}/{it}/dTm_ds"), a.dtm_ds);
            c.f(&format!("I/{sh}/{it}/mdot_air"), a.inst.mdot_air);
            c.f(&format!("I/{sh}/{it}/rung35_nu"), b.nu);
            c.f(&format!("I/{sh}/{it}/rung35_pi_c"), b.pi_c);
        }
    }
    census(&mut c, "I");

    // ------------------------------------------------- J: the TWO-STATE march
    // s_end = 3.0 reaches BOTH arms of `t_accel`: the adiabatic gets there (~2.15), the cold one
    // does not.
    let j = [
        ("cold", cts.soak_excursion(&fl, 1100.0, 1400.0, Theta0::Cold, Some(&flow), 0.05, 3.0)),
        ("hot", cts.soak_excursion(&fl, 1100.0, 1400.0, Theta0::Hot, Some(&flow), 0.05, 3.0)),
        ("adiab", cts.adiabatic_excursion(&fl, 1100.0, 1400.0, Some(&flow), 0.05, 3.0)),
    ];
    for (name, r) in j {
        c.f(&format!("J/{name}/e_surge"), r.e_surge);
        c.f(&format!("J/{name}/nu0"), r.nu0);
        c.f(&format!("J/{name}/nu_final"), r.nu_final);
        c.d(&format!("J/{name}/t_accel_is_none"), u64::from(r.t_accel.is_none()));
        c.f(&format!("J/{name}/t_accel"), r.t_accel.unwrap_or(0.0));
    }
    census(&mut c, "J");

    // ------------------------------------------------- K: the both-OFF REDUCE
    let ct_off = build(flow, 0.0, 0.0, 0.0);
    let st = SpoolTransient::new(engine(), fl, 1.0, flow);
    c.f("K/plenum_K", ct_off.plenum_k);
    c.f("K/pt4_d", ct_off.pt4_d);
    c.f("K/mdot4_d", ct_off.mdot4_d);
    for (it, tt4) in [1500.0f64, 1200.0, 900.0].into_iter().enumerate() {
        let mf = st.fuel_for_tt4(&fl, tt4, None);
        let a = ct_off.inner.equilibrium_fuel(&fl, mf, None);
        let b = st.equilibrium_fuel(&fl, mf, None);
        for (k, av, bv) in [
            ("nu", a.nu, b.nu),
            ("pi_c", a.pi_c, b.pi_c),
            ("tau_t", a.tau_t, b.tau_t),
            ("Tt4", a.tt4, b.tt4),
            ("mdot_air", a.mdot_air, b.mdot_air),
        ] {
            c.f(&format!("K/{it}/ct_{k}"), av);
            c.f(&format!("K/{it}/st_{k}"), bv);
            // GATE 1's shape, asserted here rather than only compared to Python: the OFF switches
            // are exact DISPATCH, so this is a BIT identity and not a tolerance.
            assert_eq!(
                av.to_bits(),
                bv.to_bits(),
                "both-OFF CombustorTransient is not rung 35 bit-for-bit at Tt4={tt4}: {k}"
            );
        }
    }
    census(&mut c, "K");

    c.finish("all");
}
