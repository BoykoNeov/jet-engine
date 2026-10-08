// ---------------------------------------------------------------------------- the comparison

/// The oracle's lines for one reading: its root, its subtree, and its `@rig` readback if any.
fn expected(name: &str) -> Vec<(String, String)> {
    let dot = format!("{name}.");
    let at = format!("{name}@");
    let v: Vec<(String, String)> = ORACLE.lines()
        .map(|l| {
            let (p, t) = l.split_once('\t').expect("path<TAB>token");
            (p.to_string(), t.to_string())
        })
        .filter(|(p, _)| p == name || p.starts_with(&dot) || p.starts_with(&at))
        .collect();
    assert!(!v.is_empty(), "the oracle has no reading named {name:?}");
    v
}

/// `rig` is the core the reading marched on — `None` for a derived reading, which has no
/// readback in the oracle either.
fn check(name: &str, rig: Option<&ScheduledStatorCore>, mut got: Flat) {
    if let Some(m) = rig {
        let p = format!("{name}@rig");
        got.keys(&p, 2);
        got.s(&format!("{p}.lag_coord"), m.fuel.inner.lag_coord.get());
        got.of(&format!("{p}.sm_air"), m.fuel.inner.sm_air.get());
    }
    let want = expected(name);
    let got = got.0;
    let bad: Vec<usize> = (0..want.len().min(got.len())).filter(|&k| got[k] != want[k]).collect();
    for &k in bad.iter().take(12) {
        eprintln!("{name}: line {k}: got {:?}, want {:?}", got[k], want[k]);
    }
    assert!(bad.is_empty() && got.len() == want.len(),
            "{name}: {} of {} lines differ (lengths got {}, want {})",
            bad.len(), want.len(), got.len(), want.len());
}

fn one<T>(name: &str, m: Option<&ScheduledStatorCore>, x: &T, w: fn(&mut Flat, &str, &T)) {
    let mut o = Flat::default();
    w(&mut o, name, x);
    check(name, m, o);
}

