use soroban_sdk::{Address, String};

/// Reward rate configuration for different reward types
#[derive(Clone, Debug)]
#[contracttype]
pub struct RewardRate {
    pub reward_type: String,
    pub rate: i128,
    pub enabled: bool,
}

/// User reward information
#[derive(Clone, Debug)]
#[contracttype]
pub struct UserReward {
    pub user: Address,
    pub total_earned: i128,
    pub claimed: i128,
    pub pending: i128,
    pub last_claim_timestamp: u64,
}

/// Escrow signer information
#[derive(Clone, Debug)]
#[contracttype]
pub struct EscrowSigner {
    pub address: Address,
    pub weight: u32,
}

/// Escrow parameters for creating escrow contracts
#[derive(Clone, Debug)]
#[contracttype]
pub struct EscrowParameters {
    pub depositor: Address,
    pub beneficiary: Address,
    pub token: Address,
    pub amount: i128,
    pub signers: Vec<EscrowSigner>,
    pub threshold: u32,
    pub release_time: Option<u64>,
    pub refund_time: Option<u64>,
    pub arbitrator: Address,
    pub description: String,
}

/// Escrow contract state
#[derive(Clone, Debug)]
#[contracttype]
pub struct Escrow {
    pub id: u64,
    pub parameters: EscrowParameters,
    pub status: EscrowStatus,
    pub created_at: u64,
    pub approval_count: u32,
    pub approved_signers: Vec<Address>,
}

/// Escrow status enumeration
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EscrowStatus {
    Pending,
    Approved,
    Released,
    Refunded,
    Disputed,
    Resolved,
    Cancelled,
}

/// Cross-chain message for bridge operations
#[derive(Clone, Debug)]
#[contracttype]
pub struct CrossChainMessage {
    pub message_id: u64,
    pub source_chain: u32,
    pub destination_chain: u32,
    pub amount: i128,
    pub recipient: Address,
    pub sender: Address,
    pub token: Address,
    pub nonce: u64,
    pub timestamp: u64,
    pub fee: i128,
}

/// Bridge transaction information
#[derive(Clone, Debug)]
#[contracttype]
pub struct BridgeTransaction {
    pub transaction_id: u64,
    pub message: CrossChainMessage,
    pub status: BridgeTransactionStatus,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub validator_signatures: Vec<Address>,
}

/// Bridge transaction status
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum BridgeTransactionStatus {
    Pending,
    Validated,
    Completed,
    Failed,
    Timeout,
}

/// Validator information for bridge consensus
#[derive(Clone, Debug)]
#[contracttype]
pub struct Validator {
    pub address: Address,
    pub stake: i128,
    pub is_active: bool,
    pub voting_power: u32,
}

/// Bridge configuration
#[derive(Clone, Debug)]
#[contracttype]
pub struct BridgeConfig {
    pub min_validators: u32,
    pub timeout_seconds: u64,
    pub fee_rate: i128,
    pub max_transaction_amount: i128,
    pub supported_chains: Vec<u32>,
}

/// Liquidity pool information
#[derive(Clone, Debug)]
#[contracttype]
pub struct LiquidityPool {
    pub token_a: Address,
    pub token_b: Address,
    pub reserve_a: i128,
    pub reserve_b: i128,
    pub total_supply: i128,
    pub fee_rate: i128,
}

/// Atomic swap information
#[derive(Clone, Debug)]
#[contracttype]
pub struct AtomicSwap {
    pub swap_id: u64,
    pub initiator: Address,
    pub participant: Address,
    pub token_a: Address,
    pub token_b: Address,
    pub amount_a: i128,
    pub amount_b: i128,
    pub hashlock: soroban_sdk::Bytes,
    pub timelock: u64,
    pub status: SwapStatus,
    pub created_at: u64,
}

/// Atomic swap status
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum SwapStatus {
    Pending,
    Completed,
    Refunded,
    Expired,
}

/// Mobile platform device information
#[derive(Clone, Debug)]
#[contracttype]
pub struct DeviceInfo {
    pub device_id: String,
    pub platform: String,
    pub version: String,
    pub is_active: bool,
    pub last_sync: u64,
}

/// Mobile app configuration
#[derive(Clone, Debug)]
#[contracttype]
pub struct MobileConfig {
    pub min_version: String,
    pub current_version: String,
    pub features: Vec<String>,
    pub maintenance_mode: bool,
}

impl Default for UserReward {
    fn default() -> Self {
        Self {
            user: Address::default(),
            total_earned: 0,
            claimed: 0,
            pending: 0,
            last_claim_timestamp: 0,
        }
    }
}

impl Default for RewardRate {
    fn default() -> Self {
        Self {
            reward_type: String::default(),
            rate: 0,
            enabled: false,
        }
    }
}

impl Default for EscrowStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl Default for BridgeTransactionStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl Default for SwapStatus {
    fn default() -> Self {
        Self::Pending
    }
}
