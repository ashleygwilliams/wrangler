#[macro_use]
extern crate lazy_static;

pub mod fixture;

use std::process::Command;
use std::str;

use assert_cmd::prelude::*;
use fixture::{Fixture, WranglerToml};

#[test]
fn it_builds_webpack() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    let wrangler_toml = WranglerToml::webpack_zoneless("test-build-webpack", true);
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_single_js() {
    let fixture = Fixture::new();
    fixture.create_file(
        "index.js",
        r#"
        addEventListener('fetch', event => {
            event.respondWith(handleRequest(event.request))
        })

        /**
        * Fetch and log a request
        * @param {Request} request
        */
        async function handleRequest(request) {
            return new Response('Hello worker!', { status: 200 })
        }
    "#,
    );
    fixture.create_default_package_json();

    let wrangler_toml = WranglerToml::webpack_zoneless("test-build-webpack-single-js", true);
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_function_config_js() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = () => ({ entry: "./index.js" });
    "#,
    );

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-function");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_promise_config_js() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = Promise.resolve({ entry: "./index.js" });
    "#,
    );

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-promise");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_function_promise_config_js() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = Promise.resolve({ entry: "./index.js" });
    "#,
    );

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-function-promise");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_specify_config() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.worker.js",
        r#"
        module.exports = { entry: "./index.js" };
    "#,
    );

    let wrangler_toml = WranglerToml::webpack_custom_config(
        "test-build-webpack-specify-config",
        "webpack.worker.js",
    );
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_single_js_missing_package_main() {
    let fixture = Fixture::new();
    fixture.create_empty_js();

    fixture.create_file(
        "package.json",
        r#"
        {
            "name": "webpack_single_js_missing_package_main"
        }
    "#,
    );

    let wrangler_toml =
        WranglerToml::webpack_zoneless("test-build-webpack-single-js-missing-package-main", true);
    fixture.create_wrangler_toml(wrangler_toml);

    build_fails_with(
        &fixture,
        "The `main` key in your `package.json` file is required",
    );
}

#[test]
fn it_fails_with_multiple_webpack_configs() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = [
            { entry: "./a.js" },
            { entry: "./b.js" }
        ]
    "#,
    );

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-multiple-webpack-configs");
    fixture.create_wrangler_toml(wrangler_toml);

    build_fails_with(&fixture, "Multiple webpack configurations are not supported. You can specify a different path for your webpack configuration file in wrangler.toml with the `webpack_config` field");
}

#[test]
fn it_builds_with_webpack_wast() {
    let fixture = Fixture::new();
    fixture.create_file(
        "package.json",
        r#"
        {
            "dependencies": {
                "wast-loader": "^1.8.5"
            }
        }
    "#,
    );

    fixture.create_file(
        "index.js",
        r#"
        (async function() {
            await import("./module.wast");
        })()
    "#,
    );

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = {
            entry: "./index.js",
            module: {
                rules: [
                    {
                        test: /\.wast$/,
                        loader: "wast-loader",
                        type: "webassembly/experimental"
                    }
                ]
            },
        }
    "#,
    );

    fixture.create_file("module.wast", "(module)");

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-wast");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js", "module.wasm"]);
}

#[test]
fn it_fails_with_webpack_target_node() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = {
            "entry": "./index.js",
            "target": "node"
        }
    "#,
    );

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-fails-webpack-target-node");
    fixture.create_wrangler_toml(wrangler_toml);

    build_fails_with(
        &fixture,
        "Building a Cloudflare Worker with target \"node\" is not supported",
    );
}

#[test]
fn it_fails_with_webpack_target_web() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = {
            "entry": "./index.js",
            "target": "web"
        }
    "#,
    );

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-fails-webpack-target-web");
    fixture.create_wrangler_toml(wrangler_toml);

    build_fails_with(
        &fixture,
        "Building a Cloudflare Worker with target \"web\" is not supported",
    );
}

#[test]
fn it_builds_with_webpack_target_webworker() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = {
            "entry": "./index.js",
            "target": "webworker"
        }
    "#,
    );

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-target-webworker");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

fn build_creates_assets(fixture: &Fixture, script_names: Vec<&str>) {
    let _lock = fixture.lock();
    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(fixture.get_path());
    build.arg("build").assert().success();
    for script_name in script_names {
        assert!(fixture.get_output_path().join(script_name).exists());
    }
}

fn build_fails_with(fixture: &Fixture, expected_message: &str) {
    let _lock = fixture.lock();
    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(fixture.get_path());
    build.arg("build");

    let output = build.output().expect("failed to execute process");
    assert!(!output.status.success());
    assert!(
        str::from_utf8(&output.stderr)
            .unwrap()
            .contains(expected_message),
        format!(
            "expected {:?} not found, given: {:?}",
            expected_message,
            str::from_utf8(&output.stderr)
        )
    );
}
