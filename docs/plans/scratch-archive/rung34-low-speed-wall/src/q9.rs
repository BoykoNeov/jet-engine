// Against the trial worktree: (1) the 18 crash-map requests that used to refuse and now run — is each answer the
// STABLE equilibrium (power balance positive just below the returned speed, negative just above)? (2) the words the
// page shows for the requests that still refuse, before vs after.
use turbojet::sandbox::{explain_fly, fly_solver, FlySettings};
use turbojet::sandbox_transient::SlamSettings;
use turbojet::visuals::Json;

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let now = std::fs::read_to_string(r"W:\temp\claude\rung33-dr\now-run.tsv").unwrap();
    for line in now.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        let q = Json::parse(f[1]);
        let (s, tt4s): (FlySettings, Vec<f64>) = match q.get("slam") {
            Some(j) => { let sl = SlamSettings::from_json(j).unwrap(); (sl.fly, vec![sl.from, sl.to]) }
            None => { let s = FlySettings::from_json(q.get("fly").unwrap()).unwrap(); let t = s.tt4; (s, vec![t]) }
        };
        let st = fly_solver(&s);
        let fl = s.flight();
        let cmap = s.map.map().with_phi_surge(s.phi_surge);
        for tt4 in tt4s {
            let eq = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| st.equilibrium(&fl, tt4, Some(&cmap))));
            match eq {
                Ok(e) => {
                    let ph = |nu: f64| st.try_instant(&fl, nu, tt4, Some(&cmap)).map(|i| i.phi).map_err(|e| e.0.chars().take(40).collect::<String>());
                    println!("{} Tt4 {tt4:.0} ab {:?}: nu {:.4} pi_c {:.3} | Phi(nu-0.01) {:?} Phi(nu+0.01) {:?}", f[0], s.ab.is_some(), e.nu, e.pi_c, ph(e.nu - 0.01), ph(e.nu + 0.01));
                }
                Err(_) => println!("{} Tt4 {tt4:.0}: equilibrium refuses (other slam end, or afterburner path)", f[0]),
            }
        }
    }
    // (2) words, before vs after, for every request refused in BOTH runs
    let rd = |p: &str| std::fs::read_to_string(p).unwrap();
    let (b, a) = (rd(r"W:\temp\claude\rung33-dr\fc-before.tsv"), rd(r"W:\temp\claude\rung33-dr\fc-after.tsv"));
    let (mut same, mut diff) = (0, 0);
    for (lb, la) in b.lines().zip(a.lines()) {
        let (rb, ra) = (lb.split('\t').nth(3).unwrap(), la.split('\t').nth(3).unwrap());
        if let (Some(mb), Some(ma)) = (rb.strip_prefix("PANIC "), ra.strip_prefix("PANIC ")) {
            let (wb, wa) = (explain_fly(mb), explain_fly(ma));
            if wb == wa { same += 1 } else { diff += 1; println!("WORDS {}: {:.60} -> {:.60}", lb.split('\t').next().unwrap(), wb, wa) }
        }
    }
    println!("still refused: {same} show the same words, {diff} different");
}
