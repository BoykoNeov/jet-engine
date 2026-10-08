"""Slice AJ step 2 injection sweep over rust/src/authority_clock.rs. Each injection: apply
(exact string, must match once), run slice_aj_clock with --no-fail-fast at BELOW-NORMAL priority,
record per-test verdicts, restore. The original is restored from memory and re-verified."""
import subprocess
import sys

ROOT = r"W:\Claude_projects\jet engine"
AC = ROOT + r"\rust\src\authority_clock.rs"
LOG = r"W:\temp\claude\slice-aj-step2\inject.log"

MASK_TAU = 'let tau_f = lag_of(lag.as_ref()).tau(key_of(p, "required_fuel"), key_of(p, "g_fuel"));'
INJ = {
    "J1": [('let (tau_f, lag_gap, slope_f, slope_r) = if coord == "demand" {',
            'let (tau_f, lag_gap, slope_f, slope_r) = if matches!(p.extra, PointExtra::Demand { .. }) {')],
    "J2a": [(MASK_TAU,
             'let tau_f = if coord == "demand" { demand_tau(lag_of(lag.as_ref()), '
             'key_of(p, "cap_fuel"), key_of(p, "w_fuel")) } else { lag_of(lag.as_ref())'
             '.tau(key_of(p, "required_fuel"), key_of(p, "g_fuel")) };')],
    "J2b": [('let tau_f = demand_tau(lag_of(lag), key_of(p, "cap_fuel"), key_of(p, "w_fuel"));',
             'let tau_f = lag_of(lag).tau(key_of(p, "required_fuel"), key_of(p, "g_fuel"));')],
    "J3": [("pts.iter().step_by(every).collect();", "pts.iter().step_by(1).collect();")],
    "J4": [("let alive: Vec<&MaskArm> = out.iter().filter(|a| a.riding4_valid).collect();",
            "let alive: Vec<&MaskArm> = out.iter().collect();")],
    "J5": [("(tau_f, tau_gov * sr - tau_f * sf, sf, sr)", "(tau_f, tau_f * sf - tau_gov * sr, sf, sr)")],
    "J6": [("for &i in &ride {\n                    tally(&mut census",
            "for &i in ride.iter().filter(|&&i| i != 0 && i != traj.len() - 1) {\n"
            "                    tally(&mut census")],
    "J7": [('core, flight, tt4_lo, tt4_hi, tt4_max, phi_lim, None, "clip",',
            'core, flight, tt4_lo, tt4_hi, tt4_max, phi_lim, phi_air, "clip",')],
    "J8": [("        side.sort_unstable();\n", "")],
    "J9": [('.max()\n                .expect("at least one later row");', ".sum::<usize>();")],
    "J10": [(".filter(|x| x.tau_f == x.tau_gov && x.tau_gov == 0.05)",
             ".filter(|x| x.tau_f == x.tau_gov)")],
    "J11": [("(m.triple_hooks().quad_gains_at)(&m,", "(core.triple_hooks().quad_gains_at)(&m,")],
    "J12": [("let rate = 1.0 / tt.0 + 1.0 / tt.1 + 1.0 / tt.2 + 1.0 / tt.3;",
             "let rate = 1.0 / tt.3 + 1.0 / tt.2 + 1.0 / tt.1 + 1.0 / tt.0;")],
    "J13": [(MASK_TAU,
             'let tau_f = lag_of(lag.as_ref()).tau(key_of(p, "g_fuel"), key_of(p, "required_fuel"));')],
    "J14": [("predicted: if gap < lag_gap {", "predicted: if gap <= lag_gap {")],
    "J15": [("if tf == 0.05 {\n            ccensus = cc;",
             "if tf == 0.05 && ccensus.is_empty() {\n            ccensus = cc;")],
}

orig = open(AC, "rb").read()
only = sys.argv[1:] or list(INJ)
out = open(LOG, "a", encoding="utf-8")
try:
    for name in only:
        s = orig.decode("utf-8")
        for old, new in INJ[name]:
            assert s.count(old) == 1, (name, old[:60], s.count(old))
            s = s.replace(old, new)
        open(AC, "wb").write(s.encode("utf-8"))
        r = subprocess.run(
            ["cargo", "test", "--release", "--manifest-path", ROOT + r"\rust\Cargo.toml",
             "--no-fail-fast", "--test", "slice_aj_clock"],
            capture_output=True, text=True, encoding="utf-8", errors="replace",
            creationflags=subprocess.BELOW_NORMAL_PRIORITY_CLASS)
        text = r.stdout + r.stderr
        if "error[" in text or "could not compile" in text:
            verdict, detail, msgs = "COMPILE-ERROR", [l for l in text.splitlines()
                                                      if l.startswith("error")][:3], []
        else:
            failed = [l.split()[1] for l in text.splitlines()
                      if l.startswith("test ") and l.rstrip().endswith("FAILED")]
            verdict = "KILLED" if failed else "SURVIVED"
            detail = failed
            msgs = [l.strip()[:200] for l in text.splitlines()
                    if "lines differ" in l or ": line " in l][:8]
        line = "%s %s %s\n    %s" % (name, verdict, detail, "\n    ".join(msgs))
        print(line, flush=True)
        out.write(line + "\n")
        out.flush()
        open(AC, "wb").write(orig)
finally:
    open(AC, "wb").write(orig)
    assert open(AC, "rb").read() == orig
    out.write("RESTORED-VERIFIED\n")
    out.close()
