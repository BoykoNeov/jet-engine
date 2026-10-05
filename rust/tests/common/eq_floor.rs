//! **THE EQUILIBRIUM-FLOOR DIVERGENCE (2026-10-05) — the one licensed departure from
//! `python-final` in the oracle gates, written as a RULE rather than a list of keys.**
//!
//! `gas.rs`'s equilibrium Newton floored `ln n` at `-80` and measured its step BEFORE the floor,
//! so wherever a species' equilibrium lay below `e^-80` (H below ~460 K; ~580 K as `f -> 0`) the
//! solve spun to its cap and raised `"equilibrium Newton did not converge"` on a well-posed
//! equilibrium. The floor is now `-300`. Every solve that converged before is bit-identical
//! (none ever reached `-80`), so the ONLY cells allowed to move are the ones whose Python run
//! went through that raise. `docs/plans/todo-solver-tolerance-audit.md` § "Rust-era re-audit".
//!
//! What the rule licenses, and the invariant each license is held to:
//!
//! 1. **A cell Python aborted with the Newton (code 4) may abort for a different, KNOWN reason**
//!    — `inverse: root not bracketed` (3; the 400 K cells, which still fail one step later) or
//!    the burner's own refusal (`BALANCE`; at `M0 = 2` the ram-heated `Tt3` exceeds `Tt4`, so no
//!    `f >= 0` closes the balance — the floor had been hiding it). It may NOT match, and Rust may
//!    raise the Newton NOWHERE on these grids. Every key under that cell is then exempt.
//! 2. **No other cell may move** — not even a bracket march's bookkeeping (see (2) below for the
//!    license that was drafted, measured unused, and removed).
//! 3. **Abort-code censuses may move counts between codes 4, 3 and `BALANCE` only**, with code 4
//!    emptied and every group's total conserved; the moved total must equal the cells in (1).
//!    Rejection censuses must move by exactly the sum of their cells' own `n_lo`/`n_hi` moves.
//!
//! Everything else stays held to the arm's own bar — bit-equality on PyPy.

use std::collections::{HashMap, HashSet};

/// `"equilibrium Newton"` — the code every oracle table gives the floor's raise.
pub const NEWTON: f64 = 4.0;
/// `"inverse: root not bracketed"`.
pub const INVERSE: f64 = 3.0;
/// `"rung-6 equilibrium burner balance"` — the burner's `f >= 0` refusal, new with the fix.
pub const BALANCE: f64 = 15.0;

/// Filter both sides down to the keys the bit-gate still owns, asserting the rule as it goes.
pub fn reconcile<'a>(
    ours: Vec<(String, f64)>, oracle: HashMap<&'a str, f64>,
) -> (Vec<(String, f64)>, HashMap<&'a str, f64>) {
    let mine: HashMap<&str, f64> = ours.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let mut exempt: HashSet<String> = HashSet::new();

    // --- (1) the re-classified cells --------------------------------------------------------
    let newton_now: Vec<&str> = mine.iter()
        .filter(|(k, v)| k.ends_with("/abort") && **v == NEWTON).map(|(k, _)| *k).collect();
    assert!(newton_now.is_empty(),
            "the equilibrium Newton still raises after the floor fix: {newton_now:?}");
    let mut prefixes: Vec<String> = Vec::new();
    for (k, &v) in &mine {
        if !k.ends_with("/abort") || oracle.get(k) != Some(&NEWTON) {
            continue;
        }
        assert!(v == INVERSE || v == BALANCE,
                "{k}: Python aborted in the equilibrium Newton, Rust now gives code {v} — only \
                 {INVERSE} (inverse) or {BALANCE} (burner balance) is licensed; a MATCH here is \
                 a new finding, not a re-classification");
        prefixes.push(k[..k.len() - "/abort".len()].to_string());
    }
    let mut cell_prefixes: Vec<String> = Vec::new();
    for p in &prefixes {
        cell_prefixes.push(format!("{p}/"));
        if let Some(rest) = p.strip_prefix("cell/") {
            cell_prefixes.push(format!("brk/{rest}/"));
        }
    }
    let under_cell = |k: &str| cell_prefixes.iter().any(|p| k.starts_with(p.as_str()))
        || prefixes.iter().any(|p| k == format!("{p}/abort"));
    for k in mine.keys().chain(oracle.keys()) {
        if under_cell(k) {
            exempt.insert(k.to_string());
        }
    }

    // --- (2) NO license for a shortened bracket march outside (1) -----------------------------
    // The first draft licensed a march that rejected FEWER low trials while its cell stayed
    // bit-identical. Measured: NOTHING uses it. With the burner's balance refusal returned as an
    // `Abort`, the 500-650 K marches reject exactly the trials Python rejected — those trials
    // were `Tt3 > Tt4` points all along, mislabelled by the floor — so they are bit-identical
    // and an unused license would only be a hole. Any march that moves outside (1) now fails
    // the bit-gate, which is the point.

    // --- (3) the censuses ---------------------------------------------------------------------
    let mut groups: HashSet<String> = HashSet::new();
    for k in mine.keys().chain(oracle.keys()) {
        if let (true, Some(i)) = (k.starts_with("census/"), k.find("abort_code/")) {
            groups.insert(k[..i + "abort_code/".len()].to_string());
        }
    }
    let mut moved_total = 0.0f64;
    for g in &groups {
        let r = |c: f64| mine.get(format!("{g}{c:.0}").as_str()).copied().unwrap_or(0.0);
        let o = |c: f64| oracle.get(format!("{g}{c:.0}").as_str()).copied().unwrap_or(0.0);
        let (r3, r4, rb) = (r(INVERSE), r(NEWTON), r(BALANCE));
        let (o3, o4, ob) = (o(INVERSE), o(NEWTON), o(BALANCE));
        assert_eq!(r4, 0.0, "{g}: code {NEWTON} must be empty after the fix");
        assert_eq!((r3 - o3) + (rb - ob), o4,
                   "{g}: the {o4} Newton aborts must reappear as inverse/balance aborts, no more");
        moved_total += o4;
        for c in [INVERSE, NEWTON, BALANCE] {
            exempt.insert(format!("{g}{c:.0}"));
        }
    }
    if !groups.is_empty() {
        // Each group's census counts ITS cells, and every re-classified cell sits in exactly one.
        assert_eq!(moved_total as usize, prefixes.len(),
                   "the census moves ({moved_total}) do not account for the {} re-classified \
                    cells", prefixes.len());
    }
    for (census, field) in [("census/march_reject_lo", "n_lo"), ("census/march_reject_hi", "n_hi")] {
        let (Some(&r), Some(&o)) = (mine.get(census), oracle.get(census)) else { continue };
        let want: f64 = mine.iter()
            .filter(|(k, _)| k.starts_with("brk/") && k.ends_with(&format!("/{field}")))
            .map(|(k, v)| v - oracle[k]).sum();
        assert_eq!(r - o, want, "{census}: moved by {} but its cells moved by {want}", r - o);
        exempt.insert(census.to_string());
    }

    println!("eq-floor divergence (2026-10-05): {} cell(s) re-classified, {} key(s) handed to \
              the rule", prefixes.len(), exempt.len());
    let ours: Vec<(String, f64)> = ours.into_iter().filter(|(k, _)| !exempt.contains(k)).collect();
    let oracle: HashMap<&'a str, f64> =
        oracle.into_iter().filter(|(k, _)| !exempt.contains(*k)).collect();
    (ours, oracle)
}
