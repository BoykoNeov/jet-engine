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

from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, OffDesignMatcher, MapMatcher, ComponentMap, SpoolTransient,
    CombustorTransient, build_two_spool_turbojet, TwoSpoolMatcher, TwoSpoolMapMatcher,
    TwoSpoolTransient, TwoSpoolBleedMatcher, TwoSpoolFuelTransient, SurgeLimiter,
    AsymmetricLag, VariableStatorMatcher, StageStackMatcher, ram_recovery,
    ScheduledStatorTransient, StatorSchedule, IncidenceLimiter, StatorBleedMatcher,
    ScheduledBleedTransient, BleedSchedule, LimitedBleedTransient, BleedLimiter,
    LaggedBleedTransient, TwoLagCascadeTransient, CrossLoopCascadeTransient,
    ThreeLoopCascadeTransient, StatorLimiter,
    ReferenceSplitTransient, StatorIncidenceLimiter, CrossSplitTransient,
    FullSplitTransient,
)
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, Unmixedness, MixingPDF, QuenchPDF, PocketQuenchPDF, TransportedPDF, SpatialPDF,
    SpatialDwellPDF, PromptNO, _products_composition, _equilibrium_composition, _h_molar_A,
    _HF_FUEL_DEFAULT, _M_AIR, _M_CH2, _F_STOICH, _air_mole_fractions,
    _equilibrium_no_fraction, _primary_aft, _thermal_no, _super_eq_o_multiplier,
    _quench_trajectory, _quench_no, _bell_interpolator, _beta_pdf_nodes_weights, _ideal_bell_ei,
    SpatialLocalPDF, _two_stream_ceiling, _transport_variance, _pdf_mean_ei, _spatial_segregation,
    _spatial_dwell_field, _spatial_local_field, FiniteRate, FreezeOut, _tau_chem_recomb, _Ru,
    NOFreezeOut, _tau_no_destroy, CoupledNOFreezeOut, _tau_no_exact,
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


def print_finite_rate_nozzle_table(flight):
    """Rung-25 payoff: FINITE-RATE nozzle chemistry — the Damköhler flow BETWEEN rung-14's bounds,
    which turns the two-bound bracket into a THREE-state picture.

    Rung 14 gave FROZEN (Da→0) and reversible EQUILIBRIUM (Da→∞) and named the seam between them.
    Resolving it, the physics refuses a clean two-bound interpolation: the nozzle is fed the
    frozen-in station-4 mixture, which arrives SUPER-EQUILIBRIUM (equilibrated hot at Tt4, frozen to
    Tt9). A REAL (irreversible) flow must re-equilibrate that entry, and that relaxation is
    irreversible EVEN AT INFINITE RATE. So there are three states:
      (F) frozen             — Da→0, rung-14 lower bound (the exact reduce).
      (I) irreversible-fast  — Da→∞, the ATTAINABLE ceiling (closed form: const-(H,pt9) entry
                               re-equilibration then reversible shifting). Sits STRICTLY BELOW ...
      (R) reversible-shift   — ... rung-14's upper bound, a strict UNREACHABLE ceiling.
    The (R−I) gap is the 'sliver of entry irreversibility' rung 14 named — dormant lean, ~7% of the
    bracket hot. A pure diagnostic: bit-for-bit rung 6 (the production nozzle stays frozen).
    """
    print("\nFinite-rate nozzle chemistry (rung 25): the real Damköhler flow BETWEEN rung-14's bounds.")
    print("It reveals a THREE-state picture — the super-equilibrium frozen entry re-equilibrates")
    print("IRREVERSIBLY, so even infinitely-fast chemistry (I) falls SHORT of the reversible ceiling (R).")

    # (1) The three-state ceilings — a Tt4 sweep: dormant lean, both gaps earn their keep hot.
    print("\n  Three-state ceilings (Tt4 sweep, real-loss cycle):")
    print(f"  {'Tt4 [K]':>8} {'V9 (F)':>9} {'V9 (I)':>9} {'V9 (R)':>9} "
          f"{'attain I-F':>11} {'unreach R-I':>12}")
    print("  " + "-" * 66)
    for Tt4 in (1500.0, 1800.0, 2000.0, 2200.0):
        eq = Gas.reacting_equilibrium()
        r = build_turbojet(eq, PI_C, Tt4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
        st4, st9 = r.stations["4"], r.stations["9"]
        fr = eq.finite_rate_nozzle(st4.far, st4.Tt, st4.pt, st9.Tt, st9.pt, r.p9, FiniteRate(Da=3.0))
        print(f"  {Tt4:>8.0f} {fr.V9_frozen:>9.2f} {fr.V9_irrev_fast:>9.2f} {fr.V9_reversible:>9.2f} "
              f"{fr.attainable_gap/fr.V9_frozen*100:>10.4f}% {fr.unreachable_gap/fr.V9_frozen*100:>11.4f}%")
    print("  Both gaps COLLAPSE at the cool lean design point (dissociation ~5e-6, no entry non-equilibrium)")
    print("  and EARN THEIR KEEP hot — rung-14's own arc. The unreachable (R−I) gap is the entry")
    print("  re-equilibration irreversibility: a genuine sliver lean, ~7% of the bracket at 2200 K.")

    # (2) The interior Da sweep at the HOT anchor — the payoff curve filling the ATTAINABLE bracket.
    print("\n  The finite-rate flow filling the attainable [F, I] bracket (Tt4=2200 K):")
    print(f"  {'Da':>7} {'V9 finite':>10} {'attain-filled':>13} {'dS≥0':>9} {'CO exit':>10}")
    print("  " + "-" * 54)
    eq = Gas.reacting_equilibrium()
    r = build_turbojet(eq, PI_C, 2200.0, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st4, st9 = r.stations["4"], r.stations["9"]
    for Da in (0.3, 1.0, 3.0, 10.0, 30.0):
        fr = eq.finite_rate_nozzle(st4.far, st4.Tt, st4.pt, st9.Tt, st9.pt, r.p9, FiniteRate(Da=Da))
        print(f"  {Da:>7.1f} {fr.V9_finite:>10.3f} {fr.finite_filled:>12.3f}  "
              f"{fr.dS_finite:>9.3e} {fr.co_fraction_finite_exit:>10.2e}")
    print("  V9(Da) climbs monotonically toward the ATTAINABLE ceiling (I) — never the reversible (R).")
    print("  Da is a CARTOON knob (a normalized Damköhler, not an Arrhenius τ_chem): the interior curve")
    print("  rides on it; the three-state picture and the (R−I) gap do NOT. Entropy production dS≥0 (2nd")
    print("  law) peaks at intermediate Da; freeze-out (τ_chem(T) quenching the recombination) is the")
    print("  deferred seam. The entry irreversibility is a consequence of the FROZEN turbine — a shifting")
    print("  turbine would shrink it (the rung-26 seam).")


def print_freeze_out_nozzle_table(flight):
    """Rung-26 payoff: FREEZE-OUT — an anchored recombination clock that resolves WHERE the nozzle
    chemistry quenches, and shows the freeze point MOVE with Tt4.

    Rung 25's Da is a normalized cartoon that slides the whole expansion uniformly; a CONSTANT Da
    cannot show freeze-out. Rung 26 replaces it with a LOCAL Da(T,p)=τ_res/τ_chem(T,p) from the
    ANCHORED GRI-Mech 3.0 sink H+OH+M→H2O+M (Ea=0, n=−2 — zero new constants). As the gas expands,
    the density² collapse (c_tot²∝(p/T)²) outruns k(T)'s cooling acceleration, so τ_chem grows,
    Da_local falls through 1, and the composition FREEZES — at a point that walks downstream as Tt4
    climbs. A pure diagnostic: bit-for-bit rung 6 (the production nozzle stays frozen).
    """
    print("\nFreeze-out (rung 26): an ANCHORED recombination clock over rung-25's exact integrator.")
    print("Rung-25's Da is uniform (no freeze-out); here Da_local(T,p)=τ_res/τ_chem falls through 1")
    print("PARTWAY down the nozzle — and the freeze point MOVES with Tt4 (the physics a constant Da")
    print("cannot express). Chemistry is GRI-Mech 3.0 verbatim; the only knob left is geometric (L).")

    # (1) The moving freeze point — a Tt4 sweep. Dormant lean (frozen from entry), walks downstream hot.
    print("\n  The freeze point walking downstream with Tt4 (real self-quenching integrator, L=0.5 m):")
    print(f"  {'Tt4 [K]':>8} {'Da_entry':>9} {'Da_exit':>9} {'s_freeze':>9} "
          f"{'CO_entry':>10} {'CO_exit':>10} {'frozen@entry':>13}")
    print("  " + "-" * 74)
    for Tt4 in (1500.0, 1650.0, 1800.0, 2000.0, 2200.0):
        eq = Gas.reacting_equilibrium()
        r = build_turbojet(eq, PI_C, Tt4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
        st4, st9 = r.stations["4"], r.stations["9"]
        fz = eq.freeze_out_nozzle(st4.far, st4.Tt, st4.pt, st9.Tt, st9.pt, r.p9, FreezeOut())
        print(f"  {Tt4:>8.0f} {fz.Da_entry:>9.3e} {fz.Da_exit:>9.3e} {fz.s_freeze:>9.3f} "
              f"{fz.co_fraction_entry:>10.3e} {fz.co_fraction_freeze_exit:>10.3e} "
              f"{str(fz.frozen_from_entry):>13}")
    print("  Lean (≤1650 K): Da_local<1 throughout ⇒ FROZEN FROM ENTRY (s=0) — an independent derivation")
    print("  of the production frozen nozzle. Hot: the crossing walks downstream (0.118→0.288→0.378) and")
    print("  more CO recombines. The MOTION is the certified rung; the s-VALUES are not (they ride on L).")

    # (2) The kill test — density drives the freeze DESPITE an opposing temperature effect.
    print("\n  Kill test (Tt4=2200 K, standalone clock, x_OH pinned at frozen entry):")
    eq = Gas.reacting_equilibrium()
    r = build_turbojet(eq, PI_C, 2200.0, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st4, st9 = r.stations["4"], r.stations["9"]
    ce = _equilibrium_composition(st4.far, st4.Tt, st4.pt)
    Tt9, pt9, p9 = st9.Tt, st9.pt, r.p9
    T_ex = Tt9 * (p9 / pt9) ** ((1.30 - 1.0) / 1.30)
    tau_res = 0.5 / (0.6 * r.V9)
    da = lambda tau: tau_res / tau
    da_entry = da(_tau_chem_recomb(ce, Tt9, pt9))
    da_real = da(_tau_chem_recomb(ce, T_ex, p9))
    da_killT = da(_tau_chem_recomb(ce, T_ex, p9, kill_T=Tt9))
    c_M_in = pt9 / (_Ru * Tt9) / 1.0e6
    da_killp = da(_tau_chem_recomb(ce, T_ex, p9, kill_M=c_M_in))
    print(f"    entry Da={da_entry:6.3f}  →  real exit Da={da_real:6.3f}  (T {Tt9:.0f}→{T_ex:.0f} K, "
          f"p {pt9/1e3:.0f}→{p9/1e3:.0f} kPa)")
    print(f"    kill T (k pinned, density alone): Da={da_killT:6.3f}  → STILL FREEZES  (density did it)")
    print(f"    kill p ([M] pinned, T alone)    : Da={da_killp:6.3f}  → NO FREEZE, Da RISES  (k accelerates)")
    print("  Density drives freeze-out DESPITE an opposing T effect — the OPPOSITE sign to Arrhenius")
    print("  intuition (Ea=0 ⇒ no thermal barrier), which is what refutes rung-25's 'unanchored Arrhenius")
    print("  trap' framing: the rate is anchored (GRI-Mech), and the mechanism runs the other way.")


def print_no_freeze_out_table(flight):
    """Rung-27 payoff: NO FREEZE-OUT — is the frozen-NO assumption every NO number has carried since
    rung 7 (and the rung-14/17 clamp reads OFF) actually EARNED?

    Rung 26 showed the MAJOR pool freezes only partway down. Rung 27 applies the SAME anchored-clock /
    local-Da machinery to a NO clock built from rung 7's OWN Zeldovich reverse rates (zero new
    constants) and finds the assumption is DERIVED: Da_NO≪1 from ENTRY at every Tt4 (frozen from entry
    everywhere — unlike the major pool). The kill test INVERTS rung 26's: this clock is Arrhenius
    (k CRATERS on cooling) AND bimolecular, so both its factors AGREE — both DRIVE freezing (vs rung
    26's density-DESPITE-temperature). A pure diagnostic: bit-for-bit rung 6.
    """
    print("\nNO freeze-out (rung 27): is the frozen-NO ASSUMPTION (rung 7 → rung 14/17's clamp) EARNED?")
    print("Rung 26 showed the MAJOR pool freezes only PARTWAY down. The SAME machinery on a NO clock from")
    print("rung 7's OWN Zeldovich reverse rates (zero new constants): exhaust NO is FROZEN FROM ENTRY at")
    print("EVERY Tt4 (Da_NO≪1) — the assumption is DERIVED, on an upper bound (radical-rich frozen pool).")

    # (1) frozen from entry at every Tt4 + the narrowing separation vs rung 26's recombination clock.
    print("\n  Frozen from entry at every Tt4 — the Da_NO-vs-Da_recomb separation NARROWS hot (L=0.5 m):")
    print(f"  {'Tt4 [K]':>8} {'Da_NO@in':>9} {'Da_NO@ex':>9} {'Da_recomb@in':>13} {'separation':>11} "
          f"{'max_a':>7} {'==frozen':>9}")
    print("  " + "-" * 78)
    for Tt4 in (1500.0, 1650.0, 1800.0, 2000.0, 2200.0):
        eq = Gas.reacting_equilibrium()
        r = build_turbojet(eq, PI_C, Tt4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
        st3, st4, st9 = r.stations["3"], r.stations["4"], r.stations["9"]
        s = eq.no_freeze_out_nozzle(st4.far, st3.Tt, st4.Tt, st4.pt, st9.Tt, st9.pt, r.p9, 1.0,
                                    NOFreezeOut())
        fz = eq.freeze_out_nozzle(st4.far, st4.Tt, st4.pt, st9.Tt, st9.pt, r.p9, FreezeOut())
        sep = fz.Da_entry / s.Da_entry
        matches = abs(s.max_a - s.max_a_frozen) / s.max_a_frozen < 1e-2
        print(f"  {Tt4:>8.0f} {s.Da_entry:>9.3e} {s.Da_exit:>9.3e} {fz.Da_entry:>13.3e} {sep:>11.2e} "
              f"{s.max_a:>7.1f} {str(matches):>9}")
    print("  Da_NO<1 EVERYWHERE (3–9 orders clear) at every Tt4 — the frozen-NO assumption HOLDS, unlike")
    print("  rung 26's major pool (frozen only lean, relaxes hot). The clamp max_a == its rung-14/17")
    print("  frozen value to the ≪1 margin: the firing is EARNED. Separation collapses hot (steeply")
    print("  Arrhenius NO vs Ea=0 recombination) but never crosses — no moving freeze point is claimed.")

    # (2) The kill test — the two terms AGREE (both drive): the INVERSION of rung 26.
    print("\n  Kill test (Tt4=2200 K, standalone NO clock on the frozen pool) — the INVERSION of rung 26:")
    eq = Gas.reacting_equilibrium()
    r = build_turbojet(eq, PI_C, 2200.0, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4, st9 = r.stations["3"], r.stations["4"], r.stations["9"]
    s = eq.no_freeze_out_nozzle(st4.far, st3.Tt, st4.Tt, st4.pt, st9.Tt, st9.pt, r.p9, 1.0, NOFreezeOut())
    ce = _equilibrium_composition(st4.far, st4.Tt, st4.pt)
    Tt9, pt9, p9, T9 = st9.Tt, st9.pt, r.p9, s.T9_frozen
    t_in = _tau_no_destroy(ce, Tt9, pt9)
    t_killT = _tau_no_destroy(ce, T9, p9, kill_T=Tt9)          # k pinned → density alone
    t_killc = _tau_no_destroy(ce, T9, p9, kill_c=pt9 / (_Ru * Tt9))  # c_tot pinned → T alone
    print(f"    net τ_NO growth (T {Tt9:.0f}→{T9:.0f} K, p {pt9/1e3:.0f}→{p9/1e3:.0f} kPa): "
          f"×{_tau_no_destroy(ce, T9, p9)/t_in:.2e}  → Da_NO {s.Da_entry:.2e}→{s.Da_exit:.2e}")
    print(f"    kill T (k pinned, density alone): τ ×{t_killT/t_in:6.2f}  → DRIVES freezing")
    print(f"    kill p (c_tot pinned, T alone)  : τ ×{t_killc/t_in:6.2e}  → DRIVES freezing (Arrhenius k craters)")
    print("  BOTH terms AGREE — both drive. Rung 26 had them OPPOSE (density won DESPITE a k that rose on")
    print("  cooling, Ea=0). Here the NO reverse rates carry a large barrier (θ≈20820/24560 K), so k joins")
    print("  density: same nozzle, two anchored clocks, OPPOSITE mechanism structure. NO freezes BECAUSE")
    print("  of temperature; the majors freeze DESPITE it.")


def print_coupled_no_march_table(flight):
    """Rung-28 payoff: the COUPLED march — rung 27's verdict CONFIRMED, both its reasons CORRECTED.

    Rung 27 deferred this with "coupling can ONLY slow NO further". Building it shows that is one-sided:
    coupling to rung 26 couples to ALL of rung 26, including its EXOTHERMIC heat release, which lifts T
    and — this clock being Arrhenius — SPEEDS NO destruction. Two OPPOSING channels. The conclusion
    survives anyway, for a reason rung 27 did not give: depletion is UNBOUNDED, heat release SATURATES.
    And rung 27's OTHER justification (NO arrives super-equilibrium) is false at the entry — the bound
    holds because β<1, not because a≫1. A pure diagnostic: bit-for-bit rung 6, rungs 26/27 untouched.
    """
    print("\nCoupled NO march (rung 28): rung 27's VERDICT confirmed — both its REASONS corrected.")
    print("Rung 27 said the coupling 'can ONLY slow NO further'. But rung-26 recombination is EXOTHERMIC,")
    print("so it also LIFTS T — and an Arrhenius NO clock SPEEDS UP. Two opposing channels, decomposed")
    print("by running the same clock on the two hybrid trajectories (frozen/coupled T × frozen/coupled comp).")

    band = (1500.0, 1650.0, 1800.0, 2000.0, 2200.0, 2400.0)
    states = []
    for Tt4 in band:
        eq = Gas.reacting_equilibrium()
        r = build_turbojet(eq, PI_C, Tt4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
        st3, st4, st9 = r.stations["3"], r.stations["4"], r.stations["9"]
        states.append((Tt4, eq.coupled_no_freeze_out_nozzle(
            st4.far, st3.Tt, st4.Tt, st4.pt, st9.Tt, st9.pt, r.p9, 1.0, CoupledNOFreezeOut())))

    # (1) The two channels, and the fact that the OPPOSING one is not a footnote.
    print("\n  The two channels at the nozzle exit (Da_NO relative to rung 27's), anchored, L=0.5 m:")
    print(f"  {'Tt4 [K]':>8} {'s_frz pool':>11} {'dT_exit':>8} {'ch1 depl':>9} {'ch2 heat':>9} "
          f"{'net':>8} {'|ln2/ln1|':>10} {'deeper':>7}")
    print("  " + "-" * 78)
    for Tt4, s in states:
        print(f"  {Tt4:>8.0f} {s.s_freeze_pool:>11.3f} {s.T9_pool-s.T9_frozen:>8.1f} "
              f"{s.depletion_factor:>9.4f} {s.heat_release_factor:>9.4f} {s.net_factor:>8.4f} "
              f"{s.channel_ratio:>10.3f} {str(s.deeper_frozen):>7}")
    print("  net<1 EVERYWHERE ⇒ rung 27's CONCLUSION (deeper into frozen) is CONFIRMED. But ch2>1 always,")
    print("  and |ln ch2/ln ch1| rises MONOTONICALLY 0.003→0.48: at the hot edge the opposing channel")
    print("  cancels ~HALF the depletion. 'Can ONLY slow NO further' is wrong as a MECHANISM. The net")
    print("  even turns non-monotone (deepest ~2200–2300 K) — that turnaround rides on L and is NOT claimed.")
    print("  INTERLOCK: the coupling bites exactly where rung 26's pool is alive (s_freeze 0→0.39).")

    # (2) Why depletion wins: UNBOUNDED vs SATURATING — the structural argument.
    print("\n  Why depletion wins — drive the pool rate up (Tt4=2200 K): ch1 runs away, ch2 hits a wall:")
    eq = Gas.reacting_equilibrium()
    r = build_turbojet(eq, PI_C, 2200.0, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4, st9 = r.stations["3"], r.stations["4"], r.stations["9"]
    args = (st4.far, st3.Tt, st4.Tt, st4.pt, st9.Tt, st9.pt, r.p9, 1.0)
    print(f"  {'pool rate':>10} {'ch1 depl':>10} {'ch2 heat':>10} {'net':>10}")
    print("  " + "-" * 44)
    for rs in (1.0, 1e1, 1e2, 1e3, 1e4, 1e6):
        s = eq.coupled_no_freeze_out_nozzle(*args, CoupledNOFreezeOut(pool_rate_scale=rs))
        print(f"  {rs:>10.0e} {s.depletion_factor:>10.4f} {s.heat_release_factor:>10.4f} "
              f"{s.net_factor:>10.4f}")
    print("  ch1 → 0 with NO floor (τ_NO∝1/[O],[H], and equilibrium radicals crater on cooling);")
    print("  ch2 SATURATES (heat release is capped by the FINITE frozen-in chemical enthalpy). So at any")
    print("  chemistry faster than anchored, depletion wins by orders — the verdict is STRUCTURAL.")

    # (3) The β repair — rung 27's bound is right, its stated reason is false at the entry.
    print("\n  The β repair — rung 27's clock is an a≫1 limit, but NO arrives SUB-equilibrium:")
    print(f"  {'Tt4 [K]':>8} {'a @entry':>9} {'a @exit':>9} {'sub-eq in':>10} {'β max':>8} "
          f"{'τex/τsurr':>10} {'bound OK':>9}")
    print("  " + "-" * 70)
    for Tt4, s in states:
        print(f"  {Tt4:>8.0f} {s.a_entry:>9.3f} {s.a_exit:>9.2f} {str(s.sub_equilibrium_entry):>10} "
              f"{s.beta_max:>8.3f} {s.tau_ratio_min:>10.4f} {str(s.surrogate_bounds_rate):>9}")
    print("  Rung 27 justified its a≫1 clock with 'exhaust NO arrives SUPER-equilibrium'. At the ENTRY it")
    print("  does NOT (a=0.31–0.61 hot — NO is BELOW the ceiling and tries to FORM); it goes super-eq only")
    print("  as the gas COOLS, at the exit where the clamp is read. Since freeze-FROM-ENTRY is decided at")
    print("  the entry, the premise fails where it is needed. What holds instead is β=R1/(R2+R3) < 1:")
    print("    τ_exact/τ_surr = (1+u)²/[(1+u)²−(1−β²)] > 1  for ALL a,  u=βa")
    print("  ⇒ the surrogate is an UPPER bound on the RATE in BOTH regimes (formation and destruction).")
    print("  Rung 27's NUMBERS are unaffected — only its reasoning is repaired. HONEST MARGIN: β rises to")
    print("  ~0.51 hot, HALF the β=1 threshold — a factor 2, not orders (the weak point, disclosed).")


def print_shifting_turbine_table(flight):
    """Rung-29 payoff: is the FROZEN TURBINE earned? — and why the super-eq RATIO misled us.

    Every rung since 6 freezes the station-4 mixture through the turbine; rungs 14/25 then read that
    frozen pool at the nozzle entry and build the whole (R−I) gap on its super-equilibrium. Rung 29
    brackets the turbine the way rung 14 bracketed the nozzle — frozen vs fully-shifting at the SAME
    shaft-set delta_h. Zero knobs, no rate, so the verdict is RATE-INDEPENDENT: no τ_res can make a
    real turbine exceed the bound. A pure diagnostic — the production turbine still freezes, so the
    cycle stays bit-for-bit rung 6.
    """
    print("\nThe shifting turbine (rung 29): is FREEZING the turbine — assumed since rung 6 — EARNED?")
    print("Bracket it like rung 14 bracketed the nozzle: frozen vs fully-shifting, same shaft work.")
    print("The endpoint is WORK-limited, not pressure-limited: the shaft fixes delta_h (compressor + f")
    print("only), so a shifting turbine reopens NO shaft fixed point — it moves where the flow ENDS UP.")

    band = (1500.0, 1800.0, 2100.0, 2400.0)
    states = []
    for Tt4 in band:
        eq = Gas.reacting_equilibrium()
        r = build_turbojet(eq, PI_C, Tt4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
        st2, st3, st4 = r.stations["2"], r.stations["3"], r.stations["4"]
        delta_h = (eq.h_c(st3.Tt) - eq.h_c(st2.Tt)) / (REAL_LOSSES["eta_m"] * (1.0 + st4.far))
        states.append((Tt4, eq.shifting_turbine(st4.far, st4.Tt, st4.pt, delta_h)))

    # (1) The verdict: the MAXIMUM shift, at four burner temperatures.
    print("\n  The bound (instant chemistry, reversible — nothing real can exceed it):")
    print(f"  {'Tt4 [K]':>8} {'T5 frozen':>10} {'T5 shift':>10} {'dT5 [K]':>9} {'dT5/T5':>10} "
          f"{'dp5/p5':>10} {'earned':>7}")
    print("  " + "-" * 70)
    for Tt4, s in states:
        print(f"  {Tt4:>8.0f} {s.T5_frozen:>10.2f} {s.T5_shifting:>10.2f} {s.dT5:>9.2f} "
              f"{s.dT5_fraction*100:>9.4f}% {s.dp5_fraction*100:>9.4f}% "
              f"{str(s.frozen_turbine_earned):>7}")
    print("  At the design point the freeze is EARNED OUTRIGHT: the maximum conceivable shift moves Tt5")
    print("  by 0.011% — an order BELOW the cycle's own modelling error (eta_t, pi_b are quoted to ~1%).")
    print("  Hot it BITES: 1.9% in Tt5 and 0.47% in pt5 by 2400 K, a 174× growth. So 'the turbine is")
    print("  frozen' is a DESIGN-POINT fact, not a structural one — every rung from 6 up inherits that.")

    # (2) THE RUNG: ratio ≠ energy. The two currencies move in OPPOSITE directions.
    print("\n  Why we expected the opposite — RATIO ≠ ENERGY:")
    print(f"  {'Tt4 [K]':>8} {'super-eq ratio':>15} {'radical inventory':>18} {'dT5/T5':>10}")
    print("  " + "-" * 55)
    for Tt4, s in states:
        print(f"  {Tt4:>8.0f} {s.super_eq_ratio_max:>14.1f}× {s.radical_inventory:>18.3e} "
              f"{s.dT5_fraction*100:>9.4f}%")
    r0, r1 = states[0][1], states[-1][1]
    print(f"  Across the band the RATIO falls ÷{r0.super_eq_ratio_max/r1.super_eq_ratio_max:.0f} while the "
          f"INVENTORY rises ×{r1.radical_inventory/r0.radical_inventory:.0f} and the SHIFT rises "
          f"×{r1.dT5_fraction/r0.dT5_fraction:.0f}.")
    print("  Rungs 25–28 justify the super-equilibrium entry with a RATIO (x_frozen/x_eq, [NO]/[NO]_e —")
    print("  10×, 100×, '3–9 orders'). That ratio is CORRECT for what it measures — KINETIC distance from")
    print("  equilibrium, which is what a RATE question needs. But it is NOT a proxy for exploitable")
    print("  ENTHALPY, which scales with the ABSOLUTE radical INVENTORY (x·n) — and the two ANTI-correlate.")
    print("  109× of almost nothing is still almost nothing: at the lean design point the radicals are")
    print("  ~3e-5 in mole fraction, so complete recombination releases essentially no heat. The ratio is")
    print("  LOUDEST exactly where the shift is most NEGLIGIBLE. A cross-rung correction, not a local one.")
    print("\n  NOT a finding: that a fully-shifted entry collapses rung-25's (R−I) gap to zero. That is")
    print("  STRUCTURAL — an entry pinned at equilibrium has no super-equilibrium left to relax")
    print("  irreversibly, so (R−I)→0 is a tautology. What is worth carrying is the SIZE of the move")
    print("  needed to get there (1.9% in Tt5) and that the design point sits ~170× short of needing it.")


def print_choked_nozzle_table(flight):
    """Rung-30 payoff: is FULL EXPANSION earned? — the convergent nozzle chokes.

    Every thrust number since rung 2 expands the nozzle fully to p0, which at the design point
    means M9 = 1.86 — SUPERSONIC, i.e. silently a converging-diverging nozzle. A fixed
    CONVERGENT nozzle (the standard subsonic choice, and the fixed throat rung 31 needs) can
    reach only M9 = 1 and then chokes, leaving the jet underexpanded. Bracket the two the way
    rung 29 bracketed the turbine. A pure diagnostic: the production nozzle stays ideal, so the
    cycle remains bit-for-bit rung 6.
    """
    print("\nThe choked convergent nozzle (rung 30): is FULL EXPANSION — assumed since rung 2 — EARNED?")
    print("The shipped nozzle expands fully to p0 (M9 = 1.86, SUPERSONIC — physically a C-D nozzle).")
    print("A fixed convergent nozzle chokes at M9 = 1 and leaves the jet UNDEREXPANDED (p9 = p* > p0).")

    ideal = build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    gas = Gas.reacting_equilibrium()
    conv = build_turbojet(gas, PI_C, TT4, flight.p0, nozzle_convergent=True, **REAL_LOSSES).run(flight, 1.0)
    f = conv.stations["4"].far
    R = gas.R_t_at(f)
    p0 = flight.p0

    mom_i = (1.0 + f) * ideal.V9 - ideal.V0                 # ideal: p9 = p0, no pressure term
    mom_c = (1.0 + f) * conv.V9 - conv.V0
    p_thrust = (1.0 + f) * R * conv.T9 * (1.0 - p0 / conv.p9) / conv.V9

    print(f"\n  Design point Tt4={TT4:.0f} K, pi_c={PI_C:.0f}, M0={flight.M0}: nozzle entry pt9/p0 = "
          f"{conv.stations['9'].pt / p0:.2f} (critical ~1.85 -> CHOKED)")
    print(f"  {'quantity':>26} {'ideal (full exp)':>18} {'choked convergent':>18}")
    print("  " + "-" * 64)
    print(f"  {'exit pressure p9 [kPa]':>26} {ideal.p9/1000:>18.2f} {conv.p9/1000:>18.2f}")
    print(f"  {'exit Mach M9':>26} {ideal.M9:>18.4f} {conv.M9:>18.4f}")
    print(f"  {'exit velocity V9 [m/s]':>26} {ideal.V9:>18.2f} {conv.V9:>18.2f}")
    print(f"  {'momentum thrust [N·s/kg]':>26} {mom_i:>18.2f} {mom_c:>18.2f}")
    print(f"  {'pressure thrust [N·s/kg]':>26} {0.0:>18.2f} {p_thrust:>18.2f}")
    print(f"  {'specific thrust [N·s/kg]':>26} {ideal.performance.specific_thrust:>18.2f} "
          f"{conv.performance.specific_thrust:>18.2f}")
    print(f"  {'TSFC [kg/(N·s)]':>26} {ideal.performance.tsfc:>18.4e} {conv.performance.tsfc:>18.4e}")

    drop = (ideal.performance.specific_thrust - conv.performance.specific_thrust)
    recov = p_thrust / (mom_i - mom_c)
    print(f"\n  FULL EXPANSION IS NOT EARNED here: specific thrust falls {drop:.1f} N·s/kg "
          f"({100*drop/ideal.performance.specific_thrust:.1f}%), TSFC rises "
          f"{100*(conv.performance.tsfc/ideal.performance.tsfc-1):.1f}%.")
    print(f"  THE FINDING — the pressure term rescues most of it: V9 drops "
          f"{100*(ideal.V9-conv.V9)/ideal.V9:.0f}% and momentum thrust {100*(mom_i-mom_c)/mom_i:.0f}%, but")
    print(f"  exhausting into p0 < p* turns the static-pressure excess into +{p_thrust:.0f} N·s/kg of")
    print(f"  DIRECT pressure thrust — recovering {100*recov:.0f}% of the momentum deficit. That gap between")
    print("  '51% loss' and '6.6% loss' is why high-PR engines fit C-D / variable nozzles, and it is the")
    print("  pressure-thrust term the cycle has carried honestly since rung 2. (Production stays on the")
    print("  ideal nozzle -> cycle unmoved; rung 31 uses this choke to PIN the fixed-throat off-design flow.)")


def print_offdesign_table(flight):
    """Rung-31 payoff: OFF-DESIGN MATCHING — the operating point becomes an OUTPUT.

    Every rung so far specified pi_c and Tt4. Real hardware is FIXED: with the turbine NGV
    and the (rung-30) convergent nozzle both choked, the turbine is pinned and the shaft
    balance hands BACK the compressor — pi_c is no longer a knob but a number the choked
    hardware forces. This is the first STRUCTURAL rung. A separate entry point: the
    production design run is untouched (bit-for-bit rung 6). See docs/rung31-spec.md.
    """
    print("\nOff-design matching (rung 31): with the turbine NGV + convergent nozzle both CHOKED,")
    print("the compressor has NO freedom — pi_c is an OUTPUT slaved to Tt4/(tau_r·T0), not a knob.")

    # Design REFERENCE = the choked-convergent (rung-30) design point (the fixed hardware).
    design = build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, flight.p0,
                            nozzle_convergent=True, **REAL_LOSSES)
    m = OffDesignMatcher(design, flight, 1.0)
    od0 = m.match(flight, TT4)
    print(f"\n  Reduce-to-design: matching at the design point returns pi_c = {od0.pi_c:.6f} "
          f"(input 10), mdot/mdot_R = {od0.mdot_ratio:.6f} — the spine holds by construction.")

    print(f"\n  Running line — throttle sweep (M0={flight.M0}); pi_c is the OUTPUT:")
    print(f"  {'Tt4 [K]':>7} {'pi_c':>7} {'tau_t':>9} {'mdot/mdotR':>11} {'F/mdot':>8} "
          f"{'thrust':>8} {'nozzle':>7}")
    print("  " + "-" * 62)
    for Tt4 in (1500.0, 1300.0, 1100.0, 900.0, 700.0, 600.0):
        od = m.match(flight, Tt4)
        tag = "choked" if od.nozzle_choked else "UNCHOKE"
        print(f"  {Tt4:>7.0f} {od.pi_c:>7.3f} {od.tau_t:>9.6f} {od.mdot_ratio:>11.4f} "
              f"{od.performance.specific_thrust:>8.1f} {od.thrust:>8.1f} {tag:>7}")

    # KILL-TEST the drift's driver: a 3-gas ladder (CPG / variable-cp frozen-comp / reacting),
    # ALL measured over the SAME choked range 1500→800 (both choked). Within a point both throats
    # carry the SAME frozen composition, so R cancels in MFP4/MFP9 — the drift is a gamma_t(T)-CURVE
    # effect that only the frozen-composition gas can isolate.
    def _drift(matcher):
        h, c = matcher.match(flight, 1500.0).tau_t, matcher.match(flight, 800.0).tau_t
        return 100.0 * (h - c) / h
    cpg = Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=1.3,
              cp_t=(1.3 - 1) / 1.3 * 1239.0 * 1.3 / (1.3 - 1),
              R_t=(1.3 - 1) / 1.3 * 1239.0, hPR=42.8e6)
    d_cpg = _drift(OffDesignMatcher(build_turbojet(cpg, PI_C, TT4, flight.p0,
                                                   nozzle_convergent=True, **REAL_LOSSES), flight, 1.0))
    d_tpg = _drift(OffDesignMatcher(build_turbojet(Gas.thermally_perfect(), PI_C, TT4, flight.p0,
                                                   nozzle_convergent=True, **REAL_LOSSES), flight, 1.0))
    d_react = _drift(m)                          # same reacting matcher, same 1500→800 range

    print("\n  THE VERDICT — the choked hardware strips the compressor of freedom: pi_c and mdot")
    print("  ride one fixed running line (a pumping characteristic WITHOUT a compressor map).")
    print("  THE FINDING — the textbook says tau_t is EXACTLY constant, but that is a CPG statement.")
    print(f"  On the real gas tau_t DRIFTS {d_react:.1f}% across the choked throttle range (1500→800 K).")
    print(f"  Kill-test (3-gas ladder, drift over the SAME range): CPG {d_cpg:+.3f}%  |  variable-cp")
    print(f"  frozen-composition {d_tpg:+.2f}%  |  reacting {d_react:+.2f}%. So the gamma_t(T) CURVE")
    print(f"  drives {100*d_tpg/d_react:.0f}% of it (R cancels between the two throats), composition the rest —")
    print("  same species as rung 30's '0.03% is the physics, not error'. Below Tt4≈600 the nozzle")
    print("  UNCHOKES (pt9/p0 < ~1.85) and this two-choke pin is lost — the SUBSONIC-nozzle matching")
    print("  branch (rung 33) takes over there; rung 32 earns the eta curvature the running line holds.")


def print_component_map_table(flight):
    """Rung-32 payoff: COMPONENT-MAP MATCHING — the map re-labels the choke-pinned work.

    Rung 31 closed with 'a pumping characteristic WITHOUT a compressor map'. That over-claimed by
    holding eta_c/eta_t at design. Rung 32 puts a representative compressor + turbine map on the
    matcher: the WORK schedule tau_c(Tt4) stays choke-pinned (map-free), but the map droops eta_c
    off-design, so pi_c and mdot fall BELOW rung 31's constant-eta line — a first-order correction.
    The turbine barely moves (its corrected speed is pinned on a single spool). A diagnostic beside
    the cycle; flat map => rung 31 bit-for-bit. See docs/rung32-spec.md.
    """
    print("\nComponent-map matching (rung 32): rung 31 said the running line needs NO compressor map.")
    print("That over-claimed — it held eta at design. A real map droops eta_c off-design, so pi_c and")
    print("mdot fall BELOW rung 31's line. The choke pins the WORK tau_c (map-free); the map moves pi_c.")

    design = build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, flight.p0,
                            nozzle_convergent=True, **REAL_LOSSES)
    mm = MapMatcher(design, flight, 1.0)
    base = OffDesignMatcher(build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, flight.p0,
                                           nozzle_convergent=True, **REAL_LOSSES), flight, 1.0)

    # Reduce-to-rung-31: flat map returns the rung-31 running line exactly.
    flat = ComponentMap.flat()
    r0 = mm.match(flight, TT4, flat)
    print(f"\n  Reduce-to-rung-31: flat map at design returns pi_c = {r0.pi_c:.6f}, N/N_d = "
          f"{r0.N_ratio:.6f} — rung 31 bit-for-bit (the spine).")

    # The finding: a peaked map (three shapes) droops pi_c/mdot; work tau_c stays map-free.
    cmap = ComponentMap.flow_dominated()
    print(f"\n  Running line with a peaked compressor map (flow-dominated shape); pi_c is the OUTPUT:")
    print(f"  {'Tt4':>6} {'pi_c(r31)':>9} {'pi_c(map)':>9} {'dpi_c':>7} {'dmdot':>7} "
          f"{'eta_c':>7} {'tau_c rel':>9} {'N/Nd':>6}")
    print("  " + "-" * 66)
    for Tt4 in (1500.0, 1300.0, 1100.0, 900.0):
        mo = mm.match(flight, Tt4, cmap)
        ro = base.match(flight, Tt4)
        dpc = 100.0 * (mo.pi_c - ro.pi_c) / ro.pi_c
        dmd = 100.0 * (mo.mdot_air - ro.mdot_air) / ro.mdot_air
        tcr = abs(mo.tau_c - ro.tau_c) / ro.tau_c
        print(f"  {Tt4:>6.0f} {ro.pi_c:>9.4f} {mo.pi_c:>9.4f} {dpc:>6.2f}% {dmd:>6.2f}% "
              f"{mo.eta_c:>7.4f} {tcr:>9.1e} {mo.N_ratio:>6.3f}")

    # Shape robustness of the SIGN, and the turbine-pinned sub-finding.
    droops = []
    for shape in (ComponentMap.flow_dominated(), ComponentMap.pressure_dominated(),
                  ComponentMap.tilted()):
        mo = mm.match(flight, 900.0, shape)
        ro = base.match(flight, 900.0)
        droops.append(100.0 * (mo.pi_c - ro.pi_c) / ro.pi_c)
    steep = mm.match(flight, 900.0, ComponentMap(a=0.25, b=0.05, sigma=0.3, a_t=0.5))

    print("\n  THE FINDING — rung 31's 'without a map' over-claimed. The work tau_c is choke-pinned")
    print(f"  (map-free to ~1e-6), but the map droops eta_c so pi_c/mdot fall below rung 31's line —")
    print(f"  SAME SIGN across 3 map shapes at Tt4=900: dpi_c = "
          f"{droops[0]:.1f}% / {droops[1]:.1f}% / {droops[2]:.1f}% (magnitude shape-dependent, DISCLAIMED).")
    print(f"  SUB-FINDING — the turbine barely moves: its corrected speed nu_t = {steep.nu_t:.4f} stays")
    print(f"  ~1% from design (single-spool N/sqrt(Tt4)), so |d eta_t| = {abs(steep.eta_t-0.90):.1e} even for a")
    print(f"  STEEP turbine map — the compressor is where the map bites. (No surge line modeled.)")


def print_subsonic_matching_table(flight):
    """Rung-33 payoff: the SUBSONIC-NOZZLE matching branch — the decoupling BREAKS.

    Rung 31 pinned the turbine with TWO choked throats: (★) is pure geometry, so tau_t/pi_t are
    constant (machine-const on CPG) — 'the turbine does not know the operating condition changed'.
    That holds ONLY while both throats choke. Below the nozzle-unchoke boundary only the NGV stays
    choked; the nozzle passes a SUBSONIC flow whose throughput MFP(M9) depends on pt9/p0, which moves
    with pi_c. So pi_t re-couples to the compressor. THE RUNG (the inversion of rung 31): the coupling
    is STRUCTURAL (through pi_c), not var-cp, so the subsonic tau_t VARIES even on a CPG gas — where
    rung 31's choked tau_t was machine-constant. A separate entry point; the design run stays rung-6.
    """
    print("\nSubsonic-nozzle matching (rung 33): below Tt4~600 the nozzle UNCHOKES and rung 31's two-choke")
    print("pin (star) is void — only the NGV chokes. pi_t becomes the unknown that matches the NGV-choked")
    print("supply to the subsonic-nozzle demand MFP(M9). The clean rung-31 decoupling BREAKS.")

    design = build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, flight.p0,
                            nozzle_convergent=True, **REAL_LOSSES)
    m = OffDesignMatcher(design, flight, 1.0)
    print(f"\n  Running line across the boundary (M0={flight.M0}); branch is auto-dispatched:")
    print(f"  {'Tt4 [K]':>7} {'branch':>9} {'pi_c':>7} {'tau_t':>9} {'M9':>7} {'F/mdot':>8} {'pt9/p0':>7}")
    print("  " + "-" * 60)
    for Tt4 in (700.0, 600.0, 560.0, 520.0, 480.0, 440.0, 420.0):
        try:
            od = m.match(flight, Tt4)
            print(f"  {Tt4:>7.0f} {od.branch:>9} {od.pi_c:>7.3f} {od.tau_t:>9.6f} {od.M9:>7.4f} "
                  f"{od.performance.specific_thrust:>8.1f} {od.stations['9'].pt/flight.p0:>7.3f}")
        except AssertionError:
            print(f"  {Tt4:>7.0f} {'SUB-IDLE':>9}  (net thrust <= 0: below thrust-neutral idle)")

    # THE RUNG — the CPG contrast: choked tau_t machine-constant, subsonic tau_t VARIES.
    g, cp = 1.3, 1239.0
    cpg = Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp, R_t=(g - 1) / g * cp, hPR=42.8e6)
    mc = OffDesignMatcher(build_turbojet(cpg, PI_C, TT4, flight.p0, nozzle_convergent=True,
                                         **REAL_LOSSES), flight, 1.0)
    ch = [mc.match(flight, t).tau_t for t in (1200.0, 800.0)]            # choked branch, CPG
    # subsonic branch, CPG — the FULL window (unchoke down to thrust-neutral idle).
    sub = [mc.match(flight, t).tau_t for t in (580.0, 540.0, 500.0, 460.0, 440.0)]
    print("\n  THE RUNG — rung 31: 'the turbine does not know the operating condition changed' holds")
    print("  ONLY while both throats choke. On a CPG gas the CHOKED tau_t is constant to machine zero")
    print(f"  (spread {abs(ch[0]-ch[1]):.1e}, rung 31 gate 2), but the SUBSONIC tau_t VARIES "
          f"{100*(max(sub)-min(sub))/max(sub):.2f}%")
    print("  — the coupling runs through pi_c (STRUCTURAL, 1st order), not the gamma_t(T) curve that")
    print("  drove rung 31's 2nd-order drift. So it SURVIVES CPG: the exact inversion of rung 31.")
    print("  (Coupling is to pi_c via pt9/p0, NOT ambient p0 — the cycle is pressure-homogeneous.)")
    print("  Envelope: bounded ABOVE by nozzle-unchoke, BELOW by thrust-neutral idle. Cycle: rung-6 exact.")


def print_spool_transient_table(flight):
    """Rung-34 payoff: THE SPOOL TRANSIENT — N becomes a STATE, not an output.

    Rungs 31-33 solved STEADY operating points, each closed by the shaft power balance. Rung 34
    unbalances it: a real spool has inertia, so a fuel change drives a net torque and N accelerates.
    The shaft balance becomes a DIFFERENTIAL equation and N — which rungs 31-33 computed — becomes
    the STATE. The compressor map runs FORWARD (rungs 31-32 ran it backward), the NGV choke closes
    the flow with NO shaft balance, and the leftover power drives dN/dt in nondimensional time
    s = t/tau_spool. THE FINDING is NOT 'the shape is I-independent' (a tautology in a 1-state
    model) — it is the two-timescale ratio r = tau_fuel/tau_spool: the acceleration excursion above
    the running line is F(r), max at r->0 (an algebraic map property), vanishing as r->inf. That is
    why real engines schedule fuel ramps. A separate entry point; the design run stays rung-6 exact.
    See docs/rung34-spec.md.
    """
    print("\nSpool transient (rung 34): the shaft gets INERTIA, so N becomes a STATE that N-lags a fuel")
    print("change. The compressor map runs FORWARD + NGV choke closes the flow with NO shaft balance;")
    print("the leftover power drives dN/ds. Equilibrium (dN/ds=0) reduces to the rung 31/32 running line.")

    gas = Gas.thermally_perfect()          # fast gas: the transient physics is gas-independent
    shape = ComponentMap.surge_flow()
    st = SpoolTransient(build_turbojet(gas, PI_C, TT4, flight.p0, nozzle_convergent=True,
                                       **REAL_LOSSES), flight, 1.0, comp_map=shape)
    base = OffDesignMatcher(build_turbojet(gas, PI_C, TT4, flight.p0, nozzle_convergent=True,
                                           **REAL_LOSSES), flight, 1.0)

    # Reduce: the equilibrium IS the steady running line (a different closure onto the same point).
    d = st.equilibrium(flight, TT4, ComponentMap.flat())
    print(f"\n  Reduce-to-rung-31: flat-map equilibrium at design returns pi_c = {d['pi_c']:.6f}, "
          f"N/N_d = {d['nu']:.6f} — the running line is the transient's stable attractor.")

    # THE FINDING: peak acceleration excursion above the running line vs r = tau_fuel/tau_spool.
    E0 = st.constant_speed_excursion(flight, 1100.0, 1400.0, shape)
    print(f"\n  THE FINDING — acceleration Tt4 1100->1400, peak excursion ABOVE the running line")
    print(f"  (toward lower surge margin) vs the fuel/spool time ratio r = tau_fuel/tau_spool:")
    print(f"  {'r = tau_fuel/tau_spool':>22} {'peak excursion':>15} {'E/E0':>7}")
    print("  " + "-" * 48)
    print(f"  {'0 (algebraic limit)':>22} {E0*100:>13.2f}% {1.000:>7.3f}")
    for r in (0.2, 0.5, 1.0, 2.0, 5.0):
        E = st.ramp_excursion(flight, 1100.0, 1400.0, r, shape, s_settle=5.0, ds=0.1)["E"]
        print(f"  {r:>22.1f} {E*100:>13.2f}% {E/E0:>7.3f}")

    # Spool-down: fuel cut, N coasts down and crosses the choked->subsonic (rung 33) boundary.
    def sched(s):
        return 900.0 if s <= 0 else (460.0 if s >= 6.0 else 900.0 - (900.0 - 460.0) * (s / 6.0))
    nu0 = st.equilibrium(flight, 900.0, shape)["nu"]
    traj = st.integrate(flight, sched, nu0, 21.0, 0.1, shape)
    flip = next((p for i, p in enumerate(traj) if i and traj[i].branch != traj[i - 1].branch), None)

    print("\n  Direction is shape-robust (accel + / decel - across 3 surge-realistic maps); the r->0")
    print("  step excursion is a MAP property (E/E0->1), the DYNAMICAL content is the ratio — a slow")
    print(f"  ramp (r=5) nearly stays on the line (E/E0={st.ramp_excursion(flight,1100.,1400.,5.,shape,s_settle=5.,ds=0.1)['E']/E0:.2f}). tau_spool=I·w_d^2/P_ref is the ONE disclaimed clock.")
    print(f"  SPOOL-DOWN — fuel cut 900->460: N coasts {nu0:.3f} -> {traj[-1].nu:.3f}, and at s={flip.s:.1f} the")
    print(f"  nozzle UNCHOKES (M9={flip.M9:.3f}), flipping onto rung 33's SUBSONIC branch toward thrust-neutral")
    print(f"  idle (sp. thrust {traj[0].sp_thrust:.0f} -> {traj[-1].sp_thrust:.0f} N·s/kg). Cycle: rung-6 exact.")


def print_fuel_metering_table(flight):
    """Rung 35 — FUEL is the control; Tt4 is an OUTPUT (the fuel-metering picture).

    Rung 34 commanded Tt4(t) by fiat. A real engine meters FUEL, and Tt4 falls out of the burner
    balance against the airflow the spool can CURRENTLY pump. At a frozen spool a fuel step drives
    the airflow DOWN (the NGV passes less corrected mass as Tt4 rises, (1+f) rises), so f spikes and
    Tt4 OVERSHOOTS its steady endpoint before N catches up — the turbine-inlet-temperature (TIT)
    excursion, a SECOND acceleration limit that commanding Tt4 structurally HIDES. And because the
    over-temperature amplifies the airflow deficit, it also ENLARGES rung 34's surge excursion: the
    two acceleration limits are COUPLED, not independent — a cross-rung correction of rung 34.
    Separate entry point; the design run stays rung-6 exact. See docs/rung35-spec.md.
    """
    print("\nFuel metering (rung 35): rung 34 commanded Tt4; a real engine meters FUEL and Tt4 is an")
    print("OUTPUT. At a frozen spool a fuel step starves the airflow, so f = mdot_fuel/mdot_air SPIKES")
    print("and Tt4 OVERSHOOTS — a second acceleration limit (turbine life) that commanding Tt4 hid.")

    gas = Gas.thermally_perfect()          # fast gas: the transient physics is gas-independent
    shape = ComponentMap.surge_flow()
    st = SpoolTransient(build_turbojet(gas, PI_C, TT4, flight.p0, nozzle_convergent=True,
                                       **REAL_LOSSES), flight, 1.0, comp_map=shape)
    LO, HI = 1100.0, 1400.0

    # Reduce: control-invariance — the fuel whose steady point is Tt4 reproduces that exact point.
    eqT = st.equilibrium(flight, HI, shape)
    eqF = st.equilibrium_fuel(flight, eqT["f"] * eqT["mdot_air"], shape)
    print(f"\n  Reduce (control-invariance): fuel = f_eq*mdot_air of the Tt4={HI:.0f} point returns the")
    print(f"  SAME running-line instant (nu {eqT['nu']:.6f} vs {eqF['nu']:.6f}, pi_c {eqT['pi_c']:.4f} vs "
          f"{eqF['pi_c']:.4f}, Tt4_out={eqF['Tt4']:.3f}) — a different closure onto one point.")

    # THE FINDING: fuel control ENLARGES the surge excursion, AND exposes the TIT overshoot.
    cs = st.constant_speed_excursion_fuel(flight, LO, HI, shape)
    e0T = st.constant_speed_excursion(flight, LO, HI, shape)
    print(f"\n  THE FINDING — acceleration Tt4 {LO:.0f}->{HI:.0f}, excursions vs r = tau_fuel/tau_spool.")
    print(f"  E_surge/E_temp are referenced to the running line at the CURRENT speed (E_temp is the")
    print(f"  E_surge analogue); Tt4_pk is the ABSOLUTE peak turbine-inlet temperature (a redline is absolute):")
    print(f"  {'r':>6} {'E_surge Tt4':>13} {'E_surge fuel':>13} {'gap':>7} {'E_temp':>8} {'Tt4_pk (K)':>11}")
    print("  " + "-" * 62)
    print(f"  {'0*':>6} {e0T*100:>12.2f}% {cs['E_surge0']*100:>12.2f}% "
          f"{(cs['E_surge0']-e0T)*100:>6.2f}% {cs['E_temp0']*100:>7.1f}% {cs['Tt4_peak']:>11.0f}")
    for r in (0.3, 1.0, 3.0):
        eT = st.ramp_excursion(flight, LO, HI, r, shape, s_settle=4.0, ds=0.1)["E"]
        ef = st.ramp_excursion_fuel(flight, LO, HI, r, shape, s_settle=4.0, ds=0.1)
        print(f"  {r:>6.1f} {eT*100:>12.2f}% {ef['E_surge']*100:>12.2f}% "
              f"{(ef['E_surge']-eT)*100:>6.2f}% {ef['E_temp']*100:>7.1f}% {ef['Tt4_peak']:>11.0f}")
    print(f"  (* r->0 algebraic limit, no integration.) The r->0 peak Tt4={cs['Tt4_peak']:.0f} K is "
          f"+{(cs['Tt4_peak']/HI-1)*100:.0f}% OVER the {HI:.0f} K target — a TIT excursion commanding")
    print("  Tt4 hid. Fuel control also lifts the surge excursion ABOVE rung 34's: the two limits are")
    print("  COUPLED. Magnitude claim rests on the r->0 STEP (both are steps, unconfounded); the r->inf")
    print("  vanishing is the trend. Sign shape-robust across surge maps; magnitude disclaimed. Cycle: rung-6 exact.")


def print_surge_line_table(flight):
    """Rung 36 — THE SURGE LINE: the excursion gets a boundary to be measured against.

    Rungs 32/34/35 reported the transient excursion as a distance ABOVE THE RUNNING LINE and drew NO
    surge line (a representative efficiency island is not a stability boundary; any margin number
    rides on where you draw it). Rung 36 imposes ONE disclosed constant — a stall flow coefficient
    phi_surge (the map's loading-law peak lands at phi<0 for the surge shapes, so it can't be
    inherited). The MAGNITUDE of every margin is disclaimed; what survives is a SIGN: surge margin is
    thin at LOW power (the running-line phi_op walks down toward the fixed stall floor as throttled —
    CRS Ch. 9). And because the rung-34 constant-speed excursion E0 and the margin SM_N share a
    currency (both pi_c ratios at frozen speed), the low-power burst is most surge-critical on BOTH
    axes: E0 rises AND SM_N falls there. This CONFIRMS + SHARPENS rung 34's implicit worst case (E0 was
    already largest at low power), it does not relocate it. Pure diagnostic; design run stays rung-6
    exact. See docs/rung36-spec.md.
    """
    print("\nSurge line (rung 36): rungs 32/34/35 measured the excursion ABOVE the running line but drew")
    print("NO surge line. Rung 36 imposes a stall flow coeff phi_surge (disclaimed) and finds the SIGN")
    print("that survives it: surge margin is THIN AT LOW POWER, so the binding accel is the low-power burst.")

    gas = Gas.thermally_perfect()          # fast gas: the surge margin lives in the cold-section map
    shape = ComponentMap.surge_flow()
    st = SpoolTransient(build_turbojet(gas, PI_C, TT4, flight.p0, nozzle_convergent=True,
                                       **REAL_LOSSES), flight, 1.0, comp_map=shape)
    PHI_S = 0.65
    cm = shape.with_phi_surge(PHI_S)

    # THE SCHEDULE — SM thin at low power (phi_op walks toward the fixed floor).
    print(f"\n  THE SCHEDULE (phi_surge={PHI_S}, DISCLAIMED level; the falling SIGN is the claim):")
    print(f"  {'Tt4':>5} {'phi_op':>8} {'pi_c':>7} {'SM_N':>8} {'SM_flow':>9}")
    print("  " + "-" * 40)
    for sm in st.surge_margin_schedule(flight, [1500.0, 1300.0, 1100.0, 900.0, 800.0, 700.0], cm):
        print(f"  {sm['Tt4']:>5.0f} {sm['phi_op']:>8.4f} {sm['pi_c']:>7.3f} "
              f"{sm['SM_N']*100:>7.1f}% {sm['SM_flow']*100:>8.0f}%")

    # THE COMPOUNDING — confirmation + sharpening: both axes agree the low-power burst is worst.
    print(f"\n  THE COMPOUNDING (confirm+sharpen) - full-throttle burst to Tt4=1500. E0 = rung-34 constant-N excursion;")
    print(f"  SM_N = steady margin at the START. Surge iff E0>=SM_N (== phi_step<=phi_surge, airtight):")
    print(f"  {'Tt4_lo':>6} {'E0':>7} {'SM_N':>7} {'E0/SM_N':>8} {'verdict':>8}")
    print("  " + "-" * 40)
    for lo in (1400.0, 1200.0, 1000.0, 900.0, 800.0, 700.0):
        b = st.acceleration_binding(flight, lo, 1500.0, cm)
        assert b["reaches_surge"] == b["phi_step_le_surge"]      # currency equivalence, live
        print(f"  {lo:>6.0f} {b['E0']*100:>6.1f}% {b['SM_N']*100:>6.1f}% {b['ratio']:>8.3f} "
              f"{'SURGE' if b['reaches_surge'] else 'ok':>8}")
    print("  E0/SM_N rises as start power falls (E0 UP and SM_N DOWN): low-power burst worst on BOTH axes.")
    print("  Rung 34's E0 was ALREADY largest here (no relocation); SM_N is the new info (the margin consumed).")
    print("  The CROSSING into SURGE rides on the disclaimed phi_surge (E0 is floor-independent) and is NOT")
    print("  claimed; only the trend is. Constant-flow SM is a weak sign-check only. Cycle: rung-6 exact.")


def print_combustor_dynamics_table(flight):
    """Rung 37 — THE TWO INTERNAL CLOCKS: volume-filling CONFIRMS, heat-soak CORRECTS.

    Rungs 34-36 made the shaft the only dynamic element; rung 34 bundled the omitted internal clocks
    into one concession ("no combustor volume-filling, no heat soak ... faster clocks below tau_spool,
    they do not change the r framing"). Rung 37 tests both and they SPLIT: volume-filling (a combustor
    plenum, tau_fill << tau_spool) CONFIRMS the concession — the r->0 peak surge excursion is unmoved
    (== rung-35 E0), its content the STRUCTURAL mdot_c != mdot_NGV decoupling; heat-soak (a metal state
    Tm, tau_soak ~ tau_spool) CORRECTS it — a second STATE makes E = E(r, theta0), history-dependent
    (cold < hot-reslam < adiabatic — the modeled combustor sink is surge-PROTECTED; the cost is the
    accel-time LAG. NOTE: this is the OPPOSITE sign to the operational bodie hazard, which is an unmodeled
    compressor-side channel — so this rung does not reproduce it).
    Both effects default OFF => rung 34/35 bit-for-bit; design run stays rung-6 exact. See docs/rung37-spec.md.
    """
    print("\nCombustor dynamics (rung 37): rung 34 bundled two internal clocks into 'faster clocks below")
    print("tau_spool, they do not change the r framing'. They SPLIT — volume-filling CONFIRMS it (a fast")
    print("clock, peak unmoved), heat-soak CORRECTS it (tau_soak ~ tau_spool: a second STATE, E=E(r,theta0)).")

    gas = Gas.thermally_perfect()          # fast gas: the transient physics is gas-independent
    shape = ComponentMap.surge_flow()
    eng = build_turbojet(gas, PI_C, TT4, flight.p0, nozzle_convergent=True, **REAL_LOSSES)
    LO, HI = 1100.0, 1400.0

    # EFFECT 1 — the PLENUM: peak == E0 (CONFIRMATION), independent of the fill clock; the split.
    print(f"\n  VOLUME-FILLING — frozen-spool fuel step Tt4 {LO:.0f}->{HI:.0f}, plenum clock r_v=tau_fill/tau_spool:")
    print(f"  {'r_v':>6} {'E0 (rung35)':>12} {'plenum peak':>12} {'peak-E0':>10} {'mdot split':>11}")
    print("  " + "-" * 54)
    for r_v in (0.03, 0.1):
        ct = CombustorTransient(eng, flight, 1.0, comp_map=shape, plenum_ratio=r_v)
        r = ct.plenum_frozen_peak(flight, LO, HI, shape)
        print(f"  {r_v:>6.2f} {r['E0']*100:>11.4f}% {r['peak']*100:>11.4f}% "
              f"{r['peak_minus_E0']*100:>+9.5f}% {r['split_max']*100:>10.1f}%")
    print("  The peak lands on rung-35's E0 to machine zero, INDEPENDENT of r_v (a frozen-spool map fact) —")
    print("  volume-filling CONFIRMS the concession. Its content is STRUCTURAL: the ~22% mdot_c != mdot_NGV")
    print("  split is the FIRST rung where the two mass flows differ (rung 34 tied them: pt4 = pi_b*pi_c*pt2).")

    # EFFECT 2 — HEAT-SOAK: cold < hot-reslam < adiabatic; the accel-time LAG.
    ct = CombustorTransient(eng, flight, 1.0, comp_map=shape, soak_gain=0.15, soak_ratio=3.0)
    ad = ct.adiabatic_excursion(flight, LO, HI, shape)
    cold = ct.soak_excursion(flight, LO, HI, "cold", shape)
    hot = ct.soak_excursion(flight, LO, HI, "hot", shape)
    print(f"\n  HEAT-SOAK — accel Tt4 {LO:.0f}->{HI:.0f} (G=0.15, r_m=tau_soak/tau_spool=3.0). E = peak surge")
    print(f"  excursion; t_accel = nondim time to 99% of the speed rise (the thrust-response lag):")
    print(f"  {'theta0':>12} {'E_surge':>9} {'t_accel':>9}")
    print("  " + "-" * 32)
    for tag, d in (("adiabatic", ad), ("cold accel", cold), ("hot reslam", hot)):
        ta = f"{d['t_accel']:.2f}" if d["t_accel"] is not None else ">s_end"
        print(f"  {tag:>12} {d['E_surge']*100:>8.2f}% {ta:>9}")
    print("  cold < hot-reslam < adiabatic: the cold metal's heat sink depresses Tt4_turb -> colder NGV ->")
    print("  more airflow -> AWAY from surge, so this modeled combustor sink is surge-PROTECTIVE (rung 34/35's")
    print("  adiabatic no-soak case is the CEILING); a hot reslam is just the least-protected case. The primary")
    print("  cost is the accel-time LAG (cold ~2.5x slower). E = E(r, theta0), history-dependent — NOT a function")
    print("  of r alone. HONEST SCOPE: this is the OPPOSITE sign to the operational bodie/reslam surge hazard")
    print("  (heat soak moving the working line TOWARD surge — an UNMODELED compressor-side channel); this rung")
    print("  does not reproduce it. Reduce: both OFF => rung 34/35 bit-for-bit (exact dispatch); soak equilibrium")
    print("  == rung 35 (Q=0 at steady). Sign shape/knob-robust; magnitudes disclaimed. Cycle: rung-6 exact.")


def print_two_spool_matching_table(flight):
    """Rung 38 — TWO-SPOOL MATCHING: the triangular cascade (no simultaneous solve).

    Rungs 31-37 are all single-spool. A two-spool turbojet (LPC+LPT / HPC+HPT, no bypass) adds a
    THIRD choked throat that does not exist in any single-spool rung: the LP-turbine NGV (the
    inter-turbine duct, station 45, area A45), between the HP turbine's exit and the LP turbine's
    inlet. With all three throats (A4, A45, A8) choked, rung 31's (*) mass-flow trick applies
    TWICE, chained: tau_HPT is pinned by (A4, A45) alone, tau_LPT by (A45, A8) alone -- both
    independent of either compressor. THE FINDING (corrected from an initial over-claim -- see
    docs/rung38-spec.md "the precise claim"): this is NOT "the LP spool solves independent of the
    HP spool" (eta_HPT demonstrably moves pi_LPC, since it shapes the shared Tt45). What IS airtight
    is narrower: each compressor's OWN isentropic efficiency is a terminal leaf that cannot reach the
    OTHER spool's pressure ratio -- so the two compressor ratios are never bound by a joint (2x2)
    solve, a NO-COMPRESSOR-MAP model artifact (rung-31-before-rung-32's own shape). Scope: fully-
    choked branch only; nozzle-unchoke is a rung-33-shaped follow-on, deliberately not attempted.
    """
    print("\nTwo-spool matching (rung 38): a THIRD choked throat (the LP-turbine NGV, A45) appears --")
    print("rung 31's (*) mass-flow trick chains TWICE: tau_HPT from (A4,A45), tau_LPT from (A45,A8),")
    print("both independent of either compressor. The two compressor RATIOS are not a 2x2 solve.")

    gas = Gas.reacting_equilibrium()
    pi_lpc, pi_hpc = 3.0, 6.0
    two_spool_losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    design = build_two_spool_turbojet(gas, pi_lpc, pi_hpc, TT4, flight.p0,
                                      nozzle_convergent=True, **two_spool_losses)
    m = TwoSpoolMatcher(design, flight, 1.0)

    print(f"\n  Running line (M0={flight.M0}), design pi_LPC={pi_lpc}, pi_HPC={pi_hpc}:")
    print(f"  {'Tt4 [K]':>7} {'pi_LPC':>8} {'pi_HPC':>8} {'tau_HPT':>9} {'tau_LPT':>9} {'mdot/mdot_R':>11} {'F/mdot':>8}")
    print("  " + "-" * 68)
    for Tt4 in (1500.0, 1300.0, 1100.0, 900.0, 700.0):
        od = m.match(flight, Tt4)
        print(f"  {Tt4:>7.0f} {od.pi_lpc:>8.4f} {od.pi_hpc:>8.4f} {od.tau_hpt:>9.6f} {od.tau_lpt:>9.6f} "
              f"{od.mdot_ratio:>11.4f} {od.performance.specific_thrust:>8.1f}")
    try:
        m.match(flight, 600.0)
    except AssertionError:
        print("  600      nozzle UNCHOKES here -- OUT OF SCOPE (flagged, not solved; rung-33-shaped follow-on)")

    # THE FINDING — measured directly on the cascade, at a FIXED (Tt2, Tt4, f) so the outer
    # f-loop's own (separately disclosed) cross-talk cannot confound the reading.
    state0, _ = m._fs_engine.freestream(flight, m.mdot_air_design)
    Tt2, pt2 = state0.Tt, m.pi_d_max * state0.pt
    f = 0.02
    pt4 = m.pi_b * m.pi_hpc_design * m.pi_lpc_design * pt2
    wgas = m._working_gas(f, TT4, pt4)
    base = m._cascade(wgas, Tt2, TT4, f)

    def perturbed(attr, value):
        saved = getattr(m, attr)
        setattr(m, attr, value)
        c = m._cascade(wgas, Tt2, TT4, f)
        setattr(m, attr, saved)
        return c["pi_lpc"] != base["pi_lpc"], c["pi_hpc"] != base["pi_hpc"]

    print("\n  THE FINDING — perturb ONE component parameter at fixed (Tt2, Tt4, f); does pi_LPC / pi_HPC move?")
    print(f"  {'parameter':>10} {'moves pi_LPC?':>14} {'moves pi_HPC?':>14}  role")
    print("  " + "-" * 62)
    for attr, label, role in (
        ("eta_hpc", "eta_HPC", "HP compressor's OWN pressure-inversion leaf"),
        ("eta_lpc", "eta_LPC", "LP compressor's OWN pressure-inversion leaf"),
        ("eta_hpt", "eta_HPT", "energy-path (shapes the shared Tt45)"),
        ("eta_lpt", "eta_LPT", "energy-path (shapes the shared Tt25 via Tt5)"),
    ):
        moves_lpc, moves_hpc = perturbed(attr, 0.55)
        print(f"  {label:>10} {str(moves_lpc):>14} {str(moves_hpc):>14}  {role}")
    print("  Each compressor's OWN efficiency is a dead end for the OTHER spool -- so the two")
    print("  compressor ratios are never a joint (2x2) solve. This is narrower than 'the spools")
    print("  don't talk' (eta_HPT/eta_LPT legitimately move BOTH -- an initial over-claim, corrected).")
    print("  Reduce: lp_disabled=True dispatches (not limits) to rung 31's OffDesignMatcher bit-for-bit.")
    print("  Scope: isentropic knobs only, no compressor maps yet; cycle stays rung-6 exact.")


def print_two_spool_map_table(flight):
    """Rung 39 — TWO-SPOOL + COMPONENT MAPS: the cascade acquires a DIRECTION.

    Rung 38 predicted this rung would break it ("a real map ... would very likely reintroduce the
    coupling ... the two spools DO need a joint solve"). The prediction is WRONG, and how it is
    wrong is the rung. THE ALGEBRA: refer the HPT-NGV choke to the HP compressor face, and since
    pt4/pt25 = pi_b*pi_HPC, pi_LPC CANCELS -- the LP compressor raises pressure and mass flow
    PROPORTIONALLY, so the HP core sees the same CORRECTED flow whatever the LP spool delivers.
    The HP map coordinates are therefore a closed fixed point in pi_HPC alone and cannot see
    eta_LPC; the LP face does carry pi_HPC. So the map opens EXACTLY ONE arrow (HP -> LP): the
    cascade is not dissolved into a 2x2, it acquires a DIRECTION. Rung 38's VERDICT survives;
    rung 38's stated REASON for expecting it to fail is refuted -- the rung-28 shape.
    STRUCTURAL NOVELTY: two shaft speeds (rung 38 computes none), hence the SLIP N_L/N_H --
    exactly 1 on a CPG gas with flat maps (a structural identity: (1+f) and Tt4 both cancel),
    broken PREDOMINANTLY BY THE MAP. That inverts rung 32's decomposition: there the map only
    re-labelled map-free work; here the map CREATES the object.
    """
    print("\nTwo-spool + component maps (rung 39): rung 38 predicted a map would force a joint 2x2")
    print("solve. It does not. pi_LPC CANCELS out of the HP compressor's corrected flow, so the map")
    print("opens EXACTLY ONE arrow (HP -> LP): the cascade acquires a DIRECTION instead of dissolving.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    pi_lpc, pi_hpc = 3.0, 6.0

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    def matcher(gas, mL=None, mH=None):
        design = build_two_spool_turbojet(gas, pi_lpc, pi_hpc, TT4, flight.p0,
                                          nozzle_convergent=True, **losses)
        return TwoSpoolMapMatcher(design, flight, 1.0, map_lp=mL, map_hp=mH)

    map_lp = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7, a_t=0.0)
    map_hp = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0, a_t=0.0)

    # --- THE FINDING: the asymmetry, at a FIXED (Tt2, pt2, Tt4, f) (rung 38 gate-3's protocol).
    m = matcher(cpg(), map_lp, map_hp)
    Tt4_p = 1200.0
    od0 = m.match(flight, Tt4_p)
    state0, _ = m._fs_engine.freestream(flight, m.mdot_air_design)
    Tt2 = state0.Tt
    pt2 = m.pi_d_max * ram_recovery(flight.M0) * state0.pt
    f = od0.stations["4"].far
    pt4 = m.pi_b * od0.pi_hpc * od0.pi_lpc * pt2
    wgas = m._working_gas(f, Tt4_p, pt4)
    base = m._cascade_map(wgas, Tt2, pt2, Tt4_p, f)

    def perturb(attr, delta=0.01):
        saved = getattr(m, attr)
        setattr(m, attr, saved - delta)
        c = m._cascade_map(wgas, Tt2, pt2, Tt4_p, f)
        setattr(m, attr, saved)
        return (c["pi_lpc"] / base["pi_lpc"] - 1.0, c["pi_hpc"] / base["pi_hpc"] - 1.0)

    print(f"\n  THE FINDING — perturb one efficiency by -0.01 at fixed (Tt2, pt2, Tt4={Tt4_p:.0f}, f);")
    print("  compressor maps only (a_t = 0), so the reading is the pure structural one:")
    print(f"  {'parameter':>10} {'d pi_LPC / pi':>15} {'d pi_HPC / pi':>15}  channel")
    print("  " + "-" * 74)
    for attr, label, role in (
        ("eta_lpc", "eta_LPC", "own ratio only -- CANNOT reach pi_HPC (pi_LPC cancels)"),
        ("eta_hpc", "eta_HPC", "own ratio AND pi_LPC -- THE ONE ARROW the map opens"),
        ("eta_hpt", "eta_HPT", "energy path: moves BOTH (shapes the shared Tt45)"),
        ("eta_lpt", "eta_LPT", "energy path: moves BOTH (shapes Tt25 via Tt5)"),
    ):
        dL, dH = perturb(attr)
        sL = "EXACTLY 0" if dL == 0.0 else f"{dL:+.3e}"
        sH = "EXACTLY 0" if dH == 0.0 else f"{dH:+.3e}"
        print(f"  {label:>10} {sL:>15} {sH:>15}  {role}")
    print("  eta_LPC -> pi_HPC is EXACTLY zero (bit-for-bit), by the (dagger) cancellation -- while")
    print("  eta_HPC -> pi_LPC is real and negative. ONE arrow, not two: still strictly triangular.")

    # --- the structural novelty: two speeds, hence the slip.
    print("\n  THE STRUCTURAL NOVELTY — two shaft speeds (rung 38 computes none) => the SLIP N_L/N_H:")
    print(f"  {'Tt4 [K]':>7} {'pi_LPC':>8} {'pi_HPC':>8} {'eta_LPC':>8} {'eta_HPC':>8} "
          f"{'N_L/N_Ld':>9} {'N_H/N_Hd':>9} {'slip':>9}")
    print("  " + "-" * 78)
    ms = matcher(cpg(), map_lp, map_hp)
    for Tt4 in (1500.0, 1300.0, 1100.0, 900.0):
        od = ms.match(flight, Tt4)
        print(f"  {Tt4:>7.0f} {od.pi_lpc:>8.4f} {od.pi_hpc:>8.4f} {od.eta_lpc:>8.5f} "
              f"{od.eta_hpc:>8.5f} {od.N_lp_ratio:>9.5f} {od.N_hp_ratio:>9.5f} {od.slip:>9.6f}")
    print("  The LP spool falls AWAY from the HP spool as the engine is throttled back -- the")
    print("  textbook twin-spool behaviour (at idle a twin-spool runs high N_H, much lower N_L).")

    # --- B1/B2: the slip identity and what breaks it.
    print("\n  WHERE THE SLIP COMES FROM — slip == 1 is a STRUCTURAL identity, broken by two channels:")
    print(f"  {'gas / map':>26} {'Tt4=1500':>10} {'1300':>10} {'1100':>10} {'900':>10}")
    print("  " + "-" * 70)
    for label, gas_fn, mL, mH in (
        ("CPG, FLAT maps", cpg, None, None),
        ("thermally-perfect, FLAT", Gas.thermally_perfect, None, None),
        ("reacting, FLAT", Gas.reacting_equilibrium, None, None),
        ("CPG, SHAPED maps", cpg, map_lp, map_hp),
    ):
        mm = matcher(gas_fn(), mL, mH)
        row = "".join(f"{mm.match(flight, T).slip:>10.6f}"
                      for T in (1500.0, 1300.0, 1100.0, 900.0))
        print(f"  {label:>26} {row}")
    print("  Both shaft works are eta_m*(1+f)*cp_t*Tt4*[pure geometry], so (1+f) AND Tt4 cancel in")
    print("  N_L/N_H: on CPG + flat maps the slip is EXACTLY 1 at every throttle. The cp(T) gas curve")
    print("  breaks it ~1.5% (the rung-31-gate-5 mirror); the MAP breaks it ~5% -- the larger channel.")
    print("  ON CPG that INVERTS rung 32 (where the map only re-labelled map-free work): the deviation is")
    print("  identically zero without a map, so there the map is the SOLE channel and CREATES the object.")
    print("  NOT unconditional -- on the reacting gas the same FLAT maps already give 0.9835 at Tt4=900,")
    print("  so on the real gas the map is the DOMINANT channel (~3.4x), not the only one.")
    print("  Reduce: FLAT maps => rung 38 bit-for-bit;")
    print("  lp_disabled dispatches to rung 32 (shaped) / rung 31 (flat). Cycle stays rung-6 exact.")
    print("  Disclaimed: representative maps -- every magnitude (arrow strength, slip depth) rides on")
    print("  the shapes; only the asymmetry, the identity and the slip SIGN are load-bearing.")


def print_two_shaft_transient_table(flight):
    """Rung 40 — THE TWO-SHAFT TRANSIENT: the LP map opens a COMPLEX mode.

    Rung 39 named this seam "now well-posed" (rung 38 could supply no N at all; rung 39
    supplies two). Both shaft speeds become STATES under two inertia ODEs; nondimensionalizing
    on the HP clock leaves exactly ONE parameter, the clock RATIO rho = tau_L/tau_H. That is
    the resolution of rung 34's tautology -- rung 34 had to IMPOSE a second clock (tau_fuel)
    before inertia mattered; here each spool IS the other's clock. But "rho survives" is itself
    dimensional analysis, so the content is what rho can and cannot DO, and it splits:
    STABILITY is rho-free (the sign conditions a<0, d<0, ad>bc carry no rho -- measured over
    252 points, zero violations), while OSCILLATION is not (disc = (a/rho-d)^2 + 4bc/rho
    vanishes at rho = a/d). Whenever bc < 0 a COMPLEX inter-spool mode exists -- and bc < 0
    exactly when the LP compressor map is SHAPED, with hp-only (HP shaped, LP FLAT) the
    discriminator proving it is the LP map specifically. The mode is MAP-CREATED, rung 39's
    slip pattern a third time. Much of the rest is INHERITED from rung 39 B1/B2 and says so.
    """
    print("\nTwo-shaft transient (rung 40): both shaft speeds become STATES. Nondimensionalizing")
    print("leaves ONE parameter -- the clock RATIO rho = tau_L/tau_H -- and its power SPLITS:")
    print("it can never destabilize the pair, but it decides whether the mode is real or COMPLEX.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    pi_lpc, pi_hpc = 3.0, 6.0
    FLAT = ComponentMap.flat()
    LP_S = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
    HP_S = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    def tt(gas, mL, mH):
        design = build_two_spool_turbojet(gas, pi_lpc, pi_hpc, TT4, flight.p0,
                                          nozzle_convergent=True, **losses)
        return TwoSpoolTransient(design, flight, 1.0, map_lp=mL, map_hp=mH)

    # --- the reduce: the 2-D root lands on rung 39, through the FORWARD closure only.
    t = tt(cpg(), LP_S, HP_S)
    print("\n  REDUCE — the 2-D equilibrium (Phi_L = Phi_H = 0) vs rung 39's matched point")
    print("  (via the forward closure ONLY -- it never calls the matcher, so this is non-circular):")
    print(f"  {'Tt4':>6} {'d nu_L':>11} {'d nu_H':>11} {'d pi_LPC':>11} {'d pi_HPC':>11}")
    print("  " + "-" * 56)
    for Tt4 in (1500.0, 1300.0, 1200.0):
        od, eq = t.match(flight, Tt4), t.equilibrium(flight, Tt4)
        print(f"  {Tt4:6.0f} {eq['nu_lp']/od.N_lp_ratio-1:11.2e} "
              f"{eq['nu_hp']/od.N_hp_ratio-1:11.2e} {eq['pi_lpc']/od.pi_lpc-1:11.2e} "
              f"{eq['pi_hpc']/od.pi_hpc-1:11.2e}")

    # --- sigma_crit: the lead threshold. The ==1 identity is INHERITED from rung 39 B1.
    print("\n  sigma_crit (the LEAD THRESHOLD: HP leads iff rho > sigma_crit), at Tt4=1100.")
    print("  == 1 on flat+CPG is rung 39's B1 slip identity restated for the transient")
    print("  (on the running line sigma_crit reduces to the steady slip) -- INHERITED, the")
    print("  reduce spine, NOT this rung's finding:")
    rows = [("CPG + flat maps", cpg(), FLAT, FLAT),
            ("thermally_perfect + flat", Gas.thermally_perfect(), FLAT, FLAT),
            ("reacting + flat", Gas.reacting_equilibrium(), FLAT, FLAT),
            ("CPG + shaped maps", cpg(), LP_S, HP_S)]
    print(f"  {'configuration':>26} {'sigma_crit - 1':>16}   channel")
    print("  " + "-" * 66)
    for label, gas, mL, mH in rows:
        dev = tt(gas, mL, mH).lead_threshold(flight, 1100.0, d=25.0) - 1.0
        ch = ("the IDENTITY" if abs(dev) < 1e-11 else
              "the cp(T) gas curve" if mL.is_flat() else "the MAP")
        print(f"  {label:>26} {dev:16.4e}   {ch}")
    print("  => the map channel is ~5.8x the gas channel: DOMINANT, not sole (rung 39 B2's shape).")

    lp_only = tt(cpg(), LP_S, FLAT).lead_threshold(flight, 1100.0)
    hp_only = tt(cpg(), FLAT, HP_S).lead_threshold(flight, 1100.0)
    print(f"\n  A REFUTED hypothesis, kept visible: 'the map favours the LP spool' is FALSE --")
    print(f"  shaping only the LP map gives sigma_crit = {lp_only:.4f} (< 1), only the HP map "
          f"{hp_only:.4f} (> 1).")
    print("  Both signs are reachable, so only the EXISTENCE of a material shift is claimed.")

    # --- THE FINDING: stability is rho-free; the complex mode is created by the LP map.
    print("\n  THE FINDING — J(rho) = [[a/rho, b/rho], [c, d]] on the running line, Tt4=1200:")
    print("    tr = a/rho + d,  det = (ad-bc)/rho,  disc = (a/rho - d)^2 + 4bc/rho")
    print("  STABILITY needs a<0, d<0, ad>bc -- three conditions with NO rho in them, so the")
    print("  clock ratio can NEVER destabilize the pair. OSCILLATION is different: disc kills")
    print("  its first term at rho = a/d, so bc<0 => a COMPLEX band exists.")
    print(f"\n  {'shapes':>12} {'b':>10} {'b*c':>11} {'ad-bc':>9} {'band (rho)':>20} "
          f"{'|Im/Re|max':>11}")
    print("  " + "-" * 78)
    for name, mL, mH in (("flat", FLAT, FLAT), ("hp-only", FLAT, HP_S),
                         ("lp-only", LP_S, FLAT), ("flow/press", LP_S, HP_S)):
        tx = tt(cpg(), mL, mH)
        od = tx.match(flight, 1200.0)
        nu = (od.N_lp_ratio, od.N_hp_ratio)
        tx.rho = 1.0
        J = tx.jacobian(flight, 1200.0, nu=nu)
        bc = J[0][1] * J[1][0]
        band = tx.oscillatory_band(flight, 1200.0, nu=nu)
        bs = "none" if band is None else f"[{band[0]:.3f}, {band[1]:.3f}]"
        print(f"  {name:>12} {J[0][1]:10.4f} {bc:11.4e} {J[0][0]*J[1][1]-bc:9.4f} {bs:>20} "
              f"{tx.damping_ratio_max(flight, 1200.0, nu=nu):11.3f}")
    print("\n  hp-only is the DISCRIMINATOR: its HP map IS shaped, yet no band appears -- so the")
    print("  mechanism is the LP map SPECIFICALLY, not shaping in general. A shaped LP map flips")
    print("  b = dPhi_L/dnu_H from small-negative to large-positive; with c<0 always, that")
    print("  antisymmetric cross-coupling is what makes the pair complex. The mode is MAP-CREATED")
    print("  -- rung 39's slip pattern a third time.")
    print("  |Im/Re| <= 0.25 here (no visible ringing) is a DISCLAIMED MAGNITUDE, not a verdict:")
    print("  existence + sign + mechanism are gated, the number rides on the representative maps.")
    print("\n  SCOPE (a negative, stated plainly): sigma_crit's authority is FIRST-INSTANT only.")
    print("  The finite-ramp slip excursion is SCHEDULE-SLAVED -- dominated by slip_ss(Tt4) moving")
    print("  while the speeds lag -- so the marched threshold is NOT sigma_crit (0.60x / 1.40x).")
    print("  Oscillation claim scoped to INTER-SPOOL (rung 37's shaft+metal pair is not audited).")
    print("  Reduce: lp_disabled => rung 34 SpoolTransient bit-for-bit. Cycle stays rung-6 exact.")


def print_two_spool_surge_table(flight):
    """Rung 41 — THE TWO-SPOOL SURGE LINE: the exposure SPLITS between the spools.

    Rungs 39 and 40 both closed by naming this seam in the same words ("rung 36's machinery is
    single-spool -- and now there are TWO compressors"). Drawing it on both shows the two-spool
    running line does not HALVE the low-power surge problem, it CONCENTRATES it on the LP
    compressor: rung 39's (dagger) cancellation shields the HP face (it sees only its OWN
    pressure ratio) while the LP face carries the PRODUCT. That asymmetry is closed-form, and
    its corollary is the zero-new-constant critical pressure ratio rung 36's DEAD anchor never
    got: pi* = gamma_c^(gamma_c/(gamma_c-1)). It is an INCIDENCE fact, not a margin extremum --
    and that divergence is what corrects rung 36's stated mechanism.
    """
    print("\nTwo-spool surge line (rung 41): drawing rung 36's line on BOTH compressors. The")
    print("two-spool running line does not halve the low-power surge problem -- it CONCENTRATES")
    print("it on the LP spool, and the reason is closed-form.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    single = dict(pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96, eta_t=0.92,
                  eta_m=0.99, pi_n=0.98)
    TILTED = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    def mm(gas, mL=None, mH=None, pl=3.0, ph=6.0):
        design = build_two_spool_turbojet(gas, pl, ph, TT4, flight.p0,
                                          nozzle_convergent=True, **losses)
        return TwoSpoolMapMatcher(design, flight, 1.0, map_lp=mL, map_hp=mH)

    # --- THE SPLIT: the LP takes the excursion, the HP is shielded and BOUNDED.
    m = mm(Gas.thermally_perfect(), TILTED, TILTED)
    print("\n  THE SPLIT — the two running lines in map coordinates (matched `tilted` shape):")
    print(f"  {'Tt4':>6} {'phi_L':>8} {'phi_H':>8} {'pi_LPC':>8} {'pi_HPC':>8}")
    print("  " + "-" * 42)
    for r in m.running_line_map(flight, [1500, 1300, 1100, 900, 800, 750]):
        print(f"  {r['Tt4']:6.0f} {r['phi_lp']:8.4f} {r['phi_hp']:8.4f} "
              f"{r['pi_lpc']:8.4f} {r['pi_hpc']:8.4f}")
    print("  => phi_L falls ~29%, phi_H ~7% and TURNS BACK UP. The LP takes the excursion.")

    # --- WHY: the sensitivity of each face is closed-form in the pressure ratios it SEES.
    print("\n  WHY — each face's flow-coefficient sensitivity, closed form (CPG, flat maps):")
    print("    s_H = k(1 - pi_HPC^(-1/k)) - 1                     <- pi_HPC ALONE (rung 39 (dagger))")
    print("    s_L = k(1 - pi_LPC^(-1/k)) + k(1 - pi_HPC^(-1/k))/tau_LPC - 1   <- the PRODUCT")
    mc = mm(cpg())
    k = 1.4 / 0.4
    print(f"  {'Tt4':>6} {'s_H':>8} {'pred':>8} {'s_L':>8} {'pred':>8} {'pred w/o pi_HPC':>16}")
    print("  " + "-" * 60)
    for Tt4 in (1400.0, 1200.0, 1000.0, 850.0, 750.0):
        a, b = mc.match(flight, Tt4 - 4.0), mc.match(flight, Tt4 + 4.0)
        c = mc.match(flight, Tt4)

        def _lg(o, hp):
            x = o.Tt4 / (o.stations["25"].Tt if hp else o.stations["2"].Tt)
            return math.log(o.phi_hp if hp else o.phi_lp), math.log(x)
        (ya, xa), (yb, xb) = _lg(a, True), _lg(b, True)
        sH = (yb - ya) / (xb - xa)
        (ya, xa), (yb, xb) = _lg(a, False), _lg(b, False)
        sL = (yb - ya) / (xb - xa)
        tauL = c.stations["25"].Tt / c.stations["2"].Tt
        sHp = k * (1.0 - c.pi_hpc ** (-1.0 / k)) - 1.0
        sLp = (k * (1.0 - c.pi_lpc ** (-1.0 / k))
               + k * (1.0 - c.pi_hpc ** (-1.0 / k)) / tauL - 1.0)
        sLn = k * (1.0 - c.pi_lpc ** (-1.0 / k)) - 1.0
        print(f"  {Tt4:6.0f} {sH:8.4f} {sHp:8.4f} {sL:8.4f} {sLp:8.4f} {sLn:16.4f}")
    print("  => both predictions land within ~0.013. DROPPING pi_HPC from s_L misses by ~0.8-1.0")
    print("     AND gets the SIGN wrong: the LP face cannot be written without the HP's ratio,")
    print("     while the HP's needs no LP quantity at all. The shielding, quantified.")

    # --- (star): the closed form, and its fuel-fraction kill test.
    print("\n  THE CLOSED FORM (star) — s_H = 0 gives 1 + eta_c(tau_c-1) = gamma_c, i.e.")
    print(f"    pi_c* = gamma_c^(gamma_c/(gamma_c-1)) = {mc.critical_flow_turn_pi():.5f}  "
          "(gamma_c ALONE).")
    print("  The turn's location in Tt4 moves by ~76%; its location in PRESSURE RATIO does not:")
    print(f"  {'case':>16} {'Tt4*':>8} {'pi*':>9} {'1+eta(tau-1)':>14} {'vs gamma_c':>12}")
    print("  " + "-" * 64)
    for label, kw in (("split 3x6", dict()), ("split 4.5x4", dict(pl=4.5, ph=4.0)),
                      ("split 2.25x8", dict(pl=2.25, ph=8.0))):
        t = mm(cpg(), **kw).flow_coefficient_turn(flight, "hp")
        print(f"  {label:>16} {t['Tt4_star']:8.1f} {t['pi_star']:9.4f} "
              f"{t['star_form']:14.5f} {100*(t['star_form']/1.4-1):11.3f}%")
    t = mm(cpg()).flow_coefficient_turn(FlightCondition(T0=250.0, p0=50_000.0, M0=1.60), "hp")
    print(f"  {'M0=1.60':>16} {t['Tt4_star']:8.1f} {t['pi_star']:9.4f} "
          f"{t['star_form']:14.5f} {100*(t['star_form']/1.4-1):11.3f}%")
    print("  eta_HPC (0.80/0.95), eta_HPT, gamma_t and cp_t all drop out -- verified in the gates.")
    print("\n  KILL TEST — the whole +0.44% residual is the FUEL FRACTION (f enters K and the")
    print("  choked flow; (star) is exact with f frozen). Raise hPR so f -> 0:")
    print(f"  {'hPR':>10} {'f':>9} {'1+eta(tau-1)':>14} {'residual':>10}")
    print("  " + "-" * 46)
    for hPR in (42.8e6, 4.28e8, 4.28e10):
        g, cp = 1.3, 1239.0
        gas = Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                  R_t=(g - 1.0) / g * cp, hPR=hPR)
        t = mm(gas).flow_coefficient_turn(flight, "hp")
        print(f"  {hPR:10.3g} {t['far']:9.5f} {t['star_form']:14.5f} "
              f"{100*(t['star_form']/1.4-1):9.3f}%")

    # --- the margins, and the deliberate divergence.
    print("\n  THE MARGINS — matched shape on both spools, COMMON imposed floor phi_surge=0.55")
    print("  (rung 36's one disclosed constant, DOUBLED -- every magnitude disclaimed):")
    ms = mm(Gas.thermally_perfect(), TILTED.with_phi_surge(0.55), TILTED.with_phi_surge(0.55))
    print(f"  {'Tt4':>6} {'SM_L':>8} {'SM_H':>8} {'SM_L/SM_H':>11} {'phi_H':>8}")
    print("  " + "-" * 46)
    for r in ms.surge_margin_schedule(flight, [1500, 1300, 1100, 900, 800, 750]):
        print(f"  {r['Tt4']:6.0f} {r['SM_lp']:8.4f} {r['SM_hp']:8.4f} "
              f"{r['SM_lp']/r['SM_hp']:11.4f} {r['phi_hp']:8.4f}")
    print("  => the RATIO collapses (0.61 -> 0.13): THAT is the running-line divergence, and it")
    print("     is what is gated. The ORDERING's LEVEL is partly a DESIGN-SPLIT artifact --")
    print("     SM_L < SM_H already at Tt4=1500 where phi_L = phi_H = 1, purely because")
    print("     pi_LPC=3 < pi_HPC=6 (a smaller design pressure ratio gives a smaller margin at")
    print("     the same flow-coefficient gap). Not over-attributed to exposure.")
    print("     NOTE also the deliberate DIVERGENCE:")
    print("     phi_H TURNS UP past pi* while SM_H keeps FALLING -- so (star) is an INCIDENCE")
    print("     fact, NOT a margin extremum. The worst margin is still at idle, on both spools.")

    # --- the cross-rung correction of rung 36.
    print("\n  THE CORRECTION OF RUNG 36 (the rung-28 shape: verdict confirmed, reason corrected).")
    print("  (star) is SURFACED by this rung, not created by it -- the same turn sits INSIDE rung")
    print("  36's OWN choked envelope (pi_c=10 single spool). Freezing one coordinate at a time:")
    d1 = build_turbojet(Gas.thermally_perfect(), 10.0, TT4, flight.p0,
                        nozzle_convergent=True, **single)
    cm = ComponentMap.surge_flow().with_phi_surge(0.55)
    st = SpoolTransient(d1, flight, 1.0, comp_map=cm)
    print(f"  {'Tt4':>6} {'pi_c':>8} {'phi_op':>9} {'SM_N':>8} {'SM(phi-walk)':>13} "
          f"{'SM(speed-line)':>15}")
    print("  " + "-" * 64)
    for Tt4 in (1500.0, 1100.0, 900.0, 800.0, 700.0, 650.0, 600.0):
        try:
            c = st.surge_margin_channels(flight, Tt4, cm)
        except AssertionError:
            break
        print(f"  {Tt4:6.0f} {c['pi_c']:8.4f} {c['phi_op']:9.5f} {c['SM_N']:8.4f} "
              f"{c['SM_phi_walk']:13.4f} {c['SM_speed_line']:15.4f}")
    print("  => phi_op TURNS UP at the bottom (crossing pi*), and the phi-walk channel turns with")
    print("     it -- yet SM_N keeps falling, because the SPEED LINE FLATTENS (tau_c-1 ~ n^2) and")
    print("     that channel does not reverse. Rung 36's VERDICT survives (SM_N still monotone,")
    print("     no rung-36 test changes); its stated MECHANISM ('the trend is set by phi_op') was")
    print("     SINGLE-CHANNEL -- the two are comparable (~56%/48% of the log decay). Both are")
    print("     choke-determined, so rung 36's floor-robustness conclusion is untouched.")
    print("\n  NOT claimed: 'the slip protects the LP spool' (the rigid-shaft counterfactual is")
    print("  not run); which spool binds at UNMATCHED shapes/floors; any margin magnitude.")
    print("  Reduce: a phi_surge-carrying map leaves rung 39/40 bit-for-bit. Cycle stays rung-6 exact.")



def print_interstage_bleed_table(flight):
    """Rung 42 — INTERSTAGE BLEED: the valve is a degree of freedom on ONE spool.

    Rungs 36 and 41 both closed with the same concession ("no bleed valve / variable stator --
    this rung exhibits the margin they protect, it does not model them"), and rung 41 located
    the exposure on the LP compressor. Fitting the valve shows it is a NEW degree of freedom
    on the LP spool and NOT on the HP spool: x_L = Tt4/Tt2 is EXACTLY bleed-invariant, so the
    whole dphi_L is displacement OFF the running line, while the HP -- whose corrected-flow
    referral (dagger) carries no b -- only SLIDES ALONG its own line, so its whole response is
    rung 41's closed-form s_H, including the sign reversal at pi*.
    """
    print("\nInterstage bleed (rung 42): the device rungs 36 and 41 both deferred, fitted to the")
    print("spool rung 41 showed is exposed. A fraction b is extracted at station 25 and dumped --")
    print("the project's FIRST steady mass EXTRACTION -- the first time mass LEAVES the flowpath,")
    print("so the two COMPRESSORS pass different air. (NOT 'compressor and turbine differ': (1+f)")
    print("has done that since rung 2. What is new is mass leaving, not mass changing.)")
    print("Comparison held at FIXED Tt4: the valve sets b, not the throttle.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
    FLAT = ComponentMap()

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    def bm(gas, mL, mH, b, floor=None):
        if floor is not None:
            mL, mH = mL.with_phi_surge(floor), mH.with_phi_surge(floor)
        design = build_two_spool_turbojet(gas, 3.0, 6.0, TT4, flight.p0,
                                          nozzle_convergent=True, **losses)
        return TwoSpoolBleedMatcher(design, flight, 1.0, map_lp=mL, map_hp=mH, bleed=b)

    # --- WHERE b ENTERS: three places, and not the fourth.
    print("\n  WHERE b ENTERS -- exactly three places:")
    print("    (1) the LP shaft balance   h_c(Tt25)-h_c(Tt2) = eta_m(1-b)(1+f)dh_LPT   => Tt25 FALLS")
    print("    (2) the LP face referral   (ddagger-b): mdot_corr,2 picks up an explicit 1/(1-b)")
    print("    (3) the thrust books       the dumped air keeps full ram drag, returns no momentum")
    print("  and NOT the HP face: rung 39's (dagger) mdot_corr,25 = A4 pi_b pi_HPC MFP* "
          "sqrt(Tt25/Tt4)/(1+f)")
    print("  is core flow on BOTH sides, so it carries NO b -- which is why _hp_eta_loop is reused")
    print("  VERBATIM. Its BODY is b-free; its ARGUMENTS are not (rung 39's leaf, one rung on).")

    # --- THE ASYMMETRY: LP displaced OFF its line, HP only slides ALONG its own.
    gas = Gas.thermally_perfect()
    print("\n  THE ASYMMETRY -- open the valve at a FIXED Tt4 (shapes flow/press, b = 0.10):")
    print(f"  {'Tt4':>6} {'x_L':>8} {'dphi_L':>9} {'x_H':>8} {'dphi_H':>9} {'ratio':>8} "
          f"{'dF':>8} {'dTSFC':>8}")
    print("  " + "-" * 70)
    shut, opn = bm(gas, LP, HP, 0.0), bm(gas, LP, HP, 0.10)
    for Tt4 in (1500.0, 1300.0, 1100.0, 900.0):
        a, c = shut.match(flight, Tt4), opn.match(flight, Tt4)
        xLa, xLc = Tt4 / a.stations["2"].Tt, Tt4 / c.stations["2"].Tt
        dL, dH = c.phi_lp / a.phi_lp - 1.0, c.phi_hp / a.phi_hp - 1.0
        print(f"  {Tt4:6.0f} {xLa:8.4f} {100*dL:+8.3f}% {Tt4/a.stations['25'].Tt:8.4f} "
              f"{100*dH:+8.3f}% {dL/dH:8.1f} {100*(c.thrust/a.thrust-1):+7.2f}% "
              f"{100*(c.tsfc_inlet/a.performance.tsfc-1):+7.2f}%")
        assert xLa == xLc                       # x_L is built from two INPUTS: bleed cannot move it
    print("  => x_L is EXACTLY bleed-invariant (both Tt4 and Tt2 are inputs), so the whole dphi_L")
    print("     is displacement OFF the LP running line: the LP line becomes a FAMILY indexed by b.")

    print("\n  ...and the HP stays on ONE curve. Take the bled point's x_H, find the b=0 THROTTLE")
    print("  setting with the SAME x_H, compare phi_H (CPG, flat maps, b = 0.10):")
    cshut, copn = bm(cpg(), FLAT, FLAT, 0.0), bm(cpg(), FLAT, FLAT, 0.10)
    print(f"  {'Tt4':>6} {'Tt4* (b=0)':>11} {'x_H':>9} {'HP dphi':>10} {'LP dphi (same x_L)':>20}")
    print("  " + "-" * 62)
    for Tt4 in (1400.0, 1100.0, 900.0):
        c = copn.match(flight, Tt4)
        target = Tt4 / c.stations["25"].Tt
        lo, hi = Tt4, Tt4 * 1.3
        for _ in range(60):
            mid = 0.5 * (lo + hi)
            o = cshut.match(flight, mid)
            if mid / o.stations["25"].Tt - target <= 0.0:
                lo = mid
            else:
                hi = mid
        o = cshut.match(flight, 0.5 * (lo + hi))
        a = cshut.match(flight, Tt4)
        print(f"  {Tt4:6.0f} {0.5*(lo+hi):11.2f} {target:9.5f} "
              f"{100*(c.phi_hp/o.phi_hp-1):+9.4f}% {100*(c.phi_lp/a.phi_lp-1):+19.2f}%")
    print("  => the HP leaves its running line by ~0.01% while the LP is displaced ~11% -- a")
    print("     ~1000x contrast. Bleed gives the LP spool a new freedom; the HP it merely slides.")

    # --- INHERITED: the HP response IS rung 41's s_H, and it reverses sign at pi*.
    print("\n  SO THE HP RESPONSE IS RUNG 41's (INHERITED, and the spec says so): s_H measured by")
    print("  opening the VALVE vs rung 41's closed form measured on the THROTTLE --")
    print("  perturbation-independence, which could have failed (the HP loop reads Tt4, Tt25 and f")
    print("  SEPARATELY on the real gas; only CPG at frozen f makes it one-parameter in x_H):")
    k = 1.4 / 0.4
    pi_star = 1.4 ** k
    cdb = bm(cpg(), FLAT, FLAT, 0.02)
    print(f"  {'Tt4':>6} {'pi_HPC':>8} {'s_H (valve)':>12} {'s_H closed':>11} {'diff':>9} "
          f"{'dln phi_L':>10}")
    print("  " + "-" * 62)
    for Tt4 in (1500.0, 1300.0, 1100.0, 900.0, 800.0, 790.0, 780.0, 750.0, 700.0):
        a, c = cshut.match(flight, Tt4), cdb.match(flight, Tt4)
        xa, xc = Tt4 / a.stations["25"].Tt, Tt4 / c.stations["25"].Tt
        sm = math.log(c.phi_hp / a.phi_hp) / math.log(xc / xa)
        sc = k * (1.0 - a.pi_hpc ** (-1.0 / k)) - 1.0
        mark = "  <- pi*" if 780.0 <= Tt4 <= 790.0 else ""
        print(f"  {Tt4:6.0f} {a.pi_hpc:8.5f} {sm:12.4f} {sc:11.4f} {sm-sc:+9.4f} "
              f"{math.log(c.phi_lp/a.phi_lp):+10.5f}{mark}")
    print(f"  => agreement to <=0.004 over a 2.4:1 throttle. And since s_H = 0 at")
    print(f"     pi* = gamma_c^(gamma_c/(gamma_c-1)) = {pi_star:.5f}, the bleed response passes")
    print("     through ZERO there and REVERSES SIGN below it -- bracketed above between")
    print("     Tt4 = 790 (pi_HPC = 3.2688, +) and 780 (3.2339, -). The crossing interpolates to")
    print("     pi_HPC ~ 3.260, i.e. +0.40%: the SAME fuel-fraction residual rung 41's own kill")
    print("     test isolated (+0.44%). pi* SURFACES A THIRD TIME -- its LOCATION is inherited,")
    print("     that a second, independent perturbation sweeps through it is new.")
    print("     The growing ratio is the HP denominator passing through zero (dln phi_L is nearly")
    print("     constant ~0.022 throughout) -- NOT 'infinite selectivity'.")

    # --- SELF-TARGETING, stated in phi-space (NOT in relative margin).
    print("\n  SELF-TARGETING -- stated in phi-SPACE (rung 41's surge-proximity currency), because")
    print("  the relative-margin version is CONFOUNDED (absolute dSM_L SHRINKS 0.056 -> 0.018 pp;")
    print("  only its collapsing base makes the relative gain 'grow' -- this project's own rung-41")
    print("  lesson).  b = 0.10, matched imposed floor phi_surge = 0.55:")
    fshut, fopn = bm(cpg(), LP, HP, 0.0, floor=0.55), bm(cpg(), LP, HP, 0.10, floor=0.55)
    print(f"  {'Tt4':>6} {'phi_L':>8} {'gap_L':>7} {'dphi_L':>8} {'frac':>7} | "
          f"{'phi_H':>8} {'gap_H':>7} {'dphi_H':>9} {'frac':>7}")
    print("  " + "-" * 76)
    for Tt4 in (1500.0, 1300.0, 1100.0, 950.0, 900.0):
        a, c = fshut.match(flight, Tt4), fopn.match(flight, Tt4)
        gL, gH = a.phi_lp - 0.55, a.phi_hp - 0.55
        dL, dH = c.phi_lp - a.phi_lp, c.phi_hp - a.phi_hp
        print(f"  {Tt4:6.0f} {a.phi_lp:8.4f} {gL:7.4f} {dL:+8.4f} {100*dL/gL:6.1f}% | "
              f"{a.phi_hp:8.4f} {gH:7.4f} {dH:+9.5f} {100*dH/gH:6.2f}%")
    print("  => dphi_L is nearly CONSTANT (+-1%) while dphi_H collapses ~8x. A fixed absolute")
    print("     increment into a SHRINKING LP gap => the fraction closed RISES on the LP spool")
    print("     (17% -> 42%) and FALLS on the HP (1.8% -> 0.4%). That is the honest sense in")
    print("     which the device is SELF-TARGETING.")

    # --- the trade and the envelope.
    print("\n  THE TRADE. Thrust falls 10.0% -> 14.7% and TSFC rises 6.3% -> 14.6% (b = 0.10) as")
    print("  the throttle comes back: the valve gets MORE SELECTIVE and MORE EXPENSIVE together --")
    print("  which is why real bleed is SCHEDULED, not simply left open. And bleed lowers pi_LPC")
    print("  hence pt4, so it SHRINKS the choked envelope (lowest runnable Tt4 605 -> 630 K over")
    print("  b = 0 -> 0.15): the inherited nozzle-choke guard bites sooner. It flags, it never lies.")

    print("\n  A HYPOTHESIS, REFUTED and kept visible (rung 40's convention): this rung was")
    print("  proposed as 'bleed protects the LP AT THE HP SPOOL'S EXPENSE'. FALSE -- above pi* the")
    print("  HP flow coefficient RISES too, just 10-100x less; below pi* it falls, by ~1e-4. The")
    print("  textbook trade is not what the choked two-spool hardware does.")
    print("\n  NOT claimed: any magnitude (all ride on b, the representative maps and the two")
    print("  imposed floors); a surge-SURVIVAL claim (E0 vs SM_N needs the transient, deferred);")
    print("  a bleed SCHEDULE b(n_L); variable stators (they move phi_surge -- still open).")
    print("  Reduce: bleed=0 => rung 39 bit-for-bit by exact dispatch. Cycle stays rung-6 exact.")


def print_two_shaft_fuel_table(flight):
    """Rung 43 — TWO-SHAFT FUEL METERING: the two spools sit at DIFFERENT points in ONE loop.

    Rung 35's control (meter mdot_fuel, let Tt4 float) on rung 40's two-shaft plant. The
    two-shaft content is a question one shaft structurally cannot ask: f = mdot_fuel/mdot_air
    is set at the LP FACE, but the Tt4 it produces is metered back through the HP-FED NGV
    choke -- so "which spool's lag governs the overshoot?" answers NEITHER. Freezing either
    spool makes the overshoot WORSE, and the share of the relief trades with rho.
    """
    print("\nTwo-shaft fuel metering (rung 43): rung 35's control on rung 40's plant. Fuel is")
    print("metered and Tt4 FLOATS against the airflow two lagging spools can currently pump.")
    print("Rung 35's TIT-overshoot finding re-measures unchanged and is INHERITED, not this")
    print("rung's finding. What is new is a question ONE shaft cannot ask:")
    print("    f   = mdot_fuel / mdot_air        <- the LP FACE sets the airflow")
    print("    Tt4 = burner(Tt3, f)                 (Tt4 floats up as the LP lag spikes f)")
    print("    md4 = A4 pt4 MFP*(Tt4)/sqrt(Tt4)  <- the HP-FED NGV CHOKE meters it back")
    print("The two spools sit at DIFFERENT points in the ONE overshoot loop, so with two clocks")
    print("there is a RATIO rho = tau_L/tau_H and the question is: which spool's lag governs it?")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
    LO, HI = 1250.0, 1450.0          # rung 35's own step -- apples-to-apples

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    def ft(rho=1.0):
        design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                          nozzle_convergent=True, **losses)
        return TwoSpoolFuelTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=rho)

    # --- the reduce, first: a steady point is the same however it is NAMED.
    print("\n  REDUCE -- CONTROL-INVARIANCE (the non-tautological gate). Feed the fuel of a rung-40")
    print("  Tt4-control point to the FUEL solver: it must return that point, via the forward")
    print("  BURNER (Tt4 an OUTPUT) -- a genuinely different code path. Two closures, one point:")
    f0 = ft()
    print(f"  {'Tt4':>6} {'d nu_L':>10} {'d nu_H':>10} {'d Tt4':>10} {'d pi_LPC':>10}")
    print("  " + "-" * 50)
    for Tt4 in (1500.0, 1300.0, 1100.0):
        eq = f0.equilibrium(flight, Tt4)
        fq = f0.equilibrium_fuel(flight, eq["f"] * eq["mdot_air"])
        print(f"  {Tt4:6.0f} {fq['nu_lp']/eq['nu_lp']-1:+10.2e} {fq['nu_hp']/eq['nu_hp']-1:+10.2e} "
              f"{fq['Tt4']/Tt4-1:+10.2e} {fq['pi_lpc']/eq['pi_lpc']-1:+10.2e}")
    print("  => machine zero. This also KILLS, empirically, the framing this rung was proposed")
    print("     with -- 'fuel metering breaks rung 39's (dagger) and re-couples LP into the HP")
    print("     core'. If both controls land on the SAME manifold, the knob cannot change the")
    print("     coupling. (It was a category error anyway: (dagger) is a STEADY eta-fixed-point")
    print("     artifact that does not arise in the transient closure at all.)")

    # --- THE FINDING: channel isolation. Freeze one spool at a time.
    print("\n  THE FINDING -- CHANNEL ISOLATION (rung 41's move, applied to the transient): march")
    print("  the fuel ramp with ONE spool's speed HELD at its initial value. Tt4_peak [K]:")
    print(f"  {'rho':>5} {'r':>5} {'both free':>10} {'LP frozen':>10} {'HP frozen':>10} "
          f"{'d_LP':>8} {'d_HP':>8}")
    print("  " + "-" * 60)
    for r in (0.25, 1.0):
        for rho in (0.5, 1.0, 2.0):
            fc = ft(rho).freeze_channels(flight, LO, HI, r=r)
            print(f"  {rho:5.1f} {r:5.2f} {fc['both']:10.1f} {fc['lp']:10.1f} {fc['hp']:10.1f} "
                  f"{fc['d_lp']:+8.1f} {fc['d_hp']:+8.1f}")
    print("  => (1) freezing EITHER spool makes the overshoot WORSE, 6/6 -- both spools' motion")
    print("         RELIEVES it. Neither is a bystander.")
    print("     (2) the SHARE of the relief TRADES with rho: as the LP spool slows, the LP")
    print("         channel weakens and the HP channel strengthens.")
    print("     THAT is why no single spool's clock can govern the overshoot -- the responsibility")
    print("     for quenching it is SHARED and rho-DEPENDENT. Direction only: d_LP and d_HP do")
    print("     NOT sum to the total and are not calibrated weights.")

    # --- the bounded positive: monotone in rho, ceilinged by the LP-frozen march.
    print("\n  THE POSITIVE, AND ITS CEILING. X = Tt4_peak - Tt4_target rises monotonically with")
    print("  rho (a heavier LP spool worsens the TIT excursion -- the LP-face lag is what spikes")
    print("  f). It is BOUNDED, and the bound is STRUCTURAL: rho multiplies ONLY the LP ODE")
    print("  (dnu_L/ds = Phi_L/rho), so rho -> infinity IS the LP-frozen system:")
    print(f"  {'r':>5} " + "".join(f"{('rho='+str(x)):>9}" for x in (0.25, 1, 4, 8, 32, 128))
          + f"{'LP-frozen':>11}")
    print("  " + "-" * 72)
    for r in (0.25, 1.0):
        row = "".join(f"{ft(rho).ramp_excursion_fuel(flight, LO, HI, r)['X']:9.1f}"
                      for rho in (0.25, 1.0, 4.0, 8.0, 32.0, 128.0))
        ceil = ft(1.0).ramp_excursion_fuel(flight, LO, HI, r, freeze="lp")["X"]
        print(f"  {r:5.2f} " + row + f"{ceil:11.1f}")
    a = ft(0.25).ramp_excursion_fuel(flight, LO, HI, 0.25, freeze="lp")["X"]
    b = ft(50.0).ramp_excursion_fuel(flight, LO, HI, 0.25, freeze="lp")["X"]
    print(f"  => X(rho) converges UPWARD onto the LP-frozen march, which is rho-independent")
    print(f"     BIT-FOR-BIT ({a!r} == {b!r} at rho = 0.25 vs 50: {a == b}). So the worst TIT")
    print("     excursion a heavy LP spool can produce is computable WITHOUT marching it.")

    # --- THE NEGATIVE: the currencies are circular.
    print("\n  THE NEGATIVE, STATED PLAINLY -- there is NO effective clock ratio r_eff = r/rho^q")
    print("  (q=0 => 'the HP clock governs', q=1 => 'the slow spool rate-limits'). The reason it")
    print("  APPEARED to exist is a trap worth recording: THE CURRENCIES ARE CIRCULAR -- the")
    print("  fitted exponent reads back whichever spool sits in the excursion's DENOMINATOR:")
    pts = []
    for rho in (0.25, 1.0, 4.0, 8.0):
        fx = ft(rho)
        for r in (0.25, 0.5, 1.0, 2.0):
            e = fx.ramp_excursion_fuel(flight, LO, HI, r)
            if e["complete"]:
                pts.append((r, rho, e))
    print(f"  {'currency':>10} {'denominator':>21} {'best q':>8} {'residual':>10}")
    print("  " + "-" * 53)
    qs = {}
    for key, den in (("E_temp_H", "nu_H running line"), ("X", "none (spool-neutral)"),
                     ("E_temp_L", "nu_L running line")):
        q, s = TwoSpoolFuelTransient.collapse_exponent(pts, key)
        qs[key] = (q, s)
        print(f"  {key:>10} {den:>21} {q:8.2f} {s:10.3f}")
    r0 = TwoSpoolFuelTransient.collapse_exponent(pts, "X", q=0.0)[1]
    r1 = TwoSpoolFuelTransient.collapse_exponent(pts, "X", q=1.0)[1]
    print("  => the HP-REFERENCED currency reads far below the spool-neutral one. So E_temp's")
    print("     q ~ 0 was NEVER evidence that 'the HP clock governs' -- it was the reference")
    print("     reading itself back. Only X is spool-neutral, which is why every magnitude")
    print("     above is quoted in X: THE DATA SELECTED THE INSTRUMENT, not the answer it")
    print(f"     gave. And even on X there is NO collapse: the best exponent cuts the spread")
    print(f"     ~{r0/qs['X'][1]:.1f}x vs q=0 but bottoms out at {100*qs['X'][1]:.0f}% -- points a real clock would put")
    print("     on ONE curve still differ by about a seventh. On the other shape pairs q*(X)")
    print("     and q*(E_temp_L) can TIE (press/flow: 0.45 = 0.45), so only the HP-vs-neutral")
    print("     separation is claimed -- not a strict three-way ordering.")

    print("\n  DELIBERATELY NOT CLAIMED (each was written, probed, and withdrawn):")
    print("    - 'it rides on the geometric-mean composite clock sqrt(det) ~ rho^(-1/2)'. DROPPED:")
    print("      sqrt(det)*sqrt(rho) = const IS a true rung-40 Jacobian identity, but it is not")
    print(f"      connected to the overshoot -- q*(X)={qs['X'][0]:.2f} is the MIDPOINT of the two")
    print(f"      circular currencies ({qs['E_temp_H'][0]:.2f}, {qs['E_temp_L'][0]:.2f}), "
          "an averaging artifact, not evidence for 1/2.")
    print(f"    - 'q=1 is refuted in every currency'. FALSE: on X, q=0 ({r0:.2f}) fits WORSE than")
    print(f"      q=1 ({r1:.2f}). Nothing about the exponent is currency-independent.")
    print("    - 'the overshoot is irreducibly two-dimensional'. OVERCLAIM: only POWER-LAW")
    print("      collapses were tested. The honest statement is that rung 35's single-clock r")
    print("      framing does not extend to two shafts via any effective clock ratio.")
    print("\n  NOT claimed: any magnitude (all ride on rho -- a disclaimed clock group, DOUBLED --")
    print("  on the two representative maps and on the fuel step); a surge-SURVIVAL claim (no")
    print("  surge line on either spool in transient); a TIT redline. Reduce: lp_disabled => rung")
    print("  35 bit-for-bit; Tt4-control untouched => rung 40 bit-for-bit. Cycle stays rung-6 exact.")


def print_transient_surge_table(flight):
    """Rung 44 — THE TRANSIENT TWO-SPOOL SURGE LINE: the excursion is SCHEDULE-slaved, LP eats it.

    Marches rung 40's trajectory against rung 41's imposed surge line. The accel drives BOTH
    spools toward surge, the LP eating ~1.6-2.2x the HP's (rung 41 survives dynamically), but the
    excursion is rho-INVARIANT, ramp-rate-driven, and MODE-INDEPENDENT -- rung 40's two exotic
    objects (rho, the complex mode) are both surge-irrelevant. Sign-space only: phi_surge stays
    imposed, so report the crossing, gate the flip -- no survival claim.
    """
    print("\nTransient two-spool surge line (rung 44): rungs 40 and 41 both deferred this in the")
    print("same words -- march rung 40's trajectory against rung 41's line. On an ACCEL the fuel/Tt4")
    print("step outruns the shaft inertia, so phi dips BELOW the steady running line -- toward surge.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
    FLAT = ComponentMap.flat()

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    def tt(ml=LP, mh=HP, rho=1.0):
        design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                          nozzle_convergent=True, **losses)
        return TwoSpoolTransient(design, flight, 1.0, map_lp=ml, map_hp=mh, rho=rho)

    # --- THE SPLIT SURVIVES DYNAMICALLY, and the mode is IRRELEVANT (hp-only is the tell).
    print("\n  THE SPLIT, DYNAMIC (accel Tt4 1000->1400). ext = extremum of phi(s)-phi_steady(Tt4),")
    print("  referenced to the running line. NEGATIVE = toward surge. hp-only (LP map FLAT) has NO")
    print("  complex mode -- yet the LARGEST LP/HP ratio, so the asymmetry is NOT the mode:")
    print(f"  {'shape pair':>12} {'ext_lp':>9} {'ext_hp':>9} {'|L/H|':>7} {'band?':>7} {'|Im/Re|':>8}")
    print("  " + "-" * 56)
    for name, (ml, mh) in (("flow/press", (LP, HP)), ("hp-only", (FLAT, HP))):
        t = tt(ml, mh)
        e = t.phi_excursion(flight, 1000.0, 400.0)
        band = t.oscillatory_band(flight, 1200.0)
        dr = t.damping_ratio_max(flight, 1200.0)
        print(f"  {name:>12} {e['ext_lp']:+9.4f} {e['ext_hp']:+9.4f} {e['ratio']:7.2f} "
              f"{('yes' if band else 'NONE'):>7} {dr:8.4f}")
    print("  => both spools toward surge, LP eats ~1.9-2.2x. The MODE-IRRELEVANCE claim rests on the")
    print("     DAMPING RATIO: every |Im/Re| < 0.25 (e-folds before a quarter cycle) -> the ring")
    print("     cannot cross a line the steady point clears. The mode-free pair eating the MOST is")
    print("     CORROBORATION (mode not necessary), not proof -- it also swaps LP shaped->flat.")

    # --- SCHEDULE-SLAVED: rho-invariant but ramp-rate-driven.
    print("\n  SCHEDULE-SLAVED (flow/press). Over a 25x rho range the excursion barely moves; over")
    print("  the ramp rate it moves ~5x. rho (which spool LEADS) is powerless; the SLAM RATE governs:")
    print(f"  {'':>14}" + "".join(f"{('rho='+str(x)):>9}" for x in (0.2, 1.0, 5.0)))
    row = "".join(f"{tt(rho=r).phi_excursion(flight,1000.0,400.0)['ext_lp']:9.4f}"
                  for r in (0.2, 1.0, 5.0))
    print(f"  {'ext_lp:':>14}" + row + "   <- <2% spread")
    t = tt()
    print(f"  {'':>14}" + "".join(f"{('r='+str(x)):>9}" for x in (0.1, 0.5, 2.0)))
    row = "".join(f"{t.phi_excursion(flight,1000.0,400.0,r_ramp=r,s_end=6.0)['ext_lp']:9.4f}"
                  for r in (0.1, 0.5, 2.0))
    print(f"  {'ext_lp:':>14}" + row + "   <- faster => deeper")

    # --- REPORT THE CROSSING, GATE THE FLIP.
    print("\n  REPORT THE CROSSING, GATE THE FLIP (rung 36's discipline). Arm phi_surge and place a")
    print("  floor in the gap: the steady point CLEARS it, the transient CROSSES -- on the LP spool:")
    ta = tt(LP.with_phi_surge(0.76), HP.with_phi_surge(0.55))
    m = ta.transient_surge_margin(flight, 1000.0, 400.0, r_ramp=0.3)
    print(f"    steady min LP margin = {m['steady_min_lp']:+.4f}  (clears the phi_surge=0.76 floor)")
    print(f"    transient min LP     = {m['margin_min_lp']:+.4f}  crossed_lp={m['crossed_lp']} "
          f"crossed_hp={m['crossed_hp']}")
    print("  The crossing DEPTH rides on the imposed floor + the ramp (disclaimed); the flip's SIGN")
    print("  (transient below steady, on the LP spool) is the gated object. NO survival claim.")
    print("\n  NOT claimed: any magnitude (phi_surge imposed, DOUBLED; excursion depths ride on the")
    print("  maps + the ramp); the mode's irrelevance is only at |Im/Re|<=0.164, not universal.")
    print("  Reduce: the methods only READ -> rung 40 integrate/equilibrium/jacobian bit-for-bit;")
    print("  Tt4-control (fuel path is the extension). Cycle stays rung-6 exact.")


def print_transient_fuel_surge_table(flight):
    """Rung-45 payoff: rung 44's transient surge line on rung 43's FUEL-controlled plant.

    Rung 44 measured the surge excursion with Tt4 COMMANDED (a clean ramp). On the FUEL path Tt4
    is an OUTPUT that OVERSHOOTS (rung 43), and that overshoot is strongly rho-MONOTONE. THE
    HEADLINE: the overshoot does NOT reach the reference-free surge object -- the raw transient
    min phi stays rho-INVARIANT. The rho signal is real in the PLANT, absent from the SURGE
    MARGIN; it surfaces only in reference-dependent currencies (a currency trap on the surge
    axis). Fuel ENLARGES the approach (rung 35, two shafts) and COMPRESSES the LP-eats-more
    excursion ratio. Pure diagnostic beside the cycle: reads integrate_fuel, bit-for-bit rung 6.
    """
    print("\nTransient surge on the FUEL path (rung 45): rung 44's diagnostic on rung 43's plant,")
    print("where Tt4 FLOATS and OVERSHOOTS. The overshoot is rho-loud; the surge object is rho-quiet.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)

    def ft(ml=LP, mh=HP, rho=1.0):
        return TwoSpoolFuelTransient(design, flight, 1.0, map_lp=ml, map_hp=mh, rho=rho)

    # --- THE HEADLINE: the currency trap.
    print("\n  THE CURRENCY TRAP (flow/press, accel Tt4 1000->1400, r=0.5). Sweep rho: the Tt4 PEAK")
    print("  (the plant) swings hard, the RAW min phi (the surge object) barely moves:")
    print(f"  {'':>12}" + "".join(f"{('rho='+str(x)):>10}" for x in (0.2, 1.0, 5.0)))
    peaks, mins = [], []
    for r in (0.2, 1.0, 5.0):
        e = ft(rho=r).phi_excursion_fuel(flight, 1000.0, 1400.0, r=0.5)
        peaks.append(e["Tt4_peak"])
        mins.append(e["min_phi_lp"])
    print(f"  {'Tt4_peak:':>12}" + "".join(f"{p:10.1f}" for p in peaks) + "   <- ~12% (rho-LOUD)")
    print(f"  {'min_phi_lp:':>12}" + "".join(f"{m:10.4f}" for m in mins) + "   <- <1% (rho-QUIET)")
    print("  => rung 43's rho-monotone overshoot NEVER reaches the surge object. rung 44's 'rho")
    print("     powerless over surge' SURVIVES on the reference-free object -- the currency you pick")
    print("     (output-referenced excursion would read ~40%!) decides whether rho appears to matter.")

    # --- FUEL ENLARGES the approach vs Tt4 control (rung 35 on two shafts).
    print("\n  FUEL ENLARGES the approach (rung 35, two shafts). Same endpoints + ramp, raw min phi_lp:")
    tt4 = TwoSpoolTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0)
    print(f"  {'':>12}" + "".join(f"{('r='+str(x)):>10}" for x in (1.0, 0.5, 0.3)))
    frow = "".join(f"{ft().phi_excursion_fuel(flight,1000.0,1400.0,r=r)['min_phi_lp']:10.4f}"
                   for r in (1.0, 0.5, 0.3))
    trow = "".join(f"{tt4.phi_excursion(flight,1000.0,400.0,r_ramp=r)['min_phi_lp']:10.4f}"
                   for r in (1.0, 0.5, 0.3))
    print(f"  {'fuel:':>12}" + frow + "   <- deeper toward surge (Tt4 overshoot amplifies)")
    print(f"  {'Tt4-ctrl:':>12}" + trow)

    # --- REPORT THE CROSSING, GATE THE FLIP (accel; the raw object is degenerate on a decel).
    print("\n  REPORT THE CROSSING, GATE THE FLIP (rung 36, on the ACCEL). Arm phi_surge, floor in gap:")
    fa = ft(LP.with_phi_surge(0.746), HP.with_phi_surge(0.55))
    m = fa.transient_surge_margin_fuel(flight, 1000.0, 1400.0, r=0.3)
    print(f"    steady min LP margin = {m['steady_min_lp']:+.4f}  (clears the phi_surge=0.746 floor)")
    print(f"    transient min LP     = {m['margin_min_lp']:+.4f}  crossed_lp={m['crossed_lp']} "
          f"crossed_hp={m['crossed_hp']}")
    print("  The LP crosses while the HP clears wide (the strong asymmetry; the excursion RATIO")
    print("  compresses to ~1.2-1.7 vs rung 44's 1.6-2.2). NO survival claim -- phi_surge imposed,")
    print("  tripled. Reduce: reads integrate_fuel -> rung 43 bit-for-bit; cycle stays rung-6 exact.")


def print_topping_governor_table(flight):
    """Rung-46 payoff: the TIT TOPPING GOVERNOR -- the first fuel-side FEEDBACK.

    Rung 43/45 left every fuel accel ending on 'no TIT limit is modelled'. Rung 46 clips the fuel
    to hold Tt4 <= Tt4_max (a min-select, the standard accel-schedule limiter). The governor WORKS
    (pins Tt4 at the redline). THE INVERSION: enforcing the TIT limit rebates surge margin on the
    LATE, non-binding HP spool but MACHINE-ZERO on the EARLY, binding LP spool -- a two-shaft
    SURGE-RELIEF SPLIT. The surge debit is paid on the early-ramp fuel (LP surge min at Tt4~1374,
    upstream of the redline); the governor only trims LATE fuel, too late to claw it back."""
    print("\nTIT topping governor (rung 46): clip fuel to hold Tt4 <= redline (the first fuel-side")
    print("FEEDBACK). Enforcing the TIT limit SPLITS the surge relief -- it rebates the late HP spool")
    print("but MISSES the early, binding LP one. Rung 35's coupled limits are SEQUENCED in time.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
    design = build_two_spool_turbojet(Gas.thermally_perfect(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)
    ft = TwoSpoolFuelTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0)

    # --- THE MECHANISM: the LP surge min LEADS (low Tt4), the Tt4 peak LAGS.
    tj, _ = ft._fuel_ramp_march(flight, 1000.0, 1400.0, 0.5, 2.0, 0.02)
    lp = min(tj, key=lambda p: p["phi_lp"])
    hp = min(tj, key=lambda p: p["phi_hp"])
    pk = max(tj, key=lambda p: p["Tt4"])
    print("\n  THE WINDOW (flow/press, accel Tt4 1000->1400, r=0.5). The limits are SEQUENCED:")
    print(f"    LP surge min : s={lp['s']:.2f}  Tt4={lp['Tt4']:6.1f}  (DURING the ramp, below any redline)")
    print(f"    HP surge min : s={hp['s']:.2f}  Tt4={hp['Tt4']:6.1f}  (LATE, inside the clip window)")
    print(f"    Tt4 peak     : s={pk['s']:.2f}  Tt4={pk['Tt4']:6.1f}")

    # --- THE SPLIT: enforce the redline, difference the surge object per spool.
    print("\n  THE SPLIT (redline Tt4_max=1480, in the gap above the 1400 endpoint). relief>0 = safer:")
    print(f"  {'shape':>12}{'relief_lp':>12}{'relief_hp':>12}{'held':>8}")
    for name, (ml, mh) in (("flow/press", (LP, HP)),
                           ("hp-only", (ComponentMap.flat(), HP))):
        R = TwoSpoolFuelTransient(design, flight, 1.0, map_lp=ml, map_hp=mh, rho=1.0
                                  ).topping_relief(flight, 1000.0, 1400.0, 1480.0, r=0.5, s_settle=2.0)
        print(f"  {name:>12}{R['relief_lp']:>12.5f}{R['relief_hp']:>12.5f}{str(R['held']):>8}")
    print("  => LP relief MACHINE-ZERO (the clip never reaches its early minimum), HP relief >0.")
    print("     Holds even on hp-only (LP flat, NO complex mode) -- the pure WINDOW mechanism.")

    # --- THE LEVER: fast ramp migrates the LP min INTO the clip window.
    print("\n  THE LEVER (Tt4_max=1440): a FASTER ramp lifts the LP surge min above the redline ->")
    print("  relief_lp switches ON (the governor becomes a modest LP-surge lever where it's needed):")
    print(f"  {'':>12}" + "".join(f"{('r='+str(x)):>10}" for x in (0.5, 0.3, 0.15)))
    row = "".join(f"{ft.topping_relief(flight,1000.0,1400.0,1440.0,r=r,s_settle=2.0)['relief_lp']:10.5f}"
                  for r in (0.5, 0.3, 0.15))
    print(f"  {'relief_lp:':>12}" + row)
    print("  Tt4_max imposed -> no redline-level claim; the SPLIT sign is load-bearing. Reduce: dormant")
    print("  (redline above the peak) -> rung 45/43 bit-for-bit; decel never fires; cycle rung-6 exact.")


def print_lagged_governor_table(flight):
    """Rung-47 payoff: the LAGGED topping governor (tau_gov) -- realism REFUTES rung 46's hope.

    Rung 46's governor was an idealised INSTANTANEOUS min-select and closed on 'a slow-enough
    governor could reach earlier into the LP surge point'. Rung 47 gives it a response lag (the
    limiter-loop lag; the clip AMOUNT a 3rd state) and REFUTES that: a first-order lag is a
    TRAILING-edge tool, it cannot reach the EARLY LP surge min. relief_lp stays 0 at moderate r,
    and at fast r (where rung 46's instant governor DID reach the LP) the lag ERODES that relief
    toward 0. The cost of realism: the lag BREAKS the redline hold (overshoot growing with tau_gov)
    and erodes the HP rebate. tau_gov=None is bit-for-bit rung 46; cycle stays rung-6 exact."""
    print("\nLagged topping governor (rung 47): give rung 46's governor a response lag tau_gov. A")
    print("first-order lag is a TRAILING-edge tool -- it REFUTES rung 46's 'a slow governor reaches")
    print("earlier into the LP surge point'. It breaks the redline hold and STILL misses the LP.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)
    ft = TwoSpoolFuelTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0)

    # --- THE COST OF REALISM: overshoot grows, HP rebate erodes, relief_lp pinned at 0.
    print("\n  THE COST OF REALISM (flow/press, accel 1000->1400, r=0.5, redline 1480). Sweep tau_gov:")
    print(f"  {'tau_gov:':>12}" + "".join(f"{t:>10}" for t in ('inst', 0.05, 0.2, 0.8)))
    ov, rlp, rhp = [], [], []
    for tau in (None, 0.05, 0.2, 0.8):
        o = ft.topping_relief(flight, 1000.0, 1400.0, 1480.0, r=0.5, s_settle=2.0, tau_gov=tau)
        ov.append(o["overshoot"]); rlp.append(o["relief_lp"]); rhp.append(o["relief_hp"])
    print(f"  {'overshoot:':>12}" + "".join(f"{v:10.1f}" for v in ov) + "   <- redline HOLD lost (grows)")
    print(f"  {'relief_lp:':>12}" + "".join(f"{v:10.5f}" for v in rlp) + "   <- STILL 0 (misses early LP)")
    print(f"  {'relief_hp:':>12}" + "".join(f"{v:10.5f}" for v in rhp) + "   <- HP rebate ERODES toward 0")

    # --- THE LEVER, LAGGED: at fast r the lag ERODES rung 46's positive LP relief, never enhances.
    print("\n  THE LEVER, LAGGED (r=0.15, redline 1440). rung 46's INSTANT governor reaches the LP")
    print("  (relief_lp>0); if a lag 'reached earlier' it would EXCEED that. It ERODES it instead:")
    print(f"  {'tau_gov:':>12}" + "".join(f"{t:>10}" for t in ('inst', 0.05, 0.2, 0.4)))
    lev = "".join(f"{ft.topping_relief(flight,1000.0,1400.0,1440.0,r=0.15,s_settle=2.0,tau_gov=t)['relief_lp']:10.5f}"
                  for t in (None, 0.05, 0.2, 0.4))
    print(f"  {'relief_lp:':>12}" + lev + "   <- eroded toward 0, never enhanced")
    print("  => neutral at moderate r, strictly WORSE at fast r -- in NO regime does the lag reach the")
    print("     LP better than the ideal. You can't cure a leading-edge problem with a trailing-edge tool.")

    # --- THE SECONDARY: the overshoot lives in the LOOP lag, not the valve.
    tc = ft.topping_command_trace(flight, 1000.0, 1400.0, 1480.0, r=0.5, s_settle=2.0)
    print(f"\n  WHERE THE LAG LIVES: the binding topping command is monotone-rising "
          f"({tc['monotone_nondecreasing']}, {tc['n_engaged']} pts).")
    print("  So a metering-VALVE lag is INERT (instant-up tracks it); the overshoot lives in the")
    print("  sensing/LOOP lag. tau_gov=None -> rung 46 bit-for-bit; dormant/decel -> rung 45; cycle rung-6.")


def print_accel_schedule_table(flight):
    """Rung-48 payoff: the Wf/pt3 ACCELERATION SCHEDULE -- the feedforward leg, and the
    UNIFICATION of rungs 46/47.

    Rungs 46/47 built the FEEDBACK leg (a TIT governor, then its lag) and found it rebates the
    late HP spool but never the early LP one. Rung 48 adds the leg a real FADEC uses: cap Wf by
    (1+m)*kappa_ss(n_H)*pt3, with kappa_ss READ OFF the plant's own running line (the shape is
    DERIVED; the whole imposition is the one scalar m). Because Wf steps up while pt3 can only
    rise as the spools spin up, this leg can engage EARLY -- and m maps continuously to an
    ENGAGEMENT TIME that sweeps ACROSS both surge minima. The finding: a fuel-side limiter
    rebates a spool IFF it engages UPSTREAM of THAT spool's own minimum. That one rule covers
    rung 46 (late by construction) and rung 47 (a lag makes it later still)."""
    print("\nWf/pt3 accel schedule (rung 48): the FEEDFORWARD leg -- cap fuel by the compressor")
    print("delivery pressure. It watches the INPUT, not the output, so it can engage EARLY. Sweeping")
    print("its margin m sweeps the ENGAGEMENT TIME across both surge minima -- and the relief per")
    print("spool switches on IFF the clip lands UPSTREAM of that spool's own minimum.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)
    ft = TwoSpoolFuelTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0)

    # --- THE WINDOW: the bare ratio is already far above the steady line UPSTREAM of the LP min.
    tj, _ = ft._fuel_ramp_march(flight, 1000.0, 1400.0, 0.5, 2.0, 0.02)
    acc0 = ft.accel_schedule(flight, 1000.0, 1400.0, 0.0)
    s_lp = min(tj, key=lambda p: p["phi_lp"])["s"]

    def ratio(p):
        i = ft._instant_fuel(flight, p["nu_lp"], p["nu_hp"], p["mf"])
        return p["mf"] / acc0.cap(i["n_hp"], i["pt4"] / ft.pi_b)

    print("\n  THE WINDOW (flow/press, accel 1000->1400, r=0.5). The bare (Wf/pt3)/kappa_ss ratio")
    print("  rises MONOTONICALLY and clears the steady line LONG before the LP surge min:")
    print(f"  {'s:':>12}" + "".join(f"{p['s']:>9.2f}" for p in tj[:26:5]))
    print(f"  {'ratio:':>12}" + "".join(f"{ratio(p):>9.3f}" for p in tj[:26:5])
          + f"   <- LP surge min at s={s_lp:.2f}")

    # --- THE CROSSING: sweep m, watch each spool's relief die at ITS OWN minimum.
    rows = ft.engagement_sweep(flight, 1000.0, 1400.0, (0.15, 0.35, 0.42, 0.45, 0.48),
                               r=0.5, s_settle=4.0)
    print(f"\n  THE CROSSING (s_lp*={rows[0]['s_lp_bare']:.2f}, s_hp*={rows[0]['s_hp_bare']:.2f}). "
          "relief>0 = safer; fuel_rm = fuel removed:")
    print(f"  {'m':>6}{'s_eng':>8}{'relief_lp':>12}{'relief_hp':>12}{'fuel_rm':>10}{'nu_H end':>10}")
    for x in rows:
        print(f"  {x['margin']:6.2f}{x['s_eng']:8.2f}{x['relief_lp']:12.6f}"
              f"{x['relief_hp']:12.6f}{x['fuel_removed']:10.5f}{x['nu_hp_end']:10.5f}")
    print(f"  => relief_lp dies EXACTLY as s_eng passes s_lp*={rows[0]['s_lp_bare']:.2f} -- while "
          "relief_hp is still")
    print(f"     positive, dying only as s_eng reaches s_hp*={rows[0]['s_hp_bare']:.2f}. ONE rule, "
          "TWO crossings.")
    print("  NOT rung 44's ramp-rate lever: fuel is STILL being removed where the LP gets exactly")
    print("  nothing, the endpoint is unmoved, and one clip splits the two spools. m and phi_surge")
    print("  imposed -> no level claim. Reduces: accel=None -> rungs 45/46/47 bit-for-bit; a dormant")
    print("  schedule/decel -> rung 45; the two-leg min-select -> the single leg; cycle rung-6 exact.")


def print_phi_limiter_table(flight):
    """Rung-49 payoff: the phi / SURGE-MARGIN FEEDBACK limiter -- a limiter acts on a spool
    through BOTH its edges, and the two edges answer to DIFFERENT clocks.

    docs/both-edges-limiter-negative.md closed the whole pt3-FILTER family with one fact: every
    proxy signal (pt3, Wf, n, and any filter of them) rises monotonically through the ramp, so
    such a limiter's release edge is structurally POST-ramp and the closing edge can play no
    part. It named the one escape -- phi has its minimum inside the ramp BY DEFINITION -- and
    this rung walks through it. The floor DOES close inside the ramp, and the closing edge
    turns out to carry the OPPOSITE sign: it re-opens the unwatched spool's descent."""
    print("\nphi surge-margin limiter (rung 49): the first leg that watches the PROTECTED")
    print("variable. Its window CLOSES INSIDE the ramp -- the object no pt3 filter can build --")
    print("and that closing edge DEBITS the spool it is not watching.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)
    ft = TwoSpoolFuelTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0)

    # --- THE SPLIT: one clip, two signs; and both window edges reported.
    rows = ft.floor_sweep(flight, 1000.0, 1400.0, (0.7550, 0.7500, 0.7450, 0.7400),
                          spool="lp", r=0.5, s_settle=2.0)
    s_lp, s_hp = rows[0]["s_lp_bare"], rows[0]["s_hp_bare"]
    print(f"\n  WATCHING THE LP (accel 1000->1400, r=0.5; bare minima s_lp*={s_lp:.2f}, "
          f"s_hp*={s_hp:.2f}).")
    print("  Every row engages UPSTREAM of s_hp* -- rung 48's law predicts a CREDIT on the HP:")
    print(f"  {'phi_lim':>8}{'s_eng':>7}{'s_rel':>7}{'in ramp':>9}{'relief_lp':>12}"
          f"{'relief_hp':>12}{'fuel_rm':>10}")
    for x in rows:
        print(f"  {x['phi_lim']:8.4f}{x['s_eng']:7.2f}{x['s_rel']:7.2f}"
              f"{str(x['both_edges_inside_ramp']):>9}{x['relief_lp']:12.6f}"
              f"{x['relief_hp']:12.6f}{x['fuel_removed']:10.5f}")
    print("  => the WATCHED spool is credited and the OTHER one is DEBITED, by the same clip.")
    print(f"     The unwatched minimum relocates to just AFTER s_rel (not to s_eng): that is")
    print("     where the withheld fuel is handed back to a still-ramping plant.")

    # --- THE DISCRIMINATOR, in miniature: push the release far past the ramp and the sign flips.
    fast = ft.floor_sweep(flight, 1000.0, 1400.0, (0.7500,), spool="lp", r=0.15, s_settle=2.0)[0]
    print(f"\n  THE CLOCK: at r=0.15 the SAME floor releases at s_rel={fast['s_rel']:.2f} = "
          f"{fast['s_rel'] / 0.15:.1f}x the ramp,")
    print(f"  far past the minima -- and the unwatched relief FLIPS to "
          f"{fast['relief_hp']:+.6f}. The credit is")
    print("  clocked by the spool's OWN minimum (rung 48); the debit by the RAMP END (rung 44)")
    print("  -- a WITHIN-FAMILY result. So rung 48 is BOUNDED, not refuted: its credit term")
    print("  survives verbatim and an HP floor reproduces its exact zero. Its own leg is")
    print("  empirically immune to the debit; WHY is an open seam (the ratio does not transfer).")
    print("  phi_lim rides rung 36/41's imposed phi_surge -> signs, not magnitudes.")
    print("  Reduces: surge=None -> rungs 45/46/47/48 bit-for-bit; dormant floor/decel -> rung 45;")
    print("  the min-select composite -> the single binding leg; cycle rung-6 exact.")


def print_release_edge_table(flight):
    """Rung-50 payoff: the RELEASE EDGE, ISOLATED -- the closing edge relocates BOTH spools'
    minima to itself, and a limiter's immunity is TIMING, not clip SHAPE.

    Rung 49 could only move a limiter's release edge by moving phi_lim -- which drags the
    ENGAGEMENT edge, the window length AND the clip depth along with it. So its clock result
    was hedged, correctly, as WITHIN-FAMILY, and it left an open seam: why rung 48's own leg
    escapes the release debit ("the clip SHAPE is the obvious suspect, but it is NOT measured
    here"). This rung builds the instrument that decides both: a FORCED release time s_off,
    an isolation diagnostic in the freeze='lp' tradition, which slides the closing edge ALONE
    and TWO-SIDED with everything up to it bit-identical."""
    print("\nThe release edge, ISOLATED (rung 50): force the leg to let go at s_off, with the")
    print("engagement edge and the clip depth held FIXED -- the axis rung 49 could not build.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)
    ft = TwoSpoolFuelTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0)

    # --- THE HEADLINE + THE DISCRIMINATOR: r=2.0 puts s_hp* and the ramp end 3.1x apart.
    rows = ft.release_sweep(flight, 1000.0, 1400.0, (0.30, 0.66, 1.10, 1.56, 1.80, 2.06, 2.20),
                            surge=SurgeLimiter(spool="lp", phi_lim=0.7725),
                            r=2.0, s_settle=2.0)
    s_lp, s_hp = rows[0]["s_lp_bare"], rows[0]["s_hp_bare"]
    print(f"\n  WATCHING THE LP, phi_lim=0.7725, accel 1000->1400 at r=2.0 (bare minima "
          f"s_lp*={s_lp:.2f}, s_hp*={s_hp:.2f};")
    print("  the ramp end is 2.00, so the two candidate clocks sit 3.1x apart):")
    print(f"  {'s_off':>7}{'s_eng':>7}{'s_rel':>7}{'relief_lp':>12}{'relief_hp':>12}"
          f"{'s@minLP':>9}{'s@minHP':>9}{'fuel_rm':>10}")
    for x in rows:
        print(f"  {x['s_off']:7.2f}{x['s_eng']:7.2f}{x['s_rel']:7.2f}{x['relief_lp']:12.5f}"
              f"{x['relief_hp']:12.5f}{x['s_min_lp']:9.2f}{x['s_min_hp']:9.2f}"
              f"{x['fuel_removed']:10.5f}")
    print("  => s_eng is IDENTICAL in every row, so this axis moves ONLY the release edge.")
    print("     BOTH spools' minima sit AT the release point (the two exceptions are the law's")
    print("     preconditions: row 1 releases upstream of s_hp*, row 7's LP credit branch wins).")
    print("     The debit walks straight THROUGH s_hp* without noticing it and peaks with the")
    print("     release near the RAMP END -- so rung 49's within-family clock hedge LIFTS.")
    print("     And it is not a ramp-rate lever: the LAST row removes the MOST fuel for less")
    print("     than half the peak debit (monotone removal, PEAKED debit).")

    # --- THE SEAM: rung 48's own leg, clip shape unchanged, forced to release inside the ramp.
    acc = ft.accel_schedule(flight, 1000.0, 1400.0, 0.25)
    seam = ft.release_sweep(flight, 1000.0, 1400.0, (0.30, 0.44, 0.50, 9.90),
                            accel=acc, r=0.5, s_settle=2.0)
    print("\n  THE SEAM rung 49 left open -- rung 48's OWN leg (Wf/pt3, m=0.25) at r=0.5,")
    print("  clip SHAPE unchanged, only WHEN it lets go:")
    print(f"  {'s_off':>7}{'s_rel':>7}{'relief_lp':>12}{'relief_hp':>12}")
    for x in seam:
        print(f"  {x['s_off']:7.2f}{x['s_rel']:7.2f}{x['relief_lp']:12.5f}"
              f"{x['relief_hp']:12.5f}")
    print("  => left alone it releases POST-ramp and CREDITS both spools (rung 48's finding).")
    print("     Forced inside the ramp the SAME leg DEBITS both. Its immunity is TIMING, not")
    print("     clip SHAPE -- rung 49's named suspect is refuted and the seam closes.")
    print("  s_off is an ISOLATION DIAGNOSTIC (freeze='lp' tradition), not a control law: it")
    print("  licenses the reading of rungs 48/49's real legs. phi_lim/m ride rungs 36/41/48/49's")
    print("  imposed constants -> signs, ordering and ds-convergence, not magnitudes.")
    print("  Reduces: s_off=None -> rungs 43/45/46/47/48/49 bit-for-bit; a late s_off is inert;")
    print("  an s_off before s_eng is float-for-float BARE; cycle rung-6 exact.")


def print_release_rate_table(flight):
    """Rung-51 payoff: the release RATE -- the debit is NOT a functional of the applied-fuel
    trajectory.

    Rung 50 moved WHEN the withheld fuel is handed back and named its own seam: a finite tau_rel
    would separate total deficit from deficit RATE, "and nothing measured here separates them."
    This rung fades the clip linearly over [s_off, s_off+tau_rel] -- still a pure function of s,
    so rung 50's RK4 argument carries verbatim, and the trigger stays PINNED (an asymmetric lag
    would drag the release time back into the sweep, the confound s_off exists to kill).

    The gate is a TWO-SIDED BRACKET: the two HARD releases at the fade's own two ends bound it
    pointwise in applied fuel AND in total fuel removed. A monotone functional of the fuel level
    would have to land between them."""
    print("\nThe release RATE (rung 51): fade the clip over [s_off, s_off+tau_rel] instead of")
    print("dropping it. Rung 50 moved WHEN the fuel comes back; this moves HOW FAST.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)
    ft = TwoSpoolFuelTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0)
    lim = SurgeLimiter(spool="lp", phi_lim=0.7725)
    K = dict(r=2.0, s_settle=2.0)

    def show(tag, x):
        print(f"  {tag:<26}{x['fuel_removed']:11.6f}{x['relief_lp']:12.5f}"
              f"{x['relief_hp']:12.5f}{x['s_min_hp']:9.2f}")

    print("\n  THE BRACKET (phi floor 0.7725, accel 1000->1400 at r=2.0). The faded row's fuel")
    print("  is POINTWISE between the two hard rows, and so is its total removal:")
    print(f"  {'placement':<26}{'fuel_rm':>11}{'relief_lp':>12}{'relief_hp':>12}{'s@minHP':>9}")
    show("hard   s_off=1.56", ft.release_relief(flight, 1000.0, 1400.0, 1.56, surge=lim, **K))
    show("FADED  1.56, tau_rel=0.20",
         ft.release_relief(flight, 1000.0, 1400.0, 1.56, surge=lim, tau_rel=0.20, **K))
    show("hard   s_off=1.76", ft.release_relief(flight, 1000.0, 1400.0, 1.76, surge=lim, **K))
    print("  => the two HARD rows AGREE (postponing a step release over that interval does")
    print("     essentially nothing) -- yet the faded row between them is ~1.5x SHALLOWER, on")
    print("     BOTH spools. No monotone functional of the fuel LEVEL, and no function of the")
    print("     TOTAL DEFICIT, can land outside a bracket it sits inside. The debit answers to")
    print("     the RATE => rung 50's deficit law is BOUNDED to the instantaneous hand-back.")

    print("\n  THE SCOPE (a negative, stated here and gated): at s_off=0.30 the same")
    print("  construction INTERPOLATES -- deep dives only. There, rate and deficit are not")
    print("  separable and nothing is claimed:")
    print(f"  {'placement':<26}{'fuel_rm':>11}{'relief_lp':>12}{'relief_hp':>12}{'s@minHP':>9}")
    show("hard   s_off=0.30", ft.release_relief(flight, 1000.0, 1400.0, 0.30, surge=lim, **K))
    show("FADED  0.30, tau_rel=0.20",
         ft.release_relief(flight, 1000.0, 1400.0, 0.30, surge=lim, tau_rel=0.20, **K))
    show("hard   s_off=0.50", ft.release_relief(flight, 1000.0, 1400.0, 0.50, surge=lim, **K))

    acc = ft.accel_schedule(flight, 1000.0, 1400.0, 0.15)
    print("\n  CROSS-FAMILY (rung 48's Wf/pt3 leg, m=0.15) -- the violation flips the SIGN:")
    print(f"  {'placement':<26}{'fuel_rm':>11}{'relief_lp':>12}{'relief_hp':>12}{'s@minHP':>9}")
    show("hard   s_off=1.10", ft.release_relief(flight, 1000.0, 1400.0, 1.10, accel=acc, **K))
    show("FADED  1.10, tau_rel=0.40",
         ft.release_relief(flight, 1000.0, 1400.0, 1.10, accel=acc, tau_rel=0.40, **K))
    show("hard   s_off=1.50", ft.release_relief(flight, 1000.0, 1400.0, 1.50, accel=acc, **K))
    print("  => relief_lp is EXACTLY 0 throughout: rung 48's exact-zero law survives the rate")
    print("     axis, three rungs on. NOT claimed: that a slow hand-back is a way to BUILD")
    print("     immunity -- tau_rel fades a clip already forced off at an arbitrary time.")
    print("  Also corrected here: rung 50's precondition (a) ('the release must land at or")
    print("  after that spool's bare minimum') is MIS-STATED -- the crossover sits UPSTREAM of")
    print("  it, and rung 50's own table already violated the condition. Its relocation")
    print("  headline is untouched. NEXT SEAM: the asymmetric fast-attack/slow-release LAG,")
    print("  deferred because its release edge is EMERGENT and moves with the rate.")
    print("  Reduces: tau_rel=None or 0.0 -> rungs 43/45/46/47/48/49/50 bit-for-bit; tau_rel")
    print("  without s_off ASSERTS; a trigger past the natural release is inert; cycle rung-6.")


def print_variable_stator_table(flight):
    """Rung-53 payoff: what a MARGIN IS, when the lever moves the wall.

    Every surge lever up to rung 52 moves the operating point against a FIXED floor. The
    variable stator moves the FLOOR -- and then two reference-free margins that vanish on the
    SAME boundary disagree on whether closing the stators helped: the phi-distance shrinks, the
    incidence distance grows. Only the coordinate in which the boundary is FIXED (incidence,
    T_c = a property of the blade metal) measures a margin. Read there, the model says what
    engineering practice says: closing the stators buys margin.

    Both channels are DERIVED from constants the maps already carry -- psi's swirl term from the
    map's own loading slope l (t2 = l/(1+l)), the floor law from rungs 36/41's own imposed
    phi_surge read as an incidence. Zero new constants."""
    print("\nThe VARIABLE STATOR (rung 53): the first lever that moves the SURGE FLOOR itself.")
    print("Two channels, ONE setting v = tan(alpha_1), both derived from the map's own l and")
    print("rung 36/41's own phi_surge:  psi -= v(1+l)phi   and   phi_surge(v) = phi_s0/(1+v phi_s0).")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)
    vs = VariableStatorMatcher(design, flight, 1.0, map_lp=LP, map_hp=HP)

    print("\n  THE SPLIT (LP stators, fixed throttle Tt4=1500). Both margins are reference-free")
    print("  and vanish TOGETHER; they move in OPPOSITE directions:")
    print(f"  {'v':>7}{'phi_op':>9}{'floor(v)':>10}{'M_phi':>9}{'tan_b1':>9}{'M_i':>9}"
          f"{'SM_N':>9}{'n_LP':>8}")
    for row in vs.stator_sweep(flight, TT4, [-0.2, -0.1, 0.0, 0.1, 0.2, 0.3], spool="lp"):
        d = row["lp"]
        print(f"  {row['vsv']:+7.2f}{d['phi_op']:9.5f}{d['phi_surge']:10.5f}{d['m_phi']:9.5f}"
              f"{d['tan_b1']:9.5f}{d['m_i']:9.5f}{d['sm_n']:9.5f}{d['n']:8.5f}")
    cs = vs.currency_split(flight, TT4, spool="lp")
    print(f"  => dM_phi/dv = {cs['d_m_phi']:+.5f}  (closed form -(1+l)/(2+l)+phi_s0^2 = "
          f"{-(1 + LP.l) / (2 + LP.l) + FLOOR ** 2:+.5f})")
    print(f"     dM_i/dv   = {cs['d_m_i']:+.5f}  (closed form 1/(2+l) = "
          f"{1 / (2 + LP.l):+.5f})   SPLIT = {cs['split']}")

    print("\n  WHY it is the FLOOR's fault, not a metric quirk. For any lever x,")
    print("    sign(dM_phi/dx) = sign(phi' + v' phi_surge^2)   sign(dM_i/dx) = sign(phi' + v' phi_op^2)")
    print("  so at v'=0 they differ only by the STRICTLY POSITIVE Jacobian 1/phi_op^2 and CANNOT")
    print("  split. The control -- throttle alone, stators at design (all three currencies):")
    print(f"  {'Tt4':>7}{'dM_phi':>11}{'dM_i':>11}{'dSM_N':>11}{'agree':>8}"
          f"{'ratio':>9}{'1/phi^2':>10}")
    for r in vs.throttle_currency(flight, [1500.0, 1300.0, 1100.0, 1000.0], spool="lp"):
        print(f"  {r['Tt4']:7.0f}{r['d_m_phi']:+11.6f}{r['d_m_i']:+11.6f}{r['d_sm_n']:+11.6f}"
              f"{str(r['all_three_agree']):>8}{r['ratio']:9.5f}{r['jacobian']:10.5f}")
    print("  => a FLOOR-FIXED lever can never split them: rungs 36-52's phi-currency is BOUNDED,")
    print("     not refuted -- and now licensed by derivation rather than by assumption.")

    print("\n  THE PAYOFF the correct currency makes derivable: the schedule v*(Tt4) that holds")
    print("  the rotor incidence AT ITS DESIGN VALUE (what a real VSV schedule is FOR).")
    print(f"  {'Tt4':>7}{'v*':>8}{'phi_op':>9}{'M_i':>11}{'M_phi':>9}{'M_phi,bare':>12}"
          f"{'SM_N':>9}{'N_L/N_Ld':>10}")
    rows = vs.incidence_schedule(flight, [1500.0, 1300.0, 1100.0, 1000.0], spool="lp", v_hi=1.6)
    for r in rows:
        print(f"  {r['Tt4']:7.0f}{r['vsv_star']:8.4f}{r['phi_op']:9.5f}{r['m_i']:11.7f}"
              f"{r['m_phi']:9.5f}{r['m_phi_bare']:12.5f}{r['sm_n']:9.5f}{r['n']:10.5f}")
    print(f"  => M_i EXACTLY constant (to {abs(max(r['residual'] for r in rows)):.0e}) while M_phi")
    print(f"     falls {(1 - rows[-1]['m_phi'] / rows[0]['m_phi']) * 100:.0f}% -- BELOW its own "
          f"unscheduled value at every point. One trajectory,")
    print("     one boundary, three reference-free distances, three verdicts.")

    print("\n  HONEST SCOPE, two confidence levels. The CURRENCY result is coordinate algebra and")
    print("  rides on no magnitude. The SCHEDULE's numbers do not: holding design incidence at")
    print(f"  Tt4=1000 costs N_L +{(rows[-1]['n'] - 1.0) * 100:.0f}% here, because this lumped "
          f"one-stage map has neither a")
    print("  stator-row FLOW CAPACITY nor a stage stack to rematch -- the channels that carry the")
    print("  real multistage benefit. That capacity channel needs a NEW constant: refused, and")
    print("  named as this rung's seam. Read v* as the swirl a cartoon needs, not a VSV schedule.")


def print_throat_capacity_table(flight):
    """Rung-54 payoff: what a CONSTRAINT costs, and in which coordinate.

    Rung 53 refused the stator's flow-CAPACITY channel because it "needs a new constant (area
    per unit setting)". The cascade cosine rule o/s = cos(alpha_1) derives the area law's SHAPE
    off rung 53's OWN coordinate; only its LEVEL needs one disclosed number, and every verdict
    is delivered as a THRESHOLD on that number.

    Two results. (1) An upstream throat can only BIND, never RELIEVE -- it enters no solver, so
    it cannot buy back rung 53's overspeed, and no area law could. (2) When it does bind it
    costs the SETTING 30% and the MARGIN 4%: rung 53 showed a MARGIN is coordinate-dependent;
    a CONSTRAINT'S SEVERITY is too."""
    print("\nThe STATOR-ROW THROAT (rung 54): the half rung 53 refused. The rotation that buys")
    print("incidence is the rotation that SPENDS THE THROAT -- one coordinate, now three")
    print("channels:  A_th(v)/A_th(0) = cos(alpha_1) = 1/sqrt(1+v^2)   [DERIVED, no constant].")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR, C = 0.55, 0.90

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR).with_capacity(C)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR).with_capacity(C)
    ST = ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2).with_phi_surge(FLOOR).with_capacity(C)
    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)
    vs = VariableStatorMatcher(design, flight, 1.0, map_lp=LP, map_hp=HP)
    steep = VariableStatorMatcher(design, flight, 1.0, map_lp=ST, map_hp=ST)

    print(f"\n  THE ONE CONSTANT, DISCLOSED: C = MFP(M_th0)/MFP(1) = {C:.2f}, i.e. a design "
          f"throat Mach")
    print(f"  M_th0 = {LP.design_throat_mach():.3f}. C=0 means no throat model, as phi_surge=0 "
          f"means no surge line.")
    print("  Everything load-bearing is reported as c_min = 1/X -- a threshold ON C, needing no C.")

    print("\n  THE THROAT COST is TWO-SIDED (cos is even) while the incidence benefit is ONE-sided.")
    print(f"  {'v':>7}{'area':>9}{'m':>9}{'X=m/area':>10}{'c_min':>9}{'M_c':>9}{'M_i':>9}")
    for row in vs.throat_sweep(flight, TT4, [-0.4, -0.2, 0.0, 0.2, 0.4, 0.8], spool="lp"):
        print(f"  {row['vsv']:+7.2f}{row['area']:9.5f}{row['m']:9.5f}"
              f"{row['throat_loading']:10.5f}{row['c_min']:9.5f}{row['m_c']:9.5f}"
              f"{row['m_i']:9.5f}")
    print("  => the throat is MAXIMAL at the design setting and closes whichever way the vane")
    print("     turns; only closing buys incidence. (That the peak sits AT design is inherited")
    print("     from rung 53's coordinate origin, not derived -- see the spec's Concessions.)")

    print("\n  THE HEADLINE: the throat cuts the SETTING hard and the MARGIN barely. Rung 53's")
    print("  own saturation is what makes the limit cheap -- the amputated tail was worth little.")
    print(f"  {'Tt4':>7}{'v_edge':>9}{'v_ch':>8}{'cut%':>7}{'M_i(0)':>9}{'M_i_peak':>10}"
          f"{'M_i_use':>9}{'kept%':>7}{'binds':>8}")
    for T in (1200.0, 1000.0, 800.0):
        a = vs.authority_ceiling(flight, T, spool="lp")
        print(f"  {T:7.0f}{a['v_edge']:9.2f}{a['v_ch']:8.3f}{a['setting_cut'] * 100:7.1f}"
              f"{a['m_i_0']:9.5f}{a['m_i_peak']:10.5f}{a['m_i_usable']:9.5f}"
              f"{a['retained'] * 100:7.1f}{a['binds']:>8}")
    print("  => rung 53 conceded its ceiling was solve_n's bracket, 'a map-validity edge' -- an")
    print("     ARTIFACT. Modelled, the throat beats it on every shape and throttle (20/20).")

    print("\n  BIND, NEVER RELIEVE (a theorem, not a measurement). v enters the solve through")
    print("  solve_n ALONE (rung 53 P1) and the throat enters NO solver, so X is a post-hoc read.")
    bare = VariableStatorMatcher(design, flight, 1.0, vsv_lp=1.1,
                                 map_lp=LP.with_capacity(0.0), map_hp=HP.with_capacity(0.0))
    chk = VariableStatorMatcher(design, flight, 1.0, map_lp=LP, map_hp=HP, vsv_lp=1.1)
    r = chk.throat_margin(flight, TT4)["lp"]
    a1, a0 = chk.match(flight, TT4), bare.match(flight, TT4)
    print(f"  At v=1.10, Tt4=1500 the row is CHOKED (M_c = {r['m_c']:+.5f}), and yet every matched")
    print(f"  number is bit-identical to the no-throat run: thrust {a1.thrust == a0.thrust}, "
          f"n_LP {a1.n_lp == a0.n_lp}, pi_LPC {a1.pi_lpc == a0.pi_lpc}.")
    print("  => an upstream throat REMOVES SETTINGS FROM THE FEASIBLE SET; it cannot change the")
    print("     map from setting to incidence. Rung 53's seam expected capacity to buy back its")
    print("     +26% overspeed: REFUTED, structurally. The real mechanism is STAGE REMATCHING.")

    print("\n  THE RACE, and a CONSTANT-FREE boundary. As power falls the schedule's demand v*")
    print("  RISES while the flow m FALLS. c_min > 1 means the schedule asks LESS of the throat")
    print("  than the DESIGN point -- feasible for EVERY row, whatever its C.")
    print(f"  {'Tt4':>7}{'v*':>8}{'m':>9}{'X(v*)':>9}{'c_min':>9}{'feasible':>10}")
    for r in vs.schedule_throat(flight, [1200.0, 1000.0, 900.0, 870.0, 860.0, 800.0],
                                spool="lp"):
        print(f"  {r['Tt4']:7.0f}{r['vsv_star']:8.3f}{r['m']:9.5f}{r['throat_loading']:9.5f}"
              f"{r['c_min']:9.5f}{str(r['feasible']):>10}")
    print("  => the crossing sits between Tt4 = 870 and 860: rung 53's ENTIRE published band")
    print("     (1000..1500) is above it, so the channel it refused is inert over all of it.")

    print("\n  AND A CORRECTION TO RUNG 53. Its Concessions say the incidence benefit 'does not")
    print("  turn back ... the apparent turning point is NOT reached'. True where it measured;")
    print("  false elsewhere -- on the steep shape the peak is INTERIOR and the schedule DIES:")
    for tag, m in (("flow/press", vs), ("steep", steep)):
        a = m.authority_ceiling(flight, 1000.0, spool="lp")
        sch = m.schedule_throat(flight, [1000.0], spool="lp")[0]
        star = f"{sch['vsv_star']:.3f}" if sch["exists"] else "NONE"
        print(f"    {tag:>11}: peak_interior={str(a['peak_interior']):>5}  v_peak={a['v_peak']:.3f}"
              f"  v_edge={a['v_edge']:.2f}  M_i_peak-M_i_edge={a['m_i_peak'] - a['m_i_edge']:+.5f}"
              f"  v*(1000)={star}")
    print("  => the rung-28 shape: rung 53's VERDICT (finite authority) survives, its REASON is")
    print("     corrected -- the ceiling is the incidence PEAK, not a map-validity edge, and it")
    print("     is reached INSIDE the envelope. It also defeats rung 53's monotone root ladder.")


def print_stage_stack_table(flight):
    """Rung-55 payoff: a lever that moves ROWS, and what it pays for them.

    Rung 54 refuted flow capacity as the escape from rung 53's overspeed and named STAGE
    REMATCHING as the real mechanism. Resolving each compressor into K stage blocks takes that
    seam, and the general law it exposes is about POSITIONAL levers: one that acts on part of a
    machine buys its relief from the part it does not act on, through whatever the parts share
    (here, the shaft speed). So its cost collapses with position -- a front-row stator holds the
    front stage's design incidence for +2.3 % N_L against rung 53's +66.7 %, and the collapse
    FACTORISES as (1/K) x (v* ratio) -- and its benefit has an INTERIOR optimum in how much of
    the machine it moves: relief peaks at 3-4 rows of 8 and then REVERSES, ending worse than
    bare."""
    print("\nThe STAGE STACK (rung 55): the compressor stops being ONE block. With all annuli")
    print("sized so phi_k = 1 at design, the kinematics are DERIVED with no new constant:")
    print("   phi_k = phi_1 (theta_k/theta_k,d)/(varpi_k/varpi_k,d),  n_k = n sqrt(theta_k,d/theta_k)")
    print("and phi_1 = m/n EXACTLY -- the face phi every rung since 32 reads IS the FRONT stage's.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)

    def mk(K_lp=1, K_hp=1, vs_lp=None):
        return StageStackMatcher(design, flight, 1.0, map_lp=LP, map_hp=HP,
                                 K_lp=K_lp, K_hp=K_hp, vsv_stages_lp=vs_lp)

    m8 = mk(8, 8)

    print("\n  ONE MACHINE, TWO OPPOSITE FAILURES (K=8) -- the front stalls while the rear chokes.")
    print("  A lumped block has ONE phi and can represent neither end of it:")
    for Tt4 in (1500.0, 800.0):
        r = m8.stage_margin(flight, Tt4)
        for sp in ("lp", "hp"):
            s = r[sp]
            print(f"   Tt4={Tt4:.0f} {sp.upper()}  phi_k = "
                  + " ".join(f"{st['phi']:.3f}" for st in s["stages"])
                  + f"   rear/front {s['rear_excess']*100:+5.1f}%")
    print("  The LP FRONT stage is the worst incidence in the machine; the HP REAR runs ABOVE")
    print("  design phi -- toward choke and negative incidence.")

    print("\n  WHY THIS IS CONTENT, NOT A RE-READ (the non-tautology gate). The spread above is a")
    print("  functional of the (tau_c, pi_c) rung 39 already solves. The RUNG is the feedback:")
    print("  with a per-stage psi(phi_k) the work is no longer psi(phi_face)*n^2, so the stack")
    print("  MOVES the running line. Marched vs lumped work at the SAME (m, n):")
    print(f"  {'Tt4':>7}{'LP gap':>10}{'HP gap':>10}   (fraction of tau_c-1; EXACTLY 0 at K=1)")
    for Tt4 in (1500.0, 1200.0, 1000.0, 800.0):
        g = m8.work_gap(flight, Tt4)
        print(f"  {Tt4:7.0f}{g['lp']['gap_frac']*100:9.2f}%{g['hp']['gap_frac']*100:9.2f}%")

    print("\n  SO n RISES AND phi FALLS -- and rungs 36-53 are BOUNDED, not refuted: they read the")
    print("  right object (the front stage) but the lumped solve placed it OPTIMISTICALLY.")
    print(f"  {'Tt4':>7}{'d_n LP':>9}{'d_phi LP':>10}{'d_n HP':>9}{'d_phi HP':>10}"
          f"{'d_thrust':>10}")
    for r in m8.running_line_shift(flight, (1500.0, 1200.0, 1000.0, 800.0)):
        print(f"  {r['Tt4']:7.0f}{r['lp']['d_n']*100:8.3f}%{r['lp']['d_phi']*100:9.3f}%"
              f"{r['hp']['d_n']*100:8.3f}%{r['hp']['d_phi']*100:9.3f}%"
              f"{r['d_thrust']*100:9.3f}%")
    print("  Thrust barely moves: like rung 53's stator, the stack is paid in SHAFT SPEED.")

    print("\n  THE HEADLINE -- rung 54's seam discharged. Hold the FRONT stage's design incidence")
    print("  at Tt4=1000 with a FRONT-ROW-ONLY stator (what a real VSV is) instead of rung 53's")
    print("  whole-machine lever. The cost FACTORISES into a positional leg and a setting leg:")
    T = 1000.0
    r53 = VariableStatorMatcher(design, flight, 1.0, map_lp=LP, map_hp=HP)
    row53 = r53.incidence_schedule(flight, [T], spool="lp", v_hi=4.0)[0]
    b53 = r53.at_setting(0.0, 0.0).match(flight, T)
    s53 = r53.at_setting(row53["vsv_star"], 0.0).match(flight, T)
    dN53 = (s53.N_lp_ratio - b53.N_lp_ratio) / b53.N_lp_ratio
    print(f"   rung 53 (one lumped block, EVERY row moves): v* = {row53['vsv_star']:.4f}, "
          f"dN_L = {dN53*100:+.2f}%")
    print(f"  {'K':>4}{'v*front':>10}{'dN_L':>9}{'ratio':>9}{'v*ratio':>9}{'1/K':>8}"
          f"{'(v*ratio)/K':>13}")
    for K in (2, 4, 8, 16):
        m = mk(K, 8, vs_lp=1)
        row = m.stage_incidence_schedule(flight, [T], spool="lp", stage=0, v_hi=4.0)[0]
        bare, sib = m.at_setting(0.0, 0.0), m.at_setting(row["vsv_star"], 0.0)
        b, s = bare.match(flight, T), sib.match(flight, T)
        dN = (s.N_lp_ratio - b.N_lp_ratio) / b.N_lp_ratio
        vr = row["vsv_star"] / row53["vsv_star"]
        print(f"  {K:4d}{row['vsv_star']:10.4f}{dN*100:8.2f}%{dN/dN53:9.4f}{vr:9.4f}"
              f"{1.0/K:8.4f}{vr/K:13.4f}")
    print("  The product law holds to 3 % over an 8x range in K. The 1/K leg is positional; the")
    print("  SETTING leg is that a front-only lever does not FIGHT ITS OWN SPEED RISE.")

    print("\n  AND THE HONEST HALF -- the same law read backwards. Shaft speed is the one thing")
    print("  every stage shares, so relief taken at the front is PAID BY THE ROWS LEFT BEHIND.")
    print("  Sweep how many of the 8 rows the stator moves (target unchanged):")
    m0 = mk(8, 8, vs_lp=1)
    bare0 = m0.at_setting(0.0, 0.0)
    b0 = bare0.match(flight, T)
    mi0 = bare0.stage_margin(flight, T)["lp"]["m_i_worst"]
    print(f"  {'rows':>6}{'v*':>9}{'dN_L':>9}{'worst':>7}{'M_i worst':>11}{'relief':>9}"
          f"{'per dN':>9}")
    print(f"  {'0':>6}{'--':>9}{'--':>9}{0:7d}{mi0:11.4f}{'--':>9}{'--':>9}")
    for rows in (1, 2, 3, 4, 5, 6):
        m = mk(8, 8, vs_lp=rows)
        m._V_SCAN = 0.01
        row = m.stage_incidence_schedule(flight, [T], spool="lp", stage=0, v_hi=4.0)[0]
        sib = m.at_setting(row["vsv_star"], 0.0)
        sm = sib.stage_margin(flight, T)["lp"]
        dN = (sib.match(flight, T).N_lp_ratio - b0.N_lp_ratio) / b0.N_lp_ratio
        rel = (sm["m_i_worst"] - mi0) / mi0
        print(f"  {rows:6d}{row['vsv_star']:9.4f}{dN*100:8.2f}%{sm['worst']:7d}"
              f"{sm['m_i_worst']:11.4f}{rel*100:8.2f}%{rel/dN:9.2f}")
    print("  THE OPTIMUM IS A COUNT -- and it is INTERIOR: relief peaks at 3-4 rows and then")
    print("  REVERSES, ending worse than bare, as the worst stage migrates into the rows the")
    print("  stator does NOT move. Two currencies, two optima: most relief at 4 rows, most")
    print("  relief-per-speed at 1. Rung 53's coordinate law, a third time.")

    print("\n  SCOPE: steady and two-spool. The stack ENTERS THE SOLVER, so unlike rung 54's")
    print("  throat there is no free invariance -- the transient ladders (34/40/43 and the whole")
    print("  limiter family) stay on the lumped loading law, and a test asserts it.")
    print("  REDUCE: an IDENTITY at K = 1 -- no stack object is built and both efficiency loops")
    print("  are the INHERITED rung-39 ones. Measured 0.000e+00 on every matched field.")


def print_per_row_capacity_table(flight):
    """Rung-56 payoff: rung 54's throat channel, resolved onto rung 55's stack.

    Rung 55 named the seam and predicted the rear row would bind. Half right -- and it does NOT
    need a C per row: at design every row has the SAME throat velocity (phi_k = 1 => Vx_k = U)
    while Tt_k climbs, so the design Mach profile is DERIVED off the stack's own ladder and only
    the LEVEL stays disclosed. That derived profile FIGHTS the loading, so the binding row
    MIGRATES -- front near design, rear at part power.

    The headline is where the two constraints land: a resolved machine has TWO worst rows, at
    opposite ENDS and on opposite SPOOLS -- and the one lever that exists reaches only the wrong
    one. A front-row stator restores the row that STALLS and DEBITS the row that CHOKES."""
    print("\nPER-ROW CAPACITY (rung 56): rung 54's throat, per stage. The seam asked for a C per")
    print("row; the ladder supplies all but one. At design phi_k = 1 => Vx_k = U, so every row")
    print("has the SAME throat velocity while Tt_k rises, and it is the total-referenced Mach")
    print("   nu = M/sqrt(1+(g-1)/2 M^2)   that scales:  nu_k = nu_1/sqrt(theta_k,d)")
    print("=> ONE disclosed LEVEL (the front row's C, rung 54's constant) and K-1 rows DERIVED.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR, CAP = 0.55, 0.90
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR).with_capacity(CAP)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR).with_capacity(CAP)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)

    def mk(prof="derived", vl=0.0, vs_lp=None):
        return StageStackMatcher(design, flight, 1.0, map_lp=LP, map_hp=HP, K_lp=8, K_hp=8,
                                 cap_profile=prof, vsv_lp=vl, vsv_stages_lp=vs_lp)

    m8 = mk()
    print(f"\n  the DERIVED profile (C_front = {CAP:.2f} <=> design throat Mach "
          f"{LP.design_throat_mach():.3f}):")
    for sp, st in (("LP", m8.stack_lp), ("HP", m8.stack_hp)):
        cs = st.capacities()
        print(f"   {sp}  C_k = " + " ".join(f"{c:.3f}" for c in cs)
              + f"   C_K/C_1 = {cs[-1]/cs[0]:.4f}   (the HP falls harder: bigger tau_d)")

    print("\n  THE CONTEST -- the rear rows are DESIGNED with more capacity exactly where the")
    print("  off-design march loads them hardest, so which end binds MIGRATES with throttle:")
    grid = (1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 800.0)
    print("     Tt4:      " + " ".join(f"{t:5.0f}" for t in grid))
    for prof in ("derived", "uniform"):
        mm = mk(prof)
        for sp in ("lp", "hp"):
            w = mm.throat_walk(flight, grid, sp)
            print(f"   {prof:7s} {sp.upper()} binds row " + " ".join(
                f"{r['binds']:5d}" for r in w))
    print("  Strip the derived profile ('uniform') and the contest disappears: X_k alone decides")
    print("  and the rear binds everywhere. The migration is the machine's own design Mach law.")

    print("\n  THE HEADLINE -- the machine's two BINDING rows are different rows: different END")
    print("  and different SPOOL. Minima taken over all 16 rows:")
    for Tt4 in (1000.0, 800.0):
        r = m8.stage_throat_margin(flight, Tt4)
        lp, hp = r["lp"], r["hp"]
        print(f"   Tt4={Tt4:.0f}  INCIDENCE worst: LP row {lp['inc_worst']} "
              f"(M_i={lp['m_i_worst']:.4f}, vs HP's {hp['m_i_worst']:.4f})  |  "
              f"CAPACITY worst: HP row {hp['binds']} "
              f"(M_c={hp['m_c_worst']:.4f}, vs LP's {lp['m_c_worst']:.4f})")
    print("  Per spool the ends agree (each front stalls first, each rear chokes first); the")
    print("  CROSS-SPOOL half is the comparison above -- rungs 41/44/45/53 put the surge")
    print("  exposure on the LP, and the CAPACITY exposure is the HP's. A lumped block has one")
    print("  phi and one face and cannot express either half, let alone their separation.")

    print("\n  RUNG 54 CORRECTED BY RESOLUTION -- it wrote 'the HP never approaches its throat at")
    print("  any throttle'. True at the FACE -- and down the columns the face RELAXES with")
    print("  throttle while the rear row TIGHTENS. Opposite signs on the same machine:")
    print("     Tt4      face M_c ->    rear-row M_c ->      C*      [uniform rear]")
    for Tt4 in (1200.0, 1000.0, 800.0):
        r = m8.stage_throat_margin(flight, Tt4)["hp"]
        u = mk("uniform").stage_throat_margin(flight, Tt4)["hp"]
        print(f"    {Tt4:6.0f}      {r['m_c_face']:+.4f}          "
              f"{r['stages'][-1]['m_c']:+.4f}          {r['stages'][-1]['c_min']:.4f}"
              f"      {u['stages'][-1]['m_c']:+.4f}")
    print("  Stated as a THRESHOLD ON the constant (rung 54's discipline): any HP row whose")
    print("  design capacity fraction exceeds 0.913 is CHOKED at Tt4 = 800. A machine pays for")
    print("  its rear rows' off-design capacity AT DESIGN, in the front row's Mach.")

    print("\n  THE LEVER DEBITS THE ROW IT DOES NOT MOVE (front-row stator, LP, Tt4 = 1000) --")
    print("  it reaches the rear only through the shaft speed they share:")
    base = m8.stage_throat_margin(flight, 1000.0)["lp"]
    print(f"     v      rear M_c    debit    front-only/lumped   dN ratio")
    for v in (0.20, 0.3536, 0.60):
        fr = mk(vl=v, vs_lp=1).stage_throat_margin(flight, 1000.0)["lp"]
        lu = mk(vl=v).stage_throat_margin(flight, 1000.0)["lp"]
        d_fr = base["stages"][-1]["m_c"] - fr["stages"][-1]["m_c"]
        d_lu = base["stages"][-1]["m_c"] - lu["stages"][-1]["m_c"]
        print(f"   {v:6.4f}   {fr['stages'][-1]['m_c']:+.5f}   {d_fr:+.5f}        "
              f"{d_fr/d_lu:.4f}         "
              f"{((fr['n']-base['n'])/base['n'])/((lu['n']-base['n'])/base['n']):.4f}")
    print("  The SPEED ratio is nearly v-invariant; the THROAT ratio COLLAPSES. So a positional")
    print("  lever's advantage is CURRENCY-DEPENDENT -- rung 53's law a fourth time, now about")
    print("  the LEVER'S COST. (Pre-registered 'within 25 % of the dN ratio': REFUTED.)")

    print("\n  SCOPE: DIAGNOSTIC ONLY, by rung 54's theorem -- the throat enters no solver, so a")
    print("  choked row changes nothing that is solved (making it BIND would invert rung 31's")
    print("  (*) and is a different rung). REDUCE: an INVARIANCE over the constant AND the")
    print("  profile on a stack that DOES enter the solver, plus K=1 == rung 54 bit-for-bit.")


def print_asymmetric_lag_table(flight):
    """Rung-52 payoff: the asymmetric fast-attack / slow-release LAG -- a self-releasing limiter
    PINS ITS OWN TRIGGER, and CANNOT DEBIT THE SPOOL IT WATCHES.

    Rungs 50/51 forced a release (`s_off`, then a linear fade) because rung 49's family could not
    pin one. Rung 51 deferred the realisable version -- an asymmetric lag -- on the grounds that
    "a lag's release edge is EMERGENT, so sweeping the rate drags the release with it." That is
    FALSE, and refutable in one line: tau_rel is never READ while required > g, so the whole
    march up to the first crossing is bit-identical across a rate sweep.

    Two payoffs follow. The watched spool's minimum sits upstream of that crossing (the lag's
    undershoot is largest EARLY, so rung 48's arrest pins it at the engagement edge), so a
    self-releasing leg cannot debit the spool it watches -- which BOUNDS rung 50's watched-side
    debit to FORCED releases and RESTORES rung 49's identity. And the two constants do NOT
    factor: tau_att owns the credit exactly, the debit is joint."""
    print("\nThe asymmetric fast-attack/slow-release LAG (rung 52): the clip AMOUNT a state with")
    print("TWO time constants. The physically-realisable limiter rungs 50/51 imitated by force.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)
    ft = TwoSpoolFuelTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0)
    lim = SurgeLimiter(spool="lp", phi_lim=0.7725)
    K = dict(r=2.0, s_settle=2.0)

    print("\n  THE TRIGGER PINS ITSELF (phi floor 0.7725, accel 1000->1400 at r=2.0,")
    print("  tau_att=0.02). Sweep the RELEASE constant over 20x -- the crossing does not move:")
    print(f"  {'tau_rel':>9}{'s_cross':>9}{'s_rel(.01)':>12}{'credit_LP':>12}"
          f"{'debit_HP':>12}{'fuel_rm':>11}")
    base = None
    for tr in (0.02, 0.10, 0.40):
        x = ft.lag_relief(flight, 1000.0, 1400.0, AsymmetricLag(0.02, tr), surge=lim, **K)
        base = base if base is not None else x
        print(f"  {tr:9.2f}{x['s_cross']:9.2f}{x['s_rel_0.01']:12.2f}"
              f"{x['relief_watched']:12.6f}{x['relief_other']:12.6f}{x['fuel_removed']:11.5f}")
    print("  => s_cross IDENTICAL and credit_LP identical to MACHINE ZERO, while the hand-back")
    print("     itself stretches out. tau_rel is never READ before the crossing, so the whole")
    print("     pre-crossing march is bit-identical -- the leg pins its own trigger, which is")
    print("     the property rung 50 had to FORCE with s_off. Rung 51's reason 1 is REFUTED.")
    print("  => and the debit SHRINKS while fuel_rm RISES: more fuel removed, smaller debit.")
    print("     Rung 51's headline, on a realisable leg (and far enough out it flips to CREDIT).")

    print("\n  WHY THE CREDIT'S ZERO IS NOT A TAUTOLOGY: it needs the watched spool's own")
    print("  minimum to sit UPSTREAM of the crossing. It does -- the lag's undershoot is")
    print("  largest EARLY, while g is still climbing, so rung 48's arrest pins that minimum at")
    print(f"  the engagement edge: s_min_LP={base['s_min_lp']:.2f} vs s_cross="
          f"{base['s_cross']:.2f} (bare min at 0.32).")
    print("  => A SELF-RELEASING LIMITER CANNOT DEBIT THE SPOOL IT WATCHES. Rung 50's")
    print("     watched-side debit is an ARTIFACT OF FORCING; rung 49's identity is RESTORED")
    print("     for every realisable leg, and rung 50's bound re-scoped to its own instrument.")

    g = ft.factorization_grid(flight, 1000.0, 1400.0, (0.02, 0.20), (0.02, 0.10, 0.40),
                              surge=lim, **K)
    print("\n  DO THE TWO CLOCKS FACTOR? (the premise a real fast-attack/slow-release limiter")
    print("  is DESIGNED on). Additive-separability residual on the DEBIT:")
    print(f"  {'tau_att':>9}" + "".join(f"{tr:>12.2f}" for tr in g["tau_rels"]))
    for i, ta in enumerate(g["tau_atts"]):
        print(f"  {ta:9.2f}" + "".join(f"{v:+12.6f}" for v in g["residual"][i]))
    print(f"  max residual {g['max_residual']:.6f} vs max main effect "
          f"{g['max_main_effect']:.6f} -> {g['max_residual']/g['max_main_effect']:.0%}")
    print("  => the interaction is the SAME ORDER as the main effects (and 70% at r=0.5), while")
    print("     the credit spread is machine zero. THE TWO CLOCKS SEPARATE ONE WAY: tau_att owns")
    print("     the credit EXACTLY, the debit is irreducibly JOINT. The design premise is HALF")
    print("     TRUE -- and the half that fails is the PROTECTIVE one.")
    print("  Rung 51's reason 2 was FORM-dependent: an asymmetric-RATE lag switches on")
    print("  sign(required-g) and both branches share the vanishing numerator, so the RHS is a")
    print("  KINK not a jump -- RK4-legal, and rung 47's latch hazard does not recur. Reason 3")
    print("  STANDS (an exponential never completes): the release edge is DECLARED")
    print("  fractional-of-schedule and every debit is reported at TWO epsilons.")
    print("  phi_lim rides rungs 36/41/49's imposed constant -> signs, machine-zeros and the")
    print("  ds/tau convergences are the claims, not magnitudes. NEXT SEAM: the lag's SHAPE, and")
    print("  the two-lag cascade (tau_gov + lag) this rung refuses -- what a real FADEC runs.")
    print("  Reduces: lag=None -> rungs 45/46/47/48/49/50/51 bit-for-bit; lag+s_off/tau_rel")
    print("  ASSERTS (alternative instruments); lag+tau_gov ASSERTS; cycle rung-6 exact.")


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


def print_super_eq_prompt_table(flight):
    """Rung-19 payoff: super-equilibrium O & prompt NO — LIFTING the equilibrium-O lower bound. Every
    NO number since rung 7 read the rung-6 EQUILIBRIUM [O], so it is a LOWER BOUND. Two lifts, and the
    load-bearing result is that BOTH contradict the naive "the rich primary explodes" intuition:
      • super-eq O — a COMPUTED, T-DRIVEN Westenberg multiplier m(T)∈[1.16,1.50] on the primary [O]
        (φ-independent; WEAKEST in the O2-starved rich primary — the opposite of "rich explosion");
      • prompt NO — an IMPOSED, rich-specific De Soete φ-bump that SURVIVES where thermal dies
        (prompt/thermal grows monotonically rich) and is ~27× less T-sensitive (single vs double exp).
    Pure diagnostic: NO stays trace, cycle bit-for-bit rung 6. TWO honest concessions, stated loudly.
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
    tau = 3e-3
    prompt = PromptNO()

    print("\nSuper-equilibrium O & prompt NO (rung 19): every NO number since rung 7 read the rung-6")
    print("EQUILIBRIUM [O], so it is a LOWER BOUND. Two lifts — and BOTH refute 'the rich primary")
    print("explodes': super-eq O is T-driven (weakest when rich); prompt SURVIVES where thermal dies.")
    print(f"\n  Design point: Tt3={Tt3:.0f} K, Tt4={Tt4:.0f} K, overall far={far:.4f} (φ={far/_F_STOICH:.2f}, LEAN).")

    # (1) the φ_p sweep — thermal (equilibrium-O), super-eq-lifted, prompt, and the prompt/thermal ratio.
    print("\n  (1) φ_p sweep — the two lifts on the primary (EI in g NO/kg fuel):")
    print(f"  {'φ_p':>5} {'T_p':>7} {'EI_therm':>9} {'m(T_p)':>7} {'EI_superO':>10} {'EI_prompt':>10}"
          f" {'EI_total':>9} {'p/therm':>8}")
    print("  " + "-" * 74)
    for phi_p in (0.8, 1.0, 1.1, 1.2, 1.3, 1.5):
        base = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau=tau)
        lift = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau=tau, super_eq_o=True)
        prm = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau=tau, prompt=prompt)
        ratio = prm.ei_no_prompt / base.ei_no if base.ei_no > 0 else float("inf")
        total = lift.ei_no + prm.ei_no_prompt
        print(f"  {phi_p:>5.2f} {base.T_primary:>7.0f} {base.ei_no:>9.4f} {lift.o_multiplier:>7.3f}"
              f" {lift.ei_no:>10.4f} {prm.ei_no_prompt:>10.4f} {total:>9.4f} {ratio:>8.2f}")
    print("  super-eq O: a MODEST T-driven lift (m falls as T rises), and its ABSOLUTE size COLLAPSES on")
    print("  the rich flank WITH thermal — it does NOT rescue the rich primary. prompt/thermal RISES")
    print("  monotonically rich (thermal dies, prompt persists): prompt is the rich-specific lift.")

    # (2) super-eq O is T-DRIVEN not rich — the m(T) correlation, φ-independent.
    print("\n  (2) super-eq O multiplier m(T)=(C2/C1)·T·exp((θ1−θ2)/T) — Westenberg partial-eq/eq O:")
    print(f"  {'T (K)':>7} " + " ".join(f"{T:>7.0f}" for T in (1800, 2000, 2200, 2400, 3000)))
    print(f"  {'m(T)':>7} " + " ".join(f"{_super_eq_o_multiplier(T):>7.3f}" for T in
                                        (1800, 2000, 2200, 2400, 3000)))
    print("  DIMENSIONLESS (the shared [O2]^0.5 cancels) ⇒ a pure function of T, IDENTICAL across φ;")
    print("  DECREASING in T (→1 as T→∞). The lift is T-driven, NOT rich-driven — the intuition's first fail.")

    # (3) the T-sensitivity discriminator — thermal (double exp) vs prompt (single exp).
    far_s = _F_STOICH
    t_rise = (eq.thermal_nox(far_s, 2400.0, p, tau=tau).ei_no
              / eq.thermal_nox(far_s, 2000.0, p, tau=tau).ei_no)
    p_rise = prompt.ei_prompt(1.0, 2400.0) / prompt.ei_prompt(1.0, 2000.0)
    print(f"\n  (3) T-sensitivity 2000→2400 K at stoich: thermal ×{t_rise:.0f} (DOUBLE exp: k1f·[O]_eq)"
          f" vs prompt ×{p_rise:.0f} (SINGLE exp)")
    print(f"  ⇒ prompt is ~{t_rise/p_rise:.0f}× MILDER — the quantitative face of 'survives where thermal dies'.")

    print("\n  TWO honest concessions (stated loudly; docs/rung19-spec.md § the concessions):")
    print(f"  • prompt MAGNITUDE is IMPOSED — a 0-D burnt pool has no flame structure to derive it; the")
    print(f"    scale is back-solved from a REFERENCE EI≈{prompt.peak_ei:.0f} g/kg at (φ={prompt.phi_ref},"
          f" T={prompt.T_ref:.0f} K), so the delivered peak lands near ~{prompt.peak_ei:.0f} g/kg — but a")
    print("    hotter primary (T_p>T_ref) nudges it up (single exp). Only the φ-SHAPE + the directional")
    print("    prompt/thermal ratio are certified. De Soete valid φ≤1.6 — the deep-rich flank is flagged.")
    print("  • super-eq O RATIO is semi-empirical — a full-equilibrium pool cannot self-yield super-eq O;")
    print("    the lift lives in Westenberg's fitted constants (cross-validated to ~5%, the units gate).")


def print_super_eq_quench_table(flight):
    """Rung-20 payoff: super-equilibrium O THROUGH the quench — lifting the finite-quench lower bound.

    Rung 19 lifted the equilibrium-O lower bound only on the PRIMARY (ei_no/x_no_mix). The finite-quench
    fields (ei_no_quenched, ei_no_pocket_quench) and the rung-17 clamp margins a still RE-MADE NO on
    equilibrium O — still lower bounds. Rung 20 threads the same Westenberg m(T) lift INSIDE the
    _quench_no re-making. The load-bearing result INVERTS the naive headline: because the Zeldovich
    re-making peaks at the HOTTEST stoich crossing where m(T) is at its MINIMUM, the lift is MODEST &
    PEAK-CONCENTRATED (≈m(T_peak)) — even SMALLER than the primary lift. The certified spine: the
    rung-17 a-margins RISE (numerator lifts) while the thermodynamic denominator x_no_e(T9) does NOT.
    Pure diagnostic: NO stays trace, cycle bit-for-bit rung 6.
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4, st9 = real.stations["3"], real.stations["4"], real.stations["9"]
    far, Tt3, Tt4, p = st4.far, st3.Tt, st4.Tt, st4.pt
    J, phi_p, tau = 225.0, 1.5, 3e-3
    ng, ns = 24, 200                                       # coarse (illustrative panel) — SHAPE, not digits
    mix = JetMixing(J=J, C_e=0.20, shape_n=2.0)
    pq = PocketQuenchPDF(S=0.0625, n_bell=24, n_quad=96)

    print("\nSuper-equilibrium O THROUGH the quench (rung 20): rung 19 lifted the equilibrium-O lower")
    print("bound only on the PRIMARY. The finite-quench fields RE-MADE NO on equilibrium O — still lower")
    print("bounds. Rung 20 threads the m(T) lift INSIDE the _quench_no re-making, closing that seam.")
    print(f"\n  Design point (rich φ_p={phi_p}, J={J:.0f}): quench cools through the stoich crossing.")

    # (1) the effective lift — bulk quench + per-pocket, eq-O vs super-eq-O.
    b0 = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau, mixing=mix, quench_ngrid=ng, quench_nsteps=ns)
    bL = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau, mixing=mix, super_eq_o=True,
                      quench_ngrid=ng, quench_nsteps=ns)
    p0 = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau, mixing=mix, pocket_quench=pq,
                      quench_ngrid=ng, quench_nsteps=ns)
    pL = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau, mixing=mix, pocket_quench=pq, super_eq_o=True,
                      quench_ngrid=ng, quench_nsteps=ns)
    # the PRIMARY lift (rung 19, ideal quench) for the contrast.
    z0 = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau)
    zL = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau, super_eq_o=True)
    print("\n  (1) the effective lift (EI in g NO/kg fuel):")
    print(f"  {'field':>26} {'eq-O':>9} {'super-eq-O':>11} {'factor':>7}")
    print("  " + "-" * 56)
    print(f"  {'ei_no_quenched (bulk)':>26} {b0.ei_no_quenched:>9.4f} {bL.ei_no_quenched:>11.4f}"
          f" {bL.ei_no_quenched/b0.ei_no_quenched:>7.3f}")
    print(f"  {'ei_no_pocket_quench (r16)':>26} {p0.ei_no_pocket_quench:>9.4f} {pL.ei_no_pocket_quench:>11.4f}"
          f" {pL.ei_no_pocket_quench/p0.ei_no_pocket_quench:>7.3f}")
    print(f"  {'primary ei_no (rung 19)':>26} {z0.ei_no:>9.4f} {zL.ei_no:>11.4f}"
          f" {zL.ei_no/z0.ei_no:>7.3f}")

    # (2) WHY the quench lift is SMALLER — the re-making peaks where m is smallest.
    print(f"\n  (2) the quench lift ({bL.ei_no_quenched/b0.ei_no_quenched:.3f}) is SMALLER than the primary"
          f" lift ({zL.ei_no/z0.ei_no:.3f}) because the")
    print(f"  Zeldovich re-making peaks at the HOTTEST crossing T_peak={b0.T_peak:.0f} K (hotter than the")
    print(f"  flame T_p={z0.T_primary:.0f} K), and m(T) is SMALLEST where hottest: m(T_peak)="
          f"{_super_eq_o_multiplier(b0.T_peak):.3f} vs m(T_p)={_super_eq_o_multiplier(z0.T_primary):.3f}.")
    print(f"  The cool tail carries large m (m({Tt4:.0f} K)={_super_eq_o_multiplier(Tt4):.3f}) but makes"
          f" NEGLIGIBLE NO — the lift is PEAK-concentrated.")

    # (3) the certified spine — the rung-17 a-margins rise, the denominator does not.
    c0 = eq.exhaust_no_clamp(far, Tt3, Tt4, p, st9.Tt, st9.pt, real.p9, phi_primary=phi_p,
                             mixing=mix, pocket_quench=pq, quench_ngrid=ng, quench_nsteps=ns)
    cL = eq.exhaust_no_clamp(far, Tt3, Tt4, p, st9.Tt, st9.pt, real.p9, phi_primary=phi_p,
                             mixing=mix, pocket_quench=pq, super_eq_o=True, quench_ngrid=ng, quench_nsteps=ns)
    print("\n  (3) the CERTIFIED SPINE — the rung-17 clamp margins a=[NO]/[NO]_e(T9) rise, denominator fixed:")
    print(f"  {'margin':>22} {'eq-O':>9} {'super-eq-O':>11}")
    print(f"  {'a_mixed_out':>22} {c0.a_mixed_out:>9.4f} {cL.a_mixed_out:>11.4f}   (primary lift; stays ≪1)")
    print(f"  {'a_bulk_quench':>22} {c0.a_bulk_quench:>9.3f} {cL.a_bulk_quench:>11.3f}")
    print(f"  {'a_pocket':>22} {c0.a_pocket:>9.3f} {cL.a_pocket:>11.3f}")
    print(f"  {'denom x_no_e(T9)':>22} {c0.x_no_e_exit:>9.3e} {cL.x_no_e_exit:>11.3e}   "
          f"{'← BIT-IDENTICAL' if c0.x_no_e_exit == cL.x_no_e_exit else '← MOVED (bug)'}")
    print("  The numerators lift (kinetic NO); the denominator is a THERMODYNAMIC ceiling Kp_NO·√(x_N2·x_O2)")
    print("  — NOT set by the O-atom closure — so every a rises by a BOUNDED factor. rung 17's a were lower")
    print(f"  bounds. But the clamp still does NOT fire at station 4: max_a={cL.max_a_quench:.2f}<1 — super-eq O")
    print("  speeds FORMATION, not the [NO]_e collapse, so it is not the burner-clamp lever (a slow freeze is).")

    print("\n  HONEST SCOPE: the super-eq RATIO stays semi-empirical (rung 19) ⇒ the lifted a is")
    print("  better-justified but NOT pinned. Prompt rides the quench as an INVARIANT per-kg-fuel EI")
    print("  (kept OUT of a — imposed magnitude). The ideal-bell PDF integrals (rung 13/15/18) stay")
    print("  equilibrium-O here — RUNG 21 lifts them consistently (see the next panel).")


def print_ideal_bell_lift_table(flight):
    """Rung-21 payoff: super-equilibrium O through the IDEAL-BELL PDF integrals — the last eq-O seam.

    Rung 20 lifted everything through _quench_no but LEFT the three ideal-bell composition integrals
    (ei_no_pdf r13, ei_no_pdf_quench term2 r15, ei_no_transported r18) on equilibrium O, FORBIDDING the
    combination because ei_no_pdf_quench = term1+term2 would be a HALF-LIFTED HYBRID. Rung 21 threads the
    same m(T) through the ideal bell so BOTH terms lift — the hybrid dissolves. The load-bearing result is
    the rung-20 INVERSION generalized: the bell EI(φ) is PEAKED near stoich (hottest ⇒ m smallest), the
    β-PDF integral is EI-weighted onto that peak, so the lift is ≈×1.15 — BELOW the primary ×1.28 and even
    DECREASING with segregation. Pure diagnostic: NO stays trace, cycle bit-for-bit rung 6.
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    far, Tt3, Tt4, p = st4.far, st3.Tt, st4.Tt, st4.pt
    hf = eq.hf_fuel_molar if eq.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
    J, phi_p, tau = 36.0, 1.5, 3e-3                        # J=36: a moderate over-penetration flank (g>0)
    ng, nb, nq = 24, 80, 100                               # coarse (illustrative panel) — SHAPE, not digits
    mix = JetMixing(J=J)
    xibar = far / (1.0 + far)

    print("\nSuper-equilibrium O through the IDEAL-BELL PDF integrals (rung 21): rung 20 lifted everything")
    print("through _quench_no but LEFT the composition integrals (pdf r13 / pdf_quench-term2 r15 /")
    print("transported r18) on equilibrium O — a FORBIDDEN combination (a half-lifted hybrid). Rung 21")
    print("threads m(T) through the ideal bell too, so BOTH pdf_quench terms lift and the hybrid dissolves.")
    print(f"\n  Design point (lean mean φ≈{far/_F_STOICH:.2f}, rich primary φ_p={phi_p}, J={J:.0f}, g>0).")

    def run(**kw):
        return eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau, mixing=mix, quench_ngrid=ng, **kw)

    pd = dict(pdf=MixingPDF(S=0.0625, n_bell=nb, n_quad=nq))
    qp = dict(pdf_quench=QuenchPDF(S=0.0625, n_bell=nb, n_quad=nq))
    tp = dict(transported=TransportedPDF(S=0.0625, n_bell=nb, n_quad=nq))
    p0, pL = run(**pd), run(super_eq_o=True, **pd)
    q0, qL = run(**qp), run(super_eq_o=True, **qp)
    t0, tL = run(**tp), run(super_eq_o=True, **tp)
    b0, bL = run(), run(super_eq_o=True)                   # the rung-20 bulk term1, for the composite check

    print("\n  (1) the effective ideal-bell lift (EI in g NO/kg fuel):")
    print(f"  {'field':>28} {'eq-O':>9} {'super-eq-O':>11} {'factor':>7}")
    print("  " + "-" * 58)
    print(f"  {'ei_no_pdf (rung 13)':>28} {p0.ei_no_pdf:>9.4f} {pL.ei_no_pdf:>11.4f}"
          f" {pL.ei_no_pdf/p0.ei_no_pdf:>7.3f}")
    print(f"  {'ei_no_pdf_quench (rung 15)':>28} {q0.ei_no_pdf_quench:>9.4f} {qL.ei_no_pdf_quench:>11.4f}"
          f" {qL.ei_no_pdf_quench/q0.ei_no_pdf_quench:>7.3f}")
    print(f"  {'ei_no_transported (rung 18)':>28} {t0.ei_no_transported:>9.4f} {tL.ei_no_transported:>11.4f}"
          f" {tL.ei_no_transported/t0.ei_no_transported:>7.3f}")

    # (2) WHY it is peak-concentrated — the point value / peak / primary spread.
    lift_point = (_ideal_bell_ei(far, p, Tt3, hf, tau, super_eq_o=True)
                  / _ideal_bell_ei(far, p, Tt3, hf, tau, super_eq_o=False))
    fl_p = phi_p * _F_STOICH
    lift_primary = (_ideal_bell_ei(fl_p, p, Tt3, hf, tau, super_eq_o=True)
                    / _ideal_bell_ei(fl_p, p, Tt3, hf, tau, super_eq_o=False))
    lift_pdf = pL.ei_no_pdf / p0.ei_no_pdf
    print(f"\n  (2) WHY it is PEAK-CONCENTRATED — the bell EI is peaked near stoich (hottest ⇒ m smallest):")
    print(f"  {'deep-lean point value (g→0, φ≈'+format(far/_F_STOICH,'.2f')+')':>40}  lift ×{lift_point:.2f}"
          f"   (cool flame ⇒ m LARGE, but EI≈0)")
    print(f"  {'primary flame (φ_p='+format(phi_p,'.1f')+')':>40}  lift ×{lift_primary:.2f}"
          f"   (rung 19)")
    print(f"  {'EI-weighted ⟨EI⟩_pdf (the number that counts)':>40}  lift ×{lift_pdf:.2f}"
          f"   (SMALLEST — onto the stoich peak)")
    print("  The naive 'the bell spans cool deep-lean pockets where m→1.9' is WRONG: those carry ≈0 EI.")

    # (3) the HYBRID resolved — the composite pdf_quench sits between its two terms.
    lift_composite = qL.ei_no_pdf_quench / q0.ei_no_pdf_quench
    lift_bulk = bL.ei_no_quenched / b0.ei_no_quenched
    print(f"\n  (3) the HYBRID RESOLVED — pdf_quench composite ×{lift_composite:.3f} sits BETWEEN term1")
    print(f"  (bulk quench ×{lift_bulk:.3f}, rung 20) and term2 (ideal bell ×{lift_pdf:.3f}, rung 21) —")
    print("  the measured proof BOTH terms now carry m(T). rung 20's forbidden combination is now VALID.")

    print("\n  HONEST SCOPE: a shape-preserving CONSISTENCY lift — the optimum LOCATION (pinned AT C_opt),")
    print("  the (H/S)² shift and the stoich-mean sign reversal are UNMOVED; only the magnitude lifts ≈×1.15.")
    print("  The super-eq RATIO stays semi-empirical (rung 19); prompt stays a primary-only invariant EI.")


def print_spatial_pdf_table(flight):
    """Rung-22 payoff: the RESOLVED cross-plane / spatial PDF — the INVERSION of rung 18.

    Rung 18's load-bearing result was NEGATIVE: a 0-D variance transport CANNOT derive the Holdeman C_opt
    optimum (mean-field ω ⇒ monotone g(J)), so it had to IMPOSE the coverage ω(C) — the spacing S by hand.
    Rung 22 resolves the y-z dilution cross-plane and C_opt EMERGES as an OUTPUT: the penetration
    δ=k_p·√(S·H)·J^(1/4) couples S in, a fixed mixing length + far-wall reflection penalize
    over-penetration, and the uniformity optimum is where δ fills half the height ⇒ (S/H)√J=1/(4k_p²),
    S,H-independent. The g_min VALUE is geometry-independent (the collapse); only J_opt moves, as (H/S)².
    Honest emissions: through the ideal bell, C_opt is only a LOCAL ⟨EI⟩ min (the derived floor sits just
    below the hump peak — a narrow basin); the GLOBAL min is at max segregation. Pure diagnostic, rung 6.
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    far, Tt3, Tt4, p = st4.far, st3.Tt, st4.Tt, st4.pt
    phi_p, tau = 1.5, 3e-3
    ng, nb, nq = 24, 64, 160                               # coarse-ish (illustrative panel) — SHAPE, not digits
    #                                                        (nq≥160 keeps the β-PDF mean-preserving on the
    #                                                        over-penetration flank's larger g)
    k_p = SpatialPDF().k_p
    gceil = _two_stream_ceiling(far, phi_p)

    print("\nResolved cross-plane / spatial PDF (rung 22): the INVERSION of rung 18. Rung 18 proved a 0-D")
    print("variance transport CANNOT derive the Holdeman C_opt (mean-field ⇒ monotone g(J)); it had to")
    print("IMPOSE the coverage ω(C) — the spacing S by hand. Rung 22 RESOLVES the y-z cross-plane and C_opt")
    print("comes out as an OUTPUT: δ=k_p·√(S·H)·J^(1/4) couples S in; δ fills half-height ⇒ (S/H)√J=1/(4k_p²).")
    print(f"\n  (1) THE COLLAPSE — vary S and H INDEPENDENTLY; g_min is geometry-independent, J_opt ∝ (H/S)²,")
    print(f"      C_opt=1/(4·k_p²)={1.0/(4.0*k_p**2):.3f} an OUTPUT (Holdeman's ≈2.5). No C_opt is fed in.")
    print(f"  {'S':>8} {'H':>6} {'J_opt':>7} {'C_opt(OUT)':>11} {'g_min':>8}  note")
    print("  " + "-" * 62)
    Js = [1.0 * (400.0) ** (i / 60) for i in range(61)]
    notes = {(0.0625, 0.10): "baseline", (0.03125, 0.10): "halve S ⇒ J_opt ×4",
             (0.125, 0.10): "double S ⇒ J_opt ÷4", (0.0625, 0.20): "double H ⇒ J_opt ×4",
             (0.125, 0.20): "S/H fixed ⇒ unchanged"}
    for (S, H), note in notes.items():
        best = min(((_spatial_segregation(far, phi_p, S, H, J), J) for J in Js), key=lambda t: t[0])
        gmin, Jo = best
        print(f"  {S:8.4f} {H:6.2f} {Jo:7.2f} {(S/H)*(Jo**0.5):11.3f} {gmin:8.4f}  {note}")
    print("  ⇒ the g_min VALUE is the same everywhere (the collapse); only the LOCATION J_opt shifts as (H/S)².")

    # (2) the rung-18 tie: g_spatial < g_ceiling always.
    print(f"\n  (2) g_spatial < g_ceiling={gceil:.4f} (rung-18 two-stream ceiling) — a partial-mix field is")
    print("      LESS segregated than the two-δ extreme (the one thing rung 18 derived bounds the resolved g):")
    sp = SpatialPDF(S=0.0625, n_bell=nb, n_quad=nq)
    for J in (1, 16, 400):
        st = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau, mixing=JetMixing(J=float(J), H=0.10),
                          spatial=sp, quench_ngrid=ng)
        print(f"      J={J:>4}  g_spatial={st.g_spatial:.4f}  (< {gceil:.4f})")

    # (3) emissions, honest: C_opt LOCAL min but GLOBAL min at max segregation.
    print(f"\n  (3) EMISSIONS (honest): through the ideal bell, C_opt (J_opt=16) is only a LOCAL ⟨EI⟩ min;")
    print("      the GLOBAL min is at max segregation (rung-13's descending far flank, spatialized):")
    print(f"  {'J':>6} {'C':>6} {'g_spatial':>10} {'ei_no_spatial':>14}  flank")
    print("  " + "-" * 52)
    ei, g_floor = {}, None
    for J in (9, 16, 25, 100, 400):                        # 9/25 the IMMEDIATE flanks (both up); 100/400 the far descent
        st = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau, mixing=JetMixing(J=float(J), H=0.10),
                          spatial=sp, quench_ngrid=ng)
        ei[J] = st.ei_no_spatial
        if J == 16:
            g_floor = st.g_spatial
        C = (0.0625 / 0.10) * (J ** 0.5)
        flank = "C_opt (local min)" if J == 16 else ("immediate flank ↑" if J in (9, 25)
                                                     else "far over-pen ↓")
        print(f"  {J:6d} {C:6.2f} {st.g_spatial:10.4f} {st.ei_no_spatial:14.4f}  {flank}")
    amin = min(ei, key=ei.get)
    print(f"  ⇒ local min AT J=16 (both immediate flanks up), but the GLOBAL min is at J={amin} (an ENDPOINT):")
    print(f"     the derived floor g(C_opt)≈{g_floor:.3f} sits JUST below the ideal-bell hump peak")
    print("     (≈0.021), so the C_opt basin is NARROW — which is WHY UNIFORMITY (g), not emissions, is the")
    print("     headline. rung 18 was NOT wrong: it reported the real LOCAL behaviour.")

    print("\n  HONEST SCOPE: the VALUE C_opt≈2.5 rides on the semi-empirical k_p (only the COLLAPSE + the")
    print("  (H/S)² shift are derived); rung 22 derives the WIDTH g(C), NOT the DWELL (rung-16 kink imported);")
    print("  the field is a Gaussian-plume cartoon feeding the β-PDF closure — a real PDF-transport/CFD")
    print("  cross-plane (the full shape + the dwell spectrum + rung-17's firing MAGNITUDE) stays the ceiling.")


def print_dwell_spectrum_table(flight):
    """Rung-23 payoff: the DERIVED DWELL SPECTRUM — completing rung-22's partial closure.

    Rung 22 derived the cross-plane WIDTH g(C) but fed it through the per-pocket quench with the IMPORTED
    rung-16 KINKED scalar dwell (baking C_opt in). Rung 23 develops the SAME cross-plane in TIME (over
    rung-11's τ_mix) so each pocket carries its OWN dwell τ(ξ) — rich pockets arrive LATE ⇒ dwell LONG.
    The one genuinely new quantity is the ξ–τ CORRELATION (rung-16's scalar τ_core has zero correlation),
    isolated by a MATCHED-MEAN twin (τ(ξ) spectrum vs a scalar ⟨τ⟩): corr_ratio>1 ⇒ the correlation ADDS
    NO, one-signed while formation-limited (max_a<1), concentrated under-penetration. Honest: the absolute
    magnitude/trend rides on the un-anchored τ_mix; the emissions C_opt pin is NOT recovered (the derived τ
    FALLS off-optimum — but rung 16 already declined the global-min location). Pure diagnostic, rung 6.
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    far, Tt3, Tt4, p = st4.far, st3.Tt, st4.Tt, st4.pt
    phi_p, tau = 1.5, 3e-3
    # n_quad=120 (not 56): the β-PDF mean-preservation assert fails on the C_opt-climb — g is
    # non-monotone in J (collapses to g_min at C_opt≈2.5, climbs back), and the quadrature is
    # hardest to converge on that climb, worst near J≈36. 56 only survives because the grid below
    # steps 16→64 over that point; 120 holds every C (verified <0.5% drift including J=36).
    ng, nb, nq = 24, 40, 120
    cfg = SpatialDwellPDF(S=0.0625, ny=32, nz=32, nt=24, n_bell=nb, n_quad=nq)

    print("\nDerived dwell spectrum (rung 23): completing rung-22's PARTIAL closure. Rung 22 derived the")
    print("cross-plane WIDTH g(C) but imported rung-16's KINKED scalar dwell (which bakes C_opt in). Rung 23")
    print("develops the SAME cross-plane in TIME (over rung-11's τ_mix) so each pocket carries its OWN dwell")
    print("τ(ξ) — rich pockets arrive LATE ⇒ dwell LONG. NO C_opt, NO τ_res, NO b_u; the scale is rung-11's τ_mix.")

    print("\n  (1) THE ξ–τ CORRELATION (the certified positive) — the MATCHED-MEAN twin isolates it: term2 with")
    print("      the correlated τ(ξ) vs term2 with a scalar ⟨τ⟩_PDF (same g, same mean dwell). corr_ratio>1 ⇒")
    print("      rich pockets dwell long ⇒ ADD NO. Concentrated under-penetration, fading toward C_opt:")
    print(f"  {'J':>5} {'C':>6} {'g':>7} {'τ_mix(ms)':>10} {'⟨τ⟩(ms)':>9} {'ei_corr':>9} {'ei_mean':>9} "
          f"{'corr/mean':>9} {'max_a':>6}")
    print("  " + "-" * 84)
    for J in (4, 9, 16, 64, 400):
        st = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau, mixing=JetMixing(J=float(J), C_e=0.20, U_c=75.0, H=0.10),
                          spatial_dwell=cfg, quench_ngrid=ng)
        C = (0.0625 / 0.10) * (J ** 0.5)
        taumix = JetMixing(J=float(J), C_e=0.20, U_c=75.0, H=0.10).tau_q
        note = "under-pen" if J < 16 else ("C_opt" if J == 16 else "over-pen")
        print(f"  {J:5d} {C:6.2f} {st.g_spatial_dwell:7.4f} {taumix*1e3:10.3f} {st.tau_mean_dwell*1e3:9.4f} "
              f"{st.ei_no_spatial_dwell:9.4f} {st.ei_no_spatial_dwell_meanfield:9.4f} "
              f"{st.corr_ratio:9.4f} {st.max_a_quench:6.3f}  {note}")
    print("  ⇒ corr/mean > 1 EVERYWHERE (the correlation adds NO), largest under-penetration; max_a<1 (formation-")
    print("    limited, so the sign never flips). This is the physics rung-16's SCALAR τ_core cannot express.")

    # (2) the divergence — rung-16 imposed τ_core GROWS off-optimum; rung-23 derived ⟨τ⟩ FALLS ∝1/√J.
    print("\n  (2) THE DIVERGENCE (honest) — rung-16's imposed τ_core GROWS off-optimum (|ln(C/C_opt)|); the")
    print("      derived ⟨τ⟩ FALLS ∝1/√J (rung-11 mixing time). ~3×–70× apart, opposite-trending, BOTH un-anchored:")
    cfg16 = PocketQuenchPDF(S=0.0625)
    print(f"  {'J':>5} {'C':>6} {'rung16 τ_core(ms)':>17} {'rung23 ⟨τ⟩(ms)':>15} {'ratio':>7}")
    print("  " + "-" * 54)
    for J in (4, 16, 64, 400):
        m = JetMixing(J=float(J), C_e=0.20, U_c=75.0, H=0.10)
        C = cfg16.C(m)
        t16 = cfg16.core_dwell(C)
        g_s, tau_of = _spatial_dwell_field(far, phi_p, cfg.S, m.H, m.J, m.tau_q,
                                           k_p=cfg.k_p, k_y=cfg.k_y, k_z=cfg.k_z, ny=32, nz=32, nt=24)
        xibar = far / (1.0 + far)
        nodes, wts = _beta_pdf_nodes_weights(xibar, g_s, n_quad=nq)
        t23 = sum(wi * tau_of(x) for wi, x in zip(wts, nodes))
        print(f"  {J:5d} {C:6.2f} {t16*1e3:17.4f} {t23*1e3:15.4f} {t16/t23:7.1f}")

    print("\n  HONEST SCOPE: rung 23 derives the correlation's SHAPE (sign + under-penetration concentration);")
    print("  the absolute MAGNITUDE/TREND rides on rung-11's un-anchored τ_mix (as rung-22's C_opt≈2.5 rides on")
    print("  k_p). The emissions C_opt pin is NOT recovered (the derived τ falls off-optimum) — but rung 16")
    print("  ALREADY declined the global-min location. A LOCALLY-resolved mixing time (each cell its own rate)")
    print("  — plus the full CFD-PDF shape, which would let rung 17 claim a firing MAGNITUDE — stays the ceiling.")


def print_local_mixing_table(flight):
    """RUNG 24 — the LOCALLY-RESOLVED mixing time (docs/rung24-spec.md).

    Rungs 11–23 all ran on ONE GLOBAL τ_mix. Rung 23's §9 named this successor and hypothesized it
    "could restore an off-optimum dwell GROWTH that pins the emissions optimum non-circularly". Rung 24
    ASKS that. Each cell relaxes at its OWN gradient-derived rate ω=D_t|∇ξ|²/var (D_t=σ²/(2τ_mix) REUSED
    — no new constant); the terminal field stays rung-22's, so g is identical BY CONSTRUCTION. τ_mix
    CANCELS out of the rate ⇒ ⟨τ⟩ = τ_mix(J)·F(C) EXACTLY. The answer is a SPLIT: F(C) IS U-shaped with
    its min AT C_opt (the derived dwell growth — rung 16 vindicated in SHAPE), but ~40% against τ_mix's
    ~20× swing ⇒ ⟨EI⟩ stays MONOTONE (the emissions pin still not recovered — rung 23 vindicated in the
    PRODUCT). Rung 24 localizes the RATE, not the SCALE. Pure diagnostic, cycle bit-for-bit rung 6.
    """
    eq = Gas.reacting_equilibrium()
    real = build_turbojet(eq, PI_C, TT4, flight.p0, **REAL_LOSSES).run(flight, 1.0)
    st3, st4 = real.stations["3"], real.stations["4"]
    far, Tt3, Tt4, p = st4.far, st3.Tt, st4.Tt, st4.pt
    phi_p, tau = 1.5, 3e-3
    ng, nb, nq = 24, 40, 160     # n_quad=160: 56 trips the KNOWN β-PDF guard on the C_opt climb (rung-23 §7)
    cfg = SpatialLocalPDF(S=0.0625, ny=32, nz=32, n_bell=nb, n_quad=nq)

    print("\nLocally-resolved mixing time (rung 24): the ceiling rungs 11–23 all deferred BY NAME. Every one of")
    print("them ran ONE GLOBAL τ_mix; rung 23's §9 asked whether giving each cell its OWN rate would restore an")
    print("off-optimum dwell GROWTH and pin the emissions optimum non-circularly. Rung 24 builds it and ASKS.")
    print("ω = D_t·|∇ξ|²/var with D_t = σ²/(2τ_mix) — REUSED, so NO new constant, NO C_opt, NO τ_res, NO b_u.")

    print("\n  (1) THE FACTORIZATION — τ_mix CANCELS out of u=ω·τ_mix=σ²|∇ξ|²/(2var), so the shape is a PURE")
    print("      FIELD FUNCTIONAL and ⟨τ⟩(J) = τ_mix(J)·F(C) EXACTLY. Scale × shape, cleanly separated:")
    # NOTE the two weightings, deliberately labelled apart: F_cell/⟨τ⟩_cell here are the SPATIAL
    # cell-means (the grid-converged field functional the tests gate); panel (3) prints ⟨τ⟩_PDF, the
    # β-PDF-weighted mean that actually feeds the chemistry. BOTH are U-shaped and min at C_opt, but
    # they are different numbers (J=4: 0.392 vs 0.318) — never compare across the two tables.
    print(f"  {'J':>5} {'C':>6} {'g (==r22)':>10} {'τ_mix(ms)':>10} {'F_cell':>12} {'⟨τ⟩_cell(ms)':>13}")
    print("  " + "-" * 60)
    for J in (1, 4, 9, 16, 36, 64, 144, 400):
        m = JetMixing(J=float(J), C_e=0.20, U_c=75.0, H=0.10)
        g_s, tau_of, F = _spatial_local_field(far, phi_p, cfg.S, m.H, m.J, m.tau_q,
                                              k_p=cfg.k_p, k_y=cfg.k_y, k_z=cfg.k_z, ny=32, nz=32)
        C = (cfg.S / m.H) * (J ** 0.5)
        note = "  ← C_opt (F MINIMAL)" if J == 16 else ""
        print(f"  {J:5d} {C:6.2f} {g_s:10.5f} {m.tau_q*1e3:10.3f} {F:12.4f} {m.tau_q*F*1e3:13.4f}{note}")
    print("  ⇒ F is U-SHAPED with its min AT C_opt — the off-optimum dwell GROWTH rung 16 IMPOSED as")
    print("    τ_res·(1+b_u|ln(C/C_opt)|), here DERIVED from the plume's own gradients. But ⟨τ⟩=τ_mix·F still")
    print("    FALLS monotonically: F's ~1.4× U loses to τ_mix's 20× 1/√J swing. THE SCALE SWAMPS THE SHAPE.")

    print("\n  (2) THE KILL TEST (why the U is NOT circular) — ω carries an explicit 1/g (var=g·ξ̄(1−ξ̄)) and")
    print("      rung 22 ALREADY mins g at C_opt, so 'argmin F == argmin g' is a TELL, not a confirmation.")
    print("      ⟨|∇ξ|²⟩ carries NO g algebraically — and IT is maximal at C_opt. The gradients place it:")
    print(f"  {'J':>5} {'C':>6} {'g':>9} {'⟨|∇ξ|²⟩ (g-free)':>17}")
    print("  " + "-" * 42)
    for J in (4, 9, 16, 36, 64):
        m = JetMixing(J=float(J), C_e=0.20, U_c=75.0, H=0.10)
        g_s, _, _ = _spatial_local_field(far, phi_p, cfg.S, m.H, m.J, m.tau_q, ny=32, nz=32)
        gsq = _mean_grad_sq(far, phi_p, cfg.S, m.H, m.J)
        note = "  ← C_opt (STEEPEST)" if J == 16 else ""
        print(f"  {J:5d} {(cfg.S/m.H)*(J**0.5):6.2f} {g_s:9.5f} {gsq:17.4f}{note}")
    print("  ⇒ At C_opt the residual structure sits at the plume's OWN scale σ (fine ⇒ steep ⇒ fast ⇒ SHORT")
    print("    dwell); off-optimum the air piles into WALL-SCALE slabs (coarse ⇒ shallow ⇒ slow ⇒ LONG dwell).")
    print("    HONEST: that fine-vs-coarse behaviour is a property of the FIXED-σ plume CARTOON, not a general law.")

    print("\n  (3) THE NEGATIVE HEADLINE — on the REAL per-pocket chemistry (NOT inferred from ⟨τ⟩): ⟨EI⟩ stays")
    print("      MONOTONE, so the emissions C_opt pin is STILL NOT recovered:")
    print(f"  {'J':>5} {'C':>6} {'F_cell':>7} {'⟨τ⟩_PDF(ms)':>12} {'EI_local':>9} {'EI_meanfld':>10} "
          f"{'corr':>6} {'max_a':>6}")
    print("  " + "-" * 68)
    for J in (4, 9, 16, 36, 64):
        st = eq.zoned_nox(far, Tt3, Tt4, p, phi_p, tau,
                          mixing=JetMixing(J=float(J), C_e=0.20, U_c=75.0, H=0.10),
                          spatial_local=cfg, quench_ngrid=ng)
        C = (cfg.S / 0.10) * (J ** 0.5)
        note = "under-pen" if J < 16 else ("C_opt" if J == 16 else "over-pen")
        print(f"  {J:5d} {C:6.2f} {st.f_shape:7.4f} {st.tau_mean_local*1e3:12.4f} {st.ei_no_spatial_local:9.4f} "
              f"{st.ei_no_spatial_local_meanfield:10.4f} {st.corr_ratio_local:6.3f} {st.max_a_quench:6.3f}  {note}")
    print("  ⇒ ⟨EI⟩ falls MONOTONICALLY through C_opt — no emissions optimum, even with the rate localized.")
    print("    (corr>1 throughout ALSO re-derives rung-23's ξ–τ correlation from INDEPENDENT physics: gradient")
    print("     structure, not arrival time. Rich pockets dwell longest either way.)")

    print("\n  THE ADJUDICATION: rung 23 left an explicit fork — does the off-optimum dwell GROW (rung-16,")
    print("  imposed) or FALL (rung-23, derived)? 'Neither is pinned from data.' It resolves BOTH WAYS, in")
    print("  DIFFERENT FACTORS: the SHAPE grows (rung 16 vindicated, and now DERIVED), the PRODUCT still falls")
    print("  (rung 23 vindicated). Rung-16's kink is NOT an artifact — it is real and MIS-SCALED.")
    print("  HONEST SCOPE: rung 24 localizes the RATE, not the SCALE — ⟨τ⟩'s magnitude still rides on rung-11's")
    print("  un-anchored τ_mix and F's on rung-22's k_p. So rung-23 §9's hope that this lets rung 17 claim a")
    print("  firing MAGNITUDE is NOT delivered (it buys a sharper DIRECTION only) — CORRECTED, not inherited.")


def _mean_grad_sq(far_overall, phi_primary, S, H, J, k_p=0.316, k_y=0.28, k_z=0.28, ny=32, nz=32):
    """⟨|∇ξ|²⟩ of the rung-22 terminal field — the rung-24 kill test's G-FREE witness (no variance
    normalization anywhere, so it cannot inherit rung-22's g-minimum at C_opt). See rung24-spec §2."""
    xibar = far_overall / (1.0 + far_overall)
    far_p = phi_primary * _F_STOICH
    xi_p = far_p / (1.0 + far_p)
    delta = k_p * math.sqrt(S * H) * J ** 0.25
    sig_y, sig_z = k_y * H, k_z * S
    ys = [(i + 0.5) * H / ny for i in range(ny)]
    zs = [(j + 0.5) * S / nz for j in range(nz)]
    ay = [sum(math.exp(-((y - c) ** 2) / (2 * sig_y ** 2))
              for c in (-delta, delta, 2 * H - delta, 2 * H + delta)) for y in ys]
    az = [sum(math.exp(-((z - S / 2 - m * S) ** 2) / (2 * sig_z ** 2)) for m in (-1, 0, 1)) for z in zs]
    may, maz = sum(ay) / ny, sum(az) / nz
    ayh, azh = [a / may for a in ay], [a / maz for a in az]
    beta_bar = (xi_p - xibar) / xi_p
    lo, hi = 0.0, 50.0
    for _ in range(60):
        s = 0.5 * (lo + hi)
        m_ = sum(xi_p * (1.0 - min(1.0, max(0.0, s * beta_bar * a * b)))
                 for a in ayh for b in azh) / (ny * nz)
        if m_ > xibar:
            lo = s
        else:
            hi = s
    s_star = 0.5 * (lo + hi)
    xi = [[xi_p * (1.0 - min(1.0, max(0.0, s_star * beta_bar * a * b))) for b in azh] for a in ayh]
    dy, dz = H / ny, S / nz
    tot = 0.0
    for i in range(ny):
        im, ip = max(0, i - 1), min(ny - 1, i + 1)
        for j in range(nz):
            jm, jp = (j - 1) % nz, (j + 1) % nz
            gy = (xi[ip][j] - xi[im][j]) / ((ip - im) * dy)
            gz = (xi[i][jp] - xi[i][jm]) / (2 * dz)
            tot += gy * gy + gz * gz
    return tot / (ny * nz)


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


def print_stator_schedule_table(flight):
    """Rung-57 payoff: rung 53's floor-moving lever, put inside an acceleration.

    Rungs 46-52 spent seven rungs learning that a fuel-side limiter's credit is a CLOCK -- when
    it engages relative to the spool's own surge minimum, when it releases, how fast. All of
    those move the OPERATING POINT against a fixed wall. This one moves the WALL, and the clock
    disappears: across a 20x ramp-rate range the margin swings 52 % while the share of the
    stator's rotation that survives moves one point, and rung 53's DESIGN-POINT Jacobian
    predicts that share to 3.9 % -- off design, out of equilibrium, mid-transient.

    Two thirds of the rotation never arrives (the lever's own work channel), and a state-fed
    SCHEDULE self-cancels 10-25 % because closing the stators raises the very speed it reads."""
    print("\nTHE STATOR SCHEDULE ON THE TRANSIENT PLANT (rung 57): the first lever that moves the")
    print("surge FLOOR *during* an accel. Rungs 46-52's levers all move the POINT, and every one")
    print("of them was credited by a CLOCK. This one is credited by a MAP:")
    print("   erosion = 1 - (net credit)/(pointwise credit)   <- the share the WORK channel eats")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR, V, LO, HI = 0.55, 0.20, 1000.0, 1400.0
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)

    def mk(**kw):
        return ScheduledStatorTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)

    rs = (0.1, 0.25, 0.5, 1.0, 2.0)
    rows = [mk(vsv_lp=V).stator_credit(flight, LO, HI, r=r) for r in rs]

    print(f"\n  A CONSTANT setting v = {V:.2f} on the LP spool, over a 20x range of ramp rate r:")
    print(f"    {'r':>5} {'bare M_i':>10} {'credit':>9} {'credit/v':>9} {'erosion':>9}")
    for r, d in zip(rs, rows):
        print(f"    {r:5.2f} {d['bare']:+10.5f} {d['credit']:9.5f} {d['credit'] / V:9.5f} "
              f"{d['erosion']:9.4f}")
    bare = [d["bare"] for d in rows]
    er = [d["erosion"] for d in rows]
    cf = 1.0 - rows[0]["closed_form"]
    print(f"    the MARGIN swings {100 * (max(bare) - min(bare)) / min(bare):.1f} %; the EROSION "
          f"moves {100 * (max(er) - min(er)):.2f} points.")
    print(f"    rung 53's design-point closed form 1 - 1/(2+l) = {cf:.4f}  "
          f"(max error {100 * max(abs(e - cf) for e in er) / cf:.1f} %)")
    print("    => the credit is a MAP property. A wall-moving lever has NO CLOCK.")

    n_lo = mk().equilibrium(flight, LO)["nu_lp"]
    sc = StatorSchedule(V, n_lo)
    dec = [mk(vsv_sched_lp=sc).credit_decomposition(flight, LO, HI, r=r) for r in rs]
    print(f"\n  And a state-fed SCHEDULE v(n) (closed at low speed, 0 at design speed n=1):")
    print(f"    {'r':>5} {'FULL':>9} {'START-only':>11} {'RAMP-only':>10} {'share START':>12} "
          f"{'FULL/RAMP':>10}")
    for r, d in zip(rs, dec):
        print(f"    {r:5.2f} {d['full']:+9.5f} {d['start']:+11.5f} {d['ramp']:+10.5f} "
              f"{d['share_start']:12.3f} {d['self_cancel']:10.3f}")
    print(f"    nu0_L rises {dec[0]['nu0_bare']:.4f} -> {dec[0]['nu0_armed']:.4f} when the "
          f"stators close (rung 53: paid in SHAFT SPEED)")
    print("    => the schedule READS that higher speed and opens back up: it SELF-CANCELS.")

    st = mk().arrow_toggle(flight, LO, HI, V, spool="lp")
    print(f"\n  CORRECTING rung 53's P5, at ONE fixed transient state:")
    print(f"    v_LP = {V:.2f}  ->  d_phi_LP {st['d_phi_lp']:+.4e}   "
          f"d_phi_HP {st['d_phi_hp']:+.4e}   d_Tt25 {st['d_Tt25']:+.2f} K")
    print("    rung 53 measured d_phi_HP == +0.000e+00 EXACTLY on the STEADY cascade and called")
    print("    the LP stator 'a pure-LP lever, bit-for-bit'. It is not, once the shaft speeds are")
    print("    STATES: the steady balance re-solved n_H and absorbed the Tt25 shift; rung 40 took")
    print("    that balance away. The break survives a FLAT-eta island, so it is NOT rung 53's")
    print("    eta-mediated arrow -- it is the ENERGY channel, and d_Tt25 names it.")
    print("\n  SCOPE: LP-side (the HP schedule reads shaft speed -- Tt25 is an output of the root")
    print("  it must be armed before); eta_c_at is stator-inert; no head-to-head against the")
    print("  fuel-side limiters (no common currency). See docs/rung57-spec.md § Concessions.")


def print_composite_minselect_table(flight):
    """Rung-58 payoff: rung 57's own next seam -- the stator schedule BESIDE a fuel-side leg.

    Rung 57 refused the head-to-head (fuel withheld and shaft speed paid have no common
    currency). A MIXED SECOND DIFFERENCE needs none: how much does the stator's credit change
    when a fuel leg is armed beside it? The answer is 9.5 %, it runs ONE WAY, and it lives in
    the SCHEDULE's state-feed -- a constant setting sits an order of magnitude down.

    And the obvious over-reading is refused by the same sweep: the DELIVERED credit is flatter
    in r than the bare one, so rung 57 is CONFIRMED, not bounded."""
    print("\nTHE COMPOSITE MIN-SELECT (rung 58): rung 57's lever and ONE fuel-side leg on ONE")
    print("plant. Four cells and their MIXED SECOND DIFFERENCE -- no ranking of the two levers,")
    print("which is why it dodges the head-to-head rung 57 declared trapped:")
    print("   dI = [M_i(both) - M_i(fuel)] - [M_i(stator) - M_i(neither)]")
    print("   M_i = T_c - (1/phi - v)   <- wall = the METAL, the ONE object all four cells share")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR, V, LO, HI = 0.55, 0.20, 1000.0, 1400.0
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)

    def mk(**kw):
        return ScheduledStatorTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)

    acc = mk().accel_schedule(flight, LO, HI, 0.25)     # DERIVED ONCE, on the BARE machine
    sc = StatorSchedule(V, mk().equilibrium(flight, LO)["nu_lp"])
    sch = mk(vsv_sched_lp=sc).composite_credit(flight, LO, HI, r=0.5, accel=acc)
    con = mk(vsv_lp=V).composite_credit(flight, LO, HI, r=0.5, accel=acc)

    print(f"\n  The four cells (state-fed schedule, rung 48's Wf/pt3 leg, r = 0.5):")
    print(f"    {'cell':>8} {'M_i':>10} {'s*':>7} {'v(s*)':>7} {'M_phi':>10}")
    for k in ("neither", "stator", "fuel", "both"):
        c = sch["cells"][k]
        print(f"    {k:>8} {c['m_i']:10.6f} {c['s']:7.4f} {c['v']:7.4f} {c['m_phi']:+10.6f}")
    print(f"    stator credit  bare {sch['credit_bare']:+.6f}  ->  with the fuel leg "
          f"{sch['credit_fuel']:+.6f}   dI = {sch['interaction']:+.6f} "
          f"({100 * sch['share']:+.2f} %)")

    e = mk(vsv_sched_lp=sc).engagement_shift(flight, LO, HI, r=0.5, accel=acc)
    print(f"\n  ... and the CONVERSE, sub-grid: the fuel leg's own engagement time")
    print(f"    s_eng  bare {e['bare_dormant']:.7f}  ->  stator armed {e['armed_dormant']:.7f}"
          f"   ({100 * e['rel_dormant']:+.3f} %)")
    print(f"    => the influence runs ONE WAY, by a factor of "
          f"{abs(sch['share'] / e['rel_dormant']):.0f}.")

    print(f"\n  WHERE it lives -- the same composite with a CONSTANT setting, which cannot"
          f" self-feed:")
    print(f"    {'stator leg':>12} {'credit':>10} {'dI':>10} {'share':>8} {'v(s*) ratio':>12}")
    for tag, d in (("schedule", sch), (f"constant {V:.2f}", con)):
        print(f"    {tag:>12} {d['credit_bare']:10.6f} {d['interaction']:+10.6f} "
              f"{100 * d['share']:7.2f}% {d['v_ratio']:12.5f}")
    print(f"    the fuel leg RELOCATES the incidence minimum ({sch['cells']['neither']['s']:.4f}"
          f" -> {sch['cells']['fuel']['s']:.4f}) and a state-fed schedule is MORE CLOSED there.")
    print(f"    A constant setting is read at the same v wherever the minimum goes.")
    print(f"    => the two levers DO NOT SUPERPOSE. (The {100 * con['share']:.2f} % floor is"
          f" REAL, not zero, and at r = 0.10 it flips SIGN.)")

    print("\n  AND THE OBVIOUS READING IS WRONG -- the near-miss, published with the result.")
    print(f"  dI is strongly ramp-rate-dependent, which invites 'a clock-free lever INHERITS")
    print(f"  its partner's clock'. But dI ANTI-correlates with the bare credit, so what a")
    print(f"  designer is handed -- credit_bare + dI -- is FLATTER in r than the bare credit:")
    print(f"    {'stator leg':>12} {'bare spread':>12} {'composed spread':>16}")
    print(f"    {'schedule':>12} {'8.53 %':>12} {'6.80 %':>16}")
    print(f"    {'constant':>12} {'3.11 %':>12} {'0.89 %':>16}")
    print(f"    => rung 57 is CONFIRMED on the delivered credit. Only the DECOMPOSITION is")
    print(f"       clocked, and a decomposition is not a deliverable.")

    print(f"\n  PREDICTED from the two marches that never saw the fuel leg -- the credit is a")
    print(f"  PROFILE in s, and the leg changes only WHICH POINT of it is read:")
    for tag, d in (("schedule", sch), (f"constant {V:.2f}", con)):
        print(f"    {tag:>12}  measured {d['interaction']:+.6f}   predicted {d['predicted']:+.6f}"
              f"   ({100 * d['predicted'] / d['interaction']:.0f} % recovered)")

    print(f"\n  AND A LEG THAT CANNOT COMPOSE AT ALL. Rung 49's phi floor must sit BELOW the")
    print(f"  machine's phi at s=0 and ABOVE its minimum. Those windows are DISJOINT:")
    for tag, kw in (("bare", {}), ("schedule", dict(vsv_sched_lp=sc)),
                    ("constant", dict(vsv_lp=V))):
        m = mk(**kw)
        tr, _ = m._stator_march(flight, LO, HI, 0.5, 1.2, 0.005)
        print(f"    {tag:>9}  admissible floor  ({min(p['phi_lp'] for p in tr):.4f}, "
              f"{tr[0]['phi_lp']:.4f})")
    print("    => rung 53 made a MARGIN coordinate-dependent; this is the same fact reaching a")
    print("       LIMITER'S SET POINT. Rung 48's leg composes because Wf/pt3 is stator-invariant.")

    print("\n  SCOPE: the fuel leg is ONE object derived on the BARE machine (rung 59 proves this")
    print("  IS the matched leg for an LP stator); LP-side, M_i only, one")
    print("  gas. See docs/rung58-spec.md § Concessions.")


def print_matched_floor_table(flight):
    """Rung-60 payoff: rung 58's own next seam -- the MATCHED phi floor.

    Rung 58 found a phi floor not composable with the stator (disjoint set-point bands) and
    proposed matching the set point. Matching is under-determined; the canonical repair is a
    CHANGE OF COORDINATE, to the one wall the stator does not move. It buys admissibility --
    and nothing else, because a floor PINS the coordinate it watches, so the composite's
    second difference is a difference of SET POINTS with a DERIVED value."""
    print("\nTHE MATCHED phi FLOOR (rung 60): rung 58's refused repair, built -- and it answers")
    print("the wrong question. A floor that binds holds its own coordinate AT the set point:")
    print("   leg floors phi   ->  M_i(both) - M_i(fuel) = [T_c - 1/phi_lim + v] - [..+0] = v")
    print("   leg floors M_i   ->  M_i(both) - M_i(fuel) = m_lim - m_lim               = 0")
    print("   (LP spool, r = 0.5, ds = 0.01 -- the spec's tables are the ds = 0.005 grid)")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR, LO, HI, DS = 0.55, 1000.0, 1400.0, 0.01
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0,
                                      nozzle_convergent=True, **losses)

    def mk(**kw):
        return ScheduledStatorTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)

    print("\n  BOTH ENDS OF THE TAUTOLOGY, measured against the value they are DERIVED to take:")
    print(f"    {'leg':>22} {'v':>5} {'regime':>13} {'M_i(both)-M_i(fuel)':>21} {'derived':>9}"
          f" {'residual':>10}")
    for tag, v, leg in ((" incidence M=0.509", 0.10, IncidenceLimiter(spool="lp", m_lim=0.509)),
                        (" incidence M=0.518", 0.15, IncidenceLimiter(spool="lp", m_lim=0.518)),
                        ("   phi floor 0.750", 0.15, SurgeLimiter(spool="lp", phi_lim=0.750))):
        d = mk(vsv_lp=v).floor_composite(flight, LO, HI, leg, r=0.5, ds=DS)
        print(f"    {tag:>22} {v:5.2f} {d['regime']:>13} {d['credit_fuel']:21.15f}"
              f" {d['pinned_prediction']:9.3f} {d['pinned_residual']:+10.1e}")
    print("    => a number reproduced to 1e-15 by an identity is not evidence about the machine.")
    print("       Re-referencing MOVES the tautology; only a leg that RELOCATES a minimum")
    print("       (rung 48's schedule) leaves the derivative to the plant.")

    print("\n  WHAT IT DOES BUY -- ADMISSIBILITY. The set-point bands, in both coordinates:")
    d = mk(vsv_lp=0.20).set_point_bands(flight, LO, HI, r=0.5, ds=DS)
    print(f"    phi   gap {d['gap_phi']:+.6f}  = {100 * d['gap_phi_bands']:+6.1f} % of a band"
          f"   admissible: {d['phi_admissible']}")
    print(f"    M_i   gap {d['gap_m']:+.6f}  = {100 * d['gap_m_bands']:+6.1f} % of a band"
          f"   admissible: {d['m_admissible']}")
    print(f"    and the gap is an IDENTITY: credit {d['credit']:.6f} - excursion "
          f"{d['excursion']:.6f} = {d['criterion']:+.6f}  (residual {d['identity_residual']:+.1e})")

    print("\n  ... so composability has a CLOCK, and the clock is entirely the RAMP's:")
    rows = mk().composability_ladder(flight, LO, HI,
                                     rates=[(r, dict(vsv_lp=0.20)) for r in (0.15, 0.35, 0.5, 1.0)],
                                     ds=DS)
    print(f"    {'r':>6} {'credit':>10} {'excursion':>10} {'criterion':>11}  composable")
    for row in rows:
        print(f"    {row['r']:6.2f} {row['credit']:10.6f} {row['excursion']:10.6f} "
              f"{row['criterion']:+11.6f}  {row['m_admissible']}")
    cr = [row["credit"] for row in rows]
    ex = [row["excursion"] for row in rows]
    print(f"    credit spread {100 * (max(cr) / min(cr) - 1):.2f} %  (rung 57: a wall-moving "
          f"lever has NO clock)")
    print(f"    excursion spread {max(ex) / min(ex):.2f}x  -- the ramp's, and it is what crosses")
    print("       the threshold with the lever's setting never touched.")

    print("\n  SCOPE: the body is a CONSTANT setting, where the matched floor is a scalar and")
    print("  there is no new plant; LP-side, M_i only, one gas. docs/rung60-spec.md.")


def print_stator_bleed_table(flight):
    """Rung-61 payoff: rungs 36/41's standing concession, BOTH halves on one machine.

    The seam every spec since rung 53 has carried says 'the bleed takes over where the
    stator's authority ends'. It does not. The valve can buy back the whole of the stator's
    phi-debit -- exactly -- and most of the stator's BILL survives, because the phi-drop was
    itself a partial rebate on the loading the stator removed."""
    print("\nSTATOR + BLEED (rung 61): a compensating lever buys back the COORDINATE, not the")
    print("BILL. b* is the valve setting that restores phi_op to its bare value exactly.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, flight.p0,
                                      nozzle_convergent=True, **losses)
    m = StatorBleedMatcher(design, flight, 1.0, map_lp=LP, map_hp=HP)

    print("\n  THE HEADLINE (LP spool, fixed throttle). The phi-debit goes; the overspeed stays:")
    print(f"  {'Tt4':>6}{'v':>6}{'b*':>9}{'phi bare':>10}{'phi comp':>10}"
          f"{'dn stator':>11}{'dn comp':>10}{'kept':>7}{'dF comp':>9}")
    for Tt4, v in ((1500.0, 0.10), (1500.0, 0.20), (1500.0, 0.30),
                   (1300.0, 0.20), (1100.0, 0.20), (1100.0, 0.30)):
        c = m.compensated_point(flight, Tt4, v, "lp")
        print(f"  {Tt4:6.0f}{v:6.2f}{c['b_star']:9.5f}{c['phi_bare']:10.5f}"
              f"{c['phi_comp']:10.5f}{c['dn_stator']:+11.2%}{c['dn_comp']:+10.2%}"
              f"{c['dn_comp'] / c['dn_stator']:7.0%}{c['dF_comp']:+9.2%}")
    print("    => at v=0.30 the COMPENSATED point overspeeds the bare stator (kept > 100%):")
    print("       undoing the lever is strictly WORSE than leaving it alone.")

    print("\n  WHY -- the phi-drop was a REBATE. n solves tau_c = 1 + (tau_d-1)*psi(phi)*n^2,")
    print("  and psi = base(phi) - v(1+l)phi. Restoring phi gives base(phi)'s rebate back:")
    Tt4, v = 1500.0, 0.20
    c = m.compensating_bleed(flight, Tt4, v, "lp", "phi")
    print(f"  {'cell':>14}{'b':>9}{'phi':>10}{'n':>10}{'psi':>10}{'tau_c':>10}")
    for tag, vv, bb in (("bare", 0.0, 0.0), ("stator", v, 0.0), ("compensated", v,
                                                                 c["b_star"])):
        sib = m.at_point(vv, 0.0, bb)
        od = sib.match(flight, Tt4)
        print(f"  {tag:>14}{bb:9.5f}{od.phi_lp:10.5f}{od.n_lp:10.5f}"
              f"{sib.map_lp.psi(od.phi_lp):10.5f}{od.tau_lpc:10.5f}")
    print("    => the compensated point is MORE unloaded than the stator-only one.")

    print("\n  THE SEAM AS POSED, REFUTED: opening the valve SHRINKS the stator's authority")
    print("  (it pre-spends the same incidence budget) -- it does not take over after it:")
    print(f"  {'b':>7}{'v_edge':>10}{'M_i(v=0)':>11}{'M_i peak':>11}{'span left':>11}")
    for r in m.authority_with_bleed(flight, 1500.0, (0.0, 0.05, 0.10, 0.15), "lp"):
        print(f"  {r['bleed']:7.2f}{r['v_edge']:10.4f}{r['m_i_0']:11.5f}"
              f"{r['m_i_peak']:11.5f}{r['span']:11.5f}")

    print("\n  AND IT IS SPOOL-DEPENDENT. The two levers do not span the same space: the")
    print("  stator acts on either spool, rung 42's valve on one. On the HP, b* never exists:")
    for r in m.compensability(flight, [1500.0, 1300.0, 1100.0, 900.0], 0.20):
        print(f"    Tt4={r['Tt4']:6.0f}  pi_HPC={r['pi_hpc']:7.4f}   b*_LP={r['b_lp']:.5f}"
              f"   b*_HP = none ({r['why_hp']})")

    print("\n  SCOPE: steady only; b* is a constructed point, not a control law; the compensable")
    print("  range is capped by THIS rung's own b<0.45 and no ceiling is claimed. Magnitudes")
    print("  ride on the two maps and the imposed floor. See docs/rung61-spec.md § Concessions.")


def print_bleed_schedule_table(flight):
    """Rung-62 payoff: rung 61's own seam, on the TRANSIENT plant.

    A state-fed schedule closes a feedback loop on itself through the shaft speed it reads,
    and the loop's SIGN is the sign of the lever's own dn/d(setting). Rung 57 measured its
    stator schedule SURRENDERING authority to that loop; the same instrument gives a bleed
    schedule the other sign. And the two loops, closing through ONE state, do not compose."""
    print("\nBLEED SCHEDULE + STATOR SCHEDULE (rung 62): a state-fed schedule's LOOP has a")
    print("SIGN. FULL/RAMP < 1 is self-cancellation (rung 57); > 1 is self-AMPLIFICATION.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, flight.p0,
                                      nozzle_convergent=True, **losses)
    LO, HI, N_LO, V, B = 1000.0, 1400.0, 0.65, 0.20, 0.10
    STAT = dict(vsv_sched_lp=StatorSchedule(V, N_LO))
    BLED = dict(bleed_sched=BleedSchedule(B, N_LO))

    def mk(**kw):
        return ScheduledBleedTransient(design, flight, 1.0, map_lp=LP, map_hp=HP,
                                       rho=1.0, **kw)

    print("\n  Both loop-gain factors on the steady running line (neither reverses):")
    print(f"    {'Tt4':>6} {'dn_L/db':>10} {'dn_L/dv':>10}")
    for row in mk().loop_factors(flight, (1500.0, 1300.0, 1100.0, 900.0)):
        print(f"    {row['Tt4']:6.0f} {row['dn_db']:+10.5f} {row['dn_dv']:+10.5f}")
    print("    (bleed LOWERS the speed its schedule reads, the stator RAISES it -- and both")
    print("     schedules open at LOW speed, so the two loop gains have opposite signs.)")

    print("\n  The loop, per lever, over a 4x ramp-rate range:")
    print(f"    {'lever':<22} {'r':>5} {'FULL/RAMP':>10} {'cmd RAMP':>9} {'cmd FULL':>9}")
    for name, kw in ((f"stator  v_max={V:.2f}", STAT), (f"bleed   b_max={B:.2f}", BLED)):
        for r in (0.25, 0.50, 1.00):
            d = mk(**kw).loop_decomposition(flight, LO, HI, r=r)
            print(f"    {name:<22} {r:5.2f} {d['self_cancel']:10.4f} "
                  f"{d['cmd_ramp']:9.5f} {d['cmd_full']:9.5f}")
    print("    The stator commands LESS of itself between the two legs; the bleed MORE.")

    print("\n  The two loops share ONE state, and they do not compose. The stator schedule's")
    print("  own surrender (1 - FULL/RAMP), by what is armed BESIDE it:")
    t = mk()
    cmd = mk(**BLED).commanded_level(flight, LO, HI, r=0.25)["at_min"]
    print(f"    {'neighbour':<34} {'surrendered':>12}")
    for name, nb in (("(none)", {}),
                     (f"constant b = {cmd:.4f}  (matched)", dict(bleed=cmd)),
                     (f"constant b = {B:.4f}  (MORE lever)", dict(bleed=B)),
                     (f"SCHEDULE  b_max = {B:.2f}", BLED)):
        d = t.marginal_loop(flight, LO, HI, lever=STAT, neighbour=nb, r=0.25)
        print(f"    {name:<34} {d['surrendered']:+12.4f}")
    print("    The schedule does ~3x the damage of a strictly LARGER constant valve, so it")
    print("    is the LOOP and not the LEVEL. The mirror is inert: a stator schedule leaves")
    print("    the bleed's amplification alone to within 0.7 %. A ONE-WAY arrow.")

    print("\n  And rung 61's steady superposition does not survive the shaft balance's")
    print("  removal -- the same two devices, additive to 2.3 % steady:")
    print(f"    {'r':>5} {'c_stator':>10} {'c_bleed':>10} {'c_pair':>10} {'sum':>10} "
          f"{'interaction':>12}")
    for r in (0.25, 0.50, 1.00):
        d = t.pair_interaction(flight, LO, HI, lever_a=STAT, lever_b=BLED, r=r)
        print(f"    {r:5.2f} {d['credit_a']:+10.5f} {d['credit_b']:+10.5f} "
              f"{d['credit_pair']:+10.5f} {d['credit_sum']:+10.5f} "
              f"{d['interaction_frac']:+12.3f}")
    print("    SCOPE: b(n_L) is an IMPOSED valve position, not a controlled one; n_lo is a")
    print("    placement and the MAGNITUDE rides on it (the sign does not). Every rung-57")
    print("    concession is inherited. See docs/rung62-spec.md.")


def print_fuel_bleed_table(flight):
    """Rung-63 payoff: rung 62's own seam -- a fuel-side min-select leg BESIDE the valve.

    Rung 58 found the influence between a stator and a Wf/pt3 leg runs ONE WAY, and rung 59
    explained it: the leg senses two things and a stator reaches NEITHER (a choked A4 guards
    the ordinate, rung 39's pi_LPC cancellation guards the abscissa). A bleed is the ladder's
    only lever that breaks mdot_face == mdot_core -- the identity UPSTREAM of both -- so it
    reaches both and the arrow closes."""
    print("\nFUEL + BLEED (rung 63): rung 58's ONE-WAY arrow was a fact about the LEG's two")
    print("PROTECTIONS, not about the lever's kind. A bleed breaks both, and the arrow CLOSES.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, flight.p0,
                                      nozzle_convergent=True, **losses)
    LO, HI, N_LO, V, B = 1000.0, 1400.0, 0.65, 0.20, 0.10
    STAT = dict(vsv_sched_lp=StatorSchedule(V, N_LO))
    BLED = dict(bleed_sched=BleedSchedule(B, N_LO))
    t = ScheduledBleedTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0)
    leg = t.accel_schedule(flight, LO, HI, 0.25, 13)

    print("\n  The leg's two SENSED INPUTS, per lever (rung 59's table, differenced):")
    print(f"    {'lever':<24} {'d_ordinate':>12} {'d_abscissa':>12} {'d_MFP(A4)':>12}")
    for name, kw in ((f"LP stator  v_max={V:.2f}", STAT), (f"bleed      b_max={B:.2f}", BLED)):
        d = t.sensed_inputs(flight, LO, HI, kw, margin=0.25, n=9)
        print(f"    {name:<24} {d['d_ordinate']:12.3e} {d['d_abscissa']:12.3e} "
              f"{d['d_mfp']:12.3e}")
    print("    A choked A4 keeps MFP at machine zero for BOTH -- that control never moves.")
    print("    What moves is Tt25: only the LP shaft balance carries (1-b), so Tt3 falls,")
    print("    f rises, and BOTH halves of the schedule table shift. Ten orders apart.")

    print("\n  So the leg can FEEL a bleed and cannot feel a stator (sub-grid s_eng):")
    print(f"    {'lever':<24} {'r':>5} {'s_eng ref':>10} {'s_eng armed':>12} {'shift':>9}")
    for name, kw in ((f"LP stator  v_max={V:.2f}", STAT), (f"bleed      b_max={B:.2f}", BLED)):
        for r in (0.25, 0.50, 1.00):
            d = t.leg_retiming(flight, LO, HI, kw, accel=leg, r=r)
            print(f"    {name:<24} {r:5.2f} {d['ref_dormant']:10.5f} "
                  f"{d['armed_dormant']:12.5f} {100.0 * d['rel_dormant']:+8.3f}%")
    print("    The bleed is positive and the LARGER in every cell; the stator's own shift is")
    print("    trajectory-mediated (its TABLE is bit-identical) and spans -0.03 % to +1.28 %.")
    print("    LATER, not earlier -- a sign the pressure channel alone gets WRONG (spec § 2).")

    print("\n  And a phi FLOOR beside the valve has NO composable middle -- two regimes only,")
    print("  with the boundary at the two plants' OWN minimum phi:")
    d = t.floor_dichotomy(flight, LO, HI, BLED, sm_grid=(0.36, 0.40, 0.43, 0.46))
    print(f"    band in sm = [{d['band'][0]:.4f}, {d['band'][1]:.4f}]   "
          f"min phi: ref {d['min_phi_ref']:.5f}, armed {d['min_phi_armed']:.5f}")
    print(f"    {'sm':>5} {'phi_lim':>8} {'removed(fuel)':>14} {'removed(both)':>14} "
          f"{'credit':>11}")
    for row in d["rows"]:
        print(f"    {row['sm']:5.2f} {row['phi_lim']:8.5f} {row['removed_fuel']:14.4e} "
              f"{row['removed_both']:14.4e} {row['credit']:+11.3e}")
    print("    Inside the band the valve DISARMS the floor exactly (and the armed cell is")
    print("    bit-for-bit its own leg-free march); above it both bind, the floor pins the")
    print("    currency, and the valve's credit is exactly zero -- rung 60's tautology.")
    print("    SCOPE: only the FEEDFORWARD leg composes; the valve is an IMPOSED position;")
    print("    s_eng's magnitude rides on rung 48's disclaimed margin. docs/rung63-spec.md.")


def print_bleed_limiter_table(flight):
    """Rung-64 payoff: rung 63's own seam -- the valve CONTROLLED instead of imposed.

    The question a controlled valve invites is "how much more protection does feedback buy?"
    and the answer is NONE. `b = b_max` is itself an open-loop law and it attains the ceiling,
    so the ceiling belongs to the valve's AUTHORITY -- hardware. What the loop buys is the
    BILL, and that is measurable only at a coordinate matched exactly, which rung 60's
    tautology supplies for free."""
    print("\nphi-REFERENCED BLEED LIMITER (rung 64): a limiter's LAW cannot buy PROTECTION,")
    print("only its PRICE. The ceiling is the lever's AUTHORITY; feedback buys the BILL.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, flight.p0,
                                      nozzle_convergent=True, **losses)
    LO, HI, B, PHI = 1000.0, 1400.0, 0.10, 0.80
    t = LimitedBleedTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0)

    print("\n  THE CEILING -- and the top row is an OPEN-LOOP law that reaches it:")
    c = t.authority_ceiling(flight, LO, HI, b_max=B)
    print(f"    {'law':<26} {'min phi':>9} {'sm':>9} {'b at that min':>14}")
    for key, name in (("shut", "valve SHUT"), ("schedule", f"SCHEDULE  b_max={B:.2f}"),
                      ("full", f"CONSTANT  b = {B:.2f}"),
                      ("over", f"FLOOR  phi_lim={c['phi_lim_over']:.4f}")):
        r = c["cells"][key]
        print(f"    {name:<26} {r['min_phi_lp']:9.6f} "
              f"{r['min_phi_lp'] / FLOOR - 1.0:9.6f} {r['b_at_min_lp']:14.6f}")
    print(f"    The over-set floor SATURATES and is VIOLATED by {c['over_deficit']:+.4f}, and it")
    print(f"    lands BELOW the fully-open march ({c['over_vs_full']:+.3e}) -- so no law beats")
    print("    b = b_max. The schedule's gap is PLACEMENT (it is not saturated at its own min).")

    print("\n  THE BILL -- three laws of ONE lever, matched to the SAME min phi, in rung 61's")
    print("  currency (the overspeed and the thrust, NOT the bleed integral):")
    m = t.matched_bill(flight, LO, HI, phi_target=PHI, b_cap=B)
    print(f"    match error {m['matched']:.1e};  b* = {m['b_star']:.5f}, "
          f"b_max* = {m['bmax_star']:.5f}")
    print(f"    {'law':<24} {'int b ds':>10} {'d nu_lp_end':>12} {'dF_end':>9} "
          f"{'d int F':>9} {'d min phi_hp':>13}")
    for key, name in (("constant", "1 constant   (blind)"),
                      ("schedule", "2 schedule   (state-fed)"),
                      ("floor", "3 phi FLOOR  (feedback)")):
        q = m["bill"][key]
        print(f"    {name:<24} {q['b_int']:10.6f} {q['d_nu_lp_end']:+12.6f} "
              f"{q['thrust_end_pct']:+8.3f}% {q['thrust_int_pct']:+8.3f}% "
              f"{q['d_min_phi_hp']:+13.3e}")
    print(f"    The bill falls with the INFORMATION the law uses: floor/schedule = "
          f"{m['b_ratio_sched']:.4f} in")
    print("    bleed, and the closed loop's END thrust bill is machine-zero -- it")
    print("    self-releases, so it has left the machine by settle. The honest comparator is")
    print("    law 2 (a constant valve bleeds hardest where phi is already highest).")
    print("    The state-fed laws DEBIT the HP; the state-blind one CREDITS it.")
    print("    SCOPE: the valve is INSTANTANEOUS and unlagged; phi_lim rides on rung 36's")
    print("    disclaimed phi_surge and b_max is rung 42's valve size, so the MAGNITUDES are")
    print("    disclaimed and the ORDERING is the claim. See docs/rung64-spec.md.")


def print_lagged_valve_table(flight):
    """Rung-65 payoff: rung 64's own seam -- the valve given a BANDWIDTH, so its position
    stops being a function of the state and becomes a THIRD STATE.

    The panel is ordered by what the rung actually claims. Bandwidth as a second hardware axis
    is the corollary, so it goes first and briefly; the RUNG is the pair of tables under it --
    the fuel leg's plant handed back (a span rung 64 could only derive) and the redundancy
    still there anyway, moved out of the solver and into the state."""
    print("\nTHE LAGGED BLEED VALVE (rung 65): a lag repairs the SOLVE without removing the")
    print("DEGENERACY -- the redundancy of two loops on one variable is CONSERVED.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, flight.p0,
                                      nozzle_convergent=True, **losses)
    LO, HI, B, PHI, DS = 1000.0, 1400.0, 0.10, 0.80, 0.005
    SM = PHI / FLOOR - 1.0
    TAU = 0.05                    # ds/tau = 0.1: the RK4 floor of s 0 is ds/tau <~ 2.78
    t = LaggedBleedTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0)

    print("\n  BANDWIDTH, THE SECOND HARDWARE AXIS -- and it is PURE LOSS (a slower valve")
    print("  protects LESS *and* bleeds MORE), with rung 64's instantaneous valve as the top row:")
    bc = t.bandwidth_ceiling(flight, LO, HI, PHI, B, ds=DS)
    print(f"    {'tau':>8} {'min phi_lp':>12} {'undershoot':>11} {'int b ds':>9} "
          f"{'plateau':>8} {'dev vs r64':>11}")
    print(f"    {'rung 64':>8} {bc['inst_min_phi']:12.9f} {0.0:11.3e} {bc['inst_b_int']:9.6f} "
          f"{bc['inst_plateau_pts']:8d} {'--':>11}")
    for row in bc["rows"]:
        print(f"    {row['tau']:8.3f} {row['min_phi_lp']:12.9f} {row['undershoot']:11.3e} "
              f"{row['b_int']:9.6f} {row['plateau_pts']:8d} {row['dev']:11.3e}")
    rows = bc["rows"]                              # the sweep is DESCENDING in tau, so both
    worse = all(rows[i]["undershoot"] < rows[i + 1]["undershoot"]   # currencies must improve
                for i in range(len(rows) - 1))                      # monotonically along it
    more = all(rows[i]["b_int"] > rows[i + 1]["b_int"] for i in range(len(rows) - 1))
    print(f"    Both currencies monotone in tau and the SAME way (protection {worse}, "
          f"bleed {more}):")
    print("    there is no trade to buy. And rung 64 s 4's DESTROYED argmin is RESTORED -- its")
    print(f"    floor pins phi_lp over {bc['inst_plateau_pts']} points (an interval, ~1/ds), "
          "every lagged march over 1.")
    print(f"    The tau -> 0 arm of the reduce is MEASURED, not asserted: dev shrinks "
          f"monotonically ({bc['dev_shrinks']}).")

    print("\n  THE DISCRIMINATOR -- rung 64 s 3 DERIVED that an instantaneous valve re-pins")
    print("  phi_lp at every trial fuel, deleting the fuel leg's plant. Here it is EXHIBITED,")
    print("  one bracket swept on both plants at the state where a fuel leg bites hardest:")
    fa = t.fuel_authority(flight, LO, HI, SM, B, tau=TAU, ds=DS)
    print(f"    probe: s = {fa['at']['s']:.3f}, b = {fa['at']['b']:.6f}, "
          f"phi_lp = {fa['at']['phi_lp']:.6f}, Wf x {fa['fracs']}")
    for key, name in (("inst", "INSTANTANEOUS (rung 64)"), ("lagged", f"LAGGED  tau = {TAU}")):
        q = fa[key]
        print(f"    {name:<24} phi span {q['span']:11.4e}  monotone {str(q['monotone']):>5}"
              f"  sign change {str(q['sign_change']):>5}")
    print("    The instantaneous span is ROUNDOFF (its monotonicity is meaningless), so the")
    print(f"    ratio {fa['ratio']:.1e} has a machine-zero denominator and only its ORDER is a")
    print("    statement. Inside ONE derivative evaluation a lagged valve is a CONSTANT, so the")
    print("    fuel leg sees rung 42's imposed-valve plant and rung 49's own premise -- phi")
    print("    falls monotonically with fuel -- is restored verbatim.")

    print("\n  AND THE CONTINUUM SURVIVES ANYWAY -- THE RUNG. Wherever both loops ride, every")
    print("  (b, Wf) on phi_lp = phi_lim satisfies BOTH laws, so b_cmd == b and db/ds == 0:")
    mm = t.marginal_mode(flight, LO, HI, SM, B, tau=TAU, taus=(0.2, 0.01), ds=DS)
    print(f"    {'member':<22} {'b0':>9} {'drift':>10} {'|b_cmd-b|/tau':>14} "
          f"{'fuel removed':>13} {'laws held':>10}")
    for key, cell, name in ((None, mm["natural"], "natural (the EDGE)"),
                            ("lo", mm["moved"]["lo"], "b0 - 0.01 (inside)"),
                            ("hi", mm["moved"]["hi"], "b0 + 0.01 (outside)")):
        print(f"    {name:<22} {cell['b0']:9.6f} {cell['drift']:10.2e} {cell['dbds']:14.2e} "
              f"{cell['removed']:13.6e} {cell['laws_held']:10.2e}")
    print("    b is a CONSTANT OF THE MOTION -- the drift column IS the claim (b_end = b0 to")
    print("    ~4e-16 over a whole march), so the family is one-parameter in b0 alone,")
    print(f"    and its members withhold DIFFERENT fuel (ratio "
          f"{mm['natural']['removed'] / mm['moved']['lo']['removed']:.3f} against the member")
    print("    0.01 below). The EDGE is derivable -- the valve's law is the SMALLEST position")
    print("    holding the floor, so ABOVE b_cmd(0) it is doing more than its law asks and")
    print("    CLOSES (the third row's drift), which is why the natural march looks unique.")
    print(f"    tau multiplies a machine zero and cannot reach the mode: withheld fuel is")
    print(f"    tau-invariant to {mm['tau_span_rel']:.1e} relative across a 20x range.")
    print("    SCOPE: one anchor (rung 64's grid), phi_lim/b_max imposed or inherited, and the")
    print("    RK4 floor ds/tau <~ 2.78 is asserted in the plant -- a violation looks exactly")
    print("    like the finding 'a fast valve bleeds more'. See docs/rung65-spec.md.")


def print_two_lag_cascade_table(flight):
    """Rung-66 payoff: rung 65's own seam -- rung 52's lagged FUEL leg BESIDE the lagged valve,
    so both loops on `phi_lp` have a clock. Four states, two bandwidths.

    The first table is the rung, and its CONTROL is the column beside the product: a constant
    `R_q * C_g` is evidence of a reciprocal pair only if the individual gains MOVE. They swing
    ~1.4-1.8x over the same march while the product holds."""
    print("\nTHE TWO-LAG CASCADE (rung 66): two loops on ONE variable are ONE loop with the")
    print("RATES ADDED -- R_q * C_g == 1 is an IDENTITY, so det J == 0 and the pair is degenerate.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, flight.p0,
                                      nozzle_convergent=True, **losses)
    LO, HI, B, PHI, DS = 1000.0, 1400.0, 0.10, 0.80, 0.005
    SM, TAU, TAU_ATT = PHI / FLOOR - 1.0, 0.05, 0.05
    t = TwoLagCascadeTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                               bleed_lim=BleedLimiter(phi_lim=PHI, b_max=B, tau=TAU))

    print("\n  THE IDENTITY, MEASURED -- both cross-gains central-differenced at RIDING points")
    print("  (`required > 0` AND the valve strictly inside its stops), at three fuel clocks:")
    ci = t.cascade_identity(flight, LO, HI, SM, B, tau=TAU, ds=DS)
    print(f"    {'tau_att':>8} {'R_q*C_g lo..hi':>22} {'gain swing R':>13} {'gain swing C':>13} "
          f"{'|lam| max':>10} {'1/t_g+1/t_v':>12}")
    for row in ci["rows"]:
        print(f"    {row['tau_att']:8.3f} "
              f"{row['prod_lo']:10.6f}..{row['prod_hi']:<10.6f} {row['gain_span_R']:13.3f} "
              f"{row['gain_span_C']:13.3f} {row['rho_max']:10.2f} "
              f"{row['rate_closed_form']:12.2f}")
    mid = ci["rows"][1]
    print(f"    gains at tau_att = {mid['tau_att']}:  R_q in [{mid['R_q_lo']:.4f}, "
          f"{mid['R_q_hi']:.4f}],  C_g in [{mid['C_g_lo']:.4f}, {mid['C_g_hi']:.4f}]")
    print("    The product holds to a few percent over 100x in the fuel clock while the gains")
    print("    themselves swing -- so it is a RECIPROCAL PAIR and not a constant plant, and both")
    print("    are strictly NEGATIVE (SUBSTITUTING loops). The spectrum is exactly")
    print(f"    {{0, -(1/t_g + 1/t_v)}}: real at every sampled point ({ci['all_real']}), and the")
    print(f"    non-zero root matches the SUM of the rates to {ci['rho_err_max']:.1e} relative.")

    print("\n  WHAT THE PAIR DELIVERS -- both cells LAGGED on purpose (an instantaneous control")
    print("  is a different plant, not a control), scored on the violation AREA int max(0,")
    print("  phi_lim - phi_lp) ds, which no initial condition can clamp:")
    cb = t.cascade_bill(flight, LO, HI, SM, B, tau=TAU, tau_att=TAU_ATT, ds=DS)
    print(f"    {'cell':>7} {'I (phi)':>13} {'credit':>9} {'min phi (s>0)':>14} {'s at min':>9}")
    for k, name in (("bare", "bare"), ("fuel", "fuel"), ("valve", "valve"), ("both", "both")):
        c = cb["cells"][k]
        cr = "--" if k == "bare" else f"{cb['credit'][k] * 100.0:7.2f}%"
        print(f"    {name:>7} {c['I']:13.6e} {cr:>9} {c['min_phi']:14.9f} {c['s_at_min']:9.4f}")
    print(f"    The pair beats both singles ({cb['beats_both']}) and is strongly SUB-ADDITIVE:")
    print(f"    the two standalone credits sum to {cb['sum_alone'] * 100.0:.2f}% and the pair "
          f"delivers {cb['delivered'] * 100.0:.2f}%.")
    print(f"    {'added LAST':>12} {'marginal':>10} {'alone':>9} {'erosion':>9}")
    for k, mg, al, er in (("fuel", cb["marginal_fuel"], cb["credit"]["fuel"], cb["erosion_fuel"]),
                          ("valve", cb["marginal_valve"], cb["credit"]["valve"],
                           cb["erosion_valve"])):
        print(f"    {k:>12} {mg * 100.0:9.3f}% {al * 100.0:8.2f}% {er:8.1f}x")
    print("    A WHOLE SECOND LIMITER on top of the stronger one buys almost nothing: with one")
    print("    effective actuator direction, the second loop buys the RATE, never the AUTHORITY.")
    print("    SCOPE: one anchor and one SET POINT -- both laws are referenced to the SAME")
    print("    phi_lim, which is what makes them redundant; the erosion MAGNITUDE is a")
    print("    measurement on this grid and the ordering is the claim. See docs/rung66-spec.md.")


def print_cascade_a_table(flight):
    """Rung-67 payoff: rung 66's OTHER cascade -- the two lagged loops moved onto DIFFERENT
    variables (rung 47's Tt4 governor beside the phi valve), so `det J != 0`.

    The ringing null is only worth something if the detector is shown to fire, so
    `detector_sensitivity` is printed beside the sweep that returns zero: at |P| ~ 0.02 the
    count is zero because the period is ~45 tau and the amplitude is e^-45 by then, NOT
    because the counter is blind."""
    print("\nCASCADE A (rung 67): two loops on TWO variables. ONE SCALAR P = R_q * C_g < 0 sets")
    print("BOTH faces -- it ends the degeneracy AND opens a ringing window, then damps it.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, flight.p0,
                                      nozzle_convergent=True, **losses)
    LO, HI, B, PHI, TMAX, DS = 1000.0, 1400.0, 0.10, 0.80, 1200.0, 0.005
    TAU, TAU_GOV = 0.05, 0.05
    t = CrossLoopCascadeTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                  bleed_lim=BleedLimiter(phi_lim=PHI, b_max=B, tau=TAU))

    print("\n  THE SCALAR -- the two cross-gains have OPPOSITE SIGNS (bleed makes it HOTTER,")
    print("  clipping fuel needs LESS bleed), so their product is negative and det J != 0:")
    ci = t.cross_identity(flight, LO, HI, TMAX, tau=TAU, n_sample=8, ds=DS)
    print(f"    {'tau_gov':>8} {'t_v/t_g':>8} {'P = R_q*C_g':>12} {'min R_q':>9} "
          f"{'max C_g':>10} {'complex':>9} {'window rho':>18}")
    for row in ci["rows"]:                         # the TIGHT bound on each sign: the smallest
        print(f"    {row['tau_gov']:8.3f} {row['rho_clock']:8.2f} {row['P_mid']:12.6f} "  # R_q
              f"{row['R_q_lo']:9.4f} {row['C_g_hi']:10.4f} "        # and the largest C_g
              f"{row['n_complex']:4d}/{row['n_sample']:<4d} "
              f"{row['rho_lo']:9.4f}..{row['rho_hi']:.4f}")
    print(f"    P is NEGATIVE everywhere ({ci['all_negative']}), and ~50x smaller in magnitude")
    print("    than cascade B's identically-1. The complex branch is a closed form in ONE")
    print("    coordinate -- rho + 1/rho < 2 + 4|P| -- so the edges are RECIPROCALS and the")
    print("    window is log-symmetric about matched clocks; the spectrum lands where it says.")
    print(f"    R_q is not machine-zero ({ci['R_q_min_abs']:.2e}): a zero there would mean the")
    print("    governor is not sensing the live valve, i.e. two independent loops, not a cascade.")

    print("\n  ADMISSIBLE, BUT UNOBSERVABLE. The FREE response (natural minus a b0-offset march,")
    print("  so the common fuel ramp cancels) never rings -- inside the window or outside it:")
    ow = t.oscillation_window(flight, LO, HI, TMAX, tau=TAU, rhos=(0.5, 1.0, 2.0), ds=DS)
    print(f"    P = {ow['P']:.6f};  window {ow['window']['rho_lo']:.4f}.."
          f"{ow['window']['rho_hi']:.4f},  zeta = {ow['window']['zeta']:.4f},  "
          f"T/tau = {ow['window']['T_over_tau']:.1f}")
    print(f"    {'rho':>6} {'tau_gov':>8} {'complex?':>9} {'sign changes q/g':>18} "
          f"{'offset surviving':>17}")
    for row in ow["rows"]:
        print(f"    {row['rho']:6.2f} {row['tau_gov']:8.4f} "
              f"{str(row['complex_predicted']):>9} "
              f"{row['sign_changes_q']:8d}/{row['sign_changes_g']:<9d} {row['survives']:17.2e}")
    print("    THE THRESHOLD IS TWO CROSSINGS, and it is a theorem rather than a tolerance: a")
    print("    sum of two decaying REAL exponentials has at most ONE zero, so a single crossing")
    print("    is admissible on the real branch and carries nothing. None of these reaches two.")
    print("    The SAME scalar that opens the window sets the damping -- zeta = 1/sqrt(1+|P|)")
    print("    and T/tau = 2pi/sqrt|P| contain NO time constant, so no bandwidth can make the")
    print("    mode visible. A null is worth nothing until the detector is shown to FIRE:")
    ds_ = CrossLoopCascadeTransient.detector_sensitivity()
    print(f"    {'|P|':>7} {'zeta':>8} {'T/tau':>8} {'periods in march':>17} "
          f"{'sign changes':>13} {'rings':>7}")
    for row in ds_["rows"]:
        print(f"    {abs(row['P']):7.2f} {row['zeta']:8.4f} {row['T_over_tau']:8.2f} "
              f"{row['periods']:17.2f} {row['sign_changes']:13d} {str(row['rings']):>7}")
    print("    So the plant's silence is a STATEMENT about |P| ~ 0.02 (the period is ~45 tau and")
    print("    the response is e^-45 by then), not about the counter.")

    print("\n  THE LEDGER cascade B could not build -- TWO currencies, so the 2x2 is SIGNED:")
    cb = t.cross_bill(flight, LO, HI, TMAX, tau=TAU, tau_gov=TAU_GOV, ds=DS)
    print(f"    {'cell':>7} {'I_T (Tt4)':>13} {'credit_T':>10} {'I_phi':>13} {'credit_phi':>11} "
          f"{'max Tt4':>9}")
    for k, name in (("bare", "bare"), ("gov", "governor"), ("valve", "valve"), ("both", "both")):
        c = cb["cells"][k]
        cT = "--" if k == "bare" else f"{cb['credit_T'][k] * 100.0:8.2f}%"
        cF = "--" if k == "bare" else f"{cb['credit_phi'][k] * 100.0:9.2f}%"
        print(f"    {name:>7} {c['I_T']:13.6e} {cT:>10} {c['I_phi']:13.6e} {cF:>11} "
              f"{c['max_Tt4']:9.2f}")
    print(f"    OFF-DIAGONAL, and it has no cascade-B analogue: the valve DEBITS the temperature")
    print(f"    ({cb['valve_on_T'] * 100.0:+.2f}%) while the governor CREDITS the surge margin "
          f"({cb['gov_on_phi'] * 100.0:+.2f}%) --")
    print("    one loop helps the other, the other hurts it, and both signs are derivable from")
    print("    R_q > 0 and C_g < 0 before any march is run.")
    print(f"    DIAGONAL: each loop keeps ~all of its own credit -- erosion "
          f"{cb['erosion_gov']:.2f}x (governor)")
    print(f"    and {cb['erosion_valve']:.2f}x (valve), against rung 66's 38x on the SHARED")
    print("    variable. Same instrument, same phi_lim, opposite verdict: two loops on TWO")
    print("    variables buy AUTHORITY.")
    print("    SCOPE: one anchor; Tt4_max/phi_lim/b_max are imposed or inherited, the clocks are")
    print("    swept march coordinates, and P is measured on THIS plant -- the MAGNITUDES are")
    print("    disclaimed and the signs and the ordering are the claim. See docs/rung67-spec.md.")


def print_three_loop_table(flight):
    """Rung-68 payoff: rung 66's own seam -- a THIRD loop on `phi_lp`.

    The panel is built around the thing that nearly went unnoticed. Rung 66's identity is ONE
    scalar; stating it three times over three pairs is the same measurement three times, and
    `tr` is the hardcoded diagonal. Imposing all three pairwise identities STILL leaves the
    block a free parameter -- the CYCLIC product -- so a block can be pairwise-degenerate and
    rank 2. That is why the first table shows the pairs and the cycle side by side."""
    print("\nTHREE LOOPS ON ONE VARIABLE (rung 68): `n` loops on one variable are ONE loop")
    print("with ALL `n` RATES ADDED -- J = -D c r^T is RANK ONE at every n.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, flight.p0,
                                      nozzle_convergent=True, **losses)
    LO, HI, B, PHI, VM = 1000.0, 1400.0, 0.10, 0.80, 0.20
    SM = PHI / FLOOR - 1.0
    t = ThreeLoopCascadeTransient(
        design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
        bleed_lim=BleedLimiter(phi_lim=PHI, b_max=B, tau=0.05),
        stator_lim=StatorLimiter(phi_lim=PHI, v_max=VM, tau=0.05))

    print("\n  WHAT IS ACTUALLY INDEPENDENT -- the three PAIRWISE products are rung 66's one")
    print("  identity three times; only the CYCLIC product tests JOINT collapse:")
    g = t.triple_gains(flight, LO, HI, sm=SM, every=35)
    print(f"    {'s':>7} {'R_q*C_g':>11} {'R_v*V_g':>11} {'C_v*V_q':>11} "
          f"{'CYCLIC R_q*C_v*V_g':>20}")
    for row in g["rows"]:
        o = row["on"]
        print(f"    {row['s']:7.4f} {o['pair_RC']:11.8f} {o['pair_RV']:11.8f} "
              f"{o['pair_CV']:11.8f} {o['cyclic']:20.10f}")
    print(f"    {g['n_riding']} points with all three loops riding INTERIOR. Predicted -1 "
          "(three factors")
    print("    of -phi_j/phi_i). det = (x+1)^2/x, so det carries nothing the cycle does not.")
    s = t.cyclic_sensitivity(flight, LO, HI, sm=SM)
    print(f"    The DETECTOR, measured not asserted: floor {s['floor']:.2e}, gain "
          f"{s['gain']:.2f} per unit")
    print(f"    off-manifold displacement -- it resolves delta >~ {s['resolves']:.0e}.")

    print("\n  THE LEDGER -- every subset of the three loops, and the WALL each credit is")
    print("  measured against (the stator MOVES the phi wall and not the metal one):")
    b = t.triple_bill(flight, LO, HI, sm=SM)
    print(f"    {'cell':>5} {'I (phi)':>13} {'credit':>9} {'I (incidence)':>15} {'credit':>9}")
    for k in ("bare", "F", "V", "S", "FV", "FS", "VS", "FVS"):
        c = b["cells"][k]
        print(f"    {k:>5} {c['I']:13.6e} {c['credit']:8.2f}% {c['I_inc']:15.6e} "
              f"{c['credit_inc']:8.2f}%")
    print(f"    Three standalone credits sum to {b['sum_singles']:.1f}%; the TRIPLE delivers "
          f"{b['delivered']:.1f}%.")
    print(f"    {'added LAST':>12} {'marginal (phi)':>16} {'alone':>9} {'erosion':>9} "
          f"{'marginal (incid)':>18}")
    for k in ("fuel", "valve", "stator"):
        print(f"    {k:>12} {b['marginal'][k]:15.3f}% {b['singles'][k]:8.2f}% "
              f"{b['erosion'][k]:8.1f}x {b['marginal_incidence'][k]:17.3f}%")
    print("    The stator PROTECTS phi and ERODES incidence -- rung 53's 'a margin is a")
    print("    DISTANCE' reaching the SIGN of a credit. And rung 66 s 9's guess that the")
    print("    third limiter would buy LEAST is wrong: the FUEL leg, added last, buys least.")
    print("    SCOPE: the phi-referenced stator loop moves the lever the ANTI-PHYSICAL way (a")
    print("    real VSV closes to protect; this one OPENS), all three tau are swept march")
    print("    coordinates, and phi_lim/b_max/v_max are imposed or inherited -- so the")
    print("    MAGNITUDES are disclaimed and the ORDERING is the claim. See docs/rung68-spec.md.")



def print_reference_split_table(flight):
    """Rung-69 payoff: rung 68's own named strongest seam -- the SAME stator, referenced to
    INCIDENCE instead of to `phi`.

    The panel is built around the correction, because that is the part a reader inheriting rung
    68 would get wrong: `det J` is ZERO under BOTH references, so the determinant test rung 68
    leaned on is BLIND to the split. What moves is the second invariant, and what it says is
    that the rank counts CONSTRAINTS, not loops."""
    print("\nTHE REFERENCE SPLIT (rung 69): a loop's COORDINATE, not its actuator, decides")
    print("whether it adds a ZERO or a RANK -- ZEROS = n - m, with m the CONSTRAINT count.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, flight.p0,
                                      nozzle_convergent=True, **losses)
    LO, HI, B, PHI, VM = 1000.0, 1400.0, 0.10, 0.80, 0.20
    SM = PHI / FLOOR - 1.0
    t = ReferenceSplitTransient(
        design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
        bleed_lim=BleedLimiter(phi_lim=PHI, b_max=B, tau=0.05),
        stator_inc=StatorIncidenceLimiter.from_margin(LP, VM, SM, tau=0.05))
    T_c = LP.tan_beta1_crit()
    print(f"  ONE PHYSICAL WALL, TWO COORDINATES: phi_lim = {PHI} is m_lim = T_c - 1/phi_lim = "
          f"{T_c - 1.0 / PHI:.6f}")
    print("  at the DESIGN stator setting. dphi/dv = -0.423 but dM_i/dv = +0.335, so THIS loop")
    print("  CLOSES the stators -- the physical direction, and the opposite of rung 68's.")

    print("\n  WHICH PAIRS KEEP RUNG 66's IDENTITY READS OFF WHICH LOOPS SHARE A CONSTRAINT.")
    print("  Both references, the SAME base points on ONE trajectory:")
    g = t.reference_gains(flight, LO, HI, sm=SM, every=20)
    print(f"    {'s':>7} | {'R_q*C_g':>11} {'R_v*V_g':>10} {'C_v*V_q':>10} {'CYCLIC':>10}"
          f" | {'R_q*C_g':>11} {'R_v*V_g':>9} {'CYCLIC':>9}")
    print(f"    {'':>7} | {'-- incidence-referenced (rung 69) --':^45}"
          f" | {'-- phi (rung 68) --':^33}")
    for row in g["rows"]:
        i, p = row["inc"], row["phi"]
        print(f"    {row['s']:7.4f} | {i['pair_RC']:11.8f} {i['pair_RV']:10.5f} "
              f"{i['pair_CV']:10.5f} {i['cyclic']:10.5f} | {p['pair_RC']:11.8f} "
              f"{p['pair_RV']:9.5f} {p['cyclic']:9.5f}")
    print(f"    The SHARED pair holds at 1 to {g['worst_RC_inc']:.1e} under BOTH. The split")
    print(f"    pairs BOTH go to k in [{g['k_range'][0]:.4f}, {g['k_range'][1]:.4f}] -- equal "
          f"to {100*g['worst_pair_gap']:.2f}%, which")
    print("    measures that the two walls differ by exactly the LEVER'S OWN channel.")

    print("\n  THE SPECTRUM -- and the invariant rung 68 used CANNOT SEE THIS:")
    r = t.reference_modes(flight, LO, HI, sm=SM, clocks=((0.05, 0.05, 0.05),
                                                         (0.05, 0.005, 0.05)), every=60)
    print(f"    {'taus (g,q,s)':>20} {'ref':>5} {'zeros':>7} {'|c0|/S^3':>11} "
          f"{'|c1|/S^2':>11} {'pair':>8} {'zeta':>8}")
    for arm in r["arms"]:
        for ref in ("inc", "phi"):
            a = arm["refs"][ref]
            zt = a["zeta_range"][0]
            print(f"    {str(arm['taus']):>20} {ref:>5} {str(a['zeros']):>7} "
                  f"{a['max_c0_rel']:11.2e} {a['min_c1_rel']:11.2e} "
                  f"{'complex' if a['all_complex'] else 'real':>8} {zt:8.4f}")
    print("    det J = 0 under BOTH -- the two loops still on `phi` keep exactly PARALLEL rows,")
    print("    whatever the third watches. `c1` moves by more than TEN orders. And the freed root does")
    print("    NOT land on the real axis: zeta >= 1/sqrt(1-k) for EVERY bandwidth, so ONE")
    print("    SCALAR sets the split, the cyclic product AND the ring (rung 67's P, new")
    print("    mechanism). The window has edges -- the fast-fuel arm above is back on the axis,")
    print("    and the RANK does not care.")

    print("\n  THE LEDGER -- and its WHOLE SIGN TABLE flips with the reference:")
    b = t.reference_bill(flight, LO, HI, sm=SM)
    print(f"    {'stator credit':>28} {'phi-referenced':>16} {'incidence-referenced':>22}")
    for lbl, key in ((" alone, in phi", "alone"), (" alone, in incidence", "alone_inc"),
                     (" marginal, in phi", "marginal"),
                     (" marginal, in incidence", "marginal_inc")):
        print(f"    {lbl:>28} {b['stator_credit']['phi'][key]:15.2f}% "
              f"{b['stator_credit']['inc'][key]:21.2f}%")
    si = b["inc"]["cells"]
    print(f"    The sharpest number: the incidence loop ALONE takes min phi_lp to "
          f"{si['S']['min_phi']:.6f},")
    print(f"    BELOW the bare march's own {si['bare']['min_phi']:.6f} -- measurably worse "
          "than no limiter at all,")
    print("    in the currency it does not watch. Rung 53's 'a margin is a DISTANCE' one level")
    print("    up again: rung 68 showed a credit needs its WALL named, this one shows it needs")
    print("    its loop's REFERENCE named too.")
    print("    SCOPE: v_max = 0.20 is INHERITED from rungs 57/58 and the loop SATURATES on it")
    print("    over 84% of the ramp when it runs alone, so the S/FS cells are authority-limited")
    print("    by a ceiling chosen elsewhere; all three tau are swept march coordinates; and")
    print("    the two floors are matched at the DESIGN setting, so the references are compared")
    print("    at equal WALL and not at equal excursion. See docs/rung69-spec.md.")


def print_cross_split_table(flight):
    """Rung-70 payoff: rung 68 s 10's *three loops on TWO variables* and rung 69 s 11's
    *`pair_RV != pair_CV`* -- ONE seam from two sides, closed together.

    The panel is built around what a reader of rungs 68/69 would carry in wrongly. Rung 68 says
    *quote the cyclic product*; rung 69 says *`pair_RC = 1` is the negative control*. BOTH stop
    being true here, and neither failure raises anything -- so the panel shows the identity
    MOVING and the cyclic product going HALF-BLIND, on one trajectory."""
    print("\nTHE GENERIC SPLIT (rung 70): the split buys the RANK; the RING needs the odd")
    print("constraint to be a SECOND WALL ON THE SAME LEVER.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, flight.p0,
                                      nozzle_convergent=True, **losses)
    LO, HI, B, PHI, VM, TMAX = 1000.0, 1400.0, 0.10, 0.80, 0.20, 1200.0
    SM = PHI / FLOOR - 1.0
    t = CrossSplitTransient(
        design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
        bleed_lim=BleedLimiter.from_margin(LP, B, SM, tau=0.05),
        stator_lim=StatorLimiter.from_margin(LP, VM, SM, tau=0.05))
    print("  RUNG 67's SUBSTITUTION, APPLIED TO RUNG 68's TRIPLE: the odd loop's SENSOR moves")
    print(f"  from `phi` to `Tt4` (rung 47's governor, Tt4_max = {TMAX:.0f} K -- rung 67's own")
    print("  imposed value). Same valve, same stator, same wall. n = 3, m = 2 -- RUNG 69's CELL.")

    w = t.window_overlap(flight, LO, HI, TMAX, SM)
    print(f"\n  All three windows overlap on s in [{w['joint'][0]:.3f}, {w['joint'][1]:.3f}] "
          f"({w['joint'][2]} points, {100*w['joint_fraction']:.1f}% of the march) -- a GATE,")
    print("  not a remark: a gain table over an empty intersection would report the algebra of")
    print("  loops that were never simultaneously live.")

    print("\n  THE IDENTITY MOVES -- and BOTH of rungs 68/69's readings stop being complete:")
    g = t.split_gains(flight, LO, HI, TMAX, SM, every=20)
    print(f"    {'s':>7} | {'R_q*C_g':>10} {'R_v*V_g':>10} {'C_v*V_q':>13} {'CYCLIC x':>10}"
          f" | {'R_q*C_g':>10}")
    print(f"    {'':>7} | {'-- rung 70: the GOVERNOR is the odd loop --':^47}"
          f" | {'rung 68':^10}")
    for row in g["rows"]:
        a, f = row["gov"], row["fuel"]
        # the rung-68 contrast is only readable where ITS leg is interior too; an off-regime
        # arm is a KINK, not a gain, and is shown as such rather than differenced anyway
        fuel = f"{f['pair_RC']:10.6f}" if f["interior"] else f"{'off-regime':>10}"
        print(f"    {row['s']:7.4f} | {a['pair_RC']:10.6f} {a['pair_RV']:10.6f} "
              f"{a['pair_CV']:13.10f} {a['cyclic']:10.6f} | {fuel}")
    print(f"    The SHARED pair is now (C,V) and holds at 1 to {g['worst_CV']:.1e}. At the SAME")
    print(f"    base points rung 68's fuel leg still gives pair_RC = 1 to "
          f"{g['worst_RC_fuel']:.1e} -- so the")
    print("    identity MOVED, measured on ONE trajectory. The two SPLIT pairs come back with")
    print("    OPPOSITE SIGNS, which no single scalar can summarise -- and the CYCLIC product")
    print(f"    equals -pair_RC to {g['worst_cyclic_is_RC']:.1e} while being BLIND to pair_RV.")

    print("\n  THE SPECTRUM, and the FLOOR the split leaves behind:")
    f = t.split_floor(flight, LO, HI, TMAX, SM,
                      grid=((0.05, 0.05, 0.05), (0.05, 0.05, 0.10), (0.05, 0.05, 2.00)))
    print(f"    {'taus (g,q,s)':>20} {'quiet share':>12} {'zeta':>9} {'floor':>9} {'pair':>9}")
    for row in f["rows"]:
        if "zeta" not in row:
            continue
        print(f"    {str(row['taus']):>20} {row['quiet_share']:12.4f} {row['zeta']:9.5f} "
              f"{row['floor']:9.5f} {'COMPLEX' if row['complex_pair'] else 'real':>9}")
    print("    RUNG 69 FOUND A VISIBLE RING (zeta >= 0.61) because its `k ~ -1.7` was ONE LEVER")
    print("    READING TWO WALLS. Here the odd constraint sits on a DIFFERENT lever, both split")
    print("    pairs are cross-LEVER gains, and the floor lands at ~0.990 -- which is rung 67's")
    print("    own zeta = 1/sqrt(1+|P|), because min() selects pair_RC and pair_RC IS rung 67's")
    print("    P. A third loop that SHARES a constraint adds a zero and moves the achievable")
    print("    damping NOWHERE. The pre-registered 'no complex pair at ANY bandwidth' is")
    print("    REFUTED: the last row rings -- but only where the stator carries 1% of the rate")
    print("    sum, i.e. where the third loop is dynamically inert, and at zeta = 0.992 it is")
    print("    rung 67's admissible-but-unobservable mode again.")
    print("    SCOPE: that floor identity is CONTINGENT on pair_RV > 0 (had the stator's pair")
    print("    been the more negative one, min() would select a gain rung 67 never measured);")
    print("    Tt4_max/phi_lim/b_max/v_max are imposed or inherited; all three tau are swept")
    print("    march coordinates. ORDERINGS and SIGNS are the claims, MAGNITUDES are")
    print("    disclaimed. See docs/rung70-spec.md.")


def print_full_split_table(flight):
    """Rung-71 payoff: `n = m = 3`, ZERO zeros -- the last unoccupied cell of rung 69 s 1's
    table, and rung 70's own named strongest seam.

    The panel is built around the thing a reader of rungs 68-70 would get wrong: they would
    read `zeros = n - m = 0` as *three live loops*, and it is not. The third constraint is
    IMPLIED by the second's on the whole admissible band, so the loop lives inside the valve's
    LAG -- an independent GRADIENT with a redundant FEASIBLE SET. Everything else here follows
    from that one sentence, including the ledger."""
    print("\nTHE FULL SPLIT (rung 71): a constraint can be INDEPENDENT IN RANK and REDUNDANT")
    print("ON THE BAND -- so `zeros = n - m` counts GRADIENTS, not LIVE loops.")

    losses = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
                  eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
    FLOOR = 0.55
    LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

    def cpg():
        g, cp = 1.3, 1239.0
        return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
                   R_t=(g - 1.0) / g * cp, hPR=42.8e6)

    design = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, flight.p0,
                                      nozzle_convergent=True, **losses)
    LO, HI, B, PHI, VM, TMAX = 1000.0, 1400.0, 0.10, 0.80, 0.20, 1200.0
    SM = PHI / FLOOR - 1.0
    t = FullSplitTransient(
        design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
        bleed_lim=BleedLimiter.from_margin(LP, B, SM, tau=0.05),
        stator_inc=StatorIncidenceLimiter.from_margin(LP, VM, SM, tau=0.05))
    print("  RUNG 69's MOVE, APPLIED TO RUNG 70's PLANT: the stator's REFERENCE goes from `phi`")
    print("  to INCIDENCE, beside rung 47's governor and rung 65's valve. Three loops, THREE")
    print("  constraints -- n = m = 3, and the actuator block is INVERTIBLE for the first time.")

    print("\n  THE CONTAINMENT, and it needs no new constant:")
    print("    phi = phi_lim  =>  M_i = T_c - 1/phi_lim + v = m_lim + v >= m_lim  for all v >= 0")
    bc = t.band_containment(flight, LO, HI, TMAX, SM)
    print("    so {phi >= phi_lim} INTERSECT the band [0, v_max] sits INSIDE {M_i >= m_lim}.")
    print(f"    Measured on the march: of {bc['n_delivering']} points where the valve DELIVERS,")
    print(f"    the stator rides at {bc['riding_while_delivering']} of them, and the slack "
          f"minus v bottoms out at")
    print(f"    exactly {bc['worst_slack_minus_v']:.1f} (it IS 1/phi_lim - 1/phi, zero where "
          "the valve pins the floor).")

    print("\n  SO THE THIRD LOOP'S WINDOW IS THE SECOND LOOP'S LAG -- swept from both sides:")
    wl = t.window_law(flight, LO, HI, TMAX, SM)
    print(f"    {'tau_q':>8} {'rides to s =':>14}   |   {'tau_s':>8} {'rides to s =':>14}")
    for i in range(max(len(wl["tau_qs"]), len(wl["tau_ss"]))):
        left = (f"{wl['tau_qs'][i]:8.3f} {wl['edge_q'][i]:14.3f}"
                if i < len(wl["tau_qs"]) else f"{'':>23}")
        right = (f"{wl['tau_ss'][i]:8.3f} {wl['edge_s'][i]:14.3f}"
                 if i < len(wl["tau_ss"]) else "")
        print(f"    {left}   |   {right}")
    print(f"    Monotone in the VALVE's clock over {wl['q_span']:.2f}x; a "
          f"{wl['s_span']:.2f}x NON-monotone band in its own.")
    print(f"    The stator rides over {100*wl['base']['stator'][2]/wl['base']['n']:.1f}% of the "
          f"march (rung 70's stator: 24.3%) -- THIS rung's number.")
    print(f"    The JOINT window is thinner still, {100*wl['joint_fraction']:.1f}%, but that is "
          "the 7.9% intersected with a")
    print("    governor that does not engage until s = 0.105 -- rung 67's imposed Tt4_max, NOT")
    print("    containment. Containment owns where the stator's window ENDS; Tt4_max owns where")
    print("    the joint one STARTS.")

    print("\n  THE DETERMINANT IS ALIVE FOR THE FIRST TIME -- AND STILL BLIND TO ONE GAIN:")
    g = t.full_gains(flight, LO, HI, TMAX, SM, every=3)
    print(f"    {'s':>7} | {'R_q*C_g':>10} {'R_v*V_g':>10} {'C_v*V_q':>10} | "
          f"{'det M':>10} {'-(1-RC)(1-CV)':>14}")
    for row in g["rows"]:
        a = row["gains"]
        print(f"    {row['s']:7.4f} | {a['pair_RC']:10.6f} {a['pair_RV']:10.6f} "
              f"{a['pair_CV']:10.6f} | {row['det']:10.6f} {row['det_pred']:14.6f}")
    print(f"    NO pair is 1 (closest: {g['closest_to_1']:.3f} away) -- rung 66's identity is a")
    print("    property of a SHARED constraint and nothing is shared. `pair_RC` IS rung 67's P")
    print("    and `pair_CV` IS rung 69's k, so only the middle column is new -- and the")
    print(f"    determinant FACTORS into the other two to {g['worst_det_err']:.1e}, because "
          "pair_RV cancels")
    print(f"    against the reverse cyclic product (to {g['worst_y_is_RV']:.1e}). ONE FACTOR "
          "PER RUNG.")

    print("\n  AND THE s = 0 FIXED POINT BECOMES A POINT:")
    ic = t.ic_contraction(flight, LO, HI, TMAX, SM)
    for name, lbl in (("full", "rung 71 (n = m = 3)"), ("shared", "rung 70 (n = 3, m = 2)")):
        d = ic[name]
        print(f"    {lbl:>24}: {d['n']} starts x orders -> {d['members']} limit point(s), "
              f"spread {max(d['spread'].values()):.3e}")
    print("    Rung 69 s 6 called a null space a SHOCK ABSORBER. At nullity ZERO there is")
    print("    nothing to absorb with, and the sweep REJECTS a moved start instead.")

    print("\n  THE LEDGER, IN THREE CURRENCIES -- one per loop:")
    b = t.full_bill(flight, LO, HI, TMAX, SM)
    print(f"    {'cell':>5} {'I (phi)':>12} {'E (Tt4)':>10} {'M (incid.)':>12} "
          f"{'min phi':>9} {'max Tt4':>9}")
    for k, c in b["cells"].items():
        print(f"    {k:>5} {c['I']:12.4e} {c['E']:10.3f} {c['M']:12.4e} "
              f"{c['min_phi']:9.6f} {c['max_Tt4']:9.2f}")
    print("    Each loop's MARGINAL share of its OWN currency, against its SOLO one:")
    for leg in ("gov", "valve", "stator"):
        print(f"      {leg:>7}: kept {100*b['kept'][leg]:7.1f} %")
    print("    RUNG 70 s 5 SAID *a loop is eroded by the loops it SHARES a constraint with, and")
    print("    by no others*. NO TWO LOOPS SHARE HERE and the stator keeps a few per cent --")
    print("    so erosion has a SECOND channel: a loop is eroded by any loop that pushes its")
    print("    constraint into the SLACK region. That is the containment again, integrated.")
    print(f"    The sharpest number: the VALVE, which cannot see M_i at all, delivers "
          f"{100*b['inc_credit_valve_alone']:.1f} %")
    print(f"    of the incidence credit running ALONE, against the incidence stator's own "
          f"{100*b['inc_credit_stator_alone']:.1f} %.")
    print("    SCOPE: the joint window is ~2% of the march and every gain table lives inside")
    print("    it; the CONTAINMENT is contingent on the two walls being MATCHED at the design")
    print("    setting (rung 69's choice) -- only the RANK half of the headline is general;")
    print("    Tt4_max is rung 67's imposed value, phi_lim/b_max/v_max rungs 64/57/58's;")
    print("    the determinant's FACTORING is contingent on grad(M_i) = sigma grad(phi) + e_v;")
    print("    all three tau are swept march coordinates. See docs/rung71-spec.md.")


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

    print_super_eq_prompt_table(FLIGHT)

    print_super_eq_quench_table(FLIGHT)

    print_ideal_bell_lift_table(FLIGHT)

    print_spatial_pdf_table(FLIGHT)

    print_dwell_spectrum_table(FLIGHT)

    print_local_mixing_table(FLIGHT)

    print_finite_rate_nozzle_table(FLIGHT)

    print_freeze_out_nozzle_table(FLIGHT)

    print_no_freeze_out_table(FLIGHT)

    print_coupled_no_march_table(FLIGHT)

    print_shifting_turbine_table(FLIGHT)

    print_choked_nozzle_table(FLIGHT)

    print_offdesign_table(FLIGHT)

    print_component_map_table(FLIGHT)

    print_subsonic_matching_table(FLIGHT)

    print_spool_transient_table(FLIGHT)

    print_fuel_metering_table(FLIGHT)

    print_surge_line_table(FLIGHT)

    print_combustor_dynamics_table(FLIGHT)

    print_two_spool_matching_table(FLIGHT)

    print_two_spool_map_table(FLIGHT)

    print_two_shaft_transient_table(FLIGHT)

    print_two_spool_surge_table(FLIGHT)

    print_interstage_bleed_table(FLIGHT)

    print_two_shaft_fuel_table(FLIGHT)

    print_transient_surge_table(FLIGHT)

    print_transient_fuel_surge_table(FLIGHT)

    print_topping_governor_table(FLIGHT)

    print_lagged_governor_table(FLIGHT)

    print_accel_schedule_table(FLIGHT)

    print_phi_limiter_table(FLIGHT)

    print_release_edge_table(FLIGHT)

    print_release_rate_table(FLIGHT)

    print_asymmetric_lag_table(FLIGHT)

    print_variable_stator_table(FLIGHT)

    print_throat_capacity_table(FLIGHT)

    print_stage_stack_table(FLIGHT)

    print_per_row_capacity_table(FLIGHT)

    print_stator_schedule_table(FLIGHT)

    print_composite_minselect_table(FLIGHT)

    print_matched_floor_table(FLIGHT)

    print_stator_bleed_table(FLIGHT)

    print_bleed_schedule_table(FLIGHT)
    print_fuel_bleed_table(FLIGHT)

    print_bleed_limiter_table(FLIGHT)

    print_lagged_valve_table(FLIGHT)
    print_two_lag_cascade_table(FLIGHT)
    print_cascade_a_table(FLIGHT)

    print_three_loop_table(FLIGHT)
    print_reference_split_table(FLIGHT)

    print_cross_split_table(FLIGHT)
    print_full_split_table(FLIGHT)

    plot_ts_diagram(ideal, real, FLIGHT)


if __name__ == "__main__":
    main()
