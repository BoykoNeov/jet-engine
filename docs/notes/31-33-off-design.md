# Rungs 31–33 — Running the engine you built

*Plain-language notes. The formal derivations, tests and every number's source are in
`docs/rung31-spec.md`, `docs/rung32-spec.md` and `docs/rung33-spec.md`. Numbers below are copied from those
specs and belong to the cases the specs ran; the sandbox page shows its own.*

Up to rung 30, every run was a **design**: you chose how hard the compressor squeezes the air (its pressure
ratio) and how hot the gas entering the turbine is, and the model drew you an engine that does exactly that.
But once an engine is built, its parts are fixed metal. Move the throttle, or fly higher or faster, and the
engine cannot pick a new pressure ratio. It runs wherever its fixed parts force it to. These three rungs ask:
**given the hardware, where does the engine run?**

This is the point where the model stops being a calculation you steer and becomes a machine that answers back.

---

## Rung 31 — The hardware decides where the engine runs

**In one sentence:** once the turbine and the exhaust nozzle are built, the compressor's pressure ratio is no
longer a choice — every throttle setting has exactly one place where the engine can run, and those places form
a single line, the **running line**.

**How it works.** The key idea is a **choked** passage. When gas is pushed through a narrowing passage hard
enough, it reaches the speed of sound at the narrowest point, the *throat*. After that, the flow through the
throat cannot be increased by lowering the pressure behind it; it depends only on the pressure and temperature
of the gas arriving. A jet engine has two such throats in a row: the ring of vanes at the turbine's entrance,
and the exhaust nozzle. With both choked, the amount the turbine can expand the gas between them is fixed by
the two throat areas, whatever the throttle does. So the turbine's work on each kilogram of gas is set by how
hot the gas is when it arrives. The shaft has to hand all that work to the compressor (less a little friction).
That fixes how much the compressor squeezes. The pressure ratio has become an *output*.

Throttle back, and the pressure ratio, the airflow and the thrust all fall together, along the one line.

**The surprise.** Textbooks say a turbine between two choked throats always runs at exactly the same
temperature drop, whatever the throttle. That is true only for the textbook gas, whose heat capacity never
changes. On the realistic gas the ratio of turbine-exit to turbine-inlet temperature drifts by **2.84 %** over a
2:1 throttle range (1500 → 800 K at the turbine inlet); on the textbook gas it drifts by **0.000 %**. Splitting
the cause: **81 %** of the drift comes from hot gas holding heat differently from cooler gas, the rest from the
mixture changing as less fuel is burned.

**Later corrections.**
- Rung 31 said the running line comes "without a compressor map". Rung 32 showed that is only half true (below).
- Rung 31 assumed the turbine's entrance vanes are choked. Rung 105 found they are not, for any usual turbine
  stage design. But it also found the consequences of the assumption nearly right: with the nozzle choked, the
  old assumption was within **0.15 %** on this one-shaft engine. The page keeps the old assumption as the
  default and lets you switch (*HP turbine's inlet vanes*, on *Fly it*).

**On the page.** *Fly it* freezes the hardware of the engine you designed and lets you move the throttle and the
flight. Its first point is your design point; move the throttle and the engine moves along its running line.
*Try this → Throttle back to 900 K* (experiment 9 in `docs/sandbox/EXPERIMENTS.md`).

---

## Rung 32 — The compressor map: the hardware sets the work, the map sets what it buys

**In one sentence:** the choked throats decide how much *work* the compressor does, but how much *pressure*
that work buys depends on how efficient the compressor is at that moment — and that needs a compressor map, so
rung 31's "no map needed" over-claimed.

**How it works.** A **compressor map** is the chart that describes a real compressor. For each shaft speed it
draws a line of pressure ratio against airflow; over it lie contours of efficiency, like a hill whose top, the
"efficiency island", sits at the design point. Throttle back and the compressor walks down the hill. Lower
efficiency means the same work buys less pressure, so the pressure ratio and the airflow fall **below** rung 31's
line, by more the further you throttle. The work itself does not move.

Putting a map in also brings in something the model never had before: the **shaft speed**. A map is drawn
against speed, so to read the efficiency off it you need to know how fast the shaft turns. From rung 32 on,
shaft speed is one of the engine's outputs, as a fraction of the design speed (the model gives no real blade
speed until rung 85 sizes the blades).

**Why the turbine barely notices.** One might expect the turbine to slide around its own map too. It does not,
for a structural reason. On a single shaft the shaft speed and the square root of the turbine's inlet
temperature fall together as you throttle back, and the turbine's behaviour depends on their ratio (its speed
measured against the speed of sound in its own gas). That ratio stays within about **1 %** of design over a 2:1
throttle range. So the turbine's efficiency changes by about **2 × 10⁻⁵**, even when the test made the turbine's
map deliberately steep, against about **10⁻²** for the compressor. The compressor is where the map bites.

**Honest limit.** The model has no measured compressor map. Its maps are invented, physically reasonable
shapes, and every claim was checked on three different shapes: the *directions* hold on all of them, the *sizes*
depend on the shape and are not claimed. Rung 32 also draws no stall line — that is rung 36.

**On the page.** *Fly it → Compressor map shape* picks one of the invented shapes; the map chart shows the
speed lines and where your engine sits on them.

---

## Rung 33 — Below choke: the turbine hears the compressor again

**In one sentence:** throttle back far enough and the exhaust nozzle stops being choked; from then on the
turbine's expansion depends on the compressor again — rung 31's "the turbine does not notice the throttle" holds
only while both throats are choked.

**How it works.** At low power the gas reaches the nozzle with too little pressure to reach the speed of
sound. The nozzle then simply lets the gas out at the outside air pressure, and how much it passes depends on the
pressure ratio across it. That ratio depends on how hard the compressor squeezed. So the turbine, the
compressor and the nozzle are tied together again, and the turbine's expansion moves with the throttle — **even
on the textbook gas**, where rung 31's drift was exactly zero. Here the turbine's temperature ratio varies by
about **1.2 %** across the range on the textbook gas. Rung 31's drift came from a subtle property of hot gas;
this one comes from the plumbing, so no choice of gas removes it.

**What it is not.** It is tempting to say the outside air pressure "comes back in". It does not: double the
outside pressure at the same speed and temperature, and every ratio in the engine stays the same — only the
airflow and the thrust double. What couples the turbine is the compressor's pressure *ratio*, not the pressure
*level*.

**The two edges of this range.** Above, the nozzle chokes again and rung 31 takes over (at the design flight,
at a turbine inlet temperature of about 600 K). Below, the jet slows until it leaves no faster than the air
arrived: the engine makes no net thrust, and below that it would be a drag, not an engine. At the design
flight that happens near 440 K. The model reports that as *below idle* rather than quoting a negative thrust.
On the ground, with no ram air to help, the nozzle unchokes at a higher throttle, so ground idle lives in this
range — which is where a real engine spends most of its low-power time.

**On the page.** *Fly it*'s status line says whether the nozzle is choked or unchoked; throttle back and watch it
change. Below idle the page says so in words. (Two solver repairs made in October 2026 let a few low throttles
on hot days run that were refused before; the details are in the rung 33 and 34 specs.)

**Later.** Rungs 97, 103 and 104 carry the unchoked nozzle onto the two-shaft engine, where it slows both shafts.
