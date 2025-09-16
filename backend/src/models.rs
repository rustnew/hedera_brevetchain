use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct PatentDraft {
    pub id: Uuid,
    pub user_id: Uuid,
    pub raw_idea: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub hedera_tx_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PatentStatus {
    Draft,
    Submitted,
    Rejected,
    OnBlockchain,
}

impl PatentStatus {
    pub fn from_str(status: &str) -> Result<Self, String> {
        match status.to_lowercase().as_str() {
            "draft" => Ok(PatentStatus::Draft),
            "submitted" => Ok(PatentStatus::Submitted),
            "rejected" => Ok(PatentStatus::Rejected),
            "on_blockchain" => Ok(PatentStatus::OnBlockchain),
            _ => Err(format!("Statut invalide: {}", status)),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PatentStatus::Draft => "draft",
            PatentStatus::Submitted => "submitted",
            PatentStatus::Rejected => "rejected",
            PatentStatus::OnBlockchain => "on_blockchain",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubmitIdeaRequest {
    pub user: UserInfo,
    pub patent: PatentInput,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub full_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub wallet_address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatentInput {
    pub raw_idea: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiResponse {
    pub title: String,
    pub problem: String,
    pub solution: String,
    pub claims: Vec<String>,
    pub cpc_code: String,
    pub novelty_score: u8,
}

#[derive(Debug, Serialize, Clone, FromRow)]
pub struct StructuredPatent {
    pub id: Uuid,
    pub patent_draft_id: Uuid,
    pub title: String,
    pub problem: String,
    pub solution: String,
    pub claims: Value,
    pub cpc_code: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SubmitIdeaResponse {
    pub patent_id: Uuid,
    pub message: String,
    pub structured_patent: Option<StructuredPatent>,
}
