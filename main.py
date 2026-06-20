"""Run the rung-1 validation case: print the full station table and draw the T-s diagram.

    python main.py

Requires the components in turbojet/ to be implemented. The plot needs
matplotlib (`pip install -r requirements.txt`).
"""
from turbojet.engine import FlightCondition, build_turbojet
from turbojet.gas import Gas


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


def plot_ts_diagram(result, gas):
    """T-s diagram: isentropic legs vertical, constant-pressure legs as curves.

    Rung-1 deliverable (the payoff artifact). Implement once the cycle solves;
    needs s(T, p) = cp ln(T/Tref) - R ln(p/pref) for the constant-pressure
    curves. Mark stations 0, 2, 3, 4, 5, 9. See SPEC.md deliverables checklist.
    """
    raise NotImplementedError("T-s diagram (SPEC.md deliverables)")


def main():
    gas = Gas()
    flight = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
    engine = build_turbojet(gas, pi_c=10.0, Tt4=1500.0, p_ambient=flight.p0)
    result = engine.run(flight, mdot=1.0)
    print_station_table(result)
    plot_ts_diagram(result, gas)


if __name__ == "__main__":
    main()
