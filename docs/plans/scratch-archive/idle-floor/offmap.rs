// Probe: the 3 crash-map requests the page still refuses "off the maps" (low-start repair's leftovers).
// For each: which end fails, Newton from a 2-D start grid, a residual sign map, and the load swept down.
use turbojet::bleed_transient::build_scheduled_bleed;
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::limited_bleed::build_limited_bleed;
use turbojet::sandbox::call;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{control_maps, ControlsSettings, CustomerMode, GeneratorMode, CONTROL_SHAPES, MDOT_DESIGN};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::visuals::Json;

fn core_for(s: &ControlsSettings) -> ScheduledStatorCore {
    let (ml, mh) = control_maps(CONTROL_SHAPES[s.shape]).unwrap();
    let builder = if s.valve_slow() { build_lagged_bleed } else if s.valve_on() { build_limited_bleed } else { build_scheduled_bleed };
    let mut d = BladeSettings::defaults().design();
    if let Some(cb) = s.customer_bleed() { d = d.with_customer_bleed(cb); }
    if let Some(tc) = s.turbine_cooling() { d = d.with_turbine_cooling(tc); }
    if let Some(so) = s.shaft_offtake() { d = d.with_shaft_offtake(so); }
    match builder(d, s.design_flight(), MDOT_DESIGN, Some(ml), Some(mh), s.rho, &s.lever_arm()) {
        ScheduledStatorTransient::Full(c) => c,
        _ => unreachable!(),
    }
}

fn eq(s: &ControlsSettings, tt4: f64, start: Option<(f64, f64)>) -> Result<(f64, f64, f64), String> {
    let c = core_for(s);
    c.fuel.inner.try_equilibrium(&s.flight(), tt4, start)
        .map(|(i, _, _)| (i.nu_lp, i.nu_hp, i.sp_thrust_inlet.unwrap_or(i.sp_thrust)))
        .map_err(|e| e.0)
}

fn short(e: &str) -> String { e.chars().take(150).collect() }

fn main() {
    let text = std::fs::read_to_string("W:/temp/claude/idle-rule/offmap_requests.txt").unwrap();
    for (k, line) in text.lines().enumerate() {
        let req = Json::parse(line);
        let s = ControlsSettings::from_json(req.get("controls").unwrap()).unwrap();
        println!("\n===== request {k}: shape {} from {:.1} to {:.1} lever {:?} customer {:?} cool {:.3}/{:.3} gen {:?} T0 {:.1} p0 {:.0} M {:.3}",
                 CONTROL_SHAPES[s.shape], s.from, s.to, s.lever, s.customer, s.cool_vane, s.cool_blade, s.generator, s.t0, s.p0, s.m0);
        let reply = call(line);
        println!("page now: {}", short(&reply));
        let tt4 = if eq(&s, s.from, None).is_err() { s.from } else { s.to };
        println!("failing end: {tt4:.1} K -> {}", eq(&s, tt4, None).map(|r| format!("{r:?}")).unwrap_or_else(|e| short(&e)));

        // 1. Newton from a 2-D start grid.
        let mut hits = Vec::new();
        for il in 0..9 { for ih in 0..6 {
            let st = (0.2 + 0.1 * il as f64, 0.5 + 0.1 * ih as f64);
            if let Ok(r) = eq(&s, tt4, Some(st)) { hits.push((st, r)); }
        }}
        println!("2-D starts that solve: {} of 54", hits.len());
        for (st, r) in hits.iter().take(6) { println!("  start {st:?} -> nu=({:.4},{:.4}) thrust/inlet {:.2}", r.0, r.1, r.2); }

        // 2. Residual sign map over (nu_lp, nu_hp): rows nu_hp 1.00 .. 0.40, columns nu_lp 0.05 .. 1.00.
        let c = core_for(&s);
        let t = &c.fuel.inner;
        let fl = s.flight();
        let (tt2, pt2, _) = t.inlet(&fl);
        println!("residual signs (LP power, HP power): A ++  B +-  C -+  D --  x close fails");
        let mut best = (f64::INFINITY, 0.0, 0.0);
        for ih in 0..=30 {
            let nh = 1.0 - 0.02 * ih as f64;
            let mut row = String::new();
            for il in 1..=20 {
                let nl = 0.05 * il as f64;
                let ch = match t.try_close(nl, nh, tt4, tt2, pt2).and_then(|cl| t.powers(&cl, &fl, nl, nh, tt4)) {
                    Err(_) => 'x',
                    Ok((a, b)) => {
                        let r = a.abs().max(b.abs());
                        if r < best.0 { best = (r, nl, nh); }
                        match (a >= 0.0, b >= 0.0) { (true, true) => 'A', (true, false) => 'B', (false, true) => 'C', _ => 'D' }
                    }
                };
                row.push(ch);
            }
            println!("  nu_hp {nh:.2} {row}");
        }
        println!("smallest residual on the grid {:.3e} at nu=({:.2},{:.2})", best.0, best.1, best.2);

        // 3. The load swept down.
        if s.generator != GeneratorMode::Off {
            let full = s.generator_power;
            for j in 0..=10 {
                let mut s2 = s.clone();
                s2.generator_power = full * j as f64 / 10.0;
                if j == 0 { s2.generator = GeneratorMode::Off; }
                let r = eq(&s2, tt4, None);
                println!("  generator power {:.4}: {}", s2.generator_power * (j > 0) as i32 as f64,
                         r.map(|r| format!("runs nu=({:.4},{:.4}) thrust/inlet {:.2}", r.0, r.1, r.2)).unwrap_or_else(|e| format!("refused: {}", short(&e))));
            }
        }
        if s.customer != CustomerMode::Off {
            let full = s.customer_share;
            for j in 0..=10 {
                let mut s2 = s.clone();
                s2.customer_share = full * j as f64 / 10.0;
                if j == 0 { s2.customer = CustomerMode::Off; }
                let r = eq(&s2, tt4, None);
                println!("  customer share {:.4}: {}", if j == 0 { 0.0 } else { s2.customer_share },
                         r.map(|r| format!("runs nu=({:.4},{:.4}) thrust/inlet {:.2}", r.0, r.1, r.2)).unwrap_or_else(|e| format!("refused: {}", short(&e))));
            }
            // The steady matcher (shares no closure code), when no airflow lever is on.
            let (ml, mh) = control_maps(CONTROL_SHAPES[s.shape]).unwrap();
            let d = BladeSettings::defaults().design().with_customer_bleed(CustomerBleed::share(s.customer_share));
            let m = CustomerBleedMatcher::new(d, s.design_flight(), MDOT_DESIGN, ml, mh, 0.0);
            m.core.base.below_choke.set(true);
            println!("  steady customer-bleed matcher (lever ignored): {}",
                     m.try_match_point(&fl, tt4).map(|r| format!("runs nu=({:.4},{:.4}) thrust {:.2}", r.base.n_lp_ratio, r.base.n_hp_ratio, r.base.base.thrust))
                         .unwrap_or_else(|e| format!("refused: {}", short(&e.0))));
        }
    }
}
