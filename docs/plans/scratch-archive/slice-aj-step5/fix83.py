import sys
p = r"W:/Claude_projects/jet engine/rust/tests/rung83.rs"
s = open(p, encoding="utf-8", newline="").read()
a = s.index("/// Python's `_rig(design)`. `CorrectorLawTransient`")
b = s.index("/// `test_rung83.py`'s `_kw(r)`.")
new = '''/// Python's `_rig(design)`. `CorrectorLawTransient` there; a rung-80 core here (plan § 5.34 (ii)).
fn rig() -> ScheduledStatorCore {
    let sm = 0.80 / FLOOR - 1.0;
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    };
    let m = match build_split_wall_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    };
    let t = &m.fuel.inner;
    t.lag_coord.set("demand");
    t.ref_law.set("sched");
    t.windup_law.set("none");
    t.cap_law.set("solve");
    m
}

'''
s = s[:a] + new + s[b:]
s = s.replace("rig(), ", "&m, ").replace("use std::sync::OnceLock;\n\n", "")
s = s.replace("    let f = flight();\n", "    let (m, f) = (rig(), flight());\n")
open(p, "w", encoding="utf-8", newline="").write(s)
print(s.count("&m, "), s.count("let (m, f)"), s.count("rig()"))
