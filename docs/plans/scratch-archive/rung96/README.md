# Rung 96 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\jet-rung96-probe`, deleted after the rung);
`lib.rs` is a PROTOTYPE inferred-thrust governor built from public pieces only (the pressure-ratio inversion, the LP
speed laws, a 13-row steady table thrust → reading) on the Controls engine. Every script printed REACHABILITY or an
IDENTITY only.

- `reach.rs` → `reach.txt` — anchor § 0 G1/G2/G4/G6 on the plain engine: EPR and LP acceleration rising in fuel at frozen
  speeds; equal EPR ⇒ equal gross thrust; the steady table rising; the prototype marches' clamp counts and legs.
- `reach2.rs` → `reach2.txt` — G1–G4 on the loaded and modified engines (generator HP/LP, cooling, customer bleed, the
  bleed and stator levers): the same checks, gross thrust affine in EPR (the decomposition gate's bar), and the clean
  machine's table.

After the build (the SHIPPED device):
- `score.rs` → `score.txt` — the spec's § 2 gates and § 3 table (H1–H4), the reported calibration errors and droop, the
  step check at the page's limits, and the cost timing.
- `stepcheck.rs` → `stepcheck.txt` — the LP laws' step check BEYOND the page's limits (the law swapped on a built run):
  it set `τ_N` ≥ 0.04 τ and gain ≤ 50 for the fan speed.
- `report.rs` → `report.txt` — reported, not scored: peak `Tt4`, time to 95 % thrust, the thrust overshoot and the HP
  shaft's creep per reading; and H3's named condition (does the HP shaft lead its steady speed?) MEASURED — it did not
  hold on two maps, and H3 hit anyway.
- `crash.rs` → `crash.txt` — the crash map (1 000 requests through `sandbox::call`, seed 13), timed; `slow.rs` →
  `slow.txt` re-times its slowest request alone, and against rung 95's route.
