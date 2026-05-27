use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use actix_web::{dev::Payload, error, web::Data, FromRequest, HttpRequest};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use tokio::sync::RwLock;

use crate::models::Claims;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum AuthError {
    MissingHeader,
    InvalidHeader,
    MissingKid,
    KeyNotFound,
    Jwt(jsonwebtoken::errors::Error),
    Fetch(reqwest::Error),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::MissingHeader => write!(f, "Missing Authorization header"),
            AuthError::InvalidHeader => write!(f, "Invalid Authorization header format"),
            AuthError::MissingKid => write!(f, "Missing kid in token header"),
            AuthError::KeyNotFound => write!(f, "Matching public key not found"),
            AuthError::Jwt(e) => write!(f, "Token validation failed: {e}"),
            AuthError::Fetch(e) => write!(f, "Failed to fetch JWKS: {e}"),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        AuthError::Jwt(e)
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(e: reqwest::Error) -> Self {
        AuthError::Fetch(e)
    }
}

// ── App state ─────────────────────────────────────────────────────────────────

pub struct AppState {
    jwks_cache: RwLock<Option<Arc<JwkSet>>>,
    pub domain: String,
    pub audience: String,
}

impl AppState {
    pub fn new(domain: String, audience: String) -> Self {
        Self {
            jwks_cache: RwLock::new(None),
            domain,
            audience,
        }
    }

    pub async fn get_jwks(&self) -> Result<Arc<JwkSet>, AuthError> {
        {
            let guard = self.jwks_cache.read().await;
            if let Some(ref jwks) = *guard {
                return Ok(Arc::clone(jwks));
            }
        }
        self.fetch_and_cache_jwks().await
    }

    // Busts the cache and re-fetches — called on key rotation (kid not found).
    pub async fn invalidate_and_refetch(&self) -> Result<Arc<JwkSet>, AuthError> {
        {
            let mut guard = self.jwks_cache.write().await;
            *guard = None;
        }
        self.fetch_and_cache_jwks().await
    }

    async fn fetch_and_cache_jwks(&self) -> Result<Arc<JwkSet>, AuthError> {
        let url = format!("https://{}/.well-known/jwks.json", self.domain);
        let jwks: JwkSet = reqwest::get(&url).await?.json().await?;
        let jwks = Arc::new(jwks);
        let mut guard = self.jwks_cache.write().await;
        *guard = Some(Arc::clone(&jwks));
        Ok(jwks)
    }
}

// ── Token verification ────────────────────────────────────────────────────────

pub async fn verify_token(token: &str, state: &AppState) -> Result<Claims, AuthError> {
    let header = decode_header(token)?;
    let kid = header.kid.ok_or(AuthError::MissingKid)?;

    let issuer = format!("https://{}", state.domain);
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[&issuer]);
    validation.set_audience(&[&state.audience]);

    // Try cached JWKS; if kid is absent (key rotation), bust cache and retry once.
    let jwks = state.get_jwks().await?;
    if let Some(jwk) = jwks.find(&kid) {
        let key = DecodingKey::from_jwk(jwk)?;
        let data = decode::<Claims>(token, &key, &validation)?;
        return Ok(data.claims);
    }

    let jwks = state.invalidate_and_refetch().await?;
    let jwk = jwks.find(&kid).ok_or(AuthError::KeyNotFound)?;
    let key = DecodingKey::from_jwk(jwk)?;
    let data = decode::<Claims>(token, &key, &validation)?;
    Ok(data.claims)
}

fn extract_bearer_token(req: &HttpRequest) -> Result<String, AuthError> {
    let value = req
        .headers()
        .get("Authorization")
        .ok_or(AuthError::MissingHeader)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeader)?;

    value
        .strip_prefix("Bearer ")
        .map(|t| t.trim().to_string())
        .ok_or(AuthError::InvalidHeader)
}

// ── Actix extractor ───────────────────────────────────────────────────────────

/// Extractor that validates the Bearer JWT and injects the decoded claims.
/// Use as a handler parameter to protect any route:
///   `async fn handler(user: AuthenticatedUser) -> impl Responder { ... }`
pub struct AuthenticatedUser {
    pub claims: Claims,
}

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let state = req
                .app_data::<Data<AppState>>()
                .ok_or_else(|| error::ErrorInternalServerError("missing app state"))?;

            let token = extract_bearer_token(&req)
                .map_err(|e| error::ErrorUnauthorized(e.to_string()))?;

            let claims = verify_token(&token, state)
                .await
                .map_err(|e| error::ErrorUnauthorized(e.to_string()))?;

            Ok(AuthenticatedUser { claims })
        })
    }
}
