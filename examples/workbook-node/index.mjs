// Runnable Node.js 20 ESM example for @truecalc/workbook.
//
// Run against the local build:
//   WASM_PKG_PATH=./crates/wasm-workbook/pkg/truecalc_wasm_workbook.js node examples/workbook-node/index.mjs
//
// Run against the published package (after npm install):
//   node examples/workbook-node/index.mjs

const pkgPath = process.env.WASM_PKG_PATH || '@truecalc/workbook';
const { default: init, JsWorkbook } = await import(pkgPath);
await init();

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

console.log('All assertions passed.');
