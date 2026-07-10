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
    Gas, JetMixing, Unmixedness, MixingPDF, QuenchPDF, PocketQuenchPDF, TransportedPDF,
    _products_composition, _equilibrium_composition, _h_molar_A,
    _HF_FUEL_DEFAULT, _M_AIR, _M_CH2, _F_STOICH, _air_mole_fractions,
    _equilibrium_no_fraction, _primary_aft, _thermal_no,
    _quench_trajectory, _quench_no, _bell_interpolator, _beta_pdf_nodes_weights, _ideal_bell_ei,
    _two_stream_ceiling, _transport_variance, _pdf_mean_ei,
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


def print_jet_mixing_table(flight):
    """Rung-11 payoff: the PHYSICAL mixing model — a jet-entrainment quench.

    Rung 10 resolved the quench in time but left "how fast" a FREE knob (τ_q) with an arbitrary
    LINEAR schedule. Rung 11 asks what SETS the quench rate: the dilution air enters through JETS
    IN CROSSFLOW, and the mixing rate scales with the jet momentum-flux ratio J = ρ_j U_j²/(ρ_c
    U_c²). So τ_q = H/(C_e·√J·U_c) is DERIVED from J (a stronger jet penetrates and entrains
    faster → shorter quench), and the linear schedule becomes a decelerating ENTRAINMENT shape.
    "Quick quench" = a high-momentum jet, quantified: EI_NO falls MONOTONICALLY as J rises.

    A MEAN-FIELD model (one well-mixed core): it derives the quench RATE but has NO mixing
    optimum — an over-penetrating jet leaving an un-mixed hot core is a spatial-VARIANCE effect,
    deferred to rung 12. Still a pure diagnostic: bit-for-bit rung 6 (opt-in via `mixing`).
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
    hf = eq.hf_fuel_molar if eq.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
    ng = 80

    print("\nPhysical mixing (rung 11): what SETS the quench rate — jets in crossflow. τ_q is no")
    print("longer a free knob; it is DERIVED from the jet momentum-flux ratio J. 'Quick' = strong jet.")
    print(f"  Design point: Tt3={Tt3:.0f} K, Tt4={Tt4:.0f} K, p={p/1e5:.1f} bar, overall "
          f"far={far:.4f} (φ={far/_F_STOICH:.2f})")

    # Build the (τ_q-independent) trajectory ONCE for the rich primary and reuse it across the
    # J-sweep and the schedule-shape contrast (both only change the β↔t map, not the trajectory).
    phi_p = 1.5
    far_p = phi_p * _F_STOICH
    alpha = far / far_p
    T_p = _primary_aft(far_p, p, Tt3, hf)
    comp_p = _equilibrium_composition(far_p, T_p, p)
    nox = _thermal_no(comp_p, T_p, p, 3e-3, far_p)
    n0 = alpha * nox.x_no * sum(comp_p.values())
    tab = _quench_trajectory(comp_p, T_p, alpha, far, Tt3, p, ngrid=ng)   # built ONCE, reused

    # (1) The J-sweep: a stronger jet → shorter derived τ_q → escapes the stoich peak → less NO.
    print(f"\n  Rich primary φ_p={phi_p}: derive τ_q from J (H=0.10 m, U_c=75 m/s, C_e=0.20; "
          "decelerating n=2):")
    print(f"  {'J':>5} {'τ_q ms':>8} {'EI_NO g/kg':>11}   jet")
    print("  " + "-" * 44)
    for J in (4.0, 9.0, 16.0, 25.0, 49.0, 100.0):
        m = JetMixing(J=J, C_e=0.20, shape_n=2.0)
        q = _quench_no(comp_p, T_p, alpha, far, Tt3, p, n0, m.tau_q, tab=tab, schedule=m.schedule)
        tag = "weak — mixes slow" if J <= 4 else ("strong (RQL target)" if J >= 49 else "")
        print(f"  {J:>5.0f} {m.tau_q*1e3:>8.3f} {q['ei']:>11.4g}   {tag}")
    print("  EI_NO falls MONOTONICALLY as J rises — a strong jet quenches fast and escapes the")
    print("  stoich peak. No optimum: a mean-field model has no unmixedness (that is rung 12).")

    # (2) The schedule-shape contrast at fixed J (same derived τ_q): a decelerating entrainment
    #     clears the EARLY/low-β stoich crossing faster than rung-10's linear schedule.
    J = 25.0
    tq = JetMixing(J=J, C_e=0.20).tau_q   # C_e matches the shape rows below (τ_q depends only on J,H,U_c,C_e)
    print(f"\n  Schedule shape at J={J:.0f} (τ_q={tq*1e3:.3f} ms fixed; stoich crossing is at LOW β):")
    print(f"  {'shape n':>8} {'EI_NO g/kg':>11}   entrainment")
    print("  " + "-" * 44)
    for n_shape in (0.5, 1.0, 2.0, 3.0):
        m = JetMixing(J=J, C_e=0.20, shape_n=n_shape)
        q = _quench_no(comp_p, T_p, alpha, far, Tt3, p, n0, m.tau_q, tab=tab, schedule=m.schedule)
        kind = ("accelerating" if n_shape < 1 else "LINEAR (rung 10)" if n_shape == 1
                else "decelerating (real)")
        print(f"  {n_shape:>8.1f} {q['ei']:>11.4g}   {kind}")
    print("  A DECELERATING entrainment (n>1: fast near the jet, slowing as the gradient collapses)")
    print("  clears the early stoich crossing faster → LESS NO than the linear schedule. So IF")
    print("  entrainment decelerates (as gradient-collapse suggests), rung 10's linear schedule")
    print("  over-predicted the spike by ~2× — within the shape uncertainty (n=0.5 would go the")
    print("  other way; the shape is a residual choice, so the SIGN of 'conservative' rides on it).")


def print_unmixedness_table(flight):
    """Rung-12 payoff: SPATIAL UNMIXEDNESS — the variance layer that turns the curve back up.

    Rung 11 was MEAN-FIELD (one well-mixed core), so its J-sweep is MONOTONE: a stronger jet only
    ever re-makes less NO. Real dilution jets have an OPTIMUM at the Holdeman group C=(S/H)√J≈2.5 —
    UNDER- and OVER-penetration BOTH leave a hot near-stoich core that MISSES the fast jet and
    lingers. Rung 12 adds that core as a SECOND stream (the bulk stays the mean-field reference); off-
    optimum the CORE worsens two ways — its fraction w(C) AND its dwell τ_core grow, kinked at C_opt.
    EI_NO = (1−w)·EI(τ_mean) + w·EI(τ_core): it FALLS to a minimum AT C_opt then RISES — the Holdeman
    optimum, recovered as an EMISSIONS optimum. Still a pure diagnostic: bit-for-bit rung 6 (opt-in
    via `unmixedness`).
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
    ng = 80

    print("\nSpatial unmixedness (rung 12): the variance rung 11 missed. A mean field says 'stronger")
    print("jet → less NO' forever; a real jet has an OPTIMUM. Give the quench TWO streams and the NO-")
    print("vs-J curve turns back UP — the Holdeman dilution-jet optimum, recovered AT C_opt≈2.5.")
    print(f"  Design point: Tt3={Tt3:.0f} K, Tt4={Tt4:.0f} K, p={p/1e5:.1f} bar, overall "
          f"far={far:.4f} (φ={far/_F_STOICH:.2f})")

    # (1) The turn-up: mean-field bulk (rung 11, still falling) vs the two-stream total (turns up AT C_opt).
    u = Unmixedness(S=0.0625)          # J_opt=(C_opt·H/S)²=16 — the uniformity optimum, mid-sweep
    J_opt = (u.C_opt * JetMixing(J=1.0).H / u.S) ** 2
    print(f"\n  Rich primary φ_p=1.5; unmixedness S={u.S} m (H=0.10 → uniformity J_opt={J_opt:.0f}), "
          f"τ_res={u.tau_res*1e3:.1f} ms, C_opt={u.C_opt}:")
    print(f"  {'J':>5} {'C':>6} {'w_core':>7} {'EI bulk':>8} {'EI 2-stream':>12}   note")
    print("  " + "-" * 58)
    rows = []
    for J in (4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 100.0):
        s = eq.zoned_nox(far, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=J, C_e=0.20, shape_n=2.0),
                         unmixedness=u, quench_ngrid=ng)
        rows.append((J, s.ei_no_unmixed))
        note = ("under-penetrates" if J < J_opt else "OPTIMUM (w→0)" if abs(J - J_opt) < 1e-9
                else "over-penetrates")
        print(f"  {J:>5.0f} {s.C_holdeman:>6.2f} {s.w_core:>7.3f} {s.ei_no_quenched:>8.4g} "
              f"{s.ei_no_unmixed:>12.4g}   {note}")
    imin = min(range(len(rows)), key=lambda i: rows[i][1])
    print(f"  EI_NO falls THEN rises — the minimum lands AT J={rows[imin][0]:.0f} (C=C_opt=2.5), the")
    print("  recovered Holdeman optimum. The mean-field 'EI bulk' (rung 11) is still falling at J=100")
    print("  — the un-mixed CORE (which misses the jet, quenches at an ABSOLUTE dwell so it survives")
    print("  strong jets, and lingers LONGER the further from C_opt) is what turns the TOTAL back up.")

    # (2) The optimum sits AT the Holdeman GROUP C_opt: shrink S → J_opt moves → the EI-min moves WITH it.
    print(f"\n  The optimum sits AT the Holdeman group C=(S/H)√J=C_opt — shrink S and it moves ((H/S)²):")
    print(f"  {'S (m)':>7} {'J_opt':>6} {'EI-min at J':>12}")
    print("  " + "-" * 30)
    for S in (0.0625, 0.0500):
        uu = Unmixedness(S=S)
        eis = []
        for J in (9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 100.0):
            s = eq.zoned_nox(far, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=J, C_e=0.20, shape_n=2.0),
                             unmixedness=uu, quench_ngrid=ng)
            eis.append((J, s.ei_no_unmixed))
        Jmin = min(eis, key=lambda e: e[1])[0]
        Jopt = (uu.C_opt * 0.10 / S) ** 2
        print(f"  {S:>7.4f} {Jopt:>6.0f} {Jmin:>12.0f}")
    print("  The EI-min lands ON J_opt for both spacings — the kinked unmixedness PINS it at C_opt, so")
    print("  the emissions optimum shifts as (H/S)² exactly (16→25). 'A stronger jet is better' holds")
    print("  ONLY up to the Holdeman optimum; past it, over-penetration strands a hot core and NO climbs.")


def print_mixing_pdf_table(flight):
    """Rung-13 payoff: the RESOLVED MIXING PDF — the continuous distribution that replaces rung-12's
    two hand-tuned streams, and a MECHANISM SEPARATION.

    Rung 12 split the flow into two lumps (bulk + core). Rung 13 resolves the whole mixture-fraction
    distribution as a mean-preserving β-PDF whose ONE width — the segregation g(C) — rides the same
    Holdeman kink. Integrating the IDEAL bell EI(φ) over it, ⟨EI⟩ = ∫ EI(φ(ξ))·P_β(ξ;ξ̄,g) dξ.

    The lesson, stated correctly (NOT "convexity/Jensen"): NO is sharply PEAKED at stoich, so
    segregation RAISES the mean NO whenever the mean is OFF-stoich (our lean dilution mean) — and
    REVERSES sign at a stoich mean. So the emissions minimum pins AT C_opt (perfect mixing → uniform
    lean → ≈0), both immediate flanks lifting by ORDERS. This ISOLATES the COMPOSITION mechanism and
    drops the finite-quench dwell chain, so — unlike rung 12 — it CANNOT climb on the over-penetration
    flank: past a hump ⟨EI⟩(g) descends (the β-PDF goes bimodal). Composition variance pins the
    optimum LOCATION; the DWELL effect (rung 12) makes the climb; combining them is rung 15. Still a
    pure diagnostic: bit-for-bit rung 6 (opt-in via `pdf`).
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
    hf = eq.hf_fuel_molar if eq.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
    xibar = far / (1.0 + far)

    print("\nResolved mixing PDF (rung 13): rung 12 used TWO hand-tuned streams; resolve the WHOLE")
    print("mixture-fraction distribution as a mean-preserving β-PDF instead. Integrating the ideal NO")
    print("bell over it, the emissions minimum pins AT the Holdeman optimum from a CONTINUOUS PDF — but")
    print("the over-penetration CLIMB is gone: that was rung-12's DWELL effect, which this drops.")
    print(f"  Design point: Tt3={Tt3:.0f} K, Tt4={Tt4:.0f} K, p={p/1e5:.1f} bar, overall "
          f"far={far:.4f} (φ={far/_F_STOICH:.2f}, LEAN); ξ̄={xibar:.4f}")

    # Build the IDEAL bell ONCE (J-independent) — the J-sweep just re-weights it by the PDF.
    bell = _bell_interpolator(p, Tt3, hf, 3e-3, n_bell=200)

    def pdf_ei(g_seg):
        if g_seg <= 1e-9:
            return bell(xibar)
        nodes, w = _beta_pdf_nodes_weights(xibar, g_seg, n_quad=200)
        return sum(wi * bell(x) for wi, x in zip(w, nodes))

    # (1) The convexity jump + the sign reversal — the correct framing of the lesson.
    ei_mean_lean = bell(xibar)
    xistoich = _F_STOICH / (1.0 + _F_STOICH)
    ei_mean_stoich = bell(xistoich)
    print(f"\n  The mechanism — a PEAKED bell × an OFF-stoich mean (NOT generic convexity):")
    print(f"  {'g (segregation)':>16} {'⟨EI⟩ lean mean':>15} {'⟨EI⟩ stoich mean':>17}")
    print("  " + "-" * 52)
    for g_seg in (0.0, 0.05, 0.10, 0.20):
        nodes_s, w_s = (None, None) if g_seg <= 1e-9 else _beta_pdf_nodes_weights(xistoich, g_seg, n_quad=200)
        ei_s = ei_mean_stoich if g_seg <= 1e-9 else sum(wi * bell(x) for wi, x in zip(w_s, nodes_s))
        print(f"  {g_seg:>16.2f} {pdf_ei(g_seg):>15.4g} {ei_s:>17.4g}")
    print(f"  LEAN mean (EI(mean)={ei_mean_lean:.2e}): segregation RAISES ⟨EI⟩ by ~10⁴–10⁵× (the stoich-")
    print(f"  ward tail samples the bell peak). STOICH mean (EI(mean)={ei_mean_stoich:.1f}): it LOWERS it")
    print("  (mass moves OFF the peak) — the sign reversal that proves it is 'peaked×off-mean', not Jensen.")

    # (2) The J-sweep: min PINNED at C_opt; both flanks up; the far flank DESCENDS (humped ⟨EI⟩(g)).
    S = 0.0625
    H = JetMixing(J=1.0).H
    J_opt = (2.5 * H / S) ** 2
    print(f"\n  J-sweep (S={S} m → uniformity J_opt={J_opt:.0f}, C_opt=2.5); ⟨EI⟩ over the β-PDF:")
    print(f"  {'J':>5} {'C':>6} {'g(C)':>6} {'⟨EI⟩ (g/kg)':>12}   note")
    print("  " + "-" * 60)
    pdf = MixingPDF(S=S)
    for J in (4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 100.0):
        C = pdf.C(JetMixing(J=J))
        g_seg = pdf.segregation(C)
        ei = pdf_ei(g_seg)
        note = ("OPTIMUM (g→0, ≈0)" if abs(J - J_opt) < 1e-9
                else "under-penetrates" if J < J_opt
                else "over-penetrates")
        print(f"  {J:>5.0f} {C:>6.2f} {g_seg:>6.3f} {ei:>12.4g}   {note}")
    print(f"  The minimum is a sharp NOTCH pinned AT J={J_opt:.0f} (C=C_opt): perfect mixing → uniform lean")
    print("  → ≈0 NO. Both immediate flanks lift by orders (segregation). But UNLIKE rung 12 the far")
    print("  over-penetration flank DESCENDS, not climbs — ⟨EI⟩(g) is humped (the β-PDF goes bimodal to")
    print("  pure-air + rich, both off the stoich peak). Composition variance pins the optimum LOCATION;")
    print("  rung-12's DWELL effect makes the climb; carrying the PDF through the quench (rung 15) unites")
    print("  them. (The ≈0 minimum here drops the bulk NO floor — that is the same rung-15 scope boundary.)")


def print_nozzle_flow_table(flight):
    """Rung-14 payoff: EQUILIBRIUM-vs-FROZEN NOZZLE FLOW — the rung-6 cycle-side seam, and where
    rung-10's dropped equilibrium clamp finally earns its keep.

    The production nozzle FREEZES the station-4 mixture through the expansion. Real nozzle flow lies
    between FROZEN (chemistry infinitely slow) and EQUILIBRIUM/SHIFTING (infinitely fast): as the
    exhaust cools, CO/H₂/OH/O/H recombine to CO₂/H₂O, releasing chemical energy → a higher V9. Two
    complementary lessons, both mirroring the rung-10 clamp:
      • THRUST (major species) — the bracket is DORMANT at the cool lean design point (dissociation
        ≈ 0) and EARNS ITS KEEP hot (a Tt4 sweep: ~0.006% → ~0.44%).
      • NO / THE CLAMP — on the SAME cooling path equilibrium NO COLLAPSES, so the frozen exhaust NO
        is wildly super-equilibrium and rung 7's DROPPED clamp fires (max_a ≫ 1, vs rung 10's 0.677).
    A pure diagnostic: bit-for-bit rung 6 (the production nozzle stays frozen).
    """
    print("\nEquilibrium-vs-frozen nozzle flow (rung 14): the production nozzle FREEZES the station-4")
    print("mixture; a real nozzle lets it re-equilibrate as it cools — CO/H₂/OH/O/H recombine and give")
    print("back thrust. Frozen = a LOWER bound, equilibrium = an UPPER bound; the real nozzle sits between.")

    # (1) The THRUST bracket — a Tt4 sweep: dormant at the design point, earns its keep hot.
    print("\n  Thrust bracket (Tt4 sweep, real-loss cycle) — V9 recovered by a shifting expansion:")
    print(f"  {'Tt4 [K]':>8} {'CO/(CO+CO2)':>12} {'V9 froz':>9} {'V9 equil':>9} {'ΔV9 m/s':>8} {'ΔV9 %':>8}")
    print("  " + "-" * 62)
    for Tt4 in (1500.0, 1800.0, 2000.0, 2200.0):
        eq = Gas.reacting_equilibrium()          # fresh gas per burn condition (the section freezes Tt4)
        r = build_turbojet(eq, PI_C, Tt4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
        st4, st9 = r.stations["4"], r.stations["9"]
        nf = eq.nozzle_flow(st4.far, st4.Tt, st4.pt, st9.Tt, st9.pt, r.p9)
        print(f"  {Tt4:>8.0f} {nf.co_fraction_entry:>12.2e} {nf.V9_frozen:>9.2f} "
              f"{nf.V9_equilibrium:>9.2f} {nf.dV9:>8.3f} {nf.dV9_frac*100:>7.4f}%")
    print("  At the metallurgically-capped design point (Tt4=1500 K, lean φ≈0.4) dissociation is ~5e-6")
    print("  and the bracket is DORMANT (~0.006%) — like the clamp, negligible HERE. A hot combustor")
    print("  dissociates ~1% of the carbon; recombination in the nozzle then buys ~0.4% more exhaust")
    print("  velocity. (ΔV9 is the nozzle quantity; the specific-THRUST gain is ~1.2–1.35× larger via")
    print("  the M0=0.85 ram term — ΔF/F = ΔV9/(V9−V0/(1+f)).) The real nozzle sits between the bounds.")

    # (2) The CLAMP corollary — the design-point exhaust, cooling through the nozzle.
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4, st9 = real.stations["3"], real.stations["4"], real.stations["9"]
    zn = eq.zoned_nox(st4.far, st3.Tt, st4.Tt, st4.pt, phi_primary=1.0, tau=3e-3)   # ICAO-band exhaust NO
    nf = eq.nozzle_flow(st4.far, st4.Tt, st4.pt, st9.Tt, st9.pt, real.p9, x_no_frozen=zn.x_no_mix)
    print(f"\n  The dropped clamp earns its keep (design point: Tt9={st9.Tt:.0f} K → T9={nf.T9_frozen:.0f} K):")
    print(f"  equilibrium NO collapses {nf.x_no_e_entry*1e6:.0f} → {nf.x_no_e_exit*1e6:.2f} ppm on cooling "
          f"({nf.no_collapse_ratio:.0f}× — frozen-NO-independent).")
    print(f"  A realistic zoned exhaust carries EI_NO≈{zn.ei_no:.0f} g/kg ({zn.x_no_mix*1e6:.0f} ppm), FROZEN")
    print(f"  through the nozzle → it is {nf.max_a:.0f}× super-equilibrium at the exit (max_a={nf.max_a:.0f}).")
    print("  Rung 7's cNO≤cNOe clamp would DELETE that surplus — a plausible-but-wrong low number with")
    print("  every assert green. Rung 10 DROPPED the clamp and proved it DORMANT on the combustor quench")
    print("  (max_a=0.677<1); HERE, in the near-stoich exhaust-cooling rung 10 flagged, it FIRES. That is")
    print("  the whole reason the clamp was dropped 'on principle' — this nozzle is where it bites.")


def print_pdf_quench_table(flight):
    """Rung-15 payoff: the PDF THROUGH the finite quench — the two mixing mechanisms COMBINED.

    Rungs 12–13 kept the mechanisms isolated: rung 12 had the DWELL (the over-penetration flank climbs)
    with a two-lump split; rung 13 had the resolved COMPOSITION β-PDF (the optimum pinned AT C_opt) but
    on the ideal bell — dropping the quench, so its optimum collapsed to ≈0 and its far flank descended.
    Rung 15 is the additive combination: ⟨EI⟩₁₅ = EI_bulk_quench(τ_mean) [the rung-11 mean-field FLOOR]
    + D(u)·⟨EI_bell⟩(g) [the rung-13 β-PDF integral × a rung-12 dwell]. The ≈0 rung-13 floor BECOMES
    the finite bulk NO, and the descending far flank CLIMBS again — while the nonlinear bell keeps the
    STOICH-MEAN SIGN REVERSAL a lumped-dwell rung 12 cannot. Still a pure diagnostic: bit-for-bit rung 6.
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
    hf = eq.hf_fuel_molar if eq.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
    xibar = far / (1.0 + far)
    ng = 80

    print("\nPDF through the finite quench (rung 15): rungs 12–13 isolated the two mixing mechanisms —")
    print("the DWELL (rung 12: the far flank climbs) and the resolved COMPOSITION β-PDF (rung 13: the")
    print("optimum pinned AT C_opt, but ≈0 because it dropped the quench). Rung 15 COMBINES them: the ≈0")
    print("floor becomes the finite bulk NO, and the descending rung-13 far flank climbs again.")
    print(f"  Design point: Tt3={Tt3:.0f} K, Tt4={Tt4:.0f} K, p={p/1e5:.1f} bar, overall "
          f"far={far:.4f} (φ={far/_F_STOICH:.2f}, LEAN); ξ̄={xibar:.4f}")

    # Build the shared τ-independent quench trajectory ONCE (the mean-field floor per J reuses it) and
    # the ideal bell ONCE (J-independent) — a J-sweep then re-weights both without rebuilding.
    phi_p = 1.5
    far_p = phi_p * _F_STOICH
    alpha = far / far_p
    T_p = _primary_aft(far_p, p, Tt3, hf)
    comp_p = _equilibrium_composition(far_p, T_p, p)
    nox = _thermal_no(comp_p, T_p, p, 3e-3, far_p)
    n0 = alpha * nox.x_no * sum(comp_p.values())
    tab = _quench_trajectory(comp_p, T_p, alpha, far, Tt3, p, ngrid=ng)
    bell = _bell_interpolator(p, Tt3, hf, 3e-3, n_bell=200)

    def floor_ei(J):                                        # term 1 — the rung-11 mean-field bulk quench
        m = JetMixing(J=J, C_e=0.20, shape_n=2.0)
        return _quench_no(comp_p, T_p, alpha, far, Tt3, p, n0, m.tau_q, tab=tab, schedule=m.schedule)["ei"]

    def bell_pdf(g_seg):                                    # ⟨EI_bell⟩(g) — the rung-13 integral (term 2 core)
        if g_seg <= 1e-9:
            return bell(xibar)
        nodes, w = _beta_pdf_nodes_weights(xibar, g_seg, n_quad=200)
        return sum(wi * bell(x) for x, wi in zip(nodes, w))

    # The J-sweep: rung-13 (ideal bell, ≈0 min + descending far flank) vs rung-15 (finite floor + climb).
    qp = QuenchPDF(S=0.0625)
    J_opt = _j_opt_from(qp)
    print(f"\n  J-sweep (S={qp.S} m → uniformity J_opt={J_opt:.0f}, C_opt={qp.C_opt}); rich primary φ_p={phi_p}:")
    print(f"  {'J':>5} {'C':>6} {'g':>6} {'EI floor':>9} {'⟨EI⟩13':>8} {'⟨EI⟩15':>8}   note")
    print("  " + "-" * 60)
    rows = []
    for J in (4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 100.0, 225.0, 400.0):
        C = qp.C(JetMixing(J=J))
        g_seg = qp.segregation(C)
        floor = floor_ei(J)
        ei13 = bell_pdf(g_seg)                              # rung-13 ideal-bell PDF (no floor, no dwell)
        ei15 = floor + qp.dwell_factor(C, 3e-3) * ei13      # rung-15 = floor + D(u)·⟨EI_bell⟩
        rows.append((J, ei15))
        note = ("OPTIMUM (g→0)" if abs(J - J_opt) < 1e-9 else "under" if J < J_opt else "over")
        print(f"  {J:>5.0f} {C:>6.2f} {g_seg:>6.3f} {floor:>9.4g} {ei13:>8.4g} {ei15:>8.4g}   {note}")
    imin = min(range(len(rows)), key=lambda i: rows[i][1])
    print(f"  The rung-15 minimum is a FINITE floor ({rows[imin][1]:.3g} g/kg) pinned AT J={rows[imin][0]:.0f}")
    print("  (C=C_opt) — NOT rung-13's ≈0. Both immediate flanks lift; and the far over-penetration flank")
    print("  CLIMBS again (J=100→400: the dwell restored, surviving strong jets) where rung-13's ⟨EI⟩13")
    print("  DESCENDS (bimodal PDF). The over-flank is non-monotone: the composition convexity jump near")
    print("  C_opt hands off to the dwell climb far out — BOTH parents' fingerprints, in one curve.")

    # The discriminator: term 2's nonlinear bell reverses sign at a stoich mean (a lumped dwell cannot).
    xistoich = _F_STOICH / (1.0 + _F_STOICH)

    def bell_pdf_at(mean_xi, g_seg):
        if g_seg <= 1e-9:
            return bell(mean_xi)
        nodes, w = _beta_pdf_nodes_weights(mean_xi, g_seg, n_quad=200)
        return sum(wi * bell(x) for x, wi in zip(nodes, w))

    print(f"\n  Why it is NOT rung 12 in disguise — the STOICH-MEAN SIGN REVERSAL (term 2's nonlinear bell):")
    print(f"  {'g':>6} {'⟨EI_bell⟩ lean':>15} {'⟨EI_bell⟩ stoich':>17}")
    print("  " + "-" * 40)
    for g_seg in (0.0, 0.05, 0.10, 0.20):
        print(f"  {g_seg:>6.2f} {bell_pdf_at(xibar, g_seg):>15.4g} {bell_pdf_at(xistoich, g_seg):>17.4g}")
    print("  Segregation RAISES the bell integral at a LEAN mean but LOWERS it at a STOICH mean (mass off")
    print("  the peak). A dwell-only 'PDF through the quench' rides the ~linear EI_quench, so its variance")
    print("  has the WRONG sign and cannot reverse — this reversal certifies term 2 is genuine composition")
    print("  work. (Carrying the FULL per-pocket trajectory, not the bell×dwell-ratio, is the rung-16 seam.)")


def print_pocket_quench_table(flight):
    """Rung-16 payoff: the PDF through the finite quench, PER POCKET — retiring rung-15's linearised dwell.

    Rung 15's term 2 = D(u)·⟨EI_bell⟩ rescaled each pocket's CONSTANT-T bell NO by a SCALAR dwell ratio
    D(u)=τ_core/τ_ref — exact only while EI ∝ τ, which IGNORES that a lingering pocket COOLS. Rung 16
    carries EACH rich-of-mean pocket through its OWN finite quench (`_quench_no` at τ_core), so the dwell
    acts INSIDE the cooling chemistry. The result is a REFINEMENT, not a reversal: term 2 grows
    SUBLINEARLY in τ_core (the cooling), which ERODES rung-15's over-penetration far flank into
    near-degeneracy with the C_opt notch. The composition excess still → 0 AT C_opt (the notch survives);
    the global-min LOCATION is NOT claimed (it is within the quadrature/tail/C_e ambiguity). Pure
    diagnostic: bit-for-bit rung 6.
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
    hf = eq.hf_fuel_molar if eq.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
    xibar = far / (1.0 + far)
    ng, tau = 32, 3e-3                                      # coarse (illustrative panel) — SHAPE, not digits

    print("\nPDF through the finite quench, PER POCKET (rung 16): rung 15 combined composition + dwell,")
    print("but LINEARISED the dwell — it scaled a CONSTANT-T bell by D(u)=τ_core/τ_ref (exact only while")
    print("EI ∝ τ). Rung 16 carries EACH pocket through its OWN quench, so the dwell acts INSIDE the")
    print("cooling chemistry: term 2 goes SUBLINEAR and the far over-penetration flank ERODES.")
    print(f"  Design point: Tt3={Tt3:.0f} K, Tt4={Tt4:.0f} K, p={p/1e5:.1f} bar, overall "
          f"far={far:.4f} (φ={far/_F_STOICH:.2f}, LEAN); ξ̄={xibar:.4f}")

    # term 1 — the rung-11 mean-field bulk quench pool + its shared trajectory (built ONCE)
    phi_p = 1.5
    far_p = phi_p * _F_STOICH
    alpha = far / far_p
    T_p = _primary_aft(far_p, p, Tt3, hf)
    comp_p = _equilibrium_composition(far_p, T_p, p)
    n0 = alpha * _thermal_no(comp_p, T_p, p, tau, far_p).x_no * sum(comp_p.values())
    tab = _quench_trajectory(comp_p, T_p, alpha, far, Tt3, p, ngrid=ng)

    def floor_ei(J):
        m = JetMixing(J=J, C_e=0.20, shape_n=2.0)
        return _quench_no(comp_p, T_p, alpha, far, Tt3, p, n0, m.tau_q, tab=tab, schedule=m.schedule)["ei"]

    # term 2, rung 15 (LINEARISED) — the ideal bell × the scalar dwell factor
    bell = _bell_interpolator(p, Tt3, hf, tau, n_bell=120)

    def term2_15(qp, C):
        g_seg = qp.segregation(C)
        if g_seg <= 1e-9:
            mb = bell(xibar)
        else:
            nodes, w = _beta_pdf_nodes_weights(xibar, g_seg, n_quad=120)
            mb = sum(wi * bell(x) for x, wi in zip(nodes, w))
        return qp.dwell_factor(C, tau) * mb

    # term 2, rung 16 (PER POCKET) — the bank of per-pocket trajectories, built ONCE (τ_core-independent)
    NB, NQ = 48, 80
    xi_max = (2.0 * _F_STOICH) / (1.0 + 2.0 * _F_STOICH)
    xi_grid = [xi_max * (i + 0.5) / NB for i in range(NB)]
    bank = []
    for xi in xi_grid:
        fl = xi / (1.0 - xi)
        if fl < far or fl / _F_STOICH > 2.0 + 1e-9 or fl <= 0.0:
            bank.append(("b", _ideal_bell_ei(fl, p, Tt3, hf, tau)))
            continue
        try:
            T_pk = _primary_aft(fl, p, Tt3, hf)
        except AssertionError:
            bank.append(("b", 0.0))
            continue
        al = far / fl
        cp = _equilibrium_composition(fl, T_pk, p)
        n0k = al * _thermal_no(cp, T_pk, p, tau, fl).x_no * sum(cp.values())
        bank.append(("q", (cp, T_pk, al, n0k, _quench_trajectory(cp, T_pk, al, far, Tt3, p, ngrid=ng))))

    def term2_16(pqp, C):
        g_seg = pqp.segregation(C)
        tau_core = pqp.core_dwell(C)
        vals = []
        for kind, payload in bank:
            if kind == "q":
                cp, T_pk, al, n0k, tabk = payload
                vals.append(_quench_no(cp, T_pk, al, far, Tt3, p, n0k, tau_core, tab=tabk)["ei"])
            else:
                vals.append(payload)

        def qb(x):
            if x <= xi_grid[0]:
                return vals[0]
            if x >= xi_grid[-1]:
                return 0.0
            lo, hi = 0, NB - 1
            while hi - lo > 1:
                mid = (lo + hi) // 2
                if xi_grid[mid] <= x:
                    lo = mid
                else:
                    hi = mid
            t = (x - xi_grid[lo]) / (xi_grid[hi] - xi_grid[lo])
            return vals[lo] + t * (vals[hi] - vals[lo])

        if g_seg <= 1e-9:
            return qb(xibar)
        nodes, w = _beta_pdf_nodes_weights(xibar, g_seg, n_quad=NQ)
        return sum(wi * qb(x) for x, wi in zip(nodes, w))

    qp = QuenchPDF(S=0.0625)
    pqp = PocketQuenchPDF(S=0.0625)
    J_opt = _j_opt_from(pqp)
    print(f"\n  J-sweep (S={pqp.S} m → J_opt={J_opt:.0f}, C_opt={pqp.C_opt}); rich primary φ_p={phi_p}, C_e=0.20:")
    print(f"  {'J':>5} {'C':>6} {'g':>6} {'EI floor':>9} {'⟨EI⟩15':>8} {'⟨EI⟩16':>8} {'erosion':>8}   note")
    print("  " + "-" * 66)
    for J in (16.0, 36.0, 64.0, 144.0, 225.0, 400.0, 625.0):
        C = pqp.C(JetMixing(J=J))
        g_seg = pqp.segregation(C)
        floor = floor_ei(J)
        ei15 = floor + term2_15(qp, C)
        ei16 = floor + term2_16(pqp, C)
        ero = "" if J <= J_opt + 1e-9 else f"{(ei15 - ei16) / ei15 * 100:+.0f}%"
        note = "OPTIMUM (g→0)" if abs(J - J_opt) < 1e-9 else "under" if J < J_opt else "over"
        print(f"  {J:>5.0f} {C:>6.2f} {g_seg:>6.3f} {floor:>9.4g} {ei15:>8.4g} {ei16:>8.4g} {ero:>8}   {note}")

    # the mechanism + the flattened climb
    C_lo, C_hi = pqp.C(JetMixing(J=144.0)), pqp.C(JetMixing(J=625.0))
    r15 = term2_15(qp, C_hi) / term2_15(qp, C_lo)
    r16 = term2_16(pqp, C_hi) / term2_16(pqp, C_lo)
    e15_lo, e15_hi = floor_ei(144.0) + term2_15(qp, C_lo), floor_ei(625.0) + term2_15(qp, C_hi)
    e16_lo, e16_hi = floor_ei(144.0) + term2_16(pqp, C_lo), floor_ei(625.0) + term2_16(pqp, C_hi)
    print(f"\n  THE MECHANISM — term 2 vs dwell (J=144→625): rung-15 ×{r15:.2f} (LINEAR, = the dwell ratio)")
    print(f"  vs rung-16 ×{r16:.2f} (SUBLINEAR — each pocket COOLS through its quench). That cooling erodes")
    print(f"  the far flank: rung-15 CLIMBS ({e15_lo:.3g}→{e15_hi:.3g}, +{(e15_hi/e15_lo-1)*100:.0f}%) but")
    print(f"  rung-16 is FLAT ({e16_lo:.3g}→{e16_hi:.3g}, {(e16_hi/e16_lo-1)*100:+.0f}%) — the over-penetration")
    print("  basin erodes into near-degeneracy with the C_opt notch (which SURVIVES: term 2 → 0 at C_opt).")
    print("  HONEST SCOPE: which of the two near-degenerate wells is the GLOBAL min is NOT claimed — it flips")
    print("  sign across the β-PDF quadrature (~5%), the φ>2 tail, and the C_e regime (2%→21% over 0.20→0.15).")
    print("  Rung 16 quantifies rung-15's linearisation error; it does not relocate the optimum. (Clamp")
    print("  DORMANT here, max_a<1 — the difference is cooling, not super-eq rollover.)")


def print_exhaust_clamp_table(flight):
    """Rung-17 payoff: the exhaust-NO clamp through the combustor-mixing-fidelity ladder.

    A rung-14 corollary from the RICH side. Rung 14 fired the dropped clamp on the φ_p=1.0 MIXED-OUT
    exhaust NO (a≈250) — the zoned-vs-unzoned axis. Rung 17 asks the same clamp question at the RICH
    φ_p=1.5 RQL primary, through THREE levels of combustor-mixing fidelity, carried through the SAME
    nozzle collapse to T9: MIXED-OUT (rung 8) reads DORMANT (a<1) — mixing-out HIDES the super-eq NO —
    while the BULK QUENCH (rung 11) and PER-POCKET (rung 16) models FIRE. The load-bearing content: the
    ORDERING a_mixed≤a_bulk≤a_pocket is STRUCTURAL (the quench only adds NO; the excess is additive) and
    a_mixed<1 is robust — the FIRING (a>1) holds in-band but is NOT universal (a fast quench J→∞ drives
    a_bulk→a_mixed<1, the rung-10 τ_q→0 reduce). The identity a_pocket/a_bulk = rung-16's station-4 gap
    is algebra (the nozzle cancels); every magnitude rides on un-pinned scales. Pure diag: bit-for-bit rung 6.
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4, st9 = real.stations["3"], real.stations["4"], real.stations["9"]
    far, Tt3, Tt4, p = st4.far, st3.Tt, st4.Tt, st4.pt
    J, phi_p = 225.0, 1.5
    ng, ns = 32, 400                                       # coarse (illustrative panel) — SHAPE, not digits
    pq = PocketQuenchPDF(S=0.0625, n_bell=40, n_quad=120)

    print("\nExhaust-NO clamp through the mixing-fidelity ladder (rung 17): rung 14 fired the dropped")
    print("clamp on the φ_p=1.0 MIXED-OUT exhaust NO. At the RICH φ_p=1.5 RQL primary the mixed-out NO is")
    print("deceptively LOW — so the crude shortcut reads the clamp DORMANT — but the dilution re-making")
    print("(rung 11) and the near-stoich β-PDF pockets (rung 16) put NO back, frozen super-eq in the nozzle.")

    s = eq.exhaust_no_clamp(far, Tt3, Tt4, p, st9.Tt, st9.pt, real.p9, phi_primary=phi_p,
                            mixing=JetMixing(J=J, C_e=0.20, shape_n=2.0), pocket_quench=pq,
                            quench_ngrid=ng, quench_nsteps=ns)
    print(f"\n  Design point (rich φ_p={phi_p}, J={J:.0f}): exhaust cools Tt9={st9.Tt:.0f} K → T9={s.T9:.0f} K,")
    print(f"  equilibrium NO collapses {s.no_collapse_ratio:.0f}× → the common clamp denominator x_no_e(T9).")
    print(f"  {'mixing-fidelity model':>26} {'exhaust x_no':>13} {'a=[NO]/[NO]_e(T9)':>18}  verdict")
    print("  " + "-" * 70)
    print(f"  {'MIXED-OUT (rung 8)':>26} {s.x_no_mixed_out:>13.3e} {s.a_mixed_out:>18.3f}  "
          f"{'DORMANT — hides the NO' if s.a_mixed_out < 1 else 'fires'}")
    print(f"  {'BULK QUENCH (rung 11)':>26} {s.x_no_bulk_quench:>13.3e} {s.a_bulk_quench:>18.3f}  "
          f"{'FIRES — re-making' if s.a_bulk_quench > 1 else 'dormant'}")
    print(f"  {'PER-POCKET (rung 16)':>26} {s.x_no_pocket:>13.3e} {s.a_pocket:>18.3f}  "
          f"{'FIRES — segregation lifts the mean' if s.a_pocket > 1 else 'dormant'}")
    print(f"  The ladder is MONOTONE in fidelity: a_mixed<1<a_bulk<a_pocket. The pocket/bulk ratio "
          f"{s.gap_pocket_over_bulk:.2f} =")
    print(f"  rung-16's station-4 gap EXACTLY (the nozzle denominator cancels — algebra, a witnessed no-op).")

    # The rung-14 contrast — the SAME mixed-out-through-the-nozzle construction at φ_p=1.0 vs 1.5.
    def a_mixed(phi):
        zn = eq.zoned_nox(far, Tt3, Tt4, p, phi, 3e-3)
        return eq.nozzle_flow(far, Tt4, p, st9.Tt, st9.pt, real.p9, x_no_frozen=zn.x_no_mix).max_a

    a10, a15 = a_mixed(1.0), a_mixed(1.5)
    print(f"\n  The rung-14 contrast (same mixed-out-through-the-nozzle construction): φ_p=1.0 → a={a10:.0f}")
    print(f"  (FIRES, rung 14) but φ_p=1.5 → a={a15:.3f} (DORMANT). The RICH primary hides the NO the")
    print(f"  mixed-out model never made — the same dropped-clamp lesson from the other side.")

    # Scale-sensitivity — the ORDERING is structural; the magnitudes move (firing in-band, not universal).
    print("\n  Scale-sensitivity (C_e sweep, J=225) — the ORDERING is structural; magnitudes + the gap move:")
    print(f"  {'C_e':>6} {'a_mixed':>8} {'a_bulk':>8} {'a_pocket':>9} {'gap':>6}   ladder holds?")
    for C_e in (0.15, 0.20):
        sc = s if C_e == 0.20 else eq.exhaust_no_clamp(
            far, Tt3, Tt4, p, st9.Tt, st9.pt, real.p9, phi_primary=phi_p,
            mixing=JetMixing(J=J, C_e=C_e, shape_n=2.0), pocket_quench=pq,
            quench_ngrid=ng, quench_nsteps=ns)
        ok = "YES" if sc.ladder_monotone and sc.a_mixed_out < 1 else "NO"
        print(f"  {C_e:>6.2f} {sc.a_mixed_out:>8.3f} {sc.a_bulk_quench:>8.2f} {sc.a_pocket:>9.2f} "
              f"{sc.gap_pocket_over_bulk:>6.2f}   {ok}")
    print("  HONEST SCOPE: the ORDERING (structural) + mixed-out dormancy are the certified claim. The")
    print("  FIRING (a>1) is IN-BAND, not universal — a fast quench (J→∞) drives a_bulk→a_mixed<1 (the")
    print("  rung-10 τ_q→0 reduce). a_bulk, a_pocket AND the gap ride on un-pinned scales (C_e, τ_res, H,")
    print(f"  J); the clamp is DORMANT at station 4 (max_a={s.max_a_quench:.2f}) — the super-equilibrium is a")
    print("  NOZZLE effect. Rung 17 is a synthesis of rungs 11/16/14, not new physics: it shows the crude")
    print("  mixed-out shortcut is UNCONSERVATIVE across the rich RQL operating band.")


def print_transported_variance_table(flight):
    """Rung-18 payoff: the TRANSPORTED-variance closure — what a 0-D variance equation CAN and CANNOT
    derive. Rungs 12-17 IMPOSE the β-PDF width as a kinked g(C)=k_g·|ln(C/C_opt)|. Rung 18 solves g(C)
    from a variance DECAY ODE dg/dt=−C_φ·ω(C)·g out of a DERIVED two-stream ceiling, through the
    rung-13 ideal bell. The load-bearing result is NEGATIVE: a 0-D transport CANNOT derive the C_opt
    optimum (mean-field ω(J) ⇒ monotone g(J); only a SPATIAL ω(C=(S/H)√J) gives an optimum — the
    spacing S injected). What it DOES add: the DERIVED ceiling (g_max=0.3 was ~4.4× too big), the
    RESIDUAL floor (optimum elevated off the well-mixed value), and the SMOOTH basin vs the kink corner.
    Pure diagnostic: bit-for-bit rung 6.
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
    hf = eq.hf_fuel_molar if eq.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
    xibar = far / (1.0 + far)
    tau, phi_p = 3e-3, 1.5
    NB, NQ = 48, 80
    cfg = TransportedPDF(S=0.0625, n_bell=NB, n_quad=NQ)
    kink = MixingPDF(S=0.0625, n_bell=NB, n_quad=NQ)
    J_opt = _j_opt_from(cfg)

    print("\nTransported-variance closure (rung 18): the deferred 'transported PDF' seam — derive g(C)")
    print("from a mixture-fraction variance equation instead of imposing the kink. The HONEST result is")
    print("a LIMIT: a 0-D transport CANNOT derive the C_opt optimum (it is irreducibly SPATIAL — the")
    print("rung-11/12 variance seam). What it CAN add: a DERIVED ceiling, a RESIDUAL floor, a smooth basin.")

    g_ceiling = _two_stream_ceiling(far, phi_p)
    point = _ideal_bell_ei(far, p, Tt3, hf, tau)
    print(f"\n  Design point: Tt3={Tt3:.0f} K, Tt4={Tt4:.0f} K, overall far={far:.4f} (φ={far/_F_STOICH:.2f},"
          f" LEAN); rich primary φ_p={phi_p}")
    print(f"  DERIVED two-stream ceiling g_ceiling=(ξ_p−ξ̄)/(1−ξ̄)={g_ceiling:.4f} — set by φ_p, NOT a knob;")
    print(f"  rung-13's free g_max=0.30 is {0.30/g_ceiling:.1f}× larger (which is what let rung-13's ⟨EI⟩(g)")
    print(f"  reach its humped/descending regime). Residual floor g(C_opt)=g_ceiling·exp(−Da_opt)="
          f"{g_ceiling*math.exp(-cfg.Da_opt):.4f} > 0.")

    # (1) THE NEGATIVE RESULT — a genuine variance ODE: mean-field ω(J) monotone vs spatial ω(C) optimum.
    print("\n  (1) THE NEGATIVE RESULT — integrate the REAL variance ODE, read g at the combustor exit:")
    print(f"  {'J':>5} {'C':>6} {'ω const':>10} {'ω∝√J':>10} {'ω∝J':>10} {'ω(C) spatial':>13}")
    Js = (4.0, 9.0, 16.0, 25.0, 49.0, 100.0, 225.0, 625.0)
    H = JetMixing(J=1.0).H
    cols = {"const": lambda J: 250.0, "sqrtJ": lambda J: 250.0 * math.sqrt(J / 16.0),
            "linJ": lambda J: 250.0 * (J / 16.0)}
    grids = {k: [] for k in cols}
    gcov = []
    for J in Js:
        C = (cfg.S / H) * math.sqrt(J)
        for k, om in cols.items():
            grids[k].append(_transport_variance(g_ceiling, om(J), cfg.tau_mix, c_phi=2.0))
        gcov.append(_transport_variance(g_ceiling, cfg.coverage_omega(C), cfg.tau_mix, c_phi=cfg.C_phi))
        print(f"  {J:>5.0f} {C:>6.2f} {grids['const'][-1]:>10.4f} {grids['sqrtJ'][-1]:>10.4f} "
              f"{grids['linJ'][-1]:>10.4f} {gcov[-1]:>13.4f}")

    def argmin_note(vals):
        i = min(range(len(vals)), key=lambda k: vals[k])
        if (max(vals) - min(vals)) <= 1e-4 * max(vals):
            return "FLAT — no optimum"
        return (f"interior min J={Js[i]:.0f}" if 0 < i < len(vals) - 1
                else f"monotone (min J={Js[i]:.0f}) — no optimum")
    print(f"  mean-field: const → {argmin_note(grids['const'])}; √J → {argmin_note(grids['sqrtJ'])};"
          f" J → {argmin_note(grids['linJ'])}")
    print(f"  spatial ω(C): {argmin_note(gcov)} — the optimum appears ONLY once S enters via C=(S/H)√J.")

    # (2) the SHAPE — the smooth elevated basin vs the kink notch, through the SAME ideal bell.
    print(f"\n  (2) THE SHAPE (S={cfg.S} m → J_opt={J_opt:.0f}); transported g vs the imposed kink,"
          " through the ideal bell:")
    print(f"  {'J':>5} {'C':>6} {'g_kink':>7} {'g_tr':>7} {'⟨EI⟩kink':>9} {'⟨EI⟩tr':>8}   note")
    print("  " + "-" * 64)
    for J in (9.0, 16.0, 25.0, 49.0, 100.0, 225.0):
        C = cfg.C(JetMixing(J=J))
        gk = kink.segregation(C)
        gt, _ = cfg.segregation(C, far, phi_p)
        eik = _pdf_mean_ei(far, Tt3, p, hf, tau, max(gk, 1e-12), n_bell=NB, n_quad=NQ)
        eit = _pdf_mean_ei(far, Tt3, p, hf, tau, gt, n_bell=NB, n_quad=NQ)
        note = "← C_opt: kink DIVES to floor; transp ELEVATED" if abs(J - J_opt) < 1 else ""
        print(f"  {J:>5.0f} {C:>6.2f} {gk:>7.4f} {gt:>7.4f} {eik:>9.5f} {eit:>8.5f}   {note}")
    print(f"  The kink TOUCHES the well-mixed floor (≈{point:.1e}) at C_opt (g→0); the transported basin")
    print("  sits ELEVATED (residual unmixedness). Both minima are AT C_opt — but the LOCATION is IMPOSED")
    print("  via ω(C) (proven in panel 1), while the SHARPNESS was the kink's artifact (any mixing rate")
    print("  rounds a corner). Transport tightens the closure (derived ceiling + residual floor) without")
    print("  over-claiming the one thing 0-D cannot reach: the spatial/CFD PDF stays the deferred ceiling.")


def _j_opt_from(cfg):
    """The uniformity optimum J_opt where C=(S/H)√J_opt = C_opt (H=0.10, the JetMixing default)."""
    return (cfg.C_opt * JetMixing(J=1.0).H / cfg.S) ** 2


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

    print_jet_mixing_table(FLIGHT)

    print_unmixedness_table(FLIGHT)

    print_mixing_pdf_table(FLIGHT)

    print_nozzle_flow_table(FLIGHT)

    print_pdf_quench_table(FLIGHT)

    print_pocket_quench_table(FLIGHT)

    print_exhaust_clamp_table(FLIGHT)

    print_transported_variance_table(FLIGHT)

    plot_ts_diagram(ideal, real, FLIGHT)


if __name__ == "__main__":
    main()
