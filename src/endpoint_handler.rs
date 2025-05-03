use actix_web::{web, HttpRequest, Responder, HttpResponse};
use crate::verification_handler::verification::verification_handler;
use crate::polling_handler::data_polling::{retrieve_data_with_polling, DataResponse};
use crate::webhook_config::WebhookConfig;
use crate::cache::{OrderedCache};
use crate::data_handler::data_receiver::data_receiver;
use log::{info, error};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use serde::{ Serialize, Deserialize};
use serde_json::Value as JsonValue;

pub async fn verification_endpoint_handler(
    req: HttpRequest,
    payload: web::Payload,
    config: web::Data<WebhookConfig>,
) -> impl Responder {
    let verification_config = config.get_verification_config_owned();

    match verification_handler(req, payload, verification_config).await {
        Ok(response) => response,
        Err(e) => {
            error!("Verification failed: {}", e);
            HttpResponse::BadRequest().body(format!("Verification failed: {}", e))
        }
    }
}

pub async fn data_endpoint_handler(
    _req: HttpRequest,
    payload: web::Payload,
    alias: String,
    _config: web::Data<WebhookConfig>,
    cache: web::Data<OrderedCache>,
) -> impl Responder {
    match data_receiver(payload, alias.clone(), cache).await {
        Ok((alias, key)) => {
            info!("Successfully stored data for alias: {} with key: {}", alias, key);
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            error!("Failed to process data for alias {}: {}", alias, e);
            HttpResponse::Ok().finish() // Always return 200
        }
    }
}


pub async fn data_retrieval_handler(
    alias: String,
    config: web::Data<WebhookConfig>,
    cache: web::Data<OrderedCache>,
) -> impl Responder {
    let polling_config = config.get_polling_config_owned();

    match retrieve_data_with_polling(&alias, &cache, polling_config).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            error!("Data retrieval error for alias {}: {}", alias, e);
            HttpResponse::InternalServerError().json(DataResponse {
                status: "error".to_string(),
                message: format!("Failed to retrieve data: {}", e),
                count: 0,
                data: Vec::new(),
            })
        }
    }
}
