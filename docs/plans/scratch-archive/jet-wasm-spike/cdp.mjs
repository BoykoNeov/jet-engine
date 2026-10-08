// Minimal CDP client (Node 24's global WebSocket): read the probe page's two results, then close
// the browser through its own port. Usage: node cdp.mjs PORT [close]
const port = process.argv[2];
const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
console.log('targets:', list.map(t => `${t.type} ${t.url.slice(0, 70)}`).join(' | '));
const page = list.find(t => t.type === 'page');
function session(wsUrl) {
  const ws = new WebSocket(wsUrl); let id = 0; const pending = new Map();
  ws.onmessage = m => { const d = JSON.parse(m.data); if (pending.has(d.id)) { pending.get(d.id)(d); pending.delete(d.id); } };
  const opened = new Promise(r => ws.onopen = r);
  return { opened, ws, send: (method, params = {}) => new Promise(r => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); }) };
}
if (page) {
  const s = session(page.webSocketDebuggerUrl); await s.opened;
  const r = await s.send('Runtime.evaluate', { expression: "document.getElementById('main')?.textContent + ' || ' + document.getElementById('worker')?.textContent", returnByValue: true });
  console.log('DOM:', r.result?.result?.value);
  s.ws.close();
}
if (process.argv[3] === 'close') {
  const ver = await (await fetch(`http://127.0.0.1:${port}/json/version`)).json();
  const b = session(ver.webSocketDebuggerUrl); await b.opened;
  b.send('Browser.close'); await new Promise(r => setTimeout(r, 1500)); console.log('Browser.close sent');
  process.exit(0);
}
