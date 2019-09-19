mod krate;
pub mod package;
pub mod preview;
mod route;
mod upload_form;

pub use package::Package;

use crate::settings::target::kv_namespace::KvNamespace;
use route::Route;

use upload_form::build_script_upload_form;

use std::path::Path;

use crate::commands;
use crate::commands::kv;
use crate::commands::subdomain::Subdomain;
use crate::http;
use crate::settings::global_user::GlobalUser;

use crate::settings::target::Target;
use crate::terminal::{emoji, message};

pub fn publish(
    user: &GlobalUser,
    target: &mut Target,
    push_worker: bool,
    push_bucket: bool,
) -> Result<(), failure::Error> {
    log::info!("workers_dev = {}", target.workers_dev);

    validate_target(target)?;

    if let Some(site_config) = &target.site {
        let site_namespace = kv::namespace::site(target, user)?;

        target.add_kv_namespace(KvNamespace {
            binding: "__STATIC_CONTENT".to_string(),
            id: site_namespace.id,
            bucket: Some(site_config.bucket.to_owned()),
        });
    }

    if push_bucket {
        upload_buckets(target, user)?;
    }
    if push_worker {
        commands::build(&target)?;
        publish_script(&user, &target)?;
    }

    Ok(())
}

fn publish_script(user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}",
        target.account_id, target.name,
    );

    let client = http::auth_client(user);

    let script_upload_form = build_script_upload_form(target)?;

    let mut res = client
        .put(&worker_addr)
        .multipart(script_upload_form)
        .send()?;

    if !res.status().is_success() {
        failure::bail!(
            "Something went wrong! Status: {}, Details {}",
            res.status(),
            res.text()?
        )
    }

    let pattern = if !target.workers_dev {
        let route = Route::new(&target)?;
        Route::publish(&user, &target, &route)?;
        log::info!("publishing to route");
        route.pattern
    } else {
        log::info!("publishing to subdomain");
        publish_to_subdomain(target, user)?
    };

    log::info!("{}", &pattern);
    message::success(&format!(
        "Successfully published your script to {}",
        &pattern
    ));

    Ok(())
}

fn upload_buckets(target: &Target, user: &GlobalUser) -> Result<(), failure::Error> {
    for namespace in &target.kv_namespaces() {
        if let Some(bucket) = &namespace.bucket {
            let path = Path::new(&bucket);
            kv::bucket::sync(target, user.to_owned(), &namespace.id, path, false)?;
        }
    }

    Ok(())
}

fn build_subdomain_request() -> String {
    serde_json::json!({ "enabled": true }).to_string()
}

fn publish_to_subdomain(target: &Target, user: &GlobalUser) -> Result<String, failure::Error> {
    log::info!("checking that subdomain is registered");
    let subdomain = Subdomain::get(&target.account_id, user)?;

    let sd_worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/subdomain",
        target.account_id, target.name,
    );

    let client = http::auth_client(user);

    log::info!("Making public on subdomain...");
    let mut res = client
        .post(&sd_worker_addr)
        .header("Content-type", "application/json")
        .body(build_subdomain_request())
        .send()?;

    if !res.status().is_success() {
        failure::bail!(
            "Something went wrong! Status: {}, Details {}",
            res.status(),
            res.text()?
        )
    }
    Ok(format!("https://{}.{}.workers.dev", target.name, subdomain))
}

fn validate_target(target: &Target) -> Result<(), failure::Error> {
    let mut missing_fields = Vec::new();

    if target.account_id.is_empty() {
        missing_fields.push("account_id")
    };
    if target.name.is_empty() {
        missing_fields.push("name")
    };

    match &target.kv_namespaces {
        Some(kv_namespaces) => {
            for kv in kv_namespaces {
                if kv.binding.is_empty() {
                    missing_fields.push("kv-namespace binding")
                }

                if kv.id.is_empty() {
                    missing_fields.push("kv-namespace id")
                }
            }
        }
        None => {}
    }

    let destination = if !target.workers_dev {
        // check required fields for release
        if target
            .zone_id
            .as_ref()
            .unwrap_or(&"".to_string())
            .is_empty()
        {
            missing_fields.push("zone_id")
        };
        if target.route.as_ref().unwrap_or(&"".to_string()).is_empty() {
            missing_fields.push("route")
        };
        // zoned deploy destination
        "a route"
    } else {
        // zoneless deploy destination
        "your subdomain"
    };

    let (field_pluralization, is_are) = match missing_fields.len() {
        n if n >= 2 => ("fields", "are"),
        1 => ("field", "is"),
        _ => ("", ""),
    };

    if !missing_fields.is_empty() {
        failure::bail!(
            "{} Your wrangler.toml is missing the {} {:?} which {} required to publish to {}!",
            emoji::WARN,
            field_pluralization,
            missing_fields,
            is_are,
            destination
        );
    };

    Ok(())
}
