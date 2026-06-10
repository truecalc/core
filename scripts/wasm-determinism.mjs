/**
 * wasm-determinism.mjs
 *
 * Cross-runtime Rust/WASM byte-identity check (P5.2, issue #543).
 *
 * Loads each golden workbook JSON, round-trips it through the WASM
 * JsWorkbook (fromJSON -> toJSON), and verifies the canonical JSON
 * output is byte-identical to the golden file after JSON normalization.
 *
 * Requires Node 20+. Run after:
 *   wasm-pack build crates/wasm-workbook --target web
 */

import { readFileSync, readdirSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "..");
const goldenDir = resolve(repoRoot, "crates/workbook/tests/golden");
const wasmPkg = resolve(repoRoot, "crates/wasm-workbook/pkg/truecalc_wasm_workbook.js");

// Dynamic import so Node resolves the ESM module at runtime (after wasm-pack builds it).
let init, JsWorkbook;
try {
  ({ default: init, JsWorkbook } = await import(wasmPkg));
} catch (err) {
  console.error(`ERROR: Could not load WASM package at ${wasmPkg}`);
  console.error("       Run: wasm-pack build crates/wasm-workbook --target web");
  console.error(err.message);
  process.exit(1);
}

// Pass wasm bytes directly — avoids fetch() which fails for file:// URLs in Node.js.
const wasmBytes = readFileSync(
  resolve(repoRoot, "crates/wasm-workbook/pkg/truecalc_wasm_workbook_bg.wasm")
);
await init(wasmBytes);

// --------------------------------------------------------------------------
// Helper: normalize JSON to a canonical string (no whitespace differences).
// --------------------------------------------------------------------------
function normalize(jsonString) {
  return JSON.stringify(JSON.parse(jsonString));
}

let allPassed = true;

// --------------------------------------------------------------------------
// Part 1: fromJSON -> toJSON byte-identity for every golden file.
// --------------------------------------------------------------------------
const goldenFiles = readdirSync(goldenDir)
  .filter((f) => f.endsWith(".json"))
  .sort();

for (const filename of goldenFiles) {
  const filePath = resolve(goldenDir, filename);
  const original = readFileSync(filePath, "utf8");
  const normalizedOriginal = normalize(original);

  let workbook;
  try {
    workbook = JsWorkbook.fromJSON(original);
  } catch (err) {
    console.error(`FAIL [fromJSON]: ${filename}`);
    console.error(`  ${err.message}`);
    allPassed = false;
    continue;
  }

  let output;
  try {
    output = workbook.toJSON();
  } catch (err) {
    console.error(`FAIL [toJSON]: ${filename}`);
    console.error(`  ${err.message}`);
    allPassed = false;
    continue;
  }

  const normalizedOutput = normalize(output);

  if (normalizedOriginal !== normalizedOutput) {
    console.error(`FAIL [byte-identity]: ${filename}`);
    // Print a short diff summary: first key that diverges.
    const orig = JSON.parse(normalizedOriginal);
    const out = JSON.parse(normalizedOutput);
    console.error(`  original keys: ${Object.keys(orig).join(", ")}`);
    console.error(`  output   keys: ${Object.keys(out).join(", ")}`);
    console.error(`  original (first 200 chars): ${normalizedOriginal.slice(0, 200)}`);
    console.error(`  output   (first 200 chars): ${normalizedOutput.slice(0, 200)}`);
    allPassed = false;
  } else {
    console.log(`OK [byte-identity]: ${filename}`);
  }
}

// --------------------------------------------------------------------------
// Part 2: recalc roundtrip on worked_example.json.
//
// The golden file already contains computed cell values (it is the canonical
// output). We load it, call fromJSON, recalc with a fixed context, then
// verify every cell value in the "cells" section matches the golden.
// --------------------------------------------------------------------------
const workedExamplePath = resolve(goldenDir, "worked_example.json");
const workedExampleJson = readFileSync(workedExamplePath, "utf8");
const golden = JSON.parse(workedExampleJson);

let recalcPassed = true;
try {
  const wb = JsWorkbook.fromJSON(workedExampleJson);

  const fixedContext = JSON.stringify({
    timestamp_ms: 0,
    timezone: "UTC",
    rng_seed: 0,
  });

  wb.recalc(fixedContext);

  const recalcedJson = wb.toJSON();
  const recalced = JSON.parse(recalcedJson);

  // Compare cells for each sheet.
  for (const goldenSheet of golden.sheets) {
    const recalcedSheet = recalced.sheets.find((s) => s.name === goldenSheet.name);
    if (!recalcedSheet) {
      console.error(`FAIL [recalc]: sheet "${goldenSheet.name}" missing from recalc output`);
      recalcPassed = false;
      continue;
    }

    for (const [cellAddr, goldenCell] of Object.entries(goldenSheet.cells)) {
      const recalcedCell = recalcedSheet.cells[cellAddr];
      if (!recalcedCell) {
        console.error(`FAIL [recalc]: ${goldenSheet.name}!${cellAddr} missing from recalc output`);
        recalcPassed = false;
        continue;
      }

      const goldenValue = JSON.stringify(goldenCell.value);
      const recalcedValue = JSON.stringify(recalcedCell.value);
      if (goldenValue !== recalcedValue) {
        console.error(
          `FAIL [recalc]: ${goldenSheet.name}!${cellAddr} value mismatch`
        );
        console.error(`  golden:  ${goldenValue}`);
        console.error(`  recalced: ${recalcedValue}`);
        recalcPassed = false;
      }
    }
  }

  if (recalcPassed) {
    console.log("OK [recalc-roundtrip]: worked_example.json");
  }
} catch (err) {
  console.error("FAIL [recalc-roundtrip]: worked_example.json");
  console.error(`  ${err.message}`);
  recalcPassed = false;
}

if (!allPassed || !recalcPassed) {
  process.exit(1);
}
