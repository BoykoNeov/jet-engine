import os, sys
sys.path.insert(0, r"W:\Claude_projects\jet engine")
print(sys.version)
os.chdir(r"W:\temp\claude\phase8-ar\ref")
import main
from turbojet.engine import build_turbojet
from turbojet.gas import Gas
gas = Gas()
ideal = build_turbojet(gas, main.PI_C, main.TT4, main.FLIGHT.p0).run(main.FLIGHT, mdot=1.0)
real = build_turbojet(gas, main.PI_C, main.TT4, main.FLIGHT.p0, **main.REAL_LOSSES).run(main.FLIGHT, mdot=1.0)
sys.stdout.reconfigure(encoding="utf-8")
main.plot_ts_diagram(ideal, real, main.FLIGHT)
