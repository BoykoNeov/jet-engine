// Where a lit Fly-it instant spends its time (thermally-perfect gas, the opening design, Tt7 1900).
use std::time::Instant as Clock;
use turbojet::afterburner::{fuel_frozen, rayleigh, try_afterburn, AbMode, Afterburner, FlyAfterburner};
use turbojet::components::try_choked_mfp;
use turbojet::gas::FlowState;
use turbojet::sandbox::{fly_solver, FlySettings};

fn per(n: usize, mut f: impl FnMut()) -> f64 { let t = Clock::now(); for _ in 0..n { f(); } t.elapsed().as_secs_f64() / n as f64 * 1e6 }

fn main() {
    let s = FlySettings::defaults();
    let fl = s.flight();
    let cmap = s.map.map().with_phi_surge(s.phi_surge);
    let st = fly_solver(&s);
    let ab = Afterburner { tt7: 1900.0, ..Afterburner::default() };
    let fa = FlyAfterburner::capture(&st, ab);
    let g = st.inner.inner.gas().clone();
    let e = st.equilibrium(&fl, s.tt4, Some(&cmap));
    let pt4 = e.pt2 * e.pi_c * st.inner.inner.pi_b;
    let s4 = FlowState { tt: s.tt4, pt: pt4, mdot: 0.0, far: e.f };
    let s5 = FlowState { tt: e.tt5, pt: e.pi_t * pt4, mdot: e.mdot_air * (1.0 + e.f), far: e.f };
    let f7 = fuel_frozen(&g, e.tt5, e.f, 1900.0, 0.95).unwrap().unwrap();
    println!("h_t            {:8.3} us", per(200000, || { std::hint::black_box(g.h_t(std::hint::black_box(1234.0), e.f)); }));
    println!("try_t_from_h_t {:8.3} us", per(200000, || { std::hint::black_box(g.try_t_from_h_t(std::hint::black_box(1.2e6), e.f).ok()); }));
    println!("choked_mfp     {:8.3} us", per(20000, || { std::hint::black_box(try_choked_mfp(&g, std::hint::black_box(1900.0), f7).ok()); }));
    println!("fuel_frozen    {:8.3} us", per(20000, || { std::hint::black_box(fuel_frozen(&g, e.tt5, e.f, std::hint::black_box(1900.0), 0.95).ok()); }));
    println!("rayleigh       {:8.3} us", per(5000, || { std::hint::black_box(rayleigh(&g, e.f, &g, f7, e.tt5, std::hint::black_box(0.2), 1900.0, (1.0 + f7) / (1.0 + e.f)).ok()); }));
    println!("try_afterburn  {:8.3} us", per(5000, || { std::hint::black_box(try_afterburn(&g, &s4, &s5, e.mdot_air, &Afterburner { m6: std::hint::black_box(0.2), ..ab }, AbMode::Lit).ok()); }));
    println!("shipped instant{:8.3} us", per(500, || { std::hint::black_box(st.try_instant(&fl, std::hint::black_box(0.9), s.tt4, Some(&cmap)).ok()); }));
    println!("lit instant    {:8.3} us", per(200, || { std::hint::black_box(fa.try_instant(&fl, std::hint::black_box(0.9), s.tt4, &cmap, AbMode::Lit, fa.a8_dry).ok()); }));
    println!("unlit instant  {:8.3} us", per(200, || { std::hint::black_box(fa.try_instant(&fl, std::hint::black_box(0.9), s.tt4, &cmap, AbMode::Unlit, fa.a8_dry).ok()); }));
    println!("close_compr    {:8.3} us", per(500, || { std::hint::black_box(st.try_close_compressor(s.tt4, e.tt2, e.pt2, &cmap, std::hint::black_box(0.9)).ok()); }));
}
