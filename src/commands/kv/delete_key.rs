use cloudflare::endpoints::workerskv::delete_key::DeleteKey;
use cloudflare::framework::apiclient::ApiClient;

use crate::commands::kv;
use crate::terminal::message;

pub fn delete_key(id: &str, key: &str) -> Result<(), failure::Error> {
    let client = kv::api_client()?;
    let account_id = kv::account_id()?;

    match kv::interactive_delete(&format!("Are you sure you want to delete key \"{}\"?", key)) {
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
        account_identifier: &account_id,
        namespace_identifier: id,
        key: key, // this is url encoded within cloudflare-rs
    });

    match response {
        Ok(_success) => message::success("Success"),
        Err(e) => kv::print_error(e),
    }

    Ok(())
}
