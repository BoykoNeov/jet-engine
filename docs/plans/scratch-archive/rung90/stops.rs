//! Which Controls requests does the amount mode stop or refuse? (sandbox knob, after scoring)
use turbojet::sandbox_controls::*;
use turbojet::visuals::Json;

fn main() {
    let d = ControlsSettings::defaults();
    for w in [0.2, 0.3, 0.35, 0.4, 0.45] {
        for (from, to, ramp) in [(1000.0, 1400.0, 0.5), (1400.0, 900.0, 0.1), (1400.0, 800.0, 0.04), (1200.0, 700.0, 0.1)] {
            let s = ControlsSettings { customer: CustomerMode::Amount, customer_amount: w, from, to, ramp, ..d };
            let r = Json::parse(&turbojet::sandbox::call(&jobj_req(&s)));
            let ok = matches!(r.get("ok"), Some(Json::Int(1)));
            let what = if ok {
                match r.get("stop") { Some(Json::Null) | None => "complete".to_string(),
                                      Some(st) => format!("STOP {:?}", st.get("kind")) }
            } else { format!("REFUSED {:?}", r.get("reason")).chars().take(160).collect() };
            println!("w={w} {from}->{to} ramp {ramp}: {what}");
        }
    }
}

fn jobj_req(s: &ControlsSettings) -> String {
    format!(r#"{{"op":"controls","controls":{}}}"#, s.to_json().dump_compact())
}
