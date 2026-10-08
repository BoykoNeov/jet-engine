// The browser build, checked against the native model (docs/plans/sandbox-plan.md § 6).
//
// Loads the model OUT OF THE BUILT PAGE (what ships, not a side file), sends it every request the
// native CLI answered (`turbojet -- sandbox-native`: REQUEST<TAB>REPLY per line), and compares the
// replies: same shape, same text, same integers, and every number within its bar of the native one.
//
// The bars are MEASURED, not typed: the browser's maths library differs from the Windows one in
// the last place, and the gas-table solvers carry that out to their own tolerance. See BARS.
//
// Usage: node check.mjs PAGE NATIVE_TXT        Prints a cargo-style "test result:" line.
import { readFileSync } from 'node:fs';

// THE BARS -- measured 2026-10-06 on this grid (every gas x nozzle x 3 cycles x 2 altitudes + the
// defaults, 91 runs, 17563 numbers with the T-s curves, 14311 bit-identical), and tied to WHY they differ:
// - PERFECT gas: closed forms only, so only the maths library's last place moves. Worst seen
//   3.8e-16 relative. Bar 1e-13 relative.
// - every TABLE gas: temperatures come out of the safeguarded Newton (`gas::SOLVE_TOL` = 1e-11),
//   and the two builds stop at different iterates INSIDE that tolerance. Worst seen 2.35e-11
//   relative (about 2x SOLVE_TOL). Bar 1e-9 = 100x SOLVE_TOL.
// - ENTROPY is compared ABSOLUTELY: it is a difference from the ambient datum, so a small value
//   makes its relative change meaningless (one station read 1.25e-10 "relative" for a 1.1e-8
//   J/(kg K) shift). Worst seen 1.1e-8 (table gases), 4.6e-13 (perfect). Bars 1e-6 / 1e-10.
// RE-MEASURED 2026-10-07 with slice 3's fly requests added (111 requests, 22348 numbers, 16969
// bit-identical): perfect 6.8e-15, table 9.4e-11 relative -- both worst at the constant-flow stall
// margin, a pressure-ratio quotient minus 1, which magnifies the solvers' last digits. The bars
// stand: 15x and 10x headroom. (A margin near 0 would make even that quotient's relative change
// meaningless; no grid point sits within 10 % of its stall line.)
// RE-MEASURED 2026-10-07 with slice 2's blade requests added (136 requests, 33679 numbers, 27753
// bit-identical): perfect 9.2e-15 (a blade lever's tip Mach), table unchanged. The blade lever's
// target search stops on a 1e-12 residual (`StageStackCore::INC_TOL`), yet both builds landed on the
// same travel; the integer and yes/no outputs (stage counts, `reached`) compare exactly under these
// bars, and the zeros (design pre-swirl at lambda 0, travel at the design point) came out exactly 0
// in both. The bars stand.
// SLICE 4 (B)'s Controls requests get their OWN bar, measured 2026-10-08 (15 requests, 170 in all, 80082
// numbers): perfect gas, but every point comes out of iterated solves -- the fuel closure stops at 1e-12
// (`FuelTransientCore::CLOSE_TOL`), the limiters' set points at 1e-13 -- so, like the table gases, the two
// builds stop at different iterates inside those tolerances, and a march carries that along. Worst seen
// 2.13e-13 relative (the stator setting late in the hardest run: the incidence floor on the
// stator-scheduled flat-LP map). Bar 1e-11 = 10x the closure's tolerance, ~50x the worst. `settled_lp` /
// `settled_hp` are the gap between two nearly equal speeds, so they are compared ABSOLUTELY, as entropy
// is: worst seen 8.4e-15, bar 1e-11.
// Entropy lives in the station points (`.s`) and as the first element of each curve pair
// (`ts_burner.N.0`, `ts_reject.N.0`) -- where it can be exactly 0 (the cooling curve ends ON the
// ambient datum), so only an absolute bar means anything.
const ENTROPY = /\.s$|\.ts_(burner|reject)\.\d+\.0$|\.settled_(lp|hp)$/;
const BARS = {
  perfect: { rel: 1e-13, s_abs: 1e-10 },
  table: { rel: 1e-9, s_abs: 1e-6 },
  march: { rel: 1e-11, s_abs: 1e-11 },
};

const [page, nativeTxt] = process.argv.slice(2);
const html = readFileSync(page, 'utf8');
const m = html.match(/const WASM_B64 = "([A-Za-z0-9+/=]+)";/);
if (!m) { console.log('the page holds no browser build'); process.exit(1); }
const bytes = Buffer.from(m[1], 'base64');
const enc = new TextEncoder(), dec = new TextDecoder();
let ex = (await WebAssembly.instantiate(bytes, {})).instance.exports;
ex.init();
function call(text) {
  const b = enc.encode(text), p = ex.alloc(b.length);
  new Uint8Array(ex.memory.buffer, p, b.length).set(b);
  ex.call(p, b.length);
  ex.free(p, b.length);
  return dec.decode(new Uint8Array(ex.memory.buffer, ex.reply_ptr(), ex.reply_len()));
}

const worst = {};
let numbers = 0, identical = 0;
function compare(a, b, path, fails, kind) {
  if (typeof a === 'number' && typeof b === 'number') {
    numbers++;
    if (Object.is(a, b)) { identical++; return; }
    const isS = ENTROPY.test(path);
    const d = isS ? Math.abs(a - b) : Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b));
    const bar = isS ? BARS[kind].s_abs : BARS[kind].rel;
    const key = kind + (isS ? (kind === 'march' ? ' settled gap (abs)' : ' entropy (abs)') : ' relative');
    if (!worst[key] || d > worst[key].d) worst[key] = { d, where: path };
    if (!(d <= bar)) fails.push(`${path}: browser ${a} vs native ${b} (${isS ? 'abs' : 'rel'} ${d.toExponential(2)} > ${bar})`);
    return;
  }
  if (Array.isArray(a) !== Array.isArray(b) || typeof a !== typeof b || (a === null) !== (b === null)) {
    fails.push(`${path}: shape differs`); return;
  }
  if (a && typeof a === 'object') {
    const ka = Object.keys(a), kb = Object.keys(b);
    if (ka.join() !== kb.join()) { fails.push(`${path}: keys ${ka} vs ${kb}`); return; }
    for (const k of ka) compare(a[k], b[k], `${path}.${k}`, fails, kind);
    return;
  }
  if (a !== b) fails.push(`${path}: ${JSON.stringify(a)} vs ${JSON.stringify(b)}`);
}

const lines = readFileSync(nativeTxt, 'utf8').split(/\r?\n/).filter(Boolean);
let passed = 0, failed = 0;
for (const [i, line] of lines.entries()) {
  const [req, native] = line.split('\t');
  const fails = [];
  let browser;
  try { browser = JSON.parse(call(req)); }
  catch (e) { fails.push('the browser build trapped: ' + e); ex = (await WebAssembly.instantiate(bytes, {})).instance.exports; ex.init(); }
  const r = JSON.parse(req), gas = r.settings ? r.settings.gas : r.fly ? (r.fly.gas ?? 'thermally_perfect')
    : r.blades ? (r.blades.gas ?? 'perfect')
    : r.slam ? ((r.slam.fly && r.slam.fly.gas) ?? 'thermally_perfect')
    : undefined;
  const kind = r.op && r.op.startsWith('controls') ? 'march' : gas === 'perfect' ? 'perfect' : 'table';
  if (browser !== undefined) compare(browser, JSON.parse(native), `#${i}`, fails, kind);
  if (fails.length) { failed++; console.log(`test grid #${i} ... FAILED\n  ${fails.slice(0, 5).join('\n  ')}`); }
  else passed++;
}
if (!(lines.length > 50)) { console.log(`only ${lines.length} native lines -- the grid did not run`); failed++; }
console.log(`${numbers} numbers compared, ${identical} bit-identical. Worst differences:`);
for (const [k, w] of Object.entries(worst).sort()) console.log(`  ${k.padEnd(24)} ${w.d.toExponential(2)} at ${w.where}`);
console.log(`test result: ${failed ? 'FAILED' : 'ok'}. ${passed} passed; ${failed} failed; 0 ignored`);
process.exit(failed ? 1 : 0);
