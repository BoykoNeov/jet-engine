//! Time each request in a file (one per line) with the lag on and off.
use turbojet::sandbox::call;
fn main() {
    let path = std::env::args().nth(1).unwrap();
    for req in std::fs::read_to_string(path).unwrap().lines().filter(|l| !l.is_empty()) {
        let mut row = Vec::new();
        for lag in ["1", "0"] {
            let r = req.replace(r#""gov_lag_on":1"#, &format!(r#""gov_lag_on":{lag}"#));
            let t = std::time::Instant::now();
            let out = call(&r);
            row.push(format!("lag {lag}: {:.2} s ({})", t.elapsed().as_secs_f64(), &out[..out.len().min(8)]));
        }
        println!("{}", row.join(" | "));
    }
}
