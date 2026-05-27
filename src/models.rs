use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: Option<String>,
    pub exp: u64,
    pub iat: Option<u64>,
}
