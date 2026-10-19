// Probe: why rung 40's equilibrium refuses running engines, and which fix reaches them.
//   OLD   — try_equilibrium(None), start (1,1)
//   TRACE — the same Newton re-run outside, printing where the first Err comes from
//   STEADY— rung 39's matcher with rung 103's switch on: (n_lp_ratio, n_hp_ratio)
//   SEED  — try_equilibrium(Some(steady)) : converges? same speeds as steady?
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{control_maps, CONTROL_SHAPES};
use turbojet::two_spool_transient::TwoSpoolTransientCore;
use turbojet::atmosphere::standard;

fn trace(tr: &TwoSpoolTransientCore, fl: &FlightCondition, tt4: f64) -> String {
    let (tt2, pt2, _) = tr.inlet(fl);
    let f = |a: f64, b: f64| -> Result<(f64, f64), String> {
        let c = tr.try_close(a, b, tt4, tt2, pt2).map_err(|e| e.0)?;
        tr.powers(&c, fl, a, b, tt4).map_err(|e| e.0)
    };
    let (mut nl, mut nh) = (1.0f64, 1.0f64);
    let mut s = String::new();
    for it in 0..60 {
        let (fl_, fh) = match f(nl, nh) { Ok(v) => v, Err(e) => return format!("{s} it{it} BASE-ERR at ({nl:.4},{nh:.4}): {}", &e[..e.len().min(40)]) };
        if fl_.abs().max(fh.abs()) < 1e-9 { return format!("{s} conv it{it}"); }
        let h = 1e-6;
        let (al, ah) = match f(nl + h, nh) { Ok(v) => v, Err(e) => return format!("{s} it{it} DL-ERR {}", &e[..e.len().min(40)]) };
        let (bl, bh) = match f(nl, nh + h) { Ok(v) => v, Err(e) => return format!("{s} it{it} DH-ERR {}", &e[..e.len().min(40)]) };
        let (j11, j12, j21, j22) = ((al - fl_) / h, (bl - fl_) / h, (ah - fh) / h, (bh - fh) / h);
        let det = j11 * j22 - j12 * j21;
        let dl = (-fl_ * j22 + fh * j12) / det;
        let dh = (-j11 * fh + j21 * fl_) / det;
        let damp = 1.0f64.min(0.25 / dl.abs().max(dh.abs()).max(1e-30));
        s += &format!(" [{nl:.3},{nh:.3} r{:.1e}]", fl_.abs().max(fh.abs()));
        nl += damp * dl; nh += damp * dh;
    }
    s
}

fn main() {
    let bs = BladeSettings::defaults();
    let fd = bs.flight();
    let (t_sl, p_sl) = standard(0.0);
    let flights = [("SL+40 M0.85", FlightCondition::new(t_sl + 40.0, p_sl, 0.85)),
                   ("SL std M0.85", FlightCondition::new(t_sl, p_sl, 0.85)),
                   ("SL std M0.30", FlightCondition::new(t_sl, p_sl, 0.30)),
                   ("design flight", fd)];
    let verbose = std::env::args().any(|a| a == "-v");
    for shape in CONTROL_SHAPES {
        let (ml, mh) = control_maps(shape).unwrap();
        let tr = TwoSpoolTransientCore::new(bs.design(), fd, 1.0, ml, mh, 1.0);
        tr.inner.base.below_choke.set(true);
        for (fname, fl) in flights.iter() {
            println!("\n=== {shape} | {fname}");
            let mut tt4 = 550.0;
            while tt4 <= 1500.0 + 1e-9 {
                let old = tr.try_equilibrium(fl, tt4, None);
                let st = tr.inner.try_match_point(fl, tt4);
                let mut line = format!("Tt4 {tt4:5.0}");
                match &old { Ok((i, ex, it)) => line += &format!(" OLD ok {:?}/{} nu ({:.6},{:.6})", ex, it, i.nu_lp, i.nu_hp),
                             Err(e) => line += &format!(" OLD ERR {}", &e.0[..e.0.len().min(34)]) }
                match &st {
                    Ok(m) => {
                        line += &format!(" | ST ({:.6},{:.6}) sp {:.1}", m.n_lp_ratio, m.n_hp_ratio, m.base.thrust);
                        match tr.try_equilibrium(fl, tt4, Some((m.n_lp_ratio, m.n_hp_ratio))) {
                            Ok((i, ex, it)) => line += &format!(" | SEED ok {:?}/{} d ({:.1e},{:.1e})", ex, it, i.nu_lp - m.n_lp_ratio, i.nu_hp - m.n_hp_ratio),
                            Err(e) => line += &format!(" | SEED ERR {}", &e.0[..e.0.len().min(40)]),
                        }
                        if let Ok((i, _, _)) = &old { line += &format!(" | OLD-ST ({:.1e},{:.1e})", i.nu_lp - m.n_lp_ratio, i.nu_hp - m.n_hp_ratio); }
                    }
                    Err(e) => line += &format!(" | ST ERR {}", &e.0[..e.0.len().min(40)]),
                }
                println!("{line}");
                if verbose && old.is_err() && st.is_ok() { println!("      {}", trace(&tr, fl, tt4)); }
                tt4 += 50.0;
            }
        }
    }
}
