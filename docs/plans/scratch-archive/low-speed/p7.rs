// Probe 7: for crash-map requests that ran to the end both times but differ, the largest relative difference in the
// burner temperature, fuel and thrust columns, and the old run's leanest mixture against the old wall.
use std::collections::BTreeMap;
use turbojet::visuals::Json;
fn load(path: &str) -> BTreeMap<usize, String> {
    std::fs::read_to_string(path).unwrap().lines().filter_map(|l| {
        let f: Vec<&str> = l.split('\t').collect();
        (f[0] == "REP" && f.len() >= 4).then(|| (f[1].parse().unwrap(), f[3].to_string()))
    }).collect()
}
fn list(j: &Json, k: &str) -> Vec<f64> {
    match j.get(k) { Some(Json::List(l)) => l.iter().map(|x| match x { Json::Float(v) => *v, Json::Int(n) => *n as f64, _ => f64::NAN }).collect(), _ => vec![] }
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (old, new) = (load(&a[1]), load(&a[2]));
    for (i, ro) in &old {
        let Some(rn) = new.get(i) else { continue };
        if ro == rn { continue; }
        let (jo, jn) = (Json::parse(ro), Json::parse(rn));
        if jo.get("stop").map_or(true, |s| !matches!(s, Json::Null)) || jn.get("stop").map_or(true, |s| !matches!(s, Json::Null)) { continue; }
        let rel = |k: &str| list(&jo, k).iter().zip(list(&jn, k).iter()).map(|(x, y)| ((x - y) / x.abs().max(1e-30)).abs()).fold(0.0, f64::max);
        let far = list(&jo, "far").iter().cloned().fold(f64::INFINITY, f64::min);
        let gov = jo.get("governor").map(|g| format!("{g:?}")).unwrap_or_default();
        println!("#{i}: max rel diff Tt4 {:.2e} fuel {:.2e} thrust {:.2e}; old leanest mixture {far:.6}; governed {}", rel("Tt4"), rel("fuel"), rel("thrust"), !gov.is_empty() && gov != "Null");
    }
}
