pub mod wranglerjs;

mod watch;
pub use watch::watch_and_build;

use crate::terminal::message;
use crate::settings::project::{Project, ProjectType};
use crate::{commands, install};

use std::path::PathBuf;
use std::process::Command;

pub fn build(project: &Project) -> Result<(), failure::Error> {
    let project_type = &project.project_type;
    match project_type {
        ProjectType::JavaScript => {
            message::info("JavaScript project found. Skipping unnecessary build!")
        }
        ProjectType::Rust => {
            let tool_name = "wasm-pack";
            let binary_path = install::install(tool_name, "rustwasm")?.binary(tool_name)?;
            let args = ["build", "--target", "no-modules"];

            let command = command(&args, &binary_path);
            let command_name = format!("{:?}", command);

            commands::run(command, &command_name)?;
        }
        ProjectType::Webpack => {
            wranglerjs::run_build(project)?;
        }
    }

    Ok(())
}


pub fn command(args: &[&str], binary_path: &PathBuf) -> Command {
    message::working("Compiling your project to WebAssembly...");

    let mut c = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C");
        c.arg(binary_path);
        c
    } else {
        Command::new(binary_path)
    };

    c.args(args);
    c
}
