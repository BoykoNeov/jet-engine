//! Roadmap A2: every number `docs/sandbox/EXPERIMENTS.md` quotes, held to the model the page runs.
//!
//! **The guide is the EXPECTED side; this file never holds a number of its own.** Each table is read
//! from the guide — its headers and row labels decide the requests (edit a header and the run changes)
//! — and each quoted sentence is a template here whose holes are read from the guide's text. The model
//! is asked through `sandbox::call`, the same entry point the page's worker calls, with the page's own
//! readings (thrust in kN, fuel per thrust in g/(kN·s), …). Native and browser numbers differ by ~1e-10
//! (`docs/sandbox/README.md`), far below any digit the guide shows.
//!
//! **The bar is the guide's own rounding**: half a unit in the last digit shown ("15.97" ± 0.005,
//! "1695" ± 0.5). Two cell kinds have their own rule: `~0` (rounds to 0.00) and the words naming which
//! metal limits the turbine.
//!
//! **A census holds the coverage.** Every number in the guide must be claimed — inside a checked table,
//! or a checked sentence — or listed in its section's `exempt` list WITH A REASON (a knob the user sets,
//! a date, a rounded "about"). A number in none of these fails, so a row or sentence added to the guide
//! cannot pass unchecked.
//!
//! All mismatches are collected and reported together, so one run shows every number that moved.

use std::cell::RefCell;
use std::collections::HashMap;

use turbojet::sandbox::call;
use turbojet::visuals::Json;

const GUIDE: &str = "docs/sandbox/EXPERIMENTS.md";

fn read_guide() -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(GUIDE);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {GUIDE}: {e}"))
}

// ---------------------------------------------------------------- the guide, parsed

/// One heading's text: its prose (whitespace collapsed, tables taken out) and its tables.
struct Section {
    key: String,
    prose: String,
    tables: Vec<Vec<Vec<String>>>,
}

/// `### 12. The nitrogen-oxide bell` → "12"; `## Controls` → "Controls"; before any heading → "(top)".
fn section_key(heading: &str) -> String {
    let t = heading.trim_start_matches('#').trim();
    match t.split_once(". ") {
        Some((n, _)) if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) => n.to_string(),
        _ => t.to_string(),
    }
}

fn sections(doc: &str) -> Vec<Section> {
    let mut out = vec![Section { key: "(top)".into(), prose: String::new(), tables: Vec::new() }];
    let mut in_table = false;
    for line in doc.lines() {
        let l = line.trim();
        if l.starts_with('#') {
            out.push(Section { key: section_key(l), prose: String::new(), tables: Vec::new() });
            in_table = false;
        } else if l.starts_with('|') {
            let cells: Vec<String> = l.trim_matches('|').split('|').map(|c| c.trim().to_string()).collect();
            if cells.iter().all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':')) { continue; }
            let s = out.last_mut().unwrap();
            if !in_table { s.tables.push(Vec::new()); }
            s.tables.last_mut().unwrap().push(cells);
            in_table = true;
        } else {
            in_table = false;
            let s = out.last_mut().unwrap();
            for w in l.split_whitespace() {
                if !s.prose.is_empty() { s.prose.push(' '); }
                s.prose.push_str(w);
            }
        }
    }
    out
}

/// A number as written at `s[i..]`: an optional `~` or `+`, digits, and a decimal part only if a digit
/// follows the point (a sentence's full stop is not part of the number). Returns (text, end).
fn number_at(s: &str, i: usize) -> Option<(&str, usize)> {
    let b = s.as_bytes();
    let mut j = i;
    if j < b.len() && (b[j] == b'~' || b[j] == b'+') { j += 1; }
    let d0 = j;
    while j < b.len() && b[j].is_ascii_digit() { j += 1; }
    if j == d0 { return None; }
    if j + 1 < b.len() && b[j] == b'.' && b[j + 1].is_ascii_digit() {
        j += 1;
        while j < b.len() && b[j].is_ascii_digit() { j += 1; }
    }
    Some((&s[i..j], j))
}

/// Every number token in `s` (a digit run not glued to a letter, so "A2" and "H₂O" are words, not numbers).
fn tokens(s: &str) -> Vec<String> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let glued = i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'.' || b[i - 1] == b'_');
        if b[i].is_ascii_digit() && !glued {
            let (t, j) = number_at(s, i).unwrap();
            out.push(t.to_string());
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// The span of `prose` a template matches, and the numbers in its holes (`{}`). The literal pieces must
/// match exactly; a hole takes one number as written.
fn match_template<'a>(prose: &'a str, template: &str) -> Option<(usize, usize, Vec<&'a str>)> {
    let pieces: Vec<&str> = template.split("{}").collect();
    let mut from = 0;
    while let Some(k) = prose[from..].find(pieces[0]) {
        let start = from + k;
        let mut at = start + pieces[0].len();
        let mut holes = Vec::new();
        let mut ok = true;
        for p in &pieces[1..] {
            match number_at(prose, at) {
                Some((t, j)) if prose[j..].starts_with(p) => { holes.push(t); at = j + p.len(); }
                _ => { ok = false; break; }
            }
        }
        if ok { return Some((start, at, holes)); }
        from = start + 1;
    }
    None
}

// ---------------------------------------------------------------- the bar

/// What the model gives for one quoted number.
enum Model {
    Num(f64),
    Text(String),
}

/// Compare one number as the guide shows it with the model's: within half a unit of its last digit.
/// A trailing unit ("1500 K", "0.5 τ") is dropped first.
fn agrees(shown: &str, model: &Model) -> Result<(), String> {
    let s = shown.trim().trim_end_matches(" K").trim_end_matches(" %").trim_end_matches(" τ").trim();
    match model {
        Model::Text(t) => if s == t { Ok(()) } else { Err(format!("the model says {t:?}")) },
        Model::Num(x) if s == "~0" => {
            if x.abs() < 0.005 { Ok(()) } else { Err(format!("the model gives {x} (not ~0: it would show {x:.2})")) }
        }
        Model::Num(x) => {
            let v: f64 = s.trim_start_matches('+').parse().map_err(|_| format!("{shown:?} is not a number"))?;
            let dec = s.split_once('.').map_or(0, |(_, d)| d.len());
            let half = 0.5 * 10f64.powi(-(dec as i32));
            if (x - v).abs() <= half * (1.0 + 1e-9) {
                Ok(())
            } else {
                Err(format!("the model gives {x:.prec$} (bar ±{half})", prec = dec + 2))
            }
        }
    }
}

// ---------------------------------------------------------------- the model, asked as the page asks

thread_local! { static CACHE: RefCell<HashMap<String, Json>> = RefCell::new(HashMap::new()); }

fn ask(req: Json) -> Json {
    let text = req.dump_compact();
    if let Some(r) = CACHE.with(|c| c.borrow().get(&text).cloned()) { return r; }
    let r = Json::parse(&call(&text));
    assert!(!matches!(r.get("ok"), Some(Json::Int(0))), "{text} does not run: {:?}", r.get("reason"));
    CACHE.with(|c| c.borrow_mut().insert(text, r.clone()));
    r
}

fn obj(kv: &[(&str, Json)]) -> Json { Json::Obj(kv.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()) }

/// `base` with `patch`'s keys replaced or added (the page's `Object.assign`).
fn merged(base: &Json, patch: &[(&str, Json)]) -> Json {
    let Json::Obj(b) = base else { panic!("merge needs an object") };
    let mut out = b.clone();
    for (k, v) in patch {
        match out.iter_mut().find(|(o, _)| o == k) {
            Some(slot) => slot.1 = v.clone(),
            None => out.push((k.to_string(), v.clone())),
        }
    }
    Json::Obj(out)
}

fn num(j: &Json) -> f64 {
    match j { Json::Float(x) => *x, Json::Int(n) => *n as f64, o => panic!("not a number: {o:?}") }
}
fn list(j: &Json) -> Vec<f64> {
    match j { Json::List(v) => v.iter().map(num).collect(), o => panic!("not a list: {o:?}") }
}
fn text(j: &Json) -> String { match j { Json::Str(s) => s.clone(), o => panic!("not a string: {o:?}") } }
fn f(x: f64) -> Json { Json::Float(x) }
fn s(x: &str) -> Json { Json::Str(x.into()) }
fn i(n: i64) -> Json { Json::Int(n) }

/// A number the guide wrote in a header or label, as a knob value.
fn knob(shown: &str) -> f64 {
    let t = shown.trim().trim_end_matches(" τ").trim_end_matches(" K").trim();
    t.parse().unwrap_or_else(|_| panic!("{shown:?} is not a knob value"))
}

fn defaults(op: &str, key: Option<&str>) -> Json {
    let r = ask(obj(&[("op", s(op))]));
    match key { Some(k) => r.at(k).clone(), None => r }
}

/// The Design view's run.
fn design(p: &[(&str, Json)]) -> Json {
    ask(obj(&[("op", s("run")), ("settings", merged(&defaults("defaults", None), p))]))
}
/// Fly it's operating point.
fn fly(p: &[(&str, Json)]) -> Json {
    ask(obj(&[("op", s("fly")), ("fly", merged(&defaults("fly_defaults", None), p))]))
}
/// Fly it's slam, from its starting throttle up to the fly throttle (as the page sends it).
fn slam(ramp: f64, mode: &str) -> Json {
    let fl = defaults("fly_defaults", None);
    let sl = defaults("slam_defaults", Some("slam"));
    ask(obj(&[("op", s("slam")), ("slam", obj(&[("fly", fl.clone()), ("from", sl.at("from").clone()),
        ("to", fl.at("Tt4").clone()), ("ramp", f(ramp)), ("settle", sl.at("settle").clone()), ("mode", s(mode))]))]))
}
fn controls(p: &[(&str, Json)]) -> Json {
    ask(obj(&[("op", s("controls")), ("controls", merged(&defaults("controls_defaults", Some("controls")), p))]))
}
fn burner(p: &[(&str, Json)]) -> Json {
    ask(obj(&[("op", s("burner")), ("settings", defaults("defaults", None)),
              ("burner", merged(&defaults("burner_defaults", Some("burner")), p))]))
}
fn cooling(p: &[(&str, Json)]) -> Json {
    ask(obj(&[("op", s("cooling_knob")), ("cooling", merged(&defaults("cooling_defaults", Some("cooling")), p))]))
}

// The page's readings (`docs/sandbox/template.html`: the tiles, `PERF`, the Controls rows).
fn kn(r: &Json) -> f64 { num(r.at("thrust")) / 1e3 }
fn tsfc(r: &Json) -> f64 { num(r.at("tsfc")) * 1e6 }
fn pct(r: &Json, k: &str) -> f64 { num(r.at(k)) * 100.0 }
fn peak(r: &Json) -> f64 { list(r.at("Tt4")).into_iter().fold(f64::NEG_INFINITY, f64::max) }
/// The first time the thrust has made `frac` of its change: the page's `to90` (a point of the run, not
/// interpolated). Controls shows it at 95 % ("Time to 95 % of the thrust change").
fn to_frac(r: &Json, frac: f64) -> f64 {
    let (a, b) = (num(r.at("start").at("thrust")), num(r.at("end").at("thrust")));
    let th = list(r.at("thrust"));
    let k = th.iter().position(|t| (t - a) / (b - a) >= frac).expect("the thrust never makes the fraction");
    list(r.at("s"))[k]
}
/// "Thrust at the end": the run's last point (N per kg/s), not the final steady thrust it is still short of.
fn end_thrust(r: &Json) -> f64 { *list(r.at("thrust")).last().unwrap() }

/// Sea level, as the page's atmosphere sets it (`tests/sandbox_page.rs` holds these to the atmosphere).
fn sea_level() -> [(&'static str, Json); 2] { [("T0", f(288.15)), ("p0", f(101325.0))] }

// ---------------------------------------------------------------- the experiments

type Cell = fn(&str, &str) -> Option<Model>;
type Values = fn() -> Vec<f64>;

/// One section of the guide: how its table cells are read (row label, column header → the model), the
/// sentences it quotes, and the numbers in it that are NOT model outputs, each with its reason.
struct Exp {
    key: &'static str,
    table: Option<Cell>,
    prose: Vec<(&'static str, Values)>,
    exempt: &'static [(&'static str, &'static str)],
}

fn n(x: f64) -> Option<Model> { Some(Model::Num(x)) }

fn experiments() -> Vec<Exp> {
    vec![
        Exp { key: "Experiments to try in the sandbox", table: None, prose: vec![], exempt: &[
            ("2026", "the date the numbers were read"), ("11", "the date"), ("12", "the date"),
        ] },
        Exp { key: "Design the engine", table: None, prose: vec![
            ("The opening design: pressure ratio {}, turbine inlet {} K, Mach {} at about {} km. Thrust {} kN, fuel per thrust {}.",
             || { let r = design(&[]); vec![num(r.at("settings").at("pi_c")), num(r.at("settings").at("Tt4")),
                  num(r.at("settings").at("M0")), num(r.at("ambient").at("altitude")) / 1e3, kn(&r), tsfc(&r)] }),
        ], exempt: &[] },
        Exp { key: "1", table: Some(|row, col| {
            let r = design(&[("pi_c", f(knob(col)))]);
            match row { "Thrust, kN" => n(kn(&r)), "Fuel per thrust" => n(tsfc(&r)), _ => None }
        }), prose: vec![], exempt: &[("15", "\"about 15\": read off the table above")] },
        Exp { key: "2", table: Some(|row, col| {
            let r = design(&[("Tt4", f(knob(col)))]);
            match row {
                "Thrust, kN" => n(kn(&r)), "Fuel per thrust" => n(tsfc(&r)),
                "Propulsive efficiency, %" => n(pct(&r, "eta_propulsive_eq")), _ => None,
            }
        }), prose: vec![], exempt: &[("10", "the knob set back to its opening value")] },
        Exp { key: "3", table: Some(|row, col| {
            let gas = match row {
                "Perfect gas (constant heat capacity)" => "perfect",
                "Thermally perfect (heat capacity grows with temperature)" => "thermally_perfect",
                "Reacting products / equilibrium chemistry" => "equilibrium",
                _ => return None,
            };
            let r = design(&[("gas", s(gas))]);
            match col { "Thrust, kN" => n(kn(&r)), "Fuel per thrust" => n(tsfc(&r)), _ => None }
        }), prose: vec![
            ("so it under-counts the fuel per thrust by about {} % and the thrust by about {} %",
             || { let (p, t) = (design(&[("gas", s("perfect"))]), design(&[("gas", s("thermally_perfect"))]));
                  // "about": the model's shortfall, rounded to the whole percent the guide shows
                  vec![(1.0 - tsfc(&p) / tsfc(&t)) * 100.0, (1.0 - kn(&p) / kn(&t)) * 100.0] }),
        ], exempt: &[("12", "a pointer to experiment 12")] },
        Exp { key: "4", table: None, prose: vec![
            ("Thrust rises from {} to {} kN and fuel per thrust falls from {} to {}.",
             || { let (a, b) = (design(&[]), design(&[("eta_c", i(1)), ("eta_t", i(1)), ("eta_b", i(1)), ("pi_b", i(1)),
                                                     ("pi_n", i(1)), ("pi_d_max", i(1)), ("eta_m", i(1))]));
                  vec![kn(&a), kn(&b), tsfc(&a), tsfc(&b)] }),
        ], exempt: &[("1", "the knobs' ideal value")] },
        Exp { key: "5", table: Some(|row, col| {
            let r = design(&[("M0", f(knob(col)))]);
            match row { "Thrust, kN" => n(kn(&r)), "Overall efficiency, %" => n(pct(&r, "eta_overall")), _ => None }
        }), prose: vec![
            ("(The design airflow is held at {} kg/s here;", || vec![num(defaults("defaults", None).at("mdot"))]),
        ], exempt: &[] },
        Exp { key: "6", table: None, prose: vec![
            ("a sea-level test stand. Thrust {} kN, fuel per thrust {}. Overall and propulsive efficiency read exactly {} —",
             || { let r = design(&[("M0", i(0)), sea_level()[0].clone(), sea_level()[1].clone()]);
                  // "exactly": the bar is 0 itself, not the rounding
                  assert_eq!((num(r.at("eta_overall")), num(r.at("eta_propulsive_eq"))), (0.0, 0.0),
                             "experiment 6: the standing engine's efficiencies are not exactly 0");
                  vec![kn(&r), tsfc(&r), pct(&r, "eta_overall")] }),
        ], exempt: &[("0", "the knobs set to sea level and standing still"), ("0", "the same"),
                     ("2026", "the date the views gained the standing engine"), ("11", "the date"), ("12", "the date")] },
        Exp { key: "7", table: None, prose: vec![
            ("Thrust falls from {} to {} kN.", || vec![kn(&design(&[])), kn(&design(&[("nozzle", s("convergent"))]))]),
            ("\"thermal\" drops to {} % and \"propulsive\" reads {} %.",
             || { let r = design(&[("nozzle", s("convergent"))]); vec![pct(&r, "eta_thermal"), pct(&r, "eta_propulsive")] }),
            ("for the same thrust: {} m/s, against a real jet of {} m/s) counts it: thermal {} %, propulsive {} %, near the fully expanded engine's {} % and {} %.",
             || { let (c, e) = (design(&[("nozzle", s("convergent"))]), design(&[]));
                  vec![num(c.at("V_eq")), num(c.at("V9")), pct(&c, "eta_thermal_eq"), pct(&c, "eta_propulsive_eq"),
                       pct(&e, "eta_thermal_eq"), pct(&e, "eta_propulsive_eq")] }),
            ("Overall efficiency is the same in both ({} %; {} % fully expanded).",
             || vec![pct(&design(&[("nozzle", s("convergent"))]), "eta_overall"), pct(&design(&[]), "eta_overall")]),
        ], exempt: &[("2026", "the date of roadmap A6"), ("11", "the date"), ("15", "the date")] },
        Exp { key: "8", table: None, prose: vec![
            ("Thrust rises from {} to {} kN (+{} %); fuel per thrust rises from {} to {} (+{} %). The nozzle throat must open to {} times its unlit size.",
             || { let (a, b) = (design(&[]), design(&[("afterburner", i(1))]));
                  vec![kn(&a), kn(&b), (kn(&b) / kn(&a) - 1.0) * 100.0, tsfc(&a), tsfc(&b), (tsfc(&b) / tsfc(&a) - 1.0) * 100.0,
                       num(b.at("afterburner").at("a8_ratio"))] }),
        ], exempt: &[] },
        Exp { key: "9", table: Some(|row, col| {
            let r = fly(&[("Tt4", f(knob(col)))]);
            match row {
                "Thrust, kN" => n(kn(&r)), "Shaft speed, % of design" => n(pct(&r, "nu")),
                "Stall margin, %" => n(num(r.at("stall").at("sm_n")) * 100.0), _ => None,
            }
        }), prose: vec![], exempt: &[] },
        Exp { key: "10", table: Some(|row, col| {
            let mode = match col {
                "Peak turbine inlet, temperature commanded" => "temperature", "Peak, fuel metered" => "fuel", _ => return None,
            };
            n(peak(&slam(knob(row), mode)))
        }), prose: vec![
            ("the move from {} K to {} K in half a shaft response time.",
             || { let sl = defaults("slam_defaults", Some("slam"));
                  assert_eq!(num(sl.at("ramp")), 0.5, "experiment 10: \"half a shaft response time\" is not the opening ramp");
                  vec![num(sl.at("from")), num(defaults("fly_defaults", None).at("Tt4"))] }),
        ], exempt: &[] },
        Exp { key: "11", table: None, prose: vec![
            ("thrust *falls* from {} kN ({} with the afterburner fitted but unlit) to {} kN. Now tick **Open the nozzle by the rule**: {} kN.",
             || vec![kn(&fly(&[])), kn(&fly(&[("ab", s("unlit"))])), kn(&fly(&[("ab", s("lit"))])),
                     kn(&fly(&[("ab", s("lit")), ("a8_match", i(1))]))]),
        ], exempt: &[] },
        Exp { key: "12", table: Some(|row, col| {
            let r = burner(&[("phi_p", f(knob(col)))]);
            match row {
                "Front-zone temperature, K" => n(num(r.at("state").at("t_primary"))),
                "NOx, g per kg of fuel" => n(num(r.at("ei_total"))), _ => None,
            }
        }), prose: vec![], exempt: &[("1", "the richness's meaning: 1 is exactly enough air")] },
        Exp { key: "13", table: Some(|row, col| {
            let r = burner(&[("phi_p", f(1.5)), ("quench", s("time")), ("tau_q", f(knob(col) / 1e3))]);
            match row { "NOx, g per kg of fuel" => n(num(r.at("ei_total"))), _ => None }
        }), prose: vec![
            // The hottest point on the way through the dilution, at every dilution time the table shows.
            ("(it peaks at {} K on the way)", || [0.2, 1.0, 3.0, 10.0].iter()
                .map(|ms| num(burner(&[("phi_p", f(1.5)), ("quench", s("time")), ("tau_q", f(ms / 1e3))]).at("state").at("t_peak")))
                .collect()),
        ], exempt: &[("1.5", "the knob the experiment sets"), ("12", "a pointer to experiment 12")] },
        Exp { key: "Controls", table: None, prose: vec![
            ("the throttle slammed from {} K to {} K in {} τ.",
             || { let c = defaults("controls_defaults", Some("controls"));
                  vec![num(c.at("from")), num(c.at("to")), num(c.at("ramp"))] }),
            ("the turbine inlet peaks at **{} K** before settling at {} K, and thrust reaches 95 % of its change after {} τ.",
             || { let r = controls(&[]); vec![peak(&r), num(r.at("end").at("Tt4")), to_frac(&r, 0.95)] }),
        ], exempt: &[] },
        Exp { key: "14", table: Some(|row, col| {
            let r = controls(&[("ramp", f(knob(col)))]);
            match row {
                "Peak turbine inlet, K" => n(peak(&r)),
                "Time to 95 % of the thrust change, τ" => n(to_frac(&r, 0.95)), _ => None,
            }
        }), prose: vec![], exempt: &[] },
        Exp { key: "15", table: Some(|row, col| {
            let r = match row {
                "None" => controls(&[]),
                "Temperature limiter at 1480 K" => {
                    assert_eq!(num(defaults("controls_defaults", Some("controls")).at("redline")), 1480.0, "the redline");
                    controls(&[("redline_on", i(1))])
                }
                "The same limiter with a reaction lag of 0.2 τ (untick the two-speed box)" => {
                    assert_eq!(num(defaults("controls_defaults", Some("controls")).at("gov_lag")), 0.2, "the lag");
                    controls(&[("redline_on", i(1)), ("gov_lag_on", i(1)), ("gov_lag_two", i(0))])
                }
                "Acceleration schedule" => controls(&[("accel_on", i(1))]),
                _ => return None,
            };
            match col {
                "Peak turbine inlet, K" => n(peak(&r)), "Time to 95 % thrust, τ" => n(to_frac(&r, 0.95)), _ => None,
            }
        }), prose: vec![], exempt: &[("0.5", "the opening ramp")] },
        Exp { key: "16", table: None, prose: vec![
            ("The thrust now reaches 95 % of its change in {} τ instead of {} τ — and the turbine inlet peaks at **{} K** instead of {} K.",
             || { let (g, o) = (controls(&[("ramp", f(0.1)), ("hold", s("thrust")), ("govern", i(1))]), controls(&[("ramp", f(0.1))]));
                  vec![to_frac(&g, 0.95), to_frac(&o, 0.95), peak(&g), peak(&o)] }),
        ], exempt: &[("0.1", "the ramp the experiment sets")] },
        Exp { key: "17", table: Some(|row, col| {
            let r = match row {
                "None" => controls(&[]),
                "Generator on the HP shaft" => controls(&[("generator", s("hp"))]),
                "Generator on the LP shaft" => controls(&[("generator", s("lp"))]),
                "The aircraft takes 5 % of the HP compressor's air" =>
                    controls(&[("customer", s("share")), ("customer_share", f(0.05))]),
                _ => return None,
            };
            match col { "Thrust at the end" => n(end_thrust(&r)), "Peak turbine inlet, K" => n(peak(&r)), _ => None }
        }), prose: vec![
            ("(it opens at {}):", || vec![end_thrust(&controls(&[]))]),
        ], exempt: &[] },
        Exp { key: "18", table: None, prose: vec![
            ("The same slam peaks at {} K instead of {} K and the thrust response is quicker ({} τ against {} τ)",
             || { let (sl, o) = (controls(&sea_level()), controls(&[]));
                  vec![peak(&sl), peak(&o), to_frac(&sl, 0.95), to_frac(&o, 0.95)] }),
        ], exempt: &[("0", "the altitude knob set to sea level")] },
        Exp { key: "19", table: Some(|row, col| {
            let tech = match col {
                "Convection" => "convection", "Coated" => "coated", "Film" => "film", "Transpiration" => "transpiration",
                _ => return None,
            };
            let y = cooling(&[("tech", s(tech))]).at("yours").clone();
            match row {
                "Hottest turbine inlet, K" => n(num(y.at("Tt4"))),
                "What limits it" => Some(Model::Text(format!("{}s", text(y.at("binding"))))),
                _ => None,
            }
        }), prose: vec![
            ("(with the opening cooling air: {} % to the vanes, {} % to the blades).",
             || { let c = defaults("cooling_defaults", Some("cooling")); vec![pct(&c, "vane"), pct(&c, "blade")] }),
            ("With no cooling air at all the same metal allows only {} K. Raise the air to 8 % and 5 % (film) and the limit climbs to {} K.",
             || vec![num(cooling(&[]).at("bare").at("Tt4")),
                     num(cooling(&[("vane", f(0.08)), ("blade", f(0.05))]).at("yours").at("Tt4"))]),
        ], exempt: &[("2", "a pointer to experiment 2")] },
    ]
}

// ---------------------------------------------------------------- the checks

#[test]
fn every_number_the_guide_quotes_is_the_models() {
    let doc = read_guide();
    let secs = sections(&doc);
    let mut bad: Vec<String> = Vec::new();
    for e in experiments() {
        let Some(sec) = secs.iter().find(|s| s.key == e.key) else { bad.push(format!("no section {:?}", e.key)); continue };
        if let Some(cell) = e.table {
            assert!(!sec.tables.is_empty(), "section {:?} has a table reader but no table", e.key);
            for t in &sec.tables {
                let head = &t[0];
                for row in &t[1..] {
                    for (c, shown) in row.iter().enumerate().skip(1) {
                        let (r, h) = (row[0].as_str(), head[c].as_str());
                        // A table either reads inputs along its top (a sweep) or along its side (cases): the
                        // reader is asked with (row label, column header) either way.
                        match cell(r, h) {
                            None => bad.push(format!("§ {}: no reading for row {r:?}, column {h:?} ({shown:?})", e.key)),
                            Some(m) => if let Err(why) = agrees(shown, &m) {
                                bad.push(format!("§ {}: row {r:?}, column {h:?} shows {shown}, {why}", e.key))
                            },
                        }
                    }
                }
            }
        }
        for (tpl, values) in &e.prose {
            let Some((_, _, holes)) = match_template(&sec.prose, tpl) else {
                bad.push(format!("§ {}: the sentence is not in the guide: {tpl:?}", e.key));
                continue;
            };
            let vals = values();
            // One value per hole, except where one quoted number is read at several settings (experiment 13).
            let per_hole: Vec<Vec<f64>> = if holes.len() == 1 && vals.len() > 1 {
                vec![vals]
            } else {
                assert_eq!(holes.len(), vals.len(), "§ {}: holes vs values for {tpl:?}", e.key);
                vals.into_iter().map(|v| vec![v]).collect()
            };
            for (shown, vs) in holes.iter().zip(per_hole) {
                for v in vs {
                    if let Err(why) = agrees(shown, &Model::Num(v)) {
                        bad.push(format!("§ {}: {shown} in {tpl:?}: {why}", e.key))
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "{} number(s) in {GUIDE} differ from the model:\n  {}", bad.len(), bad.join("\n  "));
}

/// Every number in the guide is claimed by a checked table or sentence, or exempt with a reason.
#[test]
fn every_number_in_the_guide_is_checked_or_exempt_with_a_reason() {
    let doc = read_guide();
    let exps = experiments();
    let mut bad: Vec<String> = Vec::new();
    for sec in sections(&doc) {
        let e = exps.iter().find(|e| e.key == sec.key);
        // What the checks do not claim: the prose outside every matched sentence, and any table no reader reads.
        let mut rest = sec.prose.clone();
        if let Some(e) = e {
            for (tpl, _) in &e.prose {
                if let Some((a, b, _)) = match_template(&rest, tpl) { rest.replace_range(a..b, " "); }
            }
        }
        let mut left = tokens(&rest);
        if e.and_then(|e| e.table).is_none() {
            for t in &sec.tables { for row in t { for c in row { left.extend(tokens(c)); } } }
        }
        let mut exempt: Vec<&str> = e.map_or(Vec::new(), |e| e.exempt.iter().map(|(t, _)| *t).collect());
        for t in &left {
            match exempt.iter().position(|x| x == t) {
                Some(k) => { exempt.remove(k); }
                None => bad.push(format!("§ {}: {t} is neither checked nor exempt", sec.key)),
            }
        }
        for t in exempt { bad.push(format!("§ {}: exempt {t} is not in the section (stale exemption)", sec.key)); }
    }
    for e in &exps { assert!(e.exempt.iter().all(|(_, why)| !why.is_empty()), "§ {}: an exemption without a reason", e.key); }
    assert!(bad.is_empty(), "{GUIDE}'s census:\n  {}", bad.join("\n  "));
}

#[test]
fn the_template_matcher_and_the_bar_behave() {
    let p = "Thrust rises from 15.97 to 17.52 kN. Then 3.";
    let (_, _, h) = match_template(p, "from {} to {} kN.").unwrap();
    assert_eq!(h, ["15.97", "17.52"]);
    assert!(match_template(p, "from {} to {} N.").is_none());
    assert_eq!(tokens("A2 at 0.5 τ, 2026-11-12, +39 % and 10."), ["0.5", "2026", "11", "12", "39", "10"]);
    assert!(agrees("15.97", &Model::Num(15.974)).is_ok() && agrees("15.97", &Model::Num(15.976)).is_err());
    assert!(agrees("1500 K", &Model::Num(1500.4)).is_ok() && agrees("1695", &Model::Num(1695.6)).is_err());
    assert!(agrees("~0", &Model::Num(0.004)).is_ok() && agrees("~0", &Model::Num(0.006)).is_err());
    assert!(agrees("+39", &Model::Num(39.4)).is_ok());
}

