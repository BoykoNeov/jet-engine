"""Which assertion makes main.py's rung-33 panel print SUB-IDLE at Tt4 = 440 / 420 K?"""
import sys
sys.path.insert(0, r"W:\Claude_projects\jet engine")
import main
from turbojet.engine import build_turbojet, OffDesignMatcher
from turbojet.gas import Gas

flight = main.FLIGHT
design = build_turbojet(Gas.reacting_equilibrium(), main.PI_C, main.TT4, flight.p0,
                        nozzle_convergent=True, **main.REAL_LOSSES)
m = OffDesignMatcher(design, flight, 1.0)
for Tt4 in (480.0, 440.0, 420.0):
    try:
        od = m.match(flight, Tt4)
        print(Tt4, "OK", od.branch, od.performance.specific_thrust)
    except AssertionError as e:
        print(Tt4, "AssertionError:", str(e)[:160])
