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
