// Part C census + crash map.
//   crashctl gen DIR N SEED  — DIR/checks.txt (every check request), DIR/none.txt, DIR/unlit.txt, DIR/lit.txt (N seeded
//                               random Controls requests: the same knobs, the afterburner not fitted / fitted / lit).
//   crashctl cmp DIR          — reads DIR/*.rep (replay outputs) and reports.
use turbojet::visuals::Json;

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); (self.0 >> 11) as f64 / (1u64 << 53) as f64 }
    fn range(&mut self, a: f64, b: f64) -> f64 { a + (b - a) * self.next() }
    fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str { xs[((self.next() * xs.len() as f64) as usize).min(xs.len() - 1)] }
    fn coin(&mut self, p: f64) -> bool { self.next() < p }
}
fn r2(x: f64) -> f64 { (x * 100.0).round() / 100.0 }

fn gen(dir: &str, n: usize, seed: u64) {
    let w = |name: &str, lines: &[String]| std::fs::write(format!("{dir}/{name}"), lines.join("\n") + "\n").unwrap();
    w("checks.txt", &turbojet::sandbox::check_requests());
    let mut g = Lcg(seed);
    let (mut none, mut unlit, mut lit) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..n {
        let ramp = ((g.range(0.1, 2.0) / 0.02).round() * 0.02 * 100.0).round() / 100.0;
        let settle = ((g.range(1.0, 4.0) / 0.02).round() * 0.02 * 100.0).round() / 100.0;
        let mut kv: Vec<String> = vec![
            format!("\"shape\":\"{}\"", g.pick(&["flow/press", "press/flow", "tilted", "flat-lp"])),
            format!("\"rho\":{}", r2(g.range(0.3, 3.0))),
            format!("\"from\":{}", (g.range(650.0, 1300.0) / 10.0).round() * 10.0),
            format!("\"to\":{}", (g.range(1100.0, 1600.0) / 10.0).round() * 10.0),
            format!("\"ramp\":{ramp}"), format!("\"settle\":{settle}"),
            format!("\"M0\":{}", r2(g.range(0.3, 1.2))),
        ];
        if g.coin(0.3) { kv.push("\"redline_on\":1".into()); kv.push(format!("\"redline\":{}", (g.range(1350.0, 1700.0) / 10.0).round() * 10.0)); }
        if g.coin(0.2) { kv.push("\"accel_on\":1".into()); }
        if g.coin(0.2) { kv.push("\"floor_on\":1".into()); kv.push(format!("\"floor_phi\":{}", r2(g.range(0.6, 0.85)))); }
        if g.coin(0.1) { kv.push("\"floor_on\":1".into()); kv.push("\"release_on\":1".into()); }
        kv.push(format!("\"lever\":\"{}\"", g.pick(&["none", "none", "stator", "bleed", "both"])));
        if g.coin(0.15) { kv.push("\"govern\":1".into()); kv.push(format!("\"hold\":\"{}\"", g.pick(&["speed", "speed", "thrust"]))); }
        let base = kv.join(",");
        let s_end = ramp + settle;
        let light = ((g.range(0.0, s_end) / 0.02).floor() * 0.02 * 100.0).round() / 100.0;
        let tt7 = (g.range(1500.0, 2300.0) / 10.0).round() * 10.0;
        let throat = if g.coin(0.3) { "\"a8_match\":1".to_string() } else { format!("\"a8_ratio\":{}", r2(g.range(1.0, 1.6))) };
        let ab = |m: &str| format!("{{\"op\":\"controls\",\"controls\":{{{base},\"ab\":\"{m}\",\"Tt7\":{tt7},\"light_at\":{light},{throat}}}}}");
        none.push(format!("{{\"op\":\"controls\",\"controls\":{{{base}}}}}"));
        unlit.push(ab("unlit"));
        lit.push(ab("lit"));
    }
    w("none.txt", &none); w("unlit.txt", &unlit); w("lit.txt", &lit);
}

fn read(path: &str) -> Vec<(String, String)> {
    std::fs::read_to_string(path).unwrap().lines().map(|l| { let (a, b) = l.split_once('\t').unwrap(); (a.to_string(), b.to_string()) }).collect()
}

fn cmp(dir: &str) {
    // The check requests: byte-equal up to the base's count, except the ops this rung changed by design.
    let (ship, base) = (read(&format!("{dir}/checks_ship.rep")), read(&format!("{dir}/checks_base.rep")));
    let mut diff = Vec::new();
    for (i, (s, b)) in ship.iter().zip(&base).enumerate() {
        if s.0 != b.0 { diff.push(format!("#{i} REQUEST differs")); continue; }
        if s.1 != b.1 { diff.push(format!("#{i} {}", &s.0[..s.0.len().min(60)])); }
    }
    println!("check requests: {} shipped, {} base; {} replies differ: {:?}", ship.len(), base.len(), diff.len(), diff);
    // The crash map.
    let (none_s, none_b) = (read(&format!("{dir}/none_ship.rep")), read(&format!("{dir}/none_base.rep")));
    let same = none_s.iter().zip(&none_b).filter(|(a, b)| a.1 == b.1).count();
    let panics = |v: &[(String, String)]| v.iter().filter(|x| x.1.starts_with("PANIC")).count();
    println!("not fitted: {same} of {} replies byte-equal to the base; panics shipped {} base {}", none_s.len(), panics(&none_s), panics(&none_b));
    let (unlit, lit) = (read(&format!("{dir}/unlit_ship.rep")), read(&format!("{dir}/lit_ship.rep")));
    println!("fitted unlit: panics {}; lit: panics {}", panics(&unlit), panics(&lit));
    let (mut kinds, mut refusals, mut unnamed) = (std::collections::BTreeMap::new(), std::collections::BTreeMap::new(), Vec::new());
    let mut same_class = 0;
    for (i, ((_, u), (_, l))) in unlit.iter().zip(&lit).enumerate() {
        if u.starts_with("PANIC") || l.starts_with("PANIC") { unnamed.push(format!("#{i} PANIC {}", &l[..l.len().min(120)])); continue; }
        let (ju, jl) = (Json::parse(u), Json::parse(l));
        let ok = |j: &Json| matches!(j.get("ok"), Some(Json::Int(1)));
        let kind = |j: &Json| j.get("stop").and_then(|s| s.get("kind")).and_then(|k| if let Json::Str(t) = k { Some(t.clone()) } else { None });
        match (ok(&ju), ok(&jl)) {
            (true, true) => {
                let (ku, kl) = (kind(&ju), kind(&jl));
                if ku == kl { same_class += 1; }
                if let Some(k) = &kl { *kinds.entry(format!("lit stop {k} (unlit {:?})", ku)).or_insert(0) += 1; }
                // A stop the unlit run lacks must name the afterburner.
                if kl.is_some() && ku.is_none() && kl.as_deref() != Some("afterburner") {
                    unnamed.push(format!("#{i} lit stop {kl:?} not named (unlit ran)"));
                }
                if let Some(Json::Str(m)) = jl.get("stop").and_then(|s| s.get("message")) {
                    if m.starts_with("afterburner refused") { unnamed.push(format!("#{i} bare marker: {m}")); }
                }
            }
            (true, false) => {
                let r = jl.get("reason").map(|r| r.dump_compact()).unwrap_or_default();
                *refusals.entry(r.chars().take(70).collect::<String>()).or_insert(0) += 1;
            }
            (false, false) => { same_class += 1; }
            (false, true) => unnamed.push(format!("#{i} unlit refused, lit runs")),
        }
    }
    println!("lit vs unlit: {same_class} of {} the same outcome class", lit.len());
    for (k, n) in &kinds { println!("  {n:4}  {k}"); }
    for (k, n) in &refusals { println!("  {n:4}  refused lit, ran unlit: {k}"); }
    println!("PROBLEMS ({}): {:?}", unnamed.len(), unnamed);
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a[1].as_str() {
        "gen" => gen(&a[2], a[3].parse().unwrap(), a[4].parse().unwrap()),
        "cmp" => cmp(&a[2]),
        _ => panic!("gen | cmp"),
    }
}
