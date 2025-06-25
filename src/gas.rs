//! Gas Metering System
//!
//! Tracks execution costs and prevents infinite loops or excessive resource usage.

use crate::error::RvmError;
use serde::{Deserialize, Serialize};

/// Gas meter for tracking execution costs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasMeter {
    /// Gas limit for this execution
    limit: u64,
    /// Gas used so far
    used: u64,
    /// Gas refunded (for storage operations)
    refunded: u64,
}

impl GasMeter {
    /// Create a new gas meter with the given limit
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            used: 0,
            refunded: 0,
        }
    }

    /// Consume gas for an operation
    pub fn consume(&mut self, amount: u64) -> Result<(), RvmError> {
        if self.used + amount > self.limit {
            return Err(RvmError::OutOfGas {
                needed: self.used + amount,
                available: self.limit,
            });
        }
        self.used += amount;
        Ok(())
    }

    /// Refund gas (for storage operations)
    pub fn refund(&mut self, amount: u64) {
        self.refunded += amount;
    }

    /// Get remaining gas
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    /// Get gas used
    pub fn used(&self) -> u64 {
        self.used
    }

    /// Get gas refunded
    pub fn refunded(&self) -> u64 {
        self.refunded
    }

    /// Get gas limit
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Check if we have enough gas for an operation
    pub fn can_consume(&self, amount: u64) -> bool {
        self.used + amount <= self.limit
    }

    /// Calculate final gas cost including refunds
    pub fn final_cost(&self) -> u64 {
        self.used.saturating_sub(self.refunded / 2) // EIP-3529: limit refunds to half of gas used
    }

    /// Reset the gas meter for a new execution
    pub fn reset(&mut self, limit: u64) {
        self.limit = limit;
        self.used = 0;
        self.refunded = 0;
    }

    /// Get gas cost for memory expansion
    pub fn memory_gas_cost(current_size: usize, new_size: usize) -> u64 {
        if new_size <= current_size {
            return 0;
        }

        let word_cost = |size: usize| -> u64 {
            let size_in_words = (size + 31) / 32;
            let linear_cost = size_in_words as u64 * 3;
            let quadratic_cost = (size_in_words * size_in_words) as u64 / 512;
            linear_cost + quadratic_cost
        };

        word_cost(new_size).saturating_sub(word_cost(current_size))
    }

    /// Get gas cost for copying data
    pub fn copy_gas_cost(size: usize) -> u64 {
        let word_size = (size + 31) / 32;
        word_size as u64 * 3
    }

    /// Get gas cost for KECCAK256 operation
    pub fn keccak256_gas_cost(size: usize) -> u64 {
        30 + ((size + 31) / 32) as u64 * 6
    }

    /// Get gas cost for LOG operations
    pub fn log_gas_cost(topics: usize, data_size: usize) -> u64 {
        375 + (topics as u64 * 375) + (data_size as u64 * 8)
    }

    /// Get gas cost for SSTORE operation based on current and new values
    pub fn sstore_gas_cost(current_value: u64, new_value: u64, original_value: u64) -> (u64, u64) {
        // EIP-2200 gas costs
        if new_value == current_value {
            return (100, 0); // SLOAD_GAS
        }

        if current_value == original_value {
            if original_value == 0 {
                return (20000, 0); // SSTORE_SET_GAS
            } else {
                return (5000, 0); // SSTORE_RESET_GAS
            }
        } else {
            let gas_cost = 100; // SLOAD_GAS
            let mut gas_refund = 0;

            if current_value != 0 && new_value == 0 {
                gas_refund += 15000; // SSTORE_CLEARS_SCHEDULE
            }

            if original_value != 0 {
                if current_value == 0 {
                    gas_refund -= 15000; // Remove refund if we're setting a cleared slot
                }
                if new_value == 0 {
                    gas_refund += 15000; // Add refund for clearing
                }
            }

            if original_value == new_value {
                if original_value == 0 {
                    gas_refund += 19900; // SSTORE_SET_GAS - SLOAD_GAS
                } else {
                    gas_refund += 4900; // SSTORE_RESET_GAS - SLOAD_GAS
                }
            }

            (gas_cost, gas_refund)
        }
    }
}

impl Default for GasMeter {
    fn default() -> Self {
        Self::new(crate::DEFAULT_GAS_LIMIT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_consumption() {
        let mut meter = GasMeter::new(1000);
        
        assert_eq!(meter.remaining(), 1000);
        assert!(meter.consume(100).is_ok());
        assert_eq!(meter.used(), 100);
        assert_eq!(meter.remaining(), 900);
        
        assert!(meter.consume(1000).is_err()); // Should exceed limit
        assert_eq!(meter.used(), 100); // Should remain unchanged after error
    }

    #[test]
    fn test_gas_refund() {
        let mut meter = GasMeter::new(1000);
        
        meter.consume(500).unwrap();
        meter.refund(100);
        
        assert_eq!(meter.used(), 500);
        assert_eq!(meter.refunded(), 100);
        assert_eq!(meter.final_cost(), 450); // 500 - 100/2
    }

    #[test]
    fn test_memory_gas_cost() {
        assert_eq!(GasMeter::memory_gas_cost(0, 32), 3);
        assert_eq!(GasMeter::memory_gas_cost(32, 64), 3);
        assert_eq!(GasMeter::memory_gas_cost(0, 64), 6);
    }

    #[test]
    fn test_sstore_gas_cost() {
        // New storage slot
        let (gas, refund) = GasMeter::sstore_gas_cost(0, 100, 0);
        assert_eq!(gas, 20000);
        assert_eq!(refund, 0);

        // Modify existing slot
        let (gas, refund) = GasMeter::sstore_gas_cost(100, 200, 100);
        assert_eq!(gas, 5000);
        assert_eq!(refund, 0);

        // Clear storage slot
        let (gas, refund) = GasMeter::sstore_gas_cost(100, 0, 100);
        assert_eq!(gas, 5000);
        assert_eq!(refund, 15000);
    }
}
