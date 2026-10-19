// Part B through the page's entry point.
use turbojet::sandbox::call;
use turbojet::visuals::Json;

fn main() {
    for req in [
        r#"{"op":"fly","fly":{"ab":"lit","Tt7":1900}}"#,
        r#"{"op":"fly","fly":{"ab":"lit","Tt7":1900,"a8_match":1}}"#,
        r#"{"op":"fly","fly":{"ab":"lit","Tt7":1900,"a8_ratio":1.15}}"#,
        r#"{"op":"fly","fly":{"ab":"unlit"}}"#,
        r#"{"op":"fly","fly":{"ab":"unlit","a8_ratio":1.3}}"#,
        r#"{"op":"fly","fly":{"ab":"lit","Tt7":2600}}"#,
        r#"{"op":"fly","fly":{"ab":"lit","Tt7":1900,"a8_ratio":2.5}}"#,
        r#"{"op":"fly","fly":{"ab":"lit","Tt7":1900,"Tt4":900}}"#,
        r#"{"op":"running_point","fly":{"ab":"lit","Tt7":1900,"a8_match":1,"Tt4":1300}}"#,
        r#"{"op":"slam","slam":{"fly":{"ab":"lit"}}}"#,
        r#"{"op":"fly_defaults"}"#,
    ] {
        let r = std::panic::catch_unwind(|| call(req));
        match r {
            Err(e) => println!("{req}\n   PANIC {:?}", e.downcast_ref::<String>()),
            Ok(out) => {
                let j = Json::parse(&out);
                let g = |k: &str| j.get(k).map(|v| v.dump_compact()).unwrap_or_default();
                if out.len() < 600 { println!("{req}\n   {out}"); continue; }
                println!("{req}\n   thrust {} nu {} choked {} stall {} M9 {}\n   ab {}", g("thrust"), g("nu"), g("choked"), g("stall"), g("M9"), g("afterburner"));
                let st = j.get("stations").map(|v| v.dump_compact()).unwrap_or_default();
                println!("   stations {}", &st[..st.len().min(400)]);
            }
        }
    }
}
