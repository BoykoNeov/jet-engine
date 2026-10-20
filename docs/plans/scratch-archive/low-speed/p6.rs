// Probe 6: compare two crash-map reply files request by request (old = the lean wall at 0.004, new = the run's wall).
// Each line: `<i>\t...\t<request>\t<reply>` (seed 36's) or `REP\t<i>\t<request>\t<reply>` (rung 101's). For each request
// whose reply differs: old/new outcome (refused / stop kind / ran), points, how far the two marches agree bit for bit,
// the old stop's start mixture, and — for runs that end the same way — which keys differ.
use std::collections::BTreeMap;
use turbojet::visuals::Json;
fn load(path: &str) -> BTreeMap<usize, (String, String)> {
    let mut m = BTreeMap::new();
    for line in std::fs::read_to_string(path).unwrap().lines() {
        let f: Vec<&str> = line.split('\t').collect();
        let (i, q, r) = if f[0] == "REP" && f.len() >= 4 { (f[1], f[2], f[3]) } else if f.len() >= 4 && f[0].parse::<usize>().is_ok() { (f[0], f[2], f[3]) } else { continue };
        m.insert(i.parse().unwrap(), (q.to_string(), r.to_string()));
    }
    m
}
fn outcome(j: &Json) -> String {
    if !matches!(j.get("ok"), Some(Json::Int(1))) { return "refused".into(); }
    match j.get("stop").and_then(|s| s.get("kind")) { Some(Json::Str(k)) => k.clone(), _ => "ran".into() }
}
fn list(j: &Json, k: &str) -> Vec<f64> {
    match j.get(k) { Some(Json::List(l)) => l.iter().map(|x| match x { Json::Float(v) => *v, Json::Int(n) => *n as f64, _ => f64::NAN }).collect(), _ => vec![] }
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (old, new) = (load(&a[1]), load(&a[2]));
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    for (i, (q, ro)) in &old {
        let Some((_, rn)) = new.get(i) else { continue };
        if ro == rn { continue; }
        let (jo, jn) = (Json::parse(ro), Json::parse(rn));
        let (oo, on) = (outcome(&jo), outcome(&jn));
        let (so, sn) = (list(&jo, "s"), list(&jn, "s"));
        let (fo, fnn) = (list(&jo, "fuel"), list(&jn, "fuel"));
        let agree = so.iter().zip(&sn).zip(fo.iter().zip(&fnn)).take_while(|((a, b), (c, d))| a.to_bits() == b.to_bits() && c.to_bits() == d.to_bits()).count();
        let mix = jo.get("stop").and_then(|s| s.get("mixture")).map(|m| format!("{m:?}")).unwrap_or_default();
        let far = list(&jn, "far").iter().cloned().fold(f64::INFINITY, f64::min);
        let far_o = list(&jo, "far").iter().cloned().fold(f64::INFINITY, f64::min);
        let mut diff_keys = String::new();
        if oo == on && so.len() == sn.len() {
            if let (Json::Obj(ko), Json::Obj(_)) = (&jo, &jn) {
                for (k, v) in ko { if jn.get(k).map(|w| format!("{w:?}")) != Some(format!("{v:?}")) { diff_keys += k; diff_keys += " "; } }
            }
        }
        let class = format!("{oo} -> {on}{}", if oo == on { if so.len() == sn.len() { " (same length)" } else { " (other length)" } } else { "" });
        *tally.entry(class.clone()).or_default() += 1;
        println!("#{i}: {class}: points {} -> {}, bit-equal for the first {agree}; old stop mixture {mix}; leanest far old {far_o:.5} new {far:.5}; differing keys [{diff_keys}]\n   {}", so.len(), sn.len(), q.chars().take(400).collect::<String>());
    }
    for (k, n) in &tally { println!("{n:>5}  {k}"); }
}
