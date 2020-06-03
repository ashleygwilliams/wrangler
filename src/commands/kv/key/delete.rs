use cloudflare::endpoints::workerskv::delete_key::DeleteKey;
use cloudflare::framework::apiclient::ApiClient;

use crate::commands::kv::{format_error, validate_target};
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::interactive;
use crate::terminal::message;

pub fn delete(
    target: &Target,
    user: &GlobalUser,
    id: &str,
    key: &str,
) -> Result<(), failure::Error> {
    validate_target(target)?;
    let client = http::cf_v4_client(user)?;

    match interactive::delete(&format!("Are you sure you want to delete key \"{}\"?", key)) {
        Ok(true) => (),
        Ok(false) => {
            message::info(&format!("Not deleting key \"{}\"", key));
            return Ok(());
        }
        Err(e) => failure::bail!(e),
    }

    let msg = format!("Deleting key \"{}\"", key);
    message::working(&msg);

    let response = client.request(&DeleteKey {
        account_identifier: &target.account_id,
        namespace_identifier: id,
        key, // this is url encoded within cloudflare-rs
    });

    match response {
        Ok(_) => message::success("Success"),
        Err(e) => print!("{}", format_error(e)),
    }

    Ok(())
}
