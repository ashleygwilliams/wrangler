use crate::commands::publish::package::Package;
use log::info;
use serde::Deserialize;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

// This structure represents the communication between {wrangler-js} and
// {wrangler}. It is send back after {wrangler-js} completion.
// FIXME(sven): make this private
#[derive(Deserialize, Debug)]
pub struct WranglerjsOutput {
    wasm: Option<String>,
    script: String,
    // {wrangler-js} will send us the path to the {dist} directory that {Webpack}
    // used; it's tedious to remove a directory with content in JavaScript so
    // let's do it in Rust!
    dist_to_clean: Option<String>,
}

impl WranglerjsOutput {}

// Directory where we should write the {Bundle}. It represents the built
// artifact.
const BUNDLE_OUT: &str = "./worker";
pub struct Bundle {}

// We call a {Bundle} the output of a {Bundler}; representing what {Webpack}
// produces.
impl Bundle {
    pub fn new() -> Bundle {
        Bundle {}
    }

    pub fn write(&self, wranglerjs_output: WranglerjsOutput) -> Result<(), failure::Error> {
        let bundle_path = Path::new(BUNDLE_OUT);
        if !bundle_path.exists() {
            fs::create_dir(bundle_path)?;
        }

        let mut metadata_file = File::create(self.metadata_path())?;
        metadata_file.write_all(create_metadata(self).as_bytes())?;

        let mut script_file = File::create(self.script_path())?;
        let mut script = create_prologue();
        script += &wranglerjs_output.script;

        if let Some(wasm) = wranglerjs_output.wasm {
            let mut wasm_file = File::create(self.wasm_path())?;
            wasm_file.write_all(wasm.as_bytes())?;
        }

        script_file.write_all(script.as_bytes())?;

        // cleanup {Webpack} dist, if specified.
        if let Some(dist_to_clean) = wranglerjs_output.dist_to_clean {
            info!("Remove {}", dist_to_clean);
            fs::remove_dir_all(dist_to_clean).expect("could not clean Webpack dist.");
        }

        Ok(())
    }

    pub fn metadata_path(&self) -> String {
        Path::new(BUNDLE_OUT)
            .join("metadata.json".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn wasm_path(&self) -> String {
        Path::new(BUNDLE_OUT)
            .join("module.wasm".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn has_wasm(&self) -> bool {
        Path::new(&self.wasm_path()).exists()
    }

    pub fn has_webpack_config(&self) -> bool {
        Path::new("webpack.config.js").exists()
    }

    pub fn get_wasm_binding(&self) -> String {
        "wasmprogram".to_string()
    }

    pub fn script_path(&self) -> String {
        Path::new(BUNDLE_OUT)
            .join("script.js".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }
}

// Path to {wrangler-js}, which should be executable.
fn executable_path() -> PathBuf {
    Path::new(".")
        .join("node_modules")
        .join(".bin")
        .join("wrangler-js")
}

// Run the underlying {wrangler-js} executable.
//
// In Rust we create a virtual file, pass the pass to {wrangler-js}, run the
// executable and wait for completion. The file will receive the a serialized
// {WranglerjsOutput} struct.
// Note that the ability to pass a fd is platform-specific
pub fn run_build(
    wasm_pack_path: PathBuf,
    bundle: &Bundle,
) -> Result<WranglerjsOutput, failure::Error> {
    let mut command = Command::new("node");
    command.arg(executable_path());
    command.env("WASM_PACK_PATH", wasm_pack_path);

    // create temp file for special {wrangler-js} IPC.
    let mut temp_file = env::temp_dir();
    temp_file.push(".wranglerjs_output");
    File::create(temp_file.clone())?;

    command.arg(format!(
        "--output-file={}",
        temp_file.clone().to_str().unwrap().to_string()
    ));
    command.arg(format!("--wasm-binding={}", bundle.get_wasm_binding()));

    // if {webpack.config.js} is not present, we infer the entry based on the
    // {package.json} file and pass it to {wrangler-js}.
    // https://github.com/cloudflare/wrangler/issues/98
    if !bundle.has_webpack_config() {
        let package = Package::new("./")?;
        let current_dir = env::current_dir()?;
        let package_main = current_dir
            .join(package.main()?)
            .to_str()
            .unwrap()
            .to_string();
        command.arg("--no-webpack-config=1");
        command.arg(format!("--use-entry={}", package_main));
    }

    info!("Running {:?}", command);

    let status = command.status()?;
    let output = fs::read_to_string(temp_file.clone()).expect("could not retrieve ouput");
    fs::remove_file(temp_file)?;

    if status.success() {
        Ok(serde_json::from_str(&output).expect("could not parse wranglerjs output"))
    } else {
        failure::bail!("failed to execute `{:?}`: exited with {}", command, status)
    }
}

pub fn run_npm_install() -> Result<(), failure::Error> {
    for tool in &["node", "npm"] {
        env_dep_installed(tool)?;
    }

    let mut command = build_npm_command();

    command.arg("install");
    info!("Running {:?}", command);

    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        failure::bail!("failed to execute `{:?}`: exited with {}", command, status)
    }
}

fn env_dep_installed(tool: &str) -> Result<(), failure::Error> {
    if which::which(tool).is_err() {
        failure::bail!("You need to install {}", tool)
    }
    Ok(())
}

// check if {wrangler-js} is present are a known location.
pub fn is_installed() -> bool {
    executable_path().exists()
}

pub fn install() -> Result<(), failure::Error> {
    let mut command = build_npm_command();
    command.arg("install").arg("wrangler-js");
    info!("Running {:?}", command);

    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        failure::bail!("failed to execute `{:?}`: exited with {}", command, status)
    }
}

// We inject some code at the top-level of the Worker; called {prologue}.
// This aims to provide additional support, for instance providing {window}.
pub fn create_prologue() -> String {
    r#"
        const window = this;
    "#
    .to_string()
}

// This metadata describe the bindings on the Worker.
fn create_metadata(bundle: &Bundle) -> String {
    info!("create metadata; wasm={}", bundle.has_wasm());
    if bundle.has_wasm() {
        format!(
            r#"
                {{
                    "body_part": "script",
                    "binding": {{
                        "name": "{name}",
                        "type": "wasm_module",
                        "part": "{name}"
                    }}
                }}
            "#,
            name = bundle.get_wasm_binding(),
        )
        .to_string()
    } else {
        r#"
                {{
                    "body_part": "script"
                }}
            "#
        .to_string()
    }
}

/// build a Command for npm
///
/// Here's the deal: on Windows, `npm` isn't a binary, it's a shell script.
/// This means that we can't invoke it via `Command` directly on Windows,
/// we need to invoke `cmd /C npm`, to run it within the cmd environment.
fn build_npm_command() -> Command {
    #[cfg(not(windows))]
    let mut command = Command::new("npm");

    #[cfg(windows)]
    let mut command = Command::new("cmd");
    #[cfg(windows)]
    command.arg("/C");
    command.arg("npm");

    command
}

// FIXME(sven): doesn't work because they have a race for the BUNDLE_OUT,
// make it configurable
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_writes_the_bundle_metadata() {
//         let wranglerjs_output = WranglerjsOutput {
//             script: "".to_string(),
//             dist_to_clean: None,
//             wasm: None,
//         };
//         let bundle = Bundle::new();

//         bundle.write(wranglerjs_output).unwrap();
//         assert!(Path::new(&bundle.metadata_path()).exists());

//         cleanup(BUNDLE_OUT);
//     }

//     #[test]
//     fn it_writes_the_bundle_script() {
//         let wranglerjs_output = WranglerjsOutput {
//             script: "foo".to_string(),
//             dist_to_clean: None,
//             wasm: None,
//         };
//         let bundle = Bundle::new();

//         bundle.write(wranglerjs_output).unwrap();
//         assert!(Path::new(&bundle.script_path()).exists());
//         assert!(!Path::new(&bundle.wasm_path()).exists());

//         cleanup(BUNDLE_OUT);
//     }

//     #[test]
//     fn it_writes_the_bundle_wasm() {
//         let wranglerjs_output = WranglerjsOutput {
//             script: "".to_string(),
//             wasm: Some("abc".to_string()),
//             dist_to_clean: None,
//         };
//         let bundle = Bundle::new();

//         bundle.write(wranglerjs_output).unwrap();
//         assert!(Path::new(&bundle.wasm_path()).exists());
//         assert!(bundle.has_wasm());

//         cleanup(BUNDLE_OUT);
//     }

//     fn cleanup(name: &str) {
//         let current_dir = env::current_dir().unwrap();
//         let path = Path::new(&current_dir).join(name);
//         println!("p: {:?}", path);
//         fs::remove_dir_all(path).unwrap();
//     }
// }
