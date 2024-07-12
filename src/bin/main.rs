use futures_util::StreamExt;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcTransactionLogsFilter;
use solana_client::rpc_config::RpcTransactionLogsConfig;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::signature::Signature;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiTransactionEncoding;
use solana_transaction_status::UiMessage;
use solana_transaction_status::EncodedTransaction;
use spl_token::state::Mint;
use solana_program::program_pack::Pack;
use std::str::FromStr;
use dotenv::dotenv;
use std::env;
use bs58;

#[tokio::main]
async fn main() {

    dotenv().ok();

    let ws_url = env::var("RPC_WEBSOCKET_ENDPOINT").unwrap();
    let ws_client = PubsubClient::new(&ws_url).await.unwrap();

    // Define the filter for the specific smart contract address
    let filter = RpcTransactionLogsFilter::Mentions(vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string()]);
    let config = RpcTransactionLogsConfig { commitment: Some(CommitmentConfig {
        commitment: CommitmentLevel::Confirmed,
    }) };

    // Subscribe to logs
    let (mut logs_stream, _unsubscribe) = ws_client
        .logs_subscribe(filter, config)
        .await.unwrap();

    // Process incoming logs
    while let Some(logs) = logs_stream.next().await {
        // Iterate through logs and find the token creation event
        for log in logs.value.logs {

            if log == "Program log: Instruction: Create" {
                if let Some(_mint_address) = fetch_mint_address_from_transaction(&logs.value.signature).await {
                    println!("");
                }
                break;
            }

        }
    }

}

// Function to fetch the mint address from the transaction details
async fn fetch_mint_address_from_transaction(tx_signature: &String) -> Option<String> {

    // Create an RPC client to fetch transaction details
    let rpc_url = env::var("RPC_ENDPOINT").unwrap();
    let rpc_client = RpcClient::new(rpc_url);

    // Decode the base58-encoded signature string to bytes
    let tx_signature_bytes = match bs58::decode(tx_signature).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => return None, // Return None if decoding fails
    };

    // Ensure the byte slice has exactly 64 bytes (required by Solana's Signature type)
    if tx_signature_bytes.len() != 64 {
        return None; // Early return if the signature length is not correct
    }

    // Create a fixed-size array of 64 bytes
    let mut signature_array = [0u8; 64];
    signature_array.copy_from_slice(&tx_signature_bytes);

    // config
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Json),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };


    // Fetch the transaction details using the signature
    if let Ok(transaction) = rpc_client.get_transaction_with_config(
        &Signature::from(signature_array),
        config
    ) {
        // Parse the transaction details to find the mint address
        // The exact implementation depends on the transaction format and the structure of the token creation

        match &transaction.transaction.transaction {
            EncodedTransaction::Json(ui_transaction) => {
                match &ui_transaction.message {
                    UiMessage::Raw(raw_message) => {
                        if let Some(mint_address) = raw_message.account_keys.get(1) {
                            println!("New token {} created.", mint_address);
                            extract_token_info(&rpc_client, mint_address.to_string()).await;
                            return Some(mint_address.to_string());
                        } else {
                            return None;
                        }
                    },
                    _ => {
                        return None;
                    }
                }
            },
            _ => {
                return None;
            }
        }
    }
    None
}

// Function to extract the token info from mint address
async fn extract_token_info(rpc_client: &RpcClient, mint_address: String) {

    let token_mint_pubkey = Pubkey::from_str(&mint_address).unwrap();

    match rpc_client.get_account(&token_mint_pubkey) {
        Ok(account) => {
            // Parse the account data to get token mint information
            if let Ok(mint) = Mint::unpack(&account.data) {
                println!("Mint Authority: {:?}", mint.mint_authority);
                println!("Supply: {:?}", mint.supply);
                println!("Decimals: {:?}", mint.decimals);
                println!("Is Initialized: {:?}", mint.is_initialized);
                println!("Freeze Authority: {:?}", mint.freeze_authority);
            } else {
                eprintln!("Failed to unpack mint account data");
            }
        },
        Err(err) => eprintln!("Failed to fetch token mint account: {:?}", err),
    }
}