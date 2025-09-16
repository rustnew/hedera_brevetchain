use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use dotenvy::dotenv;
use std::env;

mod models;
mod routes;
mod ai_client;
mod hedera_client;

async fn create_pool() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPool::connect(&database_url).await.expect("Failed to connect to DB")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = web::Data::new(create_pool().await);

    println!("Backend en cours d'exécution sur http://127.0.0.1:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")// Ajout pour prod
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE])
            .supports_credentials();

        App::new()
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
            .app_data(pool.clone())
            .service(
                web::scope("/api")
                    .route("/submit", web::post().to(routes::submit_idea))
                    .route("/structured/{patent_id}", web::get().to(routes::get_structured_patent))
                    .route("/verify/{patent_id}", web::get().to(routes::verify_patent))
                    .route("/drafts/{user_id}", web::get().to(routes::list_user_drafts)) // Bonus
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}