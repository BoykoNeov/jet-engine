// The built page, in a REAL browser (docs/plans/sandbox-plan.md § 6): the one layer the Rust tests
// and check.mjs cannot reach -- the loader, the Worker, the "does not run" panel, the linked knobs.
// The project's pages once had no test and looked fine (memory: visuals-model-binding).
//
// A headless Chrome with its OWN profile folder (under W:\temp\claude) and its own debugging port
// (Chrome picks a free one and writes it to the profile), its PID recorded at launch. It is shut
// down through that port; only if it survives is it killed -- by that PID, never by name.
//
// Usage: node browser.mjs PAGE NATIVE_TXT       Prints a cargo-style "test result:" line.
import { spawn, execFileSync } from 'node:child_process';
import { existsSync, readFileSync, mkdirSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const [page, nativeTxt] = process.argv.slice(2);
const CHROME = process.env.SANDBOX_CHROME || [
  'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
  'C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe',
].find(existsSync);
if (!CHROME) { console.log('no Chrome found (set SANDBOX_CHROME)'); console.log('test result: FAILED. 0 passed; 1 failed; 0 ignored'); process.exit(1); }
const PROFILE = `W:\\temp\\claude\\jet-sandbox-chrome-${process.pid}`;
mkdirSync(PROFILE, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));

const chrome = spawn(CHROME, ['--headless=new', '--disable-gpu', `--user-data-dir=${PROFILE}`,
  '--remote-debugging-port=0', '--no-first-run', '--disable-extensions', 'about:blank'], { stdio: 'ignore' });
console.log(`chrome PID ${chrome.pid}, profile ${PROFILE}`);

function session(wsUrl) {
  const ws = new WebSocket(wsUrl); let id = 0; const pending = new Map(); const events = [];
  ws.onmessage = m => { const d = JSON.parse(m.data); if (d.id && pending.has(d.id)) { pending.get(d.id)(d); pending.delete(d.id); } else if (d.method) events.push(d); };
  const opened = new Promise(r => ws.onopen = r);
  const send = (method, params = {}) => new Promise(r => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
  return { opened, ws, send, events };
}

let passed = 0, failed = 0;
function check(name, ok, detail = '') {
  if (ok) { passed++; console.log(`test ${name} ... ok`); }
  else { failed++; console.log(`test ${name} ... FAILED${detail ? '\n  ' + detail : ''}`); }
}

// The bars of check.mjs, for comparing the page's opening run with the native one.
function close(a, b, path = '') {
  if (typeof a === 'number') {
    const isS = /\.s$|\.ts_(burner|reject)\.\d+\.0$/.test(path);
    const d = isS ? Math.abs(a - b) : Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b), 1e-300);
    return a === b || d <= (isS ? 1e-6 : 1e-9) ? null : `${path}: ${a} vs ${b}`;
  }
  if (a && typeof a === 'object') {
    for (const k of Object.keys(b)) { const e = close(a[k], b[k], `${path}.${k}`); if (e) return e; }
    return null;
  }
  return a === b ? null : `${path}: ${JSON.stringify(a)} vs ${JSON.stringify(b)}`;
}

let version = null;     // read as soon as the port is up, so the finally below can always close
try {
  let port = null;
  for (let i = 0; i < 100 && !port; i++) {
    try { port = readFileSync(`${PROFILE}\\DevToolsActivePort`, 'utf8').split(/\r?\n/)[0]; } catch { await sleep(100); }
  }
  if (!port) throw new Error('Chrome never opened its debugging port');
  const get = async (path, method = 'GET') => (await fetch(`http://127.0.0.1:${port}${path}`, { method })).json();
  version = await get('/json/version');
  const tab = await get('/json/new?about:blank', 'PUT');
  const s = session(tab.webSocketDebuggerUrl); await s.opened;
  await s.send('Runtime.enable');
  const ev = async (expression) => {
    const r = await s.send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
    if (r.result?.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 300));
    return r.result?.result?.value;
  };
  const waitFor = async (expr, ms = 15000) => {
    for (let t = 0; t < ms; t += 100) { if (await ev(expr)) return true; await sleep(100); }
    return false;
  };
  await s.send('Page.navigate', { url: pathToFileURL(page).href });
  const native = JSON.parse(readFileSync(nativeTxt, 'utf8').split(/\r?\n/)[0].split('\t')[1]);

  // 1. The page loads from a double-click (file://) and starts the model off the page's thread.
  const started = await waitFor("window.sandbox && window.sandbox.state !== 'loading'");
  const mode = started ? await ev('window.sandbox.mode') : null;
  check('page_starts_the_model_in_a_worker', started && mode === 'worker', `state/mode: ${started}/${mode}`);

  // 2. Its opening design is the native opening run (the defaults: the production cycle).
  const first = await ev('window.sandbox.last');
  const e2 = first ? close(first, native) : 'no result';
  check('opening_design_matches_the_native_run', !e2, e2);
  const shown = await ev("document.getElementById('v-thrust').textContent");
  check('the_thrust_tile_shows_the_run', first && shown === (first.thrust / 1e3).toLocaleString('en-US', { minimumFractionDigits: 3, maximumFractionDigits: 3 }) || (first && Math.abs(parseFloat(shown.replace(/,/g, '')) - first.thrust / 1e3) < 0.01 * first.thrust / 1e3), `tile ${shown}`);

  // 3. A design that fails INSIDE the model: the panel, its plain words, the model's own message.
  const trapState = await ev(`window.sandbox.set({gas:'reacting', pi_c:1.02, Tt4:2300, M0:0.05, eta_c:0.6, eta_t:0.6, T0:216.65, p0:5529.3})`);
  const bad = await ev(`[document.getElementById('bad').classList.contains('show'), document.getElementById('bad-plain').textContent, document.getElementById('bad-raw').textContent]`);
  check('a_model_failure_shows_the_does_not_run_panel', trapState === 'does-not-run' && bad[0] && /rich/.test(bad[1]) && /rich mixture/.test(bad[2]), JSON.stringify([trapState, bad]));

  // 4. ...and the model recovers: a precheck refusal next, on the re-created instance.
  const preState = await ev(`window.sandbox.set({gas:'equilibrium', pi_c:10, Tt4:400, M0:0.85, eta_c:0.88, eta_t:0.9, T0:250, p0:50000})`);
  const plain = await ev(`document.getElementById('bad-plain').textContent`);
  check('a_precheck_refusal_names_the_reason', preState === 'does-not-run' && /cool the air/.test(plain), JSON.stringify([preState, plain]));

  // 5. Reset brings back the opening design exactly (same browser, same build: bit for bit).
  await ev(`document.getElementById('reset').click()`);
  await waitFor(`window.sandbox.state === 'ran'`);
  const again = await ev('window.sandbox.last');
  check('reset_reproduces_the_opening_design', JSON.stringify(again) === JSON.stringify(first), 'differs after reset');
  const panelGone = await ev(`!document.getElementById('bad').classList.contains('show')`);
  check('the_panel_clears_when_a_design_runs', panelGone);

  // 6. The linked knobs: moving altitude moves T0/p0 through the standard atmosphere.
  await ev(`(() => { const a = document.getElementById('altitude'); a.value = '11000'; a.dispatchEvent(new Event('input')); })()`);
  const moved = await waitFor(`window.sandbox.last && Math.abs(window.sandbox.last.ambient.altitude - 11000) < 0.01`);
  const amb = await ev('window.sandbox.last.ambient');
  check('altitude_drives_air_temperature_and_pressure', moved && amb.p0 > 22000 && amb.p0 < 23000, JSON.stringify(amb));

  // 7. Pin & compare: pin, change the turbine temperature, and the change columns fill.
  await ev(`document.getElementById('pin').click()`);
  await ev(`window.sandbox.set({Tt4: 1600})`);
  const cmp = await ev(`[document.getElementById('perf-delta').textContent, document.getElementById('d-thrust').textContent, document.getElementById('pin-legend').hidden]`);
  check('pin_and_compare_shows_the_change', cmp[0] === 'Change' && /^\+/.test(cmp[1]) && cmp[2] === false, JSON.stringify(cmp));

  // 8. Nothing threw on the page along the way.
  const thrown = s.events.filter(e => e.method === 'Runtime.exceptionThrown').map(e => e.params.exceptionDetails?.exception?.description || e.params.exceptionDetails?.text);
  check('no_uncaught_errors_on_the_page', thrown.length === 0, thrown.join(' | ').slice(0, 400));

  s.ws.close();
} catch (e) {
  check('browser_session', false, e.message);
} finally {
  // Close through its OWN port, on success AND failure.
  if (version) {
    try { const b = session(version.webSocketDebuggerUrl); await b.opened; b.send('Browser.close'); } catch {}
  }
}
for (let i = 0; i < 30 && chrome.exitCode === null; i++) await sleep(100);
if (chrome.exitCode === null) {
  console.log(`Chrome survived Browser.close; taskkill by its PID ${chrome.pid}`);
  try { execFileSync('taskkill', ['/F', '/PID', String(chrome.pid), '/T']); } catch {}
}
// The launcher PID can exit while the browser it started lives on (seen 2026-10-06), so the exit
// code alone proves nothing. Look for any process still carrying THIS run's unique profile path.
// It is never killed here: it is reported, loudly, for a person to close by its PID.
// Chrome's helpers drain AFTER the browser process exits, and under the full gate's load that
// took longer than a fixed 0.5 s (2026-10-07: eleven helpers listed, all gone a minute later). So
// look until the list is empty, and fail only if it is still not empty after 15 s.
let leftovers = '';
for (let t = 0; t < 15000; t += 500) {
  await sleep(500);
  try {
    leftovers = execFileSync('powershell', ['-NoProfile', '-Command',
      `Get-CimInstance Win32_Process -Filter "Name='chrome.exe'" | Where-Object { $_.CommandLine -like '*${PROFILE.split('\\').pop()}*' } | ForEach-Object { $_.ProcessId }`],
      { encoding: 'utf8' }).trim();
  } catch (e) { leftovers = 'could not list processes: ' + e.message; }
  if (leftovers === '') break;
}
check('chrome_left_nothing_running', leftovers === '', `still running with this run's profile, PIDs: ${leftovers.replace(/\s+/g, ' ')}`);
console.log(`test result: ${failed ? 'FAILED' : 'ok'}. ${passed} passed; ${failed} failed; 0 ignored`);
process.exit(failed ? 1 : 0);
