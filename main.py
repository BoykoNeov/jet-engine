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
from turbojet.gas import Gas  # noqa: E402

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

    plot_ts_diagram(ideal, real, FLIGHT)


if __name__ == "__main__":
    main()
