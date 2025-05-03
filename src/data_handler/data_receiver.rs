use crate::cache::OrderedCache;
use crate::data_handler::data_config::DataMap;

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use futures::StreamExt;
use sha2::{Sha256, Digest};

use log::{error, info};

pub async fn data_receiver(
    mut payload: web::Payload,
    alias: String,
    cache: web::Data<OrderedCache>,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Collect the payload bytes
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        body.extend_from_slice(&chunk);
    }

    // Parse and validate it's valid JSON
    let json_value: serde_json::Value = serde_json::from_slice(&body)?;

    // Create hash of the content for the key
    let mut hasher = Sha256::new();
    hasher.update(&body);
    let hash = hasher.finalize();
    let key = hex::encode(hash);

    // Store the JSON value in cache
    cache.insert(&alias, key.clone(), json_value).await
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok((alias, key))
}