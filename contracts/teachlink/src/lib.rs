pub mod errors;
pub mod events;
pub mod reentrancy;
pub mod rewards;
pub mod storage;
pub mod types;
pub mod validation;

use soroban_sdk::contracttype;

/// Main TeachLink contract implementation
pub struct TeachLinkBridge;

#[contracttype]
impl TeachLinkBridge {
    // Contract initialization and admin functions would go here
    // For now, we're focusing on the rewards module overflow fixes
}
