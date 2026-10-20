//! **RUNG 104 — THE VALUES THE LP TURBINE'S FLOW LAW MOVES, RETIRED BY NAME (2026-10-20).**
//!
//! Rung 104 replaced rung 38's choked `A45` with the vane's own flow law. With the nozzle choked on a perfect gas the law
//! IS the pin (to the bit — its shortcut), so those values still equal Python's; on the real and equilibrium gases, and
//! below choke, they move. The user's decision (memory `rung104-decisions`): keep every Python-held two-shaft oracle and
//! retire ONLY the values the law moves, by name. Each arm of each oracle has its list at
//! `rust/oracle/rung104_retired/<arm>.txt` — one key a line, written once by that arm itself with
//! `RUNG104_WRITE_RETIRED=1` (the keys that differed under the law, nothing else) and committed. Every other key is still
//! held to Python exactly as before, and every listed key must still be in the golden (so a list cannot hide a key the
//! dump never had). On the old pin (`pin-default`) the lists are not read: every key is held, as before rung 104.
#![allow(dead_code)]

use std::collections::BTreeSet;
use std::path::PathBuf;

fn path(arm: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("oracle").join("rung104_retired").join(format!("{arm}.txt"))
}

/// The keys this arm retires (empty under `pin-default`, where every key is held).
pub fn retired(arm: &str) -> BTreeSet<String> {
    if cfg!(feature = "pin-default") || writing() {
        return BTreeSet::new();
    }
    std::fs::read_to_string(path(arm))
        .map(|t| t.lines().filter(|l| !l.is_empty()).map(str::to_string).collect())
        .unwrap_or_default()
}

/// The one-time writer: run an arm with `RUNG104_WRITE_RETIRED=1` and it records the keys that differ.
pub fn writing() -> bool { std::env::var("RUNG104_WRITE_RETIRED").is_ok() }

/// Write `keys` as this arm's list (writer mode only).
pub fn write(arm: &str, keys: &BTreeSet<String>) {
    let p = path(arm);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    let mut s = String::new();
    for k in keys {
        s.push_str(k);
        s.push('\n');
    }
    std::fs::write(&p, s).unwrap();
    eprintln!("rung 104: {} retired keys -> {}", keys.len(), p.display());
}

/// For a harness that collects `"{key}: …"` messages: drop the retired keys' messages (or, in writer mode, record every
/// differing key and drop them all). A key ABSENT from the golden is never a move of the law: those stay failures.
pub fn filter(arm: &str, bad: Vec<String>, golden_has: impl Fn(&str) -> bool) -> Vec<String> {
    let key = |m: &String| m.split(": ").next().unwrap_or("").to_string();
    let absent = |m: &String| m.contains("absent from the dump") || m.contains("NO GOLDEN") || m.contains("never");
    if writing() {
        let keys: BTreeSet<String> = bad.iter().filter(|m| !absent(m)).map(key).collect();
        write(arm, &keys);
        return bad.into_iter().filter(absent).collect();
    }
    let retired = retired(arm);
    check_listed(arm, &retired, golden_has);
    bad.into_iter().filter(|m| absent(m) || !retired.contains(&key(m))).collect()
}

/// Every retired key must be a key of the golden.
pub fn check_listed(arm: &str, retired: &BTreeSet<String>, golden_has: impl Fn(&str) -> bool) {
    let missing: Vec<&String> = retired.iter().filter(|k| !golden_has(k)).collect();
    assert!(missing.is_empty(), "{arm}: {} retired keys are not in the golden: {:?}", missing.len(),
            missing.iter().take(5).collect::<Vec<_>>());
}
