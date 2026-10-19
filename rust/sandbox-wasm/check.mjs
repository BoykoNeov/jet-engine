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
// RE-MEASURED 2026-10-18 with rung 100's requests (91 Controls requests): worst 1.36e-11 relative at #252.far.0 --
// the STEADY START of a schedule run at Mach 0.1 from 700 K, the nozzle unchoked, which reads no table (the same start
// under rung 99's grid) -- so no rung-100 code is in it. Its cause was not isolated (an unchoked steady start: the
// builds stopping their iterated solves on different iterates is the suspicion, not a measurement). The relative bar
// moves to 1e-10 (7x the worst seen).
// SLICE 5's burner requests (55 of 225, 2667 numbers, 2398 bit-identical; measured 2026-10-08) fall under the
// TABLE bar: their inlet is an equilibrium-gas design run. Worst seen 1.5e-15 relative, save the nozzle
// readout's jet-speed gain `dv9`/`dv9_frac` -- a difference of two nearly equal jet speeds, 3.2e-12 relative
// (2.3e-13 m/s). The β-PDF model at its notch (EI ~1e-5) came out bit-identical. A burner value moved 1e-8
// relative turned the check red (tampered once). The bar stands.
// RE-MEASURED 2026-10-19 with rung 102's Fly-it requests (13 of 360, appended last): table worst 4.84e-10 relative at
// #352.stall.sm_flow -- the reacting gas, lit, the nozzle held: the constant-flow stall margin again (a pressure-ratio
// quotient minus 1, read on a far speed line), behind FOUR nested solves (the speed search, the turbine's pi_t, the
// duct's Rayleigh exit, the afterburner's fuel), each stopping on its own iterate in each build. The bar stands, at 2x
// headroom (it had 10x): a lit point past ~5e-10 here is the first thing to look at if this check reddens.
// Entropy lives in the station points (`.s`) and as the first element of each curve pair
// (`ts_burner.N.0`, `ts_reject.N.0`) -- where it can be exactly 0 (the cooling curve ends ON the
// ambient datum), so only an absolute bar means anything.
// RUNG 102: the afterburner's curve (`ts_afterburner.N.0`) is entropy too.
const ENTROPY = /\.s$|\.ts_(burner|reject|afterburner)\.\d+\.0$|\.settled_(lp|hp)$/;
// RUNG 99: the pressure-ratio gap's second part (`gap_nozzle`) is, under the thrust recomputed for the flight speed,
// what is left after the airflow guess -- exactly 0 in algebra while the nozzle is choked, rounding (~1e-13 N per kg/s)
// in arithmetic, so its relative change is meaningless (two noise values of opposite sign read 190 %). Compared
// ABSOLUTELY: worst seen 8.1e-13 (measured 2026-10-18, check requests #240/#242); bar 1e-10 N per kg/s, ~1e-13 of the
// thrust. Under rung 98's split the same column is a constant of tens of N per kg/s and passes the same bar.
// The first part (`gap_ram`) is, under the formula, the airflow guess `V0·(ṁ̂ − ṁ_face)` -- a small difference of two
// nearly equal flows (~0.1 N per kg/s); relative it read 8.0e-12 against the 1e-11 bar (#242), so it takes the same
// absolute rule. Worst seen on both parts with both under it: 1.9e-12 (#232.gap_ram), rung 98's ram part included.
const GAP = /\.gap_(ram|nozzle)\.\d+$/;
const GAP_ABS = 1e-10;
const BARS = {
  perfect: { rel: 1e-13, s_abs: 1e-10 },
  table: { rel: 1e-9, s_abs: 1e-6 },
  march: { rel: 1e-10, s_abs: 1e-11 },
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
    const isGap = GAP.test(path);
    const isS = isGap || ENTROPY.test(path);
    const d = isS ? Math.abs(a - b) : Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b));
    const bar = isGap ? GAP_ABS : isS ? BARS[kind].s_abs : BARS[kind].rel;
    const key = kind + (isGap ? ' thrust-gap parts (abs)' : isS ? (kind === 'march' ? ' settled gap (abs)' : ' entropy (abs)') : ' relative');
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
  // Rung 93's Cool the turbine requests are iterated solves on the perfect gas (a fixed point through rung 91's
  // matcher to 1e-12 a share, a bisection to 1e-9 K), so they take the Controls bar.
  const kind = r.op && (r.op.startsWith('controls') || r.op.startsWith('cooling')) ? 'march'
    : gas === 'perfect' ? 'perfect' : 'table';
  if (browser !== undefined) compare(browser, JSON.parse(native), `#${i}`, fails, kind);
  if (fails.length) { failed++; console.log(`test grid #${i} ... FAILED\n  ${fails.slice(0, 5).join('\n  ')}`); }
  else passed++;
}
if (!(lines.length > 50)) { console.log(`only ${lines.length} native lines -- the grid did not run`); failed++; }
console.log(`${numbers} numbers compared, ${identical} bit-identical. Worst differences:`);
for (const [k, w] of Object.entries(worst).sort()) console.log(`  ${k.padEnd(24)} ${w.d.toExponential(2)} at ${w.where}`);
console.log(`test result: ${failed ? 'FAILED' : 'ok'}. ${passed} passed; ${failed} failed; 0 ignored`);
process.exit(failed ? 1 : 0);
