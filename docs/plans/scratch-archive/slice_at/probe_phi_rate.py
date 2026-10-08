"""Measure what tests/test_phi_rate_limiter_negative.py's gates see, so the Rust port is checked
against MEASURED numbers (pre-registered), not the docstring's."""
import os, sys
ROOT = r"W:\Claude_projects\jet engine"
sys.path.insert(0, ROOT)
sys.path.insert(0, os.path.join(ROOT, "tests"))
import test_phi_rate_limiter_negative as T  # noqa: E402

m, traj, slope = T.rig.__wrapped__() if hasattr(T.rig, "__wrapped__") else T.rig()
print("len(traj)", len(traj), "slope", repr(slope))
for key in ("phi_lp", "phi_hp"):
    for s in T.SAMPLES:
        p = T._at(traj, s)
        print(f"== {key} s={s} at s={p['s']!r} nu_lp={p['nu_lp']!r} nu_hp={p['nu_hp']!r} mf={p['mf']!r}")
        lv = [m._instant_fuel(T.FLIGHT, p["nu_lp"], p["nu_hp"], p["mf"] * 0.97 ** k)[key] for k in range(5)]
        rt = [m.rate_of(T.FLIGHT, p["nu_lp"], p["nu_hp"], p["mf"] * 0.97 ** k, slope, key) for k in range(5)]
        print("  level", [repr(x) for x in lv])
        print("  rate ", [repr(x) for x in rt])
        full, cut = p["mf"], p["mf"] * 0.90
        dl = m._instant_fuel(T.FLIGHT, p["nu_lp"], p["nu_hp"], cut)[key] - m._instant_fuel(T.FLIGHT, p["nu_lp"], p["nu_hp"], full)[key]
        dr = m.rate_of(T.FLIGHT, p["nu_lp"], p["nu_hp"], cut, slope, key) - m.rate_of(T.FLIGHT, p["nu_lp"], p["nu_hp"], full, slope, key)
        print(f"  d_level={dl!r} d_rate={dr!r} ratio={abs(dr)/abs(dl)!r}")
        r0 = m.rate_of(T.FLIGHT, p["nu_lp"], p["nu_hp"], p["mf"], slope, key)
        target, w, seen, first_fail, worst = 0.5 * r0, p["mf"], 0, None, None
        for i in range(40):
            w *= 0.9
            try:
                r = m.rate_of(T.FLIGHT, p["nu_lp"], p["nu_hp"], w, slope, key)
            except AssertionError as e:
                if first_fail is None:
                    first_fail = (i + 1, str(e)[:120])
                continue
            seen += 1
            q = r / target
            worst = q if worst is None else min(worst, q)
        print(f"  r0={r0!r} seen={seen} first_fail={first_fail} min(rate/target)={worst!r}")
