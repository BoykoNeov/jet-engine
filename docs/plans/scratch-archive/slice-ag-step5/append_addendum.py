import io

plan = r"M:\claud_projects\jet engine\docs\plans\todo-rust-port.md"
t = io.open(plan, encoding="utf-8", newline="").read()

# The two bookkeeping corrections, fixed IN PLACE as (m) says.
old_a = ("Both are real defects in load-bearing expressions. On the suite's grid alone both score\n"
         "**SURVIVED**, and this section would have written them up beside \u00a7 (c)'s four genuine\n"
         "unreachabilities \u2014 *a defence with no reader*, four times over, with two of the four "
         "false.")
new_a = ("Both are real defects in load-bearing expressions. On the suite's grid alone both score\n"
         "**SURVIVED**, and this section would have written them up beside \u00a7 (c)'s **five** "
         "genuine\nunreachabilities \u2014 *a defence with no reader*, **seven** times over, with two "
         "of the seven\nfalse. (Corrected in place; the first draft said *four* against \u00a7 (c)'s "
         "table of five, which is\n\u00a7 5.31 (i)'s own leading lesson happening inside the section "
         "that quotes it \u2014 see \u00a7 (m).)")
assert t.count(old_a) == 1, "section (a) closing sentence not found"
t = t.replace(old_a, new_a)

old_g = ("RIGHT** \u2014 and, as at step 4, the verdict axis is the WEAK reading. The strong one is that "
         "**two of\nthose sixteen verdicts would have been WRONG on the grid this drive started "
         "with**.")
new_g = ("RIGHT** \u2014 and, as at step 4, the verdict axis is the WEAK reading. The strong one is that "
         "**two of\nthose sixteen verdicts would have been WRONG on the grid this drive started "
         "with**. (\u00a7 (m) corrects\nthe score to **15 one-sided predictions plus one hedged**: "
         "injection 15 was registered two-sidedly\nand could not lose.)")
assert t.count(old_g) == 1, "headline sentence not found"
t = t.replace(old_g, new_g)

t = t.rstrip("\n") + "\n" + io.open(r"W:\temp\claude\slice-ag-step5\addendum.md",
                                    encoding="utf-8", newline="").read()
io.open(plan, "w", encoding="utf-8", newline="").write(t)
print("plan lines:", t.count("\n") + 1)
