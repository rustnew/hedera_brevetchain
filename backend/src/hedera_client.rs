use hedera::{
    Client, TopicMessageSubmitTransaction, PrivateKey, TopicId, Hbar, AccountId, Status,
    TopicMessageQuery, TopicMessage,
};
use std::env;
use chrono::Utc;
use hex;
use serde_json::json;

pub async fn submit_to_hedera(
    hash: String,
    cpc_code: String,
    user_wallet: String,
    created_at: chrono::DateTime<Utc>,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::for_testnet();
    let private_key_hex = env::var("HEDERA_PRIVATE_KEY")?;
    let private_key_bytes = hex::decode(&private_key_hex)?;
    let private_key = PrivateKey::from_bytes_der(&private_key_bytes)?;
    let operator_account_id_str = env::var("HEDERA_ACCOUNT_ID")?;
    let operator_account_id: AccountId = operator_account_id_str.parse()?;
    client.set_operator(operator_account_id, private_key.clone());

    let topic_id_str = env::var("HEDERA_TOPIC_ID")?;
    let topic_id: TopicId = topic_id_str.parse()?;

    let message = json!({
        "hash": hash,
        "cpc_code": cpc_code,
        "user_wallet": user_wallet,
        "created_at": created_at.to_rfc3339(),
    }).to_string();

    let mut attempts = 0;
    loop {
        attempts += 1;
        let response = TopicMessageSubmitTransaction::new()
            .topic_id(topic_id)
            .message(message.as_bytes())
            .max_transaction_fee(Hbar::from_tinybars(1_000_000))
            .sign(private_key.clone())
            .execute(&client)
            .await?;

        let receipt = response.get_receipt(&client).await?;
        if receipt.status == Status::Success {
            return Ok(receipt.transaction_id.map(|id| id.to_string()).unwrap_or_else(|| "no_tx_id".to_string()));
        }
        eprintln!("Hedera tx failed (attempt {}): {:?}", attempts, receipt.status);
        if attempts >= 3 {
            return Err(format!("Hedera submission failed after 3 attempts: {:?}", receipt.status).into());
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

pub async fn verify_hedera_message(_tx_id: &str, expected_hash: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let client = Client::for_testnet();
    let topic_id_str = env::var("HEDERA_TOPIC_ID")?;
    let topic_id: TopicId = topic_id_str.parse()?;

    // Fix: Chaîner les méthodes sans réassignation
    let mut query = TopicMessageQuery::new();
    query.topic_id(topic_id).limit(1u64); // Modification en place
    let messages: Vec<TopicMessage> = query.execute(&client).await?;

    for msg in messages {
        let msg_bytes = msg.contents;
        if let Ok(msg_json) = serde_json::from_slice::<serde_json::Value>(&msg_bytes) {
            if let Some(stored_hash) = msg_json["hash"].as_str() {
                if stored_hash == expected_hash {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)

   
}