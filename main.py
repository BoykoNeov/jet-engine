"""Run the cycle and draw the T-s diagram.

    python main.py

Rung 2 payoff: run the ideal turbojet AND a real-components version at the SAME
design point, print both station tables, and overlay them on one T-s diagram so
the "isentropic" legs visibly TILT RIGHT (entropy generated) once losses are on.

Requires the components in turbojet/ and matplotlib (`pip install -r requirements.txt`).
"""
import math

import matplotlib

matplotlib.use("Agg")  # headless: render to a file, never pop a window (no plt.show)
import matplotlib.pyplot as plt  # noqa: E402

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import Gas, _products_composition  # noqa: E402

TS_DIAGRAM_PATH = "ts_diagram.png"

# Design point (the rung-1 validation case) — shared by both runs below.
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
TT4 = 1500.0

# Real-components losses (single gas, fully expanded — so the only difference from
# the ideal run is the entropy each component generates, which is what tilts the
# legs). The dual-cp gas effect is a separate teaching point (NOTES.md) exercised
# by the Mattingly anchor in tests/test_rung2.py.
REAL_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
                   eta_t=0.90, eta_m=0.99, pi_n=0.98)


def print_station_table(title, result):
    """Print Tt, pt (and far) at every station so the numbers can be watched."""
    print(f"\n{title}")
    print(f"{'Station':>8} {'Tt [K]':>10} {'pt [kPa]':>10} {'far':>9}")
    print("-" * 40)
    for label, s in result.stations.items():
        print(f"{label:>8} {s.Tt:>10.1f} {s.pt / 1000:>10.2f} {s.far:>9.5f}")
    p = result.performance
    print(f"V0 = {result.V0:7.1f} m/s    V9 = {result.V9:7.1f} m/s    M9 = {result.M9:.3f}")
    print(f"Specific thrust = {p.specific_thrust:.1f} N·s/kg    TSFC = {p.tsfc:.3e} kg/(N·s)")
    print(f"eta_brayton = {p.eta_brayton:.4f}   eta_thermal = {p.eta_thermal:.4f}   "
          f"eta_p = {p.eta_propulsive:.4f}   eta_o = {p.eta_overall:.4f}")


def print_polytropic_table(gas, flight):
    """Rung-2b knob made visible: polytropic e vs the isentropic eta it IMPLIES.

    Feed the SAME per-stage quality e_c = e_t = 0.90 to both machines and the
    implied isentropic efficiencies straddle it — eta_c < e < eta_t — with the
    split widening as pressure ratio climbs (docs/rung2b-polytropic.md § The
    asymmetry: diverging isobars make a compressor look worse, a turbine better).
    The T-s diagram can't carry this (it is not a leg-tilt effect), so it rides
    here as a table — the "show the work" contract for the new knob.
    """
    e, gc = 0.9, gas.g_c
    print("\nPolytropic knob (rung 2b): implied isentropic eta at e_c = e_t = 0.90")
    print(f"{'pi_c':>6} {'eta_c':>8} {'e':>6} {'eta_t':>8}    (eta_c < e < eta_t)")
    print("-" * 48)
    for pi_c in (2.0, 10.0, 30.0):
        eta_c = (pi_c ** gc - 1.0) / (pi_c ** (gc / e) - 1.0)
        tau_t = build_turbojet(gas, pi_c, TT4, flight.p0, e_c=e, e_t=e).run(
            flight, 1.0).stations["5"].Tt / TT4
        eta_t = (1.0 - tau_t) / (1.0 - tau_t ** (1.0 / e))
        print(f"{pi_c:>6.0f} {eta_c:>8.4f} {e:>6.2f} {eta_t:>8.4f}")

    # Equivalence made visible: the design-point engine specified via e and via the
    # CONVERTED eta is the same machine (the gate tests/test_polytropic.py pins to 1e-9).
    eta_c = (PI_C ** gc - 1.0) / (PI_C ** (gc / e) - 1.0)
    poly = build_turbojet(gas, PI_C, TT4, flight.p0, e_c=e, e_t=e).run(flight, 1.0)
    tau_t = poly.stations["5"].Tt / TT4
    eta_t = (1.0 - tau_t) / (1.0 - tau_t ** (1.0 / e))
    iso = build_turbojet(gas, PI_C, TT4, flight.p0, eta_c=eta_c, eta_t=eta_t).run(flight, 1.0)
    dF = abs(poly.performance.specific_thrust - iso.performance.specific_thrust)
    print(f"At pi_c={PI_C:.0f}, e=0.90 implies eta_c={eta_c:.4f}, eta_t={eta_t:.4f}; the "
          f"converted-eta engine\nagrees on specific thrust to {dF:.0e} N·s/kg — one machine, two knobs.")


def print_variable_cp_table(flight):
    """Rung-3 payoff: rung-2's FROZEN cp vs rung-3's variable cp(T), same design point.

    Rung 3 builds on rung 2, so the honest baseline is the rung-2 DUAL gas (frozen
    cp_c=1004 cold, cp_t=1239 hot) — the one you just left — NOT the rung-1 single
    gas (against which the fuel comparison would flip sign and mislead). Hold the
    ideal design point fixed and only thaw cp into cp(T):
      - cp climbs with T (printed below), so the SAME compressor pressure work is a
        SMALLER temperature rise -> Tt3 lands COOLER than the frozen-cp answer;
      - rung 2 had to PICK one hot cp; it froze cp_t=1239, a value cp(T) only reaches
        near the turbine inlet (~1240 K). Averaged across the burner's enthalpy climb
        the true cp is lower (~1130), so freezing it OVERstated the products' heat and
        thus the fuel -> variable cp(T) needs LESS fuel (far falls).
    cp(T) is a numbers/curvature effect, not a leg-tilt, so it rides as a table (the
    T-s diagram is left alone).
    """
    frozen_gas = Gas(gamma_t=1.3, cp_t=1239.0, R_t=285.9)   # the rung-2 dual gas
    vary_gas = Gas.thermally_perfect()                       # NASA air (cold) + lean products (hot)
    frozen = build_turbojet(frozen_gas, PI_C, TT4, flight.p0).run(flight, 1.0)
    vary = build_turbojet(vary_gas, PI_C, TT4, flight.p0).run(flight, 1.0)

    print("\nVariable cp(T) (rung 3): rung-2 frozen cp vs thermally-perfect cp(T), same design point")
    print(f"{'Station':>8} {'Tt frozen':>10} {'Tt cp(T)':>9}  {'pt frozen':>10} {'pt cp(T)':>9}")
    print("-" * 52)
    for label in ("0", "2", "3", "4", "5", "9"):
        c, t = frozen.stations[label], vary.stations[label]
        print(f"{label:>8} {c.Tt:>10.1f} {t.Tt:>9.1f}  {c.pt / 1000:>10.2f} {t.pt / 1000:>9.2f}")
    cp_c3, cp_c8 = vary_gas.cp_c_at(300.0), vary_gas.cp_c_at(800.0)
    cp_t3, cp_t15 = vary_gas.cp_t_at(300.0), vary_gas.cp_t_at(1500.0)
    avg_cp_t = vary_gas.h_t(TT4) / TT4                       # burner enthalpy-average cp
    print(f"cp(T) varies (rung 1-2 froze it): cold air {cp_c3:.0f}->{cp_c8:.0f} over 300->800 K; "
          f"hot products {cp_t3:.0f}->{cp_t15:.0f} over 300->1500 K.")
    print(f"Gas-table effect at pi_c={PI_C:.0f}: Tt3 {frozen.stations['3'].Tt:.1f} -> "
          f"{vary.stations['3'].Tt:.1f} K (cooler).")
    print(f"Frozen cp_t=1239 vs true burner-average {avg_cp_t:.0f} J/(kg·K) -> far "
          f"{frozen.stations['4'].far:.5f} -> {vary.stations['4'].far:.5f} (less fuel), "
          f"F/mdot {frozen.performance.specific_thrust:.1f} -> {vary.performance.specific_thrust:.1f} N·s/kg.")


def print_reacting_table(flight):
    """Rung-4 payoff: reacting products — the composition (and thus cp_t, R_t) TRACKS f.

    Rung 3 froze the products at one lean mixture; rung 4 computes the product mole
    numbers from f by (CH2)n lean-complete-combustion stoichiometry. Two views:
      (1) at the design point, the reacting gas solves its own f (implicit burner)
          against the composition it produces — vs the rung-3 frozen-composition run;
      (2) an f-sweep (driven by Tt4): as more fuel burns, CO2/H2O rise, excess O2
          falls, cp_t climbs (products are heavier heat sinks), and R_t rises slightly
          (each mol fuel swaps 1.5 O2 for CO2 + light H2O, lowering the mean molar mass).
    Composition(f) is a numbers effect, not a leg-tilt, so it rides as a table.
    """
    frozen = Gas.thermally_perfect()          # rung-3 fixed-composition lean products
    react = Gas.reacting()                     # rung-4 composition(f)
    rf = build_turbojet(frozen, PI_C, TT4, flight.p0).run(flight, 1.0)
    rr = build_turbojet(react, PI_C, TT4, flight.p0).run(flight, 1.0)

    print("\nReacting products (rung 4): rung-3 frozen composition vs composition(f), same design point")
    print(f"{'Station':>8} {'Tt frozen':>10} {'Tt react':>9}  {'pt frozen':>10} {'pt react':>9}")
    print("-" * 52)
    for label in ("0", "2", "3", "4", "5", "9"):
        c, t = rf.stations[label], rr.stations[label]
        print(f"{label:>8} {c.Tt:>10.1f} {t.Tt:>9.1f}  {c.pt / 1000:>10.2f} {t.pt / 1000:>9.2f}")
    print(f"far: frozen-composition {rf.stations['4'].far:.5f} -> reacting {rr.stations['4'].far:.5f}; "
          f"F/mdot {rf.performance.specific_thrust:.1f} -> {rr.performance.specific_thrust:.1f} N·s/kg.")

    print("\nf-sweep (rung 4): composition and cp_t track the fuel/air ratio (Tt4 drives f)")
    print(f"{'Tt4 [K]':>8} {'far':>8} {'cp_t@Tt4':>9} {'CO2 %':>7} {'H2O %':>7} {'O2 %':>7} "
          f"{'R_t':>7} {'F/mdot':>8}")
    print("-" * 68)
    for Tt4 in (1200.0, 1400.0, 1600.0, 1800.0):
        r = build_turbojet(react, PI_C, Tt4, flight.p0).run(flight, 1.0)
        f = r.stations["4"].far
        comp = _products_composition(f)
        tot = sum(comp.values())
        print(f"{Tt4:>8.0f} {f:>8.5f} {react.cp_t_at(Tt4, f):>9.1f} "
              f"{100 * comp['CO2'] / tot:>7.3f} {100 * comp['H2O'] / tot:>7.3f} "
              f"{100 * comp['O2'] / tot:>7.3f} {react.R_t_at(f):>7.2f} "
              f"{r.performance.specific_thrust:>8.1f}")


def _cycle_points(result, flight):
    """The six cycle points as {label: (s, T)} in (entropy, temperature) space.

    Totals for the internal stations 2..5; STATIC for the freestream (0) and the
    fully-expanded exhaust (9, p9 = p0) — see the rung-1 note below on why 0 and 9
    are static. Entropy datum is the station-0 static state (s0 = 0). This uses a
    SINGLE gas (cp_c == cp_t here), so one s(T,p) is exact; a dual-cp cycle would
    need a section-aware entropy with a combustion datum offset (deferred — it
    would muddy the diagram without changing the leg-tilt lesson).
    """
    gas, Tref, pref = Gas(), flight.T0, flight.p0

    def s(T, p):
        return gas.cp_c * math.log(T / Tref) - gas.R_c * math.log(p / pref)

    st = result.stations
    pts = [
        ("0", flight.T0, flight.p0),
        ("2", st["2"].Tt, st["2"].pt),
        ("3", st["3"].Tt, st["3"].pt),
        ("4", st["4"].Tt, st["4"].pt),
        ("5", st["5"].Tt, st["5"].pt),
        ("9", result.T9, flight.p0),
    ]
    return {label: (s(T, p), T) for label, T, p in pts}


def plot_ts_diagram(ideal, real, flight):
    """Overlay the ideal cycle (vertical "isentropic" legs) and the real cycle
    (legs tilted right by entropy generation) — the rung-2 payoff artifact.

    Each cycle draws as a closed Brayton loop: two work legs (0->2->3 and
    4->5->9) and two constant-pressure legs (combustion 3->4, heat rejection
    9->0). WHY 0 and 9 are STATIC while 2..5 are total (carried from rung 1): in
    pure total coordinates an ideal nozzle conserves the totals, so station 9
    collapses onto 5 and the heat-rejection leg vanishes; drawing 0 and 9 static
    (at ambient p0) restores the closure 9->0 and makes the ram rise (0->2) and
    nozzle expansion (5->9) read straight off the legs.

    The lesson: the ideal work legs are vertical (s constant); the real ones lean
    right because every real component generates entropy (compressor/turbine
    irreversibility, inlet/burner/nozzle pressure loss). The loop also encloses
    less area and ends hotter — that lost area is thrust the losses cost.
    """
    ci = _cycle_points(ideal, flight)
    cr = _cycle_points(real, flight)

    fig, ax = plt.subplots(figsize=(8.5, 6.5))

    def draw(coords, color, ls, lw, label, alpha):
        # Work legs (would be vertical if isentropic).
        for leg in (["0", "2", "3"], ["4", "5", "9"]):
            ax.plot([coords[l][0] for l in leg], [coords[l][1] for l in leg],
                    color=color, ls=ls, lw=lw, alpha=alpha, zorder=2)
        # Combustion leg 3->4 and heat-rejection leg 9->0 as smooth curves. The
        # cp*ln(T) term is the isobaric shape; a linear residual makes the curve
        # land EXACTLY on the true endpoint. For an isobar (ideal burner, both
        # heat-rejection points at p0) the residual is ~0. For the REAL burner
        # pt4 = pi_b*pt3 < pt3, so the residual is the pressure-loss entropy
        # -R*ln(pt4/pt3) — drawing it is what makes that loss visible (the leg ends
        # further right) instead of silently hidden.
        cp = Gas().cp_c
        for a, b in (("3", "4"), ("9", "0")):
            (sa, Ta), (sb, Tb) = coords[a], coords[b]
            residual = sb - (sa + cp * math.log(Tb / Ta))  # = -R*ln(pb/pa)
            ts = [Ta + (Tb - Ta) * i / 79 for i in range(80)]
            ss = [sa + cp * math.log(T / Ta) + residual * (T - Ta) / (Tb - Ta) for T in ts]
            ax.plot(ss, ts, color=color, ls=ls, lw=lw, alpha=alpha, zorder=2)
        ax.plot([], [], color=color, ls=ls, lw=lw, label=label)  # legend proxy
        for lbl, (sv, T) in coords.items():
            ax.scatter([sv], [T], color=color, s=28, zorder=3, alpha=alpha)

    draw(ci, "tab:blue", "--", 1.8, "ideal (isentropic legs)", 0.7)
    draw(cr, "tab:red", "-", 2.2, "real (legs tilt right)", 1.0)

    # Label the real-cycle stations (the ones that moved).
    for lbl, (sv, T) in cr.items():
        ax.annotate(f"  {lbl}", (sv, T), fontsize=11, fontweight="bold", va="center")

    ax.set_xlabel("entropy  s − s0  [J/(kg·K)]")
    ax.set_ylabel("temperature  T  [K]")
    ax.set_title("Turbojet T–s diagram — ideal vs real components\n"
                 f"(M0={flight.M0}, π_c={PI_C:.0f}, Tt4={TT4:.0f} K; "
                 "real: η_c=0.88, η_t=0.90, π losses)")
    ax.legend(loc="upper left")
    ax.grid(True, alpha=0.3)
    fig.tight_layout()
    fig.savefig(TS_DIAGRAM_PATH, dpi=120)
    plt.close(fig)
    print(f"\nT–s diagram (ideal vs real) written to {TS_DIAGRAM_PATH}")


def main():
    gas = Gas()  # single cold-air-standard gas for the design-point comparison
    ideal = build_turbojet(gas, PI_C, TT4, FLIGHT.p0).run(FLIGHT, mdot=1.0)
    real = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, **REAL_LOSSES).run(FLIGHT, mdot=1.0)

    print_station_table("IDEAL turbojet (rung-1 validation case)", ideal)
    print_station_table("REAL components (same design point, with losses)", real)

    dF = 100.0 * (real.performance.specific_thrust / ideal.performance.specific_thrust - 1.0)
    dS = 100.0 * (real.performance.tsfc / ideal.performance.tsfc - 1.0)
    print(f"\nLosses cost: specific thrust {dF:+.1f}%, TSFC {dS:+.1f}% "
          "(less thrust, burned harder).")

    print_polytropic_table(gas, FLIGHT)

    print_variable_cp_table(FLIGHT)

    print_reacting_table(FLIGHT)

    plot_ts_diagram(ideal, real, FLIGHT)


if __name__ == "__main__":
    main()
