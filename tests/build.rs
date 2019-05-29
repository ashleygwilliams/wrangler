use assert_cmd::prelude::*;
use fs_extra::dir::{copy, CopyOptions};
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

const BUNDLE_OUT: &str = "./worker";

macro_rules! settings {
    ( $f:expr, $x:expr ) => {
        let file_path = fixture_path($f).join("wrangler.toml");
        let mut file = File::create(file_path).unwrap();
        let content = format!(
            r#"
            name = "test"
            zone_id = ""
            account_id = ""
            {}
        "#,
            $x
        );
        file.write_all(content.as_bytes()).unwrap();
    };
}

#[test]
fn it_builds_with_webpack_single_js() {
    let fixture = "webpack_simple_js";
    create_temporary_copy(fixture);

    settings! {fixture, r#"
        type = "Webpack"
    "#};

    build(fixture);
    assert!(fixture_out_path(fixture).join("script.js").exists());
    assert!(fixture_out_path(fixture).join("metadata.json").exists());
    cleanup(fixture);
}

fn cleanup(fixture: &str) {
    let path = fixture_path(fixture);
    assert!(path.exists());
    fs::remove_dir_all(path.clone()).unwrap();
}

fn build(fixture: &str) {
    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    println!("dir: {:?}", fixture_path(fixture));
    build.current_dir(fixture_path(fixture));
    build.arg("build").assert().success();
}

fn fixture_path(fixture: &str) -> PathBuf {
    let mut dest = env::temp_dir();
    dest.push(fixture);
    dest
}

fn fixture_out_path(fixture: &str) -> PathBuf {
    fixture_path(fixture).join(BUNDLE_OUT)
}

fn create_temporary_copy(fixture: &str) {
    let current_dir = env::current_dir().unwrap();
    let src = Path::new(&current_dir).join("tests").join(fixture);

    let dest = env::temp_dir();

    fs::create_dir_all(dest.clone()).unwrap();
    let mut options = CopyOptions::new();
    options.overwrite = true;
    copy(src, dest, &options).unwrap();
}
