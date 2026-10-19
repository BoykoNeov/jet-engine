// Mattingly 2nd ed. §8.4 afterburning-turbojet REFERENCE (SLS, max wet), single-spool equivalent.
use turbojet::components::Nozzle;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{FlowState, Gas, GasSpec};

const BTU_LBM_R: f64 = 4186.8;
const BTU_LBM: f64 = 2326.0;
const R_PER_K: f64 = 1.0 / 1.8;

fn main() {
    let cpc = 0.24 * BTU_LBM_R; let gc: f64 = 1.4; let rc = (gc - 1.0) / gc * cpc;
    let cpt = 0.295 * BTU_LBM_R; let gt: f64 = 1.3; let rt = (gt - 1.0) / gt * cpt;
    let hpr = 18_400.0 * BTU_LBM;
    let t0 = 518.7 * R_PER_K; let p0 = 101_325.0;
    let (tt4, tt7) = (3200.0 * R_PER_K, 3600.0 * R_PER_K);
    let (pi_c, ec, et, pi_dmax, pi_b, pi_n, pi_ab, eta_b, eta_ab, eta_m) =
        (20.0f64, 0.9f64, 0.9f64, 0.98f64, 0.96, 0.98, 0.94, 0.995, 0.95, 0.995);
    println!("SI: cpc {cpc:.2} Rc {rc:.3} cpt {cpt:.2} Rt {rt:.3} hPR {hpr:.0} T0 {t0:.3} Tt4 {tt4:.2} Tt7 {tt7:.2}");

    // ---- Mattingly closed form (7.20 / 7.33 / 8.45), P0/P9 swept --------------------------------
    let a0 = (gc * rc * t0).sqrt();
    let tau_r = 1.0; let pi_r = 1.0;
    let tau_c = pi_c.powf((gc - 1.0) / (gc * ec));
    let eta_c = (pi_c.powf((gc - 1.0) / gc) - 1.0) / (tau_c - 1.0);
    let tau_lam = cpt * tt4 / (cpc * t0);
    let f = (tau_lam - tau_r * tau_c) / (hpr * eta_b / (cpc * t0) - tau_lam);
    let tau_t = 1.0 - tau_r * (tau_c - 1.0) / (eta_m * (1.0 + f) * tau_lam);
    let pi_t = tau_t.powf(gt / ((gt - 1.0) * et));
    let tau_lab = cpt * tt7 / (cpc * t0);
    let fab = (1.0 + f) * (tau_lab - tau_lam * tau_t) / (eta_ab * hpr / (cpc * t0) - tau_lab);
    println!("closed form: eta_c {eta_c:.4} tau_c {tau_c:.5} f {f:.5} tau_t {tau_t:.5} pi_t {pi_t:.5} fAB {fab:.5} fo {:.5}", f + fab);
    println!("  book: tau_tH*tau_tL {:.5} pi_tH*pi_tL {:.5} f 0.0358 fAB 0.0195 fo 0.0554", 0.8821 * 0.9033, 0.5466 * 0.6127);
    for p0p9 in [1.0, 1.02, 1.04, 1.05, 1.06] {
        let pt9_p9 = pi_r * pi_dmax * pi_c * pi_b * pi_t * pi_ab * pi_n * p0p9;
        let m9 = ((2.0 / (gt - 1.0)) * (pt9_p9.powf((gt - 1.0) / gt) - 1.0)).sqrt();
        let t9_t0 = (tt7 / t0) / pt9_p9.powf((gt - 1.0) / gt);
        let v9_a0 = m9 * (gt * rt * t9_t0 / (gc * rc)).sqrt();
        let ft = 1.0 + f + fab;
        let st = a0 * (ft * v9_a0 - 0.0 + ft * (rt / rc) * t9_t0 / v9_a0 * (1.0 - p0p9) / gc);
        let s = (f + fab) / st;
        println!("  P0/P9 {p0p9:.2}: M9 {m9:.4}  F/m0 {st:.2} N.s/kg ({:.3} lbf/(lbm/s))  S {:.3} mg/N.s ({:.4} (lbm/h)/lbf)",
                 st / 9.80665, s * 1e6, s * 1e6 / 28.3250);
    }
    println!("  book: M9 1.85  F/m0 {:.2} N.s/kg (25000/181.57 = {:.3})  S 1.4473 (lbm/h)/lbf = {:.3} mg/N.s",
             25000.0 / 181.57 * 9.80665, 25000.0 / 181.57, 1.4473 * 28.3250);

    // ---- The model: build_turbojet on a dual CPG gas + an afterburner beside it ------------------
    let gas = Gas::new(GasSpec { gamma_c: gc, cp_c: cpc, r_c: rc, gamma_t: gt, cp_t: cpt, r_t: rt, hpr,
                                 ..GasSpec::default() });
    let l = Losses { pi_d: pi_dmax, eta_c: 1.0, e_c: Some(ec), eta_b, pi_b, eta_t: 1.0, e_t: Some(et), eta_m, pi_n,
                     ..Losses::default() };
    let fl = FlightCondition::new(t0, p0, 1e-3);
    let eng = build_turbojet(gas, pi_c, tt4, p0, l);
    let r = eng.run(&fl, 1.0);
    let s5 = *r.station("5"); let s4 = *r.station("4");
    let g = &eng.gas;
    let h5 = g.h_t(s5.tt, s5.far); let h7 = g.h_t(tt7, s5.far);
    let f7 = s5.far + (1.0 + s5.far) * (h7 - h5) / (eta_ab * g.hpr() - h7);
    let s7 = FlowState { tt: tt7, pt: pi_ab * s5.pt, mdot: 1.0 + f7, far: f7 };
    let ex = Nozzle::new(p0, pi_n, None).apply(&s7, g);
    let st = (1.0 + f7) * ex.v9 - r.v0;
    let exd = Nozzle::new(p0, pi_n, None).apply(&s5, g);
    let std = (1.0 + s5.far) * exd.v9 - r.v0;
    println!("dry: F/m0 {std:.2}  S {:.3}  wet/dry thrust {:.4}  S ratio {:.4}", s5.far / std * 1e6, st / std, (f7 / st) / (s5.far / std));
    println!("model (M0 1e-3, P0/P9 1): f {:.5} tau_t {:.5} pi_t {:.5} fAB {:.5} M9 {:.4} F/m0 {st:.2} S {:.3} mg/N.s",
             s4.far, s5.tt / s4.tt, s5.pt / s4.pt, f7 - s4.far, ex.m9, f7 / st * 1e6);
}
// (appended) dry = AB off: the core into the nozzle, no AB loss (Mattingly 8.45q)
#[allow(dead_code)]
fn _unused() {}
