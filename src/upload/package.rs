use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{self, Deserialize};

#[derive(Debug, Deserialize)]
pub struct Package {
    #[serde(default)]
    main: PathBuf,
    #[serde(default)]
    module: PathBuf,
}
impl Package {
    pub fn main(&self, package_dir: &PathBuf) -> Result<PathBuf> {
        if self.main == PathBuf::from("") {
            anyhow::bail!(
                "The `main` key in your `package.json` file is required; please specify the entry point of your Worker.",
            )
        } else if !package_dir.join(&self.main).exists() {
            anyhow::bail!(
                "The entrypoint of your Worker ({}) could not be found.",
                self.main.display()
            )
        } else {
            Ok(self.main.clone())
        }
    }
    pub fn module(&self, package_dir: &PathBuf) -> Result<PathBuf> {
        if self.module == PathBuf::from("") {
            anyhow::bail!(
                "The `module` key in your `package.json` file is required when using the module script format; please specify the entry point of your Worker.",
            )
        } else if !package_dir.join(&self.module).exists() {
            anyhow::bail!(
                "The entrypoint of your Worker ({}) could not be found.",
                self.module.display()
            )
        } else {
            Ok(self.module.clone())
        }
    }
}

impl Package {
    pub fn new(package_dir: &PathBuf) -> Result<Package> {
        let manifest_path = package_dir.join("package.json");
        if !manifest_path.is_file() {
            anyhow::bail!(
                "Your JavaScript project is missing a `package.json` file; is `{}` the \
                 wrong directory?",
                package_dir.display()
            )
        }

        let package_json: String = fs::read_to_string(manifest_path.clone())?.parse()?;
        let package: Package = serde_json::from_str(&package_json).unwrap_or_else(|_| {
            panic!(
                "could not parse {}, may have invalid or missing `main` or `module` keys",
                manifest_path.display()
            )
        });

        Ok(package)
    }
}
