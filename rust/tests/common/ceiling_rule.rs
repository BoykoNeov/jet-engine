//! **THE SCHEDULE-CHECK SWITCH ONLY EXTENDS RUNS (2026-10-14)** — the licence every Python oracle
//! uses for the default flip (sandbox plan § 14.6).
//!
//! The oracles are held to Python with the switch OFF (`CeilingDefault::set(false)` around the
//! whole emitter — the pre-2026-10-14 march, bit for bit). This module licenses what the shipped
//! default (ON) does to the SAME emitter, by a RULE rather than by a list of values:
//!
//! * nothing the OFF run emits may vanish — a run the switch carries further only grows — EXCEPT
//!   a sampled trajectory's END point: emitters record every k-th point AND the last one, so the
//!   OFF run's last index drops out when the run goes on. Such a key is admitted only if ON emits
//!   a LATER index of the same trajectory and field;
//! * a key only ON emits must be a TRAJECTORY POINT (a path segment that is an index): the run
//!   went on past where the solver had stopped it;
//! * a key both emit with different bits must be a WHOLE-RUN reading ([`RUN_SUMMARY`]): a sum,
//!   an extremum, a count or an end value over a run whose length changed.
//!
//! The census behind the rule (§ 14.5) checked, march by march, that the points both runs reach
//! are bit-identical; `tests/ceiling_branches.rs` gates that directly. The counts are pinned from
//! the MEASURED flip, so a new move — even one the rule would admit — fails until it is looked at.

use std::collections::BTreeMap;

/// Fields that read a whole run. A longer run may move them; nothing else may move.
pub const RUN_SUMMARY: &[&str] = &[
    // counts and lengths
    "npts", "n", "n_ride", "prof_len", "auth_fuel", "nkeys",
    // where the run ends
    "s_last", "end_s", "nu_lp_end", "nu_hp_end", "thrust_end", "b_end",
    // sums and extrema over the run
    "removed", "fuel_removed", "removed_alone", "removed_below_bare", "removed_armed",
    "dremoved", "drift",
];

fn last(k: &str) -> &str {
    k.rsplit('/').next().unwrap_or(k)
}

fn is_point(k: &str) -> bool {
    k.split('/').any(|seg| !seg.is_empty() && seg.bytes().all(|b| b.is_ascii_digit()))
}

/// `(prefix, index, field)` around the LAST numeric segment of a key.
fn split_index(k: &str) -> Option<(String, u64, String)> {
    let segs: Vec<&str> = k.split('/').collect();
    let i = segs.iter().rposition(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))?;
    Some((segs[..i].join("/"), segs[i].parse().ok()?, segs[i + 1..].join("/")))
}

/// A lost key is a sampled trajectory's former end point: ON emits a later index of it.
fn superseded(k: &str, on: &BTreeMap<String, u64>) -> bool {
    let Some((pre, idx, field)) = split_index(k) else { return false };
    on.keys().any(|o| split_index(o).is_some_and(|(p, j, f)| p == pre && f == field && j > idx))
}

/// Check `on` against `off` under the rule; `want` = `(changed, new, lost)` as measured.
pub fn check(name: &str, off: &BTreeMap<String, u64>, on: &BTreeMap<String, u64>,
             want: (usize, usize, usize)) {
    assert!(!off.is_empty(), "{name}: the OFF run emitted nothing");
    let lost: Vec<&String> = off.keys().filter(|k| !on.contains_key(*k)).collect();
    let bad_lost: Vec<&&String> = lost.iter().filter(|k| !superseded(k, on)).collect();
    assert!(bad_lost.is_empty(),
            "{name}: the ON run lost {} keys the OFF run emits that no later point supersedes: {:?}",
            bad_lost.len(), &bad_lost[..bad_lost.len().min(12)]);
    let changed: Vec<&String> = off.iter()
        .filter(|(k, v)| on.get(*k).is_some_and(|w| w != *v)).map(|(k, _)| k).collect();
    let new: Vec<&String> = on.keys().filter(|k| !off.contains_key(*k)).collect();
    let bad_new: Vec<&&String> = new.iter().filter(|k| !is_point(k)).collect();
    assert!(bad_new.is_empty(),
            "{name}: ON emits keys that are not trajectory points: {bad_new:?}");
    let bad_changed: Vec<&&String> =
        changed.iter().filter(|k| !RUN_SUMMARY.contains(&last(k))).collect();
    assert!(bad_changed.is_empty(),
            "{name}: ON moves keys that are not whole-run readings -- the switch changed a point \
             both runs reach: {bad_changed:?}");
    assert_eq!((changed.len(), new.len(), lost.len()), want,
               "{name}: (changed, new, lost) under the switch, against the measured pin. \
                Changed: {:?}", &changed[..changed.len().min(24)]);
}
