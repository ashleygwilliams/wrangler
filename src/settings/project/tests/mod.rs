use super::*;

use std::env;
use std::path::{Path, PathBuf};

#[test]
fn it_builds_from_config() {
    const NAME: &str = "default";
    let toml_path = toml_fixture_path(NAME);

    let project = get_project_config(Some(NAME), &toml_path).unwrap();

    assert!(project.kv_namespaces.is_none());
}

#[test]
fn it_builds_from_default_config() {
    const NAME: &str = "default";
    let toml_path = toml_fixture_path(NAME);

    let default = Project::get_default_environment("build", &toml_path).unwrap();
    assert!(!default.is_none());
    assert_eq!(default, Some("default".to_string()));

    let project = get_project_config(Some(NAME), &toml_path).unwrap();

    assert!(project.kv_namespaces.is_none());
}

#[test]
fn it_builds_from_environments_config() {
    const NAME: &str = "environments";
    let toml_path = toml_fixture_path(NAME);

    let project = get_project_config(Some(NAME), &toml_path).unwrap();

    assert!(project.kv_namespaces.is_none());
}

#[test]
fn it_builds_from_environments_config_with_kv() {
    const NAME: &str = "kv_namespaces";
    let toml_path = toml_fixture_path(NAME);

    let project = get_project_config(Some(NAME), &toml_path).unwrap();

    assert!(project.kv_namespaces.is_none());
}

#[test]
fn it_builds_from_legacy_config() {
    let toml_path = legacy_toml_fixture_path("default");

    let project = get_project_config(None, &toml_path).unwrap();

    assert!(project.kv_namespaces.is_none());
}

#[test]
fn it_builds_from_legacy_config_with_kv() {
    let toml_path = legacy_toml_fixture_path("kv_namespaces");

    let project = get_project_config(None, &toml_path).unwrap();

    let kv_1 = KvNamespace {
        id: "somecrazylongidentifierstring".to_string(),
        binding: "prodKV".to_string(),
    };
    let kv_2 = KvNamespace {
        id: "anotherwaytoolongidstring".to_string(),
        binding: "stagingKV".to_string(),
    };

    match project.kv_namespaces {
        Some(kv_namespaces) => {
            assert!(kv_namespaces.len() == 2);
            assert!(kv_namespaces.contains(&kv_1));
            assert!(kv_namespaces.contains(&kv_2));
        }
        None => assert!(false),
    }
}

fn base_fixture_path() -> PathBuf {
    let current_dir = env::current_dir().unwrap();

    Path::new(&current_dir)
        .join("src")
        .join("settings")
        .join("project")
        .join("tests")
        .join("tomls")
}

fn legacy_toml_fixture_path(fixture: &str) -> PathBuf {
    base_fixture_path().join("legacy").join(fixture)
}

fn toml_fixture_path(fixture: &str) -> PathBuf {
    base_fixture_path().join(fixture)
}
