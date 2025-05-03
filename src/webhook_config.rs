use serde::Deserialize;
use crate::verification_handler::verification_config;
use crate::data_handler::data_config;

const VERIFICATION_PATH: &str = "verification";



#[derive(Clone, Debug, Deserialize)]
pub struct WebhookConfig {
    verification: verification_config::VerificationConfig,
    data: data_config::DataMap,
}

impl WebhookConfig {
    pub fn get_verification_path(&self) -> String {
        self.verification.get_path()
    }

    pub fn get_verification_method(&self) -> String {
        self.verification.get_method().to_uppercase()
    }

    pub fn is_verification_path_valid(&self) -> bool {
        let verification_path = self.get_verification_path();
        let config_segments: Vec<&str> = verification_path.split('/').collect();
        if config_segments.len() == 0 {
            return false;
        }

        // Handle various path formats
        match config_segments.first() {
            Some(&"") => {
                // Path starts with slash, like "/verification/..."
                config_segments.get(1).map_or(false, |&segment| segment == VERIFICATION_PATH)
            },
            Some(&segment) => {
                // Path doesn't start with slash, like "verification/..." or just "verification"
                segment == VERIFICATION_PATH
            },
            None => false,
        }
    }

    pub fn is_verification_path(&self, path: String) -> bool {
        // First check if base verification path is valid
        if !self.is_verification_path_valid() {
            return false;
        }

        let verification_path = self.get_verification_path();

        if verification_path.contains("...") {
            // Split both paths into segments and filter out empty segments
            let config_segments: Vec<&str> = verification_path.split('/')
                .filter(|s| !s.is_empty())
                .collect();
            let request_segments: Vec<&str> = path.split('/')
                .filter(|s| !s.is_empty())
                .collect();

            let mut request_idx = 0;
            let mut config_idx = 0;

            while config_idx < config_segments.len() && request_idx < request_segments.len() {
                match config_segments[config_idx] {
                    "..." => {
                        // Wildcard must match at least one segment
                        if config_idx == config_segments.len() - 1 {
                            // Last segment is wildcard, must have at least one remaining request segment
                            return request_segments.len() - request_idx >= 1;
                        }

                        // Find next non-wildcard segment in config
                        let next_config_idx = config_idx + 1;
                        if next_config_idx >= config_segments.len() {
                            break;
                        }

                        let next_segment = config_segments[next_config_idx];
                        if next_segment == "..." {
                            // Next segment is also wildcard, just advance
                            config_idx += 1;
                            continue;
                        }

                        // Save the current request index before looking for next segment
                        let saved_request_idx = request_idx;

                        // Find next occurrence of next_segment in request
                        while request_idx < request_segments.len() && request_segments[request_idx] != next_segment {
                            request_idx += 1;
                        }

                        if request_idx >= request_segments.len() {
                            return false; // Couldn't find next segment
                        }

                        // Ensure wildcard matched at least one segment
                        if request_idx == saved_request_idx {
                            return false; // Wildcard didn't match any segments
                        }

                        // Move past the wildcard in config
                        config_idx += 1;
                    },
                    segment if segment == request_segments[request_idx] => {
                        // Exact match
                        request_idx += 1;
                        config_idx += 1;
                    },
                    _ => {
                        // No match
                        return false;
                    }
                }
            }

            // Check if we've consumed all necessary segments
            return config_idx >= config_segments.len() ||
                config_segments[config_idx..].iter().all(|&s| s == "...");
        } else {
            // Simple exact matching for paths without wildcards
            let norm_config = verification_path.trim_end_matches('/').trim_start_matches('/');
            let norm_path = path.trim_end_matches('/').trim_start_matches('/');
            norm_config == norm_path
        }
    }
    pub fn set_token(&mut self, token: String) {
        self.verification.set_expected_token(token)
    }

    pub fn get_verification_config(&self) -> &verification_config::VerificationConfig {
        &self.verification
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Helper function to create a test WebhookConfig
    fn create_test_config(path: &str, method: &str) -> WebhookConfig {
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
        assert_eq!(config.get_verification_config().get_expected_token(), "");

        config.set_token("new_token".to_string());
        assert_eq!(config.get_verification_config().get_expected_token(), "new_token");
    }

    #[test]
    fn test_verification_config_access() {
        let config = create_test_config(VERIFICATION_PATH, "GET");

        // Test access to verification config
        let verification_config = config.get_verification_config();
        assert_eq!(verification_config.get_path(), VERIFICATION_PATH);
        assert_eq!(verification_config.get_method(), "GET");

        // Test token config
        let token_config = verification_config.get_token_config();
        assert_eq!(token_config.get_in(), "header");
        assert_eq!(token_config.get_locate(), "X-Hub-Signature");

        // Test challenge config
        let challenge_config = verification_config.get_challenge_config();
        assert_eq!(challenge_config.get_in(), "query");
        assert_eq!(challenge_config.get_locate(), "hub.challenge");

        // Test response config
        let response_config = verification_config.get_response_config();
        assert_eq!(response_config.get_content_type(), verification_config::ContentType::TextPlain);
        assert_eq!(response_config.get_data(), "{{challenge}}");
    }
}