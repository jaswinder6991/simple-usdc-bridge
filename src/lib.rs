use std::fmt::format;

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::{env, ext_contract, near, AccountId, Gas, NearToken, Promise, PromiseOrValue};
use omni_transaction::evm::evm_transaction::EVMTransaction;
use omni_transaction::evm::utils::parse_eth_address;
use omni_transaction::transaction_builder::TransactionBuilder;
use omni_transaction::transaction_builder::TxBuilder;
use omni_transaction::types::EVM;

const MPC_CONTRACT_ACCOUNT_ID: &str = "v1.signer-dev.testnet";
const USDC_NEAR_ADDRESS: &str = "3e2210e1184b45b64c8a434c0a7e7b23cc04ea7eb7a6c3c32520d03d4afcb8af";
const USDC_ETH_ADDRESS: &str = "1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"; // USDC on Ethereum Sepolia
const USDC_TREASURY_PATH: &str = "usdc-treasury";

#[near(serializers=["json"])]
pub struct SignRequest {
    pub payload: Vec<u8>,
    pub path: String,
    pub key_version: u32,
}

#[ext_contract(mpc)]
pub trait MPC {
    fn sign(&self, request: SignRequest) -> Promise;
}

#[near(contract_state)]
pub struct Contract {
    owner_id: AccountId,
    eth_nonce: u64,
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {
            owner_id: env::current_account_id(),
            eth_nonce: 0,
        }
    }
}

#[near]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            eth_nonce: 0,
        }
    }

    pub fn set_nonce(&mut self, nonce: u64) {
        self.eth_nonce = nonce;
    }

    pub fn get_nonce(&self) -> u64 {
        self.eth_nonce
    }

    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }

    pub fn set_owner_id(&mut self, owner_id: AccountId) {
        assert!(
            env::predecessor_account_id() == self.owner_id,
            "Only the owner can change the owner ID"
        );
        self.owner_id = owner_id;
    }

    fn transfer_and_sign_erc20(&mut self, eth_recipient: String, amount: U128) -> Promise {
        env::log_str(
            format!(
                "Transferring {} USDC to {} with transaction nonce {}.",
                amount.0, eth_recipient, self.eth_nonce
            )
            .as_str(),
        );
        let omni_evm_tx = self.construct_erc20_transfer_tx(
            USDC_ETH_ADDRESS.to_string(),
            eth_recipient,
            amount.0,
            self.eth_nonce,
        );

        // Encode the transaction with EIP-1559 prefix
        let omni_evm_tx_encoded = omni_evm_tx.build_for_signing();

        // Hash the encoded transaction
        let omni_evm_tx_hash = env::keccak256(&omni_evm_tx_encoded);

        self.eth_nonce += 1;

        mpc::ext(MPC_CONTRACT_ACCOUNT_ID.parse().unwrap())
            .with_static_gas(Gas::from_tgas(100))
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .sign(SignRequest {
                payload: omni_evm_tx_hash,
                path: USDC_TREASURY_PATH.to_string(),
                key_version: 0,
            })
    }

    fn construct_erc20_transfer_tx(
        &self,
        token_address: String,
        recipient_address: String,
        amount: u128,
        nonce: u64,
    ) -> EVMTransaction {
        let token_address = parse_eth_address(&token_address);
        let recipient_address = parse_eth_address(&recipient_address);

        let data: Vec<u8> = self.construct_erc20_transfer_data(recipient_address, amount);

        let evm_tx = TransactionBuilder::new::<EVM>()
            .nonce(nonce)
            .to(token_address)
            .value(0)
            .input(data)
            .max_priority_fee_per_gas(1_500_000_000)
            .max_fee_per_gas(30_000_000_000)
            .gas_limit(65_000)
            .chain_id(11155111) // Sepolia
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
}

#[near]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // Ensure the token being transferred is USDC
        assert_eq!(
            env::predecessor_account_id().as_str(),
            USDC_NEAR_ADDRESS,
            "Only USDC transfers are supported"
        );

        // Parse the message to get Ethereum recipient address
        let eth_recipient = msg.trim();
        assert!(
            !eth_recipient.is_empty(),
            "Ethereum recipient address must be provided in the message"
        );

        //Basis of this transfer, either return O or the amount
        // Initiate the Ethereum transfer
        self.transfer_and_sign_erc20(eth_recipient.to_string(), amount);

        PromiseOrValue::Value(U128::from(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context(predecessor: AccountId) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(predecessor)
            .build()
    }

    #[test]
    fn test_ft_on_transfer() {
        let context = get_context(USDC_NEAR_ADDRESS.parse().unwrap());
        testing_env!(context);

        let mut contract = Contract::new("alice.near".parse().unwrap());
        let amount = U128(1_000_000); // 1 USDC (assuming 6 decimal places)
        let eth_recipient = "d8dA6BF26964aF9D7eEd9e03E53415D37aA96045";

        let result = contract.ft_on_transfer(
            "alice.near".parse().unwrap(),
            amount,
            eth_recipient.to_string(),
        );

        match result {
            PromiseOrValue::Value(returned_amount) => {
                assert_eq!(returned_amount.0, 0, "Expected returned amount to be 0");
            }
            PromiseOrValue::Promise(_) => {
                panic!("Expected a Value, got a Promise");
            }
        }

        // Check that the nonce was incremented
        assert_eq!(contract.eth_nonce, 1, "Nonce should be incremented");
        println!("Nonce: {}", contract.eth_nonce);
    }
}
