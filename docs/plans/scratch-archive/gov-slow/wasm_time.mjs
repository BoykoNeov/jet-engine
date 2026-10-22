// Time each request line of a file through a sandbox_wasm.wasm (node). Usage: node wasm_time.mjs WASM REQS
import { readFileSync } from 'node:fs';
const [wasmPath, reqPath] = process.argv.slice(2);
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const x = instance.exports; x.init();
const enc = new TextEncoder(), dec = new TextDecoder();
for (const req of readFileSync(reqPath, 'utf8').split('\n').filter(l => l.trim())) {
  const b = enc.encode(req); const p = x.alloc(b.length);
  new Uint8Array(x.memory.buffer, p, b.length).set(b);
  const t = performance.now(); x.call(p, b.length); const dt = (performance.now() - t) / 1000;
  x.free(p, b.length);
  const out = dec.decode(new Uint8Array(x.memory.buffer, x.reply_ptr(), x.reply_len()));
  console.log(`${dt.toFixed(2)} s (${out.slice(0, 40)})`);
}
