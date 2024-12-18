use crate::models::EVMTransactionWrapper;
use crate::signer::{mpc, SignRequest, SignResult};
use crate::{Contract, ContractExt, NetworkDetails};
use near_sdk::{env, json_types::U128, near, Gas, NearToken, Promise, PromiseError};
use omni_transaction::evm::evm_transaction::EVMTransaction;
use omni_transaction::evm::types::Signature as OmniSignature;
use omni_transaction::evm::utils::parse_eth_address;
use omni_transaction::transaction_builder::TransactionBuilder;
use omni_transaction::transaction_builder::TxBuilder;
use omni_transaction::types::EVM;

const MPC_CONTRACT_ACCOUNT_ID: &str = "v1.signer-prod.testnet";
//const USDC_NEAR_ADDRESS: &str = "3e2210e1184b45b64c8a434c0a7e7b23cc04ea7eb7a6c3c32520d03d4afcb8af";
const USDC_ETH_ADDRESS: &str = "1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"; // USDC on Ethereum Sepolia
const USDC_TREASURY_PATH: &str = "usdc-treasury";

#[near]
impl Contract {
    #[private]
    pub fn transfer_and_sign_erc20(
        &mut self,
        eth_recipient: String,
        amount: U128,
        network_details: NetworkDetails,
    ) -> Promise {
        env::log_str(
            format!(
                "Transferring {} USDC to {} with transaction nonce {}.",
                amount.0, eth_recipient, network_details.eth_nonce
            )
            .as_str(),
        );
        let omni_evm_tx = self.construct_erc20_transfer_tx(
            USDC_ETH_ADDRESS.to_string(),
            eth_recipient,
            amount.0,
            network_details,
        );

        // Encode the transaction with EIP-1559 prefix
        let omni_evm_tx_encoded = omni_evm_tx.build_for_signing();

        // Hash the encoded transaction
        let omni_evm_tx_hash = env::keccak256(&omni_evm_tx_encoded);

        let promise = mpc::ext(MPC_CONTRACT_ACCOUNT_ID.parse().unwrap())
            .with_static_gas(Gas::from_tgas(100))
            .with_attached_deposit(NearToken::from_yoctonear(200000000000000000000000))
            .sign(SignRequest {
                payload: omni_evm_tx_hash,
                path: USDC_TREASURY_PATH.to_string(),
                key_version: 0,
            });

        return promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(Gas::from_tgas(5))
                .sign_callback(EVMTransactionWrapper::from_evm_transaction(&omni_evm_tx)),
        );
    }

    fn construct_erc20_transfer_tx(
        &self,
        token_address: String,
        recipient_address: String,
        amount: u128,
        network_details: NetworkDetails,
    ) -> EVMTransaction {
        let token_address = parse_eth_address(&token_address);
        let recipient_address = parse_eth_address(&recipient_address);

        let data: Vec<u8> = self.construct_erc20_transfer_data(recipient_address, amount);

        let evm_tx = TransactionBuilder::new::<EVM>()
            .nonce(network_details.eth_nonce)
            .to(token_address)
            .value(0)
            .input(data)
            .max_priority_fee_per_gas(network_details.max_priority_fee_per_gas)
            .max_fee_per_gas(network_details.max_fee_per_gas)
            .gas_limit(network_details.gas_limit)
            .chain_id(network_details.chain_id) // Sepolia
            .build();

        evm_tx
    }

    fn construct_erc20_transfer_data(&self, to: [u8; 20], amount: u128) -> Vec<u8> {
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

    #[private]
    #[must_use]
    pub fn sign_callback(
        &mut self,
        evm_tx_wrapper: EVMTransactionWrapper,
        #[callback_result] result: Result<SignResult, PromiseError>,
    ) -> Vec<u8> {
        let mpc_signature = result.unwrap();
        let big_r = &mpc_signature.big_r.affine_point;
        let s = &mpc_signature.s.scalar;

        let r = &big_r[2..];
        let v = mpc_signature.recovery_id;
        let signature_omni = OmniSignature {
            v: v,
            r: hex::decode(r).unwrap(),
            s: hex::decode(s).unwrap(),
        };
        let evm_tx = evm_tx_wrapper.to_evm_transaction();
        let evm_signed_tx = evm_tx.build_with_signature(&signature_omni);
        env::log_str(format!("Signed EVM transaction: {:?}", evm_signed_tx).as_str());
        self.latest_signed_tx = evm_signed_tx.clone();
        evm_signed_tx
    }
}
