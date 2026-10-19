// Rung 102 Part A: score H2-H6 at the anchor's § 3 cells (H1 is pinned in tests/rung102.rs).
use turbojet::afterburner::{build_afterburning_turbojet, AbMode, AbResult, Afterburner, AfterburningTurbojet};
use turbojet::components::{choked_mfp, ram_recovery};
use turbojet::engine::{EngineResult, FlightCondition, Losses};
use turbojet::gas::{FlowState, Gas};

fn losses(m0: f64) -> Losses {
    Losses { pi_d: 0.97 * ram_recovery(m0), eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99,
             pi_n: 0.98, ..Losses::default() }
}
fn mk(g: &str) -> Gas {
    match g { "cpg" => Gas::default(), "tpg" => Gas::thermally_perfect(), "reacting" => Gas::reacting(),
              _ => Gas::reacting_equilibrium() }
}
type Fl = (&'static str, f64, f64, f64);
const FLIGHTS: [Fl; 4] = [("SLS M0.05", 288.15, 101_325.0, 0.05), ("3km M0.5", 268.65, 70_120.0, 0.5),
                          ("250K M1.2", 250.0, 50_000.0, 1.2), ("11km M1.6", 216.65, 22_632.0, 1.6)];
const GASES: [&str; 4] = ["tpg", "reacting", "equilibrium", "cpg"];
struct Cell { e: AfterburningTurbojet, core: EngineResult, unlit: AbResult, wet: Result<AbResult, String> }
fn run(g: &str, fl: Fl, pi_c: f64, tt4: f64, tt7: f64) -> Cell {
    let flight = FlightCondition::new(fl.1, fl.2, fl.3);
    let e = build_afterburning_turbojet(mk(g), pi_c, tt4, fl.2, losses(fl.3), Afterburner { tt7, ..Afterburner::default() });
    let core = e.run_core(&flight, 1.0);
    let unlit = e.try_continue(&core, AbMode::Unlit).unwrap();
    let wet = e.try_continue(&core, AbMode::Lit).map_err(|x| x.0);
    Cell { e, core, unlit, wet }
}
fn ceiling(g: &str, fl: Fl, pi_c: f64, tt4: f64) -> f64 {
    let msg = run(g, fl, pi_c, tt4, 2900.0).wet.err().expect("2900 K must be refused");
    msg.split("at most ").nth(1).and_then(|s| s.split(' ').next()).and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("not a too-hot refusal: {msg}"))
}
fn tally(name: &str, hits: usize, n: usize) { println!("{name}: {hits} HIT / {} MISS of {n}\n", n - hits); }
fn fmt(v: &[f64]) -> String { v.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>().join(" ") }
/// Choked-throat area per unit of the nozzle's (shared) loss: ṁ·√Tt/(pt·MFP*).
fn area(s: &FlowState, gas: &Gas) -> f64 { s.mdot * s.tt.sqrt() / (s.pt * choked_mfp(gas, s.tt, s.far)) }

fn main() {
    // ---------------- H2
    let (mut hit, mut n) = (0, 0);
    println!("== H2: wet/dry thrust and TSFC through the flights in Mach order (dry = Unlit; [dry = no afterburner])");
    for g in GASES { for pi_c in [6.0, 15.0] { for tt4 in [1400.0, 1600.0] { for tt7 in [1700.0, 1900.0, 2100.0] {
        let cells: Vec<_> = FLIGHTS.iter().map(|&fl| run(g, fl, pi_c, tt4, tt7)).collect();
        let w = |c: &Cell| c.wet.as_ref().unwrap_or_else(|m| panic!("{g} {pi_c} {tt4} {tt7}: {m}")).performance;
        let th: Vec<f64> = cells.iter().map(|c| w(c).specific_thrust / c.unlit.performance.specific_thrust).collect();
        let sf: Vec<f64> = cells.iter().map(|c| w(c).tsfc / c.unlit.performance.tsfc).collect();
        let thn: Vec<f64> = cells.iter().map(|c| w(c).specific_thrust / c.core.performance.specific_thrust).collect();
        let ok = th.windows(2).all(|v| v[1] > v[0]) && sf.windows(2).all(|v| v[1] < v[0]);
        if g != "cpg" { n += 1; if ok { hit += 1; } }
        println!("  {g:11} pc {pi_c:4} Tt4 {tt4} Tt7 {tt7}: thrust x {} | tsfc x {} | [{}] {}", fmt(&th), fmt(&sf), fmt(&thn),
                 if g == "cpg" { "(reported)" } else if ok { "HIT" } else { "MISS" });
    }}}}
    tally("H2", hit, n);

    // ---------------- H3 + H6 (and the derived π_AB, reported)
    let (mut h3, mut n3, mut h6, mut n6) = (0, 0, 0, 0);
    let (mut lo3, mut hi3, mut lo6, mut hi6) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    let (mut lo_pi, mut hi_pi) = (f64::MAX, f64::MIN);
    println!("== H3 (excess of A8 ratio over sqrt(Tt7/Tt5)/pi_R) and H6 (Unlit vs none)");
    for g in GASES { for fl in FLIGHTS { for pi_c in [6.0, 15.0] { for tt4 in [1400.0, 1600.0] {
        let mut row = format!("  {g:11} {:9} pc {pi_c:4} Tt4 {tt4}:", fl.0);
        for tt7 in [1700.0, 1900.0, 2100.0] {
            let c = run(g, fl, pi_c, tt4, tt7);
            let wet = c.wet.as_ref().unwrap();
            let gas = &c.e.engine.gas;      // carries the station-4 and station-7 freezes
            let (s5, s7d, s7w) = (*c.core.station("5"), *c.unlit.station("7"), *wet.station("7"));
            let ratio = area(&s7w, gas) / area(&s7d, gas);
            let rule = (s7w.tt / s5.tt).sqrt() / wet.detail.pi_r;
            let ex = ratio / rule - 1.0;
            if g != "cpg" { n3 += 1; if (0.01..=0.08).contains(&ex) { h3 += 1; } lo3 = lo3.min(ex); hi3 = hi3.max(ex); }
            lo_pi = lo_pi.min(wet.detail.pi_ab); hi_pi = hi_pi.max(wet.detail.pi_ab);
            row += &format!("  Tt7 {tt7}: x{ratio:.4} rule {rule:.4} ex {:+.2}% piAB {:.4}", 100.0 * ex, wet.detail.pi_ab);
        }
        let c = run(g, fl, pi_c, tt4, 1900.0);
        let d6 = c.unlit.performance.specific_thrust / c.core.performance.specific_thrust - 1.0;
        if g != "cpg" { n6 += 1; if (-0.015..=-0.003).contains(&d6) { h6 += 1; } lo6 = lo6.min(d6); hi6 = hi6.max(d6); }
        row += &format!("  | unlit {:+.3}%", 100.0 * d6);
        println!("{row}");
    }}}}
    println!("H3 excess range (scored gases) {:+.2}% .. {:+.2}%", 100.0 * lo3, 100.0 * hi3);
    tally("H3", h3, n3);
    println!("H6 range {:+.3}% .. {:+.3}%", 100.0 * lo6, 100.0 * hi6);
    tally("H6", h6, n6);
    println!("derived pi_AB lit, all cells: {lo_pi:.4} .. {hi_pi:.4}\n");

    // ---------------- H4 + H5
    let (mut h4, mut h5) = (0, 0);
    println!("== H4 (ceiling eq in [2150,2350], reacting >= eq + 50, 2100 runs) and H5 (dissociation fuel at 2100 in [1.5%, 8%])");
    for fl in FLIGHTS { for pi_c in [6.0, 15.0] { for tt4 in [1400.0, 1600.0] {
        let (ce, cr) = (ceiling("equilibrium", fl, pi_c, tt4), ceiling("reacting", fl, pi_c, tt4));
        let ce_ = run("equilibrium", fl, pi_c, tt4, 2100.0);
        let cr_ = run("reacting", fl, pi_c, tt4, 2100.0);
        let runs = ce_.wet.is_ok();
        let ok4 = runs && (2150.0..=2350.0).contains(&ce) && cr >= ce + 50.0;
        let fab = |c: &Cell| { let w = c.wet.as_ref().unwrap(); w.detail.f7 - w.detail.f5 };
        let pen = if runs { fab(&ce_) / fab(&cr_) - 1.0 } else { f64::NAN };
        let ok5 = (0.015..=0.08).contains(&pen);
        if ok4 { h4 += 1; } if ok5 { h5 += 1; }
        println!("  {:9} pc {pi_c:4} Tt4 {tt4}: ceiling eq {ce:.1} reacting {cr:.1} (gap {:.1}) 2100 runs {runs} {} | penalty {:+.2}% {} | passes {}",
                 fl.0, cr - ce, if ok4 { "HIT" } else { "MISS" }, 100.0 * pen, if ok5 { "HIT" } else { "MISS" },
                 ce_.wet.as_ref().map(|w| w.detail.passes).unwrap_or(0));
    }}}
    tally("H4", h4, 16);
    tally("H5", h5, 16);
}
