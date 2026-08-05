// Rewrite the package.json wasm-pack generated for @truecalc/core before publish.
//
// Two jobs: set the published name, and add the conditional export that gives
// Bun a build it can actually run.
//
// Why the Bun condition exists (core#815). The primary artifact is built
// `--target bundler`, whose output relies on WebAssembly ESM integration — the
// wasm is imported as a module and its exports are wired back into the JS glue.
// Deno 2 and Node support that, which is why they need no `init()` call and no
// permission flags. Bun does not: the glue's `__wbg_set_wasm` never receives a
// usable module, so the first string crossing dereferences `undefined` and
// throws `malloc is not a function`. Measured on Bun 1.3.7, and `bun build
// --target=bun` does not rescue it either — so this is not something callers
// can bundle their way out of.
//
// A `--target web` build works in Bun, at the cost of requiring `await init()`.
// So Bun resolves to `./bun/`, and every other runtime keeps the init-free
// build it already had. `@truecalc/workbook` needs none of this: npm already
// ships it `--target web`, which is why it works in Bun today.
//
// Usage: node scripts/npm-package-json.mjs <pkg-dir>

import fs from "node:fs";
import path from "node:path";

const pkgDir = process.argv[2];
if (!pkgDir) {
  console.error("usage: node scripts/npm-package-json.mjs <pkg-dir>");
  process.exit(1);
}

const pkgPath = path.join(pkgDir, "package.json");
const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));

pkg.name = "@truecalc/core";
pkg.license = "MIT";

// Condition order is load-bearing: Node and Deno do not match "bun", so they
// fall through to "default" and resolve exactly what they resolved before.
//
// "./*" preserves deep imports. Adding an `exports` field otherwise *removes*
// every subpath a consumer could previously reach, which would make this a
// breaking change rather than an additive one.
pkg.exports = {
  ".": {
    bun: {
      types: "./bun/truecalc_wasm.d.ts",
      default: "./bun/truecalc_wasm.js",
    },
    types: "./truecalc_wasm.d.ts",
    default: "./truecalc_wasm.js",
  },
  "./*": "./*",
};

// npm publishes only what `files` lists, so the Bun build must be named here or
// it is silently omitted from the tarball and the condition resolves to
// nothing at install time.
const bunFiles = "bun/";
if (Array.isArray(pkg.files) && !pkg.files.includes(bunFiles)) {
  pkg.files.push(bunFiles);
}

fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");

// Fail loudly rather than publishing a package whose Bun condition points at
// files that are not there.
const required = [
  "bun/truecalc_wasm.js",
  "bun/truecalc_wasm_bg.wasm",
  "bun/truecalc_wasm.d.ts",
];
const missing = required.filter((f) => !fs.existsSync(path.join(pkgDir, f)));
if (missing.length > 0) {
  console.error(`missing Bun build artifacts: ${missing.join(", ")}`);
  process.exit(1);
}

console.log(`${pkg.name}@${pkg.version}: exports written, Bun build present`);
