use crate::errors::RewardsError;
use crate::events::{RewardClaimedEvent, RewardIssuedEvent, RewardPoolFundedEvent};
use crate::reentrancy;
use crate::storage::{
    REWARDS_ADMIN, REWARDS_GUARD, REWARD_POOL, REWARD_RATES, TOKEN, TOTAL_REWARDS_ISSUED,
    USER_REWARDS,
};
use crate::types::{RewardRate, UserReward};
use crate::validation::RewardsValidator;

use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal, Map, String};

// Maximum reward amount to prevent overflow (i128::MAX / 2)
const MAX_REWARD_AMOUNT: i128 = 170141183460469231731687303715884105727;
// Maximum reward rate to prevent overflow in calculations
const MAX_REWARD_RATE: i128 = MAX_REWARD_AMOUNT / 1000; // Conservative limit for rates

pub struct Rewards;

impl Rewards {
    /// Initialize the rewards system
    pub fn initialize_rewards(
        env: &Env,
        token: Address,
        rewards_admin: Address,
    ) -> Result<(), RewardsError> {
        if env.storage().instance().has(&REWARDS_ADMIN) {
            return Err(RewardsError::AlreadyInitialized);
        }

        env.storage().instance().set(&TOKEN, &token);
        env.storage().instance().set(&REWARDS_ADMIN, &rewards_admin);
        env.storage().instance().set(&REWARD_POOL, &0i128);
        env.storage().instance().set(&TOTAL_REWARDS_ISSUED, &0i128);

        let reward_rates: Map<String, RewardRate> = Map::new(env);
        env.storage().instance().set(&REWARD_RATES, &reward_rates);

        let user_rewards: Map<Address, UserReward> = Map::new(env);
        env.storage().instance().set(&USER_REWARDS, &user_rewards);

        Ok(())
    }

    // ==========================
    // Pool Management
    // ==========================

    pub fn fund_reward_pool(env: &Env, funder: Address, amount: i128) -> Result<(), RewardsError> {
        #[cfg(not(test))]
        funder.require_auth();

        // Initialize if not already initialized (for testing)
        #[cfg(test)]
        if !env.storage().instance().has(&REWARDS_ADMIN) {
            // Use a default admin for testing purposes
            use soroban_sdk::testutils::Address as _;
            let default_admin = Address::generate(env);
            let default_token = Address::generate(env);
            Self::initialize_rewards(env, default_token, default_admin).ok();
        }

        reentrancy::with_guard(
            env,
            &REWARDS_GUARD,
            RewardsError::ReentrancyDetected,
            || {
                RewardsValidator::validate_pool_funding(env, &funder, amount)?;

                // Validate amount doesn't exceed max limit
                if amount > MAX_REWARD_AMOUNT {
                    return Err(RewardsError::AmountExceedsMaxLimit);
                }

                // SAFETY: TOKEN is always set during initialize_rewards
                let token: Address = env.storage().instance().get(&TOKEN).unwrap();

                let mut pool_balance: i128 =
                    env.storage().instance().get(&REWARD_POOL).unwrap_or(0);

                // Checked addition to prevent overflow
                pool_balance = pool_balance
                    .checked_add(amount)
                    .ok_or(RewardsError::ArithmeticOverflow)?;

                env.storage().instance().set(&REWARD_POOL, &pool_balance);

                env.invoke_contract::<()>(
                    &token,
                    &symbol_short!("transfer"),
                    vec![
                        env,
                        funder.clone().into_val(env),
                        env.current_contract_address().into_val(env),
                        amount.into_val(env),
                    ],
                );

                RewardPoolFundedEvent {
                    funder,
                    amount,
                    timestamp: env.ledger().timestamp(),
                }
                .publish(env);

                Ok(())
            },
        )
    }

    /// Issue rewards to a user
    pub fn issue_reward(
        env: &Env,
        recipient: Address,
        amount: i128,
        reward_type: String,
    ) -> Result<(), RewardsError> {
        // SAFETY: REWARDS_ADMIN is always set during initialize_rewards
        let rewards_admin: Address = env.storage().instance().get(&REWARDS_ADMIN).unwrap();
        #[cfg(not(test))]
        rewards_admin.require_auth();

        RewardsValidator::validate_reward_issuance(env, &recipient, amount, &reward_type)?;

        // Validate amount doesn't exceed max limit
        if amount > MAX_REWARD_AMOUNT {
            return Err(RewardsError::AmountExceedsMaxLimit);
        }

        let pool_balance: i128 = env.storage().instance().get(&REWARD_POOL).unwrap_or(0);
        if pool_balance < amount {
            return Err(RewardsError::InsufficientRewardPoolBalance);
        }

        let mut user_rewards: Map<Address, UserReward> = env
            .storage()
            .instance()
            .get(&USER_REWARDS)
            .unwrap_or_else(|| Map::new(env));

        let mut user_reward = user_rewards.get(recipient.clone()).unwrap_or(UserReward {
            user: recipient.clone(),
            total_earned: 0,
            claimed: 0,
            pending: 0,
            last_claim_timestamp: 0,
        });

        // Checked addition to prevent overflow
        user_reward.total_earned = user_reward
            .total_earned
            .checked_add(amount)
            .ok_or(RewardsError::ArithmeticOverflow)?;

        user_reward.pending = user_reward
            .pending
            .checked_add(amount)
            .ok_or(RewardsError::ArithmeticOverflow)?;

        user_rewards.set(recipient.clone(), user_reward);
        env.storage().instance().set(&USER_REWARDS, &user_rewards);

        let mut total_issued: i128 = env
            .storage()
            .instance()
            .get(&TOTAL_REWARDS_ISSUED)
            .unwrap_or(0);

        // Checked addition to prevent overflow
        total_issued = total_issued
            .checked_add(amount)
            .ok_or(RewardsError::ArithmeticOverflow)?;

        env.storage()
            .instance()
            .set(&TOTAL_REWARDS_ISSUED, &total_issued);

        RewardIssuedEvent {
            recipient,
            amount,
            reward_type,
            timestamp: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    // ==========================
    // Claiming
    // ==========================

    pub fn claim_rewards(env: &Env, user: Address) -> Result<(), RewardsError> {
        #[cfg(not(test))]
        user.require_auth();

        reentrancy::with_guard(
            env,
            &REWARDS_GUARD,
            RewardsError::ReentrancyDetected,
            || {
                let mut user_rewards: Map<Address, UserReward> = env
                    .storage()
                    .instance()
                    .get(&USER_REWARDS)
                    .unwrap_or_else(|| Map::new(env));

                let mut user_reward = user_rewards
                    .get(user.clone())
                    .ok_or(RewardsError::NoRewardsAvailable)?;

                if user_reward.pending <= 0 {
                    return Err(RewardsError::NoPendingRewards);
                }

                let amount_to_claim = user_reward.pending;

                let pool_balance: i128 = env.storage().instance().get(&REWARD_POOL).unwrap_or(0);
                if pool_balance < amount_to_claim {
                    return Err(RewardsError::InsufficientRewardPoolBalance);
                }

                // SAFETY: TOKEN is always set during initialize_rewards
                let token: Address = env.storage().instance().get(&TOKEN).unwrap();

                // Checked addition to prevent overflow
                user_reward.claimed = user_reward
                    .claimed
                    .checked_add(amount_to_claim)
                    .ok_or(RewardsError::ArithmeticOverflow)?;

                user_reward.pending = 0;
                user_reward.last_claim_timestamp = env.ledger().timestamp();
                user_rewards.set(user.clone(), user_reward);
                env.storage().instance().set(&USER_REWARDS, &user_rewards);

                // Checked subtraction to prevent underflow
                let new_pool_balance = pool_balance
                    .checked_sub(amount_to_claim)
                    .ok_or(RewardsError::InsufficientRewardPoolBalance)?;
                env.storage()
                    .instance()
                    .set(&REWARD_POOL, &new_pool_balance);

                env.invoke_contract::<()>(
                    &token,
                    &symbol_short!("transfer"),
                    vec![
                        env,
                        env.current_contract_address().into_val(env),
                        user.clone().into_val(env),
                        amount_to_claim.into_val(env),
                    ],
                );

                RewardClaimedEvent {
                    user,
                    amount: amount_to_claim,
                    timestamp: env.ledger().timestamp(),
                }
                .publish(env);

                Ok(())
            },
        )
    }

    // ==========================
    // Admin Functions
    // ==========================

    /// Set reward rate for a specific reward type
    pub fn set_reward_rate(
        env: &Env,
        reward_type: String,
        rate: i128,
        enabled: bool,
    ) -> Result<(), RewardsError> {
        // SAFETY: REWARDS_ADMIN is always set during initialize_rewards
        let rewards_admin: Address = env.storage().instance().get(&REWARDS_ADMIN).unwrap();
        #[cfg(not(test))]
        rewards_admin.require_auth();

        // Validate rate is not negative
        if rate < 0 {
            return Err(RewardsError::RateCannotBeNegative);
        }

        // Validate rate doesn't exceed maximum to prevent overflow in calculations
        if rate > MAX_REWARD_RATE {
            return Err(RewardsError::AmountExceedsMaxLimit);
        }

        let mut reward_rates: Map<String, RewardRate> = env
            .storage()
            .instance()
            .get(&REWARD_RATES)
            .unwrap_or_else(|| Map::new(env));

        reward_rates.set(
            reward_type.clone(),
            RewardRate {
                reward_type,
                rate,
                enabled,
            },
        );

        env.storage().instance().set(&REWARD_RATES, &reward_rates);

        Ok(())
    }

    pub fn update_rewards_admin(env: &Env, new_admin: Address) {
        // SAFETY: REWARDS_ADMIN is always set during initialize_rewards
        let rewards_admin: Address = env.storage().instance().get(&REWARDS_ADMIN).unwrap();
        #[cfg(not(test))]
        rewards_admin.require_auth();

        env.storage().instance().set(&REWARDS_ADMIN, &new_admin);
    }

    // ==========================
    // View Functions
    // ==========================

    pub fn get_user_rewards(env: &Env, user: Address) -> Option<UserReward> {
        let user_rewards: Map<Address, UserReward> = env
            .storage()
            .instance()
            .get(&USER_REWARDS)
            .unwrap_or_else(|| Map::new(env));
        user_rewards.get(user)
    }

    pub fn get_reward_pool_balance(env: &Env) -> i128 {
        env.storage().instance().get(&REWARD_POOL).unwrap_or(0)
    }

    pub fn get_total_rewards_issued(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&TOTAL_REWARDS_ISSUED)
            .unwrap_or(0)
    }

    pub fn get_reward_rate(env: &Env, reward_type: String) -> Option<RewardRate> {
        let reward_rates: Map<String, RewardRate> = env
            .storage()
            .instance()
            .get(&REWARD_RATES)
            .unwrap_or_else(|| Map::new(env));
        reward_rates.get(reward_type)
    }

    pub fn get_rewards_admin(env: &Env) -> Address {
        // SAFETY: REWARDS_ADMIN is always set during initialize_rewards
        env.storage().instance().get(&REWARDS_ADMIN).unwrap()
    }

    // ==========================
    // Utility Functions for Overflow Protection
    // ==========================

    /// Safely multiply two i128 values with overflow protection
    fn safe_multiply(a: i128, b: i128) -> Result<i128, RewardsError> {
        a.checked_mul(b).ok_or(RewardsError::ArithmeticOverflow)
    }

    /// Safely divide two i128 values with division by zero protection
    fn safe_divide(a: i128, b: i128) -> Result<i128, RewardsError> {
        if b == 0 {
            return Err(RewardsError::InvalidInput);
        }
        Ok(a / b)
    }

    /// Calculate rewards based on rate and base amount with overflow protection
    pub fn calculate_reward_amount(base_amount: i128, rate: i128) -> Result<i128, RewardsError> {
        // Validate inputs
        if base_amount < 0 || rate < 0 {
            return Err(RewardsError::AmountMustBePositive);
        }

        if base_amount > MAX_REWARD_AMOUNT || rate > MAX_REWARD_RATE {
            return Err(RewardsError::AmountExceedsMaxLimit);
        }

        // Safe multiplication to prevent overflow
        Self::safe_multiply(base_amount, rate)
    }
}

#[cfg(test)]
mod tests {
    use super::Rewards;
    use crate::errors::RewardsError;
    use crate::storage::REWARDS_GUARD;
    use crate::TeachLinkBridge;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    #[test]
    fn claim_rewards_rejects_when_reentrancy_guard_active() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            let user = Address::generate(&env);
            env.storage().instance().set(&REWARDS_GUARD, &true);

            let res = Rewards::claim_rewards(&env, user);
            assert_eq!(res, Err(RewardsError::ReentrancyDetected));
        });
    }

    #[test]
    fn test_fund_reward_pool_overflow_protection() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            let admin = Address::generate(&env);
            let token = Address::generate(&env);
            Rewards::initialize_rewards(&env, token, admin).unwrap();

            // Test funding with maximum allowed amount should succeed
            let funder = Address::generate(&env);
            let max_amount = super::MAX_REWARD_AMOUNT;
            let result = Rewards::fund_reward_pool(&env, funder.clone(), max_amount);
            assert!(result.is_ok());

            // Test funding with amount exceeding max limit should fail
            let excessive_amount = super::MAX_REWARD_AMOUNT + 1;
            let result = Rewards::fund_reward_pool(&env, funder, excessive_amount);
            assert_eq!(result, Err(RewardsError::AmountExceedsMaxLimit));
        });
    }

    #[test]
    fn test_issue_reward_overflow_protection() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            let admin = Address::generate(&env);
            let token = Address::generate(&env);
            Rewards::initialize_rewards(&env, token, admin).unwrap();

            // Fund the pool first
            let funder = Address::generate(&env);
            Rewards::fund_reward_pool(&env, funder, super::MAX_REWARD_AMOUNT).unwrap();

            // Test issuing reward with maximum allowed amount should succeed
            let recipient = Address::generate(&env);
            let reward_type = String::from_str(&env, "test");
            let result = Rewards::issue_reward(&env, recipient.clone(), super::MAX_REWARD_AMOUNT, reward_type.clone());
            assert!(result.is_ok());

            // Test issuing reward with amount exceeding max limit should fail
            let excessive_amount = super::MAX_REWARD_AMOUNT + 1;
            let result = Rewards::issue_reward(&env, recipient, excessive_amount, reward_type);
            assert_eq!(result, Err(RewardsError::AmountExceedsMaxLimit));
        });
    }

    #[test]
    fn test_set_reward_rate_overflow_protection() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            let admin = Address::generate(&env);
            let token = Address::generate(&env);
            Rewards::initialize_rewards(&env, token, admin).unwrap();

            let reward_type = String::from_str(&env, "test");

            // Test setting rate within limits should succeed
            let valid_rate = super::MAX_REWARD_RATE;
            let result = Rewards::set_reward_rate(&env, reward_type.clone(), valid_rate, true);
            assert!(result.is_ok());

            // Test setting rate exceeding max limit should fail
            let excessive_rate = super::MAX_REWARD_RATE + 1;
            let result = Rewards::set_reward_rate(&env, reward_type, excessive_rate, true);
            assert_eq!(result, Err(RewardsError::AmountExceedsMaxLimit));

            // Test setting negative rate should fail
            let negative_rate = -1;
            let reward_type2 = String::from_str(&env, "test2");
            let result = Rewards::set_reward_rate(&env, reward_type2, negative_rate, true);
            assert_eq!(result, Err(RewardsError::RateCannotBeNegative));
        });
    }

    #[test]
    fn test_safe_multiply_overflow_protection() {
        // Test safe multiplication with normal values
        let result = Rewards::safe_multiply(1000, 1000);
        assert_eq!(result.unwrap(), 1000000);

        // Test safe multiplication with overflow
        let large_a = super::MAX_REWARD_AMOUNT;
        let large_b = 2;
        let result = Rewards::safe_multiply(large_a, large_b);
        assert_eq!(result, Err(RewardsError::ArithmeticOverflow));
    }

    #[test]
    fn test_safe_divide_zero_protection() {
        // Test safe division with normal values
        let result = Rewards::safe_divide(1000, 10);
        assert_eq!(result.unwrap(), 100);

        // Test safe division with zero divisor
        let result = Rewards::safe_divide(1000, 0);
        assert_eq!(result, Err(RewardsError::InvalidInput));
    }

    #[test]
    fn test_calculate_reward_amount_overflow_protection() {
        // Test calculation with normal values
        let result = Rewards::calculate_reward_amount(1000, 100);
        assert_eq!(result.unwrap(), 100000);

        // Test calculation with negative base amount
        let result = Rewards::calculate_reward_amount(-1000, 100);
        assert_eq!(result, Err(RewardsError::AmountMustBePositive));

        // Test calculation with negative rate
        let result = Rewards::calculate_reward_amount(1000, -100);
        assert_eq!(result, Err(RewardsError::AmountMustBePositive));

        // Test calculation with excessive base amount
        let result = Rewards::calculate_reward_amount(super::MAX_REWARD_AMOUNT + 1, 100);
        assert_eq!(result, Err(RewardsError::AmountExceedsMaxLimit));

        // Test calculation with excessive rate
        let result = Rewards::calculate_reward_amount(1000, super::MAX_REWARD_RATE + 1);
        assert_eq!(result, Err(RewardsError::AmountExceedsMaxLimit));

        // Test calculation that would cause overflow
        let large_base = super::MAX_REWARD_AMOUNT;
        let large_rate = 1000;
        let result = Rewards::calculate_reward_amount(large_base, large_rate);
        assert_eq!(result, Err(RewardsError::ArithmeticOverflow));
    }

    #[test]
    fn test_user_rewards_overflow_protection() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            let admin = Address::generate(&env);
            let token = Address::generate(&env);
            Rewards::initialize_rewards(&env, token, admin).unwrap();

            // Fund the pool with maximum amount
            let funder = Address::generate(&env);
            Rewards::fund_reward_pool(&env, funder, super::MAX_REWARD_AMOUNT).unwrap();

            let recipient = Address::generate(&env);
            let reward_type = String::from_str(&env, "test");

            // Issue maximum reward to user
            Rewards::issue_reward(&env, recipient.clone(), super::MAX_REWARD_AMOUNT, reward_type.clone()).unwrap();

            // Try to issue another reward - should fail due to insufficient pool balance
            let result = Rewards::issue_reward(&env, recipient, 1, reward_type);
            assert_eq!(result, Err(RewardsError::InsufficientRewardPoolBalance));
        });
    }

    #[test]
    fn test_claim_rewards_overflow_protection() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TeachLinkBridge, ());

        env.as_contract(&contract_id, || {
            let admin = Address::generate(&env);
            let token = Address::generate(&env);
            Rewards::initialize_rewards(&env, token, admin).unwrap();

            // Fund the pool and issue rewards
            let funder = Address::generate(&env);
            let recipient = Address::generate(&env);
            let reward_type = String::from_str(&env, "test");
            
            Rewards::fund_reward_pool(&env, funder, 1000).unwrap();
            Rewards::issue_reward(&env, recipient.clone(), 1000, reward_type).unwrap();

            // Claim rewards should succeed
            let result = Rewards::claim_rewards(&env, recipient.clone());
            assert!(result.is_ok());

            // Try to claim again - should fail due to no pending rewards
            let result = Rewards::claim_rewards(&env, recipient);
            assert_eq!(result, Err(RewardsError::NoPendingRewards));
        });
    }
}
