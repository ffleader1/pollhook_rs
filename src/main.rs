mod verification_handler;
mod webhook_config;
mod data_handler;
mod endpoint_handler;

use verification_handler::{ verification_config};
use webhook_config::WebhookConfig;

use actix_web::{web, App, HttpServer, guard};
use serde_yaml;
use std::{env,  path::Path};
use std::fs;
use log::{info};
use dotenv::dotenv;
use env_logger;



fn read_config(file_path: &str) -> Result<WebhookConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(file_path)?;
    let config: WebhookConfig = serde_yaml::from_str(&config_str)?;
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


    let method = actix_web::http::Method::try_from(config.get_verification_config().get_verification_method().as_str()).unwrap_or(actix_web::http::Method::GET);

    let data_routes = config.get_data_config().get_path_method_alias_vec();


    HttpServer::new(move || {
        let method = method.clone();
        let mut app = App::new()
            .app_data(web::Data::new(config.clone()))
            .route(
                "/verification/{path:.*}",
                web::route()
                    .guard(guard::fn_guard(move |ctx| ctx.head().method == method))
                    .to(endpoint_handler::verification_endpoint_handler),
            );

        // Add routes for each data endpoint
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
                    .to(move |req, payload, config| {
                        endpoint_handler::data_endpoint_handler(req, payload, alias_clone.clone(), config)
                    }),
            );
        }

        app
    })
        .bind(("0.0.0.0", port))?
        .run()
        .await
}