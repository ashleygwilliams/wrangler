mod krate;
pub mod target;

use crate::terminal::emoji;

use binary_install::{Cache, Download};
use krate::Krate;
use log::info;
use which::which;

use std::env;
use std::path::Path;

use lazy_static::lazy_static;

lazy_static! {
    static ref CACHE: Cache = get_wrangler_cache().expect("creating binary dependency cache");
}

pub fn install(tool_name: &str, owner: &str) -> Result<Download, failure::Error> {
    if let Some(download) = tool_exists(tool_name) {
        return Ok(download);
    }

    let binaries = &[tool_name];
    let latest_version = get_latest_version(tool_name)?;
    let download = download_prebuilt(tool_name, owner, &latest_version, binaries);
    match download {
        Ok(download) => Ok(download),
        Err(e) => {
            failure::bail!("could not download pre-built `{}` ({}).", tool_name, e);
        }
    }
}

pub fn install_artifact(
    tool_name: &str,
    owner: &str,
    version: &str,
) -> Result<Download, failure::Error> {
    if let Some(download) = tool_exists(tool_name) {
        return Ok(download);
    }

    let download = download_prebuilt(tool_name, owner, version, &[]);
    match download {
        Ok(download) => Ok(download),
        Err(e) => {
            failure::bail!("could not download pre-built `{}` ({}).", tool_name, e);
        }
    }
}

fn tool_exists(tool_name: &str) -> Option<Download> {
    if let Ok(path) = which(tool_name) {
        log::debug!("found global {} binary at: {}", tool_name, path.display());
        let no_parent_msg = format!("{} There is no path parent", emoji::WARN);
        Some(Download::at(path.parent().expect(&no_parent_msg)))
    } else {
        None
    }
}

fn download_prebuilt(
    tool_name: &str,
    owner: &str,
    version: &str,
    binaries: &[&str],
) -> Result<Download, failure::Error> {
    let url = match prebuilt_url(tool_name, owner, version) {
        Some(url) => url,
        None => failure::bail!(format!(
            "no prebuilt {} binaries are available for this platform",
            tool_name
        )),
    };

    info!("prebuilt artifact {}", url);

    // no binaries are expected; downloading it as an artifact
    let res = if !binaries.is_empty() {
        CACHE.download(true, tool_name, binaries, &url)?
    } else {
        CACHE.download_artifact(tool_name, &url)?
    };

    match res {
        Some(download) => {
            println!("⬇️ Installing {}...", tool_name);
            Ok(download)
        }
        None => failure::bail!("{} is not installed!", tool_name),
    }
}

fn prebuilt_url(tool_name: &str, owner: &str, version: &str) -> Option<String> {
    if tool_name == "wranglerjs" {
        Some(format!(
            "https://github.com/cloudflare/wrangler/releases/download/v{1}/{0}-v{1}.tar.gz",
            tool_name, version
        ))
    } else {
        let target = if target::LINUX && target::x86_64 {
            "x86_64-unknown-linux-musl"
        } else if target::MACOS && target::x86_64 {
            "x86_64-apple-darwin"
        } else if target::WINDOWS && target::x86_64 {
            "x86_64-pc-windows-msvc"
        } else {
            return None;
        };

        let url = format!(
            "https://github.com/{0}/{1}/releases/download/v{2}/{1}-v{2}-{3}.tar.gz",
            owner, tool_name, version, target
        );
        Some(url)
    }
}

fn get_latest_version(tool_name: &str) -> Result<String, failure::Error> {
    Ok(Krate::new(tool_name)?.max_version)
}

fn get_wrangler_cache() -> Result<Cache, failure::Error> {
    if let Ok(path) = env::var("WRANGLER_CACHE") {
        Ok(Cache::at(Path::new(&path)))
    } else {
        Cache::new("wrangler")
    }
}
