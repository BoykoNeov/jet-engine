// One-off: the burner requests' browser-vs-native differences, worst relative and the smallest values.
import { readFileSync } from 'node:fs';
const [page, nativeTxt] = process.argv.slice(2);
const html = readFileSync(page, 'utf8');
const bytes = Buffer.from(html.match(/const WASM_B64 = "([A-Za-z0-9+/=]+)";/)[1], 'base64');
const enc = new TextEncoder(), dec = new TextDecoder();
let ex = (await WebAssembly.instantiate(bytes, {})).instance.exports; ex.init();
const call = t => { const b = enc.encode(t), p = ex.alloc(b.length); new Uint8Array(ex.memory.buffer, p, b.length).set(b); ex.call(p, b.length); ex.free(p, b.length); return dec.decode(new Uint8Array(ex.memory.buffer, ex.reply_ptr(), ex.reply_len())); };
let n = 0, same = 0, worst = [];
function walk(a, b, path) {
  if (typeof a === 'number') { n++; if (Object.is(a, b)) { same++; return; }
    worst.push({ rel: Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)), abs: Math.abs(a - b), a, path }); return; }
  if (a && typeof a === 'object') for (const k of Object.keys(a)) walk(a[k], b[k], path + '.' + k);
}
for (const [i, line] of readFileSync(nativeTxt, 'utf8').split(/\r?\n/).filter(Boolean).entries()) {
  const [req, nat] = line.split('\t');
  if (!JSON.parse(req).op.startsWith('burner')) continue;
  walk(JSON.parse(call(req)), JSON.parse(nat), '#' + i);
}
worst.sort((x, y) => y.rel - x.rel);
console.log(`${n} burner numbers, ${same} bit-identical; worst relative:`);
for (const w of worst.slice(0, 8)) console.log(`  ${w.rel.toExponential(2)} (abs ${w.abs.toExponential(2)}, value ${w.a.toExponential(3)}) at ${w.path}`);
const small = worst.filter(w => Math.abs(w.a) < 1e-6).sort((x, y) => y.rel - x.rel);
console.log('smallest-magnitude differing values:'); for (const w of small.slice(0, 5)) console.log(`  ${w.rel.toExponential(2)} value ${w.a.toExponential(3)} at ${w.path}`);
