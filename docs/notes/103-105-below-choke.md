# Rungs 103–105 — Below choke on two shafts, and two throats that were never choked

*Plain-language notes. The formal derivations, tests and every number's source are in `docs/rung103-spec.md` to
`docs/rung105-spec.md`. Numbers below are copied from those specs and belong to the cases the specs ran; the
sandbox page shows its own.*

Rung 31's whole picture rests on **choked throats**: passages where the gas reaches the speed of sound, so the flow
through them no longer depends on the pressure behind. The two-shaft engine (rung 38) assumed three: the HP
turbine's entrance vanes, the LP turbine's entrance vanes, and the exhaust nozzle. Rung 97 let the nozzle unchoke.
These three rungs follow that through the steady engine and then question the other two throats. Both turn out never
to have been choked. One mattered; the other barely did.

---

## Rung 103 — The steady engine below choke

**In one sentence:** once the nozzle unchokes, the HP shaft can "hear" the LP compressor, something the fully choked
engine made impossible.

**How it works.** Rung 39 found a one-way street on the choked two-shaft engine: the LP compressor's efficiency
cannot reach the HP compressor's pressure ratio at all. Below choke, the LP turbine's expansion depends on the
pressure the nozzle pushes with, and that pressure carries the LP compressor's work. So a change in the LP
compressor now reaches the HP shaft. All six steady solvers were re-solved with the LP turbine free, and on every
one the LP shaft runs slower than it did with the nozzle assumed choked (174 of 174 cases), by up to 69 % deep below choke.

**Later correction.** Rung 103 measured the *direction* of that new link: a better LP compressor *lowered* the HP
pressure ratio. Rung 104 found the direction belonged to the old choked assumption. With the LP turbine's real flow
law, a better LP compressor *raises* the HP pressure ratio (50 of 50 cases). The link is real; its sign was not.

---

## Rung 104 — The LP turbine's throat was never choked

**In one sentence:** the LP turbine's entrance vanes never quite choke, so below choke the nozzle's back-pressure
slows **both** shafts, not the LP shaft alone.

**How it works.** The LP turbine's vanes reach at most 0.998 of the pressure ratio at which they would choke. So
the flow they pass follows the LP turbine's own expansion, and the HP turbine, whose exit pressure those vanes set,
feels the LP turbine. The model now carries a **flow law** for that turbine. The default is **Stodola's ellipse**,
the classic rule for a turbine of many rows; a two-row and a one-row law are there to compare, and so is the old
choked assumption.

**What it found.** Rung 97 said the back-pressure took work off the LP shaft alone: the HP shaft's slowdown was 1/70
of the LP's. Under the ellipse the HP shaft slows **at least 0.535×** as much as the LP shaft in every one of 251
cases, and the LP shaft slows about half as much as rung 97 said. The stiffer the law, the more is shared.

**Where it does nothing.** With the nozzle choked, every law gives the old answer exactly on the textbook gas: the
choked nozzle fixes the LP turbine's expansion anyway. So the page's default engine moves only below choke.

**What it corrected.** Rung 97 (the LP shaft *alone*), rung 102 (the HP shaft's speed-up at a held light-off was
really a slowdown) and rung 103 (the sign of the new link).

**On the page.** *LP turbine's inlet vanes* (on *Controls*, *Blades* and *Cool the turbine*): *Many rows: Stodola's
ellipse (the default) / Vane and rotor, half the drop each / The vane takes the whole drop / Choked, as rungs
38-103 assumed*.

---

## Rung 105 — The HP turbine's vanes are not choked either

**In one sentence:** the HP turbine's vanes would choke only in a stage built quite unlike real ones, so the old
assumption was wrong, but its consequences were nearly right.

**How it works.** A turbine stage splits its pressure drop between the fixed vanes and the spinning rotor. The
rotor's share is the stage's **reaction**. The vanes choke only if the reaction is at most **0.18–0.20** on these
engines; published single stages run **0.25–0.5**. So at any usual reaction the vanes never choke, not even at the
design point.

**What it found.** With the nozzle choked, the old assumption was within **0.15 %** on the one-shaft engine and
**0.20 %** on the two-shaft one (reaction 0.3, realistic gases), and exactly right on the textbook gas. Below choke
the real vanes hand back a little of rung 104's HP slowdown: the HP shaft keeps a little more speed. Rung 104's
finding survives (the HP shaft's share stays 0.52–0.57).

**The user's choice.** Because the law costs computing time and changes so little, it ships as a knob, **off by
default**.

**On the page.** *HP turbine's inlet vanes*: *Choked, as rungs 31-104 assumed (the default)* or *Vane and rotor in
series, split by the stage's reaction*, with *Stage reaction* (on *Fly it*, *Controls*, *Blades*, *Cool the
turbine*).
