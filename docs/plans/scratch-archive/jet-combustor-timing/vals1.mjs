import { readFileSync } from 'node:fs';
const ex = (await WebAssembly.instantiate(readFileSync(process.argv[2]), {})).instance.exports;
const fmt = (x) => x.toExponential().replace('e+', 'e');
for (const [pc, t4] of [[10, 1500], [4, 1000], [20, 1900], [40, 1600]]) {
  const i = [0, 1, 2, 3].map((k) => ex.inlet_part(4, pc, t4, k));
  const d = `d${pc}/${t4}`;
  console.log(`${d} inlet ${i.map(fmt).join(' ')}`);
  for (const c of [0, 1]) for (const p of [0.8, 1.0, 1.5]) console.log(`${d} c${c} phi${p} ${fmt(ex.zn_ei(...i, c, p, 25.0, 1))}`);
  for (const c of [2, 3, 4, 5, 6, 8, 9]) console.log(`${d} c${c} J25 ${fmt(ex.zn_ei(...i, c, 1.5, 25.0, 1))}`);
}
