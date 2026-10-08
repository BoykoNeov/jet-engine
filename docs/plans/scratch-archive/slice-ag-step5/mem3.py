import io

p = r"C:\Users\boiko\.claude\projects\M--claud-projects-jet-engine\memory\rust-port-slice-ag-step5.md"
s = io.open(p, encoding="utf-8", newline="").read()

reps = [
("and on the first grid this step would have written them up as *defences with no reader* beside the\nfour genuine ones.",
 "and on the first grid this step would have written them up as *defences with no reader* beside the\nfive genuine ones \u2014 seven in all, two of them false."),
("Census `10 / 59 / 47` \u2192 `10 / 63 / 51`, **no arrears** \u2014 step 4's procedural repair held, and this\nstep ran `pytest` before shipping rather than two steps later.",
 "Census `10 / 59 / 47` \u2192 `10 / 63 / 51`, then \u2192 **`10 / 70 / 56`** in the addendum's own five\nsignature citations, **no arrears** at either \u2014 step 4's procedural repair held, and this step ran\n`pytest` before shipping rather than two steps later."),
("faithfulness (`dS == 0` at 0 of 29 rows). Not a regression \u2014 a property of the kind of step.",
 "faithfulness (`dS == 0` at 0 of 36 rows, both stator arms). Not a regression \u2014 a property of the\nkind of step."),
("`solve_gain`'s `fixed_point` is `+0.0` bit for bit at **8 of 10 rows at the suite's own margin**, so\na reader that wrongly compared the solve with ITSELF returns the same float there; the injection is\ncaught only by the minority of rows where the last bits differ (17 keys of 1 814, none at those\neight).",
 "`solve_gain`'s `fixed_point` is `+0.0` bit for bit at **8 of 10 rows at the suite's own margin** and\n18 of the 36 driven rows overall, so a reader that wrongly compared the solve with ITSELF returns\nthe same float there; the injection is caught only by the rows where the last bits differ (28 keys\nof 2 691, of which the suite's own cell contributes 4)."),
("Also measured: dropping the `accel` argument in the READER is **silent** \u2014 killed by value at 144\nkeys with no panic",
 "Also measured: dropping the `accel` argument in the READER is **silent** \u2014 killed by value at 216\nkeys with no panic"),
("`fuel_int`'s fold direction moves **exactly 4 keys**, which is P2 asked of the port.",
 "`fuel_int`'s fold direction moves **exactly 6 keys** \u2014 two arms of each of the three driven bill\ncells \u2014 which is P2 asked of the port."),
]
for old, new in reps:
    assert s.count(old) == 1, repr(old[:60])
    s = s.replace(old, new)
io.open(p, "w", encoding="utf-8", newline="").write(s)
print("stale numbers repaired")
