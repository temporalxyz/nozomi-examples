use std::{env, time::Instant};

use jupiter_swap_api_client::{
    quote::QuoteRequest,
    swap::{SwapInstructionsResponse, SwapRequest},
    transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    address_lookup_table::{
        state::AddressLookupTable, AddressLookupTableAccount,
    },
    commitment_config::CommitmentConfig,
    message::{v0, VersionedMessage},
    native_token::sol_to_lamports,
    pubkey,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::transfer,
    transaction::VersionedTransaction,
};

const JUPITER_URL: &str = "https://quote-api.jup.ag/v6";
const NOZOMI_URL: &str = "http://ams1.nozomi.temporal.xyz";

const NATIVE_MINT: Pubkey =
    pubkey!("So11111111111111111111111111111111111111112");
const USDC_MINT: Pubkey =
    pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const NOZOMI_TIP_ADDRESS: Pubkey =
    pubkey!("TEMPaMeCRFAS9EKF53Jd6KpHxgL47uWLcpFArU1Fanq");

const NOZOMI_TIP_SOL: f64 = 0.001;

#[tokio::main]
async fn main() {
    let nozomi_uuid =
        env::var("NOZOMI_UUID").expect("NOZOMI_UUID not set");
    let keypair_str =
        env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");

    let rpc_client = RpcClient::new_with_commitment(
        env::var("RPC_URL")
            .unwrap_or("https://api.mainnet-beta.solana.com".into()),
        CommitmentConfig::confirmed(),
    );
    let nozomi_rpc_client =
        RpcClient::new(format!("{NOZOMI_URL}?c={nozomi_uuid}"));
    let jupiter_swap_api_client =
        JupiterSwapApiClient::new(JUPITER_URL.into());

    let wallet = Keypair::from_base58_string(&keypair_str);

    // Swapping SOL to USDC with input 0.1 SOL and 0.5% slippage
    let quote_request = QuoteRequest {
        amount: sol_to_lamports(0.1),
        input_mint: NATIVE_MINT,
        output_mint: USDC_MINT,
        slippage_bps: 50,
        ..QuoteRequest::default()
    };

    let quote_response = jupiter_swap_api_client
        .quote(&quote_request)
        .await
        .expect("Failed to get quote");

    // get serialized transactions for the swap
    let SwapInstructionsResponse {
        token_ledger_instruction,
        compute_budget_instructions,
        setup_instructions,
        swap_instruction,
        address_lookup_table_addresses,
        cleanup_instruction,
        other_instructions,
        ..
    } = jupiter_swap_api_client
        .swap_instructions(&SwapRequest {
            user_public_key: wallet.pubkey(),
            quote_response: quote_response.clone(),
            config: TransactionConfig::default(),
        })
        .await
        .expect("Failed to get swap instructions");

    let nozomi_tip_ix = transfer(
        &wallet.pubkey(),
        &NOZOMI_TIP_ADDRESS,
        sol_to_lamports(NOZOMI_TIP_SOL),
    );

    let addres_lookup_table_accounts = rpc_client
        .get_multiple_accounts(&address_lookup_table_addresses)
        .await
        .unwrap()
        .into_iter()
        .zip(address_lookup_table_addresses)
        .map(|(maybe_account, key)| {
            let account = maybe_account.expect("Failed to get account");

            let address_lookup_table =
                AddressLookupTable::deserialize(&account.data)
                    .expect("Failed to deserialize");

            AddressLookupTableAccount {
                key,
                addresses: address_lookup_table.addresses.to_vec(),
            }
        })
        .collect::<Vec<_>>();

    // get the latest block hash
    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("Failed to get blockhash");

    let ixs = token_ledger_instruction
        .into_iter()
        .chain(compute_budget_instructions.into_iter())
        .chain(setup_instructions.into_iter())
        .chain([swap_instruction, nozomi_tip_ix].into_iter())
        .chain(other_instructions.into_iter())
        .chain(cleanup_instruction.into_iter())
        .collect::<Vec<_>>();

    // sign the transaction
    let signed_versioned_transaction = VersionedTransaction::try_new(
        VersionedMessage::V0(
            v0::Message::try_compile(
                &wallet.pubkey(),
                &ixs,
                &addres_lookup_table_accounts,
                blockhash,
            )
            .expect("Failed to compile message"),
        ),
        &[wallet],
    )
    .expect("Failed to sign transaction");

    let start = Instant::now();
    let sig = nozomi_rpc_client
        .send_transaction(&signed_versioned_transaction)
        .await
        .expect("Failed to send transaction");

    println!("Nozomi response: txid: {}", sig);

    loop {
        let confirmed = rpc_client
            .confirm_transaction(&sig)
            .await
            .expect("Failed to confirm transaction");
        if confirmed {
            break;
        }
    }
    println!("Confirmed in: {} seconds", start.elapsed().as_secs());
}
