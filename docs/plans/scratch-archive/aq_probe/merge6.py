p = r"W:/Claude_projects/jet engine/rust/src/panels/readers.rs"
s = open(p, encoding='utf-8', newline='').read()
s += open(r"W:/temp/claude/aq_probe/part6.rs", encoding='utf-8', newline='').read()
reps = [
("use super::Design;\n",
 "use super::Design;\n"
 "use crate::authority_clock::{authority_clock, authority_mask};\n"),
("use crate::bleed_transient::LeverArm;\n",
 "use crate::bleed_transient::LeverArm;\n"
 "use crate::corrector_law::{corrector_read, residual_shape};\n"
 "use crate::engine::FlightCondition;\n"),
("use crate::pyfmt::{py_list, py_tuple, Printer, PyFormat};\n",
 "use crate::pyfmt::{py_dict, py_list, py_tuple, Printer, PyFormat};\n"),
("use crate::stator_transient::ScheduledStatorCore;\n",
 "use crate::split_wall::{build_split_wall_cascade, split_arrest, split_gains as split_wall_gains,\n"
 "                        split_liveness};\n"
 "use crate::staircase_law::{classify, edge_read, root_class, Kind, RootClass};\n"
 "use crate::stator_transient::ScheduledStatorCore;\n"),
("use crate::three_loop::StatorLimiter;\n",
 "use crate::three_loop::StatorLimiter;\n"
 "use crate::threshold_law::{threshold_reference, ScanKw, ThresholdReference};\n"),
]
for a, b in reps:
    assert s.count(a) == 1, a
    s = s.replace(a, b)
open(p, 'w', encoding='utf-8', newline='').write(s)

p = r"W:/Claude_projects/jet engine/rust/src/panels/mod.rs"
s = open(p, encoding='utf-8', newline='').read()
a = '    ("print_state_coordinate_table", readers::state_coordinate_table),\n'
assert s.count(a) == 1
s = s.replace(a, a + '    ("print_split_wall_table", readers::split_wall_table),\n'
              '    ("print_authority_clock_table", readers::authority_clock_table),\n'
              '    ("print_threshold_law_table", readers::threshold_law_table),\n'
              '    ("print_corrector_law_table", readers::corrector_law_table),\n'
              '    ("print_staircase_law_table", readers::staircase_law_table),\n')
open(p, 'w', encoding='utf-8', newline='').write(s)
print("ok")
