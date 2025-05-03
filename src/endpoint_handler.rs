use actix_web::{web, HttpRequest, Responder, HttpResponse, http::header};
use crate::verification_handler::verification::verification_handler;
use crate::polling_handler::data_polling::{retrieve_data_with_polling, DataResponse};
use crate::webhook_config::WebhookConfig;
use crate::cache::{OrderedCache};
use crate::data_handler::data_receiver::data_receiver;
use log::{info, error};
use serde_json::json;

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


pub async fn data_retrieval_handler_with_auth(
    req: HttpRequest,
    path: web::Path<String>,
    config: web::Data<WebhookConfig>,
    cache: web::Data<OrderedCache>,
    token: web::Data<String>,
) -> HttpResponse {
    // Check Authorization header
    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let provided_token = &auth[7..]; // Skip "Bearer " prefix
            if provided_token == token.get_ref() {
                let alias = path.into_inner();
                let polling_config = config.get_polling_config_owned();

                match retrieve_data_with_polling(&alias, &cache, polling_config).await {
                    Ok(response) => HttpResponse::Ok().json(response),
                    Err(e) => {
                        error!("Data retrieval error for alias {}: {}", alias, e);
                        HttpResponse::InternalServerError().json(DataResponse {
                            success: false,
                            message: format!("Failed to retrieve data: {}", e),
                            count: 0,
                            data: Vec::new(),
                        })
                    }
                }
            } else {
                HttpResponse::Unauthorized().json(json!({
                    "error": "Unauthorized",
                    "message": "Invalid token"
                }))
            }
        }
        _ => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": "Missing or invalid Authorization header. Use 'Bearer <token>' format."
        })),
    }
}

pub async fn health_check_handler() -> impl Responder {
    web::Json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": env!("CARGO_PKG_NAME"),
    }))
}