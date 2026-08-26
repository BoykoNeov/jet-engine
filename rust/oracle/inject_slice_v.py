"""SLICE V steps 3 and 4 — the injection harness.

**Step 3 mode (default).** For each named injection: patch `rust/src/stator_transient.rs`,
re-run the PROBE (the did-it-move column, measured BEFORE any "caught" number is believed),
then run the four ported suites and record which tests fail.

**Step 4 mode (`--oracle`).** The same injections against `slice_v_oracle.rs` instead, which
answers a question the step-3 table could not: the 59 ported gates are RELATIONAL and caught
0 of 6 on the two carrier injections, so *does the ORACLE see them?* Reports, per injection,
how many of the oracle's keys stop matching Python — read out of the gate's own panic line, so
the number is EMITTED rather than typed. It needs no probe and no `slice_v_probe.rs`.

**Step 5 mode (`--self`).** The injections above all patch `rust/src/`, i.e. they ask *does the
gate notice when the PORT is wrong?* They say nothing about the mutation the gate itself carries
-- the two wrapper cells in `slice_v_dispatch.rs`. A wrapper that restored only ONE map would be
a PARTIAL carrier bug in the instrument, and the four difference-asserting gates might still pass
at their pinned values. So `--self` patches the TEST FILE instead and reports the same table.
(Measured: both variants fire 4 of 6 -- the two spools are coupled through the shaft state, so an
HP-only defect is not confined to the HP armings.)

**Step 5 mode (`--dispatch`).** The same injections against `slice_v_dispatch.rs` — the
MANUFACTURED carrier gate. It answers the question neither of the two above can: the oracle
catches I1/I2 only by comparing against a committed golden, so *does a gate that manufactures
the bug in Rust, with no golden behind the assertion, flip?* Reports which of the six `#[test]`
fail, by name, read out of cargo's own `... FAILED` lines.

Restores the file unconditionally in all three modes.

    .venv\Scripts\python.exe rust\oracle\inject_slice_v.py --oracle
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


DISPATCH_SRC = os.path.join(ROOT, "rust", "tests", "slice_v_dispatch.rs")

# Mutations of THE GATE'S OWN manufactured bug, not of the port.
SELF_INJECTIONS = [
    ("S1_partial_carrier_both_wrappers", [
        ("""    let out = r57_try_close(t, nu_lp, nu_hp, tt4, tt2, pt2);
    t.inner.set_map_lp(sl);
    t.inner.set_map_hp(sh);
    out""",
         """    let out = r57_try_close(t, nu_lp, nu_hp, tt4, tt2, pt2);
    t.inner.set_map_lp(sl);
    let _ = sh;
    out"""),
        ("""    let out = r57_try_close_fuel(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2);
    ft.inner.inner.set_map_lp(sl);
    ft.inner.inner.set_map_hp(sh);
    out""",
         """    let out = r57_try_close_fuel(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2);
    ft.inner.inner.set_map_lp(sl);
    let _ = sh;
    out"""),
    ]),
    ("S2_partial_carrier_fuel_wrapper_only", [
        ("""    let out = r57_try_close_fuel(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2);
    ft.inner.inner.set_map_lp(sl);
    ft.inner.inner.set_map_hp(sh);
    out""",
         """    let out = r57_try_close_fuel(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2);
    ft.inner.inner.set_map_lp(sl);
    let _ = sh;
    out"""),
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


def ran_at_all(path):
    """Did the binary compile AND report a test result? `0 failing` off a run that never built
    is a green-looking zero, which is the exact shape this port keeps finding."""
    with io.open(path, encoding="utf-8", errors="replace") as fh:
        return "test result:" in fh.read()


def write_src(text):
    """One writer for `stator_transient.rs`, with an explicit close. A dangling
    `io.open(...).write(...)` leaves the write unflushed under PyPy, which is this project's
    own recorded Windows-tooling hazard and would leave an INJECTED source on disk."""
    with io.open(SRC, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)


def oracle_keys_moved(path):
    """`N of M compared keys differ` / `N CPG float keys drifted`, straight out of the gate's
    own panic text. Parsed rather than recomputed: an instrument that re-derives the thing it
    measures is measuring itself (slice R's rule)."""
    n_diff, n_seen, arms = 0, 0, []
    with io.open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            m = re.search(r"^(\d+) of (\d+) compared keys differ", line)
            if m:
                n_diff += int(m.group(1))
                n_seen = max(n_seen, int(m.group(2)))
            m = re.match(r"^test (\S+) \.\.\. FAILED", line)
            if m:
                arms.append(m.group(1))
    return n_diff, n_seen, arms


def main_oracle():
    """Step 4's question: which injections does the ORACLE catch that the 59 gates do not?"""
    original = io.open(SRC, encoding="utf-8").read()
    rows = []
    try:
        only = sys.argv[1:]
        only = [a for a in only if a != "--oracle"]
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
                rows.append((name, "PATTERN-MISS", 0, 0, []))
                continue
            write_src(text)
            op = os.path.join(TMP, "oracle_%s.txt" % name)
            run(["cargo", "test", "--release", "--test", "slice_v_oracle"], op)
            n_diff, n_seen, arms = oracle_keys_moved(op)
            # `0 keys differing` is AMBIGUOUS on its own -- it reads the same whether nothing
            # moved or the run died before the comparison ran. I5 does the latter (an empty
            # trajectory panics inside `refine_min`). Slice S step 3's lesson: an injection
            # reporting "nothing moved" must be distinguishable from one that could not move
            # anything. So the two cases get DIFFERENT statuses.
            if arms and n_seen == 0:
                status = "PANIC-BEFORE-COMPARE"
            elif arms:
                status = "CAUGHT"
            else:
                status = "MISSED"
            rows.append((name, status, n_diff, n_seen, arms))
            print("%-28s %-21s keys differing %5d / %d   failing gates: %s"
                  % (name, status, n_diff, n_seen, ", ".join(arms) or "NONE"))
    finally:
        write_src(original)
        print("\nsource restored")
    out = os.path.join(TMP, "oracle_injection_table.txt")
    with io.open(out, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\t".join(["injection", "status", "keys_differing", "keys_compared",
                            "failing_gates"]) + "\n")
        for name, status, n_diff, n_seen, arms in rows:
            fh.write("\t".join([name, status, str(n_diff), str(n_seen),
                                ";".join(arms)]) + "\n")
    print("wrote oracle_injection_table.txt")


def write_dispatch_src(text):
    """One writer for the GATE file, and it keeps a full copy first.

    `io.open(path, "w")` TRUNCATES before it validates its own arguments -- a bad `newline=`
    emptied `slice_v_dispatch.rs` to 0 bytes here, and the `finally` then hit the same error
    and could not put it back. So the restore path holds the text in memory AND on disk."""
    with io.open(DISPATCH_SRC, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)


def main_self():
    """Does the gate survive a PARTIAL version of its own manufactured bug?"""
    original = io.open(DISPATCH_SRC, encoding="utf-8").read()
    backup = os.path.join(TMP, "slice_v_dispatch.rs.bak")
    with io.open(backup, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(original)
    rows = []
    try:
        only = [a for a in sys.argv[1:] if a != "--self"]
        for name, subs in SELF_INJECTIONS:
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
                rows.append((name, "PATTERN-MISS", []))
                continue
            write_dispatch_src(text)
            op = os.path.join(TMP, "self_%s.txt" % name)
            run(["cargo", "test", "--release", "--test", "slice_v_dispatch"], op)
            bad = failed_tests(op)
            built = ran_at_all(op)
            status = "CAUGHT" if bad else ("BUILD-FAILED" if not built else "MISSED")
            rows.append((name, status, bad))
            print("%-36s %-13s %d/6 gates fail: %s"
                  % (name, status, len(bad), ", ".join(bad) or "NONE"))
    finally:
        write_dispatch_src(original)
        print("\ntest file restored")
    out = os.path.join(TMP, "self_injection_table.txt")
    with io.open(out, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\t".join(["injection", "status", "n_failing", "failing_gates"]) + "\n")
        for name, status, bad in rows:
            fh.write("\t".join([name, status, str(len(bad)), ";".join(bad)]) + "\n")
    print("wrote self_injection_table.txt")


def main_dispatch():
    """Step 5's question: does the MANUFACTURED gate flip? It has no golden behind its
    scoped-side assertions, so a `MISSED` here is a real statement about coverage rather than
    about a file on disk."""
    original = io.open(SRC, encoding="utf-8").read()
    rows = []
    try:
        only = [a for a in sys.argv[1:] if a != "--dispatch"]
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
                rows.append((name, "PATTERN-MISS", []))
                continue
            write_src(text)
            op = os.path.join(TMP, "dispatch_%s.txt" % name)
            run(["cargo", "test", "--release", "--test", "slice_v_dispatch"], op)
            bad = failed_tests(op)
            built = ran_at_all(op)
            # Same discipline as `--oracle`: a run that never compiled or never reached a test
            # is NOT "nothing moved". It gets its own status.
            status = "CAUGHT" if bad else ("BUILD-FAILED" if not built else "MISSED")
            rows.append((name, status, bad))
            print("%-28s %-13s %d/6 gates fail: %s"
                  % (name, status, len(bad), ", ".join(bad) or "NONE"))
    finally:
        write_src(original)
        print("\nsource restored")
    out = os.path.join(TMP, "dispatch_injection_table.txt")
    with io.open(out, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\t".join(["injection", "status", "n_failing", "failing_gates"]) + "\n")
        for name, status, bad in rows:
            fh.write("\t".join([name, status, str(len(bad)), ";".join(bad)]) + "\n")
    print("wrote dispatch_injection_table.txt")


def main():
    if "--self" in sys.argv:
        return main_self()
    if "--dispatch" in sys.argv:
        return main_dispatch()
    if "--oracle" in sys.argv:
        return main_oracle()
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
