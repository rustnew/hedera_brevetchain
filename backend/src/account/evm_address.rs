
// Hedera SDK v0.33.0
use hedera::{AccountId, Client, PrivateKey, Hbar, TransferTransaction};
use std::{env};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {


    let my_account_id: AccountId = env::var("HEDERA_ACCOUNT_ID")?.parse()?;
    let my_private_key: PrivateKey = env::var("HEDERA_PRIVATE_KEY")?.parse()?;

    //Pre-configured client for test network (testnet)
    let client = Client::for_testnet();

    //Set the operator with the account ID and private key
    client.set_operator(my_account_id, my_private_key.clone());

            
    //Create a transaction to transfer 1 HBAR
    let mut tx_transfer = TransferTransaction::new();
    tx_transfer
      .hbar_transfer(my_account_id, Hbar::new(-1))
      .hbar_transfer(my_account_id, Hbar::new(1)); //Fill in the receiver account ID

    //Submit the transaction to a Hedera network
    let tx_transfer_response = tx_transfer.execute(&client).await?;

    //Request the receipt of the transaction
    let receipt_transfer_tx = tx_transfer_response.get_receipt(&client).await?;

    //Get the transaction consensus status
    let status_transfer_tx = receipt_transfer_tx.status;
    
    //Get the Transaction ID
    let tx_id_transfer = tx_transfer_response.transaction_id.to_string();

    println!("-------------------------------- Transfer HBAR ------------------------------ ");
    println!("Receipt status       : {:?}", status_transfer_tx);
    println!("Transaction ID       : {:?}", tx_id_transfer);
    println!("Hashscan URL         : https://hashscan.io/testnet/tx/{tx_id_transfer}");

    //Start your code here

    Ok(())
}