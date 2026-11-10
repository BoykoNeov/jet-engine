//! Rung 107's scoring on the SHIPPED code (anchor § 3, reading rules frozen there): H1–H6 and the reported quantities.
//! "Governed" = `governor_release::march` with the governor; "open" = the same march with the open ramp handed in (≡ rung
//! 52's route bit for bit, `tests/rung107.rs`); "instant" = rung 95's governed march with the instant schedule / floor.
//! All marches called directly (1.6 lies past the page's release slider).
//!
//! Vacuity rules as coded (written BEFORE the first run — the advisor's check):
//! * a run that never reaches 95 % thrust scores its cell VACUOUS (frozen);
//! * H2: a cell whose governed 95 % time falls at or before its own first crossing scores VACUOUS (frozen: "τ_rel cannot
//!   move it") — the τ_rel-monotone cell is VACUOUS if any of its four runs is, a vs-instant cell if its run is;
//! * H3 has no frozen rule of its own: it INHERITS H2's (any of its four runs — governed or open, τ_rel 0.2 / 1.6 —
//!   reaching 95 % at or before its own crossing, or never, ⇒ VACUOUS);
//! * H4: VACUOUS where the floor is not cutting at the LP minimum (`req == 0` there, the floor-only case), or where the
//!   minimum is ≥ 0.75 (frozen); no crossing at all ⇒ the leg never released ⇒ the "before the crossing" half holds
//!   trivially, reported;
//! * H5: only the general rule (the leg never engages ⇒ VACUOUS); an HP minimum before the crossing (τ_rel-free) is a
//!   MISS with its reason;
//! * H6: the drawdown from index round(ramp/ds) on (the move's end by INDEX), running max started there.
//! A pair where one run stops early is read over the stretch both reach and the cell says so.
use std::time::Instant;
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::{AsymmetricLag, FuelLimiters, FuelPoint, PointExtra};
use turbojet::governor::{self as gv, GovMark, Leg};
use turbojet::governor_lag::Demand;
use turbojet::governor_release as gr;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, SpeedLaw, ThrustBy, CONTROL_SHAPES, DS};

const RELS: [f64; 4] = [0.2, 0.4, 0.8, 1.6];
const TA: f64 = 0.2;

fn base() -> ControlsSettings { ControlsSettings { govern: true, hold: Hold::Thrust, accel_on: true, ..ControlsSettings::defaults() } }
fn floor_case() -> ControlsSettings { ControlsSettings { accel_on: false, floor_on: true, ..base() } }
fn lag(p: &FuelPoint) -> (f64, f64) { match p.extra { PointExtra::Asym { g, required } => (g, required), _ => panic!() } }
fn solver(s: &ControlsSettings) -> ControlsSolver { build(&ControlsSettings { release_on: false, ..*s }).unwrap() }
fn nu0(sv: &ControlsSolver) -> (f64, f64) { (sv.start.nu_lp, sv.start.nu_hp) }
fn run(sv: &ControlsSolver, ta: f64, tr: f64, open: bool) -> (Vec<FuelPoint>, Vec<GovMark>) {
    let lim = FuelLimiters { lag: Some(AsymmetricLag::new(ta, tr)), ..sv.limiters() };
    let ramp = sv.settings.schedule(sv.mf_lo, sv.mf_hi);
    let dem = if open { Demand::Ramp(&ramp) } else { Demand::Governor(sv.governor.as_ref().unwrap()) };
    gr::march(&sv.core.fuel, &sv.flight, dem, &lim, None, nu0(sv), sv.settings.s_end(), DS)
}
fn gov(sv: &ControlsSolver, ta: f64, tr: f64) -> Vec<FuelPoint> { run(sv, ta, tr, false).0 }
fn open(sv: &ControlsSolver, ta: f64, tr: f64) -> Vec<FuelPoint> { run(sv, ta, tr, true).0 }
fn instant(sv: &ControlsSolver) -> Vec<FuelPoint> {
    gv::march(&sv.core.fuel, &sv.flight, sv.governor.as_ref().unwrap(), &sv.limiters(), None, nu0(sv), sv.settings.s_end(), DS).0
}
fn crossing(p: &[FuelPoint]) -> Option<usize> { p.iter().position(|x| { let (g, r) = lag(x); g > 0.0 && r < g }) }
fn engage(p: &[FuelPoint]) -> Option<usize> { p.iter().position(|x| lag(x).1 > 0.0) }
fn max_g(p: &[FuelPoint]) -> f64 { p.iter().map(|x| lag(x).0).fold(0.0, f64::max) }
/// Sign changes of req − g after engagement: (releases, re-attacks).
fn recross(p: &[FuelPoint]) -> (usize, usize) {
    let (mut rel, mut re, mut att) = (0, 0, None::<bool>);
    for x in p {
        let (g, r) = lag(x);
        if g == 0.0 && r == 0.0 { continue; }
        let a = r > g;
        if let Some(prev) = att { if prev && !a { rel += 1 } else if !prev && a { re += 1 } }
        att = Some(a);
    }
    (rel, re)
}
fn shares(sv: &ControlsSolver, p: &[FuelPoint]) -> Vec<f64> {
    let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
    p.iter().map(|x| (gv::thrust(&sv.reread(x).unwrap().base) - f0) / (f1 - f0)).collect()
}
fn t95(p: &[FuelPoint], fr: &[f64]) -> Option<f64> {
    (0..fr.len()).find(|&k| fr[k] >= 0.95).map(|k| {
        if k == 0 { return p[0].s; }
        let (s0, s1, a, b) = (p[k - 1].s, p[k].s, fr[k - 1], fr[k]);
        if a >= 0.95 { s0 } else { s0 + (s1 - s0) * (0.95 - a) / (b - a) }
    })
}
fn drawdown(fr: &[f64], k0: usize) -> f64 {
    let (mut hi, mut dd) = (f64::NEG_INFINITY, 0.0f64);
    for &v in fr.iter().skip(k0) { hi = hi.max(v); dd = dd.max(hi - v); }
    dd
}
fn trapz(p: &[FuelPoint], f: impl Fn(&FuelPoint) -> f64) -> f64 {
    p.windows(2).map(|w| 0.5 * (w[1].s - w[0].s) * (f(&w[0]) + f(&w[1]))).sum()
}
fn v(hit: bool, vac: bool) -> &'static str { if vac { "VACUOUS" } else if hit { "HIT" } else { "MISS" } }
fn add(t: &mut [usize; 3], hit: bool, vac: bool) { t[if vac { 2 } else if hit { 0 } else { 1 }] += 1; }
fn tally(name: &str, t: [usize; 3]) { println!("== {name}: {} HIT, {} MISS, {} VACUOUS", t[0], t[1], t[2]); }
fn n(x: Option<f64>) -> f64 { x.unwrap_or(f64::NAN) }

fn main() {
    let t0 = Instant::now();
    let k_move = (base().ramp / DS).round_ties_even() as usize;
    // --- the advisor's check: the re-read reproduces the recorded point on THIS route (mf = applied, mf_sched = demand)
    {
        let sv = solver(&base());
        let g = gov(&sv, TA, 0.4);
        let bad = g.iter().filter(|p| sv.reread(p).unwrap().base.tt4.to_bits() != p.tt4.to_bits()).count();
        let differ = g.iter().filter(|p| p.mf != p.mf_sched).count();
        println!("reread check: {bad} of {} points differ in Tt4 (applied != demand at {differ} points)", g.len());
        assert_eq!(bad, 0);
    }
    let (mut h1a, mut h1b, mut h2m, mut h2i, mut h3, mut h6, mut h6o, mut h6i) =
        ([0usize; 3], [0usize; 3], [0usize; 3], [0usize; 3], [0usize; 3], [0usize; 3], [0usize; 3], [0usize; 3]);
    println!("-- the schedule cells (thrust law, margin 0.25, attack {TA})");
    for shape in 0..4 {
        let name = CONTROL_SHAPES[shape];
        let sv = solver(&ControlsSettings { shape, ..base() });
        let inst = instant(&sv);
        let fr_i = shares(&sv, &inst);
        let gs: Vec<Vec<FuelPoint>> = RELS.iter().map(|&tr| gov(&sv, TA, tr)).collect();
        let os: Vec<Vec<FuelPoint>> = RELS.iter().map(|&tr| open(&sv, TA, tr)).collect();
        let lens: Vec<usize> = gs.iter().chain(&os).map(|p| p.len()).chain([inst.len()]).collect();
        let nmin = *lens.iter().min().unwrap();
        let cut = lens.iter().any(|&l| l != nmin);
        println!("{name}: points {lens:?}{}", if cut { " (CUT to the stretch all reach)" } else { "" });
        let g: Vec<&[FuelPoint]> = gs.iter().map(|p| &p[..nmin]).collect();
        let o: Vec<&[FuelPoint]> = os.iter().map(|p| &p[..nmin]).collect();
        // H1 — max g τ_rel-free? deeper governed? asked earlier?
        let mg: Vec<f64> = g.iter().map(|p| max_g(p)).collect();
        let mo: Vec<f64> = o.iter().map(|p| max_g(p)).collect();
        let free = mg.iter().all(|x| x.to_bits() == mg[0].to_bits()) && mo.iter().all(|x| x.to_bits() == mo[0].to_bits());
        let (ig, io) = (engage(g[1]), engage(o[1]));
        let vac1 = ig.is_none() || io.is_none();
        add(&mut h1a, mg[1] > mo[1], vac1);
        let earlier = matches!((ig, io), (Some(a), Some(b)) if a < b);
        add(&mut h1b, earlier, vac1);
        println!("  H1 max g gov {:.6e} open {:.6e} (ratio {:.3}) {} | tau_rel-free bit for bit: {free} (gov {:?}) | first req>0 gov {:?} (s={:.2}) open {:?} (s={:.2}) {}",
                 mg[1], mo[1], mg[1] / mo[1], v(mg[1] > mo[1], vac1), mg.iter().map(|x| format!("{x:.9e}")).collect::<Vec<_>>(),
                 ig, ig.map(|k| g[1][k].s).unwrap_or(f64::NAN), io, io.map(|k| o[1][k].s).unwrap_or(f64::NAN), v(earlier, vac1));
        // H2 / H3 / H6
        let frg: Vec<Vec<f64>> = g.iter().map(|p| shares(&sv, p)).collect();
        let fro: Vec<Vec<f64>> = o.iter().map(|p| shares(&sv, p)).collect();
        let xi = t95(&inst[..nmin], &fr_i[..nmin]);
        let xg: Vec<Option<f64>> = g.iter().zip(&frg).map(|(p, f)| t95(p, f)).collect();
        let xo: Vec<Option<f64>> = o.iter().zip(&fro).map(|(p, f)| t95(p, f)).collect();
        let kg: Vec<Option<usize>> = g.iter().map(|p| crossing(p)).collect();
        let ko: Vec<Option<usize>> = o.iter().map(|p| crossing(p)).collect();
        // VACUOUS: never 95 %, or 95 % at or before its own crossing (or no crossing at all: the release never begins)
        let vacx = |x: Option<f64>, k: Option<usize>, p: &[FuelPoint]| match (x, k) {
            (None, _) | (_, None) => true,
            (Some(t), Some(k)) => t <= p[k].s,
        };
        let vg: Vec<bool> = (0..4).map(|i| vacx(xg[i], kg[i], g[i])).collect();
        let vo: Vec<bool> = (0..4).map(|i| vacx(xo[i], ko[i], o[i])).collect();
        let mono = (0..3).all(|i| n(xg[i + 1]) > n(xg[i]));
        add(&mut h2m, mono, vg.iter().any(|&b| b));
        println!("  H2 95% gov by tau_rel {:?} (crossings at s {:?}) instant {:.4} open {:?} -> rising {}",
                 xg.iter().map(|x| format!("{:.4}", n(*x))).collect::<Vec<_>>(),
                 kg.iter().zip(&g).map(|(k, p)| k.map(|k| format!("{:.2}", p[k].s)).unwrap_or("-".into())).collect::<Vec<_>>(),
                 n(xi), xo.iter().map(|x| format!("{:.4}", n(*x))).collect::<Vec<_>>(), v(mono, vg.iter().any(|&b| b)));
        for i in 0..4 {
            let vac = vg[i] || xi.is_none();
            let later = n(xg[i]) > n(xi);
            add(&mut h2i, later, vac);
            println!("  H2 tau_rel {}: gov {:.4} vs instant {:.4} {}", RELS[i], n(xg[i]), n(xi), v(later, vac));
        }
        let dg = n(xg[3]) - n(xg[0]);
        let dop = n(xo[3]) - n(xo[0]);
        let vac3 = vg[0] || vg[3] || vo[0] || vo[3];
        add(&mut h3, dg < dop, vac3);
        println!("  H3 release cost (1.6 - 0.2): gov {dg:.4} open {dop:.4} {}", v(dg < dop, vac3));
        let ddi = drawdown(&fr_i[..nmin], k_move);
        for i in 0..4 {
            let (ddg, ddo) = (drawdown(&frg[i], k_move), drawdown(&fro[i], k_move));
            add(&mut h6, ddg > 0.002, false);
            add(&mut h6o, ddo <= 0.002, false);
            println!("  H6 tau_rel {}: drawdown gov {ddg:.5} {} | open {ddo:.5} {} | instant {ddi:.5} {}",
                     RELS[i], v(ddg > 0.002, false), if ddo <= 0.002 { "no sag" } else { "SAGS" },
                     if ddi <= 0.002 { "no sag" } else { "SAGS" });
        }
        add(&mut h6i, ddi <= 0.002, false);
        // Reported: crossing times; D1 let-through / withheld; re-crossings
        for i in 0..4 {
            let (p, tr) = (g[i], RELS[i]);
            let (rc, ra) = recross(p);
            if let Some(k) = kg[i] {
                let gc = lag(&p[k]).0;
                let thru = trapz(&p[..k], |x| lag(x).1 - lag(x).0);
                let held = trapz(&p[k..], |x| lag(x).0 - lag(x).1);
                let gend = lag(p.last().unwrap()).0;
                println!("  reported tau_rel {tr}: crossing gov s={:.2} open s={} | let through {thru:.4e} (tau_att g_c {:.4e}) withheld {held:.4e} (D1 -tau_rel dg = {:.4e}); g_c {gc:.4e} g_end {gend:.3e} | releases {rc} re-attacks {ra}",
                         p[k].s, ko[i].map(|k| format!("{:.2}", o[i][k].s)).unwrap_or("-".into()), TA * gc, -tr * (gend - gc));
            }
        }
    }
    tally("H1 max clip deeper governed", h1a);
    tally("H1 cut asked earlier governed", h1b);
    tally("H2 95% time rises with tau_rel", h2m);
    tally("H2 later than instant", h2i);
    tally("H3 release cost smaller governed", h3);
    tally("H6 governed sags (> 0.002)", h6);
    tally("H6 comparison: open does not sag", h6o);
    tally("H6 comparison: instant does not sag", h6i);

    println!("-- the floor cells (thrust law, LP floor, flow/press, press/flow, tilted)");
    let (mut h4, mut h5) = ([0usize; 3], [0usize; 3]);
    for shape in 0..3 {
        let name = CONTROL_SHAPES[shape];
        let sv = solver(&ControlsSettings { shape, ..floor_case() });
        for ta in [0.2, 0.4] {
            let runs: Vec<(Vec<FuelPoint>, Vec<GovMark>)> = RELS.iter().map(|&tr| run(&sv, ta, tr, false)).collect();
            let nmin = runs.iter().map(|r| r.0.len()).min().unwrap();
            let cut = runs.iter().any(|r| r.0.len() != nmin);
            let mut lp = Vec::new();
            let mut hp = Vec::new();
            let mut notes = Vec::new();
            let mut vac4 = false;
            let mut before = true;
            let mut never = false;
            for (i, (p, m)) in runs.iter().enumerate() {
                let p = &p[..nmin];
                let (il, ml) = p.iter().enumerate().map(|(k, x)| (k, x.phi_lp)).fold((0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a });
                let (ih, mh) = p.iter().enumerate().map(|(k, x)| (k, x.phi_hp)).fold((0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a });
                let k = crossing(p);
                never |= engage(p).is_none();
                let cutting = lag(&p[il]).1 > 0.0;
                vac4 |= !cutting || ml >= 0.75;
                before &= k.map_or(true, |k| il < k);
                let lean_rel = k.map_or(0, |k| m[k..nmin].iter().filter(|x| x.leg == Leg::MinFuel).count());
                let (rc, ra) = recross(p);
                lp.push(ml);
                hp.push(mh);
                notes.push(format!("tau_rel {}: LP min {ml:.9} at {il} (s={:.2}, req {:.3e}) HP min {mh:.9} at {ih} (s={:.2}) crossing {:?} | lean holds in release {lean_rel} | releases {rc} re-attacks {ra}",
                                   RELS[i], p[il].s, lag(&p[il]).1, p[ih].s, k));
            }
            let same = lp.iter().all(|x| x.to_bits() == lp[0].to_bits());
            add(&mut h4, same && before, vac4 || never);
            let rising = hp.windows(2).all(|w| w[1] > w[0]);
            add(&mut h5, rising, never);
            println!("{name} tau_att {ta}{}: H4 LP min identical {same}, before the crossing {before} -> {} | H5 HP min {:?} -> {}",
                     if cut { " (CUT)" } else { "" }, v(same && before, vac4 || never),
                     hp.iter().map(|x| format!("{x:.6}")).collect::<Vec<_>>(), v(rising, never));
            for l in notes { println!("    {l}"); }
        }
    }
    tally("H4 the LP minimum is tau_rel-free and before the crossing", h4);
    tally("H5 the HP minimum rises with tau_rel", h5);
    println!("   (H4's precondition 'the solvable-ceiling fallback never sets the release fuel' is NOT recorded, so NOT checked)");

    // Reported: floor + schedule + redline
    println!("-- reported: floor + schedule + redline (attack {TA})");
    for shape in 0..3 {
        let sv = solver(&ControlsSettings { shape, accel_on: true, floor_on: true, redline_on: true, ..base() });
        let inst = instant(&sv);
        let fi = shares(&sv, &inst);
        let row: Vec<String> = RELS.iter().map(|&tr| {
            let (p, m) = run(&sv, TA, tr, false);
            let f = shares(&sv, &p);
            let (rc, ra) = recross(&p);
            format!("{tr}: pts {} maxg {:.3e} 95% {:.4} dd {:.5} LPmin {:.6} HPmin {:.6} rel {rc} reatt {ra} redline {} lean {}",
                    p.len(), max_g(&p), n(t95(&p, &f)), drawdown(&f, k_move),
                    p.iter().map(|x| x.phi_lp).fold(f64::INFINITY, f64::min), p.iter().map(|x| x.phi_hp).fold(f64::INFINITY, f64::min),
                    m.iter().filter(|x| x.leg == Leg::Redline).count(), m.iter().filter(|x| x.leg == Leg::MinFuel).count())
        }).collect();
        println!("{}: instant 95% {:.4} dd {:.5}", CONTROL_SHAPES[shape], n(t95(&inst, &fi)), drawdown(&fi, k_move));
        for r in row { println!("    {r}"); }
    }
    // Reported: other laws, H2 (vs instant) and H6 at tau_rel 0.4
    println!("-- reported: other laws at tau_rel 0.4 (schedule)");
    let laws: [(&str, ControlsSettings); 5] = [
        ("HP rate .1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1, ..base() }),
        ("HP prop 10", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0, ..base() }),
        ("EPR", ControlsSettings { thrust_by: ThrustBy::Epr, ..base() }),
        ("N1 rate .15", ControlsSettings { thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.15, ..base() }),
        ("N1 prop 7", ControlsSettings { thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 7.0, ..base() }),
    ];
    for (lname, ls) in laws {
        for shape in 0..4 {
            let sv = solver(&ControlsSettings { shape, ..ls });
            let p = gov(&sv, TA, 0.4);
            let inst = instant(&sv);
            let nmin = p.len().min(inst.len());
            let (f, fi) = (shares(&sv, &p[..nmin]), shares(&sv, &inst[..nmin]));
            let (x, xi) = (t95(&p[..nmin], &f), t95(&inst[..nmin], &fi));
            let k = crossing(&p);
            let (rc, ra) = recross(&p);
            println!("{lname:11} {:10}: pts {}/{} 95% gov {:.4} inst {:.4} (crossing s={}) later {} | dd gov {:.5} inst {:.5} sag {} | rel {rc} reatt {ra}",
                     CONTROL_SHAPES[shape], p.len(), inst.len(), n(x), n(xi), k.map(|k| format!("{:.2}", p[k].s)).unwrap_or("-".into()),
                     n(x) > n(xi), drawdown(&f, k_move), drawdown(&fi, k_move), drawdown(&f, k_move) > 0.002);
        }
    }
    // Reported (off the frozen protocol): H5 with the governor OFF at the same attacks — is the HP minimum's move before
    // the crossing the governor's, or the slower attack's? And rung 52's page default attack 0.02.
    println!("-- reported: the floor cells OPEN (governor off), HP minimum and crossing by tau_rel");
    for shape in 0..3 {
        let sv = solver(&ControlsSettings { shape, ..floor_case() });
        for ta in [0.02, 0.2, 0.4] {
            let row: Vec<String> = RELS.iter().map(|&tr| {
                let p = open(&sv, ta, tr);
                let (ih, mh) = p.iter().enumerate().map(|(k, x)| (k, x.phi_hp)).fold((0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a });
                format!("{tr}: HP min {mh:.6} at {ih} crossing {:?}", crossing(&p))
            }).collect();
            println!("{} open tau_att {ta}: {}", CONTROL_SHAPES[shape], row.join(" | "));
        }
    }
    // Reported (off the frozen protocol): H2/H3 with a longer settle (4 instead of 2), so tau_rel 1.6 can reach 95 %.
    println!("-- reported: settle 4 (off protocol) — 95% times by tau_rel, governed and open");
    for shape in 0..4 {
        let sv = solver(&ControlsSettings { shape, settle: 4.0, ..base() });
        let xs = |open_: bool| -> Vec<String> {
            RELS.iter().map(|&tr| { let p = run(&sv, TA, tr, open_).0; let f = shares(&sv, &p); format!("{:.4}", n(t95(&p, &f))) }).collect()
        };
        println!("{}: gov {:?} open {:?}", CONTROL_SHAPES[shape], xs(false), xs(true));
    }
    // Cost
    let sv = solver(&base());
    let t = Instant::now(); let _ = gov(&sv, TA, 0.4); let tg = t.elapsed().as_secs_f64();
    let t = Instant::now(); let _ = instant(&sv); let ti = t.elapsed().as_secs_f64();
    println!("-- cost: governed release {tg:.3} s, rung 95 instant {ti:.3} s; scoring took {:.0} s", t0.elapsed().as_secs_f64());
}
