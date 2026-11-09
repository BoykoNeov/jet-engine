//! Time each request line of a file natively. Usage: slow <reqs>
fn main() {
    let p = std::env::args().nth(1).unwrap();
    for q in std::fs::read_to_string(p).unwrap().lines().filter(|l| !l.trim().is_empty()) {
        let q = q.to_string();
        let t = std::time::Instant::now();
        let r = std::thread::Builder::new().stack_size(64 << 20).spawn(move || turbojet::sandbox::call(&q)).unwrap().join().unwrap();
        println!("{:.2} s ({})", t.elapsed().as_secs_f64(), &r[..r.len().min(60)]);
    }
}
