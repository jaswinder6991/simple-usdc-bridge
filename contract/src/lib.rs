use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{env, json_types::U128, near, AccountId, PromiseOrValue};

mod evm;
mod models;
mod signer;

const USDC_NEAR_ADDRESS: &str = "3e2210e1184b45b64c8a434c0a7e7b23cc04ea7eb7a6c3c32520d03d4afcb8af";

#[near(serializers=["json"])]
pub struct BridgeRequest {
    eth_address: String,
    network_details: NetworkDetails,
}

#[near(serializers=["json"])]
pub struct NetworkDetails {
    max_priority_fee_per_gas: u128,
    max_fee_per_gas: u128,
    gas_limit: u128,
    chain_id: u64,
    eth_nonce: u64,
}

#[near(contract_state)]
pub struct Contract {
    owner_id: AccountId,
    latest_signed_tx: Vec<u8>,
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {
            owner_id: env::current_account_id(),
            latest_signed_tx: Vec::new(),
        }
    }
}

#[near]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            latest_signed_tx: Vec::new(),
        }
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

    pub fn get_latest_signed_tx(&self) -> Vec<u8> {
        self.latest_signed_tx.clone()
    }
}

#[near]
impl FungibleTokenReceiver for Contract {
    //If there is an error here somehwere like wrong request etc, refund. Don't send back 0.
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

        let bridge_request: BridgeRequest = near_sdk::serde_json::from_str(&msg).unwrap();

        //Basis of this transfer, either return O or the amount
        // Initiate the Ethereum transfer
        self.transfer_and_sign_erc20(
            bridge_request.eth_address,
            amount,
            bridge_request.network_details,
        );
        PromiseOrValue::Value(U128::from(0))
    }
}
