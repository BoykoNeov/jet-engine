//! Time the crash map's slow governed requests (keys rung 95 already knew) on whichever turbojet this crate points at.
fn main() {
    let path = std::env::args().nth(1).unwrap();
    for req in std::fs::read_to_string(path).unwrap().lines().filter(|l| !l.is_empty()) {
        let t = std::time::Instant::now();
        let out = turbojet::sandbox::call(req);
        println!("{:.2} s ({})", t.elapsed().as_secs_f64(), &out[..out.len().min(60)]);
    }
}
