use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
#[derive(Clone, Debug, Deserialize)]
pub struct EndpointDataMap {
    path: String
}

pub type DataMap = HashMap<String, EndpointDataMap>;
