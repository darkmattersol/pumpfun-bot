use futures_util::StreamExt;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcTransactionLogsFilter;
use solana_client::rpc_config::RpcTransactionLogsConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_transaction_status::UiMessage;
use solana_sdk::signature::Signature;
use solana_client::rpc_client::RpcClient;
use solana_transaction_status::UiTransactionEncoding;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_transaction_status::EncodedTransaction;
use std::env;
use dotenv::dotenv;
use fern::colors::{Color, ColoredLevelConfig};
use chrono::Local;
use log::{info};

#[tokio::main]
async fn main() {

    dotenv().ok();

    console_color_initialize();

    let ws_url = env::var("RPC_WEBSOCKET_ENDPOINT").unwrap(); // ws url
    let program_id_watch = env::var("PROGRAM_PUBLIC_KEY").unwrap(); // program id
    let watched_wallet_address = env::var("WATCHED_WALLET_ADDRESS").unwrap(); // watch wallet address

    // ws client
    let ws_client = PubsubClient::new(&ws_url).await.unwrap();

    let filter = RpcTransactionLogsFilter::Mentions(vec![program_id_watch.to_string()]);
    let config = RpcTransactionLogsConfig { commitment: Some(CommitmentConfig {
        commitment: CommitmentLevel::Confirmed,
    }) };

    let (mut subscription, _unsubscribe) = ws_client
        .logs_subscribe(filter, config)
        .await.unwrap();

    info!("Monitoring started, fetching the buy/sell transaction for wallet address {}", watched_wallet_address);

    while let Some(logs) = subscription.next().await {

        for log in &logs.value.logs {
            if log == "Program log: Instruction: Buy" {
                if let Some(mint_address) = fetch_mint_address_from_transaction(&logs.value.signature.to_string()).await {
                    info!("detected a buy transaction. Sig: {}", logs.value.signature);
                    info!("token mint address: {}", mint_address);
                }
                break;
            }
            else if log == "Program log: Instruction: Sell" {
                if let Some(mint_address) = fetch_mint_address_from_transaction(&logs.value.signature.to_string()).await {
                    info!("detected a sell transaction. Sig: {}", logs.value.signature);
                    info!("token mint address: {}", mint_address);
                }
                break;
            }

        }

    }


}


fn console_color_initialize() {
    let colors = ColoredLevelConfig::new()
    .error(Color::Red)
    .warn(Color::Yellow)
    .info(Color::Green)
    .debug(Color::Magenta);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

}

async fn fetch_mint_address_from_transaction(tx_signature: &String) -> Option<String> {

    // Create an RPC client to fetch transaction details
    let rpc_url = env::var("RPC_ENDPOINT").unwrap();
    let rpc_client = RpcClient::new(rpc_url);

    // wallet address
    let wallet_address = env::var("WATCHED_WALLET_ADDRESS").unwrap();

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
        match &transaction.transaction.transaction {
            EncodedTransaction::Json(ui_transaction) => {
                match &ui_transaction.message {
                    UiMessage::Raw(raw_message) => {
                        for address1 in &raw_message.account_keys {
                            // check wallet address
                            if address1 == &wallet_address.to_string() {
                                for address2 in &raw_message.account_keys {
                                    // Check token
                                    if address2.ends_with("pump") {
                                        return Some(address2.to_string());
                                    }
                                }
                            }
                        }
                        return None;
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
}use futures_util::StreamExt;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcTransactionLogsFilter;
use solana_client::rpc_config::RpcTransactionLogsConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_transaction_status::UiMessage;
use solana_sdk::signature::Signature;
use solana_client::rpc_client::RpcClient;
use solana_transaction_status::UiTransactionEncoding;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_transaction_status::EncodedTransaction;
use std::env;
use dotenv::dotenv;
use fern::colors::{Color, ColoredLevelConfig};
use chrono::Local;
use log::{info};

#[tokio::main]
async fn main() {

    dotenv().ok();

    console_color_initialize();

    let ws_url = env::var("RPC_WEBSOCKET_ENDPOINT").unwrap(); // ws url
    let program_id_watch = env::var("PROGRAM_PUBLIC_KEY").unwrap(); // program id
    let watched_wallet_address = env::var("WATCHED_WALLET_ADDRESS").unwrap(); // watch wallet address

    // ws client
    let ws_client = PubsubClient::new(&ws_url).await.unwrap();

    let filter = RpcTransactionLogsFilter::Mentions(vec![program_id_watch.to_string()]);
    let config = RpcTransactionLogsConfig { commitment: Some(CommitmentConfig {
        commitment: CommitmentLevel::Confirmed,
    }) };

    let (mut subscription, _unsubscribe) = ws_client
        .logs_subscribe(filter, config)
        .await.unwrap();

    info!("Monitoring started, fetching the buy/sell transaction for wallet address {}", watched_wallet_address);

    while let Some(logs) = subscription.next().await {

        for log in &logs.value.logs {
            if log == "Program log: Instruction: Buy" {
                if let Some(mint_address) = fetch_mint_address_from_transaction(&logs.value.signature.to_string()).await {
                    info!("detected a buy transaction. Sig: {}", logs.value.signature);
                    info!("token mint address: {}", mint_address);
                }
                break;
            }
            else if log == "Program log: Instruction: Sell" {
                if let Some(mint_address) = fetch_mint_address_from_transaction(&logs.value.signature.to_string()).await {
                    info!("detected a sell transaction. Sig: {}", logs.value.signature);
                    info!("token mint address: {}", mint_address);
                }
                break;
            }

        }

    }


}


fn console_color_initialize() {
    let colors = ColoredLevelConfig::new()
    .error(Color::Red)
    .warn(Color::Yellow)
    .info(Color::Green)
    .debug(Color::Magenta);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

}

async fn fetch_mint_address_from_transaction(tx_signature: &String) -> Option<String> {

    // Create an RPC client to fetch transaction details
    let rpc_url = env::var("RPC_ENDPOINT").unwrap();
    let rpc_client = RpcClient::new(rpc_url);

    // wallet address
    let wallet_address = env::var("WATCHED_WALLET_ADDRESS").unwrap();

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
        match &transaction.transaction.transaction {
            EncodedTransaction::Json(ui_transaction) => {
                match &ui_transaction.message {
                    UiMessage::Raw(raw_message) => {
                        for address1 in &raw_message.account_keys {
                            // check wallet address
                            if address1 == &wallet_address.to_string() {
                                for address2 in &raw_message.account_keys {
                                    // Check token
                                    if address2.ends_with("pump") {
                                        return Some(address2.to_string());
                                    }
                                }
                            }
                        }
                        return None;
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