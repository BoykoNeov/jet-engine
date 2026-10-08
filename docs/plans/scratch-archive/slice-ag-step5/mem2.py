import io

# --- the step-5 memory file: add the addendum's lesson -----------------------------------------
p = r"C:\Users\boiko\.claude\projects\M--claud-projects-jet-engine\memory\rust-port-slice-ag-step5.md"
s = io.open(p, encoding="utf-8", newline="").read()

old_desc = ('description: "Slice AG step 5 (rung 76\'s readers) \u2014 a mutation sweep\'s verdict is a '
            'property of the GRID, and a grid copied from the test suite inherits the suite\'s '
            'coverage, not the code\'s"')
assert s.count(old_desc) == 1
s = s.replace(old_desc, ('description: "Slice AG step 5 (rung 76\'s readers) \u2014 a mutation sweep\'s '
                         'verdict is a property of the GRID; a grid copied from the suite inherits '
                         'the suite\'s coverage, and one NARROWED from it inherits nothing"'))

old_head = ("`py_max`/`py_min`, and `REF_SCHED`. No gate file \u2014 steps 3/4's precedent. "
            "**1 814 keys,\n`Rust == PyPy` bit for bit on the first run that ever compiled, both "
            "key sets equal, no port fix.**\nSweep: 16 injections, 12 killed, 4 survived, "
            "**16 of 16 verdicts right**.")
assert s.count(old_head) == 1, "head anchor"
s = s.replace(old_head, (
 "`py_max`/`py_min`, `REF_SCHED` and seven named reader defaults. No gate file \u2014 steps 3/4's "
 "precedent.\n**2 691 keys, `Rust == PyPy` bit for bit on the first run that ever compiled, both "
 "key sets equal,\nno port fix.** Sweep: 16 injections, 12 killed, 4 survived \u2014 **15 one-sided "
 "predictions all\nright, plus one hedged that could not lose** (scored *16 of 16* in the first "
 "draft, corrected in\nplace)."))

tail = """
**FIFTH, and it is the same lesson turned on itself: I did not COPY the suite's grid on the `inc`
axis, I NARROWED it.** Every reader call passed `inc = false`, so none of the first 1 814 keys
touched rung 69's incidence-referenced plant \u2014 while `test_rung76.py` sweeps `for inc in (False,
True)` in three tests, all three of them `solve_gain`, the reader whose gate the FOURTH finding had
just booked as self-certifying. `LeverArm` already carries `stator_inc`, so this was a gap in the
drive and not a width gap in the port \u2014 checked first, because if it had not carried it that
would have been a step-1 finding rather than a reason to skip the arm. Re-driven and the whole
sweep re-run on 2 691 keys: **verdicts unchanged**, and three things sharpen. The `tau_auth`
row \u2014 the claim genuinely at risk, since `cap_march` marches past the ramp end with
`tau_rel = 3\u00b7tau_f` and a RELEASING point would have split the clocks \u2014 survives on a second
plant (`n_tau_split = 0` at all four cells). The two grid-sensitive injections stay confined to
(`phi` arm, `margin = 0.20`), and the incidence arm does NOT open a second route even though the
`accel_binds` guard is LIVE there at the suite's own margin: **a branch being live is not the same
claim as a mutation of it being observable.** And the identity's exact-zero count is 18 of 36 rows,
1 of 1/2/4 on the incidence arm against 8 of 10 on the suite's \u2014 so the sharper population for
step 6's gate is also the thinner one.

**And the sweep's own attribution columns went blind on that same edit.** They bucket keys by the
prefixes `E/0.1/`/`F/0.1/`; the new axis renamed everything to `E/i0/0.1/`, so all sixteen rows
reported `0 @0.10, 0 @0.20` \u2014 including the two whose entire point is the split. Caught because
two rows that must be one-sided read zero on both sides beside a nonzero total, which is
arithmetically impossible. **An instrument that partitions can stop matching the data, and it stops
on exactly the edit that makes the partition worth having.**
"""
s = s.rstrip("\n") + "\n" + tail
io.open(p, "w", encoding="utf-8", newline="").write(s)
print("step-5 memory updated")

# --- MEMORY.md hook ---------------------------------------------------------------------------
q = r"C:\Users\boiko\.claude\projects\M--claud-projects-jet-engine\memory\MEMORY.md"
m = io.open(q, encoding="utf-8", newline="").read()
old = ("what a REFUSAL protects and a SWEEP GRID copied from the suite all have an EXPIRY DATE, and")
assert m.count(old) == 1
m = m.replace(old, ("what a REFUSAL protects and a SWEEP GRID copied from \u2014 or NARROWED from \u2014 "
                    "the suite all have an EXPIRY DATE, and"))
io.open(q, "w", encoding="utf-8", newline="").write(m)
print("index updated; lines", m.count("\n") + 1)
