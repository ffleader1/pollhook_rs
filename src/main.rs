mod verification_handler;
mod webhook_config;
mod data_handler;

use verification_handler::{extractors, verification_config};
use webhook_config::WebhookConfig;

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder, http::header, Error, guard};
use serde::{Deserialize, Serialize, Deserializer};
use serde_yaml;
use std::{env, collections::HashMap, path::Path};
use futures::StreamExt;
use std::fs;
use bytes::{Bytes, BytesMut};
use log::{info, error, debug};
use dotenv::dotenv;
use env_logger;



fn read_config(file_path: &str) -> Result<WebhookConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(file_path)?;
    let config: WebhookConfig = serde_yaml::from_str(&config_str)?;
    Ok(config)
}

// Handler for all webhook requests
async fn verification_handler(
    req: HttpRequest,
    payload: web::Payload,
    config: web::Data<WebhookConfig>,
) -> impl Responder {
    // Collect the payload if body extraction is needed
    let mut body = BytesMut::new();
    let mut payload_stream = payload;
    
    if !config.is_verification_path(req.path().to_string()) {
        error!("Error bad verification path");
        return HttpResponse::BadRequest().finish();
    }

    while let Some(chunk) = payload_stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(e) => {
                error!("Error reading payload: {}", e);
                return HttpResponse::BadRequest().finish();
            }
        };
        body.extend_from_slice(&chunk);
    }

    let body_bytes = Bytes::from(body);

    // Call our verification function with config file path
    match verify_from_config(req, Some(body_bytes), config.get_verification_config()).await {
        Ok(response) => response,
        Err(e) => {
            error!("Verification failed: {}", e);
            HttpResponse::BadRequest().body(format!("Verification failed: {}", e))
        }
    }
}

async fn verify_from_config(
    req: HttpRequest,
    body: Option<Bytes>,
    config: &verification_config::VerificationConfig
) -> Result<HttpResponse, Error> {

    let path = req.path().to_string();
    let method = req.method().to_string();

    debug!("Processing request: {} {}", method, path);
    
    let request_token = extractors::extract_token(&req, config.get_token_config(), &body)?;
    
    if !config.is_token_valid(request_token) {
        error!("Token verification failed");
        return Ok(HttpResponse::Forbidden().finish());
    }

    // Extract challenge from request based on config
    let challenge = extractors::extract_challenge(&req, config.get_challenge_config(), &body)?;
    
    let response_config = config.get_response_config();

    // Prepare response based on config
    let response_data = if let Some(in_path) = response_config.get_in_path() {
        // Handle the case where data needs to be inserted at a specific path
        let path_parts: Vec<&str> = in_path.split("::").collect();
        let mut json_value = serde_json::Value::Object(serde_json::Map::new());

        // Create nested structure
        let mut current = &mut json_value;
        for (i, part) in path_parts.iter().enumerate() {
            if i < path_parts.len() - 1 {
                // Create nested objects for path
                if !current.as_object().unwrap().contains_key(*part) {
                    current.as_object_mut().unwrap().insert(
                        part.to_string(),
                        serde_json::Value::Object(serde_json::Map::new()),
                    );
                }
                current = current.as_object_mut().unwrap().get_mut(*part).unwrap();
            } else {
                // Insert the challenge at the final path
                let value = response_config.get_data().replace("@challenge", &challenge);
                current.as_object_mut().unwrap().insert(
                    part.to_string(),
                    serde_json::Value::String(value),
                );
            }
        }

        serde_json::to_string(&json_value).unwrap_or_else(|_| {
            error!("Failed to serialize JSON response");
            "{}".to_string()
        })
    } else {
        // Simple string replacement
        response_config.get_data().replace("@challenge", &challenge)
    };

    // Return appropriate response with proper content type
    let mut response_builder = HttpResponse::Ok();
    response_builder.insert_header((header::CONTENT_TYPE, response_config.get_content_type().as_str()));
    
    Ok(response_builder.body(response_data))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));


    // Get configuration file path from environment variable or use default
    let config_path = env::var("CONFIG_FILE_PATH").unwrap_or_else(|_| "config_webhook.yaml".to_string());


    // Check if the config file exists
    if !Path::new(&config_path).exists() {
        // Throw an error instead of creating a default file
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Config file not found: {}", config_path)
        ).into());
    }

    let mut config = read_config(&config_path).expect("Failed to read config file");

    let verification_token = env::var("VERIFY_TOKEN").expect("VERIFY_TOKEN is not set");

    config.set_token(verification_token);

    // Get port from environment or use default
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().unwrap();

    info!("Starting webhook verification server on port {}", port);
    info!("Using config file: {}", config_path);


    let method = actix_web::http::Method::try_from(config.get_verification_method().as_str()).unwrap_or(actix_web::http::Method::GET);

    HttpServer::new(move || {
        let method = method.clone();
        App::new()
            .app_data(web::Data::new(config.clone()))
            .route(
                "/verification/{path:.*}",
                web::route()
                    .guard(guard::fn_guard(move |ctx| ctx.head().method == method))
                    .to(verification_handler),
            )
    })
        .bind(("0.0.0.0", port))?
        .run()
        .await
}