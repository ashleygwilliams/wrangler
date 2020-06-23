use std::fmt;

use serde::{Deserialize, Serialize};

use crate::settings::binding::Binding;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConfigKvNamespace {
    pub binding: String,
    pub id: Option<String>,
    pub preview_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KvNamespace {
    pub id: String,
    pub binding: String,
}

impl fmt::Display for KvNamespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}, binding: {}", self.id, self.binding)
    }
}

impl KvNamespace {
    pub fn binding(&self) -> Binding {
        Binding::new_kv_namespace(self.binding.clone(), self.id.clone())
    }
}
