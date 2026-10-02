# PHASE 8 SLICE AK (plan § 8.2). Run once, 2026-10-02; its paths are that run's scratch folder.
# Build rust/oracle/main_segments.tsv from the instrumented capture: the print_*/plot_* rows,
# plus a row for every gap (text main() prints itself), each gap NAMED here by hand -- an
# unnamed gap is an error, so nothing is tiled silently.
import sys
CAP = r"W:\temp\claude\phase8-ak\capture\segments.tsv"
OUT = r"W:\Claude_projects\jet engine\rust\oracle\main_segments.tsv"
GAPS = {(1129, 1207): "main:losses_cost"}
partial = len(sys.argv) > 1 and sys.argv[1] == "--partial"
rows = [l.rstrip("\n").split("\t") for l in open(CAP, encoding="utf-8") if not l.startswith("#")]
out, pos = [], 0
for name, s, e, _secs in rows:
    s, e = int(s), int(e)
    if s != pos:
        out.append((GAPS.pop((pos, s)), pos, s))
    out.append((name, s, e)); pos = e
assert not GAPS, f"declared gaps never met: {GAPS}"
with open(OUT, "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# name\tstart_byte\tend_byte -- main()'s steps in order, cut from the PyPy stdout golden\n")
    fh.write("# (rust/oracle/main_stdout.txt) by an instrumented PyPy run of main.py whose own stdout was\n")
    fh.write("# byte-identical to the golden; 'main:<what>' is text main() prints itself. See tests/cli_golden.rs.\n")
    for n, s, e in out: fh.write(f"{n}\t{s}\t{e}\n")
print(len(out), "rows, end", pos, "(partial)" if partial else "")
