use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use authaction::{actix::AuthenticatedUser, Verifier};

async fn public_route() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "This is a public message!"
    }))
}

async fn protected_route(user: AuthenticatedUser) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "This is a protected message!",
        "sub": user.claims.sub
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let domain = std::env::var("AUTHACTION_DOMAIN").expect("AUTHACTION_DOMAIN must be set");
    let audience = std::env::var("AUTHACTION_AUDIENCE").expect("AUTHACTION_AUDIENCE must be set");

    let verifier = web::Data::new(Verifier::new(&domain, &audience));

    println!("Server running at http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(verifier.clone())
            .route("/public", web::get().to(public_route))
            .route("/protected", web::get().to(protected_route))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
