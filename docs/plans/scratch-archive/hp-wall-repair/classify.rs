// Classify the crash map's remaining "off the maps" refusals: does the steady matcher (bare plant, rung 103's switch on)
// find a point at the refused throttle, and does the bare equilibrium solve there (from full speed / from that point)?
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{control_maps, ControlsSettings, CONTROL_SHAPES};
use turbojet::two_spool_transient::TwoSpoolTransientCore;
use turbojet::visuals::Json;

fn main() {
    let text = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let bs = BladeSettings::defaults();
    for line in text.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        let req = Json::parse(f[1]);
        let Json::Obj(top) = &req else { panic!() };
        let c = top.iter().find(|(k, _)| k == "controls").unwrap().1.clone();
        let s = ControlsSettings::from_json(&c).unwrap();
        let reason = f[2];
        let at = reason.find("throttle (").map(|k| &reason[k + 10..]).unwrap();
        let tt4: f64 = at[..at.find(' ').unwrap()].parse().unwrap();
        let which = if reason.contains("starting") { "start" } else { "final" };
        let shape = CONTROL_SHAPES[s.shape];
        let (ml, mh) = control_maps(shape).unwrap();
        let tr = TwoSpoolTransientCore::new(bs.design(), bs.flight(), 1.0, ml, mh, 1.0);
        tr.inner.base.below_choke.set(true);
        let fl = s.flight();
        let st = tr.inner.try_match_point(&fl, tt4);
        let eq = tr.try_equilibrium(&fl, tt4, None);
        let extras = format!("{} {}{}{}", s.lever.key(), if s.customer_bleed().is_some() { "cust " } else { "" },
                             if s.turbine_cooling().is_some() { "cool " } else { "" }, if s.shaft_offtake().is_some() { "gen " } else { "" });
        let sts = match &st { Ok(m) => format!("steady ok ({:.3},{:.3}) F={:.1}", m.n_lp_ratio, m.n_hp_ratio, m.base.thrust), Err(e) => format!("steady ERR {}", &e.0[..e.0.len().min(60)]) };
        let eqs = match &eq { Ok(_) => "bare eq ok".to_string(), Err(e) => format!("bare eq ERR {}", &e.0[..e.0.len().min(45)]) };
        let seed = match &st { Ok(m) => match tr.try_equilibrium(&fl, tt4, Some((m.n_lp_ratio, m.n_hp_ratio))) { Ok(_) => " | seeded ok".into(), Err(e) => format!(" | seeded ERR {}", &e.0[..e.0.len().min(40)]) }, Err(_) => String::new() };
        println!("{} {shape:10} {which} {tt4:4.0} K T0 {:.0} p0 {:.0} M {:.2} [{extras}] | {sts} | {eqs}{seed}", f[0], fl.t0, fl.p0, fl.m0);
    }
}
