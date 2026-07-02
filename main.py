"""Run the cycle and draw the T-s diagram.

    python main.py

Rung 2 payoff: run the ideal turbojet AND a real-components version at the SAME
design point, print both station tables, and overlay them on one T-s diagram so
the "isentropic" legs visibly TILT RIGHT (entropy generated) once losses are on.

Requires the components in turbojet/ and matplotlib (`pip install -r requirements.txt`).
"""
import math
import sys

import matplotlib

matplotlib.use("Agg")  # headless: render to a file, never pop a window (no plt.show)
import matplotlib.pyplot as plt  # noqa: E402

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, _products_composition, _equilibrium_composition, _h_molar_A,
    _HF_FUEL_DEFAULT, _M_AIR, _M_CH2, _F_STOICH, _air_mole_fractions,
    _equilibrium_no_fraction, _primary_aft, _thermal_no,
    _quench_trajectory, _quench_no,
)

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


def print_forkb_table(flight):
    """Rung-5 payoff: Fork B — heat release DERIVED from formation enthalpies.

    Rung 4 ASSUMED hPR = 42.8 MJ/kg. Fork B carries each species' formation enthalpy,
    so the LHV FALLS OUT of the chemistry and the burner's balance is on the absolute
    (formation + sensible) scale. The headline is a NON-event by design: for complete
    combustion the released energy is identically f*LHV, so Fork B reproduces the
    rung-4 (Fork A) cycle to machine precision. The point is structural, not numeric:
      - hPR is now EXPLAINED (derived), not typed in;
      - enthalpies live on the absolute scale that rung-6 dissociation needs;
      - the heat release will track composition automatically once products shift.
    """
    fa = Gas.reacting(hPR=42.8e6)          # rung-4 Fork A: assumed hPR
    fb = Gas.reacting_forkb()              # rung-5 Fork B: DERIVED LHV
    ra = build_turbojet(fa, PI_C, TT4, flight.p0).run(flight, 1.0)
    rb = build_turbojet(fb, PI_C, TT4, flight.p0).run(flight, 1.0)

    print("\nFork B (rung 5): assumed hPR (Fork A) vs DERIVED heat release, same design point")
    print(f"  hPR: Fork A assumes {fa.hPR / 1e6:.4f} MJ/kg  ->  "
          f"Fork B DERIVES {fb.lhv / 1e6:.4f} MJ/kg from formation enthalpies "
          f"(fuel ΔHf = {fb.hf_fuel_molar / 1000:.2f} kJ/mol)")
    print(f"{'Station':>8} {'Tt A':>9} {'Tt B':>9}  {'pt A':>9} {'pt B':>9}")
    print("-" * 50)
    for label in ("0", "2", "3", "4", "5", "9"):
        a, b = ra.stations[label], rb.stations[label]
        print(f"{label:>8} {a.Tt:>9.1f} {b.Tt:>9.1f}  {a.pt / 1000:>9.2f} {b.pt / 1000:>9.2f}")
    df = abs(ra.stations["4"].far - rb.stations["4"].far)
    print(f"far: Fork A {ra.stations['4'].far:.6f} vs Fork B {rb.stations['4'].far:.6f} "
          f"(|Δ| = {df:.1e} — EXACT: released energy ≡ f·LHV for complete combustion).")
    print("  Fork B buys structure, not digits: absolute-enthalpy scale for rung-6 "
          "dissociation, and heat release that will track composition. Products carry")
    print(f"  formation enthalpy {fb.hf_products_mass(rb.stations['4'].far) / 1e6:.3f} MJ/kg "
          f"(vs air 0.000), so absolute h_t(Tt4) = {fb.h_t_abs(TT4, rb.stations['4'].far) / 1e6:.3f} "
          f"MJ/kg sits below sensible {fb.h_t(TT4, rb.stations['4'].far) / 1e6:.3f} MJ/kg.")


def _aft_ch2(f, p, dissociate):
    """(CH2)n constant-p adiabatic flame temp per mol air, SCALE A (the physical datum,
    docs/rung6-anchor § 2b). dissociate=False -> complete combustion (rung-5 value)."""
    x = _air_mole_fractions()
    n_fuel = f * _M_AIR / _M_CH2
    H_react = n_fuel * _HF_FUEL_DEFAULT                  # air = 0 on scale A at 298.15
    lo, hi = 800.0, 3200.0
    for _ in range(100):
        T = 0.5 * (lo + hi)
        if dissociate:
            comp = _equilibrium_composition(f, T, p)
        else:
            comp = {"CO2": n_fuel, "H2O": n_fuel, "O2": x["O2"] - 1.5 * n_fuel,
                    "N2": x["N2"], "Ar": x["Ar"]}
        H_prod = sum(comp[s] * _h_molar_A(s, T) for s in comp)
        lo, hi = (lo, T) if H_prod > H_react else (T, hi)
    return 0.5 * (lo + hi)


def print_equilibrium_table(flight):
    """Rung-6 payoff: chemical equilibrium — products DISSOCIATE at high temperature.

    The headline mirrors rung 5's: at the cycle's station 4 the numbers barely move,
    because dissociation is suppressed twice — LEAN combustion (excess O2) AND high
    combustor pressure (mole-increasing reactions shift back). The drama lives in the
    unconstrained ADIABATIC FLAME TEMPERATURE, which finally falls into the real band:
      - the cycle: Fork B vs equilibrium is identical to ~0.15% on far (measured below);
      - the diagnostic: stoichiometric AFT drops ~115 K (2375 -> ~2259 K), because
        dissociating CO2/H2O ABSORBS heat, capping the peak — the gap rung 5 flagged.
    """
    fb = Gas.reacting_forkb()               # rung-5 Fork B: complete combustion
    eq = Gas.reacting_equilibrium()         # rung-6 equilibrium: dissociating products
    rb = build_turbojet(fb, PI_C, TT4, flight.p0).run(flight, 1.0)
    re = build_turbojet(eq, PI_C, TT4, flight.p0).run(flight, 1.0)

    print("\nEquilibrium (rung 6): Fork B (complete) vs dissociating products, same design point")
    print(f"{'Station':>8} {'Tt B':>9} {'Tt eq':>9}  {'pt B':>9} {'pt eq':>9}")
    print("-" * 50)
    for label in ("0", "2", "3", "4", "5", "9"):
        b, e = rb.stations[label], re.stations[label]
        print(f"{label:>8} {b.Tt:>9.1f} {e.Tt:>9.1f}  {b.pt / 1000:>9.2f} {e.pt / 1000:>9.2f}")
    fB, fE = rb.stations["4"].far, re.stations["4"].far
    pt4 = re.stations["4"].pt
    print(f"far: Fork B {fB:.6f} -> equilibrium {fE:.6f} (+{100*(fE-fB)/fB:.3f}%) — a tiny "
          f"correction: at pt4={pt4/1e5:.1f} bar, lean, dissociation is doubly suppressed.")
    comp = _equilibrium_composition(fE, TT4, pt4)
    tot = sum(comp.values())
    trace = {s: f"{100*comp[s]/tot:.4f}%" for s in ("CO", "OH", "O", "H", "H2")}
    print(f"  station-4 dissociation products (frozen downstream): {trace}")

    print("\n  Adiabatic flame temperature (the diagnostic that finally drops), 1 atm:")
    print(f"  {'f':>8} {'no-dissoc (rung 5)':>18} {'equilibrium (rung 6)':>20} {'drop K':>8}")
    for f in (0.030, 0.050, _F_STOICH * 0.999):
        tf = _aft_ch2(f, 101325.0, dissociate=False)
        te = _aft_ch2(f, 101325.0, dissociate=True)
        tag = "  (≈stoich)" if f > 0.06 else ""
        print(f"  {f:>8.4f} {tf:>18.1f} {te:>20.1f} {tf-te:>8.1f}{tag}")
    print("  Stoich ~2259 K lands in the real kerosene-air band; the ~115 K gap IS "
          "dissociation (endothermic), the rung-5 AFT overshoot explained.")

    print("\n  The Kp (p/p0)^Δν factor, live — dissociation falls as pressure rises "
          "(stoich, T=2300 K):")
    f = _F_STOICH * 0.999
    for p_atm in (1.0, 5.0, 13.0):
        c = _equilibrium_composition(f, 2300.0, p_atm * 101325.0)
        print(f"    p={p_atm:>5.1f} atm:  CO/(CO+CO2) = {c['CO']/(c['CO']+c['CO2']):.4f}")


def print_nox_table(flight):
    """Rung-7 payoff: thermal NOx — kinetically-limited NO (the lesson INVERTS rung 6).

    Rung 6: the major species DO reach equilibrium (cycle barely moves; drama in the AFT
    drop). Rung 7: NO does NOT — at realistic residence times it is kinetically frozen far
    below its equilibrium value, and it is EXPONENTIALLY temperature-sensitive. NO is trace
    (ppm), so it is a DECOUPLED diagnostic: the cycle stays bit-for-bit rung 6. Two views,
    mirroring how rung 6 showed the AFT diagnostic separately from the cycle f-delta:
      (1) a stoich FLAME-TEMPERATURE sweep carrying the drama (equilibrium NO, the kinetic
          freezing at tau=3 ms, the characteristic time tau_NO >> residence, the ~exp(-38370/T)
          T-sensitivity);
      (2) the honest STATION-4 mixed-out number at the (capped, lean) design point — where
          thermal NO is negligible, because real engine NOx is a hot primary-zone effect this
          single-Tt4 cycle model does not resolve (a stated seam).
    """
    eq = Gas.reacting_equilibrium()
    fst = _F_STOICH * 0.999
    tau = 3e-3                                   # 3 ms: typical primary-zone residence (a knob)

    print("\nThermal NOx (rung 7): kinetically-limited NO — the lesson inverts rung 6")
    print(f"  Stoich (CH2)n flame sweep, 1 atm, residence tau = {tau*1e3:.0f} ms:")
    print(f"  {'T [K]':>7} {'x_O ppm':>9} {'NO_eq ppm':>10} {'NO_kin ppm':>11} "
          f"{'kin/eq %':>9} {'tau_NO ms':>10} {'rate rel':>9}")
    print("  " + "-" * 70)
    rate_ref = eq.thermal_nox(fst, 2000.0, 101325.0, tau=tau).initial_rate   # 2000 K reference
    for T in (1800.0, 2000.0, 2200.0, 2400.0):
        n = eq.thermal_nox(fst, T, 101325.0, tau=tau)
        comp = _equilibrium_composition(fst, T, 101325.0)
        xO = comp["O"] / sum(comp.values()) * 1e6
        print(f"  {T:>7.0f} {xO:>9.1f} {n.ppm_eq:>10.0f} {n.ppm:>11.2f} "
              f"{100*n.fraction_of_equil:>9.2f} {n.char_time*1e3:>10.1f} {n.initial_rate/rate_ref:>9.3f}")
    print("  NO is frozen at a few % of equilibrium (tau_NO >> ms residence); the initial")
    print("  rate explodes ~30x per 200 K — it is PEAK FLAME TEMPERATURE, not the capped")
    print("  mixed-out Tt4, that governs NOx (why NOx, not just blade metal, caps the flame).")

    print(f"\n  Kinetic NO climbs toward equilibrium as residence grows (stoich, 2300 K, 1 atm):")
    print(f"  {'tau [ms]':>9} {'NO_kin ppm':>11} {'% of equil':>11}")
    for tau_ms in (0.5, 1.0, 3.0, 10.0, 100.0, 1000.0):
        n = eq.thermal_nox(fst, 2300.0, 101325.0, tau=tau_ms * 1e-3)
        print(f"  {tau_ms:>9.1f} {n.ppm:>11.1f} {100*n.fraction_of_equil:>11.2f}")

    # The honest cycle number: the (capped, lean, mixed-out) turbine inlet makes almost none.
    re = build_turbojet(eq, PI_C, TT4, flight.p0).run(flight, 1.0)
    st4 = re.stations["4"]
    n4 = eq.thermal_nox(st4.far, st4.Tt, st4.pt, tau=tau)
    print(f"\n  Station 4 (this design point: Tt4={st4.Tt:.0f} K, {st4.pt/1e5:.1f} bar, lean "
          f"far={st4.far:.4f}): equilibrium NO {n4.ppm_eq:.0f} ppm, but kinetic NO only "
          f"{n4.ppm:.2e} ppm\n  ({100*n4.fraction_of_equil:.4f}% of equil, EI_NO={n4.ei_no:.2e} "
          "g/kg) — too cool AND kinetically frozen. Real NOx is a hot primary-zone effect.")

    # Pressure inverts rung 6: dissociation was p-suppressed; equilibrium NO is Dnu=0.
    print("\n  Contrast rung 6: equilibrium NO carries NO (p/p0) factor (Dnu=0). At a LEAN "
          "f=0.030, 2000 K,")
    xs = [_equilibrium_no_fraction(_equilibrium_composition(0.030, 2000.0, p * 101325.0), 2000.0)
          for p in (1.0, 13.0)]
    print(f"  NO_eq is {xs[0]*1e6:.0f} ppm at 1 atm vs {xs[1]*1e6:.0f} ppm at 13 atm — high "
          "combustor pressure does NOT directly suppress NOx.")


def print_zoning_table(flight):
    """Rung-8 payoff: combustor zoning COMPLETES rung 7's inversion.

    Rung 7 ended honestly: at the capped, lean, mixed-out station 4 (Tt4=1500 K, φ≈0.40)
    thermal NO is ~zero (EI_NO ≈ 1e-5 g/kg), because "real NOx is a hot primary-zone effect
    this single-Tt4 model does not resolve." Rung 8 resolves it — WITHOUT touching the cycle
    (NO is still trace; every station is bit-for-bit rung 6). It runs the SAME rung-7
    Zeldovich integrator on a two-zone combustor: a near-stoichiometric PRIMARY (φ_p ≤ 1,
    AFT ≈ 2000–2450 K, where NO forms) then DILUTION air that cools the mixed-out gas back to
    Tt4 while FREEZING the NO. EI_NO climbs from ~zero into the measured ICAO band (~13–20
    g/kg at φ_p ≈ 0.9–1.0) — a ~6-order lift purely from resolving WHERE the chemistry runs.

    The invariants on display: (1) T_mix is split-independent and returns to ≈ Tt4 (the
    re-equilibration gate — enthalpy conserved, majors recombine); (2) dilution lowers the NO
    mole FRACTION (primary → mix ppm) but NOT the EI (per kg fuel), the clean
    concentration-vs-emission-index separation.
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
    tau = 3e-3

    n_mixed = eq.thermal_nox(far, Tt4, p, tau=tau)          # the rung-7 mixed-out ~zero
    print("\nCombustor zoning (rung 8): the primary-zone NOx effect — completes rung 7")
    print(f"  Design point: Tt3={Tt3:.0f} K, Tt4={Tt4:.0f} K, p={p/1e5:.1f} bar, overall "
          f"far={far:.4f} (φ={far/_F_STOICH:.2f}), τ={tau*1e3:.0f} ms")
    print(f"  Mixed-out station 4 (rung 7): EI_NO = {n_mixed.ei_no:.2e} g/kg — essentially zero.")
    print(f"  {'φ_p':>5} {'α_air':>7} {'far_p':>7} {'AFT K':>7} {'NO_eq':>8} {'NO_kin':>9} "
          f"{'EI_NO':>8} {'T_mix':>7} {'NO_mix':>8}")
    print(f"  {'':>5} {'':>7} {'':>7} {'(prim)':>7} {'ppm':>8} {'ppm':>9} {'g/kg':>8} "
          f"{'K':>7} {'ppm':>8}")
    print("  " + "-" * 74)
    for phi_p in (0.7, 0.8, 0.9, 1.0):
        z = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau=tau)
        print(f"  {phi_p:>5.2f} {z.alpha:>7.4f} {z.far_primary:>7.5f} {z.T_primary:>7.0f} "
              f"{z.primary.ppm_eq:>8.0f} {z.ppm_primary:>9.1f} {z.ei_no:>8.2f} "
              f"{z.T_mix:>7.0f} {z.ppm_mix:>8.0f}")
    z1 = eq.zoned_nox(far, Tt3, Tt4, p, 1.0, tau=tau)
    z07 = eq.zoned_nox(far, Tt3, Tt4, p, 0.7, tau=tau)
    print(f"  Primary φ_p 0.7→1.0 lifts the AFT {z07.T_primary:.0f}→{z1.T_primary:.0f} K "
          f"(+{z1.T_primary-z07.T_primary:.0f} K) and swings EI_NO {z1.ei_no/z07.ei_no:.0f}× "
          "— the rung-7 exp-in-T rate showing through.")
    print(f"  At φ_p≈1 EI_NO ≈ {z1.ei_no:.0f} g/kg (ICAO take-off band 18–64), vs the mixed-out "
          f"{n_mixed.ei_no:.0e} — a ~{math.log10(z1.ei_no/n_mixed.ei_no):.0f}-order lift.")
    print("  T_mix is IDENTICAL across the sweep and returns to ≈Tt4 (majors re-equilibrate on")
    print("  dilution); NO_mix < NO_kin (fraction diluted) but EI_NO is conserved (moles fixed).")
    print("  The capped mixed-out turbine inlet never made the NO — a hot zone you averaged")
    print("  away did. THIS is why real combustors fight peak flame T (lean-premixed, RQL).")


def print_rql_table(flight):
    """Rung-9 payoff: the RICH flank of the NOx bell — why RQL burns rich.

    Rung 8 resolved the primary but held it LEAN-to-stoich (φ_p ≤ 1) — it could only climb the
    lean side of the NO-vs-φ bell. Rung 9 lets the primary run RICH (φ_p up to 2.0): the
    8-species equilibrium pool now carries MAJOR CO/H2 (a branched seed in _equil_solve — no
    new species, reactions, or datum). Sweeping φ_p across the bell, EI_NO PEAKS near
    stoichiometric and then COLLAPSES on the rich flank (the AFT rolls over and the O-starved
    pool crashes [O]/[OH]). A rich primary is therefore a LOW-NOx regime — the whole reason a
    Rich-burn/Quick-Quench/Lean-burn (RQL) combustor burns its primary rich. Mix-out here is
    the IDEAL (infinitely-fast) quench (NO frozen at the primary value); the finite-rate quench
    — NO spiking as the gas DWELLS at stoich while mixing — is the next seam.

    Still a pure diagnostic: NO is trace, every cycle station is bit-for-bit rung 6.
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
    tau = 3e-3

    print("\nRich primary / RQL (rung 9): the rich flank of the NOx bell — completes rung 7's")
    print("inversion on the OTHER side. EI_NO peaks near stoich, then falls as the primary goes rich.")
    print(f"  Design point: Tt3={Tt3:.0f} K, Tt4={Tt4:.0f} K, p={p/1e5:.1f} bar, overall "
          f"far={far:.4f} (φ={far/_F_STOICH:.2f}), τ={tau*1e3:.0f} ms")
    print(f"  {'φ_p':>5} {'AFT K':>7} {'xCO':>7} {'xH2':>7} {'NO_eq':>8} {'NO_kin':>9} "
          f"{'EI_NO':>8} {'T_mix':>7}")
    print(f"  {'':>5} {'(prim)':>7} {'%':>7} {'%':>7} {'ppm':>8} {'ppm':>9} {'g/kg':>8} {'K':>7}")
    print("  " + "-" * 63)
    best = (0.0, 0.0)
    for phi_p in (0.8, 0.9, 1.0, 1.05, 1.1, 1.3, 1.5, 1.8, 2.0):
        z = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau=tau)
        comp = _equilibrium_composition(z.far_primary, z.T_primary, p)
        nt = sum(comp.values())
        if z.ei_no > best[1]:
            best = (phi_p, z.ei_no)
        flank = "  <- peak" if phi_p == 1.0 else ("  rich, low-NOx" if phi_p >= 1.3 else "")
        print(f"  {phi_p:>5.2f} {z.T_primary:>7.0f} {100*comp['CO']/nt:>7.2f} "
              f"{100*comp['H2']/nt:>7.2f} {z.primary.ppm_eq:>8.0f} {z.ppm_primary:>9.1f} "
              f"{z.ei_no:>8.3f} {z.T_mix:>7.0f}{flank}")
    z_stoich = eq.zoned_nox(far, Tt3, Tt4, p, 1.0, tau=tau)
    z_rich = eq.zoned_nox(far, Tt3, Tt4, p, 1.4, tau=tau)
    print(f"  Peak EI_NO ≈ {best[1]:.0f} g/kg near φ_p≈{best[0]:.2f} (ICAO band); a RICH primary "
          f"φ_p=1.4 cuts it to {z_rich.ei_no:.2f} g/kg")
    print(f"  — {z_stoich.ei_no/z_rich.ei_no:.0f}× lower — even though it still burns ALL the fuel. "
          "The rich pool's")
    print("  CO/H2 (major, unoxidized) + rolled-over AFT starve the O/OH the Zeldovich rate needs.")
    print("  THAT is why RQL burns rich, then quick-quenches PAST stoich (the NO peak) to lean.")
    print("  T_mix still returns to ≈Tt4 for every φ_p — the CO/H2 oxidation energy releases on")
    print("  re-equilibration (mix-out is the ideal, infinitely-fast quench; NO frozen).")


def print_finite_quench_table(flight):
    """Rung-10 payoff: the FINITE-rate quench — the RQL hazard, quantified.

    Rung 9's mix-out was the IDEAL (infinitely-fast) quench: NO frozen at the primary value, so
    a rich primary read as low-NOx. But a real quench mixes over a finite time τ_q, and while it
    does the LOCAL mixture sweeps far_p → f_stoich → far_overall — through STOICHIOMETRIC, the
    peak of the NO bell. So a rich primary's temperature RISES through the stoich peak on the way
    down, and the extended-Zeldovich rate RE-MAKES NO along that path (a clamp-free integrator —
    super-equilibrium NO on cooling must not be capped). A SLOW quench dwells at stoich and
    re-makes the NO the rich primary avoided; a FAST quench escapes past the peak. "A rich
    primary is low-NOx" is therefore CONTINGENT on a fast quench — the whole RQL design tension.

    Still a pure diagnostic: NO is trace, every cycle station is bit-for-bit rung 6 (the finite
    quench is opt-in via τ_q; τ_q=None is the exact rung-9 ideal quench).
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
    hf = eq.hf_fuel_molar if eq.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
    # Finite-quench resolution: EI is within ~0.3% of the 240-point anchor by ngrid=80, and a
    # 240-point trajectory is ~25 s (each point re-equilibrates the diluting majors). Use 80 here
    # to keep `python main.py` interactive; the production default (240) stays anchor-exact.
    ng = 80

    print("\nFinite-rate quench (rung 10): the RQL hazard — NO re-made as the gas dwells at stoich")
    print("while the quench air mixes in. Rung 9's rich-flank collapse holds ONLY if the quench is fast.")
    print(f"  Design point: Tt3={Tt3:.0f} K, Tt4={Tt4:.0f} K, p={p/1e5:.1f} bar, overall "
          f"far={far:.4f} (φ={far/_F_STOICH:.2f})")

    # (1) The τ_q sweep at a RICH primary (φ_p=1.5): T rises through the stoich peak, and the NO
    #     spike grows with τ_q. Build the (τ_q-independent) trajectory ONCE and reuse it.
    phi_p = 1.5
    far_p = phi_p * _F_STOICH
    alpha = far / far_p
    T_p = _primary_aft(far_p, p, Tt3, hf)
    comp_p = _equilibrium_composition(far_p, T_p, p)
    nox = _thermal_no(comp_p, T_p, p, 3e-3, far_p)
    n0 = alpha * nox.x_no * sum(comp_p.values())
    tab = _quench_trajectory(comp_p, T_p, alpha, far, Tt3, p, ngrid=ng)   # built ONCE, reused
    T_peak = max(r["T"] for r in tab)
    ei9 = nox.ei_no
    print(f"\n  A rich primary φ_p={phi_p}: AFT={T_p:.0f} K RISES to a stoich peak of {T_peak:.0f} K")
    print(f"  as it quenches (rung-9 ideal-quench EI_NO = {ei9:.4f} g/kg — NO frozen at the primary).")
    print(f"  {'τ_q ms':>8} {'EI_NO g/kg':>11} {'×rung9':>9}   quench speed")
    print("  " + "-" * 46)
    for tau_q in (0.01e-3, 0.1e-3, 0.3e-3, 1e-3, 3e-3, 10e-3):
        q = _quench_no(comp_p, T_p, alpha, far, Tt3, p, n0, tau_q, tab=tab)
        tag = "fast (RQL target)" if tau_q <= 0.3e-3 else ("slow — NO re-made" if tau_q >= 3e-3 else "")
        print(f"  {tau_q*1e3:>8.3f} {q['ei']:>11.4g} {q['ei']/ei9:>8.0f}×   {tag}")

    # (2) The bell with a finite quench: the rung-9 rich-flank collapse is FILLED BACK IN.
    print("\n  The bell re-filled — rung-9 ideal vs a 3 ms quench (the rich flank comes back):")
    print(f"  {'φ_p':>5} {'ideal EI':>10} {'quench 3ms':>11}   note")
    print("  " + "-" * 46)
    for phi in (0.9, 1.0, 1.1, 1.3, 1.5, 1.8):
        z = eq.zoned_nox(far, Tt3, Tt4, p, phi, tau_q=3e-3, quench_ngrid=ng)
        note = ("peak (already at stoich)" if phi == 1.0
                else ("rich: collapsed → re-filled" if phi >= 1.3 else ""))
        print(f"  {phi:>5.2f} {z.ei_no:>10.4g} {z.ei_no_quenched:>11.4g}   {note}")
    print("  The ideal rich flank (φ_p≥1.3) collapses to ≈0; a finite quench fills it to a")
    print("  ~φ_p-independent ~3 g/kg floor — every rich mix passes the SAME stoich peak. The")
    print("  'quick' in quick-quench is the whole game: only a sub-ms quench keeps the rich win.")
    print(f"  (Clamp dormant here: max [NO]/[NO]_e = {z.max_a_quench:.3f} < 1 — NO stays sub-equilibrium;")
    print("  the dropped clamp is correct-on-principle, dormant-on-numbers at this lean point.)")


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
    # The tables carry unicode (Δ, ·, ≡, ≈); force UTF-8 so `python main.py` renders
    # on any console (a stock Windows cp1252 console would otherwise crash on them).
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except (AttributeError, ValueError):
        pass

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

    print_forkb_table(FLIGHT)

    print_equilibrium_table(FLIGHT)

    print_nox_table(FLIGHT)

    print_zoning_table(FLIGHT)

    print_rql_table(FLIGHT)

    print_finite_quench_table(FLIGHT)

    plot_ts_diagram(ideal, real, FLIGHT)


if __name__ == "__main__":
    main()
