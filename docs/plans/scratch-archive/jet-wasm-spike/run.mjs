// Load the spike .wasm under Node, print the same lines as examples/native.rs, then force a panic.
import { readFileSync } from 'node:fs';
const bytes = readFileSync(new URL('./target/wasm32-unknown-unknown/release/spike.wasm', import.meta.url));
const hex = x => { const b = new DataView(new ArrayBuffer(8)); b.setFloat64(0, x); return b.getBigUint64(0).toString(16).padStart(16, '0'); };
let { instance } = await WebAssembly.instantiate(bytes, {});
let e = instance.exports;
e.init();
for (let k = 0; k < 4; k++) for (const [p, t] of [[10, 1500], [20, 1700], [4, 1100]]) {
  const n = 20, t0 = performance.now(); let s = 0;
  for (let i = 0; i < n; i++) s = e.st(k, p, t);
  const us = (performance.now() - t0) / n * 1e3;
  console.log(`${k} ${p} ${t} ${hex(s)} ${hex(e.tsfc(k, p, t))} ${us.toFixed(0)}`);
}
// An impossible design: Tt4 below the compressor exit.
try { e.st(0, 30, 500); console.log('no panic?'); }
catch (err) {
  const m = new Uint8Array(e.memory.buffer, e.msg_ptr(), e.msg_len());
  console.log('TRAP:', err.constructor.name, '| message:', new TextDecoder().decode(m));
  // Does the same instance still answer after a trap?
  try { console.log('after trap, same instance:', e.st(0, 10, 1500)); } catch (e2) { console.log('same instance dead:', e2.message); }
  ({ instance } = await WebAssembly.instantiate(bytes, {})); e = instance.exports; e.init();
  console.log('fresh instance:', e.st(0, 10, 1500));
}
console.log('wasm bytes:', bytes.length);
