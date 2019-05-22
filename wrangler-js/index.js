#!/usr/bin/env node

const webpack = require("webpack");
const { join } = require("path");

const rawArgs = process.argv.slice(2);
const args = rawArgs.reduce((obj, e) => {
  if (e.indexOf("--") === -1 && e.indexOf("=") === -1) {
    throw new Error("malformed arguments");
  }

  const [name, value] = e.split("=");
  const normalizedName = name.replace("--", '');
  obj[normalizedName] = value;
  return obj;
}, {});

let config;
if (args["no-webpack-config"] === "1") {
  config = { entry: args["use-entry"] };
} else {
  config = require(join(process.cwd(), "./webpack.config.js"));
}

let compilerOutput = "";
const oldConsoleLog = console.log;
console.log = (...msg) => (compilerOutput += msg.join(" "));

const compiler = webpack(config);

function filterByExtension(ext) {
  return v => v.indexOf("." + ext) !== -1;
}

function emitForWrangler(assets) {
  const bundle = {
    wasm: null,
    wasm_name: "",
    script: null,
    compiler_output: compilerOutput
  };

  const wasmModuleAsset = Object.keys(assets).find(filterByExtension("wasm"));
  const jsAssets = Object.keys(assets).filter(filterByExtension("js"));
  const hasWasmModule = wasmModuleAsset !== undefined;

  bundle.script = jsAssets.reduce((acc, k) => {
    const asset = assets[k];
    return acc + asset.source();
  }, "");

  if (hasWasmModule === true) {
    bundle.wasm = Buffer.from(assets[wasmModuleAsset].source()).toString();
    bundle.wasm_name = wasmModuleAsset;
  }

  console.log(JSON.stringify(bundle));
}

compiler.run((err, stats) => {
  if (err) {
    throw err;
  }

  console.log = oldConsoleLog;
  emitForWrangler(stats.compilation.assets);
  // console.log(stats.toString({ colors: true }));
});
