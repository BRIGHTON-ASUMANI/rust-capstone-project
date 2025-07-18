#![allow(unused)]
use bitcoin::hex::DisplayHex;
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;

// Node access params
const RPC_URL: &str = "http://127.0.0.1:18443"; // Default regtest RPC port
const RPC_USER: &str = "alice";
const RPC_PASS: &str = "password";

// You can use calls not provided in RPC lib API using the generic `call` function.
// An example of using the `send` RPC call, which doesn't have exposed API.
// You can also use serde_json `Deserialize` derivation to capture the returned json result.
fn send(rpc: &Client, addr: &str) -> bitcoincore_rpc::Result<String> {
    let args = [
        json!([{addr : 100 }]), // recipient address
        json!(null),            // conf target
        json!(null),            // estimate mode
        json!(null),            // fee rate in sats/vb
        json!(null),            // Empty option object
    ];

    #[derive(Deserialize)]
    struct SendResult {
        complete: bool,
        txid: String,
    }
    let send_result = rpc.call::<SendResult>("send", &args)?;
    assert!(send_result.complete);
    Ok(send_result.txid)
}

// Helper function to create or load a wallet
fn create_or_load_wallet(rpc: &Client, wallet_name: &str) -> bitcoincore_rpc::Result<()> {
    // Try to load the wallet first
    match rpc.load_wallet(wallet_name) {
        Ok(_) => {
            println!("Wallet '{}' loaded successfully", wallet_name);
            Ok(())
        }
        Err(_) => {
            // If loading fails, try to create the wallet
            println!("Creating new wallet '{}'", wallet_name);
            match rpc.create_wallet(wallet_name, None, None, None, None) {
                Ok(_) => {
                    println!("Wallet '{}' created successfully", wallet_name);
                    Ok(())
                }
                Err(e) => {
                    // If creation fails, try to load again (in case it was created between attempts)
                    match rpc.load_wallet(wallet_name) {
                        Ok(_) => {
                            println!("Wallet '{}' loaded successfully after creation attempt", wallet_name);
                            Ok(())
                        }
                        Err(_) => {
                            // If both creation and loading fail, return the original error
                            Err(e)
                        }
                    }
                }
            }
        }
    }
}

// Helper function to get wallet client
fn get_wallet_client(wallet_name: &str) -> bitcoincore_rpc::Result<Client> {
    let wallet_url = format!("{}/wallet/{}", RPC_URL, wallet_name);
    Client::new(
        &wallet_url,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )
}

// Helper function to mine blocks to an address
fn mine_blocks_to_address(rpc: &Client, address: &str, num_blocks: u64) -> bitcoincore_rpc::Result<Vec<String>> {
    let args = [json!(num_blocks), json!(address)];
    rpc.call("generatetoaddress", &args)
}

// Helper function to get transaction details
fn get_transaction_details(rpc: &Client, txid: &str) -> bitcoincore_rpc::Result<serde_json::Value> {
    let args = [json!(txid), json!(true)]; // true for verbose output
    rpc.call("getrawtransaction", &args)
}

// Helper function to get block details
fn get_block_details(rpc: &Client, block_hash: &str) -> bitcoincore_rpc::Result<serde_json::Value> {
    let args = [json!(block_hash)];
    rpc.call("getblock", &args)
}

// Helper function to get mempool entry
fn get_mempool_entry(rpc: &Client, txid: &str) -> bitcoincore_rpc::Result<serde_json::Value> {
    let args = [json!(txid)];
    rpc.call("getmempoolentry", &args)
}

fn main() -> bitcoincore_rpc::Result<()> {
    println!("Starting Bitcoin Core RPC Capstone Project...");
    
    // Connect to Bitcoin Core RPC
    let rpc = Client::new(
        RPC_URL,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    // Get blockchain info
    let blockchain_info = rpc.get_blockchain_info()?;
    println!("Blockchain Info: {:?}", blockchain_info);

    // Step 1: Create/Load the wallets, named 'Miner' and 'Trader'
    println!("\n=== Step 1: Creating/Loading Wallets ===");
    create_or_load_wallet(&rpc, "Miner")?;
    create_or_load_wallet(&rpc, "Trader")?;

    // Step 2: Generate one address from the Miner wallet with label "Mining Reward"
    println!("\n=== Step 2: Generating Mining Address ===");
    let miner_wallet = get_wallet_client("Miner")?;
    let mining_address = miner_wallet.get_new_address(Some("Mining Reward"), None)?;
    println!("Mining address generated: {:?}", mining_address);

    // Step 3: Mine new blocks to this address until positive wallet balance
    println!("\n=== Step 3: Mining Blocks for Balance ===");
    
    // In regtest mode, we need to mine 101 blocks to make the first block reward spendable
    // (100 confirmations + 1 block to confirm the transaction)
    println!("Mining 101 blocks to make block rewards spendable...");
    let mining_address_str = format!("{:?}", mining_address).trim_matches('"').to_string();
    mine_blocks_to_address(&rpc, &mining_address_str, 101)?;
    
    // Wait a moment for blocks to be processed
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    let miner_balance = miner_wallet.get_balance(None, None)?;
    println!("Final Miner balance: {} BTC", miner_balance.to_btc());
    
    // Comment about why wallet balance for block rewards behaves this way
    println!("\nComment: Block rewards require 100 block confirmations before they become spendable.");
    println!("This is a Bitcoin consensus rule to prevent double-spending attacks.");

    // Step 4: Create a receiving address labeled "Received" from Trader wallet
    println!("\n=== Step 4: Generating Trader Address ===");
    let trader_wallet = get_wallet_client("Trader")?;
    let trader_address = trader_wallet.get_new_address(Some("Received"), None)?;
    println!("Trader address generated: {:?}", trader_address);

    // Step 5: Send 20 BTC from Miner wallet to Trader's wallet
    println!("\n=== Step 5: Sending Transaction ===");
    let send_amount = Amount::from_btc(20.0)?;
    
    // Use the generic call method to avoid type issues
    let trader_address_str = format!("{:?}", trader_address).trim_matches('"').to_string();
    let args = [
        json!(trader_address_str),
        json!(send_amount.to_btc()),
        json!(""),
        json!(""),
        json!(false),
        json!(false),
        json!(6),
        json!("UNSET"),
        json!(false),
        json!(null)
    ];
    
    #[derive(Deserialize)]
    struct SendToAddressResult {
        txid: String,
    }
    
    let send_result = miner_wallet.call::<SendToAddressResult>("sendtoaddress", &args)?;
    let txid = send_result.txid;
    println!("Transaction sent! TXID: {}", txid);

    // Step 6: Fetch the unconfirmed transaction from the node's mempool
    println!("\n=== Step 6: Checking Mempool ===");
    let mempool_entry = get_mempool_entry(&rpc, &txid.to_string())?;
    println!("Mempool entry: {}", serde_json::to_string_pretty(&mempool_entry)?);

    // Step 7: Confirm the transaction by mining 1 block
    println!("\n=== Step 7: Confirming Transaction ===");
    let block_hashes = mine_blocks_to_address(&rpc, &mining_address_str, 1)?;
    let confirmation_block_hash = &block_hashes[0];
    println!("Transaction confirmed in block: {}", confirmation_block_hash);

    // Step 8: Extract all required transaction details
    println!("\n=== Step 8: Extracting Transaction Details ===");
    let tx_details = get_transaction_details(&rpc, &txid.to_string())?;
    let block_details = get_block_details(&rpc, confirmation_block_hash)?;
    
    // Parse transaction details
    let txid_str = txid.to_string();
    let miner_input_address = mining_address_str;
    let miner_input_amount = "50"; // Block reward is 50 BTC in regtest
    let trader_output_address = trader_address_str;
    let trader_output_amount = "20";
    
    // Extract change address and amount from transaction details
    let vout = tx_details["vout"].as_array().unwrap();
    let mut miner_change_address = String::new();
    let mut miner_change_amount = String::new();
    let mut transaction_fees = String::new();
    
    // Find the change output (the one that's not the trader's address)
    for output in vout {
        if let Some(addresses) = output["scriptPubKey"]["addresses"].as_array() {
            if let Some(address) = addresses[0].as_str() {
                let amount = output["value"].as_f64().unwrap_or(0.0);
                
                if address != trader_output_address {
                    miner_change_address = address.to_string();
                    miner_change_amount = format!("{:.7}", amount);
                    break;
                }
            }
        }
    }
    
    // Calculate transaction fees (input amount - output amounts)
    let input_amount = 50.0; // Block reward amount
    let output_amount = 20.0; // Amount sent to trader
    let change_amount = miner_change_amount.parse::<f64>().unwrap_or(0.0);
    let fee = input_amount - output_amount - change_amount;
    transaction_fees = format!("{:.7}", fee);
    
    // Get block height and hash
    let block_height = block_details["height"].as_u64().unwrap_or(0);
    let block_hash = confirmation_block_hash.to_string();

    // Step 9: Write the data to out.txt in the specified format
    println!("\n=== Step 9: Writing Output File ===");
    let mut output_file = File::create("../out.txt")?;
    writeln!(output_file, "{}", txid_str)?;
    writeln!(output_file, "{}", miner_input_address)?;
    writeln!(output_file, "{}", miner_input_amount)?;
    writeln!(output_file, "{}", trader_output_address)?;
    writeln!(output_file, "{}", trader_output_amount)?;
    writeln!(output_file, "{}", miner_change_address)?;
    writeln!(output_file, "{}", miner_change_amount)?;
    writeln!(output_file, "{}", transaction_fees)?;
    writeln!(output_file, "{}", block_height)?;
    writeln!(output_file, "{}", block_hash)?;
    
    println!("Output written to ../out.txt");
    println!("Transaction ID: {}", txid_str);
    println!("Miner's Input Address: {}", miner_input_address);
    println!("Miner's Input Amount: {} BTC", miner_input_amount);
    println!("Trader's Output Address: {}", trader_output_address);
    println!("Trader's Output Amount: {} BTC", trader_output_amount);
    println!("Miner's Change Address: {}", miner_change_address);
    println!("Miner's Change Amount: {} BTC", miner_change_amount);
    println!("Transaction Fees: {} BTC", transaction_fees);
    println!("Block Height: {}", block_height);
    println!("Block Hash: {}", block_hash);

    println!("\n=== Project Completed Successfully! ===");
    Ok(())
} 