"""Slice AJ step 5 injection sweep. Predictions: predictions.md (written first).
Each injection: exact-once string replace in rust/src, cargo test the four new binaries
--no-fail-fast (below normal), record FAILED tests, restore the original bytes (finally)."""
import json, os, re, subprocess, sys, time
SRC = r"W:/Claude_projects/jet engine/rust/src"
RUST = r"W:/Claude_projects/jet engine/rust"
OUT = r"W:/temp/claude/slice-aj-step5/sweep"
os.makedirs(OUT, exist_ok=True)
INJ = [
    ("I1", "authority_clock.rs", "(tf, tg, tau_q, tau_s), r, s_settle", "(tg, tf, tau_q, tau_s), r, s_settle"),
    ("I2", "threshold_law.rs", "(tau_f, kw.tau_gov, kw.tau_q, kw.tau_s)", "(tau_f, kw.tau_gov, kw.tau_s, kw.tau_q)"),
    ("I3", "authority_clock.rs", "(tf, tg, tau_q, tau_s), r, s_settle", "(tf, tg, tau_s, tau_q), r, s_settle"),
    ("I4", "threshold_law.rs", "        tau_gov: kw.tau_gov,\n", "        tau_gov: kw.tau_q,\n"),
    ("I5", "corrector_law.rs", 'abort = Some("V4: kappa impure at a start point".into());\n            break;',
     'abort = Some("V4: kappa impure at a start point".into());\n            continue;'),
    ("I6", "corrector_law.rs", 'if read.f.is_none() { "V4: kappa impure" }', 'if false { "V4: kappa impure" }'),
    ("I7", "corrector_law.rs", 'format!("S2: flat pair, |dg| < {}", py_g(flat))', 'format!("S2: flat pair, |dg| < {}", flat)'),
    ("I8", "threshold_law.rs", '"V3: threshold not strictly inside [{}, {}]", py_g(lo), py_g(hi))',
     '"V3: threshold not strictly inside [{}, {}]", lo, hi)'),
    ("I9", "staircase_law.rs", '"V5: no edge move in [{}, {}]", py_g(tau_lo), py_g(tau_hi))',
     '"V5: no edge move in [{}, {}]", tau_lo, tau_hi)'),
    ("I10", "staircase_law.rs", '"V6: bracket narrower than {}", py_g(eps))', '"V6: bracket narrower than {}", eps)'),
    ("I11", "staircase_law.rs", '"V3: an edge off the march grid".into()', '"V3: no edge".into()'),
    ("I12", "staircase_law.rs", "    if !(a.kappa_pure && b.kappa_pure) {\n        return StaircaseNumber::V2",
     "    if !(a.kappa_pure && b.kappa_pure) && c.edge_moved {\n        return StaircaseNumber::V2"),
    ("I13", "threshold_law.rs",
     """    if !(s_lo.window_open && s_hi.window_open) {
        return Bisect::VoidEnd {
            void: "V1: four-loop window empty at a bracket end".into(),
            lo, hi, at_lo: s_lo, at_hi: s_hi,
        };
    }
    if key(&s_lo) || !key(&s_hi) {
        return Bisect::VoidStraddle {
            void: format!("V3: threshold not strictly inside [{}, {}]", py_g(lo), py_g(hi)),
            lo, hi, below: key(&s_lo), above: !key(&s_hi), at_lo: s_lo, at_hi: s_hi,
        };
    }
""",
     """    if key(&s_lo) || !key(&s_hi) {
        return Bisect::VoidStraddle {
            void: format!("V3: threshold not strictly inside [{}, {}]", py_g(lo), py_g(hi)),
            lo, hi, below: key(&s_lo), above: !key(&s_hi), at_lo: s_lo, at_hi: s_hi,
        };
    }
    if !(s_lo.window_open && s_hi.window_open) {
        return Bisect::VoidEnd {
            void: "V1: four-loop window empty at a bracket end".into(),
            lo, hi, at_lo: s_lo, at_hi: s_hi,
        };
    }
"""),
]
only = sys.argv[1:] or None
env = dict(os.environ, CARGO_TARGET_DIR=r"W:\temp\claude\slice-aj-step5\target")
BELOW = 0x00004000
res = {}
for tag, fn, old, new in INJ:
    if only and tag not in only:
        continue
    path = os.path.join(SRC, fn)
    orig = open(path, "rb").read()
    text = orig.decode("utf-8")
    n = text.count(old)
    assert n == 1, (tag, fn, n)
    try:
        open(path, "wb").write(text.replace(old, new).encode("utf-8"))
        t0 = time.time()
        p = subprocess.run(["cargo", "test", "--release", "--no-fail-fast", "--test", "rung81",
                            "--test", "rung82", "--test", "rung83", "--test", "rung84"],
                           cwd=RUST, env=env, capture_output=True, text=True,
                           encoding="utf-8", errors="replace", creationflags=BELOW)
    finally:
        open(path, "wb").write(orig)
    log = p.stdout + "\n==== STDERR ====\n" + p.stderr
    open(os.path.join(OUT, tag + ".log"), "w", encoding="utf-8").write(log)
    # attribute each FAILED test to its binary by the preceding "Running tests\\rungNN.rs"
    failed, cur = [], None
    for line in (p.stderr + p.stdout).splitlines():
        pass
    # stdout carries the test lines in binary order; stderr the "Running" lines -> use result blocks
    blocks = re.split(r"\nrunning \d+ tests?\n", "\n" + p.stdout)[1:]
    names = re.findall(r"Running tests\\(rung8\d)\.rs", p.stderr)
    for name, blk in zip(names, blocks):
        for m in re.finditer(r"^test (\S+) \.\.\. FAILED", blk, re.M):
            failed.append(name + "::" + m.group(1))
    built = "error[" not in p.stderr and "could not compile" not in p.stderr
    res[tag] = dict(built=built, failed=failed, n_blocks=len(blocks), seconds=round(time.time() - t0))
    print(tag, res[tag], flush=True)
    json.dump(res, open(os.path.join(OUT, "results.json"), "w"), indent=1)
print("DONE")
