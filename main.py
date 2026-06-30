"""Run the rung-1 validation case: print the full station table and draw the T-s diagram.

    python main.py

Requires the components in turbojet/ to be implemented. The plot needs
matplotlib (`pip install -r requirements.txt`).
"""
import math

import matplotlib

matplotlib.use("Agg")  # headless: render to a file, never pop a window (no plt.show)
import matplotlib.pyplot as plt  # noqa: E402

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import Gas  # noqa: E402

TS_DIAGRAM_PATH = "ts_diagram.png"


def print_station_table(result):
    """Print Tt, pt (and far) at every station so the numbers can be watched.

    Pure formatting over an EngineResult — the physics lives in the engine.
    """
    print(f"{'Station':>8} {'Tt [K]':>10} {'pt [kPa]':>10} {'far':>9}")
    print("-" * 40)
    for label, s in result.stations.items():
        print(f"{label:>8} {s.Tt:>10.1f} {s.pt / 1000:>10.2f} {s.far:>9.5f}")
    print()
    print(f"V0 = {result.V0:7.1f} m/s    V9 = {result.V9:7.1f} m/s    M9 = {result.M9:.3f}")
    p = result.performance
    print(f"Specific thrust = {p.specific_thrust:.1f} N·s/kg    TSFC = {p.tsfc:.3e} kg/(N·s)")
    print(f"eta_th = {p.eta_thermal:.4f}    eta_p = {p.eta_propulsive:.4f}    eta_o = {p.eta_overall:.4f}")


def plot_ts_diagram(result, gas, flight):
    """T-s diagram of the ideal turbojet cycle — the payoff artifact (SPEC.md).

    The cycle draws as a CLOSED Brayton loop with two isentropic legs (vertical)
    and two constant-pressure legs (curves), exactly as the spec asks:

        0 -> 2 -> 3   left isentrope  (ram + compression), s held fixed
        3 -> 4        top curve       (combustion) at constant p = pt3
        4 -> 5 -> 9   right isentrope (turbine + nozzle), s held fixed
        9 -> 0        bottom curve    (heat rejection) at constant p = p0

    WHY 0 and 9 are STATIC while 2..5 are total: an ideal nozzle conserves the
    totals, so in pure total coordinates station 9 collapses onto station 5 and
    the loop has only ONE constant-pressure leg (the burner). The second leg --
    heat rejection -- exists only as the STATIC closure 9 -> 0 at ambient p0,
    joining the static exhaust (T9, p0) back to the static freestream (T0, p0).
    Drawing 0 and 9 static also makes the physics visible: the 0->2 rise IS the
    ram heating, and the 5->9 drop IS the nozzle expansion -- both read straight
    off the isentropes as temperature changes.

    Entropy uses s(T, p) = cp ln(T/Tref) - R ln(p/pref) with the datum at the
    freestream static state, so station 0 sits at s = 0 (SPEC.md deliverables).
    """
    Tref, pref = flight.T0, flight.p0  # entropy datum: station-0 static -> s0 = 0

    def s(T, p):
        return gas.cp * math.log(T / Tref) - gas.R * math.log(p / pref)

    # The six cycle points as (label, T, p): totals for the internal stations,
    # STATIC for the freestream (0) and the fully-expanded exhaust (9, p9 = p0).
    st = result.stations
    points = [
        ("0", flight.T0, flight.p0),
        ("2", st["2"].Tt, st["2"].pt),
        ("3", st["3"].Tt, st["3"].pt),
        ("4", st["4"].Tt, st["4"].pt),
        ("5", st["5"].Tt, st["5"].pt),
        ("9", result.T9, flight.p0),
    ]
    coords = {label: (s(T, p), T) for label, T, p in points}

    def pressure_curve(p, T_start, T_end, n=80):
        """Sample a constant-pressure leg: T sweeps start->end, s = s(T, p).

        Endpoints reuse the station temperatures so each curve meets the
        isentrope points exactly and the loop closes with no visible gap.
        """
        ts = [T_start + (T_end - T_start) * i / (n - 1) for i in range(n)]
        return [s(T, p) for T in ts], ts

    fig, ax = plt.subplots(figsize=(8, 6))

    # Isentropic legs: connect the station points directly. They share s to
    # within ~0.4 J/(kg.K) -- a sub-pixel slant from cp = 1004.0 disagreeing with
    # gamma*R/(gamma-1) = 1004.5 (the same rounded-constant artifact noted in the
    # Nozzle), NOT a physics bug. Plotting the honest (s, T) is the "show the
    # work" choice; the drift is invisible at this scale.
    for leg in (["0", "2", "3"], ["4", "5", "9"]):
        ax.plot([coords[l][0] for l in leg], [coords[l][1] for l in leg],
                color="tab:blue", lw=2, zorder=2)

    # Constant-pressure legs as curves (combustion at pt3, heat rejection at p0).
    s_comb, T_comb = pressure_curve(st["3"].pt, st["3"].Tt, st["4"].Tt)
    ax.plot(s_comb, T_comb, color="tab:red", lw=2, zorder=2,
            label=f"combustion  (p = {st['3'].pt / 1000:.0f} kPa)")
    s_rej, T_rej = pressure_curve(flight.p0, result.T9, flight.T0)
    ax.plot(s_rej, T_rej, color="tab:green", lw=2, zorder=2,
            label=f"heat rejection  (p = {flight.p0 / 1000:.0f} kPa)")

    # Mark and label the six stations.
    for label, (sv, T) in coords.items():
        ax.scatter([sv], [T], color="black", zorder=3)
        ax.annotate(f"  {label}", (sv, T), fontsize=11, fontweight="bold",
                    va="center")

    ax.set_xlabel("entropy  s - s0  [J/(kg·K)]")
    ax.set_ylabel("temperature  T  [K]")
    ax.set_title("Ideal turbojet — T–s diagram\n"
                 f"(M0={flight.M0}, π_c={st['3'].pt / st['2'].pt:.0f}, "
                 f"Tt4={st['4'].Tt:.0f} K)")
    ax.legend(loc="upper left")
    ax.grid(True, alpha=0.3)
    fig.tight_layout()
    fig.savefig(TS_DIAGRAM_PATH, dpi=120)
    plt.close(fig)
    print(f"\nT–s diagram written to {TS_DIAGRAM_PATH}")


def main():
    gas = Gas()
    flight = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
    engine = build_turbojet(gas, pi_c=10.0, Tt4=1500.0, p_ambient=flight.p0)
    result = engine.run(flight, mdot=1.0)
    print_station_table(result)
    plot_ts_diagram(result, gas, flight)


if __name__ == "__main__":
    main()
