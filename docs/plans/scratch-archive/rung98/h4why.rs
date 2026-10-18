// Rung 98 — after scoring H4 (4 MISS): where does the uncorrected proportional law read its steady table? Prints the
// carried table's HP-speed range, the demand's range over the run, and how many points read beyond either end row;
// plus the exponent k of the steady fuel against HP speed on the design table (the derivation's assumption, k > 1).
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{controls, ControlsSettings, SpeedLaw, CONTROL_SHAPES};

fn main() {
    for shape in 0..4 {
        for (fname, t0) in [("hot", 290.0), ("cold", 216.65)] {
            for ct in [false, true] {
                let s = ControlsSettings { shape, govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 20.0,
                                           corr_t: ct, t0, ..ControlsSettings::defaults() };
                let (sv, o) = controls(&s).expect("runs");
                let g = sv.governor.as_ref().unwrap();
                let (lo, hi) = (g.table[0].0, g.table[g.table.len() - 1].0);
                let dem: Vec<f64> = o.points.iter().map(|p| g.demand(p.s)).collect();
                let (dlo, dhi) = (dem.iter().cloned().fold(f64::MAX, f64::min), dem.iter().cloned().fold(f64::MIN, f64::max));
                let beyond = dem.iter().filter(|&&d| d < lo || d > hi).count();
                // k = dln W / dln ν over the table's end rows
                let k = (g.table[g.table.len() - 1].1 / g.table[0].1).ln() / (hi / lo).ln();
                println!("{:10} {fname:4} θ-corr {ct:5}: table νH {lo:.4}–{hi:.4}, demand {dlo:.4}–{dhi:.4}, points beyond the table {beyond}/{}, k {k:.2}",
                         CONTROL_SHAPES[shape], dem.len());
            }
        }
    }
}
