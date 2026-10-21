//! Rung 106's scoring on the SHIPPED code (anchor § 3, reading rules frozen there): H1, H1b, H2, H3, H4, H5 and the
//! reported quantities. "Governed" = `governor_lag::march` with the governor; "open" = the same march with the open ramp
//! handed in (≡ rung 47's route bit for bit, `tests/rung106.rs`); "instant" = rung 95's governed march with the instant
//! redline. All marches called directly (the page's lag floor 0.1 would refuse the anchor's 0.05).
use std::time::Instant;
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::governor::{self as gv};
use turbojet::governor_lag::{self as gl, ClipLag, Demand};
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, SpeedLaw, ThrustBy, CONTROL_SHAPES, DS};

const TAUS: [f64; 5] = [0.05, 0.1, 0.2, 0.4, 0.8];
const RL: f64 = 1480.0;

fn base(hold: Hold) -> ControlsSettings { ControlsSettings { govern: true, redline_on: true, hold, ..ControlsSettings::defaults() } }
fn clip(p: &FuelPoint) -> (f64, f64) { match p.extra { PointExtra::Asym { g, required } => (g, required), _ => panic!() } }
fn solver(s: &ControlsSettings) -> ControlsSolver { build(&ControlsSettings { gov_lag_on: false, ..*s }).unwrap() }
fn nu0(sv: &ControlsSolver) -> (f64, f64) { (sv.start.nu_lp, sv.start.nu_hp) }
fn governed(sv: &ControlsSolver, tau: f64) -> Vec<FuelPoint> {
    gl::march(&sv.core.fuel, &sv.flight, Demand::Governor(sv.governor.as_ref().unwrap()), ClipLag { tt4_max: RL, tau },
              &sv.lag_limiters(), None, nu0(sv), sv.settings.s_end(), DS).0
}
fn open(sv: &ControlsSolver, tau: f64) -> Vec<FuelPoint> {
    let ramp = sv.settings.schedule(sv.mf_lo, sv.mf_hi);
    gl::march(&sv.core.fuel, &sv.flight, Demand::Ramp(&ramp), ClipLag { tt4_max: RL, tau }, &sv.lag_limiters(), None,
              nu0(sv), sv.settings.s_end(), DS).0
}
fn instant(sv: &ControlsSolver) -> Vec<FuelPoint> {
    gv::march(&sv.core.fuel, &sv.flight, sv.governor.as_ref().unwrap(), &sv.limiters(), None, nu0(sv), sv.settings.s_end(), DS).0
}
fn unlimited(sv: &ControlsSolver) -> Vec<FuelPoint> {
    let lim = turbojet::fuel_transient::FuelLimiters { tt4_max: None, ..sv.lag_limiters() };
    gv::march(&sv.core.fuel, &sv.flight, sv.governor.as_ref().unwrap(), &lim, None, nu0(sv), sv.settings.s_end(), DS).0
}
/// Over the stretch both reach.
fn both<'a>(a: &'a [FuelPoint], b: &'a [FuelPoint]) -> (&'a [FuelPoint], &'a [FuelPoint], bool) {
    let n = a.len().min(b.len());
    (&a[..n], &b[..n], a.len() != b.len())
}
fn peak(p: &[FuelPoint]) -> f64 { p.iter().map(|x| x.tt4).fold(f64::NEG_INFINITY, f64::max) }
fn stored(p: &[FuelPoint]) -> f64 { p.iter().map(|x| { let (g, r) = clip(x); g - r }).fold(f64::NEG_INFINITY, f64::max) }
fn engage(p: &[FuelPoint]) -> Option<usize> { p.iter().position(|x| clip(x).1 > 0.0) }
/// The 95 % crossing of the thrust step, interpolated linearly (None: never).
fn cross95(sv: &ControlsSolver, p: &[FuelPoint]) -> Option<f64> {
    let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
    let fr: Vec<f64> = p.iter().map(|x| (gv::thrust(&sv.reread(x).unwrap().base) - f0) / (f1 - f0)).collect();
    (1..fr.len()).find(|&k| fr[k] >= 0.95).map(|k| {
        let (s0, s1, a, b) = (p[k - 1].s, p[k].s, fr[k - 1], fr[k]);
        if a >= 0.95 { s0 } else { s0 + (s1 - s0) * (0.95 - a) / (b - a) }
    })
}
fn tally(name: &str, hit: usize, miss: usize, vac: usize) {
    println!("== {name}: {hit} HIT, {miss} MISS, {vac} VACUOUS");
}

fn main() {
    let t0 = Instant::now();
    // H1, H2, H3, H4 — thrust law, four maps, five lags
    let (mut h1, mut h2, mut h3, mut h4) = ([0usize; 3], [0usize; 3], [0usize; 3], [0usize; 3]);
    for shape in 0..4 {
        let s = ControlsSettings { shape, ..base(Hold::Thrust) };
        let sv = solver(&s);
        let inst = instant(&sv);
        let x_inst = cross95(&sv, &inst);
        let mut overs = Vec::new();
        for tau in TAUS {
            let (g, o) = (governed(&sv, tau), open(&sv, tau));
            let (gb, ob, cut) = both(&g, &o);
            let (pg, po) = (peak(gb) - RL, peak(ob) - RL);
            let hit1 = pg > po;
            h1[if hit1 { 0 } else { 1 }] += 1;
            overs.push(peak(&g) - RL);
            let (sg, so) = (stored(gb), stored(ob));
            let v3 = !(sg > 0.0) && !(so > 0.0);
            h3[if v3 { 2 } else if sg > so { 0 } else { 1 }] += 1;
            let xg = cross95(&sv, &g);
            let v4 = xg.is_none() || x_inst.is_none();
            let hit4 = !v4 && xg.unwrap() > x_inst.unwrap();
            h4[if v4 { 2 } else if hit4 { 0 } else { 1 }] += 1;
            println!("{:10} tau {tau:4}: pts gov {} open {}{} | H1 overshoot gov {pg:7.1} K open {po:7.1} K {} | H3 stored cut gov {sg:.3e} open {so:.3e} {} | H4 95% gov {:.4} inst {:.4} {}",
                     CONTROL_SHAPES[shape], g.len(), o.len(), if cut { " (CUT to both)" } else { "" },
                     if hit1 { "HIT" } else { "MISS" },
                     if v3 { "VACUOUS" } else if sg > so { "HIT" } else { "MISS" },
                     xg.unwrap_or(f64::NAN), x_inst.unwrap_or(f64::NAN), if v4 { "VACUOUS" } else if hit4 { "HIT" } else { "MISS" });
        }
        let mono = overs.windows(2).all(|w| w[1] > w[0]);
        h2[if mono { 0 } else { 1 }] += 1;
        println!("{:10} H2 overshoot by tau {:?} {}", CONTROL_SHAPES[shape], overs.iter().map(|x| format!("{x:.1}")).collect::<Vec<_>>(),
                 if mono { "HIT" } else { "MISS" });
        println!("{:10} rung 95 instant-redline peak {:.1} K, unlimited governed peak {:.1} K, 95% at {:.4}",
                 CONTROL_SHAPES[shape], peak(&inst), peak(&unlimited(&sv)), x_inst.unwrap_or(f64::NAN));
    }
    tally("H1 governed overshoot > open (thrust)", h1[0], h1[1], h1[2]);
    tally("H2 overshoot rises with tau", h2[0], h2[1], h2[2]);
    tally("H3 governed stored cut > open", h3[0], h3[1], h3[2]);
    tally("H4 95% thrust later than instant", h4[0], h4[1], h4[2]);

    // H1b — every other law at tau 0.2
    let laws: [(&str, ControlsSettings); 5] = [
        ("HP rate .1", ControlsSettings { speed_law: SpeedLaw::Rate, gov_tau: 0.1, ..base(Hold::HpSpeed) }),
        ("HP prop 20", ControlsSettings { speed_law: SpeedLaw::Prop, gov_gain: 20.0, ..base(Hold::HpSpeed) }),
        ("EPR", ControlsSettings { thrust_by: ThrustBy::Epr, ..base(Hold::Thrust) }),
        ("N1 rate .2", ControlsSettings { thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.2, ..base(Hold::Thrust) }),
        ("N1 prop 5", ControlsSettings { thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 5.0, ..base(Hold::Thrust) }),
    ];
    let mut h1b = [0usize; 3];
    for (name, ls) in laws {
        for shape in 0..4 {
            let sv = solver(&ControlsSettings { shape, ..ls });
            let (g, o) = (governed(&sv, 0.2), open(&sv, 0.2));
            let (gb, ob, cut) = both(&g, &o);
            let (pg, po) = (peak(gb) - RL, peak(ob) - RL);
            h1b[if pg > po { 0 } else { 1 }] += 1;
            println!("H1b {name:10} {:10}: overshoot gov {pg:7.1} K open {po:7.1} K{} {}", CONTROL_SHAPES[shape],
                     if cut { " (CUT)" } else { "" }, if pg > po { "HIT" } else { "MISS" });
        }
    }
    tally("H1b other laws, tau 0.2", h1b[0], h1b[1], h1b[2]);

    // H5 — engagement earlier (independent of tau before engagement: read at 0.2)
    let mut h5 = [0usize; 3];
    for shape in 0..4 {
        let sv = solver(&ControlsSettings { shape, ..base(Hold::Thrust) });
        let (g, o) = (governed(&sv, 0.2), open(&sv, 0.2));
        match (engage(&g), engage(&o)) {
            (Some(a), Some(b)) => {
                h5[if a < b { 0 } else { 1 }] += 1;
                println!("H5 {:10}: engages at point {a} (s={:.2}) governed, {b} (s={:.2}) open {}", CONTROL_SHAPES[shape],
                         g[a].s, o[b].s, if a < b { "HIT" } else { "MISS" });
            }
            x => { h5[2] += 1; println!("H5 {:10}: VACUOUS {x:?}", CONTROL_SHAPES[shape]); }
        }
    }
    tally("H5 engages earlier governed", h5[0], h5[1], h5[2]);

    // Reported: rung 47's reliefs under the governor (lagged vs unlimited governed), lowest phi each compressor
    println!("-- reported: reliefs (lowest phi, lagged governed minus unlimited governed; open lagged minus open unlimited)");
    for shape in 0..4 {
        let sv = solver(&ControlsSettings { shape, ..base(Hold::Thrust) });
        let un = unlimited(&sv);
        let ramp = sv.settings.schedule(sv.mf_lo, sv.mf_hi);
        let o_un = gl::march(&sv.core.fuel, &sv.flight, Demand::Ramp(&ramp), ClipLag { tt4_max: 1e9, tau: 0.2 }, &sv.lag_limiters(),
                             None, nu0(&sv), sv.settings.s_end(), DS).0;
        let low = |p: &[FuelPoint], f: fn(&FuelPoint) -> f64| p.iter().map(f).fold(f64::INFINITY, f64::min);
        for tau in TAUS {
            let (g, o) = (governed(&sv, tau), open(&sv, tau));
            println!("{:10} tau {tau:4}: relief_lp gov {:+.6} open {:+.6}; relief_hp gov {:+.6} open {:+.6}", CONTROL_SHAPES[shape],
                     low(&g, |x| x.phi_lp) - low(&un, |x| x.phi_lp), low(&o, |x| x.phi_lp) - low(&o_un, |x| x.phi_lp),
                     low(&g, |x| x.phi_hp) - low(&un, |x| x.phi_hp), low(&o, |x| x.phi_hp) - low(&o_un, |x| x.phi_hp));
        }
    }
    // Reported: the burner-temperature hold to 1600 K (above the redline), overshoot governed vs open, and the clip peak
    println!("-- reported: burner-temperature hold 1000 -> 1600 K");
    for shape in 0..4 {
        let sv = solver(&ControlsSettings { shape, to: 1600.0, ..base(Hold::BurnerTemp) });
        for tau in [0.1, 0.4] {
            let (g, o) = (governed(&sv, tau), open(&sv, tau));
            println!("{:10} tau {tau}: overshoot gov {:.1} open {:.1}; clip peak gov {:.3e} open {:.3e}", CONTROL_SHAPES[shape],
                     peak(&g) - RL, peak(&o) - RL, g.iter().map(|x| clip(x).0).fold(0.0, f64::max), o.iter().map(|x| clip(x).0).fold(0.0, f64::max));
        }
    }
    // Reported: where H4's sign flips at long lags (flow/press, flat-LP), and the Tt4 hold's overshoot to more digits
    println!("-- reported: H4 at long lags (95% crossing, governed lagged vs instant)");
    for shape in [0usize, 3] {
        let sv = solver(&ControlsSettings { shape, ..base(Hold::Thrust) });
        let xi = cross95(&sv, &instant(&sv)).unwrap();
        let xu = cross95(&sv, &unlimited(&sv)).unwrap();
        let row: Vec<String> = [0.4, 0.6, 0.8, 1.0, 1.2, 1.6, 2.4, 3.2, 6.4].iter()
            .map(|&t| format!("{t}:{:.3}", cross95(&sv, &governed(&sv, t)).unwrap_or(f64::NAN))).collect();
        println!("{:10} instant {xi:.3}, unlimited {xu:.3}; lagged {}", CONTROL_SHAPES[shape], row.join(" "));
    }
    println!("-- reported: the thrust fraction's first peak and the sag after it");
    for shape in 0..4 {
        println!("{}", CONTROL_SHAPES[shape]);
        let sv = solver(&ControlsSettings { shape, ..base(Hold::Thrust) });
        let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
        let sag = |name: &str, g: &[FuelPoint]| {
            let fr: Vec<f64> = g.iter().map(|x| (gv::thrust(&sv.reread(x).unwrap().base) - f0) / (f1 - f0)).collect();
            match (1..fr.len()).find(|&k| fr[k] < fr[k - 1]) {
                None => println!("{name}: thrust never falls ({} pts)", fr.len()),
                Some(k) => {
                    let lo = fr[k..].iter().cloned().fold(f64::INFINITY, f64::min);
                    println!("{name}: first falls after s={:.2} at {:.4}, lowest after {:.4}", g[k - 1].s, fr[k - 1], lo);
                }
            }
        };
        sag("instant (rung 95 + instant redline)", &instant(&sv));
        sag("unlimited governed", &unlimited(&sv));
        for t in [0.2, 0.8] { sag(&format!("open lagged (rung 47) tau {t}"), &open(&sv, t)); }
        for t in [0.2, 0.4, 0.8, 0.9, 1.0, 1.6] {
            let g = governed(&sv, t);
            let fr: Vec<f64> = g.iter().map(|x| (gv::thrust(&sv.reread(x).unwrap().base) - f0) / (f1 - f0)).collect();
            let kp = (1..fr.len()).find(|&k| fr[k] < fr[k - 1]).map(|k| k - 1).unwrap_or(fr.len() - 1);
            let (km, fm) = fr.iter().enumerate().skip(kp).fold((kp, f64::INFINITY), |m, (k, &v)| if v < m.1 { (k, v) } else { m });
            println!("tau {t}: first peak {:.4} at s={:.2}, then down to {:.4} at s={:.2}", fr[kp], g[kp].s, fm, g[km].s);
        }
    }
    for shape in 0..4 {
        let sv = solver(&ControlsSettings { shape, to: 1600.0, ..base(Hold::BurnerTemp) });
        let g = governed(&sv, 0.1);
        let k = (0..g.len()).max_by(|&i, &j| g[i].tt4.total_cmp(&g[j].tt4)).unwrap();
        println!("{:10} Tt4 hold tau 0.1: peak {:.6} K at s={:.2}", CONTROL_SHAPES[shape], g[k].tt4, g[k].s);
    }
    // Reported: cost on the page's default lag
    let sv = solver(&base(Hold::Thrust));
    let t = Instant::now(); let _ = governed(&sv, 0.2); let tg = t.elapsed().as_secs_f64();
    let t = Instant::now(); let _ = instant(&sv); let ti = t.elapsed().as_secs_f64();
    println!("-- cost: governed lagged {tg:.3} s, rung 95 instant redline {ti:.3} s; scoring took {:.0} s", t0.elapsed().as_secs_f64());
}
