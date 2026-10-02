# Generates rust/oracle/pyfmt_*.tsv: Python's own formatting of fixed value batteries, so the
# Rust shim can be gated against PYTHON after the Python is gone. Every input is written as its
# u64 bit pattern (floats) or literally (ints/strings/bools); the random draws are committed as
# data, not as a seed, because random.seed dies with the interpreter.
import ast, math, random, re, struct, sys
print(sys.version, file=sys.stderr)
OUT = sys.argv[1]
SRC = r"W:\Claude_projects\jet engine\main.py"
def bits(v): return f"{struct.unpack('<Q', struct.pack('<d', v))[0]:016x}"
def safe(s):
    assert "\t" not in s and "\n" not in s, s
    return s

# ---- float battery -----------------------------------------------------------------------
random.seed(20261002)
F = []
for n in range(1, 12):
    for k in range(1, 40, 2): F.append(k / 2 ** n)            # exact binary ties
F += [x + 0.5 for x in range(0, 20)] + [-(x + 0.5) for x in range(0, 6)]
F += [0.0, -0.0, math.inf, -math.inf, math.nan, -math.nan, 5e-324, -5e-324, 2.2250738585072014e-308,
      1.7976931348623157e308, 1e300, 1e-300, 1e16, 9999999999999998.0, 1e15, 1e17, 1e22, 1e-4,
      1e-5, 9.99999e-5, 0.000123456, 0.0001, 0.00009999999999999999, 123456789.0, 1234567.0,
      999999.5, 9999995.0, 99999.95, 9.9995, 9.99949999, 0.95, 0.05, 0.15, 0.25, 0.35, 2.675,
      1.005, 0.125, 0.375, 1.0, -1.0, 10.0, 100.0, 1e6, 1e-6, 0.1, 0.2, 0.3, 1 / 3, 2 / 3,
      math.pi, math.e, 1500.0, 250.0, 50000.0, 0.85, 12.5, -12.5, 0.0005, 0.00049999999999999999]
for _ in range(250):
    e = random.uniform(-13, 10)
    F.append(random.choice([1, -1]) * 10 ** e)
for _ in range(60):  # values near a rounding boundary at some precision
    p = random.randint(0, 8); m = random.randint(1, 10 ** 6)
    F.append(random.choice([1, -1]) * (m + 0.5) / 10 ** p)
I = [0, 1, -1, 2, 7, 9, 10, 42, -42, 99, 100, 123, -123, 999, 1000, 1366, 12345, -12345,
     10 ** 6, 2 ** 31, -2 ** 31, 2 ** 53, 10 ** 15, -10 ** 15]
S = ["", "a", "ab", "rho=0.2", "r=0.5", "inst", "shared", "ARRESTED", "marches", "Δ", "·≡≈",
     "phi_lp", "a very long label that is wider than every width", "x"]
B = [True, False]

# ---- spec census (the same AST walk as the census) -------------------------------------
t = ast.parse(open(SRC, encoding="utf-8").read())
specs = set()
for n in ast.walk(t):
    if isinstance(n, ast.FormattedValue) and n.format_spec is not None:
        specs.add("".join(v.value for v in n.format_spec.values))
pspecs = set()
for n in ast.walk(t):
    if isinstance(n, ast.BinOp) and isinstance(n.op, ast.Mod) and isinstance(n.left, ast.Constant) \
            and isinstance(n.left.value, str):
        for m in re.finditer(r"%([-+ #0]*)(\d*)(?:\.(\d+))?([sdfeg%r])", n.left.value):
            if m.group(4) != "%": pspecs.add(m.group(0))
# the same specs AT OTHER precisions/widths, so the shim is general and not tuned to these
for ty in "feg%":
    for p in (0, 2, 7): specs.add(f"+.{p}{ty}")
    specs.add(f"{ty}"); specs.add(f">12{ty}"); specs.add(f"<12{ty}"); specs.add(f"^13.3{ty}")
specs |= {"", ">9", "<9", "^9", "d", ">6d", "<6d", "+d", "s", ">7s"}
pspecs |= {"%f", "%e", "%g", "%.0g", "%.10g", "%10.3g", "%-10.3g", "%+d", "%5s", "%-5s", "%r"}

SPEC0 = 11 * 20 + 26
FS = F[:4] + F[220:SPEC0:9] + F[SPEC0:SPEC0 + 63] + F[-10:-5] + F[400:405]   # ties, half-ints, ALL specials, boundary, random
with open(OUT + "_battery.tsv", "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# kind\tvalue (float: u64 bits; int: decimal; str: literal; bool: True/False)\n")
    for v in F: fh.write(f"float\t{bits(v)}\n")
    for v in FS: fh.write(f"float_small\t{bits(v)}\n")
    for v in I: fh.write(f"int\t{v}\n")
    for v in S: fh.write(f"str\t{safe(v)}\n")
    for v in B: fh.write(f"bool\t{v}\n")

def row(fh, style, spec, kind, vals, fmt):
    outs = []
    for v in vals:
        try: outs.append(safe(fmt(v)))
        except (ValueError, TypeError, OverflowError): outs.append("\x01ERR")
    if all(o == "\x01ERR" for o in outs): return
    fh.write(f"{style}\t{spec}\t{kind}\t" + "\t".join(outs) + "\n")

with open(OUT + "_expect.tsv", "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# style\tspec\tkind\toutputs over that kind's battery, in battery order (\x01ERR = Python raised)\n")
    # DEEP: every float through the digit-producing core, at every precision 0..15.
    core = [f".{p}{ty}" for ty in "feg%" for p in (*range(9), 10, 12, 15)] + ["", "e", "g", "f", "%"]
    fh.write("# DEEP\n")
    row(fh, "str", "", "float", F, str)
    row(fh, "repr", "", "float", F, repr)
    for sp in core: row(fh, "new", sp, "float", F, lambda v, sp=sp: format(v, sp))
    # BROAD: every spec the census found (and its neighbours) on the SMALL float battery --
    # width / align / sign / '%'-style layout is independent of the digits.
    fh.write("# BROAD\n")
    for kind, vals in (("float_small", FS), ("int", I), ("str", S), ("bool", B)):
        row(fh, "str", "", kind, vals, str)
        row(fh, "repr", "", kind, vals, repr)
        for sp in sorted(specs):
            row(fh, "new", sp, kind, vals, lambda v, sp=sp: format(v, sp))
        for sp in sorted(pspecs):
            row(fh, "pct", sp, kind, vals, lambda v, sp=sp: sp % (v,))
print(len(F), len(I), len(S), len(specs), len(pspecs), file=sys.stderr)

# ---- CONTAINERS and None (slice AL pre-flight, added 2026-10-02) -------------------------
# str()/repr()/'%s' of the containers and of None the panels print. Each is named; the Rust
# test builds the SAME value under the same name, so the inputs live in both files by design.
CONTAINERS = {
    "none": None,
    "tuple_floats": (0.05, 0.2),
    "tuple_one": (0.05,),
    "tuple_empty": (),
    "tuple_ints": (3, 7, 12),
    "tuple_strs": ("demand", "applied"),
    "tuple_mixed": (1, 0.5, "x", True, None),
    "tuple_small_big": (1e-05, 1e16, -0.0, 2.5e-07),
    "list_floats": [0.1, 1e-05, 2.0, 1500.0],
    "list_strs": ["clip", "demand", "demand-latched"],
    "list_empty": [],
    "list_nested": [(0.2, 1.0), (5.0,)],
    "list_bools": [True, False],
    "dict_str_str": {"CO": "0.0123%", "OH": "1.5000%"},
    "dict_str_float": {"a": 0.25, "b": 1e-07},
}
with open(OUT + "_containers.tsv", "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# name\tstr(v)\trepr(v)\t'%s' % (v,)\tformat(v, '')\n")
    for name, v in CONTAINERS.items():
        fh.write("\t".join([name, safe(str(v)), safe(repr(v)), safe("%s" % (v,)), safe(format(v, ""))]) + "\n")
