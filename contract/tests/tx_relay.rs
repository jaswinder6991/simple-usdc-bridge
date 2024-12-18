//run using
//cargo run --bin tx_relay

//use ethers::ethers_middleware::Middleware;
//use ethers::ethers_middleware::Middleware;
use ethers::providers::{Http, Middleware, Provider};
//use ethers::types::U256;
use eyre::Result;
use omni_transaction::evm::types::Signature as OmniSignature;
use omni_transaction::evm::utils::parse_eth_address;
use omni_transaction::transaction_builder::{
    TransactionBuilder as OmniTransactionBuilder, TxBuilder,
};
use omni_transaction::types::EVM;
//use std::str::FromStr;

//100000000000000000 0.1
//20000000000000000

#[tokio::test]
async fn test_send_raw_transaction_created_with_omnitransactionbuilder_for_evm() -> Result<()> {
    // Sepolia testnet provider
    let provider = Provider::<Http>::try_from("https://rpc.sepolia.org/")?;

    let nonce: u64 = 0; // Make sure to use the correct nonce for your account
    let to_address_str = "8530748420d2A67D59E1fb0d2f33b71B51Ff3965";
    let to_address = parse_eth_address(to_address_str);
    let usdc_address = parse_eth_address("1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"); // Sepolia USDC contract address

    let data = construct_erc20_transfer_data(to_address, 1);

    // Build the transaction using OmniTransactionBuilder
    let omni_evm_tx = OmniTransactionBuilder::new::<EVM>()
        .nonce(nonce)
        .to(usdc_address)
        .value(0)
        .input(data)
        .max_priority_fee_per_gas(1_500_000_000)
        .max_fee_per_gas(20_000_000_000)
        .gas_limit(500_000)
        .chain_id(11155111) // replace with Sepolia chain ID
        .build();

    //10000000000000000
    //1950000000000000

    //0.001

    //0.000195

    // Encode the transaction
    //let omni_evm_tx_encoded = omni_evm_tx.build_for_signing();

    // Create OmniSignature from the MPC contract response
    let signature_omni = OmniSignature {
        v: 0, // 27 + recovery_id (1 in this case)
        r: hex::decode("71E5F3EE59A6CF387DD0D292454680B7B8E48E1B85FD40E03DE306F80498AD23").unwrap(),
        s: hex::decode("5D137EA74234FA722AB0BDE5304A7754F0D685A72D8EEB24C8B7F7BBF783ED9D").unwrap(),
    };

    let omni_evm_tx_encoded_with_signature = omni_evm_tx.build_with_signature(&signature_omni);

    // Send the transaction
    match provider
        .send_raw_transaction(omni_evm_tx_encoded_with_signature.into())
        .await
    {
        Ok(tx_hash) => println!("Transaction sent successfully. Hash: {:?}", tx_hash),
        Err(e) => println!("Failed to send transaction: {:?}", e),
    }

    Ok(())
}

fn construct_erc20_transfer_data(to: [u8; 20], amount: u128) -> Vec<u8> {
    let mut data = Vec::new();
    // Function selector for "transfer(address,uint256)"
    data.extend_from_slice(&[0xa9, 0x05, 0x9c, 0xbb]);
    // Pad the 'to' address to 32 bytes
    data.extend_from_slice(&[0; 12]);
    data.extend_from_slice(&to);
    // Pad the amount to 32 bytes
    data.extend_from_slice(&[0; 16]);
    data.extend_from_slice(&amount.to_be_bytes());
    data
}

// USDC treasury account Eth: 0x349f2bc4138b0f66871f8d734cef2e1b6c36ddc9 // signer-dev
// 0xd6ceeffa721575c53181346b6cc49647167085c7 // signer-prod

// succesful tx where treasury had USDC as well
//---- test_send_raw_transaction_created_with_omnitransactionbuilder_for_evm stdout ----
//Transaction sent successfully. Hash: PendingTransaction { tx_hash: 0x8e7de1833d1b5b5d09d2393123295a04b92e494fa9f629cd4b316f8f6a824a50, confirmations: 1, state: PendingTxState { state: "InitialDelay" } }
