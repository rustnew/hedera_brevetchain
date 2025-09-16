use actix_web::{web, HttpResponse, Result as ActixResult, error::ErrorInternalServerError};
use serde_json::json;
use crate::models::{SubmitIdeaRequest, SubmitIdeaResponse, PatentStatus, StructuredPatent, UserInfo};
use crate::models::PatentDraft;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::ai_client;
use crate::hedera_client;
use sha2::{Sha256, Digest};







pub async fn submit_idea(
    data: web::Json<SubmitIdeaRequest>,
    pool: web::Data<PgPool>,
) -> ActixResult<HttpResponse> {
    let user_info = data.user.clone();
    let raw_idea = data.patent.raw_idea.clone();

    let user_id = match get_or_create_user(&pool, &user_info).await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur création utilisateur: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"message": "Échec de création de l'utilisateur"})));
        }
    };

    let patent_id = Uuid::new_v4();

    let ai_response = match ai_client::call_ai_service(raw_idea.clone()).await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Erreur IA: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"message": "Service IA indisponible"})));
        }
    };

    if ai_response.novelty_score < 50 {
        return Ok(HttpResponse::BadRequest().json(json!({"message": "Idée probablement non brevetable"})));
    }

    // Fix: Insérer d'abord dans patent_drafts
    let draft = PatentDraft {
        id: patent_id,
        user_id,
        raw_idea,
        status: PatentStatus::Submitted.as_str().to_string(),
        created_at: Utc::now(),
        hedera_tx_id: None,
    };

    if let Err(e) = sqlx::query!(
        "INSERT INTO patent_drafts (id, user_id, raw_idea, status, created_at, hedera_tx_id) VALUES ($1, $2, $3, $4, $5, $6)",
        draft.id,
        draft.user_id,
        draft.raw_idea,
        draft.status,
        draft.created_at,
        draft.hedera_tx_id
    )
    .execute(pool.as_ref())
    .await
    {
        eprintln!("Erreur SQL draft: {}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({"message": "Échec insertion draft"})));
    }

    // Ensuite, insérer dans structured_patents
    let structured_patent = StructuredPatent {
        id: Uuid::new_v4(),
        patent_draft_id: patent_id,
        title: ai_response.title,
        problem: ai_response.problem,
        solution: ai_response.solution,
        claims: serde_json::to_value(ai_response.claims).unwrap(),
        cpc_code: ai_response.cpc_code.clone(),
        created_at: Utc::now(),
    };

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO structured_patents (id, patent_draft_id, title, problem, solution, claims, cpc_code, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        structured_patent.id,
        structured_patent.patent_draft_id,
        structured_patent.title,
        structured_patent.problem,
        structured_patent.solution,
        &structured_patent.claims,
        structured_patent.cpc_code,
        structured_patent.created_at
    )
    .execute(pool.as_ref())
    .await
    {
        eprintln!("Erreur SQL structured: {}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({"message": "Échec stockage données structurées"})));
    }

    let structured_data_json = match serde_json::to_string(&structured_patent) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Erreur sérialisation: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"message": "Échec sérialisation"})));
        }
    };

    let mut hasher = Sha256::new();
    hasher.update(structured_data_json.as_bytes());
    let patent_hash = format!("{:x}", hasher.finalize());

    let hedera_tx_id = match hedera_client::submit_to_hedera(
        patent_hash.clone(),
        ai_response.cpc_code,
        user_info.wallet_address,
        structured_patent.created_at,
    ).await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Échec Hedera: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"message": "Échec enregistrement blockchain"})));
        }
    };

    if let Err(e) = sqlx::query!(
        "UPDATE patent_drafts SET status = $1, hedera_tx_id = $2 WHERE id = $3",
        PatentStatus::OnBlockchain.as_str(),
        &hedera_tx_id,
        patent_id
    )
    .execute(pool.as_ref())
    .await
    {
        eprintln!("Échec update DB: {}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({"message": "Échec update tx_id"})));
    }

    Ok(HttpResponse::Ok().json(SubmitIdeaResponse {
        patent_id,
        message: "Idée enregistrée avec succès".to_string(),
        structured_patent: Some(structured_patent),
    }))
}







async fn get_or_create_user(pool: &PgPool, info: &UserInfo) -> Result<Uuid, sqlx::Error> {
    if let Some(row) = sqlx::query!("SELECT id FROM users WHERE email = $1", info.email)
        .fetch_optional(pool).await?
    {
        Ok(row.id)
    } else {
        let new_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, full_name, email, phone, country, wallet_address, created_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            new_id,
            info.full_name,
            info.email,
            info.phone,
            info.country,
            info.wallet_address,
            Utc::now()
        )
        .execute(pool)
        .await?;
        Ok(new_id)
    }
}






pub async fn get_structured_patent(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> ActixResult<HttpResponse> {
    let patent_id = path.into_inner();

    let row = sqlx::query!(
        r#"
        SELECT id, patent_draft_id, title, problem, solution, claims, cpc_code, created_at
        FROM structured_patents WHERE patent_draft_id = $1
        "#,
        patent_id
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| {
        eprintln!("Erreur SQL get structured: {}", e);
        ErrorInternalServerError("Erreur récupération données")
    })?;

    match row {
        Some(r) => Ok(HttpResponse::Ok().json(StructuredPatent {
            id: r.id,
            patent_draft_id: r.patent_draft_id,
            title: r.title,
            problem: r.problem,
            solution: r.solution,
            claims: r.claims,
            cpc_code: r.cpc_code,
            created_at: r.created_at,
        })),
        None => Ok(HttpResponse::NotFound().json(json!({"message": "Non trouvé"}))),
    }
}




pub async fn verify_patent(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> ActixResult<HttpResponse> {
    let patent_id = path.into_inner();

    let draft_row = sqlx::query!(
        "SELECT id, status, hedera_tx_id FROM patent_drafts WHERE id = $1",
        patent_id
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| {
        eprintln!("Erreur SQL verify draft: {}", e);
        ErrorInternalServerError("Erreur récupération draft")
    })?;

    let Some(draft) = draft_row else {
        return Ok(HttpResponse::NotFound().json(json!({"status": "not_found", "message": "Draft non trouvé"})));
    };

    let Some(tx_id) = &draft.hedera_tx_id else {
        return Ok(HttpResponse::BadRequest().json(json!({"status": "pending", "message": "Pas encore sur blockchain"})));
    };

    let structured_row = sqlx::query!(
        "SELECT title, problem, solution, claims, cpc_code, created_at FROM structured_patents WHERE patent_draft_id = $1",
        patent_id
    )
    .fetch_one(pool.as_ref())
    .await
    .map_err(|e| {
        eprintln!("Erreur SQL verify structured: {}", e);
        ErrorInternalServerError("Erreur récupération structured")
    })?;

    let structured = StructuredPatent {
        id: Uuid::default(),
        patent_draft_id: patent_id,
        title: structured_row.title,
        problem: structured_row.problem,
        solution: structured_row.solution,
        claims: structured_row.claims,
        cpc_code: structured_row.cpc_code,
        created_at: structured_row.created_at,
    };

    let structured_data_json = serde_json::to_string(&structured).map_err(|e| {
        eprintln!("Erreur sérialisation verify: {}", e);
        ErrorInternalServerError("Erreur sérialisation")
    })?;

    let mut hasher = Sha256::new();
    hasher.update(structured_data_json.as_bytes());
    let calculated_hash = format!("{:x}", hasher.finalize());

    let is_valid_on_chain = match hedera_client::verify_hedera_message(tx_id, &calculated_hash).await {
        Ok(valid) => valid,
        Err(e) => {
            eprintln!("Erreur vérif Hedera: {}", e);
            false
        }
    };

    if is_valid_on_chain && PatentStatus::from_str(&draft.status).map_or(false, |s| matches!(s, PatentStatus::OnBlockchain)) {
        Ok(HttpResponse::Ok().json(json!({
            "status": "valid",
            "calculated_hash": calculated_hash,
            "hedera_tx_id": tx_id,
            "message": "Preuve d'antériorité valide sur blockchain"
        })))
    } else {
        Ok(HttpResponse::BadRequest().json(json!({
            "status": "invalid",
            "calculated_hash": calculated_hash,
            "hedera_tx_id": tx_id,
            "message": "Vérification échouée (hash ou status mismatch)"
        })))
    }
}



pub async fn list_user_drafts(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> ActixResult<HttpResponse> {
    let user_id = path.into_inner();
    let drafts = sqlx::query_as!(PatentDraft, "SELECT * FROM patent_drafts WHERE user_id = $1 ORDER BY created_at DESC", user_id)
        .fetch_all(pool.as_ref())
        .await
        .map_err(|e| {
            eprintln!("Erreur list drafts: {}", e);
            ErrorInternalServerError("Erreur liste drafts")
        })?;
    Ok(HttpResponse::Ok().json(drafts))
}