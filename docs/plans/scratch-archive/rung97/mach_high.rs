use turbojet::sandbox_controls::{controls, ControlsSettings};
fn main() {
    let d = ControlsSettings::defaults();
    for shape in 0..4 { for m0 in [3.0, 3.5] { for (from, to) in [(1000.0, 1400.0), (1300.0, 1600.0), (1500.0, 1700.0)] {
        let s = ControlsSettings { shape, m0, from, to, ..d };
        match controls(&s) { Ok(_) => println!("shape {shape} M {m0} {from}->{to}: OK"),
            Err(e) => println!("shape {shape} M {m0} {from}->{to}: REFUSED {}", e.0.chars().take(70).collect::<String>()) }
    }}}
}
