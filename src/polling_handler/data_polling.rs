use std::time::Duration;
use serde::Serialize;
use tokio::time::{sleep, timeout};
use crate::cache::OrderedCache;
use crate::polling_handler::polling_config::PollingConfig;
use serde_json::Value as JsonValue;

#[derive(Serialize)]
pub struct DataResponse {
    pub success: bool,
    pub message: String,
    pub count: usize,
    pub data: Vec<serde_json::Value>,
}


pub async fn retrieve_data_with_polling(
    alias: &str,
    cache: &OrderedCache,
    polling_config: PollingConfig,
) -> Result<DataResponse, Box<dyn std::error::Error>> {
    // Check if alias exists
    if !cache.has_alias(alias) {
        return Ok(DataResponse {
            success: false,
            message: format!("Alias '{}' not found", alias),
            count: 0,
            data: Vec::new(),
        });
    }
    
    let max_polled_item = polling_config.get_max_polled_item();
    
    // Long polling with timeout
    match timeout(polling_config.get_timeout(), poll_for_data(cache, alias, &max_polled_item)).await {
        Ok(Ok(data_items)) if !data_items.is_empty() => {
            let values: Vec<JsonValue> = data_items.into_iter().map(|(_, v)| v).collect();
            Ok(DataResponse {
                success: true,
                message: format!("Retrieved {} items after polling", values.len()),
                count: values.len(),
                data: values,
            })
        }
        Ok(Ok(_)) | Ok(Err(_)) | Err(_) => {
            Ok(DataResponse {
                success: false,
                message: "No data available within timeout period".to_string(),
                count: 0,
                data: Vec::new(),
            })
        }
    }
}

async fn poll_for_data(
    cache: &OrderedCache,
    alias: &str,
    max_polled_items: &usize,
) -> Result<Vec<(String, JsonValue)>, Box<dyn std::error::Error>> {
    let poll_interval = Duration::from_millis(100);

    loop {
        match cache.remove_oldest(alias, *max_polled_items).await {
            Ok(data_items) if !data_items.is_empty() => {
                return Ok(data_items);
            }
            Ok(_) => {
                sleep(poll_interval).await;
            }
            Err(e) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e
                )));
            }
        }
    }
}

