# Rungs 25–30 — The nozzle and the turbine: which shortcuts were earned?

*Plain-language notes. The formal derivations, tests and every number's source are in `docs/rung25-spec.md`
to `docs/rung30-spec.md`. Numbers below are copied from those specs and belong to the cases the specs ran; the
sandbox page shows its own.*

Rung 14 (in `NOTES.md`) left the model with a **bracket** for the exhaust nozzle. The model's own nozzle keeps
the gas's chemistry **frozen**: whatever mixture leaves the burner goes out of the back unchanged. A real
nozzle lets the hot gas **re-react** as it cools: the pieces that heat split apart (carbon monoxide, hydrogen,
and loose oxygen and hydrogen atoms) join back up, and that releases a little more energy into the jet. Frozen is
the lower bound, fully re-reacting the upper, and the truth lies between.

These six rungs look at the shortcuts the engine model takes in its hot end and ask, one at a time, whether
each was **earned**. Four ask about chemistry (25–28), one about the turbine (29), and one about the nozzle's
shape (30). None of them changes the engine's own numbers: each is a check run beside the cycle.

**Two things to know first.**
- **Hot versus lean.** The engine's opening design burns at a turbine inlet temperature of 1500 K. That is
  **lean**: plenty of spare air and little split-apart gas. Most of these effects are negligible there and
  grow fast as the burner runs hotter (the specs push to 2200 or 2400 K). "Dormant lean, real hot" is the
  family's refrain.
- **A race between two clocks.** Whether a reaction keeps up as the gas rushes through the nozzle depends on how
  fast the chemistry runs compared with how long the gas spends there. When the chemistry is much faster it
  keeps up; when it is much slower the mixture **freezes** where it is.

**One correction runs across the whole family.** Rungs 25–28 justified their starting point with a ratio: how far
above its equilibrium amount a species is when it reaches the nozzle. Rung 29 showed that this ratio is largest
exactly where the energy at stake is smallest. Read every "times above equilibrium" below as a statement about
*rates*, not about how much thrust is on the table.

---

## Rung 25 — Even infinitely fast chemistry cannot reach the upper bound

**In one sentence:** a nozzle whose chemistry runs at a finite speed lands between rung 14's two bounds as
expected, but its fastest possible version stops short of the upper bound, because that bound is out of reach
for any real nozzle.

**How it works.** One knob, the **Damköhler number**, says how fast the chemistry runs compared with the gas's
time in the nozzle. At zero the gas stays frozen, and the model reproduces the frozen nozzle exactly. The plan was
that turning the knob all the way up would land on rung 14's fully re-reacting bound. It does not.

**Why not.** The gas reaching the nozzle has already been through the turbine with its chemistry frozen. It is
cooler than when it left the burner but still carries the burner's hotter mixture, with more split-apart pieces
than its new temperature would hold. A real flow must first settle that mismatch, and settling it is a one-way
process that wastes a little of the energy, however fast it happens. Rung 14's upper bound assumed a perfectly
reversible path that no real flow can take. So there are **three** states, not two:
- frozen (the model's nozzle, the lower bound);
- infinitely fast but real (the true ceiling for any actual nozzle);
- perfectly reversible (rung 14's upper bound, which nothing reaches).

**The sizes, in the spec's cases.** At 1500 K the unreachable part is a sliver, 0.001 %. At 2200 K, re-reacting buys about 0.43 % of jet speed, and the unreachable part is a further
0.031 % of jet speed, which is about 7 % of the whole bracket. Those are two different yardsticks, not two
answers.

**Later.** An attempt at a turbine version of this rung (`docs/turbine-march-negative.md`, not a rung) found
that the unreachable gap is *made* by freezing the turbine. It is not built into expansion itself. A turbine
that re-reacted on the way through would hand the nozzle a settled gas and the gap would vanish. Rung 29 explains
why it hardly matters at the opening design.

**On the page.** *Burner → Through the nozzle* shows rung 14's two bounds: "Jet speed: frozen gas / re-reacting"
and "Speed gained by re-reacting". Rung 25 says a real nozzle would gain a little less than that last row, never
more.

---

## Rung 26 — Where the chemistry freezes, and why it is not the temperature

**In one sentence:** with a real reaction rate the chemistry keeps up at the nozzle entrance and freezes partway
down, and that freeze point moves downstream as the burner runs hotter; the cause is the gas thinning out, which
wins *against* the cooling.

**How it works.** Rung 25's one knob is replaced by a real clock. The joining-up of split-apart pieces is
dominated by reactions where three molecules must meet, and their speeds are taken from a standard published
chemistry set (GRI-Mech 3.0, which the model already used for these species' heat data), so no new constants are
invented. At each step down the nozzle the model compares this local chemistry speed with the gas's time there.
Where the chemistry falls behind, the mixture freezes.

**What it found.**
- **Lean, it never switches on.** At 1500 K the chemistry is slower than the gas from the very first step, so the
  nozzle is frozen from the entrance. That independently justifies the model's frozen nozzle at its design point.
- **Hot, it freezes partway, and later the hotter it is.** That motion is what a single fixed knob could never
  show.
- **The cause is density, not cold.** Everyday intuition says reactions slow as gas cools. These reactions do the
  opposite: they need no energy to start and actually run *faster* when cooler. In the spec's 2200 K case, cooling
  alone speeds the clock by ×2.69. What freezes the gas is that it thins out: three molecules must meet, so the
  rate falls with the square of the density, by ×27.1 here. The test: hold the temperature effect fixed and the
  gas still freezes; hold the density fixed instead and by the nozzle exit the chemistry runs 12.1 times faster
  than the gas passes through, so it never freezes.

**What it does not claim.** *Where* exactly the freeze happens rides on one length the model guesses (how long the
nozzle is). Only that the freeze exists when hot, is absent when lean, and moves with temperature is claimed.

**Later.** A study that tried to derive that length from the nozzle's shape (`docs/tau-res-negative.md`, not a
rung) failed to retire the guess (it only swapped it for another), but confirmed every claim above and found the
freeze point moves about three times less than this rung's own run suggests. Rung 29's correction to the
"times above equilibrium" reasoning applies here too.

**On the page.** Not shown.

---

## Rung 27 — The exhaust NO really is frozen

**In one sentence:** every nitric oxide (NO) number since rung 7 assumed the NO the burner makes passes through
the nozzle unchanged, and this rung shows that assumption was right, by a wide margin, at every temperature
tried.

**How it works.** The same method as rung 26, with a clock built from rung 7's own NO reactions (no new
constants). NO is removed when it meets loose oxygen or hydrogen atoms.

**What it found.** The NO clock is slower than the gas from the nozzle entrance onward at every burner
temperature, by 3 to 9 orders of magnitude on this rung's range. Nothing moves; there is no freeze point to
report. And the reason is the **opposite** of rung 26's:

| | the main chemistry (rung 26) | NO (rung 27) |
|---|---|---|
| needs energy to start? | no, so cooling *speeds* it | yes, a large barrier, so cooling *stalls* it |
| how many molecules meet | three, so density counts twice | two, so density counts once |
| temperature and density | pull opposite ways | pull the same way |

In the spec's 2200 K case, cooling alone slows the NO clock more than a thousand times; density alone, ×5.2.
Two clocks in one nozzle, freezing for opposite reasons.

**Later corrections.** Rung 28 kept the verdict and replaced its reasoning. This rung said NO arrives *above* its
equilibrium amount, so its fate is to be destroyed. At the nozzle entrance it actually arrives *below* it when the
burner runs hot. Rung 28 found another reason the bound still holds. It also found the margin is narrower at a
hotter burner than this rung's range showed.

**On the page.** *Burner → Through the nozzle*, the last row, "Exhaust NO over its equilibrium at the exit".
That number is only meaningful because the NO really is frozen, and the card's note says so.

---

## Rung 28 — Right answer, both reasons wrong

**In one sentence:** let the NO clock run on the re-reacting gas of rung 26 instead of a frozen one, and NO freezes
even harder, which rung 27 predicted; but both of rung 27's reasons were wrong, and this rung replaces them.

**Correction one: there are two effects, pulling opposite ways.** Rung 27 expected only one: as the loose oxygen
and hydrogen atoms join back up, there are fewer of them to destroy NO, so the NO clock slows. True. But joining up
also *releases heat*, which warms the gas, and NO's chemistry speeds up sharply when warmer. In the spec's 2200 K
case, the first effect slows the clock to ×0.382, the second speeds it by ×1.130, and together they come to
×0.432. These figures are about the *clock*, how far below the freeze threshold it sits. The NO itself does not
move at all.

**Why the first effect always wins.** Make the chemistry faster and faster: the atoms keep disappearing without
limit, but the heat they release is capped by how much energy was stored in them to begin with. With the chemistry
sped up a million times, at 2200 K, the slowing reaches ×0.0001 while the speeding stops at ×1.21. That limit, not a
lucky balance, is why rung 27's verdict holds.

**Correction two: NO arrives below equilibrium, not above.** At the nozzle entrance with the burner at 1800, 2200
and 2400 K, NO sits at 0.61, 0.31 and 0.45 of its equilibrium amount. It only rises *above* equilibrium as the gas
cools further down. Rung 27's bound still holds, for a reason that covers both cases: under the conditions met
here, rung 27's simple clock is always at least as fast as the exact one, so "the simple clock says frozen"
guarantees "frozen".

**The honest margin.** That reason rests on a quantity staying below 1. It reaches about half of 1 at the hottest
burner, a factor of two, not orders. A later check found the worst case is 0.544, that higher compressor pressure
makes it *smaller*, and that it would only reach 1 at a gas 1.6 to 1.9 times hotter than any this engine can reach
at the nozzle.

**On the page.** Not shown.

---

## Rung 29 — The frozen turbine is fine at design, and the wrong yardstick said otherwise

**In one sentence:** freezing the chemistry through the turbine costs almost nothing at the opening design and
starts to matter only hot; and the "times above equilibrium" ratio that rungs 25–28 relied on gets this backwards.

**How it works.** The same bracket rung 14 built for the nozzle, now for the turbine: frozen chemistry against
chemistry that keeps up perfectly, both giving up the same work to the compressor. No rates, no guessed lengths:
the fully re-reacting turbine is the most any real turbine could shift, so a small answer is final.

**What it found** (the spec's case: compressor pressure ratio 10, cruise at Mach 0.85). The extra temperature at
the turbine exit if the chemistry kept up:

| turbine inlet | extra exit temperature |
|---|---|
| 1500 K | 0.0107 % |
| 2400 K | 1.8592 % |

The threshold chosen for "earned" is 0.1 %, one order below how well the model knows its component losses. At 1500 K
the freeze is earned outright; somewhere between 1800 and 2100 K it stops being earned. Later checks held this over
compressor pressure ratios from 2 to 80 (with a 9.4× margin) and flight speeds from Mach 0.3 to 3.0 (8.8×, the
worst case at take-off speed), and the boundary never fell below 1846 K.

**The finding: the ratio is the wrong yardstick.** At 1500 K the split-apart pieces are 109.4 times above their
equilibrium amount; at 2400 K only 3.3 times. Read that ratio and you would expect the shift to be biggest at
1500 K. It is the reverse: from 1500 to 2400 K the ratio falls 33×, the *amount* of split-apart gas rises 121×, and
the shift rises 174×. The ratio tells you how hard the chemistry is pushing, which is what a *rate* question needs.
It does not tell you how much energy there is, because a hundred times almost nothing is still almost nothing.
A later check sharpened this: the energy is the amount of split-apart gas times the share of it the expansion
actually joins back up.

**On the page.** The Design view's turbine is always frozen. Type a turbine inlet temperature of 2100 K or more on
*Gas model → Equilibrium chemistry* and you are in the band where, in the spec's case, that shortcut is no longer
earned (by under 2 % even at 2400 K).

---

## Rung 30 — The convergent nozzle: the pressure push saves most of the loss

**In one sentence:** a plain narrowing nozzle cannot expand the jet fully at the design pressure ratio; it chokes,
the jet leaves at the speed of sound and above the outside pressure, and specific thrust drops, but only by
6.6 %, because the leftover pressure pushes on the nozzle.

**Why it matters.** Since rung 2 every thrust number expanded the jet all the way down to the outside pressure. At
the design point that means a jet leaving at 1.864 times the speed of sound. A narrowing nozzle cannot do that:
the fastest it can make the gas go is the speed of sound, at its narrowest point, which for a narrowing nozzle is
the exit. So the "fully expanded" nozzle the model always quoted is really an ideal narrowing-then-widening nozzle.
Rung 30 asks what a simple narrowing one, the usual choice for a subsonic engine, actually gives.

**What it found** (the spec's case: compressor pressure ratio 10, Mach 0.85, 1500 K at the turbine inlet).
- The nozzle chokes. Its exit pressure is 3.42 times the outside pressure.
- The jet is 38 % slower, which alone would cut the jet's push by 51 %.
- But the pressure left in the jet pushes on the nozzle exit directly. That **pressure thrust** wins back 87 % of
  the lost push, and makes up 48 % of this nozzle's whole thrust.
- Net: specific thrust 6.6 % lower, fuel per thrust 7.1 % higher.

This is why subsonic engines live with a simple nozzle and supersonic ones fit a widening section or a variable
one.

**What it led to.** Rung 31 (in `31-33-off-design.md`) gave this nozzle a fixed throat area, and a choked throat
of fixed size is what pins where a built engine runs.

**On the page.** *Design the engine → Nozzle and gas → Nozzle → Fixed convergent (the flow picks; may choke)*, or
*Try this → A nozzle that cannot finish the job*. In the page's own engine (experiment 7) thrust falls from 15.97
to 14.91 kN. Note the efficiency split there: "thermal" reads 15 % and "propulsive" 115 %, because that split
counts only the jet's speed and misses the pressure push. *Fly it* always re-runs the design with a convergent
nozzle, since a built engine needs a fixed throat.
