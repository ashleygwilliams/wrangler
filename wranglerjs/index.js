const webpack = require("webpack");
const { join } = require("path");
const fs = require("fs");
const WasmMainTemplatePlugin = require("webpack/lib/wasm/WasmMainTemplatePlugin");

function error(msg) {
  console.error("Error: " + msg);
  process.exit(1);
  return new Error("error");
}

const rawArgs = process.argv.slice(2);
const args = rawArgs.reduce((obj, e) => {
  if (e.indexOf("--") === -1 && e.indexOf("=") === -1) {
    throw error("malformed arguments");
  }

  const [name, value] = e.split("=");
  const normalizedName = name.replace("--", "");
  obj[normalizedName] = value;
  return obj;
}, {});

let config;
if (args["no-webpack-config"] === "1") {
  config = { entry: args["use-entry"] };
} else {
  config = require(join(process.cwd(), args["webpack-config"]));
}

if (Array.isArray(config)) {
  throw error(
    "Multiple webpack configurations are not supported. You can specify a different path for your webpack configuration file in wrangler.toml with the `webpack_config` field\n" +
      "Please make sure that your webpack configuration exports an Object."
  );
}

const compiler = webpack(config);
const fullConfig = compiler.options;

function filterByExtension(ext) {
  return v => v.indexOf("." + ext) !== -1;
}

// Override the {FetchCompileWasmTemplatePlugin} and inject our new runtime.
const [
  fetchCompileWasmTemplatePlugin
] = compiler.hooks.thisCompilation.taps.filter(
  tap => tap.name === "FetchCompileWasmTemplatePlugin"
);
fetchCompileWasmTemplatePlugin.fn = function(compilation) {
  const mainTemplate = compilation.mainTemplate;
  const generateLoadBinaryCode = () => `
      // Fake fetch response
      Promise.resolve({
        arrayBuffer() { return Promise.resolve(${args["wasm-binding"]}); }
      });
    `;

  const plugin = new WasmMainTemplatePlugin({
    generateLoadBinaryCode,
    mangleImports: false,
    supportsStreaming: false
  });
  plugin.apply(mainTemplate);
};

let lastHash = "";
const compilerCallback = (err, stats) => {
  if (err) {
    throw err;
  }

  if (stats.hash != lastHash) {
    const assets = stats.compilation.assets;
    const jsonStats = stats.toJson();
    const bundle = {
      wasm: null,
      script: "",
      errors: jsonStats.errors
    };

    const wasmModuleAsset = Object.keys(assets).find(filterByExtension("wasm"));
    const jsAssets = Object.keys(assets).filter(filterByExtension("js"));
    const hasWasmModule = wasmModuleAsset !== undefined;

    bundle.script = jsAssets.reduce((acc, k) => {
      const asset = assets[k];
      return acc + asset.source();
    }, "");

    if (hasWasmModule === true) {
      bundle.wasm = Buffer.from(assets[wasmModuleAsset].source()).toString("base64");
    }

    fs.writeFileSync(args["output-file"], JSON.stringify(bundle));
  }
  lastHash = stats.hash;
};

if (args["watch"] === "1") {
  compiler.watch(fullConfig.watchOptions, compilerCallback);
} else {
  compiler.run(compilerCallback);
}
