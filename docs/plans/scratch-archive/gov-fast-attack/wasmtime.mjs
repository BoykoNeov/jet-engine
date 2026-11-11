// Plan § 45 B5: time the crash map's slowest requests in WebAssembly (Node's V8, the page's engine), the page's own
// call path (alloc / call / reply). Usage: node wasmtime.mjs <wasm> <requests file>
import { readFileSync } from "fs";
const [wasmPath, reqPath] = process.argv.slice(2);
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const ex = instance.exports; ex.init();
const enc = new TextEncoder(), dec = new TextDecoder();
function raw(text) {
  const b = enc.encode(text), p = ex.alloc(b.length);
  new Uint8Array(ex.memory.buffer, p, b.length).set(b);
  ex.call(p, b.length); ex.free(p, b.length);
  return dec.decode(new Uint8Array(ex.memory.buffer, ex.reply_ptr(), ex.reply_len()));
}
for (const line of readFileSync(reqPath, "utf8").split("\n").filter(Boolean)) {
  const t0 = performance.now(); const r = JSON.parse(raw(line)); const ms = performance.now() - t0;
  const c = JSON.parse(line).controls;
  console.log(`${(ms / 1000).toFixed(2)} s  ok=${r.ok} route=${r.route} ds=${r.ds} pts=${r.s ? r.s.length : "-"}/${r.expected_points} att=${c.tau_att} rel=${c.tau_rel} ramp=${c.ramp}`);
}
