use actix_web::{web, HttpRequest, Responder};
use crate::verification_handler::verification::verification_handler;
use crate::webhook_config::WebhookConfig;

pub async fn verification_endpoint_handler(
    req: HttpRequest,
    payload: web::Payload,
    config: web::Data<WebhookConfig>,
) -> impl Responder {
    let verification_config = config.get_verification_config_owned(); //This clone
    verification_handler(
        req,
        payload,
        verification_config,  
    ).await
}

pub async fn data_endpoint_handler(
    req: HttpRequest,
    payload: web::Payload,
    alias: String,
    config: web::Data<WebhookConfig>,
) -> impl Responder {
    let verification_config = config.get_verification_config_owned(); //This clone
    verification_handler(
        req,
        payload,
        verification_config,
    ).await
}