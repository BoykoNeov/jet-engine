import io

p = r"M:\claud_projects\jet engine\rust\src\sensed_cap.rs"
s = io.open(p, encoding="utf-8").read()
old = """use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::demand_coordinate::LAG_COORD_CLIP;
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelLimiters, FuelPoint, FuelTransientCore,
    FuelTransientHooks,
};
use crate::gas::Abort;
use crate::map::ComponentMap;
use crate::shared_actuator::SharedRigArm;
"""
new = """use crate::anti_windup::{rhs_gains_at, WindupScope, WINDUP_LAW_NONE};
use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::demand_coordinate::{applied_demand, LAG_COORD_CLIP, LAG_COORD_DEMAND};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, AsymmetricLag, Authority, Floor, FuelLimiters, FuelPoint, FuelTransientCore,
    FuelTransientHooks, PointExtra,
};
use crate::gas::Abort;
use crate::map::ComponentMap;
use crate::shared_actuator::{charpoly4, quartic_roots_c, riding4, SharedRigArm};
"""
assert s.count(old) == 1, s.count(old)
s = s.replace(old, new)

anchor = "pub const CAP_LAWS_DECLARED: [&str; 2] = [CAP_LAW_SOLVE, CAP_LAW_SENSED];\n"
assert s.count(anchor) == 1
s = s.replace(anchor, anchor + """
/// Rung 73's `"sched"` reference, under this module's own name.
///
/// [`anti_windup`](crate::anti_windup)'s `REF_APPLIED` one rung on, and for the mirror reason:
/// `cap_gains`'s `row_err` target BRANCHES on which reference the cell was read under, so the
/// string is compared rather than merely threaded, and a bare literal at that comparison is the
/// one place in this file a typo could not be caught by a type.
const REF_SCHED: &str = crate::applied_reference::REF_LAWS_DECLARED[0];
""")

add = io.open(r"W:\temp\claude\slice-ag-step5\append.rs", encoding="utf-8").read()
s = s.rstrip("\n") + "\n" + add
io.open(p, "w", encoding="utf-8", newline="\n").write(s)
print("lines:", s.count("\n") + 1)
