import { execSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const cwd = import.meta.dirname;

// Build WASM before running tests
execSync("wasm-pack build --target web", { cwd, stdio: "inherit" });

// Dynamic import after build generates the package (resolved via tsconfig paths)
const { initSync } = await import("brewdio-wasm");
const wasmPath = resolve(cwd, "pkg/brewdio_wasm_bg.wasm");
const wasmBytes = readFileSync(wasmPath);
initSync({ module: wasmBytes });
