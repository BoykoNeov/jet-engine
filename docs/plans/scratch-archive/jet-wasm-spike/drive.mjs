// Launch a headless Chrome with its OWN profile + port (PID recorded), open URL, poll an
// expression until it stops saying "pending", print it, then close via the port; kill by PID
// only if it survives. Usage: node drive.mjs URL "EXPR"
import { spawn, execFileSync } from 'node:child_process';
const [url, expr] = process.argv.slice(2);
const PORT = 9334, PROFILE = 'W:\\temp\\claude\\jet-sandbox-chrome-profile';
const chrome = spawn('C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
  ['--headless=new', '--disable-gpu', `--user-data-dir=${PROFILE}`, `--remote-debugging-port=${PORT}`,
   '--no-first-run', '--disable-extensions', 'about:blank'], { stdio: 'ignore' });
console.log('chrome PID', chrome.pid);
const sleep = ms => new Promise(r => setTimeout(r, ms));
async function json(path, method = 'GET') {
  for (let i = 0; i < 50; i++) {
    try { return await (await fetch(`http://127.0.0.1:${PORT}${path}`, { method })).json(); } catch { await sleep(200); }
  }
  throw new Error('no CDP port');
}
function session(wsUrl) {
  const ws = new WebSocket(wsUrl); let id = 0; const pending = new Map();
  ws.onmessage = m => { const d = JSON.parse(m.data); if (pending.has(d.id)) { pending.get(d.id)(d); pending.delete(d.id); } };
  const opened = new Promise(r => ws.onopen = r);
  return { opened, ws, send: (method, params = {}) => new Promise(r => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); }) };
}
let code = 1;
try {
  const ver = await json('/json/version');
  const tab = await json('/json/new?' + encodeURI(url), 'PUT');
  const s = session(tab.webSocketDebuggerUrl); await s.opened;
  let v = 'pending';
  for (let i = 0; i < 60 && /pending/.test(v); i++) {
    await sleep(250);
    const r = await s.send('Runtime.evaluate', { expression: expr, returnByValue: true });
    v = String(r.result?.result?.value ?? JSON.stringify(r.result?.exceptionDetails ?? r));
  }
  console.log('RESULT:', v); code = /FAILED|pending/.test(v) ? 1 : 0;
  s.ws.close();
  const b = session(ver.webSocketDebuggerUrl); await b.opened; b.send('Browser.close');
} catch (e) { console.log('driver error:', e.message); }
await sleep(2000);
if (chrome.exitCode === null) {
  console.log('still alive; taskkill by PID', chrome.pid);
  try { execFileSync('taskkill', ['/F', '/PID', String(chrome.pid), '/T']); } catch {}
}
process.exit(code);
