import { initSync } from "brewdio-wasm";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const wasmPath = resolve(import.meta.dirname, "../brewdio-wasm/pkg/brewdio_wasm_bg.wasm");
const wasmBytes = readFileSync(wasmPath);
initSync({ module: wasmBytes });
