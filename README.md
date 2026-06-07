# authaction-rust-actix-example

A Rust Actix-web application demonstrating API authorization using [AuthAction](https://app.authaction.com/) with the `authaction` crate.

## Overview

This application shows how to configure and handle authorization using AuthAction's access tokens in an Actix-web API. It validates JSON Web Tokens (JWT) using the `authaction` crate (with the `actix` feature), which provides an `AuthenticatedUser` extractor that handles JWKS fetching and RS256 validation automatically.

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
│   └── main.rs      # Actix-web app setup, Verifier, and route handlers
├── Cargo.toml
├── .env.example
└── README.md
```

## Code Explanation

### `src/main.rs` — App Setup and Routes

- **`Verifier::new(&domain, &audience)`** — Creates an `authaction::Verifier` from the `authaction` crate. The verifier is wrapped in `web::Data` and shared across all Actix workers.

- **`AuthenticatedUser`** — An Actix-web `FromRequest` extractor from `authaction::actix`. Adding it as a handler parameter automatically validates the Bearer token using the shared `Verifier` and injects the decoded claims. Returns HTTP 401 on any validation failure.

- **`GET /public`** — No extractor, accessible without authentication.
- **`GET /protected`** — Takes `user: AuthenticatedUser`; Actix calls `FromRequest` before the handler runs, rejecting invalid or missing tokens.

## Common Issues

**Invalid token errors** — Verify that `AUTHACTION_DOMAIN` and
`AUTHACTION_AUDIENCE` match the values in your AuthAction dashboard exactly.

**Public key fetching errors** — Check that your application can reach
`https://{AUTHACTION_DOMAIN}/.well-known/jwks.json`.

**Unauthorized access** — Ensure the `Authorization: Bearer <token>` header is
present and the token was issued for the correct audience.

## Contributing

Feel free to submit issues or pull requests if you encounter bugs or have suggestions for improvement!
