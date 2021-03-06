use http::{HeaderValue, Response};
use hyper::{Body, Uri};
use url::Url;

pub(super) fn get_path_as_str(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

/// Rewrites redirects to host to be localhost
pub(super) fn rewrite_redirect(
    resp: &mut Response<Body>,
    upstream_host: &str,
    local_host: &str,
    https: bool,
) {
    if resp.status().is_redirection() {
        let headers = resp.headers_mut();
        if let Some(destination) = headers.get("Location") {
            if let Ok(destination) = destination.to_str() {
                if let Ok(destination_url) = Url::parse(destination) {
                    if let Some(destination_domain) = destination_url.domain() {
                        if destination_domain == upstream_host {
                            // Rewrite domain to localhost if redirect is to host
                            let mut rewritten_destination =
                                destination.replace(destination_domain, local_host);

                            // Change protocol to match local protocol
                            if &rewritten_destination[..5] == "https" && !https {
                                rewritten_destination.remove(4);
                            } else if &rewritten_destination[..5] != "https" && https {
                                rewritten_destination.insert(4, 's');
                            }

                            if let Ok(header_value) = HeaderValue::from_str(&rewritten_destination)
                            {
                                headers.insert("Location", header_value);
                            }
                        }
                    }
                }
            }
        }
    }
}
