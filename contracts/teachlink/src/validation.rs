use crate::errors::EscrowError;
use crate::types::EscrowSigner;
use soroban_sdk::{Address, Bytes, Env, String, Vec};

/// Validation configuration constants
pub mod config {
    pub const MIN_AMOUNT: i128 = 1;
    pub const MAX_AMOUNT: i128 = i128::MAX / 2; // Prevent overflow
    pub const MIN_SIGNERS: u32 = 1;
    pub const MAX_SIGNERS: u32 = 100;
    pub const MIN_THRESHOLD: u32 = 1;
    pub const MAX_STRING_LENGTH: u32 = 256;
    pub const MIN_CHAIN_ID: u32 = 1;
    pub const MAX_CHAIN_ID: u32 = 999999;
    pub const MAX_ESCROW_DESCRIPTION_LENGTH: u32 = 1000;
    pub const MIN_TIMEOUT_SECONDS: u64 = 60; // 1 minute minimum
    pub const MAX_TIMEOUT_SECONDS: u64 = 31536000 * 10; // 10 years maximum
    pub const MAX_PAYLOAD_SIZE: u32 = 4096; // 4 KB max packet payload
    /// Bridge-specific amount bounds
    pub const MIN_BRIDGE_AMOUNT: i128 = 1;
    pub const MAX_BRIDGE_AMOUNT: i128 = 1_000_000_000_000_000_000; // 1e18
    /// Rewards-specific amount bounds
    pub const MIN_REWARD_AMOUNT: i128 = 1;
    pub const MAX_REWARD_AMOUNT: i128 = 170141183460469231731687303715884105727; // i128::MAX / 2
    pub const MAX_REWARD_RATE: i128 = 170141183460469231731687303715884105; // MAX_REWARD_AMOUNT / 1000
}

/// Validation errors
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    InvalidAddressFormat,
    BlacklistedAddress,
    InvalidAmountRange,
    InvalidSignerCount,
    InvalidThreshold,
    InvalidStringLength,
    InvalidChainId,
    InvalidTimeout,
    EmptySignersList,
    DuplicateSigners,
    InvalidBytesLength,
    InvalidCrossChainData,
}

/// Result type for validation operations
pub type ValidationResult<T> = core::result::Result<T, ValidationError>;

/// Address validation utilities
pub struct AddressValidator;

impl AddressValidator {
    /// Validates address format and basic constraints
    pub fn validate_format(_env: &Env, _address: &Address) -> ValidationResult<()> {
        // In Soroban, Address format is validated at the SDK level
        // Additional validation can be added here if needed
        // For now, we'll just check that it's not a zero address
        Ok(())
    }

    /// Checks if address is blacklisted (placeholder for future implementation)
    pub fn check_blacklist(env: &Env, address: &Address) -> ValidationResult<()> {
        let blacklist_key = soroban_sdk::symbol_short!("blacklist");
        let blacklist: Vec<Address> = env
            .storage()
            .instance()
            .get(&blacklist_key)
            .unwrap_or_else(|| Vec::new(env));

        if blacklist.contains(address) {
            return Err(ValidationError::BlacklistedAddress);
        }
        Ok(())
    }

    /// Comprehensive address validation
    pub fn validate(env: &Env, address: &Address) -> ValidationResult<()> {
        Self::validate_format(env, address)?;
        Self::check_blacklist(env, address)?;
        Ok(())
    }
}

/// Numerical validation utilities
pub struct NumberValidator;

impl NumberValidator {
    /// Validates amount within allowed range
    pub fn validate_amount(amount: i128) -> ValidationResult<()> {
        if amount < config::MIN_AMOUNT {
            return Err(ValidationError::InvalidAmountRange);
        }
        if amount > config::MAX_AMOUNT {
            return Err(ValidationError::InvalidAmountRange);
        }
        Ok(())
    }

    /// Validates reward amount within allowed range
    pub fn validate_reward_amount(amount: i128) -> ValidationResult<()> {
        if amount < config::MIN_REWARD_AMOUNT {
            return Err(ValidationError::InvalidAmountRange);
        }
        if amount > config::MAX_REWARD_AMOUNT {
            return Err(ValidationError::InvalidAmountRange);
        }
        Ok(())
    }

    /// Validates reward rate within allowed range
    pub fn validate_reward_rate(rate: i128) -> ValidationResult<()> {
        if rate < 0 {
            return Err(ValidationError::InvalidAmountRange);
        }
        if rate > config::MAX_REWARD_RATE {
            return Err(ValidationError::InvalidAmountRange);
        }
        Ok(())
    }

    /// Validates signer count
    #[allow(clippy::cast_possible_truncation)]
    pub fn validate_signer_count(count: usize) -> ValidationResult<()> {
        if count == 0 {
            return Err(ValidationError::EmptySignersList);
        }
        if (count as u32) < config::MIN_SIGNERS {
            return Err(ValidationError::InvalidSignerCount);
        }
        if (count as u32) > config::MAX_SIGNERS {
            return Err(ValidationError::InvalidSignerCount);
        }
        Ok(())
    }

    /// Validates threshold against signer count
    pub fn validate_threshold(threshold: u32, signer_count: u32) -> ValidationResult<()> {
        if threshold < config::MIN_THRESHOLD {
            return Err(ValidationError::InvalidThreshold);
        }
        if threshold > signer_count {
            return Err(ValidationError::InvalidThreshold);
        }
        Ok(())
    }

    /// Validates chain ID
    pub fn validate_chain_id(chain_id: u32) -> ValidationResult<()> {
        if !(config::MIN_CHAIN_ID..=config::MAX_CHAIN_ID).contains(&chain_id) {
            return Err(ValidationError::InvalidChainId);
        }
        Ok(())
    }

    /// Validates timeout duration
    pub fn validate_timeout(timeout_seconds: u64) -> ValidationResult<()> {
        if timeout_seconds < config::MIN_TIMEOUT_SECONDS {
            return Err(ValidationError::InvalidTimeout);
        }
        if timeout_seconds > config::MAX_TIMEOUT_SECONDS {
            return Err(ValidationError::InvalidTimeout);
        }
        Ok(())
    }
}

/// String validation utilities
pub struct StringValidator;

impl StringValidator {
    /// Validates string length
    pub fn validate_length(string: &String, max_length: u32) -> ValidationResult<()> {
        if string.is_empty() {
            return Err(ValidationError::InvalidStringLength);
        }
        if string.len() > max_length {
            return Err(ValidationError::InvalidStringLength);
        }
        Ok(())
    }

    /// Validates string contains only allowed characters
    pub fn validate_characters(string: &String) -> ValidationResult<()> {
        // Allow alphanumeric, spaces, and basic punctuation
        let string_bytes = string.to_bytes();
        for byte in string_bytes.iter() {
            let char = byte as char;
            if !char.is_alphanumeric()
                && !char.is_whitespace()
                && !matches!(
                    char,
                    '-' | '_' | '.' | ',' | '!' | '?' | '@' | '#' | '$' | '%' | '&' | '*'
                        | '+' | '=' | ':'
                )
            {
                return Err(ValidationError::InvalidStringLength);
            }
        }
        Ok(())
    }

    /// Comprehensive string validation
    pub fn validate(string: &String, max_length: u32) -> ValidationResult<()> {
        Self::validate_length(string, max_length)?;
        Self::validate_characters(string)?;
        Ok(())
    }
}

/// Bytes validation utilities
pub struct BytesValidator;

impl BytesValidator {
    /// Validates bytes for cross-chain addresses
    pub fn validate_cross_chain_address(bytes: &Bytes) -> ValidationResult<()> {
        // Most blockchain addresses are 20-32 bytes
        if bytes.len() < 20 || bytes.len() > 32 {
            return Err(ValidationError::InvalidBytesLength);
        }
        // Reject all-zero addresses (null address)
        let all_zero = bytes.iter().all(|b| b == 0);
        if all_zero {
            return Err(ValidationError::InvalidAddressFormat);
        }
        Ok(())
    }

    /// Validates bytes for general use
    pub fn validate_length(bytes: &Bytes, min_len: u32, max_len: u32) -> ValidationResult<()> {
        if bytes.len() < min_len || bytes.len() > max_len {
            return Err(ValidationError::InvalidBytesLength);
        }
        Ok(())
    }

    /// Validates packet payload (non-empty, within size limit)
    pub fn validate_payload(bytes: &Bytes) -> ValidationResult<()> {
        if bytes.is_empty() {
            return Err(ValidationError::InvalidCrossChainData);
        }
        if bytes.len() > config::MAX_PAYLOAD_SIZE {
            return Err(ValidationError::InvalidBytesLength);
        }
        Ok(())
    }
}

/// Parameter sanitization utilities
pub struct InputSanitizer;

impl InputSanitizer {
    /// Sanitizes reward amount to ensure it's within valid range
    pub fn sanitize_reward_amount(amount: i128) -> ValidationResult<i128> {
        NumberValidator::validate_reward_amount(amount)?;
        Ok(amount)
    }

    /// Sanitizes reward rate to ensure it's within valid range
    pub fn sanitize_reward_rate(rate: i128) -> ValidationResult<i128> {
        NumberValidator::validate_reward_rate(rate)?;
        Ok(rate)
    }
}

/// Rewards-specific validation utilities
pub struct RewardsValidator;

impl RewardsValidator {
    /// Validates reward issuance parameters
    pub fn validate_reward_issuance(
        env: &Env,
        recipient: &Address,
        amount: i128,
        reward_type: &String,
    ) -> Result<(), crate::errors::RewardsError> {
        AddressValidator::validate(env, recipient)
            .map_err(|_| crate::errors::RewardsError::AmountMustBePositive)?;

        // Use reward-specific validation
        InputSanitizer::sanitize_reward_amount(amount)
            .map_err(|_| crate::errors::RewardsError::AmountMustBePositive)?;

        StringValidator::validate(reward_type, config::MAX_STRING_LENGTH)
            .map_err(|_| crate::errors::RewardsError::AmountMustBePositive)?;

        Ok(())
    }

    /// Validates reward pool funding
    pub fn validate_pool_funding(
        env: &Env,
        funder: &Address,
        amount: i128,
    ) -> Result<(), crate::errors::RewardsError> {
        AddressValidator::validate(env, funder)
            .map_err(|_| crate::errors::RewardsError::AmountMustBePositive)?;

        // Use reward-specific validation
        InputSanitizer::sanitize_reward_amount(amount)
            .map_err(|_| crate::errors::RewardsError::AmountMustBePositive)?;

        Ok(())
    }

    /// Validates reward rate setting
    pub fn validate_reward_rate_setting(
        env: &Env,
        rate: i128,
        reward_type: &String,
    ) -> Result<(), crate::errors::RewardsError> {
        // Use reward-specific validation for rate
        InputSanitizer::sanitize_reward_rate(rate)
            .map_err(|_| crate::errors::RewardsError::RateCannotBeNegative)?;

        StringValidator::validate(reward_type, config::MAX_STRING_LENGTH)
            .map_err(|_| crate::errors::RewardsError::AmountMustBePositive)?;

        Ok(())
    }
}
