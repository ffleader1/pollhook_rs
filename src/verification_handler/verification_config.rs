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
    pub fn get_path(&self) -> String{
        self.path.clone()
    }

    pub fn get_method(&self) -> String{
        self.method.clone()
    }

    pub fn get_expected_token(&self) -> String {
        self.raw_token.clone()
    }

    pub fn set_expected_token(&mut self, expected_token: String)  {
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
}