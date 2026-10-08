import io

p = r"M:\claud_projects\jet engine\rust\src\sensed_cap.rs"
s = io.open(p, encoding="utf-8", newline="").read()

reps = [
("// 144 of 1 814 keys, NO panic**, because a reader reaches `_cap_fuel` directly and never enters",
 "// 216 of 2 691 keys, NO panic**, because a reader reaches `_cap_fuel` directly and never enters"),

("""    /// **AND THE WHOLE GUARD IS INERT AT THE SUITE'S OWN MARGIN.** Step 5's drive measured
    /// `n_inert = 0` in every cell at `margin = 0.10` \u2014 10 of 10 rows bind under both laws, so
    /// nothing is filtered and the three caps above are computed to decide a constant. Replacing
    /// this `max` with a `min` moves **0 keys at 0.10 and 33 at 0.20**, where 7 of 9 rows bind.
    /// That is step 4 \u00a7 (b)'s *a drive that picks one cell scores the bug as correct* moved from a
    /// reload guard onto a min-select, and it is why the drive carries two margins.""",
 """    /// **AND THE WHOLE GUARD IS INERT AT THE SUITE'S OWN MARGIN.** Step 5's drive measured
    /// `n_inert = 0` in every cell at `margin = 0.10` on the `phi`-referenced stator arm \u2014 10 of
    /// 10 rows bind under both laws, so nothing is filtered and the three caps above are computed
    /// to decide a constant. Replacing this `max` with a `min` moves **0 keys there and 33 at
    /// `margin = 0.20`**, where 7 of 9 rows bind. That is step 4 \u00a7 (b)'s *a drive that picks one
    /// cell scores the bug as correct* moved from a reload guard onto a min-select, and it is why
    /// the drive carries two margins.
    ///
    /// **AND THE INCIDENCE ARM DOES NOT PROVIDE A SECOND ROUTE, WHICH IS WORTH MORE THAN IF IT
    /// DID.** Rung 69's incidence-referenced rig has `n_inert = 1` of its 2 rows at the suite's
    /// own margin \u2014 the guard is LIVE there where it is dead on the `phi` arm \u2014 and the same
    /// injection still moves **0 keys on it**, because its one filtered row is filtered under both
    /// folds. So exactly ONE of the four driven (arm, margin) cells can score this expression at
    /// all, and *a branch being live* is not the same claim as *a mutation of it being
    /// observable*."""),

("""    /// READER** ([[rust-port-slice-aa-steps2345]]). `taus[1]` is `0.05` and rung 52's lag returns
    /// its ATTACK clock, also `0.05`, at every riding point of an accel ramp \u2014 so step 5's sweep
    /// SWAPPED the two conditions and **0 of 1 814 keys moved**, at both margins. The distinction
    /// is real in the source and undrivable on this plant; it is written the source's way and the
    /// unreachability is disclosed here rather than left for a later slice to wonder about.""",
 """    /// READER** ([[rust-port-slice-aa-steps2345]]). `taus[1]` is `0.05` and rung 52's lag returns
    /// its ATTACK clock, also `0.05`, at every riding point of an accel ramp \u2014 so step 5's sweep
    /// SWAPPED the two conditions and **0 of 2 691 keys moved**.
    ///
    /// **AND THAT IS MEASURED ON TWO STRUCTURALLY DIFFERENT PLANTS, WHICH IS WHY IT IS A CLAIM
    /// ABOUT THE READER AND NOT ABOUT ONE ARM.** `cap_march` sets `tau_rel = 3\u00b7tau_f = 0.15` and
    /// marches to `s_settle = 1.2`, well past the ramp end `r = 0.5`, so a RELEASING point among
    /// the `riding4` survivors would make the two clocks differ and kill the swap \u2014 and whether
    /// one occurs is a property of the trajectory, not of the expression. The drive therefore
    /// carries both stator arms, the `phi`-referenced limiter and rung 69's INCIDENCE-referenced
    /// one, at two margins each: `n_tau_split = 0` at all four. Undrivable on either plant, and
    /// the unreachability is disclosed here rather than left for a later slice to wonder about."""),

("""        // the other way round and 0 of 1 814 keys moved. Written the source's way because the""",
 """        // the other way round and 0 of 2 691 keys moved. Written the source's way because the"""),

("""                    // the relative form alone moves 0 of 1 814 keys, because a live 4x4's masked""",
 """                    // the relative form alone moves 0 of 2 691 keys, because a live 4x4's masked"""),

("""/// `max` return the same number and step 5's sweep measured the substitution at 0 of 1 814 keys.""",
 """/// `max` return the same number and step 5's sweep measured the substitution at 0 of 2 691 keys."""),

("""/// Folding the 341-point trajectory right-to-left instead of left-to-right moves **exactly 4
/// keys** \u2014 the two arms of the two driven bill cells and nothing else in the file.""",
 """/// Folding the 341-point trajectory right-to-left instead of left-to-right moves **exactly 6
/// keys** \u2014 the two arms of each of the three driven bill cells, and nothing else in the file."""),

("""    /// Step 5's drive measured this quantity at `+0.0` **BIT FOR BIT at 8 of 10 rows at the
    /// suite's own margin** (4 of 10 at `0.05`, 3 of 9 at `0.40`). At every one of those rows
    /// `sensed(q, w0)` returns `w0` itself, so a reader that had wrongly written `solve(q) - w0`
    /// \u2014 comparing the solve with itself \u2014 would report the identical `0.0` and be scored as
    /// verifying the identity. The sweep catches it only through the minority of rows where the
    /// last bits differ: **17 keys of 1 814, none of them at `margin = 0.10`'s eight zeros.** So a
    /// step-6 gate on identity (1) must assert on a row where this field is NONZERO, or it is
    /// [[instrument-fed-by-what-it-certifies]] in its purest form \u2014 an instrument certified by
    /// the exactness it exists to report.""",
 """    /// Step 5's drive measured this quantity at `+0.0` **BIT FOR BIT at 18 of the 36 driven
    /// rows**, and the split across the six (arm, margin) cells is the useful part: on the
    /// `phi`-referenced stator arm it is **8 of 10 at the suite's own margin**, 4 of 10 at `0.05`
    /// and 3 of 9 at `0.40`; on rung 69's INCIDENCE arm it is 1 of 1, 1 of 2 and 1 of 4. At every
    /// one of those rows `sensed(q, w0)` returns `w0` itself, so a reader that had wrongly written
    /// `solve(q) - w0` \u2014 comparing the solve with itself \u2014 reports the identical `0.0` and is
    /// scored as verifying the identity. The sweep catches it only through the rows where the last
    /// bits differ: **28 keys of 2 691, of which the suite's own cell contributes 4.** So a step-6
    /// gate on identity (1) must assert on a row where this field is NONZERO, or it is
    /// [[instrument-fed-by-what-it-certifies]] in its purest form \u2014 an instrument certified by
    /// the exactness it exists to report. The incidence arm is the better population and the
    /// thinner one (1, 2 and 4 rows), which is its own constraint on how such a gate is written."""),

("""/// them, and the count is **0 of 29 rows across all three margins** \u2014 so the NaN is unreachable on
/// this plant, the two fold spellings are indistinguishable by any value it can produce, and""",
 """/// them, and the count is **0 of 36 rows \u2014 all three margins, both stator arms** \u2014 so the NaN is
/// unreachable on this plant, the two spellings are indistinguishable by any value it produces, and"""),
]

for old, new in reps:
    assert s.count(old) == 1, repr(old[:80])
    s = s.replace(old, new)

io.open(p, "w", encoding="utf-8", newline="").write(s)
print("docs updated; lines", s.count("\n") + 1)
