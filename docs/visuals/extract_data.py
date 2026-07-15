# Extract model data for the interactive visuals page (docs/visuals/).
#
# Runs the actual turbojet package at the rung-1 design point (M0=0.85, pi_c=10,
# Tt4=1500 K) and dumps data.json for template.html (spliced by build.py).
# Sweep grids are REDUCED vs the production defaults where noted -- these are
# illustration curves (shape, not digits); the verification gates live in tests/.
#
#   python extract_data.py     (~10 min; the rung-23 dwell sweep dominates, then the rung-17 ladder)
#   python build.py            -> turbojet-visuals.html
import sys, json, math, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1]))          # repo root -> import turbojet

from turbojet.engine import FlightCondition, build_turbojet
from turbojet.gas import (
    Gas, JetMixing, Unmixedness, MixingPDF, PocketQuenchPDF, SpatialPDF,
    SpatialDwellPDF, PromptNO,
    _super_eq_o_multiplier, _quench_trajectory, _equilibrium_composition,
    _F_STOICH, _HF_FUEL_DEFAULT, _bell_interpolator, _beta_pdf_nodes_weights,
    _ideal_bell_ei, _spatial_segregation, _two_stream_ceiling, _Ru,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C, TT4 = 10.0, 1500.0
REAL_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
                   eta_t=0.90, eta_m=0.99, pi_n=0.98)
TAU = 3e-3
OUT = {}
T0 = time.time()


def mark(msg):
    print(f"[{time.time() - T0:7.1f}s] {msg}", flush=True)


def r(x, sig=6):
    if x is None:
        return None
    if x == 0:
        return 0.0
    return round(x, sig - 1 - int(math.floor(math.log10(abs(x)))))


# ---------------------------------------------------------------- 1. CPG cycle
mark("CPG cycle (ideal vs real) + T-s legs")
gas = Gas()
ideal = build_turbojet(gas, PI_C, TT4, FLIGHT.p0).run(FLIGHT, 1.0)
real = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, **REAL_LOSSES).run(FLIGHT, 1.0)


def cycle_points(result):
    Tref, pref = FLIGHT.T0, FLIGHT.p0

    def s(T, p):
        return gas.cp_c * math.log(T / Tref) - gas.R_c * math.log(p / pref)

    st = result.stations
    pts = [("0", FLIGHT.T0, FLIGHT.p0), ("2", st["2"].Tt, st["2"].pt),
           ("3", st["3"].Tt, st["3"].pt), ("4", st["4"].Tt, st["4"].pt),
           ("5", st["5"].Tt, st["5"].pt), ("9", result.T9, FLIGHT.p0)]
    return {l: (s(T, p), T) for l, T, p in pts}


def ts_legs(coords):
    # Work legs as straight polylines + isobar-shaped combustion / heat-rejection
    # curves, exactly as main.py's plot_ts_diagram draws them.
    out = []
    for leg in (["0", "2", "3"], ["4", "5", "9"]):
        out.append([[r(coords[l][0], 5), r(coords[l][1], 5)] for l in leg])
    cp = gas.cp_c
    for a, b in (("3", "4"), ("9", "0")):
        (sa, Ta), (sb, Tb) = coords[a], coords[b]
        residual = sb - (sa + cp * math.log(Tb / Ta))
        pts = []
        for i in range(60):
            T = Ta + (Tb - Ta) * i / 59
            pts.append([r(sa + cp * math.log(T / Ta) + residual * (T - Ta) / (Tb - Ta), 5),
                        r(T, 5)])
        out.append(pts)
    return out


def pack_run(result):
    st = result.stations
    perf = result.performance
    return dict(
        stations={l: dict(Tt=r(s.Tt), pt=r(s.pt), far=r(s.far)) for l, s in st.items()},
        V0=r(result.V0), V9=r(result.V9), M9=r(result.M9), T9=r(result.T9),
        specific_thrust=r(perf.specific_thrust), tsfc=r(perf.tsfc),
        eta_brayton=r(perf.eta_brayton), eta_thermal=r(perf.eta_thermal),
        eta_propulsive=r(perf.eta_propulsive), eta_overall=r(perf.eta_overall),
        points={l: [r(sv, 5), r(T, 5)] for l, (sv, T) in cycle_points(result).items()},
        legs=ts_legs(cycle_points(result)),
    )


OUT["design"] = dict(T0=FLIGHT.T0, p0=FLIGHT.p0, M0=FLIGHT.M0, pi_c=PI_C, Tt4=TT4,
                     losses=REAL_LOSSES, tau_ms=TAU * 1e3)
OUT["ideal"] = pack_run(ideal)
OUT["real"] = pack_run(real)

# ------------------------------------------------ 2. equilibrium design point
mark("equilibrium-gas design point (rung-6 cycle)")
eq = Gas.reacting_equilibrium()
er = build_turbojet(eq, PI_C, TT4, FLIGHT.p0, **REAL_LOSSES).run(FLIGHT, 1.0)
st3, st4, st5 = er.stations["3"], er.stations["4"], er.stations["5"]
Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
OUT["eq_design"] = dict(Tt3=r(Tt3), Tt4=r(Tt4), far=r(far), p=r(p),
                        phi_overall=r(far / _F_STOICH))

# --------------------------------------------------------- 3. NOx bell sweep
mark("rung-9/19 phi_p bell sweep")
phis = [0.6, 0.7, 0.8, 0.9, 0.95, 1.0, 1.05, 1.1, 1.2, 1.3,
        1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0]
bell_rows = []
for phi in phis:
    z = eq.zoned_nox(far, Tt3, Tt4, p, phi, tau=TAU)
    z2 = eq.zoned_nox(far, Tt3, Tt4, p, phi, tau=TAU,
                      super_eq_o=True, prompt=PromptNO())
    comp = _equilibrium_composition(z.far_primary, z.T_primary, p)
    nt = sum(comp.values())
    bell_rows.append(dict(
        phi=phi, T_p=r(z.T_primary), ei=r(z.ei_no), ppm_eq=r(z.primary.ppm_eq),
        ppm_kin=r(z.ppm_primary), T_mix=r(z.T_mix),
        xco=r(comp["CO"] / nt), xh2=r(comp["H2"] / nt),
        ei_lift=r(z2.ei_no), ei_prompt=r(z2.ei_no_prompt), m=r(z2.o_multiplier),
    ))
    mark(f"  phi={phi:.2f}  T_p={z.T_primary:.0f}  EI={z.ei_no:.3g}")
OUT["bell"] = bell_rows

# ------------------------------------------------------ 4. quench trajectory
mark("rung-10 quench trajectories + tau_q sweep")
quench = {}
for tag, phi in (("rich", 1.4), ("lean", 0.9)):
    z = eq.zoned_nox(far, Tt3, Tt4, p, phi, tau=TAU)
    comp_prim = _equilibrium_composition(z.far_primary, z.T_primary, p)
    tab = _quench_trajectory(comp_prim, z.T_primary, z.alpha, far, Tt3, p, ngrid=81)
    rows = []
    for i, row in enumerate(tab):
        b = i / (len(tab) - 1)
        conc = p / (_Ru * row["T"])
        rows.append(dict(beta=r(b, 4), T=r(row["T"], 5),
                         xnoe_ppm=r(1e6 * row["cNOe"] / conc, 4)))
    quench[tag] = dict(phi=phi, T_primary=r(z.T_primary), rows=rows)
tq_rows = []
for tq_ms in (0.3, 0.5, 1.0, 2.0, 3.0, 5.0):
    z = eq.zoned_nox(far, Tt3, Tt4, p, 1.4, tau=TAU, tau_q=tq_ms * 1e-3,
                     quench_ngrid=80, quench_nsteps=600)
    tq_rows.append(dict(tau_q_ms=tq_ms, ei=r(z.ei_no_quenched), T_peak=r(z.T_peak)))
    mark(f"  tau_q={tq_ms} ms -> EI={z.ei_no_quenched:.3g}")
z_ideal = eq.zoned_nox(far, Tt3, Tt4, p, 1.4, tau=TAU)
quench["tau_sweep"] = tq_rows
quench["ideal_ei"] = r(z_ideal.ei_no)
OUT["quench"] = quench

# ------------------------------------------------------------- 5. J-sweeps
mark("rung-11/12/13 J-sweeps (bulk / two-stream / beta-PDF)")
S, H = 0.0625, 0.10
um = Unmixedness(S=S)
pdfc = MixingPDF(S=S)
hf = _HF_FUEL_DEFAULT
mark("  building the ideal bell (n_bell=80) ...")
bell_f = _bell_interpolator(p, Tt3, hf, TAU, n_bell=80)
point_val = _ideal_bell_ei(far, p, Tt3, hf, TAU)


def pdf_ei(g):
    if g <= 1e-9:
        return point_val
    xibar = far / (1.0 + far)
    nodes, w = _beta_pdf_nodes_weights(xibar, g, n_quad=160)
    return sum(wi * bell_f(x) for wi, x in zip(w, nodes))


j_rows = []
for J in (4.0, 6.0, 9.0, 12.0, 16.0, 20.0, 25.0, 36.0, 49.0, 64.0,
          100.0, 144.0, 225.0, 400.0):
    m = JetMixing(J=J, C_e=0.20, shape_n=2.0)
    z = eq.zoned_nox(far, Tt3, Tt4, p, 1.5, tau=TAU, mixing=m, unmixedness=um,
                     quench_ngrid=60, quench_nsteps=400)
    g = pdfc.segregation(pdfc.C(JetMixing(J=J)))
    j_rows.append(dict(J=J, C=r(z.C_holdeman), tau_q_ms=r(z.tau_q * 1e3),
                       ei_bulk=r(z.ei_no_quenched), ei_unmixed=r(z.ei_no_unmixed),
                       w_core=r(z.w_core), g=r(g), ei_pdf=r(pdf_ei(g))))
    mark(f"  J={J:.0f}  C={z.C_holdeman:.2f}  bulk={z.ei_no_quenched:.3g}  "
         f"two-stream={z.ei_no_unmixed:.3g}  pdf={pdf_ei(g):.3g}")
OUT["jsweep"] = dict(S=S, H=H, C_opt=um.C_opt, J_opt=(um.C_opt * H / S) ** 2,
                     point_val=r(point_val), rows=j_rows)

# ------------------------------------------------------------ 6. m(T) curve
mark("rung-19 m(T) Westenberg multiplier")
OUT["m_of_T"] = [dict(T=T, m=r(_super_eq_o_multiplier(T)))
                 for T in range(1500, 2801, 50)]
pr = PromptNO()
OUT["prompt_shape"] = [dict(phi=r(0.6 + 0.02 * i, 4),
                            f=r(max(pr.f_correction(0.6 + 0.02 * i), 0.0)))
                       for i in range(0, 71)]

# --------------------------------------------------------- 7. rung-22 spatial
mark("rung-22 spatial collapse (3 geometries)")
kp = SpatialPDF().k_p
spatial = dict(k_p=kp, C_opt=1.0 / (4.0 * kp ** 2),
               g_ceiling=r(_two_stream_ceiling(far, 1.5)), curves=[])
js = [2.0, 3.0, 4.0, 5.5, 7.5, 10.0, 13.0, 16.0, 20.0, 25.0, 32.0,
      42.0, 56.0, 75.0, 100.0, 140.0, 200.0, 280.0, 400.0]
for Sg, Hg in ((0.0625, 0.10), (0.03125, 0.10), (0.0625, 0.05)):
    rows = []
    for J in js:
        g = _spatial_segregation(far, 1.5, Sg, Hg, J, ny=32, nz=32)
        rows.append(dict(J=J, C=r((Sg / Hg) * math.sqrt(J)), g=r(g)))
    spatial["curves"].append(dict(S=Sg, H=Hg, rows=rows))
    mark(f"  geometry S={Sg} H={Hg} done")
OUT["spatial"] = spatial

# --------------------------------------------------------- 8. rung-17 ladder
# The rung-17 panel's own design point: rich phi_p=1.5, J=225 (main.py
# print_exhaust_clamp_table), coarse quench grids -- SHAPE, not digits.
mark("rung-17 exhaust-NO clamp ladder (J=225, reduced grids)")
try:
    Tt9, pt9, p9 = st5.Tt, REAL_LOSSES["pi_n"] * st5.pt, FLIGHT.p0
    mix = JetMixing(J=225.0, C_e=0.20, shape_n=2.0)
    pq = PocketQuenchPDF(S=0.0625, n_bell=40, n_quad=120)
    ladder = {}
    for tag, seo in (("eq_o", False), ("super_eq_o", True)):
        c = eq.exhaust_no_clamp(far, Tt3, Tt4, p, Tt9, pt9, p9, 1.5,
                                mixing=mix, pocket_quench=pq, tau=TAU,
                                super_eq_o=seo, quench_ngrid=60, quench_nsteps=400)
        ladder[tag] = dict(a_mixed=r(c.a_mixed_out), a_bulk=r(c.a_bulk_quench),
                           a_pocket=r(c.a_pocket),
                           collapse=r(c.no_collapse_ratio))
        mark(f"  {tag}: a_mixed={c.a_mixed_out:.3g} a_bulk={c.a_bulk_quench:.3g} "
             f"a_pocket={c.a_pocket:.3g}")
    OUT["ladder"] = ladder
except Exception as e:  # ladder is a nice-to-have; never sink the whole run
    mark(f"  ladder FAILED ({e!r}) -- omitting")
    OUT["ladder"] = None

# --------------------------------------------------- 9. rung-23 dwell spectrum
# Rung 22 derived the cross-plane WIDTH but imported rung-16's SCALAR dwell. Rung 23
# develops that same cross-plane in TIME, so each pocket carries its OWN tau(xi) --
# rich pockets are reached last, so they dwell longest. The MATCHED-MEAN twin (a scalar
# <tau>_PDF: same g, same mean dwell) isolates ONLY the xi-tau correlation, so
# corr_ratio = term2_corr/term2_meanfield > 1 means the correlation ADDS NO.
# Mirrors main.py print_dwell_spectrum_table, EXCEPT n_quad: main.py's 56 nodes trip the
# beta-PDF mean-preservation assert at J=36 (main.py's own grid steps 16 -> 64, over it).
# Not a J-monotone effect -- g is NOT monotone in J: it COLLAPSES to g_min at C_opt (J=16,
# rung 22's headline) and climbs back out, and the quadrature is hardest to converge on that
# climb. 120 nodes hold every J here under 0.5% mean drift (56 -> ~0.5% and two ASSERTs).
# One rule for ALL points on purpose: the correlation signal is only ~5%, so switching
# quadrature between points would inject scatter comparable to the effect being plotted.
# ~60 s per point: n_quad barely moves the cost (56 -> 120 was ~free), because the pockets that
# dominate the bill are the few hot ones -- the cold/lean nodes leave the quench almost at once.
mark("rung-23 derived dwell spectrum (correlated vs matched-mean twin)")
dwell_cfg = SpatialDwellPDF(S=0.0625, ny=32, nz=32, nt=24, n_bell=40, n_quad=120)
dwell_rows = []
for J in (4.0, 9.0, 16.0, 36.0, 64.0, 144.0, 400.0):
    m = JetMixing(J=J, C_e=0.20, U_c=75.0, H=0.10)
    z = eq.zoned_nox(far, Tt3, Tt4, p, 1.5, tau=TAU, mixing=m,
                     spatial_dwell=dwell_cfg, quench_ngrid=24)
    dwell_rows.append(dict(
        J=J, C=r(dwell_cfg.C(m)), g=r(z.g_spatial_dwell),
        tau_mix_ms=r(m.tau_q * 1e3), tau_mean_ms=r(z.tau_mean_dwell * 1e3),
        ei_corr=r(z.ei_no_spatial_dwell), ei_mean=r(z.ei_no_spatial_dwell_meanfield),
        corr=r(z.corr_ratio), max_a=r(z.max_a_quench)))
    mark(f"  J={J:.0f}  C={dwell_cfg.C(m):.2f}  corr/mean={z.corr_ratio:.4f}  "
         f"max_a={z.max_a_quench:.3f}")
OUT["dwell"] = dict(S=dwell_cfg.S, k_p=dwell_cfg.k_p,
                    C_opt=1.0 / (4.0 * dwell_cfg.k_p ** 2), rows=dwell_rows)

with open(HERE / "data.json", "w") as fh:
    json.dump(OUT, fh)
mark("DONE -> data.json")
