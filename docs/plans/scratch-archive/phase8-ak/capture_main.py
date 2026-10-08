# One instrumented PyPy run of main.py. Records, WITHOUT changing what main.py computes:
#  (1) stdout as UTF-8 with LF line ends (must be byte-identical to the preflight golden);
#  (2) each TOP-LEVEL print_*/plot_* call's [start, end) byte offsets in that stdout;
#  (3) every array matplotlib's Axes.plot / Axes.scatter / annotate RECEIVES, as u64 bits.
import struct, sys, time
print(sys.version, file=sys.stderr, flush=True)
REPO = r"W:\Claude_projects\jet engine"
OUTDIR = r"W:\temp\claude\phase8-ak\capture"
sys.path.insert(0, REPO)
import main  # noqa: E402  (forces Agg; runs nothing under import)
import matplotlib.axes  # noqa: E402

out = open(OUTDIR + r"\stdout_lf.txt", "w", encoding="utf-8", newline="\n")
sys.stdout = out
seg = open(OUTDIR + r"\segments.tsv", "w", encoding="utf-8", newline="\n")
seg.write("# name\tstart_byte\tend_byte\tseconds\n")
ts = open(OUTDIR + r"\ts_calls.tsv", "w", encoding="utf-8", newline="\n")
ts.write("# call_index\tkind\tseries\tn\tbits...\n")

def bits(x):
    return f"{struct.unpack('<Q', struct.pack('<d', float(x)))[0]:016x}"

calls = [0]
def rec(kind, series, xs):
    xs = list(xs)
    ts.write(f"{calls[0]}\t{kind}\t{series}\t{len(xs)}\t" + "\t".join(bits(v) for v in xs) + "\n")

_plot, _scatter, _annot = matplotlib.axes.Axes.plot, matplotlib.axes.Axes.scatter, matplotlib.axes.Axes.annotate
def plot(self, *a, **k):
    if len(a) >= 2 and len(a[0]) > 0:
        rec("plot", "x", a[0]); rec("plot", "y", a[1])
        ts.write(f"{calls[0]}\tplot\tstyle\t0\t{k.get('color')}|{k.get('ls')}|{k.get('lw')}|{k.get('alpha')}\n")
        calls[0] += 1
    return _plot(self, *a, **k)
def scatter(self, x, y, *a, **k):
    rec("scatter", "x", x); rec("scatter", "y", y)
    ts.write(f"{calls[0]}\tscatter\tstyle\t0\t{k.get('color')}|{k.get('alpha')}\n")
    calls[0] += 1
    return _scatter(self, x, y, *a, **k)
def annotate(self, text, xy, *a, **k):
    rec("annotate", repr(text), xy)
    calls[0] += 1
    return _annot(self, text, xy, *a, **k)
matplotlib.axes.Axes.plot, matplotlib.axes.Axes.scatter, matplotlib.axes.Axes.annotate = plot, scatter, annotate

depth = [0]
def wrap(name, fn):
    def w(*a, **k):
        top = depth[0] == 0
        depth[0] += 1
        out.flush(); s = out.buffer.tell() if False else out.tell(); t0 = time.time()
        try:
            return fn(*a, **k)
        finally:
            depth[0] -= 1
            out.flush()
            if top:
                seg.write(f"{name}\t{s}\t{out.tell()}\t{time.time() - t0:.1f}\n"); seg.flush()
    return w
n = 0
for name in list(vars(main)):
    if name.startswith("print_") or name == "plot_ts_diagram":
        setattr(main, name, wrap(name, getattr(main, name))); n += 1
print(f"wrapped {n}", file=sys.stderr, flush=True)
import os
os.chdir(OUTDIR)
main.main()
out.flush(); seg.close(); ts.close()
print("DONE", file=sys.stderr, flush=True)
