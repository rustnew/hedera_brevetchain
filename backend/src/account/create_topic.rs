use hedera::{
    Client, Hbar, PrivateKey, AccountId, TopicCreateTransaction,
    TopicId, TransactionResponse, TransactionReceipt
};

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration du client
    let client = Client::for_testnet();

    
    let account_id: AccountId = env::var("HEDERA_ACCOUNT_ID")?.parse()?;
    let private_key: PrivateKey = env::var("HEDERA_PRIVATE_KEY")?.parse()?;

    client.set_operator(account_id, private_key.clone());

    // Créer le topic
    let transaction: TransactionResponse = TopicCreateTransaction::new()
        .topic_memo("Mon topic Rust".to_string())
        .admin_key(private_key.public_key()) // Optionnel
        .submit_key(private_key.public_key()) // Optionnel
        .max_transaction_fee(Hbar::new(2))
        .execute(&client)
        .await?;

    // Obtenir le reçu
    let receipt: TransactionReceipt = transaction.get_receipt(&client).await?;
    
    // Récupérer le Topic ID
    let topic_id: TopicId = receipt.topic_id.expect("Topic ID manquant");
    
    println!("✅ Topic créé avec succès !");
    println!("📋 Topic ID: {}", topic_id);
    

    Ok(())
}