//! GhostChain 4-Token Gas Metering System
//!
//! Implements the advanced gas metering system supporting GCC, SPIRIT, MANA, and GHOST tokens
//! as outlined in the GCC_GAMEPLAN_FALL2025.md

use crate::error::RvmError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token types in the GhostChain ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    /// GCC (GhostChain Credits) - Base gas and transaction fees
    GCC,
    /// SPIRIT - Staking and governance token
    SPIRIT,
    /// MANA - AI operations and smart contracts (Jarvis integration)
    MANA,
    /// GHOST - Identity and domain operations
    GHOST,
}

/// Gas configuration for the 4-token economy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostChainGasConfig {
    /// Base gas price in GCC
    pub gcc_gas_price: u64,
    /// SPIRIT holders get gas discounts (percentage)
    pub spirit_discount_percentage: f64,
    /// Minimum SPIRIT balance required for discount
    pub spirit_discount_threshold: u64,
    /// MANA earned per gas unit for contract execution
    pub mana_reward_rate: f64,
    /// Premium gas price multiplier for .ghost domain operations
    pub ghost_premium_multiplier: f64,
    /// Gas costs for specific token operations
    pub token_operation_costs: HashMap<TokenType, u64>,
}

/// Token balances for an address
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenBalances {
    pub gcc: u64,
    pub spirit: u64,
    pub mana: u64,
    pub ghost: u64,
}

/// Enhanced gas meter supporting 4-token economy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostChainGasMeter {
    /// Base gas meter
    limit: u64,
    used: u64,
    refunded: u64,

    /// GhostChain-specific fields
    config: GhostChainGasConfig,
    executor_address: [u8; 20],
    executor_balances: TokenBalances,

    /// Token rewards/costs during execution
    mana_rewards_earned: u64,
    gcc_gas_cost: u64,
    spirit_discount_applied: u64,
    ghost_premium_paid: u64,
}

/// Gas payment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPayment {
    /// Primary payment token
    pub primary_token: TokenType,
    /// Amount paid in primary token
    pub primary_amount: u64,
    /// Additional token payments (for discounts/premiums)
    pub additional_payments: HashMap<TokenType, u64>,
    /// Total gas units purchased
    pub gas_units: u64,
}

/// Execution context for gas calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasExecutionContext {
    /// Address executing the transaction
    pub executor: [u8; 20],
    /// Contract being called (if any)
    pub contract_address: Option<[u8; 20]>,
    /// Whether this is a domain operation (.ghost, .gcc, etc.)
    pub is_domain_operation: bool,
    /// Domain name if applicable
    pub domain_name: Option<String>,
    /// Whether AI/Jarvis operations are involved
    pub has_ai_operations: bool,
}

impl GhostChainGasMeter {
    /// Create a new GhostChain gas meter
    pub fn new(
        limit: u64,
        config: GhostChainGasConfig,
        executor_address: [u8; 20],
        executor_balances: TokenBalances,
    ) -> Self {
        Self {
            limit,
            used: 0,
            refunded: 0,
            config,
            executor_address,
            executor_balances,
            mana_rewards_earned: 0,
            gcc_gas_cost: 0,
            spirit_discount_applied: 0,
            ghost_premium_paid: 0,
        }
    }

    /// Calculate gas cost for an operation with token-specific pricing
    pub fn calculate_gas_cost(
        &self,
        base_gas: u64,
        context: &GasExecutionContext,
    ) -> Result<GasPayment, RvmError> {
        let mut base_cost = base_gas * self.config.gcc_gas_price;
        let mut payment = GasPayment {
            primary_token: TokenType::GCC,
            primary_amount: base_cost,
            additional_payments: HashMap::new(),
            gas_units: base_gas,
        };

        // Apply SPIRIT discount if applicable
        if self.executor_balances.spirit >= self.config.spirit_discount_threshold {
            let discount = (base_cost as f64 * self.config.spirit_discount_percentage / 100.0) as u64;
            base_cost = base_cost.saturating_sub(discount);
            payment.primary_amount = base_cost;
        }

        // Apply GHOST premium for domain operations
        if context.is_domain_operation {
            let premium = (base_cost as f64 * (self.config.ghost_premium_multiplier - 1.0)) as u64;
            payment.additional_payments.insert(TokenType::GHOST, premium);
        }

        // Calculate MANA rewards for AI operations
        if context.has_ai_operations {
            let mana_reward = (base_gas as f64 * self.config.mana_reward_rate) as u64;
            // MANA rewards are negative payments (earnings)
            payment.additional_payments.insert(TokenType::MANA, mana_reward);
        }

        Ok(payment)
    }

    /// Consume gas with multi-token payment
    pub fn consume_with_tokens(
        &mut self,
        gas_amount: u64,
        context: &GasExecutionContext,
    ) -> Result<GasPayment, RvmError> {
        // Check if we have enough gas limit
        if self.used + gas_amount > self.limit {
            return Err(RvmError::OutOfGas {
                needed: self.used + gas_amount,
                available: self.limit,
            });
        }

        // Calculate token costs
        let payment = self.calculate_gas_cost(gas_amount, context)?;

        // Verify token balances (simplified - in production, this would be checked by the runtime)
        if payment.primary_amount > self.executor_balances.gcc {
            return Err(RvmError::InsufficientTokenBalance {
                token: "GCC".to_string(),
                required: payment.primary_amount,
                available: self.executor_balances.gcc,
            });
        }

        // Update gas usage
        self.used += gas_amount;
        self.gcc_gas_cost += payment.primary_amount;

        // Track SPIRIT discounts
        if self.executor_balances.spirit >= self.config.spirit_discount_threshold {
            let discount = (payment.primary_amount as f64 * self.config.spirit_discount_percentage / 100.0) as u64;
            self.spirit_discount_applied += discount;
        }

        // Track GHOST premiums
        if let Some(ghost_premium) = payment.additional_payments.get(&TokenType::GHOST) {
            self.ghost_premium_paid += ghost_premium;
        }

        // Track MANA rewards
        if let Some(mana_reward) = payment.additional_payments.get(&TokenType::MANA) {
            self.mana_rewards_earned += mana_reward;
        }

        Ok(payment)
    }

    /// Standard gas consumption (for backwards compatibility)
    pub fn consume(&mut self, amount: u64) -> Result<(), RvmError> {
        let context = GasExecutionContext {
            executor: self.executor_address,
            contract_address: None,
            is_domain_operation: false,
            domain_name: None,
            has_ai_operations: false,
        };

        self.consume_with_tokens(amount, &context)?;
        Ok(())
    }

    /// Mint MANA rewards for contract execution
    pub fn mint_mana_rewards(&mut self, gas_used: u64) -> u64 {
        let mana_reward = (gas_used as f64 * self.config.mana_reward_rate) as u64;
        self.mana_rewards_earned += mana_reward;
        mana_reward
    }

    /// Apply token-specific discounts
    pub fn apply_token_discounts(&self, base_cost: u64) -> u64 {
        if self.executor_balances.spirit >= self.config.spirit_discount_threshold {
            let discount = (base_cost as f64 * self.config.spirit_discount_percentage / 100.0) as u64;
            base_cost.saturating_sub(discount)
        } else {
            base_cost
        }
    }

    /// Get gas cost for specific token operations
    pub fn get_token_operation_cost(&self, token_type: TokenType) -> u64 {
        self.config.token_operation_costs
            .get(&token_type)
            .copied()
            .unwrap_or(1000) // Default cost
    }

    /// Get final token costs breakdown
    pub fn get_token_costs_breakdown(&self) -> HashMap<TokenType, u64> {
        let mut breakdown = HashMap::new();

        breakdown.insert(TokenType::GCC, self.gcc_gas_cost);

        if self.spirit_discount_applied > 0 {
            breakdown.insert(TokenType::SPIRIT, self.spirit_discount_applied);
        }

        if self.ghost_premium_paid > 0 {
            breakdown.insert(TokenType::GHOST, self.ghost_premium_paid);
        }

        if self.mana_rewards_earned > 0 {
            breakdown.insert(TokenType::MANA, self.mana_rewards_earned);
        }

        breakdown
    }

    /// Standard gas meter interface methods
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    pub fn used(&self) -> u64 {
        self.used
    }

    pub fn limit(&self) -> u64 {
        self.limit
    }

    pub fn refund(&mut self, amount: u64) {
        self.refunded += amount;
    }

    pub fn refunded(&self) -> u64 {
        self.refunded
    }

    pub fn can_consume(&self, amount: u64) -> bool {
        self.used + amount <= self.limit
    }

    pub fn final_cost(&self) -> u64 {
        self.used.saturating_sub(self.refunded / 2)
    }

    /// Get MANA rewards earned during execution
    pub fn mana_rewards(&self) -> u64 {
        self.mana_rewards_earned
    }

    /// Get SPIRIT discount applied
    pub fn spirit_discount(&self) -> u64 {
        self.spirit_discount_applied
    }

    /// Get GHOST premium paid
    pub fn ghost_premium(&self) -> u64 {
        self.ghost_premium_paid
    }
}

impl Default for GhostChainGasConfig {
    fn default() -> Self {
        let mut token_costs = HashMap::new();
        token_costs.insert(TokenType::GCC, 1000);
        token_costs.insert(TokenType::SPIRIT, 5000);
        token_costs.insert(TokenType::MANA, 2000);
        token_costs.insert(TokenType::GHOST, 10000);

        Self {
            gcc_gas_price: 1_000_000_000, // 1 Gwei equivalent
            spirit_discount_percentage: 10.0, // 10% discount
            spirit_discount_threshold: 1000, // Minimum 1000 SPIRIT
            mana_reward_rate: 0.1, // 0.1 MANA per gas unit
            ghost_premium_multiplier: 1.5, // 50% premium for .ghost operations
            token_operation_costs: token_costs,
        }
    }
}

impl TokenBalances {
    pub fn new(gcc: u64, spirit: u64, mana: u64, ghost: u64) -> Self {
        Self { gcc, spirit, mana, ghost }
    }

    pub fn get_balance(&self, token_type: TokenType) -> u64 {
        match token_type {
            TokenType::GCC => self.gcc,
            TokenType::SPIRIT => self.spirit,
            TokenType::MANA => self.mana,
            TokenType::GHOST => self.ghost,
        }
    }

    pub fn set_balance(&mut self, token_type: TokenType, amount: u64) {
        match token_type {
            TokenType::GCC => self.gcc = amount,
            TokenType::SPIRIT => self.spirit = amount,
            TokenType::MANA => self.mana = amount,
            TokenType::GHOST => self.ghost = amount,
        }
    }

    pub fn add_balance(&mut self, token_type: TokenType, amount: u64) {
        match token_type {
            TokenType::GCC => self.gcc += amount,
            TokenType::SPIRIT => self.spirit += amount,
            TokenType::MANA => self.mana += amount,
            TokenType::GHOST => self.ghost += amount,
        }
    }

    pub fn subtract_balance(&mut self, token_type: TokenType, amount: u64) -> Result<(), RvmError> {
        let current = self.get_balance(token_type);
        if current < amount {
            return Err(RvmError::InsufficientTokenBalance {
                token: format!("{:?}", token_type),
                required: amount,
                available: current,
            });
        }

        match token_type {
            TokenType::GCC => self.gcc -= amount,
            TokenType::SPIRIT => self.spirit -= amount,
            TokenType::MANA => self.mana -= amount,
            TokenType::GHOST => self.ghost -= amount,
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_balances() {
        let mut balances = TokenBalances::new(1000, 500, 200, 100);

        assert_eq!(balances.get_balance(TokenType::GCC), 1000);
        assert_eq!(balances.get_balance(TokenType::SPIRIT), 500);

        balances.add_balance(TokenType::MANA, 50);
        assert_eq!(balances.get_balance(TokenType::MANA), 250);

        assert!(balances.subtract_balance(TokenType::GHOST, 50).is_ok());
        assert_eq!(balances.get_balance(TokenType::GHOST), 50);

        assert!(balances.subtract_balance(TokenType::GHOST, 100).is_err());
    }

    #[test]
    fn test_gas_cost_calculation() {
        let config = GhostChainGasConfig::default();
        let balances = TokenBalances::new(10000, 2000, 1000, 500); // Has SPIRIT discount
        let meter = GhostChainGasMeter::new(100000, config, [1u8; 20], balances);

        let context = GasExecutionContext {
            executor: [1u8; 20],
            contract_address: None,
            is_domain_operation: false,
            domain_name: None,
            has_ai_operations: false,
        };

        let payment = meter.calculate_gas_cost(100, &context).unwrap();
        assert_eq!(payment.primary_token, TokenType::GCC);
        assert!(payment.primary_amount < 100 * meter.config.gcc_gas_price); // Should have discount
    }

    #[test]
    fn test_ghost_premium() {
        let config = GhostChainGasConfig::default();
        let balances = TokenBalances::new(10000, 100, 1000, 500); // No SPIRIT discount
        let meter = GhostChainGasMeter::new(100000, config, [1u8; 20], balances);

        let context = GasExecutionContext {
            executor: [1u8; 20],
            contract_address: None,
            is_domain_operation: true, // Domain operation
            domain_name: Some("test.ghost".to_string()),
            has_ai_operations: false,
        };

        let payment = meter.calculate_gas_cost(100, &context).unwrap();
        assert!(payment.additional_payments.contains_key(&TokenType::GHOST));
    }

    #[test]
    fn test_mana_rewards() {
        let config = GhostChainGasConfig::default();
        let balances = TokenBalances::new(10000, 100, 1000, 500);
        let meter = GhostChainGasMeter::new(100000, config, [1u8; 20], balances);

        let context = GasExecutionContext {
            executor: [1u8; 20],
            contract_address: None,
            is_domain_operation: false,
            domain_name: None,
            has_ai_operations: true, // AI operations
        };

        let payment = meter.calculate_gas_cost(100, &context).unwrap();
        assert!(payment.additional_payments.contains_key(&TokenType::MANA));
    }
}