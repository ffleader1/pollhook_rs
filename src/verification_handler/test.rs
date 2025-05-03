use super::*;

#[cfg(test)]
mod tests {
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