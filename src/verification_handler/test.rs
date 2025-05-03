#[allow(unused_imports)]
use super::*;
#[allow(dead_code)]
const VERIFICATION_PATH: &str = "callhook";
#[cfg(test)]
mod test_exact_query {
    use super::extractors::*;
    
    use actix_web::test;
    use actix_web::http::header;
    use bytes::Bytes;
    use serde_json::json;

    #[actix_web::test]
    async fn test_extract_from_query() {
        let req = test::TestRequest::with_uri("/?param=value").to_http_request();
        let result = extract_value(&req, "query", "param", &None, "Test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "value");

        // Test missing param
        let req = test::TestRequest::with_uri("/?other=value").to_http_request();
        let result = extract_value(&req, "query", "param", &None, "Test");
        assert!(result.is_err());
    }

    #[actix_web::test]
    async fn test_extract_from_header() {
        let req = test::TestRequest::default()
            .insert_header((header::AUTHORIZATION, "Bearer token123"))
            .to_http_request();

        let result = extract_value(&req, "header", "authorization", &None, "Test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Bearer token123");

        // Test missing header
        let req = test::TestRequest::default().to_http_request();
        let result = extract_value(&req, "header", "authorization", &None, "Test");
        assert!(result.is_err());
    }

    #[actix_web::test]
    async fn test_extract_from_path() {
        let req = test::TestRequest::with_uri("/api/v1/resource").to_http_request();

        // Get the third segment (index 2)
        let result = extract_value(&req, "path", "3", &None, "Test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "resource");

        // Test out of bounds
        let result = extract_value(&req, "path", "5", &None, "Test");
        assert!(result.is_err());

        // Test invalid index
        let result = extract_value(&req, "path", "not-a-number", &None, "Test");
        assert!(result.is_err());
    }

    #[actix_web::test]
    async fn test_extract_from_body() {
        let json_body = json!({
            "data": {
                "user": {
                    "id": "user123",
                    "name": "John Doe"
                },
                "token": "secret-token"
            }
        });

        let body_bytes = Some(Bytes::from(json_body.to_string()));
        let req = test::TestRequest::default().to_http_request();

        // Test simple path
        let result = extract_value(&req, "body", "data::token", &body_bytes, "Test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "secret-token");

        // Test nested path
        let result = extract_value(&req, "body", "data::user::id", &body_bytes, "Test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "user123");

        // Test missing path
        let result = extract_value(&req, "body", "data::missing", &body_bytes, "Test");
        assert!(result.is_err());

        // Test no body provided
        let result = extract_value(&req, "body", "data::token", &None, "Test");
        assert!(result.is_err());

        // Test non-string value
        let result = extract_value(&req, "body", "data::user", &body_bytes, "Test");
        assert!(result.is_err()); // Should fail because user is an object, not a string
    }

    #[actix_web::test]
    async fn test_unsupported_location() {
        let req = test::TestRequest::default().to_http_request();
        let result = extract_value(&req, "unsupported", "param", &None, "Test");
        assert!(result.is_err());
    }

    #[actix_web::test]
    async fn test_invalid_json_body() {
        let invalid_json = Bytes::from("not a json");
        let req = test::TestRequest::default().to_http_request();
        let result = extract_value(&req, "body", "data", &Some(invalid_json), "Test");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_verification {
    use super::*;
    use serde_json::json;
    use crate::verification_handler::verification_config::VerificationConfig;

    // Helper function to create a test WebhookConfig
    fn create_test_config(path: &str, method: &str) -> VerificationConfig {
        // Create a JSON representation that can be deserialized into WebhookConfig
        let config_json = json!({
            "verification": {
                "path": path,
                "method": method,
                "token": {
                    "in": "header",
                    "locate": "X-Hub-Signature"
                },
                "challenge": {
                    "in": "query",
                    "locate": "hub.challenge"
                },
                "response": {
                    "type": "text/plain",
                    "data": "{{challenge}}"
                }
            },
            "data": {}
        });

        serde_json::from_value(config_json).unwrap()
    }

    #[test]
    fn test_get_verification_path() {
        let config = create_test_config(&format!("{}/endpoint", VERIFICATION_PATH), "GET");
        assert_eq!(config.get_verification_path(), format!("{}/endpoint", VERIFICATION_PATH));
    }

    #[test]
    fn test_get_verification_method() {
        let config = create_test_config(&format!("{}/endpoint", VERIFICATION_PATH), "post");
        assert_eq!(config.get_verification_method(), "POST");
    }

    #[test]
    fn test_is_verification_path_valid() {
        // Valid paths
        assert!(create_test_config(VERIFICATION_PATH, "GET").is_verification_path_valid());
        assert!(create_test_config(&format!("{}/endpoint", VERIFICATION_PATH), "GET").is_verification_path_valid());
        assert!(create_test_config(&format!("/{}", VERIFICATION_PATH), "GET").is_verification_path_valid());
        assert!(create_test_config(&format!("/{}/endpoint", VERIFICATION_PATH), "GET").is_verification_path_valid());

        // Invalid paths
        assert!(!create_test_config("", "GET").is_verification_path_valid());
        assert!(!create_test_config("/", "GET").is_verification_path_valid());
        assert!(!create_test_config("invalid", "GET").is_verification_path_valid());
        assert!(!create_test_config("/invalid", "GET").is_verification_path_valid());
        assert!(!create_test_config(&format!("/invalid/{}", VERIFICATION_PATH), "GET").is_verification_path_valid());
    }

    #[test]
    fn test_is_verification_path_exact_matching() {
        // Test cases without wildcards
        let config = create_test_config(&format!("{}/endpoint", VERIFICATION_PATH), "GET");

        // Exact matches
        assert!(config.is_verification_path(format!("{}/endpoint", VERIFICATION_PATH)));
        assert!(config.is_verification_path(format!("/{}/endpoint", VERIFICATION_PATH)));
        assert!(config.is_verification_path(format!("{}/endpoint/", VERIFICATION_PATH)));
        assert!(config.is_verification_path(format!("/{}/endpoint/", VERIFICATION_PATH)));

        // Non-matches
        assert!(!config.is_verification_path(VERIFICATION_PATH.to_string()));
        assert!(!config.is_verification_path(format!("{}/wrong", VERIFICATION_PATH)));
        assert!(!config.is_verification_path(format!("{}/endpoint/extra", VERIFICATION_PATH)));
        assert!(!config.is_verification_path(format!("wrong/{}/endpoint", VERIFICATION_PATH)));
    }

    #[test]
    fn test_is_verification_path_with_wildcards() {
        // Test with a wildcard in the middle
        let config_middle = create_test_config(&format!("{}/.../endpoint", VERIFICATION_PATH), "GET");

        assert!(config_middle.is_verification_path(format!("{}/anything/endpoint", VERIFICATION_PATH)));
        assert!(config_middle.is_verification_path(format!("{}/multi/level/path/endpoint", VERIFICATION_PATH)));
        assert!(config_middle.is_verification_path(format!("/{}/anything/endpoint", VERIFICATION_PATH)));

        assert!(!config_middle.is_verification_path(format!("{}/endpoint", VERIFICATION_PATH)));
        assert!(!config_middle.is_verification_path(format!("{}/endpoint/extra", VERIFICATION_PATH)));
        assert!(!config_middle.is_verification_path(VERIFICATION_PATH.to_string()));

        // Test with a wildcard at the end
        let config_end = create_test_config(&format!("{}/endpoint/...", VERIFICATION_PATH), "GET");

        assert!(config_end.is_verification_path(format!("{}/endpoint", VERIFICATION_PATH)));
        assert!(config_end.is_verification_path(format!("{}/endpoint/extra", VERIFICATION_PATH)));
        assert!(config_end.is_verification_path(format!("{}/endpoint/multi/level/extra", VERIFICATION_PATH)));

        assert!(!config_end.is_verification_path(VERIFICATION_PATH.to_string()));
        assert!(!config_end.is_verification_path(format!("{}/wrong", VERIFICATION_PATH)));

        // Test with a wildcard at the beginning. All should NOT be allowed.
        let config_start = create_test_config(&format!(".../{}/endpoint", VERIFICATION_PATH), "GET");

        assert!(!config_start.is_verification_path(format!("prefix/{}/endpoint", VERIFICATION_PATH)));
        assert!(!config_start.is_verification_path(format!("multi/level/prefix/{}/endpoint", VERIFICATION_PATH)));
        assert!(!config_start.is_verification_path(format!("{}/endpoint", VERIFICATION_PATH)));

        assert!(!config_start.is_verification_path(VERIFICATION_PATH.to_string()));
        assert!(!config_start.is_verification_path(format!("{}/endpoint/extra", VERIFICATION_PATH)));
    }

    #[test]
    fn test_is_verification_path_multiple_wildcards() {
        // Test with multiple wildcards
        let config = create_test_config(&format!("{}/.../middle/.../endpoint", VERIFICATION_PATH), "GET");

        assert!(config.is_verification_path(format!("{}/anything/middle/something/endpoint", VERIFICATION_PATH)));
        assert!(config.is_verification_path(format!("{}/a/b/c/middle/x/y/z/endpoint", VERIFICATION_PATH)));

        assert!(!config.is_verification_path(format!("{}/middle/endpoint", VERIFICATION_PATH)));
        assert!(!config.is_verification_path(format!("{}/anything/endpoint", VERIFICATION_PATH)));
        assert!(!config.is_verification_path(format!("{}/anything/middle", VERIFICATION_PATH)));
    }

    #[test]
    fn test_is_verification_path_adjacent_wildcards() {
        // Test with adjacent wildcards (should treat each wildcard separately)
        let config = create_test_config(&format!("{}/.../.../.../endpoint", VERIFICATION_PATH), "GET");

        // This should match, with one segment per wildcard
        assert!(config.is_verification_path(format!("{}/a/b/c/endpoint", VERIFICATION_PATH)));

        // This should match, with multiple segments for the first wildcard and none for the others
        assert!(config.is_verification_path(format!("{}/a/b/c/d/endpoint", VERIFICATION_PATH)));

        // This should match, with segments distributed across wildcards
        assert!(config.is_verification_path(format!("{}/a/b/c/endpoint", VERIFICATION_PATH)));

        // This should not match (missing endpoint)
        assert!(!config.is_verification_path(format!("{}/a/b/c", VERIFICATION_PATH)));

        // This should not match (wrong path)
        assert!(!config.is_verification_path(format!("{}/a/b/c/wrong", VERIFICATION_PATH)));
    }

    #[test]
    fn test_is_verification_path_edge_cases() {
        // Test with invalid verification path
        let invalid_config = create_test_config("invalid", "GET");

        assert!(!invalid_config.is_verification_path("invalid".to_string()));
        assert!(!invalid_config.is_verification_path(VERIFICATION_PATH.to_string()));

        // Test with empty path
        let empty_config = create_test_config("", "GET");
        assert!(!empty_config.is_verification_path("".to_string()));
        assert!(!empty_config.is_verification_path(VERIFICATION_PATH.to_string()));
    }

    #[test]
    fn test_set_token() {
        let mut config = create_test_config(VERIFICATION_PATH, "GET");

        config.set_expected_token("new_token".to_string());
        assert_eq!(config.get_expected_token(), "new_token");
    }

}