use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::{env, ext_contract, near, AccountId, Gas, NearToken, Promise, PromiseOrValue};
use omni_transaction::evm::utils::parse_eth_address;
use omni_transaction::transaction_builder::TransactionBuilder;
use omni_transaction::transaction_builder::TxBuilder;
use omni_transaction::types::EVM;

use hex::decode;

const MPC_CONTRACT_ACCOUNT_ID: &str = "v1.signer-dev.testnet";
const USDC_NEAR_ADDRESS: &str = "3e2210e1184b45b64c8a434c0a7e7b23cc04ea7eb7a6c3c32520d03d4afcb8af";
const USDC_ETH_ADDRESS: &str = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"; // USDC on Ethereum Sepolia
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
    eth_nonce: u64,
}

#[near]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self { eth_nonce: 0 }
    }

    fn transfer_and_sign_erc20(&mut self, eth_recipient: String, amount: U128) -> Promise {
        let rlp_encoded = self.construct_erc20_transfer_tx(
            USDC_ETH_ADDRESS.to_string(),
            eth_recipient,
            amount.0,
            self.eth_nonce,
        );

        // hash rlp encoded payload
        let payload: [u8; 32] = env::keccak256_array(&decode(rlp_encoded).unwrap())
            .try_into()
            .unwrap();

        self.eth_nonce += 1;

        mpc::ext(MPC_CONTRACT_ACCOUNT_ID.parse().unwrap())
            .with_static_gas(Gas::from_tgas(100))
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .sign(SignRequest {
                payload: payload.to_vec(),
                path: USDC_TREASURY_PATH.to_string(),
                key_version: 1,
            })
    }

    fn construct_erc20_transfer_tx(
        &self,
        token_address: String,
        recipient_address: String,
        amount: u128,
        nonce: u64,
    ) -> Vec<u8> {
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
            .chain_id(1) // Ethereum mainnet
            .build();

        evm_tx.build_for_signing()
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
        _sender_id: AccountId,
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

        let mut contract = Contract::new();
        let amount = U128(1_000_000); // 1 USDC (assuming 6 decimal places)
        let eth_recipient = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";

        let result = contract.ft_on_transfer(
            "alice.near".parse().unwrap(),
            amount,
            eth_recipient.to_string(),
        );

        // In a real test, you'd need to set up the MPC contract mock and assert on the result
        // For now, we'll just check that the function doesn't panic
        assert!(result.is_ok());
    }
}
