use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
#[derive(Clone, Debug, Deserialize)]
pub struct EndpointDataMap {
    path: String,
    #[serde(default = "default_method")]
    method: String,
}

fn default_method() -> String {
    actix_web::http::Method::GET.to_string()
}

#[derive(Clone, Debug, Deserialize)]
pub struct DataMap(pub HashMap<String, EndpointDataMap>);

impl DataMap{
    pub fn get_alias_path_method_vec(&self) -> Vec<(String, String, String)> {
        self.0
            .iter()
            .map(|(alias, endpoint)| {
                (
                    alias.clone(),
                    endpoint.path.clone(),
                    endpoint.method.clone(),
                )
            })
            .collect()
    }
}
