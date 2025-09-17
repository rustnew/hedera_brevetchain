use actix_cors::Cors;
use actix_web::{App, HttpServer, web,  middleware};
use actix_files::Files;
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

    println!("🚀 Backend MVP BrevetChain démarré sur http://127.0.0.1:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(pool.clone())
            .service(
                web::scope("/api/v1")
                    .route("/register", web::post().to(routes::register_user)) // ✅ Création de compte OBLIGATOIRE
                    .route("/submit-idea", web::post().to(routes::submit_idea)) // ✅ Fonction 1
                    .route("/generate-summary/{idea_id}", web::post().to(routes::generate_summary)) // ✅ Fonction 2
                    .route("/register-proof/{summary_id}", web::post().to(routes::register_proof)) // ✅ Fonction 3
                    .route("/certificate/{summary_id}", web::get().to(routes::get_certificate)) // ✅ Fonction 4
                    .route("/status/{idea_id}", web::get().to(routes::get_status)) // ✅ Fonction 5
                    .route("/health", web::get().to(routes::health)) // ✅ Fonction 7
                    // Fonction 6 (CRUD agents/offices) est structurée mais désactivée → placeholder
                    .route("/agent/register", web::post().to(routes::agent_register_placeholder))
                    .route("/office/register", web::post().to(routes::office_register_placeholder))
            )
            .service(Files::new("/", "../frontend").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}