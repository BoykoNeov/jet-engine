// Probe: every crash-map request (rung 101's, seed 99) whose run now stops "lean" — its trial mixture, points, start.
use turbojet::sandbox::call;
fn main() {
    let reqs = std::fs::read_to_string("W:/temp/claude/idle-warn/c101_reqs.txt").unwrap();
    for l in reqs.lines() {
        let mut it = l.splitn(3, '\t');
        let (_, i, req) = (it.next(), it.next().unwrap(), it.next().unwrap());
        let r = call(req);
        if !r.contains(r#""kind":"lean""#) { continue; }
        let num = |k: &str| r.find(&format!("\"{k}\":")).map(|p| r[p + k.len() + 3..].split(|c| c == ',' || c == '}').next().unwrap_or("").to_string()).unwrap_or("-".into());
        let npts = r.find("\"s\":[").map(|p| r[p..].split(']').next().unwrap().matches(',').count() + 1).unwrap_or(0);
        let below = r.find("\"start\":").map(|p| r[p..].find("\"end\":").map(|q| r[p..p+q].contains("below_idle")).unwrap_or(false)).unwrap_or(false);
        let from: f64 = req.split("\"from\":").nth(1).unwrap().split(',').next().unwrap().parse().unwrap();
        let to: f64 = req.split("\"to\":").nth(1).unwrap().split(',').next().unwrap().parse().unwrap();
        println!("{i}\tmixture {}\tpoints {npts}\tfrom {from} to {to}\tstart below idle {below}", num("mixture"));
    }
}
