pub mod kv;

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, ClientBuilder, RedirectPolicy};
use std::time::Duration;

use crate::install;
use crate::settings::global_user::GlobalUser;

fn headers() -> HeaderMap {
    let version = if install::target::DEBUG {
        "dev"
    } else {
        env!("CARGO_PKG_VERSION")
    };
    let user_agent = format!("wrangler/{}", version);

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(&user_agent).unwrap());
    headers
}

fn builder() -> ClientBuilder {
    let builder = reqwest::Client::builder();
    builder
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
}

pub fn client() -> Client {
    builder()
        .default_headers(headers())
        .build()
        .expect("could not create http client")
}

pub fn auth_client(user: &GlobalUser) -> Client {
    let mut headers = headers();
    headers.insert("X-Auth-Key", HeaderValue::from_str(&user.api_key).unwrap());
    headers.insert("X-Auth-Email", HeaderValue::from_str(&user.email).unwrap());

    builder()
        .default_headers(headers)
        .redirect(RedirectPolicy::none())
        .build()
        .expect("could not create authenticated http client")
}
