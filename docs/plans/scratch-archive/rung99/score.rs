// Rung 99 — anchor § 3 scored on the SHIPPED code (H1–H6), and the reported quantities.
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, CustomerMode, GeneratorMode, Holder, SpeedLaw, ThrustBy,
                                 CONTROL_SHAPES};

fn gov(shape: usize, by: ThrustBy) -> ControlsSettings {
    ControlsSettings { shape, govern: true, hold: Hold::Thrust, thrust_by: by, ..ControlsSettings::defaults() }
}
fn at(s: ControlsSettings, m0: f64) -> ControlsSettings { ControlsSettings { t0: 250.0, p0: 50_000.0, m0, ..s } }

/// Each end's miss (achieved − target)/target, or a refusal scored by direction: Err(+1 over / −1 under / 0 other, words).
fn miss(s: &ControlsSettings) -> Result<[f64; 2], (i32, String)> {
    match build(s) {
        Ok(sv) => {
            let (a, b) = (sv.hold_start.unwrap(), sv.hold_end.unwrap());
            Ok([(a.achieved - a.target) / a.target, (b.achieved - b.target) / b.target])
        }
        Err(e) => {
            let d = if e.0.contains("below the least") { -1 } else if e.0.contains("more than it reads") { 1 } else { 0 };
            Err((d, e.0))
        }
    }
}
fn pct(x: f64) -> String { format!("{:+.3}%", 100.0 * x) }
fn rn(by: ThrustBy) -> &'static str { if by == ThrustBy::Epr { "EPR" } else { "fan" } }

struct Tally { hit: usize, miss: usize, vac: usize }
impl Tally {
    fn new() -> Self { Tally { hit: 0, miss: 0, vac: 0 } }
    fn add(&mut self, ok: bool) -> &'static str { if ok { self.hit += 1; "HIT" } else { self.miss += 1; "MISS" } }
}

fn main() {
    let shapes = 0..4;
    let reads = [ThrustBy::Epr, ThrustBy::N1];
    let mut total = (0, 0, 0);
    let mut fin = |name: &str, t: &Tally| {
        println!("== {name}: {} HIT, {} MISS, {} VACUOUS", t.hit, t.miss, t.vac);
        total.0 += t.hit; total.1 += t.miss; total.2 += t.vac;
    };

    // H1 + H2 (and H4's grid side), with rung 98's miss beside.
    println!("# H1 sign / H2 size — grid; rung 98 (none) beside");
    let (mut h1, mut h2) = (Tally::new(), Tally::new());
    let mut grid_miss = std::collections::HashMap::new();
    for shape in shapes.clone() {
        for by in reads {
            for m in [0.5, 1.4] {
                let s = at(gov(shape, by), m);
                let g = miss(&s);
                let n = miss(&ControlsSettings { speed_corr: SpeedCorr::None, ..s });
                let ns = match &n { Ok(x) => format!("{} / {}", pct(x[0]), pct(x[1])), Err((_, w)) => format!("refused: {w}") };
                match g {
                    Ok(x) => {
                        grid_miss.insert((shape, rn(by), m.to_bits()), x);
                        for (k, end) in ["1000K", "1400K"].iter().enumerate() {
                            let size = h2.add(x[k].abs() < 0.01);
                            let sign = if m == 0.5 { Some(x[k] < 0.0) } else if k == 0 { Some(x[k] > 0.0) } else { None };
                            let sgn = match sign { Some(ok) => h1.add(ok), None => "(not scored)" };
                            println!("{:10} {} M{m} {end}: grid {}  H1 {sgn}  H2 {size}   [none {ns}]", CONTROL_SHAPES[shape], rn(by), pct(x[k]));
                        }
                    }
                    Err((d, w)) => {
                        println!("{:10} {} M{m}: grid REFUSED ({d}): {w}", CONTROL_SHAPES[shape], rn(by));
                        h2.add(false); h2.add(false);
                        h1.add(if m == 0.5 { d == -1 } else { d == 1 });
                        if m == 0.5 { h1.add(d == -1); }
                    }
                }
            }
        }
    }
    fin("H1", &h1);
    fin("H2", &h2);

    // H3: Mach 0.15, extended below the grid.
    println!("# H3 — Mach 0.15 (extended): over");
    let mut h3 = Tally::new();
    for shape in shapes.clone() {
        for by in reads {
            let s = at(gov(shape, by), 0.15);
            match miss(&s) {
                Ok(x) => for (k, end) in ["1000K", "1400K"].iter().enumerate() {
                    println!("{:10} {} {end}: {}  {}", CONTROL_SHAPES[shape], rn(by), pct(x[k]), h3.add(x[k] > 0.0));
                },
                Err((d, w)) => { println!("{:10} {}: REFUSED ({d}) {w}", CONTROL_SHAPES[shape], rn(by)); h3.add(d == 1); h3.add(d == 1); }
            }
        }
    }
    fin("H3", &h3);

    // H4: formula against grid.
    println!("# H4 — formula vs grid (|formula| < |grid| unless marked GRID-WINS)");
    let mut h4 = Tally::new();
    for shape in shapes.clone() {
        for by in reads {
            for m in [0.5, 1.4] {
                let f = miss(&ControlsSettings { speed_corr: SpeedCorr::Formula, ..at(gov(shape, by), m) });
                let g = grid_miss.get(&(shape, rn(by), m.to_bits()));
                let (Ok(f), Some(g)) = (&f, g) else { println!("{:10} {} M{m}: formula {:?}", CONTROL_SHAPES[shape], rn(by), f.as_ref().err()); continue };
                for (k, end) in ["1000K", "1400K"].iter().enumerate() {
                    let scored = m == 0.5 || by == ThrustBy::Epr;
                    let grid_wins = m == 1.4 && k == 1;
                    let res = if !scored { "(not scored)" } else if grid_wins { h4.add(g[k].abs() < f[k].abs()) } else { h4.add(f[k].abs() < g[k].abs()) };
                    println!("{:10} {} M{m} {end}: formula {}  grid {}  {}{res}", CONTROL_SHAPES[shape], rn(by), pct(f[k]), pct(g[k]),
                             if grid_wins { "GRID-WINS " } else { "" });
                }
            }
        }
    }
    fin("H4", &h4);

    // H5: formula, Mach 0.05, from 700 K, the starting end, EPR: under. Fan reported.
    println!("# H5 — formula at the unchoked 700 K start, Mach 0.05: under (EPR scored, fan reported)");
    let mut h5 = Tally::new();
    for shape in shapes.clone() {
        for by in reads {
            let s = ControlsSettings { speed_corr: SpeedCorr::Formula, from: 700.0, ..at(gov(shape, by), 0.05) };
            match build(&s) {
                Ok(sv) => {
                    let a = sv.hold_start.unwrap();
                    let x = (a.achieved - a.target) / a.target;
                    let unch = sv.start.branch != turbojet::matcher::Branch::Choked;
                    let res = if by == ThrustBy::Epr { h5.add(x < 0.0) } else { "(reported)" };
                    // the rest (unchoked part) through the run
                    let rest = match controls(&s) {
                        Ok((sv2, o)) => o.points.iter().filter_map(|p| sv2.read(p).ok()?.gap_nozzle.map(|g| (g, p.sp_thrust * p.mdot_air)))
                            .map(|(g, f)| (g / f).abs()).fold(0.0f64, f64::max),
                        Err(_) => f64::NAN,
                    };
                    println!("{:10} {} start {}: {} (start unchoked {unch}, end at {:.1} K) {res}; max |rest|/F over run {:.2e}",
                             CONTROL_SHAPES[shape], rn(by), a.set, pct(x), sv.start.tt4, rest);
                }
                Err(e) => {
                    let d = if e.0.contains("below the least") { -1 } else if e.0.contains("more than it reads") { 1 } else { 0 };
                    if by == ThrustBy::Epr { h5.add(d == -1); }
                    println!("{:10} {}: REFUSED ({d}) {}", CONTROL_SHAPES[shape], rn(by), e.0);
                }
            }
        }
    }
    fin("H5", &h5);

    // H6: schedule peak within 10 K; proportional law settles within 0.1 %.
    println!("# H6 — schedule peak (10 K) and proportional law (0.1 %), grid vs none");
    let mut h6 = Tally::new();
    for shape in shapes.clone() {
        for m in [0.5, 1.4] {
            let a = at(ControlsSettings { shape, accel_on: true, ..ControlsSettings::defaults() }, m);
            let run = |s: &ControlsSettings| {
                let (sv, o) = controls(s).unwrap_or_else(|e| panic!("{}", e.0));
                let held = o.points.iter().any(|p| sv.read(p).unwrap().holder == Holder::Accel);
                (o.points.iter().map(|p| p.tt4).fold(f64::MIN, f64::max), held)
            };
            let ((pg, hg), (pn, hn)) = (run(&a), run(&ControlsSettings { speed_corr: SpeedCorr::None, ..a }));
            let res = if hg || hn { h6.add((pg - pn).abs() < 10.0) } else { h6.vac += 1; "VACUOUS" };
            println!("{:10} M{m} schedule: peak grid {pg:.2} none {pn:.2} (Δ {:+.3} K) held {hg}/{hn} {res}", CONTROL_SHAPES[shape], pg - pn);
            let p = at(ControlsSettings { shape, govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, ..ControlsSettings::defaults() }, m);
            let end = |s: &ControlsSettings| controls(s).unwrap_or_else(|e| panic!("{}", e.0)).1.points.last().unwrap().nu_hp;
            let (eg, en) = (end(&p), end(&ControlsSettings { speed_corr: SpeedCorr::None, ..p }));
            println!("{:10} M{m} prop law: ν_H grid {eg:.6} none {en:.6} (Δ {:+.4}%) {}", CONTROL_SHAPES[shape], 100.0 * (eg - en) / en,
                     h6.add(((eg - en) / en).abs() < 0.001));
        }
    }
    fin("H6", &h6);
    println!("== TOTAL: {} HIT, {} MISS, {} VACUOUS", total.0, total.1, total.2);

    // Reported, not scored.
    println!("# Reported: grid misses across Mach (both readings), none beside");
    for shape in [0usize, 3] {
        for by in reads {
            for m in [0.05, 0.9, 1.05, 1.1, 3.2] {
                let s = at(gov(shape, by), m);
                let f = |r: Result<[f64; 2], (i32, String)>| match r { Ok(x) => format!("{} / {}", pct(x[0]), pct(x[1])), Err((d, w)) => format!("refused({d}) {}", &w[..w.len().min(90)]) };
                println!("{:10} {} M{m}: grid {}   formula {}   none {}", CONTROL_SHAPES[shape], rn(by), f(miss(&s)),
                         f(miss(&ControlsSettings { speed_corr: SpeedCorr::Formula, ..s })), f(miss(&ControlsSettings { speed_corr: SpeedCorr::None, ..s })));
            }
        }
    }
    println!("# Reported: schedule and prop law at Mach 0.15 from 700 K (unchoked slice rows), grid vs none");
    for shape in shapes.clone() {
        let a = ControlsSettings { from: 700.0, ..at(ControlsSettings { shape, accel_on: true, ..ControlsSettings::defaults() }, 0.15) };
        let pk = |s: &ControlsSettings| controls(s).map(|(_, o)| o.points.iter().map(|p| p.tt4).fold(f64::MIN, f64::max));
        let p = ControlsSettings { from: 700.0, ..at(ControlsSettings { shape, govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, ..ControlsSettings::defaults() }, 0.15) };
        let end = |s: &ControlsSettings| controls(s).map(|(_, o)| o.points.last().unwrap().nu_hp);
        println!("{:10} schedule peak grid {:?} none {:?}; prop ν_H grid {:?} none {:?}", CONTROL_SHAPES[shape],
                 pk(&a).ok(), pk(&ControlsSettings { speed_corr: SpeedCorr::None, ..a }).ok(),
                 end(&p).ok(), end(&ControlsSettings { speed_corr: SpeedCorr::None, ..p }).ok());
    }
    println!("# Reported: fixed-amount variants at Mach 0.5 / 1.4, EPR, grid (none beside)");
    for (name, v) in [("gen HP power", ControlsSettings { generator: GeneratorMode::Hp, ..ControlsSettings::defaults() }),
                      ("cust amount", ControlsSettings { customer: CustomerMode::Amount, ..ControlsSettings::defaults() })] {
        for m in [0.5, 1.4] {
            let s = at(ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..v }, m);
            let f = |r: Result<[f64; 2], (i32, String)>| match r { Ok(x) => format!("{} / {}", pct(x[0]), pct(x[1])), Err((d, _)) => format!("refused({d})") };
            println!("{name} M{m}: grid {}  none {}", f(miss(&s)), f(miss(&ControlsSettings { speed_corr: SpeedCorr::None, ..s })));
        }
    }
}
