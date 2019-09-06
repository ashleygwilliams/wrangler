use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeys;
use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeysParams;
use cloudflare::endpoints::workerskv::Key;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::HttpApiClient;

use serde_json::value::Value as JsonValue;

use crate::settings::project::Project;

pub struct KeyList {
    keys_result: Option<Vec<Key>>,
    prefix: Option<String>,
    client: HttpApiClient,
    account_id: String,
    namespace_id: String,
    cursor: Option<String>,
}

impl KeyList {
    pub fn fetch(
        project: &Project,
        client: HttpApiClient,
        namespace_id: &str,
        prefix: Option<&str>,
    ) -> KeyList {
        KeyList {
            keys_result: None,
            prefix: prefix.map(str::to_string),
            client,
            account_id: project.account_id.to_owned(),
            namespace_id: namespace_id.to_string(),
            cursor: None,
        }
    }

    fn request_params(&self) -> ListNamespaceKeys {
        let params = ListNamespaceKeysParams {
            limit: None, // Defaults to 1000 (the maximum)
            cursor: self.cursor.to_owned(),
            prefix: self.prefix.to_owned(),
        };

        ListNamespaceKeys {
            account_identifier: &self.account_id,
            namespace_identifier: &self.namespace_id,
            params: params,
        }
    }

    fn get_batch(&mut self) -> Option<Result<Key, ApiFailure>> {
        let response = self.client.request(&self.request_params());

        let (mut result, error) = match response {
            // if we succeed, we need to store the cursor, and extract the keys
            Ok(success) => {
                self.cursor = extract_cursor(success.result_info.clone());
                log::info!("{:?}", self.cursor);
                (success.result, None)
            }
            Err(e) => (Vec::new(), Some(e)),
        };

        // if the API comes back with an error, we should pass it up to the caller
        if let Some(error) = error {
            Some(Err(error))
        // otherwise, we can return the first element of the returned list now,
        // and store the remainder for subsequent `next`s
        } else {
            let key = result.pop()?;
            self.keys_result = Some(result);

            Some(Ok(key))
        }
    }
}

impl Iterator for KeyList {
    type Item = Result<Key, ApiFailure>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.keys_result.to_owned() {
            // if this is None, we have not made a request yet
            None => self.get_batch(),
            // if this is Some, we can start doling out keys
            Some(mut keys) => {
                let key = keys.pop();
                self.keys_result = Some(keys);

                if let Some(k) = key {
                    Some(Ok(k))
                // if the key vec is empty, we should check for a cursor from the last request
                } else {
                    // if there's a cursor, we need to ask the API for more keys
                    if self.cursor.is_some() {
                        self.get_batch()
                    // if not, this is the end of the list
                    } else {
                        None
                    }
                }
            }
        }
    }
}

// Returns Some(cursor) if cursor is non-empty, otherwise returns None.
fn extract_cursor(result_info: Option<JsonValue>) -> Option<String> {
    let result_info = result_info.unwrap();
    let returned_cursor_value = &result_info["cursor"];
    let returned_cursor = returned_cursor_value.as_str().unwrap().to_string();
    if returned_cursor.is_empty() {
        None
    } else {
        Some(returned_cursor)
    }
}
