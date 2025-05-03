use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web::http::header;
use bytes::{Bytes, BytesMut};
use log::{debug, error};
use futures::StreamExt;
use crate::verification_handler::{extractors};
use crate::verification_handler::verification_config::VerificationConfig;



// Handler for all webhook requests
pub async fn verification_handler(
    req: HttpRequest,
    payload: web::Payload,
    config: VerificationConfig,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    // Check verification path
    if !config.is_verification_path(req.path().to_string()) {
        return Err("Invalid verification path".into());
    }

    // Collect the payload
    let mut body = BytesMut::new();
    let mut payload_stream = payload;

    while let Some(chunk) = payload_stream.next().await {
        let chunk = chunk?;
        body.extend_from_slice(&chunk);
    }

    let body_bytes = Bytes::from(body);

    // Call our verification function with config
    verify_from_config(req, Some(body_bytes), &config)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn verify_from_config(
    req: HttpRequest,
    body: Option<Bytes>,
    config: &VerificationConfig
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


