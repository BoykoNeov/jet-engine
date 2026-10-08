"""Slice AJ step 4's injection sweep. Predictions: predictions.md (written first).

Each injection is ONE exact-string edit (asserted unique) to a source file; the two step-4 gates
run `--no-fail-fast`; the original BYTES are restored in `finally`. Results -> sweep_results.md.
Usage: python sweep.py [ID ...]   (no IDs = all)
"""
import os
import re
import subprocess
import sys
import time

REPO = r"W:\Claude_projects\jet engine"
SRC = os.path.join(REPO, "rust", "src")
C = os.path.join(SRC, "corrector_law.rs")
S = os.path.join(SRC, "staircase_law.rs")
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "sweep_results.md")

INJ = [
    ("C1", C, "(Some(t / k), Some(t - k * tau_f))", "(Some(t / k), Some(t - tau_f * k))"),
    ("C2", C, "(Some(t / k), Some(t - k * tau_f))",
     "(Some(s.h.map_or(f64::NAN, |h| h / k + tau_f)), Some(t - k * tau_f))"),
    ("C3", C, "Some(f) if !((1.0 - c).abs() < 1e-12) => {",
     "Some(f) if !((1.0 - c).abs() <= 1e-12) => {"),
    ("C4", C, 'void: if read.f.is_none() { "V4: kappa impure" }',
     'void: if false { "V4: kappa impure" }'),
    ("C6", C, ".map(|i| corrector_read(core, lo + py_div((hi - lo) * i as f64, nm1), kw))",
     ".map(|i| corrector_read(core, lo + i as f64 * step, kw))"),
    ("C8", C, "jump_in_f: if truthy(a.f) {", "jump_in_f: if a.f.is_some() {"),
    ("C9", C, "n_argmin_switches: pts.windows(2).filter(|w| w[0].s_bind != w[1].s_bind).count(),",
     "n_argmin_switches: pts.windows(2).filter(|w| w[0].s_bind.is_some() && "
     "w[1].s_bind.is_some() && w[0].s_bind != w[1].s_bind).count(),"),
    ("C10", C, "clamps += cl as usize;", "clamps += 0 * cl as usize;"),
    ("C11", C, "(tn, cl) = (bhi, true);", "cl = true;"),
    ("C12", C, 'abort = Some(format!("S2: flat pair, |dg| < {}", py_g(flat)));',
     'abort = Some(format!("S2: flat pair, |dg| < {}", flat));'),
    ("C13", C, 'abort = Some("V4: kappa impure at a start point".into());\n            break;',
     'abort = Some("V4: kappa impure at a start point".into());\n            continue;'),
    ("C14", C, "converged: last.is_some_and(|x| x.g.abs() < 1e-12) && clamps == 0,",
     "converged: last.is_some_and(|x| x.g.abs() < 1e-12),"),
    ("C16", C, 'abort = Some("V4: kappa impure at an iterate".into());',
     'abort = Some("V4: kappa impure at a start point".into());'),
    ("S1", S, "n_scored: summ.len(),", "n_scored: cells.len(),"),
    ("S2", S, "let edge = ride.first().map(|&i| round9(traj[i].s));",
     "let edge = hs.first().map(|x| round9(x.s));"),
    ("S3", S, "(true, Some(h)) => Some(py_div(h, kap[0])),",
     "(true, Some(_)) => Some(py_div(py_min_of(&hs.iter().map(|x| x.hat).collect::<Vec<_>>()), "
     "kap[0]) - tau_f),"),
    ("S4", S, "dict_put(&mut summ, round9(x.s), x.hat - x.tau_eff);",
     "{ let k = round9(x.s); if !summ.iter().any(|&(q, _)| q == k) { "
     "summ.push((k, x.hat - x.tau_eff)); } }"),
    ("S5", S, "if sb.map_or(true, |(_, bv)| v < bv) {", "if sb.map_or(true, |(_, bv)| v <= bv) {"),
    ("S6", S, "edge_index: edge.map(|e| py_div(e, kw.ds).round_ties_even() as i64),",
     "edge_index: edge.map(|e| py_div(e, kw.ds).round() as i64),"),
    ("S8", S, "set_changed: !entered.is_empty() || !left.is_empty(),",
     "set_changed: !entered.is_empty(),"),
    ("S9", S, "kind: if !sign_full {", "kind: if false {"),
    ("S11", S, "(Some(a), Some(b)) => a >= b,", "(Some(a), Some(b)) => a > b,"),
    ("S12", S, 'Some(a.edge_index.expect("ok") - b.edge_index.expect("ok"))',
     'Some(b.edge_index.expect("ok") - a.edge_index.expect("ok"))'),
    ("S14", S, "let tread = if truthy(spacing) {", "let tread = if spacing.is_some() {"),
    ("S15", S, 'void: format!("V5: no edge move in [{}, {}]", py_g(tau_lo), py_g(tau_hi)),',
     'void: format!("V5: no edge move in [{}, {}]", tau_lo, tau_hi),'),
    ("S16", S, "if !(a.kappa_pure && b.kappa_pure) {",
     "if !(a.kappa_pure && b.kappa_pure) && c.d_membership.is_some() {"),
    ("S17", S, 'void: format!("V6: bracket narrower than {}", py_g(eps)),',
     'void: format!("V6: bracket narrower than {}", eps),'),
    ("S18", S, "root_exists: c.kind == Some(Kind::Crossing),", "root_exists: c.kind.is_some(),"),
    ("S19", S, "threshold_law::bisect(core, key_residual, bracket.0,",
     "threshold_law::bisect(core, threshold_law::key_fuel, bracket.0,"),
]

BELOW_NORMAL = 0x00004000
CMD = ["cargo", "test", "--release", "--no-fail-fast",
       "--manifest-path", os.path.join(REPO, "rust", "Cargo.toml"),
       "--test", "slice_aj_corrector", "--test", "slice_aj_staircase"]


def run():
    t = time.time()
    p = subprocess.run(CMD, capture_output=True, text=True, encoding="utf-8", errors="replace",
                       creationflags=BELOW_NORMAL)
    txt = p.stdout + "\n" + p.stderr
    if "error[" in txt or "could not compile" in txt:
        return None, None, txt, time.time() - t
    failed = sorted(set(re.findall(r"^test (\w+) \.\.\. FAILED", txt, re.M)))
    readings = sorted(set(re.findall(r"(\w+): \d+ of \d+ lines differ", txt)))
    return failed, readings, txt, time.time() - t


want = set(sys.argv[1:])
rows = []
for iid, path, old, new in INJ:
    if want and iid not in want:
        continue
    raw = open(path, "rb").read()
    text = raw.decode("utf-8")
    assert text.count(old) == 1, (iid, text.count(old))
    try:
        open(path, "wb").write(text.replace(old, new).encode("utf-8"))
        failed, readings, txt, dt = run()
    finally:
        open(path, "wb").write(raw)
    assert open(path, "rb").read() == raw
    if failed is None:
        verdict = "BUILD FAILED"
        with open(OUT + "." + iid + ".log", "w", encoding="utf-8") as fh:
            fh.write(txt)
    elif failed:
        verdict = "KILLED — " + ", ".join(readings) + "  [tests: " + ", ".join(failed) + "]"
    else:
        verdict = "SURVIVED"
    rows.append((iid, verdict))
    print("%-4s %6.0f s  %s" % (iid, dt, verdict), flush=True)
    with open(OUT, "a", encoding="utf-8") as fh:
        fh.write("| %s | %s |\n" % (iid, verdict))
