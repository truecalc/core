// Runnable Node.js 20 ESM example for @truecalc/workbook.
//
// Run against the local build:
//   WASM_PKG_PATH=./crates/wasm-workbook/pkg/truecalc_wasm_workbook.js node examples/workbook-node/index.mjs
//
// Run against the published package (after npm install):
//   node examples/workbook-node/index.mjs

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

// When using a local build, resolve the path against CWD (not the file's dir)
// and pass wasm bytes directly to avoid fetch() failure on file:// URLs.
const wasmPkgJs = process.env.WASM_PKG_PATH
  ? resolve(process.cwd(), process.env.WASM_PKG_PATH)
  : null;
const pkgSpecifier = wasmPkgJs ? pathToFileURL(wasmPkgJs).href : '@truecalc/workbook';
const { default: init, JsWorkbook, translateFormula } = await import(pkgSpecifier);
await init(wasmPkgJs ? readFileSync(wasmPkgJs.replace(/\.js$/, '_bg.wasm')) : undefined);

// ── Example 1: basic formulas on a single sheet ──────────────────────────────

const wb = new JsWorkbook('sheets');
wb.addSheet('Sheet1');

wb.set('Sheet1', 'A1', '10');
wb.set('Sheet1', 'A2', '20');
wb.set('Sheet1', 'A3', '=SUM(A1:A2)');
wb.set('Sheet1', 'B1', '=A3 * 2');

wb.recalc(JSON.stringify({ timestamp_ms: 0, timezone: 'UTC', rng_seed: 0 }));

const a3 = wb.resolved('Sheet1', 'A3');
const b1 = wb.resolved('Sheet1', 'B1');

console.assert(a3.type === 'number' && a3.value === 30, `A3 expected 30, got ${JSON.stringify(a3)}`);
console.assert(b1.type === 'number' && b1.value === 60, `B1 expected 60, got ${JSON.stringify(b1)}`);
console.log('Sheet1 A3:', a3);  // { type: 'number', value: 30 }
console.log('Sheet1 B1:', b1);  // { type: 'number', value: 60 }

// ── Example 2: cross-sheet reference ─────────────────────────────────────────

wb.addSheet('Summary');
wb.set('Summary', 'A1', '=Sheet1!A3');

wb.recalc(JSON.stringify({ timestamp_ms: 0, timezone: 'UTC', rng_seed: 0 }));

const summaryA1 = wb.resolved('Summary', 'A1');
console.assert(
  summaryA1.type === 'number' && summaryA1.value === 30,
  `Summary!A1 expected 30, got ${JSON.stringify(summaryA1)}`
);
console.log('Summary A1 (cross-sheet):', summaryA1);  // { type: 'number', value: 30 }

// ── Example 3: text formulas ──────────────────────────────────────────────────

wb.set('Sheet1', 'C1', 'hello');
wb.set('Sheet1', 'C2', '=UPPER(C1)');

wb.recalc(JSON.stringify({ timestamp_ms: 0, timezone: 'UTC', rng_seed: 0 }));

const c2 = wb.resolved('Sheet1', 'C2');
console.assert(c2.type === 'text' && c2.value === 'HELLO', `C2 expected 'HELLO', got ${JSON.stringify(c2)}`);
console.log('Sheet1 C2 (UPPER):', c2);  // { type: 'text', value: 'HELLO' }

// ── Example 4: serialization roundtrip ───────────────────────────────────────

const json = wb.toJSON();
console.assert(typeof json === 'string' && json.length > 0, 'toJSON() should return a non-empty string');

const wb2 = JsWorkbook.fromJSON(json);
wb2.recalc(JSON.stringify({ timestamp_ms: 0, timezone: 'UTC', rng_seed: 0 }));

const a3After = wb2.resolved('Sheet1', 'A3');
const summaryAfter = wb2.resolved('Summary', 'A1');

console.assert(
  a3After.type === 'number' && a3After.value === 30,
  `roundtrip A3 expected 30, got ${JSON.stringify(a3After)}`
);
console.assert(
  summaryAfter.type === 'number' && summaryAfter.value === 30,
  `roundtrip Summary!A1 expected 30, got ${JSON.stringify(summaryAfter)}`
);
console.log('Roundtrip Sheet1 A3:', a3After);    // { type: 'number', value: 30 }
console.log('Roundtrip Summary A1:', summaryAfter);  // { type: 'number', value: 30 }

// ── Example 5: reference translation (fill / paste) ──────────────────────────

// Route fill/paste reference adjustment through the engine's own parser instead
// of a re-implemented tokenizer. `=A1+$A$2` filled down one row: the relative
// row shifts, the `$`-absolute reference stays put.
const translated = translateFormula('=A1+$A$2', 1, 0);
console.assert(
  translated.formula === '=A2+$A$2',
  `translateFormula expected '=A2+$A$2', got ${JSON.stringify(translated)}`
);
console.log('translateFormula =A1+$A$2 (down 1 row):', translated.formula);  // =A2+$A$2

// The rewritten text feeds straight back into the workbook.
wb.set('Sheet1', 'A4', '5');
wb.set('Sheet1', 'A5', '7');
wb.set('Sheet1', 'B5', translateFormula('=A4', 1, 0).formula);  // =A5
wb.recalc(JSON.stringify({ timestamp_ms: 0, timezone: 'UTC', rng_seed: 0 }));
const b5 = wb.resolved('Sheet1', 'B5');
console.assert(b5.type === 'number' && b5.value === 7, `B5 expected 7, got ${JSON.stringify(b5)}`);
console.log('Sheet1 B5 (translated =A4 -> =A5):', b5);  // { type: 'number', value: 7 }

// ── Example 6: date-typed cells via setDate (issue #721) ─────────────────────

// A host stores a serial *as a Date* so the engine keeps offset arithmetic
// rendering as a date. `resolved` returns a tagged JSON string, so parse it to
// read the type. These checks throw on regression (a real gate, not a smoke log).
function must(cond, msg) { if (!cond) throw new Error(`Example 6 failed: ${msg}`); }

wb.addSheet('Dates');
wb.setDate('Dates', 'A1', 46180);     // a date-typed serial (2026-06-07)
wb.setDate('Dates', 'A2', -1.5);      // a pre-1900 serial round-trips exactly
wb.set('Dates', 'B1', '=A1+1');       // date + number → date
wb.set('Dates', 'C1', '=A1-7');       // date − number → date
wb.set('Dates', 'D1', '=A1-A2');      // date − date → plain number of days
wb.recalc(JSON.stringify({ timestamp_ms: 0, timezone: 'UTC', rng_seed: 0 }));

const dA1 = JSON.parse(wb.resolved('Dates', 'A1'));
const dA2 = JSON.parse(wb.resolved('Dates', 'A2'));
const dB1 = JSON.parse(wb.resolved('Dates', 'B1'));
const dC1 = JSON.parse(wb.resolved('Dates', 'C1'));
const dD1 = JSON.parse(wb.resolved('Dates', 'D1'));

must(dA1.type === 'date' && dA1.value === 46180, `A1 date round-trip, got ${JSON.stringify(dA1)}`);
must(dA2.type === 'date' && dA2.value === -1.5, `A2 pre-1900 serial round-trip, got ${JSON.stringify(dA2)}`);
must(dB1.type === 'date' && dB1.value === 46181, `A1+1 stays a date, got ${JSON.stringify(dB1)}`);
must(dC1.type === 'date' && dC1.value === 46173, `A1-7 stays a date, got ${JSON.stringify(dC1)}`);
must(dD1.type === 'number' && dD1.value === 46181.5, `A1-A2 is a plain number, got ${JSON.stringify(dD1)}`);
console.log('setDate date-typed arithmetic:', { A1: dA1, B1: dB1, C1: dC1, D1: dD1 });

// ── Example 7: the wasm32-only size caps (issue #911 regression guard) ───────

// Nothing in this repo compiles crates/workbook/tests/ for wasm32 (a
// dev-dependency in that tree doesn't build for the target), so a
// #[cfg(target_arch = "wasm32")] test for cap *behaviour* never runs in
// CI — CI's wasm32 stage only runs `wasm-pack build` (build only). This
// example does run against the built wasm package in CI (see
// .github/workflows/ci.yml, "Run workbook-node example"), so it is the only
// thing that exercises `exceeds_cell_cap` / `exceeds_serialized_cap` against
// a real wasm32 build. Both caps, both directions, including the exact `>`
// boundary (a `>=` regression would only show up landing exactly on the cap).

function must7(cond, msg) { if (!cond) throw new Error(`Example 7 failed: ${msg}`); }
function mustThrow7(fn, msg) {
  try {
    fn();
  } catch {
    return;
  }
  throw new Error(`Example 7 failed: expected to throw — ${msg}`);
}

// -- cell-count cap (MAX_CELLS_PER_WORKBOOK = 1,000,000) --

const MAX_CELLS_PER_WORKBOOK = 1_000_000;
const capWb = new JsWorkbook('sheets');
capWb.addSheet('Cap');

const cellCapStart = Date.now();
for (let row = 1; row <= MAX_CELLS_PER_WORKBOOK; row++) {
  capWb.set('Cap', `A${row}`, '1'); // the millionth call is exactly at the cap and must succeed
}
console.log(
  `cell cap: ${MAX_CELLS_PER_WORKBOOK.toLocaleString()} set calls (up to and including the cap) took ${Date.now() - cellCapStart}ms`
);

mustThrow7(
  () => capWb.set('Cap', `A${MAX_CELLS_PER_WORKBOOK + 1}`, '1'),
  'the (cap + 1)th cell must be rejected'
);

// -- serialized-byte cap (MAX_SERIALIZED_BYTES = 100 MiB) --

// Build a document landing at *exactly* MAX_SERIALIZED_BYTES so both sides of
// the `>` boundary are exercised, not just "comfortably under" and
// "comfortably over". The per-cell JSON overhead is calibrated from two small
// measurements rather than hardcoded, so this doesn't silently drift if the
// canonical JSON encoding changes shape.
const MAX_SERIALIZED_BYTES = 100 * 1024 * 1024;
const MAX_TEXT_LEN = 50_000;
const FULL_TEXT = 'x'.repeat(MAX_TEXT_LEN);
const addrLen = (row) => 1 + String(row).length; // "A" + row digits

const bytesWb = new JsWorkbook('sheets');
bytesWb.addSheet('Bytes');
const baseBytes = Buffer.byteLength(bytesWb.toJSON(), 'utf8');
bytesWb.set('Bytes', 'A1', FULL_TEXT);
const oneCellBytes = Buffer.byteLength(bytesWb.toJSON(), 'utf8');
// Fixed per-cell wrapper overhead (braces/keys/quotes), excluding the address
// string itself and the inter-element comma -- both accounted for below.
const perCellOverhead = oneCellBytes - baseBytes - MAX_TEXT_LEN - addrLen(1);

function plannedBytes(fullCellCount, tailRow, tailLen) {
  let total = baseBytes;
  for (let row = 1; row <= fullCellCount; row++) {
    total += perCellOverhead + addrLen(row) + MAX_TEXT_LEN;
  }
  total += perCellOverhead + addrLen(tailRow) + tailLen;
  total += fullCellCount; // one comma per element after the first
  return total;
}

let fullCellCount = 1; // A1 is already set above
while (plannedBytes(fullCellCount, fullCellCount + 1, 1) < MAX_SERIALIZED_BYTES - MAX_TEXT_LEN) {
  fullCellCount++;
}
const tailRow = fullCellCount + 1;
const tailLen = 1 + (MAX_SERIALIZED_BYTES - plannedBytes(fullCellCount, tailRow, 1));

for (let row = 2; row <= fullCellCount; row++) {
  bytesWb.set('Bytes', `A${row}`, FULL_TEXT);
}
bytesWb.set('Bytes', `A${tailRow}`, 'y'.repeat(tailLen));

const byteCapStart = Date.now();
const atCapJson = bytesWb.toJSON(); // exactly at the cap: must not throw
console.log(
  `serialized-byte cap: building and serializing exactly ${MAX_SERIALIZED_BYTES.toLocaleString()} bytes took ${Date.now() - byteCapStart}ms`
);
must7(
  Buffer.byteLength(atCapJson, 'utf8') === MAX_SERIALIZED_BYTES,
  `calibration missed the cap exactly: got ${Buffer.byteLength(atCapJson, 'utf8')} bytes`
);

JsWorkbook.fromJSON(atCapJson); // round trip at the exact cap: must not throw

// One byte over, from the workbook side (to_json) -- must throw.
bytesWb.set('Bytes', `A${tailRow}`, 'y'.repeat(tailLen + 1));
mustThrow7(() => bytesWb.toJSON(), 'canonical JSON one byte over the cap must be rejected');

// One byte over, from the input side (from_json). String surgery on the
// at-cap document keeps it syntactically valid JSON one byte larger, so this
// tests the cap itself rather than JSON parsing.
const overCapJson = atCapJson.replace('y'.repeat(tailLen), 'y'.repeat(tailLen + 1));
mustThrow7(() => JsWorkbook.fromJSON(overCapJson), 'input one byte over the cap must be rejected');

console.log('All assertions passed.');
