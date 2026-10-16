//! Rung 93 — the Cool the turbine view's crash map (sandbox-plan § 14.3's instrument): 1 000 random requests over
//! the view's knob box, each through `sandbox::call`, counted as answered / refused (by reason) / PANICKED.
use std::collections::BTreeMap;
use turbojet::sandbox::call;

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str { xs[(self.next() * xs.len() as f64) as usize % xs.len()] }
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut r = Lcg(11);
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    let mut panics = Vec::new();
    let t0 = std::time::Instant::now();
    for i in 0..1000 {
        let framing = r.pick(&["fixed", "fresh"]);
        let tech = r.pick(&["convection", "coated", "film", "transpiration"]);
        let year = (1950.0 + 80.0 * r.next()).round();
        let life = r.pick(&["1000", "3000", "10000", "30000", "100000"]);
        let shape = r.pick(&["flow/press", "press/flow", "tilted", "flat-lp"]);
        let vane = (0.24 * r.next() / 0.005).round() * 0.005;
        let blade = (0.24 * r.next() / 0.005).round() * 0.005;
        let tt4 = 1300.0 + 50.0 * (r.next() * 19.0).floor();
        let knob = r.next() < 0.3;
        let cool = format!(r#"{{"framing":"{framing}","tech":"{tech}","year":{year},"life":{life},"shape":"{shape}","vane":{vane},"blade":{blade}}}"#);
        let req = if knob { format!(r#"{{"op":"cooling_knob","cooling":{cool}}}"#) }
                  else { format!(r#"{{"op":"cooling_point","cooling":{cool},"Tt4":{tt4}}}"#) };
        let out = std::panic::catch_unwind(|| call(&req));
        let key = match out {
            Err(e) => {
                let m = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                panics.push(format!("#{i} {req}\n   {m}"));
                "PANIC".to_string()
            }
            Ok(s) if s.starts_with(r#"{"ok":1"#) && knob => {
                let y = if s.contains(r#""yours":{"ok":1"#) { "yours ok" } else if s.contains("holds all the way") { "yours holds to the top" } else if s.contains("over its limit") { "yours over at the bottom" } else { println!("KNOB OTHER #{i}: {s}"); "yours refused, other" };
                let b = if s.contains(r#""bare":{"ok":1"#) { "bare ok" } else { "bare refused" };
                format!("knob: {y}, {b}")
            }
            Ok(s) if s.starts_with(r#"{"ok":1"#) => "point: answered".to_string(),
            Ok(s) => {
                let why = if s.contains("material year") { "year" } else if s.contains("quarter") { "stream cap" }
                          else if s.contains("below the cooling air") { "limit below coolant" }
                          else if s.contains("nozzle stops choking") { "unchoked" } else { "other" };
                if why == "other" { println!("OTHER #{i}: {s}"); }
                format!("point refused: {why}")
            }
        };
        *tally.entry(key).or_default() += 1;
    }
    for (k, n) in &tally { println!("{n:>5}  {k}"); }
    println!("{} panics; {:.1} s", panics.len(), t0.elapsed().as_secs_f64());
    for p in &panics { println!("{p}"); }
}
