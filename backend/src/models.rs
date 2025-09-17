use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub wallet_address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterUserRequest {
    pub full_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub wallet_address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterUserResponse {
    pub user_id: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Idea {
    pub id: Uuid,
    pub user_id: Uuid,
    pub raw_idea: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubmitIdeaRequest {
    pub user_id: Uuid, // ✅ Obligatoire — l'utilisateur doit être enregistré
    pub raw_idea: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubmitIdeaResponse {
    pub idea_id: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Summary {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub title: String,
    pub problem: String,
    pub solution: String,
    pub claim: String, // ✅ MVP: une seule revendication principale
    pub cpc_code: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiResponse {
    pub title: String,
    pub problem: String,
    pub solution: String,
    pub claim: String, // ✅ Simplifié pour MVP
    pub cpc_code: String,
    pub novelty_score: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Proof {
    pub id: Uuid,
    pub summary_id: Uuid,
    pub hash: String,
    pub hedera_tx_id: String,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CertificateResponse {
    pub hash: String,
    pub timestamp: String,
    pub hedera_tx_id: String,
    pub explorer_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusResponse {
    pub idea_received: bool,
    pub ia_summary_ready: bool,
    pub hedera_proof_registered: bool,
    pub agent_validated: bool, // ✅ Toujours false dans MVP — placeholder
    pub office_submitted: bool, // ✅ Toujours false dans MVP — placeholder
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthResponse {
    pub status: String,
    pub services: Vec<String>,
}