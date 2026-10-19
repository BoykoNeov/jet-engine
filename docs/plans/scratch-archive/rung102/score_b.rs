// Rung 102 Part B scoring: H7 (32), H8 (32), H9 (16) on the thermally-perfect gas, the page's opening design,
// maps flow + pressure (anchor § 3b); tilted REPORTED. Plus the reported-not-scored items and the cost (best of 9,
// on the product's path: the page's fly reply).
use std::time::Instant as Clock;
use turbojet::afterburner::{AbMode, Afterburner, FlyAfterburner, Throat};
use turbojet::atmosphere::Ambient;
use turbojet::sandbox::{call, fly_solver, stall_reading, FlySettings, MapShape, Stall};
use turbojet::visuals::Json;

fn best(n: usize, mut f: impl FnMut()) -> f64 {
    let mut b = f64::INFINITY;
    for _ in 0..n { let t = Clock::now(); f(); b = b.min(t.elapsed().as_secs_f64()); }
    b * 1e3
}

fn main() {
    let a3 = Ambient::from_altitude(3000.0, 0.0);
    let a11 = Ambient::from_altitude(11000.0, 0.0);
    let flights = [("250K 50kPa M0.5", 250.0, 50_000.0, 0.5), ("3 km M0.3", a3.t0, a3.p0, 0.3),
                   ("11 km M1.4", a11.t0, a11.p0, 1.4), ("SL M0.1", 288.15, 101_325.0, 0.1)];
    let (mut h7, mut h8, mut h9) = ([0usize; 2], [0usize; 2], [0usize; 2]);   // [hit, total]
    let mut worst_cost: f64 = 0.0;
    for map in MapShape::ALL {
        let scored = map != MapShape::Tilted;
        println!("== map {}{}", map.key(), if scored { "" } else { "  (REPORTED, not scored)" });
        for &(fname, t0, p0, m0) in &flights {
            for tt4 in [1300.0, 1500.0] {
                let s = FlySettings { t0, p0, m0, tt4, map, ..FlySettings::defaults() };
                let fl = s.flight();
                let cmap = s.map.map().with_phi_surge(s.phi_surge);
                let st = fly_solver(&s);
                let sm = |eq: &turbojet::spool::Instant| match stall_reading(&st, eq, &cmap) {
                    Stall::Margin { sm_n, .. } => Some(sm_n), Stall::PastStallLine => Some(f64::NEG_INFINITY), _ => None };
                for tt7 in [1700.0, 1900.0] {
                    let fa = FlyAfterburner::capture(&st, Afterburner { tt7, ..Afterburner::default() });
                    let dry = fa.try_point(&fl, tt4, &cmap, AbMode::Unlit, Throat::Held(1.0)).expect("dry runs");
                    let sm_d = sm(&dry.eq).unwrap();
                    let held = fa.try_point(&fl, tt4, &cmap, AbMode::Lit, Throat::Held(1.0));
                    let mt = fa.try_lit_matched(&fl, tt4, &cmap, &dry);
                    match (&held, &mt) {
                        (Ok(h), Ok(m)) => {
                            let thr = h.eq.thrust / dry.eq.thrust;
                            let fuel = (h.eq.mdot_air * h.detail.f7) / (dry.eq.mdot_air * dry.eq.f);
                            let sm_h = sm(&h.eq);
                            let drop = sm_h.map(|x| if x == f64::NEG_INFINITY { 1.0 } else { 1.0 - x / sm_d });
                            let h7_hit = thr < 1.0;
                            let bar = if tt7 == 1900.0 { 0.60 } else { 0.40 };
                            let h8_hit = drop.map(|d| d > bar).unwrap_or(false);
                            if scored { h7[1] += 1; h8[1] += 1; if h7_hit { h7[0] += 1 } if h8_hit { h8[0] += 1 } }
                            println!("  {fname:<16} Tt4 {tt4:.0} Tt7 {tt7:.0}: held thrust x{thr:.3} fuel x{fuel:.3} sm {:.4}->{} (drop {}) H7 {} H8 {} | match s {:.4} (sqrt {:.4}) thrust x{:.3} pi_AB held {:.4} match {:.4} M6 held {:.3} {:?}",
                                     sm_d, sm_h.map(|x| format!("{x:.4}")).unwrap_or("none".into()), drop.map(|d| format!("{:.0}%", d * 100.0)).unwrap_or("?".into()),
                                     if h7_hit { "HIT" } else { "MISS" }, if h8_hit { "HIT" } else { "MISS" },
                                     m.s, (tt7 / dry.eq.tt5).sqrt(), m.eq.thrust / dry.eq.thrust, h.detail.pi_ab, m.detail.pi_ab, h.detail.m6, h.eq.branch);
                            if tt7 == 1900.0 {
                                // H9: 11 equal steps of s from 1 to s_match.
                                let mut prev: Option<(f64, f64)> = None;
                                let (mut mono, mut cross_at) = (true, None);
                                let mut row = String::new();
                                for k in 0..=10 {
                                    let sv = 1.0 + (m.s - 1.0) * k as f64 / 10.0;
                                    let p = if k == 10 { *m } else { fa.try_point(&fl, tt4, &cmap, AbMode::Lit, Throat::Held(sv)).expect("step runs") };
                                    let (f, g) = (p.eq.thrust, sm(&p.eq).unwrap_or(f64::NAN));
                                    row.push_str(&format!(" {:.3}/{:.3}", f / dry.eq.thrust, g));
                                    if let Some((pf, pg)) = prev { if !(f > pf && g > pg) { mono = false; } }
                                    if cross_at.is_none() && f > dry.eq.thrust { cross_at = Some(sv); }
                                    prev = Some((f, g));
                                }
                                let inside = cross_at.map(|c| c > 1.0 && c < m.s).unwrap_or(false);
                                let hit = mono && inside;
                                if scored { h9[1] += 1; if hit { h9[0] += 1 } }
                                println!("      H9 {} (strictly rising {mono}, crosses dry at s {:?}):{row}", if hit { "HIT" } else { "MISS" }, cross_at.map(|c| (c * 1e4).round() / 1e4));
                            }
                        }
                        _ => {
                            if scored { h7[1] += 1; h8[1] += 1; if tt7 == 1900.0 { h9[1] += 1; } }
                            println!("  {fname:<16} Tt4 {tt4:.0} Tt7 {tt7:.0}: REFUSED held {:?} match {:?} — MISS", held.as_ref().err().map(|e| &e.0), mt.as_ref().err().map(|e| &e.0));
                        }
                    }
                    // Cost on the product's path (the page's fly reply), best of 9, against the dry reply (no afterburner).
                    if scored && tt7 == 1900.0 {
                        let base = FlySettings { ..s };
                        let req = |extra: &str| { let mut j = base.to_json().dump_compact(); j.pop(); format!("{{\"op\":\"fly\",\"fly\":{j}{extra}}}}}") };
                        let (rd, rh, rm) = (req(""), req(&format!(",\"ab\":\"lit\",\"Tt7\":{tt7}")), req(&format!(",\"ab\":\"lit\",\"Tt7\":{tt7},\"a8_match\":1")));
                        assert!(Json::parse(&call(&rh)).get("afterburner").is_some(), "lit reply");
                        let (td, th, tm) = (best(9, || { call(&rd); }), best(9, || { call(&rh); }), best(9, || { call(&rm); }));
                        worst_cost = worst_cost.max(th / td).max(tm / td);
                        println!("      cost (page reply, best of 9): dry {td:.1} ms, lit held {th:.1} ms (x{:.2}), lit match {tm:.1} ms (x{:.2})", th / td, tm / td);
                    }
                }
            }
        }
    }
    println!("\nH7 {}/{}  H8 {}/{}  H9 {}/{}   worst cost ratio x{worst_cost:.2} (trigger 3)", h7[0], h7[1], h8[0], h8[1], h9[0], h9[1]);
}
