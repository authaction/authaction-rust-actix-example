# authaction-rust-actix-example

A Rust Actix-web application demonstrating API authorization using [AuthAction](https://app.authaction.com/) with JWKS-based JWT validation.

## Overview

This application shows how to configure and handle authorization using AuthAction's access tokens in an Actix-web API. It validates JSON Web Tokens (JWT) signed with RS256 by fetching public keys dynamically from AuthAction's JWKS endpoint. Public keys are cached in-process using a `tokio::sync::RwLock`, with automatic single-retry on key rotation.

## Prerequisites

- **Rust 1.75+** (install via [rustup](https://rustup.rs))
- **AuthAction credentials**: `tenantDomain` and `apiIdentifier` from your AuthAction account.

## Installation

1. **Clone the repository**:

   ```bash
   git clone git@github.com:authaction/authaction-rust-actix-example.git
   cd authaction-rust-actix-example
   ```

2. **Configure your AuthAction credentials**:

   ```bash
   cp .env.example .env
   ```

   Edit `.env` and replace the placeholders:

   ```env
   AUTHACTION_DOMAIN=your-authaction-tenant-domain
   AUTHACTION_AUDIENCE=your-authaction-api-identifier
   ```

## Usage

1. **Start the server**:

   ```bash
   cargo run
   ```

   The API will be available at `http://localhost:8080`.

2. **Obtain an access token** via client credentials:

   ```bash
   curl --request POST \
     --url https://your-authaction-tenant-domain/oauth2/m2m/token \
     --header 'content-type: application/json' \
     --data '{
       "client_id": "your-authaction-app-clientid",
       "client_secret": "your-authaction-app-client-secret",
       "audience": "your-authaction-api-identifier",
       "grant_type": "client_credentials"
     }'
   ```

3. **Call the public endpoint** (no token required):

   ```bash
   curl http://localhost:8080/public
   ```

   ```json
   { "message": "This is a public message!" }
   ```

4. **Call the protected endpoint** with the access token:

   ```bash
   curl --request GET \
     --url http://localhost:8080/protected \
     --header 'Authorization: Bearer YOUR_ACCESS_TOKEN'
   ```

   ```json
   { "message": "This is a protected message!", "sub": "client-id@clients" }
   ```

## Project Structure

```
authaction-rust-actix-example/
├── src/
│   ├── main.rs      # Actix-web app setup and route handlers
│   ├── auth.rs      # AppState, JWKS cache, JWT validation, AuthenticatedUser extractor
│   └── models.rs    # Claims struct
├── Cargo.toml
├── .env.example
└── README.md
```

## Code Explanation

### `src/auth.rs` — JWT Validation

Equivalent to `JwtStrategy` in the NestJS example.

- **`AppState`** — Holds the shared `domain`, `audience`, and a
  `tokio::sync::RwLock<Option<Arc<JwkSet>>>` JWKS cache. Wrapped in
  `web::Data<AppState>` and shared across all Actix workers.

- **`get_jwks()`** — Acquires a read lock and returns the cached key set. On a
  cache miss, upgrades to a write lock, fetches
  `https://{AUTHACTION_DOMAIN}/.well-known/jwks.json` via `reqwest`, and stores
  an `Arc<JwkSet>` so cloning is O(1).

- **`verify_token()`** — Decodes the token header to extract `kid`, then
  validates the JWT using `jsonwebtoken::decode` with:
  - Algorithm: `RS256`
  - Issuer: `https://{AUTHACTION_DOMAIN}` (`set_issuer`)
  - Audience: `{AUTHACTION_AUDIENCE}` (`set_audience`)

  If the `kid` is absent from the cached key set (key rotation), it calls
  `invalidate_and_refetch()` and retries once before returning `KeyNotFound`.

- **`AuthenticatedUser`** — An Actix-web `FromRequest` extractor. Adding it as
  a handler parameter automatically validates the Bearer token and injects the
  decoded `Claims`. Returns HTTP 401 on any validation failure.

### `src/main.rs` — Routes

- **`GET /public`** — No extractor, accessible without authentication.
- **`GET /protected`** — Takes `user: AuthenticatedUser`; Actix calls
  `FromRequest` before the handler runs, rejecting invalid/missing tokens.

## Common Issues

**Invalid token errors** — Verify that `AUTHACTION_DOMAIN` and
`AUTHACTION_AUDIENCE` match the values in your AuthAction dashboard exactly.

**Public key fetching errors** — Check that your application can reach
`https://{AUTHACTION_DOMAIN}/.well-known/jwks.json`.

**Unauthorized access** — Ensure the `Authorization: Bearer <token>` header is
present and the token was issued for the correct audience.

## Contributing

Feel free to submit issues or pull requests if you encounter bugs or have suggestions for improvement!
