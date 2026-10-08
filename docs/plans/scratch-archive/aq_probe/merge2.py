p = r"W:/Claude_projects/jet engine/rust/src/panels/cascades.rs"
s = open(p, encoding='utf-8', newline='').read()
s += open(r"W:/temp/claude/aq_probe/part2.rs", encoding='utf-8', newline='').read()
reps = [
("use crate::cross_loop::{build_cross_loop_cascade, detector_sensitivity, OscRow};\n",
 "use crate::cross_loop::{build_cross_loop_cascade, detector_sensitivity, OscRow};\n"
 "use crate::cross_split::{build_cross_split_cascade, split_floor, split_gains, window_overlap};\n"),
("use crate::engine::FlightCondition;\n",
 "use crate::engine::FlightCondition;\n"
 "use crate::full_split::{\n    band_containment, build_full_split_cascade, full_bill, full_gains, ic_contraction, window_law,\n};\n"),
("use crate::pyfmt::{py_tuple, Printer, PyFormat};\n",
 "use crate::pyfmt::{py_list, py_tuple, Printer, PyFormat};\n"
 "use crate::reference_split::{\n    build_reference_split_cascade, reference_bill, reference_gains, reference_modes,\n    StatorIncidenceLimiter,\n};\n"),
("use crate::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient};\n",
 "use crate::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient};\n"
 "use crate::three_loop::{\n    build_three_loop_cascade, cyclic_sensitivity, triple_bill, triple_gains, StatorLimiter,\n    TripleRigArm,\n};\n"),
]
for a, b in reps:
    assert s.count(a) == 1, a
    s = s.replace(a, b)
open(p, 'w', encoding='utf-8', newline='').write(s)

p = r"W:/Claude_projects/jet engine/rust/src/panels/mod.rs"
s = open(p, encoding='utf-8', newline='').read()
a = '    ("print_cascade_a_table", cascades::cascade_a_table),\n'
assert s.count(a) == 1
s = s.replace(a, a + '    ("print_three_loop_table", cascades::three_loop_table),\n'
              '    ("print_reference_split_table", cascades::reference_split_table),\n'
              '    ("print_cross_split_table", cascades::cross_split_table),\n'
              '    ("print_full_split_table", cascades::full_split_table),\n')
open(p, 'w', encoding='utf-8', newline='').write(s)
print("ok")
