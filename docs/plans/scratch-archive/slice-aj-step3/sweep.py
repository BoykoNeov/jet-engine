"""Slice AJ step 3 injection sweep. Each injection: one exact replacement in threshold_law.rs,
`cargo test --release --test slice_aj_threshold --no-fail-fast`, per-test verdicts recorded,
the file restored byte for byte (also in `finally`)."""
import re
import subprocess
import sys

SRC = r"W:\Claude_projects\jet engine\rust\src\threshold_law.rs"
MAN = r"W:\Claude_projects\jet engine\rust\Cargo.toml"
OUT = r"W:\temp\claude\slice-aj-step3\sweep.out"

BISECT_TAIL_OLD = """    let (mut a, mut b) = (lo, hi);
    for _ in 0..n {
        let mid = 0.5 * (a + b);
        let s = threshold_law::scan(core, mid, kw);
        if !s.window_open {
            return Bisect::VoidMid { void: "V1: window closed mid-bisection".into(), lo: a, hi: b };
        }
        if key(&s) { b = mid } else { a = mid }
    }
    Bisect::Ok(BisectOk {
        lo: a, hi: b, mid: 0.5 * (a + b), width: b - a, at_hi: threshold_law::scan(core, b, kw),
    })"""
BISECT_TAIL_NEW = """    let (mut a, mut b) = (lo, hi);
    let mut at_b = s_hi.clone();
    for _ in 0..n {
        let mid = 0.5 * (a + b);
        let s = threshold_law::scan(core, mid, kw);
        if !s.window_open {
            return Bisect::VoidMid { void: "V1: window closed mid-bisection".into(), lo: a, hi: b };
        }
        if key(&s) { b = mid; at_b = s } else { a = mid }
    }
    Bisect::Ok(BisectOk {
        lo: a, hi: b, mid: 0.5 * (a + b), width: b - a, at_hi: at_b,
    })"""
WINDOW_OLD = "    if !(s_lo.window_open && s_hi.window_open) {\n"
WINDOW_NEW = """    if key(&s_lo) || !key(&s_hi) {
        return Bisect::VoidStraddle {
            void: format!("V3: threshold not strictly inside [{}, {}]", py_g(lo), py_g(hi)),
            lo, hi, below: key(&s_lo), above: !key(&s_hi), at_lo: s_lo, at_hi: s_hi,
        };
    }
""" + WINDOW_OLD

INJ = [
    ("K1", "tau_hat_min: if hat_vals.is_empty() { None } else { Some(py_min_of(&hat_vals)) },",
     "tau_hat_min: arg.map(|a| a.hat),"),
    ("K2", "if arg.map_or(true, |a| k < a.hat - a.tau_eff) {",
     "if arg.map_or(true, |a| k <= a.hat - a.tau_eff) {"),
    ("K3", "let mut v = sorted_f(&cells.iter().map(|c| round6(c.tau_f / tau_f)).collect::<Vec<_>>());",
     "let mut v = sorted_f(&hs.iter().map(|c| round6(c.tau_eff / tau_f)).collect::<Vec<_>>());"),
    ("K4", BISECT_TAIL_OLD, BISECT_TAIL_NEW),
    ("K5", 'format!("V3: threshold not strictly inside [{}, {}]", py_g(lo), py_g(hi))',
     'format!("V3: threshold not strictly inside [{}, {}]", lo, hi)'),
    ("K6", 'gap_falls: a.gap_bind.is_some_and(|ga| need(b.gap_bind, "gap_bind") < ga),',
     'gap_falls: truthy(a.gap_bind) && need(b.gap_bind, "gap_bind") < a.gap_bind.unwrap(),'),
    ("K7", "sorted_f(phi_lims).iter().map(|&pl| at(pl, 0.05))",
     "phi_lims.iter().map(|&pl| at(pl, 0.05))"),
    ("K8", "phi_lims: phi_lims.to_vec(),", "phi_lims: sorted_f(phi_lims),"),
    ("K9", "meas.void().or(fix.void())", "fix.void().or(meas.void())"),
    ("K10", "all_kappa_pure: rows.iter().all(ThresholdRow::kappa_pure),",
     "all_kappa_pure: live.iter().all(|x| x.kappa_pure),"),
    ("K11", "ds_stable: ds_ok,", "ds_stable: ds_ok.or(Some(true)),"),
    ("K12", "let fix = bisect(core, key_residual, lo, hi, n_bisect, &kw);",
     "let fix = if meas.void().is_some() { meas.clone() } else { bisect(core, key_residual, lo, hi, n_bisect, &kw) };"),
    ("K13", '(true, Some(t)) => Some(t / kap.expect("truthy")),', "(true, Some(t)) => Some(t),"),
    ("K14", ".find(|c| at_bind(c) && c.slope_r != 0.0)", ".find(|c| at_bind(c))"),
    ("K15", "xs.sort_by(|a, b| a.dist.partial_cmp(&b.dist)", "xs.sort_by(|a, b| a.err.partial_cmp(&b.err)"),
    ("K16", WINDOW_OLD, WINDOW_NEW),
    ("K17", 'return Bisect::VoidMid { void: "V1: window closed mid-bisection".into(), lo: a, hi: b };',
     'return Bisect::VoidMid { void: "V1: window closed mid-bisection".into(), lo, hi };'),
    ("K18", "let pred = frozen_coef(b);", "let pred = frozen_coef(a);"),
    ("K19", 'let dr = frac_move(a.slope_r, b.slope_r, "slope_r");',
     'let dr = a.slope_r.map(|x| (need(b.slope_r, "slope_r") - x).abs() / x.abs());'),
    ("K20", "sorted_f(phi_lims).iter().map(|&pl| at(pl, 0.05))",
     "sorted_f(phi_lims).iter().map(|&pl| at(pl, tau_govs[0]))"),
    ("K21", ".filter(|&&i| 0 < i && i < traj.len() - 1)", ".filter(|&&i| i < traj.len())"),
    ("K22", "n_fuel: count(Authority::Fuel),",
     "n_fuel: cells.iter().filter(|c| c.measured == Authority::Fuel).count(),"),
]

only = set(sys.argv[1:])
orig = open(SRC, "rb").read()
text = orig.decode("utf-8")
for name, old, _ in INJ:
    assert text.count(old) == 1, (name, text.count(old))
res = open(OUT, "a", encoding="utf-8")
try:
    for name, old, new in INJ:
        if only and name not in only:
            continue
        open(SRC, "wb").write(text.replace(old, new).encode("utf-8"))
        p = subprocess.run(["cargo", "test", "--release", "--manifest-path", MAN, "--test",
                            "slice_aj_threshold", "--no-fail-fast"],
                           capture_output=True, text=True, encoding="utf-8", errors="replace")
        log = p.stdout + p.stderr
        if "error[" in log or "could not compile" in log:
            verdict = "BUILD-FAILED"
            failed = []
        else:
            failed = sorted(re.findall(r"^test (\S+) \.\.\. FAILED", log, re.M))
            verdict = "KILLED" if failed else "SURVIVED"
        line = "%s %s %s" % (name, verdict, ",".join(failed))
        res.write(line + "\n")
        res.flush()
        print(line, flush=True)
        if verdict == "BUILD-FAILED":
            open(OUT + "." + name + ".log", "w", encoding="utf-8").write(log)
finally:
    open(SRC, "wb").write(orig)
    res.write("RESTORED\n")
    res.close()
assert open(SRC, "rb").read() == orig
print("restored", flush=True)
