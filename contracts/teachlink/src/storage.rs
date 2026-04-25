use soroban_sdk::{symbol_short, Address, Map, Vec};

/// Storage keys for rewards module
pub const REWARDS_ADMIN: soroban_sdk::Symbol = symbol_short!("radmin");
pub const REWARDS_GUARD: soroban_sdk::Symbol = symbol_short!("rguard");
pub const REWARD_POOL: soroban_sdk::Symbol = symbol_short!("rpool");
pub const REWARD_RATES: soroban_sdk::Symbol = symbol_short!("rrates");
pub const TOKEN: soroban_sdk::Symbol = symbol_short!("token");
pub const TOTAL_REWARDS_ISSUED: soroban_sdk::Symbol = symbol_short!("total");
pub const USER_REWARDS: soroban_sdk::Symbol = symbol_short!("urew");

/// Storage keys for escrow module
pub const ESCROW_ADMIN: soroban_sdk::Symbol = symbol_short!("eadmin");
pub const ESCROW_COUNT: soroban_sdk::Symbol = symbol_short!("ecount");
pub const ESCROWS: soroban_sdk::Symbol = symbol_short!("escrows");
pub const ESCROW_GUARD: soroban_sdk::Symbol = symbol_short!("eguard");

/// Storage keys for bridge module
pub const BRIDGE_ADMIN: soroban_sdk::Symbol = symbol_short!("badmin");
pub const SUPPORTED_CHAINS: soroban_sdk::Symbol = symbol_short!("chains");
pub const VALIDATORS: soroban_sdk::Symbol = symbol_short!("vals");
pub const BRIDGE_CONFIG: soroban_sdk::Symbol = symbol_short!("bconf");
pub const BRIDGE_TRANSACTIONS: soroban_sdk::Symbol = symbol_short!("btxs");
pub const NONCE_TRACKER: soroban_sdk::Symbol = symbol_short!("nonce");
pub const BRIDGE_GUARD: soroban_sdk::Symbol = symbol_short!("bguard");

/// Storage keys for liquidity module
pub const LIQUIDITY_POOLS: soroban_sdk::Symbol = symbol_short!("lpools");
pub const LP_TOKENS: soroban_sdk::Symbol = symbol_short!("lptoks");

/// Storage keys for atomic swap module
pub const ATOMIC_SWAPS: soroban_sdk::Symbol = symbol_short!("aswaps");
pub const SWAP_COUNT: soroban_sdk::Symbol = symbol_short!("scount");

/// Storage keys for mobile platform module
pub const MOBILE_CONFIG: soroban_sdk::Symbol = symbol_short!("mconf");
pub const REGISTERED_DEVICES: soroban_sdk::Symbol = symbol_short!("devs");
