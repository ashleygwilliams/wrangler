mod server_config;
#[cfg(not(target_os = "windows"))]
mod socket;
use server_config::ServerConfig;
mod headers;
use headers::{destructure_response, structure_request};

use chrono::prelude::*;

use hyper::client::{HttpConnector, ResponseFuture};
use hyper::header::{HeaderName, HeaderValue};

use hyper::http::uri::InvalidUri;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client as HyperClient, Request, Response, Server, Uri};

use hyper_tls::HttpsConnector;

use tokio::runtime::Runtime as TokioRuntime;

use uuid::Uuid;

use crate::commands;
use crate::commands::preview::upload;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

use crate::terminal::emoji;
#[cfg(target_os = "windows")]
use crate::terminal::message;

const PREVIEW_HOST: &str = "rawhttp.cloudflareworkers.com";

pub fn dev(
    target: Target,
    user: Option<GlobalUser>,
    host: Option<&str>,
    port: Option<&str>,
    ip: Option<&str>,
) -> Result<(), failure::Error> {
    commands::build(&target)?;
    let server_config = ServerConfig::new(host, ip, port)?;
    let session_id = get_session_id()?;
    let preview_id = get_preview_id(target, user, &server_config, &session_id)?;

    // create a new thread to listen for devtools messages
    // ...but not on windows: https://github.com/housleyjk/ws-rs/issues/306
    #[cfg(not(target_os = "windows"))]
    std::thread::spawn(move || socket::listen(session_id));
    #[cfg(target_os = "windows")]
    message::warn("console.log messages are currently not supported on Windows. See https://github.com/housleyjk/ws-rs/issues/306 for more information.\n");

    // spawn tokio runtime on the main thread to handle incoming HTTP requests
    let mut runtime = TokioRuntime::new()?;
    runtime.block_on(serve(server_config, preview_id))?;

    Ok(())
}

async fn serve(server_config: ServerConfig, preview_id: String) -> Result<(), failure::Error> {
    // set up https client to connect to the preview service
    let https = HttpsConnector::new();
    let client = HyperClient::builder().build::<_, Body>(https);

    let listening_address = server_config.listening_address.clone();
    // create a closure that hyper will use later to handle HTTP requests
    let make_service = make_service_fn(move |_| {
        let client = client.to_owned();
        let preview_id = preview_id.to_owned();
        let server_config = server_config.to_owned();
        async move {
            Ok::<_, failure::Error>(service_fn(move |req| {
                let client = client.to_owned();
                let preview_id = preview_id.to_owned();
                let server_config = server_config.to_owned();
                async move {
                    let resp = preview_request(req, client, preview_id, server_config).await?;
                    let (mut parts, body) = resp.into_parts();

                    destructure_response(&mut parts)?;
                    let resp = Response::from_parts(parts, body);
                    Ok::<_, failure::Error>(resp)
                }
            }))
        }
    });

    let server = Server::bind(&listening_address.address).serve(make_service);
    println!("{} Listening on http://{}", emoji::EAR, listening_address);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}

fn get_preview_url(path_string: &str) -> Result<Uri, InvalidUri> {
    format!("https://{}{}", PREVIEW_HOST, path_string).parse()
}

fn get_path_as_str(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

fn preview_request(
    req: Request<Body>,
    client: HyperClient<HttpsConnector<HttpConnector>>,
    preview_id: String,
    server_config: ServerConfig,
) -> ResponseFuture {
    let (mut parts, body) = req.into_parts();

    let path = get_path_as_str(&parts.uri);
    let method = parts.method.to_string();
    let now: DateTime<Local> = Local::now();
    let preview_id = &preview_id;

    structure_request(&mut parts);

    parts.headers.insert(
        HeaderName::from_static("host"),
        HeaderValue::from_static(PREVIEW_HOST),
    );

    parts.headers.insert(
        HeaderName::from_static("cf-ew-preview"),
        HeaderValue::from_str(preview_id).expect("Could not create header for preview id"),
    );

    parts.uri = get_preview_url(&path).expect("Could not get preview url");

    let req = Request::from_parts(parts, body);

    println!(
        "[{}] \"{} {}{} {:?}\"",
        now.format("%Y-%m-%d %H:%M:%S"),
        method,
        server_config.host,
        path,
        req.version()
    );
    client.request(req)
}

fn get_session_id() -> Result<String, failure::Error> {
    Ok(Uuid::new_v4().to_simple().to_string())
}

fn get_preview_id(
    mut target: Target,
    user: Option<GlobalUser>,
    server_config: &ServerConfig,
    session_id: &str,
) -> Result<String, failure::Error> {
    let verbose = true;
    let sites_preview = false;
    let script_id: String = upload(&mut target, user.as_ref(), sites_preview, verbose)?;
    Ok(format!(
        "{}{}{}{}",
        &script_id,
        session_id,
        server_config.host.is_https() as u8,
        server_config.host
    ))
}
