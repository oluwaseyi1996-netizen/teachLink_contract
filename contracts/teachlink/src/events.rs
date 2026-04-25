use soroban_sdk::{contracteventfn, Address, String};

/// Event emitted when reward pool is funded
#[contracteventfn]
pub struct RewardPoolFundedEvent {
    pub funder: Address,
    pub amount: i128,
    pub timestamp: u64,
}

/// Event emitted when rewards are issued to a user
#[contracteventfn]
pub struct RewardIssuedEvent {
    pub recipient: Address,
    pub amount: i128,
    pub reward_type: String,
    pub timestamp: u64,
}

/// Event emitted when rewards are claimed by a user
#[contracteventfn]
pub struct RewardClaimedEvent {
    pub user: Address,
    pub amount: i128,
    pub timestamp: u64,
}
