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
import { join } from 'node:path';

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
  const shot = async (name, width = 1280, dark = false) => {
    if (!process.env.SANDBOX_SHOTS) return;
    const { writeFileSync } = await import('node:fs');
    mkdirSync(process.env.SANDBOX_SHOTS, { recursive: true });
    await s.send('Emulation.setDeviceMetricsOverride', { width, height: 900, deviceScaleFactor: 1, mobile: width < 500 });
    await s.send('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: dark ? 'dark' : 'light' }] });
    await sleep(400);
    const r = await s.send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true });
    writeFileSync(join(process.env.SANDBOX_SHOTS, `${name}.png`), Buffer.from(r.result.data, 'base64'));
    await s.send('Emulation.clearDeviceMetricsOverride'); await s.send('Emulation.setEmulatedMedia', { features: [] });
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

  // 8. FLY IT (slice 3). Back to the opening design, then freeze it: the first fly point is the
  // design flown at its own point -- the native `{"op":"fly","fly":{}}` -- and lands on 100 % speed.
  await ev(`document.getElementById('reset').click()`);
  // Wait for the RESET's own run (an older finished run also reads 'ran').
  await waitFor(`window.sandbox.state === 'ran' && window.sandbox.last.nu === undefined && window.sandbox.last.thrust === ${first.thrust}`);
  await shot('design-light');
  await ev(`document.getElementById('mode-fly').click()`);
  const flew = await waitFor(`window.sandbox.view() === 'fly' && window.sandbox.state === 'ran' && window.sandbox.last && window.sandbox.last.nu !== undefined`);
  const fly0 = await ev('window.sandbox.last');
  const nativeFly = JSON.parse(readFileSync(nativeTxt, 'utf8').split(/\r?\n/).find(l => l.startsWith('{"op":"fly","fly":{}}')).split('\t')[1]);
  const e8 = fly0 ? close(fly0, nativeFly) : 'no fly result';
  check('fly_it_opens_on_the_design_point_and_matches_native', flew && !e8 && Math.abs(fly0.nu - 1) < 1e-9, e8 || `nu ${fly0 && fly0.nu}`);
  const vis = await ev(`['fly-set','fly-tiles','map-card','slam-set','slam-card','cycle-set'].map(i => document.getElementById(i).hidden)`);
  check('fly_mode_shows_its_knobs_and_hides_the_design', JSON.stringify(vis) === '[false,false,false,false,false,true]', JSON.stringify(vis));
  // The frozen line shows BOTH: the design as set (slice 1's own run) and the convergent re-run.
  const frozen = await ev(`document.getElementById('frozen').textContent`);
  const kn = (frozen.match(/[\d,.]+ kN/g) || []).map(t => parseFloat(t.replace(/,/g, '')));
  check('the_frozen_line_shows_the_design_as_set_and_the_re_run', /As you set it \(a fully expanded nozzle, equilibrium-chemistry gas\)/.test(frozen)
        && kn.length === 2 && Math.abs(kn[0] - first.thrust / 1e3) < 0.01 && Math.abs(kn[1] - fly0.design_point.thrust / 1e3) < 0.01, frozen);

  // 9. Throttle back: the shaft slows, and the running line streams onto the map.
  await ev(`(() => { const a = document.getElementById('fly-Tt4'); a.value = '1200'; a.dispatchEvent(new Event('input')); })()`);
  const slowed = await waitFor(`window.sandbox.last && window.sandbox.last.fly && window.sandbox.last.fly.Tt4 === 1200`);
  const nu12 = await ev('window.sandbox.last.nu');
  const lined = await waitFor(`window.sandbox.line > 8`, 30000);
  const drawn = await ev(`document.querySelectorAll('#map .rline, #map .stall, #map .speed, #map .now').length`);
  check('throttle_back_slows_the_shaft_and_draws_the_running_line', slowed && nu12 < 0.95 && lined && drawn >= 8, `nu ${nu12}, line ${await ev('window.sandbox.line')}, drawn ${drawn}`);

  // 10. Below idle: the fly view's own plain words, then recovery.
  const idle = await ev(`window.sandbox.fly({Tt4: 400})`);
  const idleText = await ev(`[document.getElementById('bad-title').textContent, document.getElementById('bad-plain').textContent]`);
  check('below_idle_says_so_in_the_fly_views_words', idle === 'does-not-run' && /does not run here/.test(idleText[0]) && /idle|shaft speed|throttle|Raise Tt4/.test(idleText[1]), JSON.stringify([idle, idleText]));
  const back = await ev(`window.sandbox.fly({Tt4: 1300})`);
  check('the_fly_view_recovers_after_a_failure', back === 'ran' && !(await ev(`document.getElementById('bad').classList.contains('show')`)), back);

  // 11. In fly mode the flight knobs move the FLIGHT, never the frozen design.
  const designBefore = await ev('JSON.stringify(window.sandbox.last.design_point)');
  await ev(`(() => { const a = document.getElementById('altitude'); a.value = '11000'; a.dispatchEvent(new Event('input')); })()`);
  // (11 km on the design flight's day, ~2 K below standard: the pressure is not the standard 22 632 Pa,
  // so read the altitude the result reports, as step 6 does.)
  const climbed = await waitFor(`window.sandbox.last && window.sandbox.last.fly && Math.abs(window.sandbox.last.ambient.altitude - 11000) < 0.01 && window.sandbox.last.fly.p0 < 23000`);
  const designAfter = await ev('JSON.stringify(window.sandbox.last.design_point)');
  const where = await ev(`JSON.stringify([window.sandbox.state, window.sandbox.last.fly, document.getElementById('altitude').value, document.getElementById('bad-plain').textContent])`);
  check('fly_altitude_moves_the_flight_not_the_design', climbed && designBefore === designAfter, `${climbed} ${where}`);

  if (process.env.SANDBOX_SHOTS) {
    await shot('fly-light'); await shot('fly-dark', 1280, true); await shot('fly-phone', 390);
    // The states nobody had looked at: the nozzle unchoked (the stall tile's words) and a failure.
    await ev(`window.sandbox.fly({T0: 250, p0: 50000, Tt4: 450})`); await shot('fly-unchoked');
    await ev(`window.sandbox.fly({Tt4: 400})`); await shot('fly-failed');
    await ev(`window.sandbox.fly({Tt4: 1300})`);
  }

  // 11b. SLAM THE THROTTLE (slice 4 A). Back at the design flight and throttle, the page's opening slam
  // (temperature commanded) is the native `{"op":"slam","slam":{}}`; the fuel-metered run overshoots; both
  // paths reach the time chart and the map.
  await ev(`window.sandbox.fly({T0: 250, p0: 50000, M0: 0.85, Tt4: 1500})`);
  const sl = await ev(`window.sandbox.slam({from: 1100, ramp: 0.5, settle: 3, mode: 'both'}).then(r => r)`);
  const nativeSlam = JSON.parse(readFileSync(nativeTxt, 'utf8').split(/\r?\n/).find(l => l.startsWith('{"op":"slam","slam":{}}\t')).split('\t')[1]);
  const es = sl && sl.temperature ? close(sl.temperature, nativeSlam) : 'no slam result';
  const over = sl && sl.fuel && sl.fuel.ok ? Math.max(...sl.fuel.Tt4) - sl.fuel.end.Tt4 : NaN;
  const drawnS = await ev(`[document.querySelectorAll('#slam-chart .tcmd, #slam-chart .tfuel').length, document.querySelectorAll('#map .tcmd, #map .tfuel').length, document.getElementById('slam-table').tBodies[0].rows.length, document.getElementById('slam-card').hidden]`);
  check('the_slam_matches_native_and_the_fuel_run_overshoots', !es && over > 100 && drawnS[0] === 2 && drawnS[1] === 2 && drawnS[2] === 8 && drawnS[3] === false,
        (es || '') + ` overshoot ${over} K, drawn ${JSON.stringify(drawnS)}`);
  // A fuel slam that outruns the model's fuel range: the points it has, and why it stopped, in words.
  const st = await ev(`window.sandbox.slam({from: 1000, ramp: 0.1, settle: 3, mode: 'fuel'}).then(r => r)`);
  const stopText = await ev(`document.getElementById('slam-stops').textContent`);
  check('a_stopped_run_shows_its_points_and_says_why', st && st.fuel && st.fuel.ok === 1 && st.fuel.stop && st.fuel.stop.kind === 'fuel_cap'
        && st.fuel.s.length === 5 && /stopped at .* after 5 of \d+ points/.test(stopText) && /fuel arrived faster/.test(stopText) && /Model message/.test(stopText),
        JSON.stringify([st && st.fuel && st.fuel.stop, stopText.slice(0, 200)]));
  // A starting throttle below idle: the run traps in its steady solve, and the words name WHICH throttle.
  await ev(`window.sandbox.slam({from: 400, ramp: 0.5, settle: 1, mode: 'temperature'})`);
  const idleSlam = await ev(`document.getElementById('slam-stops').textContent`);
  check('a_slam_from_below_idle_names_the_starting_throttle', /does not run\. At the starting throttle \(400 K\)/.test(idleSlam), idleSlam.slice(0, 200));
  // A knob moved during a slow run stops it (its worker is killed), and the new run finishes.
  const k0 = await ev('window.sandbox.kills');
  await ev(`window.sandbox.slam({from: 1100, settle: 10, mode: 'both'}); null`);
  await waitFor(`window.sandbox.slamState === 'running'`, 5000);
  await ev(`(() => { const a = document.getElementById('slam-ramp'); a.value = '0.3'; a.dispatchEvent(new Event('input')); })()`);
  const fin = await waitFor(`window.sandbox.slamState === 'done' && window.sandbox.slamRuns().fuel && window.sandbox.slamRuns().fuel.slam && window.sandbox.slamRuns().fuel.slam.ramp === 0.3`, 60000);
  check('a_knob_move_stops_a_running_slam_and_the_new_one_finishes', fin && (await ev('window.sandbox.kills')) > k0, `finished ${fin}, kills ${k0} -> ${await ev('window.sandbox.kills')}`);
  await ev(`window.sandbox.slam({from: 1100, ramp: 0.5, settle: 3, mode: 'both'})`);
  if (process.env.SANDBOX_SHOTS) {
    await shot('slam-light'); await shot('slam-dark', 1280, true); await shot('slam-phone', 390);
    await ev(`document.getElementById('q-phi').click()`); await shot('slam-phi');
    await ev(`window.sandbox.slam({from: 1000, ramp: 0.1, settle: 3, mode: 'fuel'})`); await shot('slam-stopped');
    await ev(`document.getElementById('q-Tt4').click()`);
    await ev(`window.sandbox.slam({from: 1100, ramp: 0.5, settle: 3, mode: 'both'})`);
  }

  // 12. The slow gas runs when the knob is let go, not per step of a drag.
  await ev(`(() => { const g = document.getElementById('fly-gas'); g.value = 'equilibrium'; g.dispatchEvent(new Event('change')); })()`);
  const eq = await waitFor(`window.sandbox.last && window.sandbox.last.fly && window.sandbox.last.fly.gas === 'equilibrium' && window.sandbox.state === 'ran'`, 20000);
  const n0 = await ev('window.sandbox.last.fly.Tt4');
  await ev(`(() => { const r = document.getElementById('fly-Tt4-r'); r.value = '1250'; r.dispatchEvent(new Event('input')); })()`);
  await sleep(1500);
  const held = await ev('window.sandbox.last.fly.Tt4');
  await ev(`document.getElementById('fly-Tt4-r').dispatchEvent(new Event('change'))`);
  const released = await waitFor(`window.sandbox.last.fly.Tt4 === 1250 && window.sandbox.state === 'ran'`, 20000);
  check('the_slow_gas_runs_on_release', eq && held === n0 && released, `eq ${eq}, held ${held} (was ${n0}), released ${released}`);
  // ...and on the equilibrium gas the slam is refused, in words.
  const eqSlam = await waitFor(`window.sandbox.slamState !== 'running' && /does not run on the equilibrium gas/.test(document.getElementById('slam-stops').textContent)`, 20000);
  check('the_slam_is_refused_on_the_equilibrium_gas_in_words', eqSlam, await ev(`document.getElementById('slam-stops').textContent`));

  // 13. SIZE THE BLADES (slice 2). The opening sizing is the native `blade_size` of the defaults (rung
  // 85's default cell), and the lever sweep streams in on its own worker.
  const nativeOf = req => JSON.parse(readFileSync(nativeTxt, 'utf8').split(/\r?\n/).find(l => l.startsWith(req + '\t')).split('\t')[1]);
  await ev(`document.getElementById('mode-blades').click()`);
  const sized = await waitFor(`window.sandbox.view() === 'blades' && window.sandbox.state === 'ran' && window.sandbox.last && window.sandbox.last.lp !== undefined`);
  const b0 = await ev('window.sandbox.last');
  const eb = b0 ? close(b0, nativeOf('{"op":"blade_size","blades":{}}')) : 'no sizing';
  const tiles = await ev(`['v-klp','v-khp'].map(i => document.getElementById(i).textContent)`);
  check('blades_open_on_rung_85s_default_cell_and_match_native', sized && !eb && tiles.join() === '2,5', eb || JSON.stringify(tiles));
  const bvis = await ev(`['blades-set','bk-set','lever-set','bl-machine-card','bl-lever-card','flight-set','ts-card','out'].map(i => document.getElementById(i).hidden)`);
  check('blades_view_shows_its_cards_and_hides_the_rest', JSON.stringify(bvis) === '[false,false,false,false,false,true,true,true]', JSON.stringify(bvis));
  const swept = await waitFor(`window.sandbox.lever === 9`, 30000);
  const p0 = await ev('window.sandbox.points()[0]');
  // The native grid's design-point lever request (the defaults at 1500 K), found by its fields: JS and
  // Rust spell 1500.0 differently, so request text cannot be matched.
  const nativeLever = JSON.parse(readFileSync(nativeTxt, 'utf8').split(/\r?\n/).filter(Boolean).map(l => l.split('\t'))
    .find(([q]) => { const r = JSON.parse(q); return r.op === 'blade_lever' && r.Tt4 === 1500 && JSON.stringify(r.blades) === JSON.stringify(b0.blades); })?.[1] ?? 'null');
  const e13 = !p0 ? 'no point' : !nativeLever ? 'no native design-point lever line' : close(p0, nativeLever);
  const drawn13 = await ev(`[document.querySelectorAll('#bl-lever-chart .lp, #bl-lever-chart .hp, #bl-lever-chart .redline').length, document.getElementById('bl-lever-table').tBodies[0].rows.length, document.querySelectorAll('#bl-stair .steps, #bl-stair .rnow').length]`);
  check('the_lever_sweep_streams_and_its_design_point_matches_native', swept && !e13 && drawn13[0] >= 6 && drawn13[1] === 9 && drawn13[2] === 2, (e13 || '') + ' ' + JSON.stringify(drawn13));

  // 14. A design whose low throttles never reach the target or are not modelled: both shown in words.
  await ev(`window.sandbox.blades({pi_lpc: 4, pi_hpc: 10, Tt4: 1200})`);
  // The PREVIOUS sweep's count still reads 9 until this design's sweep starts (200 ms later): wait for
  // THIS design's grid, which starts at its own 1200 K.
  await waitFor(`window.sandbox.lever === 9 && window.sandbox.points()[0].Tt4 === 1200`, 30000);
  const rows = await ev(`[...document.getElementById('bl-lever-table').tBodies[0].rows].map(r => r.textContent)`);
  check('unreached_and_not_modelled_throttles_are_said_in_words', rows.length === 9 && rows.filter(r => /no \(last setting\)/.test(r)).length >= 1
        && rows.filter(r => /Not modelled: .*nozzle unchokes/.test(r)).length === 2, JSON.stringify(rows.slice(-3)));
  if (process.env.SANDBOX_SHOTS) { await shot('blades-light'); await shot('blades-dark', 1280, true); await shot('blades-phone', 390); }

  // 15. Blades that cannot be built: the panel in plain words, then recovery.
  const badB = await ev(`window.sandbox.blades({hp: {h: 1.0}})`);
  const badText = await ev(`[document.getElementById('bad-title').textContent, document.getElementById('bad-plain').textContent]`);
  check('blades_that_cannot_be_built_say_why', badB === 'does-not-run' && /cannot be built/.test(badText[0]) && /high-pressure spool's hub-to-tip/.test(badText[1]), JSON.stringify([badB, badText]));
  if (process.env.SANDBOX_SHOTS) await shot('blades-failed');
  await ev(`document.getElementById('reset').click()`);
  const rebuilt = await waitFor(`window.sandbox.state === 'ran' && window.sandbox.last.lp && window.sandbox.last.blades.pi_lpc === 3`);
  check('the_blades_view_recovers_on_reset', rebuilt && JSON.stringify(await ev('window.sandbox.last')) === JSON.stringify(b0), 'differs after reset');

  // 16. On the slow gas a knob move STOPS the running sweep (its worker is killed) and a new one finishes.
  const killsBefore = await ev('window.sandbox.kills');
  await ev(`window.sandbox.blades({gas: 'thermally_perfect'})`);
  const busy = await waitFor(`/computing throttle [2-9]/.test(document.getElementById('bl-progress').textContent)`, 30000);
  await ev(`(() => { const a = document.getElementById('bk-h'); a.value = '0.55'; a.dispatchEvent(new Event('input')); })()`);
  const done16 = await waitFor(`window.sandbox.lever === 9 && window.sandbox.last.blades.lp.h === 0.55 && window.sandbox.last.blades.hp.h === 0.55`, 120000);
  const kills = await ev('window.sandbox.kills');
  check('a_knob_move_stops_a_slow_sweep_and_the_new_one_finishes', busy && done16 && kills > killsBefore, `busy ${busy}, done ${done16}, kills ${killsBefore} -> ${kills}`);

  // 17. Back to design: the design result is the opening one exactly -- the fly and blade sessions changed no design knob.
  await ev(`document.getElementById('mode-design').click()`);
  // A DESIGN result: it has a thrust and no shaft speed (a blade result has neither, so `nu === undefined`
  // alone would pass on the blade view's last result before the design run arrives).
  const home = await waitFor(`window.sandbox.view() === 'design' && window.sandbox.state === 'ran' && window.sandbox.last.nu === undefined && window.sandbox.last.thrust !== undefined`, 20000);
  check('back_to_design_is_the_opening_design', home && JSON.stringify(await ev('window.sandbox.last')) === JSON.stringify(first), 'differs');

  // 18. Nothing threw on the page along the way.
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
