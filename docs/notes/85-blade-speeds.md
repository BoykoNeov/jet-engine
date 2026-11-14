# Rung 85 — Blade speeds: how fast the blades may spin, and the redline

*Plain-language notes. The formal record is `docs/rung85-spec.md`; its derivations, sources and every
pre-registered bar are in `docs/plans/rung85-anchor-blade-speed.md`. Numbers below are copied from the spec and
belong to the cases it ran; the sandbox page shows its own.*

Rungs 53–56 moved the variable stators and paid for it in shaft speed: hold the blades' design angle at low power
and the low-pressure shaft spins much faster than its design speed. But the model had no idea how fast a shaft is
*allowed* to spin. Its blades had no size, no material and no speed limit; the number of stages (rung 55) was just
a setting. This rung gives each compressor's blades a real speed, from two physical limits, so that "the shaft
speeds up" can finally be checked against a **redline**.

It is also the first rung built as **knobs** from the start, after the user's decision that the project is a
sandbox: every result is shown for whatever is chosen, and predictions written down beforehand are scored as hits
or misses, a miss being a finding.

---

## Rung 85 — Two walls size the blades, and the redline comes out

**In one sentence:** each compressor's blade speed is set by the slower of two limits, how fast the air may meet
the blade tips and how fast the blade root can spin before the metal yields; the stage count and the redline then
follow, and holding the blades' design angle turns out not to depend on the compressor map's slope at all.

**The two walls.**
- **The airflow wall** is an efficiency target: how fast, relative to the blade, the air may meet the tips of the
  front row at design. Faster means shock losses. The default is 1.4 times the speed of sound, with 1.3 the
  typical airliner value. This sets the design blade speed, and from that the number of stages the compressor
  needs to do its work, rounded up to whole rows.
- **The strength wall** is the material: a spinning blade pulls on its root, and the root yields at a certain
  speed. The default metal is titanium Ti-6Al-4V (827.4 MPa yield, 4429 kg/m³). Certification (14 CFR 33.27) says a
  rotor must survive 1.2 times its redline, so the **redline** is the yield speed divided by 1.2.

At design, the airflow wall sizes the machine; off design, only the strength wall can be crossed. Speed rises as
the stators turn, and a lever **crosses** when a shaft's speed passes its redline.

**The rounding knob.** Stages come in whole numbers, so the design ends up a little short of its airflow limit.
The user's knob decides what to do with the slack: at 0 the blades slow down to close the gap (the earlier
rungs' machine), at 1 they stay at the limit and each row does lighter work.

**What it found.**
- **Holding the design angle cancels the map's slope.** Treat the compressor as one block, hold it at its design
  angle, and the work the compressor does no longer depends on how steeply its speed lines slope; the shaft speed needed is the same for a
  slope anywhere from 0.3 to 2.0, to the solver's precision, while the unscheduled speed moves from 0.78 to 0.70.
  So a speed penalty quoted against the unscheduled speed (rung 53's) depends on the slope only through what it is
  compared with. The rounding knob can therefore reach a stator schedule only through the map's droop, and a
  switch chooses how that droop scales (*Droop grows with blade speed / shrinks with row work*).
- **Where the redlines are crossed** (rounding knob at 0, the default map): only by the lever that turns every
  row's vanes on the low-pressure spool, and only on the way to a target it never reaches, at vane angles of about
  66 to 69 degrees that no real stator turns to. The high-pressure spool never crosses. On the *tilted* map the
  low-pressure whole-block lever crosses too at 1000 K (1.4123 times design speed against its redline of 1.337).
- **The front-row lever stays well inside.** At 1000 K it lowers the front row's tip speed relative to the air by
  13 to 16 % while the low-pressure shaft speeds up.

**What it does to its neighbours.**
- **Rung 55** is bounded: its "the stage count is only a resolution" holds for a stack at a fixed blade speed;
  once the two walls set the design, the stage count is an output of the design.
- **Rung 53**: its lumped speed penalty is corrected in how it should be read (the slope point above).
- **Rung 55**: its finding that turning every row's vanes cannot reach the target below about 1300 K is
  confirmed, though this rung had predicted the opposite.

**What it leaves out.** The strength wall is optimistic: an untapered blade, no disc, no loss of strength when hot,
yield rather than breaking strength. So "crosses the redline" survives the model's errors, while "stays under" is
only one-sided.

**On the page.** *Size the blades* is this rung's view. *The blades* holds the knobs: *Hub-to-tip ratio*, *Airflow
level*, *Material strength ÷ density* (with a *Titanium Ti-6Al-4V* preset), *Overspeed factor*, *Design flow
coefficient*, *Rounding knob λ* and the droop switch. The tiles show each spool's redline; the chart *Stage count and
redline against the airflow level* shows the stages stepping as the airflow level changes; *The lever against the
redlines* runs rung 53's, rung 55's or the front-row stator lever (*A stator lever → Which vanes turn*) down the
throttle and marks each shaft's speed against its redline.
