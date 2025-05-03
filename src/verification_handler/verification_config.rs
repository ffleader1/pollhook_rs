use serde::{Deserialize, Deserializer};


#[derive(Debug, Deserialize, Clone)]
pub struct TokenConfig {
    #[serde(rename = "in")]
    location: String,
    locate: String,
}

impl TokenConfig {
    pub fn get_in(&self) -> String {
        self.location.clone()
    }

    pub fn get_locate(&self) -> String {
        self.locate.clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChallengeConfig {
    #[serde(rename = "in")]
    location: String,
    locate: String,
}

impl ChallengeConfig {
    pub fn get_in(&self) -> String {
        self.location.clone()
    }

    pub fn get_locate(&self) -> String {
        self.locate.clone()
    }
}

// Define an enum for content types
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    TextPlain,
    ApplicationJson,
}

// Custom deserialization for ContentType
impl<'de> Deserialize<'de> for ContentType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "text/plain" => Ok(ContentType::TextPlain),
            "application/json" => Ok(ContentType::ApplicationJson),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid content type: {}. Expected 'text/plain' or 'application/json'",
                s
            ))),
        }
    }
}

// Implementation to get string representation
impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::TextPlain => "text/plain",
            ContentType::ApplicationJson => "application/json",
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResponseConfig {
    #[serde(rename = "type")]
    content_type: ContentType,
    data: String,
    #[serde(default)]
    in_path: Option<String>, // Optional field for path in response data
}

impl ResponseConfig {
    pub fn get_content_type(&self) -> ContentType {
        self.content_type.clone()
    }

    pub fn get_data(&self) -> String {
        self.data.clone()
    }

    pub fn get_in_path(&self) -> Option<String> {
        self.in_path.clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct VerificationConfig {
    path: String,
    #[serde(default = "default_method")]
    method: String,
    #[serde(skip)]
    raw_token: String,
    token: TokenConfig,
    challenge: ChallengeConfig,
    response: ResponseConfig,
}

fn default_method() -> String {
    actix_web::http::Method::GET.to_string()
}

impl VerificationConfig {
    #[allow(dead_code)]
    pub fn get_verification_path(&self) -> String {
        self.path.clone()
    }

    pub fn get_verification_method(&self) -> String {
        self.method.clone().to_uppercase()
    }

    #[allow(dead_code)]
    pub fn get_expected_token(&self) -> String {
        self.raw_token.clone()
    }

    pub fn set_expected_token(&mut self, expected_token: String) {
        self.raw_token = expected_token;
    }


    pub fn get_token_config(&self) -> &TokenConfig {
        &self.token
    }

    pub fn get_challenge_config(&self) -> &ChallengeConfig {
        &self.challenge
    }

    pub fn get_response_config(&self) -> &ResponseConfig {
        &self.response
    }

    pub fn is_token_valid(&self, token: String) -> bool {
        self.raw_token.trim() == token.trim()
    }

    pub fn is_verification_path_valid(&self) -> bool {
        let verification_prefix = super::super::CALLBACK_PATH;
        let verification_path = self.path.clone();
        let config_segments: Vec<&str> = verification_path.split('/').collect();
        if config_segments.len() == 0 {
            return false;
        }

        // Handle various path formats
        match config_segments.first() {
            Some(&"") => {
                // Path starts with slash, like "/verification/..."
                config_segments.get(1).map_or(false, |&segment| segment == verification_prefix)
            },
            Some(&segment) => {
                // Path doesn't start with slash, like "verification/..." or just "verification"
                segment == verification_prefix
            },
            None => false,
        }
    }

    pub fn is_verification_path(&self, path_to_check: String) -> bool {
        // First check if base verification path is valid
        if !self.is_verification_path_valid() {
            return false;
        }

        let path_template = self.path.clone();

        if path_template.contains("...") {
            // Split both paths into segments and filter out empty segments
            let config_segments: Vec<&str> = path_template.split('/')
                .filter(|s| !s.is_empty())
                .collect();
            let request_segments: Vec<&str> = path_to_check.split('/')
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
            let norm_config = path_template.trim_end_matches('/').trim_start_matches('/');
            let norm_path = path_to_check.trim_end_matches('/').trim_start_matches('/');
            norm_config == norm_path
        }
    }

}