//! Crash map for the marched low wall: random temperature-commanded slams/chops through sandbox::call.
use std::collections::BTreeMap;
use turbojet::visuals::Json;

struct Lcg(u64);
impl Lcg {
    fn u(&mut self) -> f64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); (self.0 >> 11) as f64 / (1u64 << 53) as f64 }
    fn r(&mut self, a: f64, b: f64) -> f64 { a + (b - a) * self.u() }
}

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(600);
    let seed: u64 = std::env::args().nth(2).and_then(|a| a.parse().ok()).unwrap_or(7);
    let mut g = Lcg(seed);
    let gases = ["perfect", "thermally_perfect", "reacting", "fork_b"];
    let maps = ["flow", "pressure"];
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    let mut examples: BTreeMap<String, String> = BTreeMap::new();
    std::panic::set_hook(Box::new(|_| {}));
    for i in 0..n {
        let gas = gases[if i % 4 == 0 { 0 } else { (g.u() * 4.0) as usize % 4 }];
        let (t0, p0, m0) = (g.r(200.0, 290.0), g.r(5000.0, 101325.0), g.r(0.1, 2.5));
        let design = format!(r#"{{"T0":{},"p0":{},"M0":{},"pi_c":{},"Tt4":{},"mdot":20,"pi_d_max":{},"eta_c":{},"eta_t":{},"eta_b":{},"pi_b":{},"eta_m":{},"pi_n":{}}}"#,
            g.r(200.0, 290.0), g.r(5000.0, 101325.0), g.r(0.1, 2.0), g.r(3.0, 35.0), g.r(1200.0, 2200.0),
            g.r(0.85, 0.99), g.r(0.7, 0.95), g.r(0.7, 0.95), g.r(0.85, 1.0), g.r(0.88, 0.99), g.r(0.97, 1.0), g.r(0.9, 1.0));
        let (from, to) = (g.r(600.0, 2100.0), g.r(600.0, 2100.0));
        let ramp = ((g.r(1.0, 50.0)).floor() * 0.02 * 1e9).round() / 1e9;
        let req = format!(r#"{{"op":"slam","slam":{{"fly":{{"design":{design},"gas":"{gas}","T0":{t0},"p0":{p0},"M0":{m0},"map":"{}","phi_surge":{}}},"from":{from},"to":{to},"ramp":{ramp},"settle":{},"mode":"temperature"}}}}"#,
            maps[(g.u() * 2.0) as usize % 2], g.r(0.4, 0.85), g.r(0.5, 3.0));
        let out = std::panic::catch_unwind(|| turbojet::sandbox::call(&req));
        let key = match out {
            Err(e) => format!("PANIC {}", e.downcast_ref::<String>().cloned().or(e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default().chars().take(70).collect::<String>()),
            Ok(s) => {
                let j = Json::parse(&s);
                match j.get("ok") {
                    Some(Json::Int(1)) => match j.get("stop") {
                        Some(Json::Null) | None => "complete".to_string(),
                        Some(st) => match st.get("kind") { Some(Json::Str(k)) if k == "other" || k == "burner" || k == "flame_out" => format!("stop {k}: {}", match st.get("message") { Some(Json::Str(m)) => m.chars().take(90).collect::<String>(), _ => String::new() }), Some(Json::Str(k)) => format!("stop {k}"), _ => "stop ?".into() },
                    },
                    _ => "refused".to_string(),
                }
            }
        };
        *tally.entry(key.clone()).or_default() += 1;
        examples.entry(key).or_insert(req);
    }
    for (k, v) in &tally { println!("{v:5}  {k}"); }
    for (k, v) in &examples { if k.starts_with("PANIC") || k.contains("burner") || k.contains("other") || k.contains("flame") { println!("\n{k}\n{v}"); } }
}
