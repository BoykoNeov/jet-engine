"""SLICE V step 3 — the injection harness.

For each named injection: patch `rust/src/stator_transient.rs`, re-run the PROBE (the
did-it-move column, measured BEFORE any "caught" number is believed), then run the four
ported suites and record which tests fail. Restores the file unconditionally.
"""
import io
import os
import re
import subprocess
import sys

ROOT = r"M:\claud_projects\jet engine"
SRC = os.path.join(ROOT, "rust", "src", "stator_transient.rs")
TMP = r"M:\claud_projects\temp\rust-phase7"
SUITES = ["rung57", "rung58", "rung59", "rung60"]

# (name, [(old, new), ...])
INJECTIONS = [
    ("I1_local_armed_core", [
        ("""    t.arm(nu_lp, nu_hp, tt2);
    (R40.try_close)(t, nu_lp, nu_hp, tt4, tt2, pt2)""",
         """    let (_sl, _sh) = (t.inner.map_lp(), t.inner.map_hp());
    t.arm(nu_lp, nu_hp, tt2);
    let _out = (R40.try_close)(t, nu_lp, nu_hp, tt4, tt2, pt2);
    t.inner.set_map_lp(_sl);
    t.inner.set_map_hp(_sh);
    _out"""),
        ("""    ft.inner.arm(nu_lp, nu_hp, tt2);
    (crate::fuel_transient::R43.try_close_fuel)(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2)""",
         """    let (_sl, _sh) = (ft.inner.inner.map_lp(), ft.inner.inner.map_hp());
    ft.inner.arm(nu_lp, nu_hp, tt2);
    let _out = (crate::fuel_transient::R43.try_close_fuel)(
        ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2);
    ft.inner.inner.set_map_lp(_sl);
    ft.inner.inner.set_map_hp(_sh);
    _out"""),
    ]),
    ("I2_hp_arm_dropped", [
        ("""    if let Some(s) = a.sched_hp {
        // See `ScheduledStatorCore`'s CONCESSIONS: this reads the HP SHAFT speed, not its""",
         """    if let Some(s) = None::<StatorSchedule>.or(if true { None } else { a.sched_hp }) {
        // See `ScheduledStatorCore`'s CONCESSIONS: this reads the HP SHAFT speed, not its"""),
    ]),
    ("I3_smooth_shape_cubed", [
        ("Shape::Smooth => x * x * (3.0 - 2.0 * x),", "Shape::Smooth => x * x * x,"),
    ]),
    ("I4_erosion_inverted", [
        ("erosion: if c_pw != 0.0 { 1.0 - c_net / c_pw } else { f64::NAN },",
         "erosion: if c_net != 0.0 { 1.0 - c_pw / c_net } else { f64::NAN },"),
    ]),
    ("I5_incidence_lever_sign", [
        ("        let d = t_c + v - self.m_lim;", "        let d = t_c - v - self.m_lim;"),
    ]),
    ("I6_arm_reads_the_wrong_shaft", [
        ("        let v = s.at(nu_lp * powp(t.inner.tt2_d / tt2, 0.5));",
         "        let v = s.at(nu_hp * powp(t.inner.tt2_d / tt2, 0.5));"),
    ]),
]


def run(cmd, out_path, timeout=1800):
    with io.open(out_path, "w", encoding="utf-8", errors="replace") as fh:
        pr = subprocess.run(cmd, cwd=os.path.join(ROOT, "rust"), stdout=fh,
                            stderr=subprocess.STDOUT, timeout=timeout)
    return pr.returncode


def read_probe(path):
    out = {}
    with io.open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if "\t" in line:
                k, v = line.rstrip("\n").split("\t", 1)
                try:
                    out[k] = float(v)
                except ValueError:
                    pass
    return out


def failed_tests(path):
    bad = []
    with io.open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            m = re.match(r"^test (\S+) \.\.\. FAILED", line)
            if m:
                bad.append(m.group(1))
    return bad


def main():
    original = io.open(SRC, encoding="utf-8").read()
    base = read_probe(os.path.join(TMP, "probe_base.txt"))
    assert base, "no baseline probe"
    rows = []
    try:
        only = sys.argv[1:] 
        for name, subs in INJECTIONS:
            if only and name not in only:
                continue
            text = original
            for old, new in subs:
                n = text.count(old)
                if n != 1:
                    print("!! %s: pattern matched %d times, SKIPPED" % (name, n))
                    text = None
                    break
                text = text.replace(old, new)
            if text is None:
                rows.append((name, "PATTERN-MISS", 0, 0.0, [], 0))
                continue
            io.open(SRC, "w", encoding="utf-8", newline="\n").write(text)

            pp = os.path.join(TMP, "probe_%s.txt" % name)
            rc = run(["cargo", "test", "--release", "--test", "slice_v_probe",
                      "--", "--nocapture"], pp)
            got = read_probe(pp)
            if rc != 0 and not got:
                rows.append((name, "PROBE-DIED", 0, 0.0, [], 0))
                print("!! %s: probe failed to build/run" % name)
                continue
            moved, worst = 0, 0.0
            for k, v in base.items():
                w = got.get(k)
                if w is None:
                    continue
                if w != v:
                    moved += 1
                    d = abs(w - v) / max(abs(v), 1e-300)
                    worst = max(worst, d)
            bad, ntests = [], 0
            for s in SUITES:
                sp = os.path.join(TMP, "inj_%s_%s.txt" % (name, s))
                run(["cargo", "test", "--release", "--test", s], sp)
                bad += failed_tests(sp)
            ntests = 59
            rows.append((name, "OK", moved, worst, bad, ntests))
            print("%-28s moved %3d/%d  worst %.3g  caught %d/59"
                  % (name, moved, len(base), worst, len(bad)))
            for t in bad:
                print("      FAIL %s" % t)
    finally:
        io.open(SRC, "w", encoding="utf-8", newline="\n").write(original)
        print("\nsource restored")

    with io.open(os.path.join(TMP, "injection_table.txt"), "w", encoding="utf-8",
                 newline="\n") as fh:
        fh.write("injection\tstatus\tmoved\tof\tworst_rel\tcaught\tof_tests\tfailing\n")
        for name, status, moved, worst, bad, n in rows:
            fh.write("%s\t%s\t%d\t%d\t%.6g\t%d\t%d\t%s\n"
                     % (name, status, moved, len(base), worst, len(bad), n, ";".join(bad)))
    print("wrote injection_table.txt")


if __name__ == "__main__":
    sys.exit(main())
