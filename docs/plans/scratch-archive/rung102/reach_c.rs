// Part C probe: the hook's REACH — every Controls check request run fitted-unlit, lit held and lit matched (light-off
// at s = 0.25); an UNPATCHED panic names a site the afterburner does not reach yet.
use std::panic::{catch_unwind, AssertUnwindSafe};
use turbojet::afterburner::{self as abm, Afterburner, LpAb, Throat};
use turbojet::sandbox_controls::{build, check_requests, controls, ControlsSettings};
use turbojet::visuals::Json;

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let reqs = check_requests();
    let mut n = 0;
    for (i, r) in reqs.iter().enumerate() {
        let j = Json::parse(r);
        if !matches!(j.get("op"), Some(Json::Str(o)) if o == "controls") { continue; }
        let empty = Json::Obj(Vec::new());
        let Ok(s) = ControlsSettings::from_json(j.get("controls").unwrap_or(&empty)) else { continue };
        n += 1;
        abm::set_lp_ab(None); abm::set_light(None);
        let Ok(sv0) = build(&s) else { println!("#{i}: refused unfitted"); continue };
        let base = &sv0.core.fuel.inner.inner.base;
        let mut line = format!("#{i:3} gov={} thr={:?} gen={:?} cool={} cust={:?} lever={:?}:", s.govern as u8, s.thrust_by, s.generator,
                               (s.cool_vane + s.cool_blade > 0.0) as u8, s.customer, s.lever);
        let nf = catch_unwind(AssertUnwindSafe(|| controls(&s)));
        line += &format!("  NF: {}", match nf { Ok(Ok((_, o))) => o.stop.map_or("ok".to_string(), |st| format!("stop@{:.2}[{}]", st.s_last, st.failure.map_or("?".into(), |f| f.message.chars().take(60).collect::<String>()))), Ok(Err(e)) => format!("refused[{}]", e.0.chars().take(40).collect::<String>()), Err(_) => "PANIC".into() });
        for (lab, th, s_on) in [("unlit", Throat::Held(1.0), 99.0), ("held", Throat::Held(1.0), 0.24), ("match", Throat::Match, 0.24)] {
            abm::LP_TRIALS.with(|c| c.set(0));
            let x = LpAb::capture(base, Afterburner::default(), th);
            abm::set_lp_ab(Some(x)); abm::set_light(Some((s_on, 0.1)));
            let out = catch_unwind(AssertUnwindSafe(|| controls(&s)));
            let word = match out {
                Ok(Ok((_, o))) => match &o.stop {
                    None => "ok".to_string(),
                    Some(st) => format!("stop@{:.2}[{}]", st.s_last, st.failure.as_ref().map_or("?".into(), |f| f.message.chars().take(60).collect::<String>())),
                },
                Ok(Err(e)) => format!("refused[{}]", e.0.chars().take(60).collect::<String>()),
                Err(p) => format!("PANIC[{}]", p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default().chars().take(70).collect::<String>()),
            };
            line += &format!("  {lab}: {word} (lit trials {})", abm::LP_TRIALS.with(|c| c.get()));
            abm::set_lp_ab(None); abm::set_light(None);
        }
        println!("{line}");
    }
    println!("{n} controls requests");
}
