use actix_web::{web, HttpResponse, Result as ActixResult};
use serde_json::json;
use crate::models::*;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::ai_client;
use crate::hedera_client;
use sha2::{Sha256, Digest};

// ✅ Fonction 6 (Partielle) — Placeholder pour agents/offices
pub async fn agent_register_placeholder(_data: web::Json<serde_json::Value>) -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "coming_soon",
        "message": "L'enregistrement des agents sera disponible dans la prochaine version."
    })))
}

pub async fn office_register_placeholder(_data: web::Json<serde_json::Value>) -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "coming_soon",
        "message": "L'intégration avec les offices sera disponible dans la prochaine version."
    })))
}

// ✅ Création de compte OBLIGATOIRE — Première étape
pub async fn register_user(
    data: web::Json<RegisterUserRequest>,
    pool: web::Data<PgPool>,
) -> ActixResult<HttpResponse> {
    let user_id = Uuid::new_v4();

    if let Err(e) = sqlx::query!(
        "INSERT INTO users (id, full_name, email, phone, country, wallet_address, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        user_id,
        data.full_name,
        data.email,
        data.phone,
        data.country,
        data.wallet_address,
        Utc::now()
    )
    .execute(pool.as_ref())
    .await
    {
        eprintln!("Erreur création utilisateur: {}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({"message": "Échec de création de l'utilisateur"})));
    }

    Ok(HttpResponse::Ok().json(RegisterUserResponse {
        user_id,
        message: "Utilisateur enregistré avec succès".to_string(),
    }))
}

// ✅ Fonction 1: Soumettre une idée (texte)
pub async fn submit_idea(
    data: web::Json<SubmitIdeaRequest>,
    pool: web::Data<PgPool>,
) -> ActixResult<HttpResponse> {
    let idea_id = Uuid::new_v4();

    if let Err(e) = sqlx::query!(
        "INSERT INTO ideas (id, user_id, raw_idea, created_at) VALUES ($1, $2, $3, $4)",
        idea_id,
        data.user_id,
        data.raw_idea,
        Utc::now()
    )
    .execute(pool.as_ref())
    .await
    {
        eprintln!("Erreur insertion idée: {}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({"message": "Échec d'enregistrement de l'idée"})));
    }

    Ok(HttpResponse::Ok().json(SubmitIdeaResponse {
        idea_id,
        message: "Idée enregistrée avec succès".to_string(),
    }))
}

// ✅ Fonction 2: Générer le résumé IA
pub async fn generate_summary(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> ActixResult<HttpResponse> {
    let idea_id = path.into_inner();

    // ✅ Correction 1 : Utiliser match au lieu de ? après map_err
    let idea_row = match sqlx::query_as!(Idea, "SELECT * FROM ideas WHERE id = $1", idea_id)
        .fetch_optional(pool.as_ref())
        .await
    {
        Ok(row) => row,
        Err(e) => {
            eprintln!("Erreur récupération idée: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"message": "Erreur serveur"})));
        }
    };

    let idea = match idea_row {
        Some(i) => i,
        None => return Ok(HttpResponse::NotFound().json(json!({"message": "Idée non trouvée"}))),
    };

    let ai_response = match ai_client::call_ai_service(idea.raw_idea.clone()).await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Erreur IA: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"message": "Service IA indisponible"})));
        }
    };

    if ai_response.novelty_score < 50 {
        return Ok(HttpResponse::BadRequest().json(json!({"message": "Idée probablement non brevetable"})));
    }

    let summary_id = Uuid::new_v4();

    if let Err(e) = sqlx::query!(
        r#"INSERT INTO summaries (id, idea_id, title, problem, solution, claim, cpc_code, created_at)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        summary_id,
        idea_id,
        ai_response.title,
        ai_response.problem,
        ai_response.solution,
        ai_response.claim,
        ai_response.cpc_code,
        Utc::now()
    )
    .execute(pool.as_ref())
    .await
    {
        eprintln!("Erreur insertion résumé: {}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({"message": "Échec stockage résumé"})));
    }

    Ok(HttpResponse::Ok().json(json!({
        "summary_id": summary_id,
        "status": "completed",
        "message": "Résumé IA généré avec succès"
    })))
}

// ✅ Fonction 3: Enregistrer la preuve sur Hedera
pub async fn register_proof(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> ActixResult<HttpResponse> {
    let summary_id = path.into_inner();

    // ✅ Correction 2 : Utiliser match au lieu de ? après map_err
    let summary_row = match sqlx::query_as!(Summary, "SELECT * FROM summaries WHERE id = $1", summary_id)
        .fetch_optional(pool.as_ref())
        .await
    {
        Ok(row) => row,
        Err(e) => {
            eprintln!("Erreur récupération résumé: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"message": "Erreur serveur"})));
        }
    };

    let summary = match summary_row {
        Some(s) => s,
        None => return Ok(HttpResponse::NotFound().json(json!({"message": "Résumé non trouvé"}))),
    };

    let structured_data = format!(
        r#"{{"title":"{}","problem":"{}","solution":"{}","claim":"{}","cpc_code":"{}"}}"#,
        summary.title, summary.problem, summary.solution, summary.claim, summary.cpc_code
    );

    let mut hasher = Sha256::new();
    hasher.update(structured_data.as_bytes());
    let patent_hash = format!("{:x}", hasher.finalize());

    let hedera_tx_id = match hedera_client::submit_to_hedera(
        patent_hash.clone(),
        summary.cpc_code.clone(),
        "PLACEHOLDER_WALLET".to_string(), // À remplacer par user_wallet dans v2
        summary.created_at,
    ).await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Échec Hedera: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"message": "Échec enregistrement blockchain"})));
        }
    };

    let proof_id = Uuid::new_v4();

    if let Err(e) = sqlx::query!(
        "INSERT INTO proofs (id, summary_id, hash, hedera_tx_id, timestamp, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6)",
        proof_id,
        summary_id,
        patent_hash,
        hedera_tx_id,
        summary.created_at,
        Utc::now()
    )
    .execute(pool.as_ref())
    .await
    {
        eprintln!("Erreur insertion preuve: {}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({"message": "Échec stockage preuve"})));
    }

    Ok(HttpResponse::Ok().json(json!({
        "transaction_id": hedera_tx_id,
        "timestamp": summary.created_at.to_rfc3339(),
        "status": "registered",
        "message": "Preuve enregistrée sur Hedera avec succès"
    })))
}

// ✅ Fonction 4: Récupérer le certificat
pub async fn get_certificate(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> ActixResult<HttpResponse> {
    let summary_id = path.into_inner();

    // ✅ Correction 3 : Utiliser match au lieu de ? après map_err
    let proof_row = match sqlx::query_as!(Proof, "SELECT * FROM proofs WHERE summary_id = $1", summary_id)
        .fetch_optional(pool.as_ref())
        .await
    {
        Ok(row) => row,
        Err(e) => {
            eprintln!("Erreur récupération preuve: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"message": "Erreur serveur"})));
        }
    };

    let proof = match proof_row {
        Some(p) => p,
        None => return Ok(HttpResponse::NotFound().json(json!({"message": "Preuve non trouvée"}))),
    };

    let explorer_url = format!("https://hashscan.io/testnet/transaction/{}", proof.hedera_tx_id);

    Ok(HttpResponse::Ok().json(CertificateResponse {
        hash: proof.hash,
        timestamp: proof.timestamp.to_rfc3339(),
        hedera_tx_id: proof.hedera_tx_id,
        explorer_url,
    }))
}

// ✅ Fonction 5: Vérifier le statut
pub async fn get_status(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> ActixResult<HttpResponse> {
    let idea_id = path.into_inner();

    // ✅ Correction 4 : Remplacer SELECT 1 par SELECT true AS exists
    let idea_exists = sqlx::query!("SELECT true AS exists FROM ideas WHERE id = $1", idea_id)
        .fetch_optional(pool.as_ref())
        .await
        .map(|r| r.is_some())
        .unwrap_or(false);

    let summary = sqlx::query!("SELECT true AS exists FROM summaries WHERE idea_id = $1", idea_id)
        .fetch_optional(pool.as_ref())
        .await
        .map(|r| r.is_some())
        .unwrap_or(false);

    let proof = sqlx::query!(
        "SELECT true AS exists FROM proofs p JOIN summaries s ON p.summary_id = s.id WHERE s.idea_id = $1",
        idea_id
    )
    .fetch_optional(pool.as_ref())
    .await
    .map(|r| r.is_some())
    .unwrap_or(false);

    Ok(HttpResponse::Ok().json(StatusResponse {
        idea_received: idea_exists,
        ia_summary_ready: summary,
        hedera_proof_registered: proof,
        agent_validated: false, // Placeholder
        office_submitted: false, // Placeholder
    }))
}

// ✅ Fonction 7: Health check
pub async fn health() -> ActixResult<HttpResponse> {
    let services = vec!["database".to_string(), "hedera".to_string(), "ai".to_string()];
    Ok(HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        services,
    }))
}