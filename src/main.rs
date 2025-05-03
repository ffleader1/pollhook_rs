mod verification_handler;
mod webhook_config;
mod data_handler;
mod endpoint_handler;
mod cache;
mod polling_handler;

use verification_handler::verification_config;
use webhook_config::WebhookConfig;

use actix_web::{web, App, HttpServer, guard};
use serde_yaml;
use std::{env, path::Path};
use cache::OrderedCache;
use std::fs;
use log::info;
use dotenv::dotenv;
use env_logger;
use rustls::ServerConfig as RustlsServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::io::BufReader;

pub const CALLBACK_PATH: &str = "callhook";
pub const POLLING_PATH: &str = "pollhook";

fn read_config(file_path: &str) -> Result<WebhookConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(file_path)?;
    let config: WebhookConfig = serde_yaml::from_str(&config_str)?;
    Ok(config)
}

fn load_rustls_config() -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
    let cert_file = &mut BufReader::new(fs::File::open("cert.pem")?);
    let key_file = &mut BufReader::new(fs::File::open("key.pem")?);

    let cert_chain = certs(cert_file)?
        .into_iter()
        .map(Certificate)
        .collect::<Vec<_>>();

    let mut keys: Vec<_> = pkcs8_private_keys(key_file)?
        .into_iter()
        .collect();

    if keys.is_empty() {
        return Err("No PKCS8-encoded private keys found in key.pem".into());
    }

    let key = PrivateKey(keys.remove(0));

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)?;

    Ok(config)
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
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Config file not found: {}", config_path)
        ));
    }

    let mut config = read_config(&config_path).expect("Failed to read config file");

    let verification_token = env::var("VERIFY_TOKEN").expect("VERIFY_TOKEN is not set");
    let data_retrieve_token = env::var("DATA_RETRIEVE_TOKEN").expect("DATA_RETRIEVE_TOKEN is not set");

    config.set_token(verification_token);
    config.init_polling_config();

    // Get port from environment or use default
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().unwrap();

    info!("Starting webhook verification server on port {}", port);
    info!("Using config file: {}", config_path);

    let method = actix_web::http::Method::try_from(config.get_verification_config().get_verification_method().as_str())
        .unwrap_or(actix_web::http::Method::GET);

    let data_routes = config.get_data_config().get_alias_path_method_vec();
    let ordered_cache = OrderedCache::new(data_routes.iter().map(|t| t.0.clone()).collect());

    // Check if HTTPS should be used
    let use_https =
        (Path::new("cert.pem").exists() && Path::new("key.pem").exists()) ||
            (env::var("SSL_CERT_FILE").is_ok() && env::var("SSL_KEY_FILE").is_ok());

    let server = HttpServer::new(move || {
        let method = method.clone();

        let mut app = App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(ordered_cache.clone()))
            .app_data(web::Data::new(data_retrieve_token.clone()))
            .route(
                &format!("/{}/{{path:.*}}", CALLBACK_PATH),
                web::route()
                    .guard(guard::fn_guard(move |ctx| ctx.head().method == method))
                    .to(endpoint_handler::verification_endpoint_handler),
            )
            // Health check route
            .route("/health", web::get().to(endpoint_handler::health_check_handler));

        // Add routes for each data endpoint (for receiving data)
        for (alias, path, method_str) in &data_routes {
            let route_path = if path.starts_with('/') {
                path.clone()
            } else {
                format!("/{}", path)
            };

            let alias_clone = alias.clone();
            let method = actix_web::http::Method::try_from(method_str.as_str())
                .unwrap_or(actix_web::http::Method::GET);

            app = app.route(
                &route_path,
                web::route()
                    .guard(guard::fn_guard(move |ctx| ctx.head().method == method))
                    .to(move |req, payload, config, cache| {
                        endpoint_handler::data_endpoint_handler(req, payload, alias_clone.clone(), config, cache)
                    }),
            );
        }

        // Add data retrieval route with authentication
        app = app.route(
            &format!("/{}/{{alias}}", POLLING_PATH),
            web::get().to(endpoint_handler::data_retrieval_handler_with_auth),
        );

        app
    });

    if use_https {
        let rustls_config = if let (Ok(cert_file), Ok(key_file)) =
            (env::var("SSL_CERT_FILE"), env::var("SSL_KEY_FILE")) {
            // Load from environment variables
            let cert_file = &mut BufReader::new(fs::File::open(cert_file)?);
            let key_file = &mut BufReader::new(fs::File::open(key_file)?);

            let cert_chain = certs(cert_file)?
                .into_iter()
                .map(Certificate)
                .collect::<Vec<_>>();

            let mut keys: Vec<_> = pkcs8_private_keys(key_file)?
                .into_iter()
                .collect();

            if keys.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "No PKCS8-encoded private keys found"
                ));
            }

            let key = PrivateKey(keys.remove(0));

            ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(cert_chain, key)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        } else {
            // Load from default files
            load_rustls_config()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        };

        info!("Starting server with HTTPS on port {}", port);
        server
            .bind_rustls(("0.0.0.0", port), rustls_config)?
            .run()
            .await
    } else {
        info!("Starting server with HTTP on port {}", port);
        server
            .bind(("0.0.0.0", port))?
            .run()
            .await
    }
}