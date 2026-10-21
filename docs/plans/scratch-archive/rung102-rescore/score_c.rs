// Part C SCORING — the addendum's § 5.3 cells (C1–C12) on the shipped Controls view.
use turbojet::afterburner::AbMode;
use turbojet::atmosphere::standard;
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::FuelPoint;
use turbojet::sandbox_controls::{classify, controls, ControlsSettings, ControlsSolver};
use turbojet::two_spool::Spool;
use turbojet::turbine_law::TurbineLaw;

struct Run { sv: ControlsSolver, pts: Vec<FuelPoint>, stop: Option<String> }

fn run(s: &ControlsSettings) -> Result<Run, String> {
    let (sv, o) = controls(s).map_err(|e| e.0)?;
    let stop = o.stop.as_ref().map(|st| format!("{:?}: {}", classify(&sv, st, o.points.last()),
                                                 st.failure.as_ref().map_or("?".into(), |f| f.message.clone())));
    Ok(Run { sv, pts: o.points, stop })
}

/// [LP flow, LP pressure, HP flow, HP pressure] margins at a point.
fn margins(sv: &ControlsSolver, p: &FuelPoint) -> [f64; 4] {
    let i = sv.reread(p).expect("rereads");
    let c = &i.base.close;
    let mc = &sv.core.fuel.inner.inner;
    let (vl, sl) = sv.stall(Spool::Lp, p.nu_lp, p.nu_hp);
    let (vh, sh) = sv.stall(Spool::Hp, p.nu_lp, p.nu_hp);
    let ml = sv.core.design_map(Spool::Lp).with_vsv(vl);
    let mh = sv.core.design_map(Spool::Hp).with_vsv(vh);
    let pl = mc.pi_c_spool(&ml, mc.tau_lpc_d, mc.base.eta_lpc, c.n_lp, ml.phi_surge_at(), c.tt2);
    let ph = mc.pi_c_spool(&mh, mc.tau_hpc_d, mc.base.eta_hpc, c.n_hp, mh.phi_surge_at(), c.tt25);
    [c.phi_lp / sl - 1.0, pl / c.pi_lpc - 1.0, c.phi_hp / sh - 1.0, ph / c.pi_hpc - 1.0]
}

struct Tally { hit: usize, miss: usize, lines: Vec<String> }
impl Tally {
    fn cell(&mut self, id: &str, ok: bool, what: String) {
        if ok { self.hit += 1 } else { self.miss += 1 }
        self.lines.push(format!("{} {id}: {what}", if ok { "HIT " } else { "MISS" }));
    }
    fn miss_all(&mut self, ids: &[&str], why: &str) {
        for id in ids { self.cell(id, false, why.to_string()); }
    }
}

const HELD: [&str; 10] = ["C2", "C2", "C3", "C4", "C5", "C6", "C7", "C7", "C8", "C8"];

fn main() {
    let law = TurbineLaw::from_key(&std::env::args().nth(1).expect("law key")).unwrap();
    println!("LAW {}", law.key());
    let d = ControlsSettings { turbine: law, ..ControlsSettings::defaults() };
    let (t5k, p5k) = standard(5000.0);
    let mut t = Tally { hit: 0, miss: 0, lines: Vec::new() };
    let names = ["flow/press", "press/flow", "tilted", "flat-lp"];
    for shape in [1usize, 2, 3] {
        for rho in [0.5, 3.0] {
            for (fl, t0, p0, m0) in [("design", d.t0, d.p0, d.m0), ("5km M0.6", t5k, p5k, 0.6)] {
                for s_on in [0.30, 3.50] {
                    let base = ControlsSettings { shape, rho, t0, p0, m0, from: 900.0, to: 1350.0, settle: 6.0, tt7: 1900.0,
                                                  light_at: s_on, ..d };
                    let cell = format!("{} rho {rho} {fl} light {s_on}", names[shape]);
                    let u = run(&ControlsSettings { ab: Some(AbMode::Unlit), ..base }).expect("unlit runs (grid_dry)");
                    let lit = |th: ControlsSettings| run(&ControlsSettings { ab: Some(AbMode::Lit), ..th });
                    let h = lit(base);
                    let h12 = lit(ControlsSettings { a8_ratio: 1.2, ..base });
                    let m = lit(ControlsSettings { a8_match: true, ..base });
                    let ue = u.pts.last().unwrap();
                    let ur = u.sv.read(ue).unwrap();
                    let um = margins(&u.sv, ue);
                    // C1 — match.
                    match &m {
                        Ok(m) if m.stop.is_none() => {
                            let me = m.pts.last().unwrap();
                            let ratio = m.sv.read(me).unwrap().thrust / ur.thrust;
                            let bar = (1900.0 / ur.tt5).sqrt();
                            t.cell("C1", ratio > bar, format!("{cell}: F ratio {ratio:.4} vs sqrt(Tt7/Tt5) {bar:.4}"));
                            let same = m.pts.len() == u.pts.len()
                                && m.pts.iter().zip(&u.pts).all(|(a, b)| a.nu_lp.to_bits() == b.nu_lp.to_bits()
                                    && a.nu_hp.to_bits() == b.nu_hp.to_bits() && a.tt4.to_bits() == b.tt4.to_bits());
                            t.lines.push(format!("GATE match = unlit bit for bit: {same} ({cell})"));
                        }
                        Ok(m) => t.cell("C1", false, format!("{cell}: match STOPPED {}", m.stop.as_ref().unwrap())),
                        Err(e) => t.cell("C1", false, format!("{cell}: match REFUSED {e}")),
                    }
                    // C2–C8 — held.
                    let mut held_d = None;
                    match &h {
                        Ok(h) if h.stop.is_none() => {
                            let he = h.pts.last().unwrap();
                            let hr = h.sv.read(he).unwrap();
                            let hm = margins(&h.sv, he);
                            let (dl, dh, dt, df) = (he.nu_lp - ue.nu_lp, he.nu_hp - ue.nu_hp, he.tt4 - ue.tt4, hr.thrust - ur.thrust);
                            held_d = Some((dl, dh, dt));
                            t.cell("C2", dl < 0.0, format!("{cell}: dnuL {dl:.5}"));
                            let rl = dl / ue.nu_lp;
                            t.cell("C2", (-0.50..=-0.08).contains(&rl), format!("{cell}: dnuL/nuL {:.2} %", 100.0 * rl));
                            let rh = dh / ue.nu_hp;
                            t.cell("C3", (0.02..=0.20).contains(&rh), format!("{cell}: dnuH/nuH {:.2} %", 100.0 * rh));
                            t.cell("C4", dt > 0.0, format!("{cell}: dTt4 {dt:.1} K"));
                            t.cell("C5", df > 0.0, format!("{cell}: dF {df:.2} ({:.2} %)", 100.0 * df / ur.thrust));
                            let win = |r: &Run| r.pts.iter().filter(|p| p.s >= s_on - 1e-9).map(|p| margins(&r.sv, p)[1])
                                .fold(f64::INFINITY, f64::min);
                            let (lmin, umin) = (win(h), win(&u));
                            t.cell("C6", lmin < umin, format!("{cell}: LP pressure-margin min after light-off {lmin:.4} vs unlit {umin:.4}"));
                            t.cell("C7", hm[0] > um[0], format!("{cell}: end LP flow margin {:.4} vs {:.4}", hm[0], um[0]));
                            t.cell("C7", hm[1] < um[1], format!("{cell}: end LP pressure margin {:.4} vs {:.4}", hm[1], um[1]));
                            t.cell("C8", hm[2] > um[2], format!("{cell}: end HP flow margin {:.4} vs {:.4}", hm[2], um[2]));
                            t.cell("C8", hm[3] > um[3], format!("{cell}: end HP pressure margin {:.4} vs {:.4}", hm[3], um[3]));
                        }
                        Ok(h) => t.miss_all(&HELD, &format!("{cell}: held STOPPED {}", h.stop.as_ref().unwrap())),
                        Err(e) => t.miss_all(&HELD, &format!("{cell}: held REFUSED {e}")),
                    }
                    // C9 — x1.2 strictly between.
                    match (&h12, held_d) {
                        (Ok(x), Some((dl, dh, dt))) if x.stop.is_none() => {
                            let xe = x.pts.last().unwrap();
                            for (nm, a, b) in [("nuL", xe.nu_lp - ue.nu_lp, dl), ("nuH", xe.nu_hp - ue.nu_hp, dh), ("Tt4", xe.tt4 - ue.tt4, dt)] {
                                t.cell("C9", 0.0 < a.abs() && a.abs() < b.abs(), format!("{cell}: {nm} d(x1.2) {a:.5} vs d(x1) {b:.5}"));
                            }
                        }
                        (Ok(x), _) if x.stop.is_some() => t.miss_all(&["C9"; 3], &format!("{cell}: x1.2 STOPPED {}", x.stop.as_ref().unwrap())),
                        (Err(e), _) => t.miss_all(&["C9"; 3], &format!("{cell}: x1.2 REFUSED {e}")),
                        _ => t.miss_all(&["C9"; 3], &format!("{cell}: held stopped, no comparison")),
                    }
                    eprintln!("done {cell}");
                }
            }
        }
    }
    // C10 — below choke.
    for (shape, from) in [(1usize, 650.0), (2, 650.0), (3, 700.0)] {
        let base = ControlsSettings { shape, from, to: 1350.0, m0: 0.5, settle: 6.0, tt7: 1900.0, light_at: 0.06, ..d };
        let cell = format!("{} from {from} M0.5 light 0.06", names[shape]);
        let u = run(&ControlsSettings { ab: Some(AbMode::Unlit), ..base }).expect("unlit runs");
        let ue = u.pts.last().unwrap();
        let ur = u.sv.read(ue).unwrap();
        t.lines.push(format!("     C10 {cell}: unlit at 0.06/0.10 branch {:?}/{:?}", u.pts[3].branch, u.pts[5].branch));
        match run(&ControlsSettings { ab: Some(AbMode::Lit), ..base }) {
            Ok(h) if h.stop.is_none() => {
                let he = h.pts.last().unwrap();
                t.cell("C10", true, format!("{cell}: held completes (lit at 0.10 branch {:?})", h.pts[5].branch));
                t.cell("C10", he.nu_lp < ue.nu_lp, format!("{cell}: dnuL {:.5}", he.nu_lp - ue.nu_lp));
                t.cell("C10", he.nu_hp > ue.nu_hp, format!("{cell}: dnuH {:.5}", he.nu_hp - ue.nu_hp));
                t.cell("C10", he.tt4 > ue.tt4, format!("{cell}: dTt4 {:.1}", he.tt4 - ue.tt4));
            }
            Ok(h) => t.miss_all(&["C10"; 4], &format!("{cell}: held STOPPED {}", h.stop.as_ref().unwrap())),
            Err(e) => t.miss_all(&["C10"; 4], &format!("{cell}: held REFUSED {e}")),
        }
        match run(&ControlsSettings { ab: Some(AbMode::Lit), a8_match: true, ..base }) {
            Ok(m) if m.stop.is_none() => {
                let ratio = m.sv.read(m.pts.last().unwrap()).unwrap().thrust / ur.thrust;
                let bar = (1900.0 / ur.tt5).sqrt();
                t.cell("C10", ratio > bar, format!("{cell}: match F ratio {ratio:.4} vs {bar:.4}"));
            }
            Ok(m) => t.cell("C10", false, format!("{cell}: match STOPPED {}", m.stop.as_ref().unwrap())),
            Err(e) => t.cell("C10", false, format!("{cell}: match REFUSED {e}")),
        }
    }
    // C11 — the HP-speed governor.
    for shape in [1usize, 2, 3] {
        let base = ControlsSettings { shape, govern: true, hold: Hold::HpSpeed, from: 900.0, to: 1350.0, settle: 6.0,
                                      tt7: 1900.0, light_at: 0.30, ..d };
        let cell = format!("{} governed HP speed", names[shape]);
        let u = run(&ControlsSettings { ab: Some(AbMode::Unlit), ..base }).expect("unlit runs");
        let ue = u.pts.last().unwrap();
        match run(&ControlsSettings { ab: Some(AbMode::Lit), ..base }) {
            Ok(h) if h.stop.is_none() => {
                let he = h.pts.last().unwrap();
                let rh = (he.nu_hp - ue.nu_hp) / ue.nu_hp;
                let rl = (he.nu_lp - ue.nu_lp) / ue.nu_lp;
                t.cell("C11", rh.abs() <= 0.005, format!("{cell}: dnuH/nuH {:.3} %", 100.0 * rh));
                t.cell("C11", rl < -0.05, format!("{cell}: dnuL/nuL {:.2} %", 100.0 * rl));
            }
            Ok(h) => t.miss_all(&["C11"; 2], &format!("{cell}: STOPPED {}", h.stop.as_ref().unwrap())),
            Err(e) => t.miss_all(&["C11"; 2], &format!("{cell}: REFUSED {e}")),
        }
    }
    // C12 — the redline through a late held light-off.
    for shape in [1usize, 2, 3] {
        let base = ControlsSettings { shape, redline_on: true, from: 900.0, to: 1350.0, settle: 6.0, tt7: 1900.0,
                                      light_at: 3.50, ..d };
        let cell = format!("{} redline, held light 3.50", names[shape]);
        match run(&ControlsSettings { ab: Some(AbMode::Lit), ..base }) {
            Ok(h) if h.stop.is_none() => {
                let mx = h.pts.iter().map(|p| p.tt4).fold(0.0, f64::max);
                let binds = h.pts.iter().filter(|p| p.s > 3.5).any(|p| (p.tt4 - 1480.0).abs() <= 1.0);
                t.cell("C12", mx <= 1480.5 && binds, format!("{cell}: completes, max Tt4 {mx:.2}, binds after light-off {binds}"));
            }
            Ok(h) => t.cell("C12", false, format!("{cell}: STOPPED {}", h.stop.as_ref().unwrap())),
            Err(e) => t.cell("C12", false, format!("{cell}: REFUSED {e}")),
        }
    }
    for l in &t.lines { println!("{l}"); }
    println!("SCORE: {} HIT, {} MISS of {}", t.hit, t.miss, t.hit + t.miss);
}
