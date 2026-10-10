// Slice 5: the browser build's timing + values on the same grid as `native vals` / `native time`.
import { readFileSync } from 'node:fs';
const bytes = readFileSync(process.argv[2]);
let ex;
const fresh = async () => { ex = (await WebAssembly.instantiate(bytes, {})).instance.exports; };
await fresh();
const fmt = (x) => { let s = x.toExponential(); return s.replace('e+', 'e'); };
const i = [0, 1, 2, 3].map((k) => ex.inlet_part(4, 10.0, 1500.0, k));
const mode = process.argv[3] || 'vals';
if (mode === 'vals') {
  console.log(`inlet ${i.map(fmt).join(' ')}`);
  const PHIS = [0.6, 0.7, 0.8, 0.9, 0.95, 1.0, 1.05, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0];
  for (let c = 0; c < 2; c++) for (const p of PHIS) console.log(`c${c} phi${p} ${fmt(ex.zn_ei(...i, c, p, 25.0, 2))}`);
  for (let c = 2; c < 10; c++) for (const j of [9, 25, 100]) console.log(`c${c} J${j} ${fmt(ex.zn_ei(...i, c, 1.5, j, 2))}`);
  for (let c = 10; c < 12; c++) console.log(`c${c} J25 ${fmt(ex.zn_ei(...i, c, 1.5, 25.0, 2))}`);
} else {
  const names = ["ideal", "ideal+O+prompt", "tau_q 1ms", "jet(11)", "two-stream(12)", "pdf(13)", "pdf_quench(15)",
    "pocket(16)", "transported(18)", "spatial(22)", "spatial_dwell(23)", "spatial_local(24)"];
  for (const level of [2, 1]) for (let c = 0; c < 12; c++) {
    const reps = c <= 1 ? 5 : 1;
    const ts = [];
    for (let k = 0; k < reps; k++) { const t = performance.now(); ex.zn_ei(...i, c, c <= 1 ? 1.0 : 1.5, 25.0, level); ts.push(performance.now() - t); }
    console.log(`L${level} ${names[c].padEnd(18)} point ${Math.min(...ts).toFixed(1)} ms (first ${ts[0].toFixed(1)})`);
  }
  const PHIS = [0.6, 0.7, 0.8, 0.9, 0.95, 1.0, 1.05, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0];
  let t = performance.now();
  for (const p of PHIS) ex.zn_ei(...i, 0, p, 25.0, 1);
  console.log(`ideal phi-bell 17 pts ${(performance.now() - t).toFixed(0)} ms`);
}
