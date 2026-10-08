// Build a file:// probe page: the spike .wasm as base64, run once on the main thread and once in a
// Worker made from a blob: URL. Each outcome is written into the DOM for --dump-dom to read.
import { readFileSync, writeFileSync } from 'node:fs';
const b64 = readFileSync(new URL('./target/wasm32-unknown-unknown/release/spike.wasm', import.meta.url)).toString('base64');
const page = `<!doctype html><meta charset="utf-8"><title>probe</title>
<pre id="main">main: pending</pre><pre id="worker">worker: pending</pre>
<script>
const B64 = "${b64}";
const bytes = Uint8Array.from(atob(B64), c => c.charCodeAt(0));
WebAssembly.instantiate(bytes, {}).then(({instance}) => {
  document.getElementById('main').textContent = 'main: ' + instance.exports.st(3, 10, 1500);
}).catch(e => { document.getElementById('main').textContent = 'main FAILED: ' + e; });
try {
  const src = "onmessage = async (ev) => { try { const {instance} = await WebAssembly.instantiate(ev.data, {}); postMessage('ok ' + instance.exports.st(3, 10, 1500)); } catch (e) { postMessage('FAILED in worker: ' + e); } };";
  const w = new Worker(URL.createObjectURL(new Blob([src], {type: 'text/javascript'})));
  w.onmessage = ev => { document.getElementById('worker').textContent = 'worker: ' + ev.data; };
  w.onerror = ev => { document.getElementById('worker').textContent = 'worker FAILED onerror: ' + (ev.message || 'no message'); };
  w.postMessage(bytes);
} catch (e) { document.getElementById('worker').textContent = 'worker FAILED to start: ' + e; }
</script>`;
writeFileSync(new URL('./probe.html', import.meta.url), page);
console.log('probe.html', page.length, 'bytes');
