//! **THE TURBINE-WALL DIVERGENCE (2026-10-10) — the second licensed departure from
//! `python-final` in the oracle gates, written as a RULE rather than a list of keys.**
//!
//! Rung 31's choke solve (`matcher.rs` `try_r31_solve_turbine`) bisects `pi_t` from a wall at
//! `0.02`. On a cold cell that expansion asks the gas tables for an isentropic `Tt5s` below their
//! 150 K floor, and the PANIC (`inverse: root not bracketed`, code 3) aborted `match_point` before
//! rung 33's dispatch could look further (`docs/rung33-spec.md` § SUB-IDLE). The wall is now
//! marched in until `Tt5s` is in the tables; wherever the 0.02 wall was in the tables the loop
//! never iterates and the bisection is the same arithmetic. So the ONLY cells allowed to move are
//! the ones whose 0.02 trial is below the tables — named here by that PHYSICAL test, on the
//! cell's first pass (the design mixture), never by a list.
//!
//! What the rule licenses, and the invariant each license is held to:
//!
//! 1. **A wall cell** may now end at a LATER known guard — SUB-IDLE (1), the off-design burner
//!    (5), the nozzle back-pressure check in the choked rebuild (6), the equilibrium burner balance
//!    (15) — or return a result. Python must have aborted it: with code 3, or (equilibrium gas)
//!    with the Newton, code 4, which [`super::eq_floor`] had re-classified to 3 and now licenses
//!    onward for exactly these cells. Every `cell/` key of a code-3 wall cell is then exempt.
//! 2. **Every Python code-3 cell is a wall cell** — the physical test must account for the whole
//!    family the panic produced, or the test is naming the wrong cells.
//! 3. **Nothing else moves** — not the bracket-march replica (`brk/`), not a census: they stay on
//!    the bit-gate. Measured: neither moved (the replica never calls the choke solve).

use std::collections::{HashMap, HashSet};

/// The guards a wall cell may now reach past the turbine solve — held in `eq_floor.rs`, which
/// licenses the equilibrium-gas half of the same cells.
use super::eq_floor::WALL_LATER as LATER;
const INVERSE: f64 = 3.0;
const NEWTON: f64 = 4.0;

/// Filter both sides down to the keys the bit-gate still owns, asserting the rule as it goes.
/// `wall` holds the cell tags (`"{gas}/{M0}/{Tt4}"`) the physical test names.
pub fn reconcile<'a>(
    ours: Vec<(String, f64)>, oracle: HashMap<&'a str, f64>, wall: &HashSet<String>,
) -> (Vec<(String, f64)>, HashMap<&'a str, f64>) {
    assert!(!wall.is_empty(), "the physical test names no wall cell — the rule would be vacuous");
    let mine: HashMap<&str, f64> = ours.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let mut exempt: HashSet<String> = HashSet::new();
    for tag in wall {
        let key = format!("cell/{tag}/abort");
        let py = *oracle.get(key.as_str()).unwrap_or_else(|| panic!("{key}: not in the oracle"));
        let rs = *mine.get(key.as_str()).unwrap_or_else(|| panic!("{key}: not produced"));
        assert!(py == INVERSE || py == NEWTON,
                "{tag}: a wall cell Python did NOT abort at the table floor (code {py})");
        assert!(rs == 0.0 || LATER.contains(&rs),
                "{tag}: past the marched wall the cell gives code {rs}, not a later guard {LATER:?}");
        if py == INVERSE {
            let pre = format!("cell/{tag}/");
            for k in mine.keys().chain(oracle.keys()) {
                if k.starts_with(pre.as_str()) {
                    exempt.insert(k.to_string());
                }
            }
        }
    }
    for (k, &v) in &oracle {
        if k.starts_with("cell/") && k.ends_with("/abort") && v == INVERSE {
            let tag = &k["cell/".len()..k.len() - "/abort".len()];
            assert!(wall.contains(tag), "{tag}: Python's table-floor abort is not a wall cell");
        }
    }
    println!("turbine-wall divergence (2026-10-10): {} wall cell(s), {} key(s) handed to the rule",
             wall.len(), exempt.len());
    let ours: Vec<(String, f64)> = ours.into_iter().filter(|(k, _)| !exempt.contains(k)).collect();
    let oracle: HashMap<&'a str, f64> =
        oracle.into_iter().filter(|(k, _)| !exempt.contains(*k)).collect();
    (ours, oracle)
}
