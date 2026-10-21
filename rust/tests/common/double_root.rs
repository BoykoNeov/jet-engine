//! **THE DOUBLE-ROOT DIVERGENCE (2026-10-21) — the third licensed departure from `python-final`
//! in the oracle gates, written as a RULE rather than a list of keys.**
//!
//! Rung 33's subsonic residual (`matcher.rs` `match_subsonic`) has TWO roots over `pi_t` from the
//! lowest throttle that runs up to near choke, off the design flight: the working point and a
//! weak one where the turbine barely expands. Above the weak root the residual is positive again,
//! so the two-ended march read the same sign at both ends and refused SUB-IDLE (code 1) on points
//! that run with healthy thrust — Python too, so its oracles hold those refusals. The march now
//! walks up from its low end to the first sign change wherever both ends read positive; wherever
//! they straddled, that walk never runs and the arithmetic is the old one. So the ONLY cells allowed
//! to move are the ones whose march ends share a sign while the residual has a root between them —
//! named by that PHYSICAL test — on a fine grid ([`two_roots`]; the map oracle), or, where a grid on
//! the equilibrium gas costs minutes, by evaluating the residual once at the returned point
//! (`offdesign_oracle::double_root_cells`) — never by a list.
//!
//! What the rule licenses, and the invariant each license is held to:
//!
//! 1. **A double-root cell** was refused SUB-IDLE by Python and now returns a SUBSONIC point; every
//!    key of that cell is exempt (off-design: `cell/{tag}/`; the map oracle's rung-31 cross-check:
//!    `red/{tag}/`).
//! 2. **Its censuses move by exactly the named cells** — aborted down, matched-subsonic up, one more
//!    distinct subsonic `tau_t`/`pi_t`/`pi_c` per cell; the cross-check's subsonic count up, its
//!    bit-equal count NOT (rung 32 does not dispatch, so a new subsonic cell cannot agree with it).
//! 3. **Nothing else moves** — not the bracket-march replica (`brk/`), which replicates only the
//!    two marches and so still reads the shared sign: measured, it did not.

use std::collections::{HashMap, HashSet};

use turbojet::components::ram_recovery;
use turbojet::engine::FlightCondition;
use turbojet::matcher::{Branch, OffDesignMatcher};

const SUB_IDLE: f64 = 1.0;

/// THE PHYSICAL TEST: the march's two ends (its own replica — in from 0.15 and from 0.9995 by
/// 0.02) read the SAME sign, a 400-interval grid between them shows a sign change, and the
/// matcher does not settle the cell on the CHOKED branch.
pub fn two_roots(m: &OffDesignMatcher, flight: &FlightCondition, tt4: f64) -> bool {
    root_between_same_sign_ends(m, flight, tt4)
        && !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.match_point(flight, tt4)))
            .is_ok_and(|od| od.branch == Branch::Choked)
}

/// The first two parts of [`two_roots`] — for a caller that already knows which branch the matcher
/// settled each cell on (a CHOKED cell's subsonic residual has the same shape, measured 900–1500 K at
/// Mach 0.3, but rung 31's choke solve settles it first: that path runs before the dispatch and the
/// repair never reaches it).
pub fn root_between_same_sign_ends(m: &OffDesignMatcher, flight: &FlightCondition, tt4: f64) -> bool {
    let (state0, _) = m.freestream_for(flight);
    let (tt2, pt2) = (state0.tt, m.pi_d_max * ram_recovery(flight.m0) * state0.pt);
    let r = |y: f64| m.try_subsonic_operating(tt4, tt2, pt2, y, flight.p0).map(|op| op.resid).ok();
    let (mut lo, mut rlo) = (f64::NAN, f64::NAN);
    let mut pt = 0.15;
    while pt < 0.95 {
        if let Some(v) = r(pt) {
            (lo, rlo) = (pt, v);
            break;
        }
        pt += 0.02;
    }
    let (mut hi, mut rhi) = (f64::NAN, f64::NAN);
    let mut pt = 0.9995;
    while !lo.is_nan() && pt > lo {
        if let Some(v) = r(pt) {
            (hi, rhi) = (pt, v);
            break;
        }
        pt -= 0.02;
    }
    if lo.is_nan() || hi.is_nan() || rlo * rhi < 0.0 {
        return false;
    }
    let mut prev = rlo;
    (1..=400).any(|i| {
        let Some(v) = r(lo + (hi - lo) * i as f64 / 400.0) else { return false };
        let c = prev * v < 0.0;
        prev = v;
        c
    })
}

/// Exempt every key under `prefixes`, after asserting the abort/branch flips on each cell.
fn exempt_prefixed<'a>(
    mine: &HashMap<&str, f64>, oracle: &HashMap<&'a str, f64>, prefixes: &[String],
) -> HashSet<String> {
    let mut out = HashSet::new();
    for p in prefixes {
        for k in mine.keys().copied().chain(oracle.keys().copied()) {
            if k.starts_with(p.as_str()) {
                out.insert(k.to_string());
            }
        }
    }
    out
}

/// Assert `census` moved by exactly `by`, and exempt it.
fn census_moves(mine: &HashMap<&str, f64>, oracle: &HashMap<&str, f64>, census: &str, by: f64,
                exempt: &mut HashSet<String>) {
    let r = *mine.get(census).unwrap_or_else(|| panic!("{census}: not produced"));
    let o = *oracle.get(census).unwrap_or_else(|| panic!("{census}: not in the oracle"));
    assert_eq!(r - o, by, "{census}: moved by {} (rust {r}, oracle {o}), the named cells account for {by}", r - o);
    exempt.insert(census.to_string());
}

fn split<'a>(
    ours: Vec<(String, f64)>, oracle: HashMap<&'a str, f64>, exempt: &HashSet<String>,
) -> (Vec<(String, f64)>, HashMap<&'a str, f64>) {
    let ours = ours.into_iter().filter(|(k, _)| !exempt.contains(k)).collect();
    let oracle = oracle.into_iter().filter(|(k, _)| !exempt.contains(*k)).collect();
    (ours, oracle)
}

/// `offdesign_oracle`: `cells` holds the tags `"{gas}/{M0}/{Tt4}"` the physical test names.
pub fn reconcile_offdesign<'a>(
    ours: Vec<(String, f64)>, oracle: HashMap<&'a str, f64>, cells: &HashSet<String>, gases: &[&str],
) -> (Vec<(String, f64)>, HashMap<&'a str, f64>) {
    assert!(!cells.is_empty(), "the physical test names no double-root cell — the rule would be vacuous");
    let mine: HashMap<&str, f64> = ours.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let mut prefixes = Vec::new();
    for tag in cells {
        let key = format!("cell/{tag}/abort");
        assert_eq!(oracle.get(key.as_str()), Some(&SUB_IDLE), "{tag}: Python did not refuse it SUB-IDLE");
        assert_eq!(mine.get(key.as_str()), Some(&0.0), "{tag}: the walk did not make it match");
        assert_eq!(mine.get(format!("cell/{tag}/branch").as_str()), Some(&1.0), "{tag}: not on the subsonic branch");
        prefixes.push(format!("cell/{tag}/"));
    }
    let mut exempt = exempt_prefixed(&mine, &oracle, &prefixes);
    let n = cells.len() as f64;
    census_moves(&mine, &oracle, "census/aborted", -n, &mut exempt);
    census_moves(&mine, &oracle, "census/matched_subsonic", n, &mut exempt);
    for g in gases {
        let ng = cells.iter().filter(|t| t.starts_with(&format!("{g}/"))).count() as f64;
        for q in ["tau_t", "pi_t", "pi_c"] {
            census_moves(&mine, &oracle, &format!("roots/{g}/subsonic/{q}_distinct"), ng, &mut exempt);
        }
    }
    println!("double-root divergence (2026-10-21): {} cell(s), {} key(s) handed to the rule", cells.len(), exempt.len());
    split(ours, oracle, &exempt)
}

/// `map_oracle`'s flat-map-vs-rung-31 cross-check: `cells` holds `"{gas}/flat/{M0}/{Tt4}"` tags.
pub fn reconcile_map<'a>(
    ours: Vec<(String, f64)>, oracle: HashMap<&'a str, f64>, cells: &HashSet<String>,
) -> (Vec<(String, f64)>, HashMap<&'a str, f64>) {
    assert!(!cells.is_empty(), "the physical test names no double-root cell — the rule would be vacuous");
    let mine: HashMap<&str, f64> = ours.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let mut prefixes = Vec::new();
    for tag in cells {
        let key = format!("red/{tag}/r31_ok");
        assert_eq!(oracle.get(key.as_str()), Some(&0.0), "{tag}: Python's rung 31 did not refuse it");
        assert_eq!(mine.get(key.as_str()), Some(&1.0), "{tag}: the walk did not make rung 31 match");
        assert_eq!(mine.get(format!("red/{tag}/r31_branch").as_str()), Some(&1.0), "{tag}: not subsonic");
        prefixes.push(format!("red/{tag}/"));
    }
    let mut exempt = exempt_prefixed(&mine, &oracle, &prefixes);
    census_moves(&mine, &oracle, "census/reduce_subsonic_cells", cells.len() as f64, &mut exempt);
    census_moves(&mine, &oracle, "census/reduce_subsonic_bitequal", 0.0, &mut exempt);
    println!("double-root divergence (2026-10-21): {} cross-check cell(s), {} key(s) handed to the rule",
             cells.len(), exempt.len());
    split(ours, oracle, &exempt)
}
