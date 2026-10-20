use turbojet::sandbox::call;
fn main() {
    for (shape, t, extra) in [("flat-lp", 540, ""), ("flow/press", 550, ""), ("flat-lp", 520, r#","lever":"bleed""#), ("flat-lp", 560, r#","lever":"bleed""#)] {
        let r = call(&format!(r#"{{"op":"controls","controls":{{"shape":"{shape}","from":{t},"T0":288.15,"p0":101325{extra}}}}}"#));
        let msg = r.find("\"message\":\"").map(|p| r[p + 11..].split('"').next().unwrap_or("").to_string()).unwrap_or("-".into());
        let w = r.find("\"start\":").and_then(|p| r[p..].find("below_idle").map(|q| r[p+q..].chars().take(30).collect::<String>())).unwrap_or("none".into());
        println!("SL {shape} {t} {extra}: ok {} stopped {} warn {w} {}", r.starts_with(r#"{"ok":1"#), r.contains(r#""stopped":1"#), msg.chars().take(160).collect::<String>());
    }
}
