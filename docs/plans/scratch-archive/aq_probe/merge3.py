p = r"W:/Claude_projects/jet engine/rust/src/panels/cascades.rs"
s = open(p, encoding='utf-8', newline='').read()
reps = [
("const FLOOR: f64 = 0.55;\nconst LO: f64 = 1000.0;\nconst HI: f64 = 1400.0;\nconst B: f64 = 0.10;\nconst PHI: f64 = 0.80;\n",
 "pub(crate) const FLOOR: f64 = 0.55;\npub(crate) const LO: f64 = 1000.0;\npub(crate) const HI: f64 = 1400.0;\n"
 "pub(crate) const B: f64 = 0.10;\npub(crate) const PHI: f64 = 0.80;\n"),
("type Build = fn(", "pub(crate) type Build = fn("),
("fn machine(build: Build,", "pub(crate) fn machine(build: Build,"),
]
for a, b in reps:
    assert s.count(a) == 1, a
    s = s.replace(a, b)
open(p, 'w', encoding='utf-8', newline='').write(s)

p = r"W:/Claude_projects/jet engine/rust/src/panels/mod.rs"
s = open(p, encoding='utf-8', newline='').read()
a = "pub mod airflow;\n"
assert s.count(a) == 1
s = s.replace(a, "pub mod actuator;\n" + a)
a = '    ("print_full_split_table", cascades::full_split_table),\n'
assert s.count(a) == 1
s = s.replace(a, a + '    ("print_shared_actuator_table", actuator::shared_actuator_table),\n'
              '    ("print_applied_reference_table", actuator::applied_reference_table),\n'
              '    ("print_demand_coordinate_table", actuator::demand_coordinate_table),\n')
open(p, 'w', encoding='utf-8', newline='').write(s)
print("ok")
