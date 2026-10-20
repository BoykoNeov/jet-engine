# The fuel closures' lean wall follows the run — probes (sandbox plan § 40; the scratch crates, the throwaway worktree and the crash-map builds were temporary and are deleted)

All `[[bin]]`s of one scratch crate built against the repo (HEAD `e4d9756` before the change, the repo after), and of a
twin crate built against a throwaway worktree. Their temp-folder paths (`W:/temp/claude/low-speed/…`) point at a folder
now deleted.

- `p1.rs` → `p1.txt` — the design flight, four maps, starts 400–600 K: each start's own mixture against the wall
  (0.004), the points each slam gets, its stop. Two mechanisms: first-step stops (start leaner than, or just above, the
  wall) and later stops at the RICH wall.
- `p2.rs` → `p2.txt` — every first-step stop on § 39's grid (five variants × six flights × four maps, HEAD): 1 691 =
  1 553 starts leaner than 0.004 + 138 roots within one scan step under it, 0 other (rung 43's wall formula, which the
  bled closures' walls follow only approximately — `p3` checks the cause by running the closure instead).
- `p3.rs` → `p3_head.txt` (HEAD), `p3_wt.txt` (a throwaway worktree with `F_FLOOR` = 0.0005), `p3_new.txt` (the
  change) and `p3_cmp_new.txt` (HEAD vs the change, per cell): the lowest start that builds, the lowest whose whole
  default slam finishes, how the slams between stop, and the first point against the start's own mixture.
- `p4.rs` → `p4.txt` (the worktree) — the design flight's low starts with the acceleration schedule on (all run), and
  the worktree's leftover first-step stops (starts at 0.14–0.48 g/kg, under its 0.5 g/kg).
- `p5.rs` — the one first-step stop the change leaves (customer bleed, SL +40 K, tilted, 440 K: LP shaft at 15 %).
- `crash101_rep.rs` — `../rung101/crash101.rs` with one added line printing each full reply (`REP`); `../idle-floor/crash.rs`
  unchanged (it prints its replies). Both run against HEAD and the change.
- `p6.rs` → `p6_36.txt`, `p6_101.txt` — the two maps' replies compared request by request (outcome, points, the
  bit-equal prefix). `p7.rs` — the runs that completed both times but differ: how much (all governed).
