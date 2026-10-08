// Item 3: which equilibrium failures does rung 33's real matcher hit, and at what (f, T, p)?
use std::panic::{catch_unwind, AssertUnwindSafe};
use turbojet::engine::{build_turbojet, FlightCondition};
use turbojet::gas::{Gas, EQFAILS};
use turbojet::matcher::OffDesignMatcher;
use turbojet::panels::marches::conv_losses;

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let design = FlightCondition::new(250.0, 50_000.0, 0.85);
    let m = OffDesignMatcher::new(build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, 50_000.0, conv_losses()), design, 1.0);
    println!("M0\tTt4\toutcome\tn_eq_fails\tmax_T_failing\tmin_b_c\tmax_b_c\tfloored\tfirst_panic_msg");
    for m0 in [0.0f64, 0.3, 0.5, 0.85, 1.2, 1.6] {
        let fl = FlightCondition::new(250.0, 50_000.0, m0);
        for tt4 in [800.0f64, 700.0, 650.0, 600.0, 560.0, 520.0, 500.0, 480.0, 440.0, 420.0] {
            EQFAILS.with(|c| c.borrow_mut().clear());
            let r = catch_unwind(AssertUnwindSafe(|| m.match_point(&fl, tt4)));
            let msg = match &r { Ok(_) => String::new(), Err(e) => e.downcast_ref::<String>().cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default() };
            let fails = EQFAILS.with(|c| c.borrow().clone());
            let maxt = fails.iter().fold(f64::NEG_INFINITY, |a, x| a.max(x.1));
            let minb = fails.iter().fold(f64::INFINITY, |a, x| a.min(x.0));
            let maxb = fails.iter().fold(0.0f64, |a, x| a.max(x.0));
            let mut sp: Vec<String> = fails.iter().map(|x| x.3.clone()).collect(); sp.sort(); sp.dedup();
            println!("{m0}\t{tt4}\t{}\t{}\t{maxt}\t{minb:.3e}\t{maxb:.3e}\t{}\t{}",
                     if r.is_ok() { "ok" } else { "PANIC" }, fails.len(), sp.join(","),
                     msg.chars().take(90).collect::<String>());
        }
    }
}
