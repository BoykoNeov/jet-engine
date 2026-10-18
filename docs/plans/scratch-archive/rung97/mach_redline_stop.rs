//! The crash map's first-step redline stop (#379): rerun with the redline raised, and off.
use turbojet::sandbox::call;
fn main() {
    let req = std::fs::read_to_string("docs/plans/scratch-archive/rung97/mach_redline_stop.json").unwrap();
    let req = req.trim();
    let brief = |s: &str| { let st = s.contains("\"stopped\":1"); let n = s.find("\"s\":[").map(|p| s[p..].split(']').next().unwrap().matches(',').count()).unwrap_or(0);
        format!("ok1 {} stopped {} s-points~{}", s.starts_with("{\"ok\":1"), st, n) };
    println!("as is: {}", brief(&call(req)));
    for rl in [1550.0, 1650.0, 1800.0] {
        let r = req.replace("\"redline_on\":1", &format!("\"redline_on\":1,\"redline\":{rl:.1}")); assert!(r != req);
        println!("redline {rl}: {}", brief(&call(&r)));
    }
    println!("redline off: {}", brief(&call(&req.replace("\"redline_on\":1", "\"redline_on\":0"))));
}
