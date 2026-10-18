// Rung 100 § 3 scored, exactly as frozen in docs/plans/rung100-anchor-unchoked-formula.md (anchor 4b5c769).
// Miss = the calibrated START's (achieved − target)/target, in %. "over" > 0.
use turbojet::atmosphere::Ambient;
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, ControlsSettings, CustomerMode, GeneratorBy, GeneratorMode, LeverChoice, ThrustBy, CONTROL_SHAPES};

enum M { Miss(f64), Refused(String) }
fn miss(s: &ControlsSettings) -> M {
    match build(s) {
        Ok(sv) => { let a = sv.hold_start.unwrap(); M::Miss(100.0 * (a.achieved / a.target - 1.0)) }
        Err(e) => M::Refused(e.0),
    }
}
fn own_end(m: &str) -> bool { m.starts_with("At the starting throttle") && m.contains("no steady operating point") }
fn gov(by: ThrustBy) -> ControlsSettings { ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, ..ControlsSettings::defaults() } }
struct Tally { hit: usize, miss: usize, vac: usize, name: &'static str }
impl Tally {
    fn new(name: &'static str) -> Self { Tally { hit: 0, miss: 0, vac: 0, name } }
    fn score(&mut self, key: &str, ok: Option<bool>, detail: String) {
        match ok { Some(true) => self.hit += 1, Some(false) => { self.miss += 1; println!("  MISS {} {key}: {detail}", self.name) }, None => { self.vac += 1; println!("  VACUOUS {} {key}: {detail}", self.name) } }
    }
    fn report(&self) { println!("{}: {} HIT, {} MISS, {} VACUOUS", self.name, self.hit, self.miss, self.vac); }
}
fn main() {
    let d = ControlsSettings::defaults();
    let variants3 = [("plain", d), ("cool", ControlsSettings { cool_vane: 0.05, cool_blade: 0.05, ..d }),
                     ("cust", ControlsSettings { customer: CustomerMode::Share, ..d })];
    // ---------------- H1
    let mut h1 = Tally::new("H1");
    let a3 = Ambient::from_altitude(3000.0, 0.0);
    let flights = [(318.15, 101_325.0, 0.08), (318.15, 101_325.0, 0.22), (a3.t0, a3.p0, 0.12), (a3.t0, a3.p0, 0.22)];
    let v1 = [("plain", d), ("genLP", ControlsSettings { generator: GeneratorMode::Lp, generator_by: GeneratorBy::Share, ..d }),
              ("bleed", ControlsSettings { lever: LeverChoice::Bleed, ..d })];
    let mut worst1 = 0.0f64;
    for &(t0, p0, m0) in &flights { for from in [750.0, 800.0] { for (vn, v) in v1 { for shape in 0..4 { for by in [ThrustBy::Epr, ThrustBy::N1] {
        let s = ControlsSettings { shape, from, t0, p0, m0, ..ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, ..v } };
        let key = format!("{t0:.1}K {p0:.0}Pa M{m0} {from} {vn} {} {by:?}", CONTROL_SHAPES[shape]);
        let (n, o) = (miss(&s), miss(&ControlsSettings { low_grid: false, ..s }));
        match (n, o) {
            (M::Miss(n), M::Miss(o)) => { worst1 = worst1.max(n.abs());
                h1.score(&key, Some(n.abs() < 1.0 && (o.abs() <= 0.3 || n.abs() < o.abs())), format!("new {n:+.4}% rung99 {o:+.4}%")); println!("  h1 {key}: new {n:+.4} r99 {o:+.4}") }
            (M::Refused(e), _) if own_end(&e) => h1.score(&key, None, format!("own end: {}", &e[..80.min(e.len())])),
            (M::Refused(e), _) => h1.score(&key, Some(false), format!("refused: {e}")),
            (M::Miss(n), M::Refused(e)) => { worst1 = worst1.max(n.abs()); h1.score(&key, Some(n.abs() < 1.0), format!("new {n:+.4}% rung99 refused: {}", &e[..60.min(e.len())])) }
        }
    } } } } }
    h1.report(); println!("H1 worst |new| {worst1:.4}%");
    // ---------------- H2 / H3 (EPR formula), H5 / H6 (fan formula)
    let (mut h2, mut h3, mut h4, mut h5, mut h6) = (Tally::new("H2"), Tally::new("H3"), Tally::new("H4"), Tally::new("H5"), Tally::new("H6"));
    for m0 in [0.05, 0.15, 0.5] { for (vn, v) in variants3 { for shape in 0..4 {
        let base = |by| ControlsSettings { shape, from: 700.0, t0: 250.0, p0: 50_000.0, m0, speed_corr: SpeedCorr::Formula,
                                           ..ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, ..v } };
        let key = format!("M{m0} {vn} {}", CONTROL_SHAPES[shape]);
        // EPR
        let s = base(ThrustBy::Epr);
        match (miss(&s), miss(&ControlsSettings { formula_choked: true, ..s })) {
            (M::Miss(n), M::Miss(o)) => {
                println!("  epr {key}: new {n:+.4} r99 {o:+.4}");
                if m0 != 0.5 { h2.score(&key, Some(n > 0.0 && n.abs() < o.abs()), format!("new {n:+.4}% rung99 {o:+.4}%")); }
                let bar = match vn { "plain" => 0.3, "cool" => 2.0, _ => 1.3 };
                h3.score(&key, Some(n.abs() < bar && (m0 != 0.5 || n > 0.0)), format!("new {n:+.4}% bar {bar}%"));
            }
            (M::Refused(e), _) if own_end(&e) => { if m0 != 0.5 { h2.score(&key, None, "own end".into()) } h3.score(&key, None, "own end".into()) }
            (M::Refused(e), _) | (_, M::Refused(e)) => { if m0 != 0.5 { h2.score(&key, Some(false), e.clone()) } h3.score(&key, Some(false), e) }
        }
        // fan
        let s = base(ThrustBy::N1);
        match (miss(&s), miss(&ControlsSettings { formula_choked: true, ..s })) {
            (M::Miss(n), M::Miss(o)) => {
                println!("  fan {key}: new {n:+.4} r99 {o:+.4} ratio {:.3}", n.abs() / o.abs());
                h5.score(&key, Some(n < 0.0 && n.abs() < o.abs()), format!("new {n:+.4}% rung99 {o:+.4}%"));
                if o.abs() > 0.5 { let q = n.abs() / o.abs(); h6.score(&key, Some((0.15..=0.6).contains(&q)), format!("ratio {q:.3} (new {n:+.4}% rung99 {o:+.4}%)")); }
                else { println!("  H6 below floor {key}: rung99 {o:+.4}%") }
            }
            (M::Refused(e), _) if own_end(&e) => { h5.score(&key, None, "own end".into()) }
            (M::Refused(e), _) | (_, M::Refused(e)) => h5.score(&key, Some(false), e),
        }
    } } }
    // ---------------- H4
    for shape in 0..3 {
        let s = ControlsSettings { shape, from: 700.0, t0: 250.0, p0: 50_000.0, m0: 1.4, speed_corr: SpeedCorr::Formula, ..gov(ThrustBy::Epr) };
        let key = format!("M1.4 700 {}", CONTROL_SHAPES[shape]);
        match (miss(&s), miss(&ControlsSettings { formula_choked: true, ..s })) {
            (M::Miss(n), M::Miss(o)) => h4.score(&key, Some(n.abs() < o.abs()), format!("new {n:+.4}% rung99 {o:+.4}%")),
            (M::Refused(e), _) | (_, M::Refused(e)) => h4.score(&key, Some(false), e),
        }
    }
    for t in [&h2, &h3, &h4, &h5, &h6] { t.report(); }
    let (hit, mis, vac) = [&h1, &h2, &h3, &h4, &h5, &h6].iter().fold((0, 0, 0), |a, t| (a.0 + t.hit, a.1 + t.miss, a.2 + t.vac));
    println!("TOTAL: {hit} HIT, {mis} MISS, {vac} VACUOUS");
}
