//! Byte-compare census: N random Controls requests weighted to rung 94's held ends (generator on, thrust/speed holds),
//! across governor, metal cap, inferred readings + carried tables, flights, turbine laws, HP vane, afterburner, levers.
//! Usage: census <out file> <n> <seed>. Writes "i<TAB>request<TAB>reply" per line, in order; 8 worker threads.
use std::sync::atomic::{AtomicUsize, Ordering};
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str { xs[(self.next() * xs.len() as f64) as usize % xs.len()] }
    fn range(&mut self, a: f64, b: f64, step: f64) -> f64 { ((a + (b - a) * self.next()) / step).round() * step }
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let a: Vec<String> = std::env::args().collect();
    let n: usize = a[2].parse().unwrap();
    let mut r = Lcg(a[3].parse().unwrap());
    let mut reqs = Vec::new();
    for _ in 0..n {
        let govern = (r.next() < 0.6) as i32;
        let hold = r.pick(&["thrust", "thrust", "speed", "speed", "tt4"]);
        let law = r.pick(&["rate", "prop"]);
        let gen = r.pick(&["hp", "hp", "lp", "off"]);
        let gp = r.range(0.0, 0.08, 0.005);
        let by = r.pick(&["true", "true", "epr", "n1"]);
        let cap = (r.next() < 0.25) as i32;
        let metal = (cap == 1 || r.next() < 0.2) as i32;
        let clean = (r.next() < 0.5) as i32;
        let table_at = r.pick(&["design", "flight"]);
        let corr = r.pick(&["grid", "formula", "none"]);
        let airflow = r.pick(&["table", "grid", "exhaust"]);
        let (t0, p0) = [(288.15, 101325.0), (288.15, 101325.0), (250.0, 50000.0), (220.0, 20000.0), (300.0, 101325.0)]
            [(r.next() * 5.0) as usize % 5];
        let m0 = r.range(0.05, 0.9, 0.05);
        let turbine = r.pick(&["ellipse", "ellipse", "two", "one", "pin"]);
        let vane = if r.next() < 0.2 { r#","hp_vane":"rows""# } else { "" };
        let ab = r.pick(&["none", "none", "none", "unlit", "lit"]);
        let shape = r.pick(&["flow/press", "press/flow", "tilted", "flat-lp"]);
        let lever = r.pick(&["none", "none", "stator", "bleed", "both"]);
        let cool = r.next() < 0.3;
        let (cv, cb) = if cool { (r.range(0.0, 0.1, 0.01), r.range(0.0, 0.1, 0.01)) } else { (0.0, 0.0) };
        let redline_on = (r.next() < 0.4) as i32;
        let lag_on = (redline_on == 1 && r.next() < 0.3) as i32;
        let from = r.range(700.0, 1600.0, 10.0);
        let to = r.range(700.0, 1600.0, 10.0);
        let ramp = r.range(0.1, 2.0, 0.02);
        let rho = r.range(0.5, 2.0, 0.1);
        let tau = r.range(0.15, 1.0, 0.01);
        let gain = r.range(1.0, 7.0, 1.0);
        reqs.push(format!(r#"{{"op":"controls","controls":{{"govern":{govern},"hold":"{hold}","speed_law":"{law}","gov_tau":{tau},"gov_gain":{gain},"generator":"{gen}","generator_power":{gp},"hold_cap":{cap},"metal_on":{metal},"thrust_by":"{by}","table_clean":{clean},"table_at":"{table_at}","speed_corr":"{corr}","airflow":"{airflow}","T0":{t0},"p0":{p0},"M0":{m0},"turbine":"{turbine}"{vane},"ab":"{ab}","shape":"{shape}","lever":"{lever}","cool_vane":{cv},"cool_blade":{cb},"redline_on":{redline_on},"redline":1650,"gov_lag_on":{lag_on},"gov_lag":0.3,"from":{from},"to":{to},"ramp":{ramp},"rho":{rho}}}}}"#));
    }
    let next = AtomicUsize::new(0);
    let out: Vec<std::sync::Mutex<String>> = (0..n).map(|_| std::sync::Mutex::new(String::new())).collect();
    let t0 = std::time::Instant::now();
    std::thread::scope(|s| {
        for _ in 0..8 {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                if i >= n { break; }
                let rep = std::thread::Builder::new().stack_size(64 << 20).spawn({
                    let q = reqs[i].clone();
                    move || match std::panic::catch_unwind(|| turbojet::sandbox::call(&q)) {
                        Ok(s) => s, Err(_) => "PANIC".to_string() }
                }).unwrap().join().unwrap();
                *out[i].lock().unwrap() = rep;
            });
        }
    });
    let mut f = String::new();
    for i in 0..n { f += &format!("{i}\t{}\t{}\n", reqs[i], out[i].lock().unwrap()); }
    std::fs::write(&a[1], f).unwrap();
    let ok = out.iter().filter(|o| o.lock().unwrap().starts_with(r#"{"ok":1"#)).count();
    eprintln!("{n} requests, {ok} ran, {:.0} s", t0.elapsed().as_secs_f64());
}
