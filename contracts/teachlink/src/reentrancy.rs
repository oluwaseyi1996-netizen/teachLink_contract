use soroban_sdk::Env;

/// Reentrancy guard utility to prevent recursive calls
pub struct ReentrancyGuard;

impl ReentrancyGuard {
    /// Execute a function with reentrancy protection
    pub fn with_guard<F, R, E>(
        env: &Env,
        guard_key: &soroban_sdk::Symbol,
        error: E,
        f: F,
    ) -> Result<R, E>
    where
        F: FnOnce() -> Result<R, E>,
    {
        // Check if guard is already set
        if env.storage().instance().get(guard_key).unwrap_or(false) {
            return Err(error);
        }

        // Set the guard
        env.storage().instance().set(guard_key, &true);

        // Execute the function
        let result = f();

        // Clear the guard regardless of success or failure
        env.storage().instance().set(guard_key, &false);

        result
    }

    /// Check if reentrancy guard is currently active
    pub fn is_guard_active(env: &Env, guard_key: &soroban_sdk::Symbol) -> bool {
        env.storage().instance().get(guard_key).unwrap_or(false)
    }

    /// Force clear the reentrancy guard (emergency use only)
    pub fn force_clear_guard(env: &Env, guard_key: &soroban_sdk::Symbol) {
        env.storage().instance().set(guard_key, &false);
    }
}
