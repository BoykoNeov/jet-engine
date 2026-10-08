// ---------------------------------------------------------------------------- the rig
//
// `tests/test_rung84.py`'s module constants (identical to `test_rung83.py`'s).

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const SM: f64 = 0.80 / FLOOR - 1.0;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const TT4_MAX: f64 = 1200.0;
const PHI_FUEL: f64 = 0.75;
const PHI_AIR: Option<f64> = Some(0.77);
const DS: f64 = 0.005;
/// § 3.2's jump window, `r = 0.25`.
const JUMP_LO: f64 = 0.0197750;
const JUMP_HI: f64 = 0.0197875;
/// § 3.4's crossing window, `r = 0.35`.
const CROSS_LO: f64 = 0.037000;
const CROSS_HI: f64 = 0.037333;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

/// Python's `_rig`, with its four knob assignments. The class is `StaircaseLawTransient` there;
/// here it is a rung-80 core, because rung 84 adds no cell (plan § 5.34 (ii)).
fn rig() -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))),
        ..Default::default()
    };
    let m = match build_split_wall_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    };
    let t = &m.fuel.inner;
    t.lag_coord.set("demand");
    t.ref_law.set("sched");
    t.windup_law.set("none");
    t.cap_law.set("solve");
    m
}

/// `test_rung84.py`'s `_kw(r, ds)`.
fn kw(f: &FlightCondition, r: f64, ds: f64) -> ScanKw<'_> {
    ScanKw {
        flight: f, tt4_lo: LO, tt4_hi: HI, tt4_max: TT4_MAX, phi_lim: PHI_FUEL, phi_air: PHI_AIR,
        tau_gov: 0.05, tau_q: 0.05, tau_s: 0.05, r, s_settle: 1.2, ds, v_max: 0.20,
        inc: false,
    }
}

// ---------------------------------------------------------------------------- the flattening

#[derive(Default)]
struct Flat(Vec<(String, String)>);

impl Flat {
    fn put(&mut self, p: &str, t: String) {
        self.0.push((p.to_string(), t));
    }
    fn f(&mut self, p: &str, x: f64) {
        if x.is_nan() {
            self.put(p, "f:nan".into());
        } else {
            self.put(p, format!("f:{:016x}", x.to_bits()));
        }
    }
    fn of(&mut self, p: &str, x: Option<f64>) {
        match x {
            Some(x) => self.f(p, x),
            None => self.none(p),
        }
    }
    fn i(&mut self, p: &str, x: usize) {
        self.put(p, format!("i:{x}"));
    }
    fn b(&mut self, p: &str, x: bool) {
        self.put(p, format!("b:{}", x as u8));
    }
    fn ob(&mut self, p: &str, x: Option<bool>) {
        match x {
            Some(x) => self.b(p, x),
            None => self.none(p),
        }
    }
    fn s(&mut self, p: &str, x: &str) {
        self.put(p, format!("s:{x}"));
    }
    fn os(&mut self, p: &str, x: Option<&str>) {
        match x {
            Some(x) => self.s(p, x),
            None => self.none(p),
        }
    }
    fn none(&mut self, p: &str) {
        self.put(p, "n".into());
    }
    fn len(&mut self, p: &str, n: usize) {
        self.put(p, format!("len:{n}"));
    }
    fn keys(&mut self, p: &str, n: usize) {
        self.put(p, format!("keys:{n}"));
    }
    fn fs(&mut self, p: &str, xs: &[f64]) {
        self.len(p, xs.len());
        for (k, &x) in xs.iter().enumerate() {
            self.f(&format!("{p}.{k}"), x);
        }
    }
}

