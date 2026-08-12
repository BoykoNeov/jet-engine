"""THE ORACLE, phase 1 — dump every gas-property value the Rust port must reproduce.

Single-use by design (docs/plans/todo-rust-port.md): this exists to validate the Rust and
is deleted at phase 8. It reaches into `turbojet.gas`'s private names on purpose — it is
not an API consumer, it is a reference dump.

Output is TSV, one row per value:  key <TAB> u64-bits <TAB> repr
Bits rather than text because the comparison is about last digits; `f64::from_bits` on the
Rust side is exact and needs no JSON crate, keeping the engine dependency-free.

Run under BOTH interpreters. The project already ships on two (the gate runs PyPy, the
fingerprint goldens are CPython), so whatever PyPy and CPython disagree by is a deviation
the project ALREADY tolerates -- that gap is the principled tolerance floor for the port,
rather than a number picked out of the air.

    C:\\Python314\\python.exe rust/oracle/dump_gas.py rust/oracle/gas_cpython.tsv
    .venv\\Scripts\\python.exe  rust/oracle/dump_gas.py rust/oracle/gas_pypy.tsv
"""
import os
import struct
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet import gas as G


ROWS = []


def put(key, value):
    """Record one float. Rejects non-finite so a NaN cannot slip through as 'equal'."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    bits = struct.unpack("<Q", struct.pack("<d", v))[0]
    ROWS.append((key, bits, repr(v)))


# --- the grids ------------------------------------------------------------------------
# Straddle the 1000 K polynomial join deliberately, and sit exactly ON it: the piecewise
# h/phi are continuous by construction, so the join is where a transcription slip hides.
T_GRID = [200.0, 288.15, 298.15, 500.0, 800.0, 999.0, 999.9999, 1000.0, 1000.0001,
          1001.0, 1200.0, 1500.0, 1800.0, 2200.0, 2600.0, 3000.0]
# Lean only -- rich trips _products_composition's own guard (rung-4 scope).
FAR_GRID = [0.0, 0.005, 0.01, 0.02, 0.025, 0.03, 0.04, 0.05, 0.06, 0.065]


# --- module constants -----------------------------------------------------------------
put("const/Ru", G._Ru)
put("const/T_break", G._T_BREAK)
put("const/T_ref", G._T_REF)
put("const/p_ref", G._P_REF)
put("const/M_CH2", G._M_CH2)
put("const/M_air", G._M_AIR)
put("const/f_stoich", G._F_STOICH)
put("const/hf_fuel_default", G._HF_FUEL_DEFAULT)
put("const/lhv_default", G._lhv_from_fuel(G._HF_FUEL_DEFAULT))

for s, x in sorted(G._air_mole_fractions().items()):
    put(f"air_x/{s}", x)

# --- the two frozen mixtures (rung 3) -------------------------------------------------
for name, frac in (("air", G._AIR), ("products", G._PRODUCTS)):
    A_low, A_high, R = G._mixture(frac)
    put(f"mixture/{name}/R", R)
    for k in range(5):
        put(f"mixture/{name}/A_low/{k}", A_low[k])
        put(f"mixture/{name}/A_high/{k}", A_high[k])

# --- reacting composition + mixture, per f (rung 4) ------------------------------------
for f in FAR_GRID:
    comp = G._products_composition(f)
    for s in sorted(comp):
        put(f"comp/{f!r}/{s}", comp[s])
    A_low, A_high, R = G._mixture(comp)
    put(f"react_mix/{f!r}/R", R)
    for k in range(5):
        put(f"react_mix/{f!r}/A_low/{k}", A_low[k])
        put(f"react_mix/{f!r}/A_high/{k}", A_high[k])
    put(f"hf_prod/{f!r}", G._formation_products_mass(f))


def dump_section(tag, sec, far=0.0):
    """Every public property of a section, plus BOTH inverses round-tripped.

    The inverses are fed h(T) and pr(T) from the same section, so a mismatch localises:
    if the forward value agrees and the inverse does not, the safeguarded Newton is the
    suspect, not the polynomial.
    """
    put(f"{tag}/R", sec.R_at(far))
    for T in T_GRID:
        put(f"{tag}/cp/{T!r}", sec.cp(T, far))
        h = sec.h(T, far)
        put(f"{tag}/h/{T!r}", h)
        pr = sec.pr(T, far)
        put(f"{tag}/pr/{T!r}", pr)
        put(f"{tag}/gamma/{T!r}", sec.gamma_at(T, far))
        put(f"{tag}/T_from_h/{T!r}", sec.T_from_h(h, far))
        put(f"{tag}/T_from_pr/{T!r}", sec.T_from_pr(pr, far))


# --- rungs 1-2: the calorically-perfect section ----------------------------------------
dump_section("cpg", G._CPGSection(1.4, 1004.0, 287.0))

# --- rung 3: the two frozen thermally-perfect sections ---------------------------------
Alo_c, Ahi_c, R_c = G._mixture(G._AIR)
dump_section("tpg_air", G._TPGSection((Alo_c, Ahi_c), R_c))
Alo_t, Ahi_t, R_t = G._mixture(G._PRODUCTS)
dump_section("tpg_prod", G._TPGSection((Alo_t, Ahi_t), R_t))

# --- rung 4: the reacting section, per f ------------------------------------------------
react = G._ReactingSection()
for f in FAR_GRID:
    dump_section(f"react/{f!r}", react, far=f)


# --- rung 6: the equilibrium substrate ------------------------------------------------
# a6/a7 first, then the four molar functions built on them, then lnKp. Ordered that way on
# purpose: if a6 is wrong, everything downstream is wrong, and this makes it obvious which.
EQ_SPECIES = ("CO2", "H2O", "CO", "H2", "OH", "O", "H", "O2", "N2", "Ar")
EQ_T = [800.0, 1000.0, 1500.0, 1800.0, 2200.0, 2600.0, 3000.0]

for s in EQ_SPECIES:
    put(f"a6/{s}", G._a6_of(s))
    put(f"a7/{s}", G._a7_of(s))
    for T in EQ_T:
        put(f"sens_h/{s}/{T!r}", G._sens_h(s, T))
        put(f"sens_phi/{s}/{T!r}", G._sens_phi(s, T))
        put(f"h_molar_A/{s}/{T!r}", G._h_molar_A(s, T))
        put(f"s_molar/{s}/{T!r}", G._s_molar(s, T))
        put(f"g_molar/{s}/{T!r}", G._g_molar(s, T))
        put(f"h_molar_B/{s}/{T!r}", G._h_molar_B(s, T))

for i, rxn in enumerate(G._REACTIONS):
    for T in EQ_T:
        put(f"lnKp/{i}/{T!r}", G._lnKp(rxn, T))

# The dense solve, on a fixed conditioned system. Exercises partial pivoting (row 0's
# leading entry is NOT the column max) without depending on the equilibrium Jacobian.
_A = [[1.0 / (i + j + 1) + (10.0 if i == j else 0.0) for j in range(8)] for i in range(8)]
_A[0][0] = 0.5                      # force a pivot swap on the first column
_b = [float(i + 1) for i in range(8)]
for i, xi in enumerate(G._gauss_solve(_A, _b)):
    put(f"gauss/{i}", xi)

# --- rung 6: the equilibrium composition ------------------------------------------------
EQ_FAR = [0.02, 0.025, 0.03]
EQ_BURN = [(1500.0, 101325.0), (1800.0, 1000000.0), (2200.0, 2500000.0),
           (2500.0, 2500000.0)]

for (Tb, pb) in EQ_BURN:
    for f in EQ_FAR:
        comp = G._equilibrium_composition(f, Tb, pb)
        for s in EQ_SPECIES:
            put(f"equilcomp/{Tb!r}/{pb!r}/{f!r}/{s}", comp[s])

# --- rung 6: the frozen equilibrium hot section ------------------------------------------
# One section per burn condition -- reusing one across two (Tt4, pt4) trips its own guard,
# which is the point of the guard.
for (Tb, pb) in EQ_BURN:
    sec = G._EquilibriumSection()
    for f in EQ_FAR:
        sec.freeze(f, Tb, pb)
    for f in EQ_FAR:
        dump_section(f"equil/{Tb!r}/{pb!r}/{f!r}", sec, far=f)


# --- write --------------------------------------------------------------------------
out = sys.argv[1] if len(sys.argv) > 1 else "gas_oracle.tsv"
seen = set()
for key, _, _ in ROWS:
    assert key not in seen, f"duplicate key {key} -- a later row would silently win"
    seen.add(key)
with open(out, "w") as fh:
    fh.write(f"# turbojet gas oracle -- {len(ROWS)} values\n")
    fh.write(f"# interpreter: {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")
print(f"{len(ROWS)} values -> {out}   [{sys.implementation.name} {sys.version.split()[0]}]")
