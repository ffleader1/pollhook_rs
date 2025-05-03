// extractors.rs
use crate::verification_config::{ChallengeConfig,TokenConfig};
use actix_web::{web, HttpRequest, Error};
use bytes::Bytes;
use serde_json::Value;
use std::collections::HashMap;
use log::error;

/// Generic function to extract a value from a request based on the location and path
pub fn extract_value(
    req: &HttpRequest,
    location: &str,
    locate_path: &str,
    body: &Option<Bytes>,
    value_type: &str
) -> Result<String, Error> {
    match location {
        "query" => {
            // Extract from query string
            let query_string = req.query_string();
            let query = web::Query::<HashMap<String, String>>::from_query(query_string)?;
            if let Some(value) = query.get(locate_path) {
                Ok(value.clone())
            } else {
                error!("{} not found in query parameter: {}", value_type, locate_path);
                Err(Error::from(actix_web::error::ErrorBadRequest(format!("{} not found in query", value_type))))
            }
        },
        "header" => {
            // Extract from headers
            if let Some(header_value) = req.headers().get(locate_path) {
                if let Ok(value) = header_value.to_str() {
                    Ok(value.to_string())
                } else {
                    error!("Invalid header value for: {}", locate_path);
                    Err(Error::from(actix_web::error::ErrorBadRequest("Invalid header value")))
                }
            } else {
                error!("{} not found in header: {}", value_type, locate_path);
                Err(Error::from(actix_web::error::ErrorBadRequest(format!("{} not found in header", value_type))))
            }
        },
        "path" => {
            // Extract from path segment
            if let Ok(index) = locate_path.parse::<usize>() {
                let path_segments: Vec<&str> = req.path().split('/').collect();
                if index < path_segments.len() {
                    Ok(path_segments[index].to_string())
                } else {
                    error!("Path segment index out of bounds: {}", index);
                    Err(Error::from(actix_web::error::ErrorBadRequest("Path segment index out of bounds")))
                }
            } else {
                error!("Invalid path segment index: {}", locate_path);
                Err(Error::from(actix_web::error::ErrorBadRequest("Invalid path segment index")))
            }
        },
        "body" => {
            // Extract from request body
            if let Some(body_bytes) = body {
                let body_str = String::from_utf8_lossy(body_bytes);
                if let Ok(json) = serde_json::from_str::<Value>(&body_str) {
                    let path_parts: Vec<&str> = locate_path.split("::").collect();
                    let mut current_value = &json;

                    for part in path_parts {
                        if let Some(next_value) = current_value.get(part) {
                            current_value = next_value;
                        } else {
                            error!("{} path not found in body: {}", value_type, locate_path);
                            return Err(Error::from(actix_web::error::ErrorBadRequest(format!("{} path not found in body", value_type))));
                        }
                    }

                    if let Some(value) = current_value.as_str() {
                        Ok(value.to_string())
                    } else {
                        error!("{} value in body is not a string", value_type);
                        Err(Error::from(actix_web::error::ErrorBadRequest(format!("{} value in body is not a string", value_type))))
                    }
                } else {
                    error!("Failed to parse body as JSON");
                    Err(Error::from(actix_web::error::ErrorBadRequest("Failed to parse body as JSON")))
                }
            } else {
                error!("Body expected but not provided");
                Err(Error::from(actix_web::error::ErrorBadRequest("Body expected but not provided")))
            }
        },
        _ => {
            error!("Unsupported {} location: {}", value_type, location);
            Err(Error::from(actix_web::error::ErrorBadRequest(format!("Unsupported {} location", value_type))))
        }
    }
}

// Re-export the specific functions with proper type imports
pub fn extract_token(
    req: &HttpRequest,
    token_config: &TokenConfig,
    body: &Option<Bytes>
) -> Result<String, Error> {
    extract_value(req, &token_config.get_in(), &token_config.get_locate(), body, "Token")
}

pub fn extract_challenge(
    req: &HttpRequest,
    challenge_config: &ChallengeConfig,
    body: &Option<Bytes>
) -> Result<String, Error> {
    extract_value(req, &challenge_config.get_in(), &challenge_config.get_locate(), body, "Challenge")
}

