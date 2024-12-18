use near_sdk::{ext_contract, near, PromiseOrValue};

#[near(serializers = [json])]
pub struct SignRequest {
    pub payload: Vec<u8>,
    pub path: String,
    pub key_version: u32,
}

#[near(serializers = [json])]
pub struct SignResult {
    pub big_r: AffinePoint,
    pub s: Scalar,
    pub recovery_id: u64,
}

#[near(serializers = [json])]
pub struct AffinePoint {
    pub affine_point: String,
}

#[near(serializers = [json])]
pub struct Scalar {
    pub scalar: String,
}

#[ext_contract(mpc)]
pub trait MPC {
    fn sign(&self, request: SignRequest) -> PromiseOrValue<SignResult>;
}
