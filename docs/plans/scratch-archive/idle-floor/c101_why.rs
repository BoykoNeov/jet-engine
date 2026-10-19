// For each crash-map request index given on stdin: Mach, altitude-free summary, the new reply's end and thrust.
use turbojet::sandbox::call;
fn main() {
    let reqs = std::fs::read_to_string("W:/temp/claude/idle-rule/c101_reqs.txt").unwrap();
    let want: Vec<usize> = std::fs::read_to_string("W:/temp/claude/idle-rule/c101_run_to_refused.txt").unwrap()
        .lines().filter_map(|l| l.trim().parse().ok()).collect();
    std::panic::set_hook(Box::new(|_| {}));
    for l in reqs.lines() {
        let mut it = l.splitn(3, '\t');
        let (_, i, req) = (it.next(), it.next().unwrap().parse::<usize>().unwrap(), it.next().unwrap());
        if !want.contains(&i) { continue; }
        let m0: f64 = req.split("\"M0\":").nth(1).unwrap().split(',').next().unwrap().parse().unwrap();
        let r = std::panic::catch_unwind(|| call(req)).unwrap_or_else(|_| "PANIC".into());
        let end = if r.contains("At the starting throttle") { "start" } else if r.contains("At the final throttle") { "final" } else { "other" };
        let sp = r.split("gives more drag than thrust (").nth(1).and_then(|s| s.split(' ').next()).unwrap_or("-");
        let lever = req.split("\"lever\":\"").nth(1).unwrap().split('"').next().unwrap();
        let cust = req.split("\"customer\":\"").nth(1).unwrap().split('"').next().unwrap();
        let below = r.contains("below idle");
        println!("{i}\tM {m0:.2}\t{end}\tbelow_idle {below}\tsp {sp}\tlever {lever}\tcust {cust}");
    }
}
