//! Storage System
//!
//! Manages contract storage, account state, and persistent data.

use crate::{error::RvmError, core::Contract};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Storage backend for contracts and state
#[derive(Debug, Clone)]
pub struct Storage {
    /// Contract storage: address -> key -> value
    contract_storage: HashMap<[u8; 20], HashMap<u64, u64>>,
    /// Deployed contracts
    contracts: HashMap<[u8; 20], Contract>,
    /// Account balances
    balances: HashMap<[u8; 20], u64>,
    /// Nonces for accounts
    nonces: HashMap<[u8; 20], u64>,
    /// Storage state for gas calculations
    original_storage: HashMap<([u8; 20], u64), u64>,
}

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account balance
    pub balance: u64,
    /// Account nonce
    pub nonce: u64,
    /// Code hash (for contracts)
    pub code_hash: Option<[u8; 32]>,
    /// Storage root (for contracts)
    pub storage_root: Option<[u8; 32]>,
}

/// Storage change for tracking state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChange {
    /// Account address
    pub address: [u8; 20],
    /// Storage key
    pub key: u64,
    /// Previous value
    pub previous_value: u64,
    /// New value
    pub new_value: u64,
}

impl Storage {
    /// Create a new storage instance
    pub fn new() -> Self {
        Self {
            contract_storage: HashMap::new(),
            contracts: HashMap::new(),
            balances: HashMap::new(),
            nonces: HashMap::new(),
            original_storage: HashMap::new(),
        }
    }

    /// Get storage value for a contract
    pub async fn get(&self, key: u64) -> Result<u64, RvmError> {
        // For now, use a default address. In a real implementation,
        // this would be based on the current execution context
        let address = [0u8; 20];
        
        Ok(self.contract_storage
            .get(&address)
            .and_then(|storage| storage.get(&key))
            .copied()
            .unwrap_or(0))
    }

    /// Set storage value for a contract
    pub async fn set(&mut self, key: u64, value: u64) -> Result<(), RvmError> {
        let address = [0u8; 20];
        
        // Track original value for gas calculations
        if !self.original_storage.contains_key(&(address, key)) {
            let original_value = self.get_raw(&address, key);
            self.original_storage.insert((address, key), original_value);
        }

        self.contract_storage
            .entry(address)
            .or_insert_with(HashMap::new)
            .insert(key, value);
        
        Ok(())
    }

    /// Get storage value for a specific address
    pub fn get_storage(&self, address: &[u8; 20], key: u64) -> u64 {
        self.contract_storage
            .get(address)
            .and_then(|storage| storage.get(&key))
            .copied()
            .unwrap_or(0)
    }

    /// Set storage value for a specific address
    pub fn set_storage(&mut self, address: [u8; 20], key: u64, value: u64) {
        // Track original value for gas calculations
        if !self.original_storage.contains_key(&(address, key)) {
            let original_value = self.get_raw(&address, key);
            self.original_storage.insert((address, key), original_value);
        }

        self.contract_storage
            .entry(address)
            .or_insert_with(HashMap::new)
            .insert(key, value);
    }

    /// Get raw storage value without async
    fn get_raw(&self, address: &[u8; 20], key: u64) -> u64 {
        self.contract_storage
            .get(address)
            .and_then(|storage| storage.get(&key))
            .copied()
            .unwrap_or(0)
    }

    /// Get original storage value for gas calculation
    pub fn get_original_storage(&self, address: &[u8; 20], key: u64) -> u64 {
        self.original_storage
            .get(&(*address, key))
            .copied()
            .unwrap_or_else(|| self.get_raw(address, key))
    }

    /// Store a contract
    pub async fn set_contract(&mut self, address: [u8; 20], contract: Contract) -> Result<(), RvmError> {
        self.contracts.insert(address, contract);
        Ok(())
    }

    /// Get a contract
    pub fn get_contract(&self, address: &[u8; 20]) -> Option<&Contract> {
        self.contracts.get(address)
    }

    /// Get account balance
    pub fn get_balance(&self, address: &[u8; 20]) -> u64 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    /// Set account balance
    pub fn set_balance(&mut self, address: [u8; 20], balance: u64) {
        self.balances.insert(address, balance);
    }

    /// Transfer balance between accounts
    pub fn transfer(&mut self, from: [u8; 20], to: [u8; 20], amount: u64) -> Result<(), RvmError> {
        let from_balance = self.get_balance(&from);
        if from_balance < amount {
            return Err(RvmError::InsufficientBalance {
                available: from_balance,
                required: amount,
            });
        }

        let to_balance = self.get_balance(&to);
        
        self.set_balance(from, from_balance - amount);
        self.set_balance(to, to_balance + amount);
        
        Ok(())
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &[u8; 20]) -> u64 {
        self.nonces.get(address).copied().unwrap_or(0)
    }

    /// Set account nonce
    pub fn set_nonce(&mut self, address: [u8; 20], nonce: u64) {
        self.nonces.insert(address, nonce);
    }

    /// Increment account nonce
    pub fn increment_nonce(&mut self, address: [u8; 20]) {
        let current_nonce = self.get_nonce(&address);
        self.set_nonce(address, current_nonce + 1);
    }

    /// Get account information
    pub fn get_account(&self, address: &[u8; 20]) -> Account {
        Account {
            balance: self.get_balance(address),
            nonce: self.get_nonce(address),
            code_hash: self.contracts.get(address).map(|_| [0u8; 32]), // Simplified
            storage_root: None, // Simplified
        }
    }

    /// Check if an account exists
    pub fn account_exists(&self, address: &[u8; 20]) -> bool {
        self.balances.contains_key(address) || 
        self.contracts.contains_key(address) ||
        self.nonces.get(address).map_or(false, |&n| n > 0)
    }

    /// Create a new account
    pub fn create_account(&mut self, address: [u8; 20], balance: u64) {
        self.set_balance(address, balance);
        self.set_nonce(address, 0);
    }

    /// Delete an account (for SELFDESTRUCT)
    pub fn delete_account(&mut self, address: [u8; 20]) {
        self.balances.remove(&address);
        self.nonces.remove(&address);
        self.contracts.remove(&address);
        self.contract_storage.remove(&address);
    }

    /// Get all storage changes for an address
    pub fn get_storage_changes(&self, address: &[u8; 20]) -> Vec<StorageChange> {
        let mut changes = Vec::new();
        
        if let Some(storage) = self.contract_storage.get(address) {
            for (&key, &new_value) in storage.iter() {
                let original_value = self.get_original_storage(address, key);
                if original_value != new_value {
                    changes.push(StorageChange {
                        address: *address,
                        key,
                        previous_value: original_value,
                        new_value,
                    });
                }
            }
        }
        
        changes
    }

    /// Commit storage changes (finalize transaction)
    pub fn commit(&mut self) {
        // In a real implementation, this would write to persistent storage
        self.original_storage.clear();
    }

    /// Revert storage changes (rollback transaction)
    pub fn revert(&mut self) {
        // Restore original values
        for ((address, key), original_value) in self.original_storage.drain() {
            if original_value == 0 {
                // Remove the key if original value was 0
                if let Some(storage) = self.contract_storage.get_mut(&address) {
                    storage.remove(&key);
                }
            } else {
                // Restore original value
                self.contract_storage
                    .entry(address)
                    .or_insert_with(HashMap::new)
                    .insert(key, original_value);
            }
        }
    }

    /// Create a snapshot of the current state
    pub fn snapshot(&self) -> StorageSnapshot {
        StorageSnapshot {
            contract_storage: self.contract_storage.clone(),
            contracts: self.contracts.clone(),
            balances: self.balances.clone(),
            nonces: self.nonces.clone(),
            original_storage: self.original_storage.clone(),
        }
    }

    /// Restore from a snapshot
    pub fn restore_snapshot(&mut self, snapshot: StorageSnapshot) {
        self.contract_storage = snapshot.contract_storage;
        self.contracts = snapshot.contracts;
        self.balances = snapshot.balances;
        self.nonces = snapshot.nonces;
        self.original_storage = snapshot.original_storage;
    }
}

/// Storage snapshot for state management
#[derive(Debug, Clone)]
pub struct StorageSnapshot {
    contract_storage: HashMap<[u8; 20], HashMap<u64, u64>>,
    contracts: HashMap<[u8; 20], Contract>,
    balances: HashMap<[u8; 20], u64>,
    nonces: HashMap<[u8; 20], u64>,
    original_storage: HashMap<([u8; 20], u64), u64>,
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_operations() {
        let mut storage = Storage::new();
        
        // Test basic storage operations
        assert_eq!(storage.get(1).await.unwrap(), 0);
        storage.set(1, 42).await.unwrap();
        assert_eq!(storage.get(1).await.unwrap(), 42);
    }

    #[test]
    fn test_balance_operations() {
        let mut storage = Storage::new();
        let addr1 = [1u8; 20];
        let addr2 = [2u8; 20];
        
        storage.set_balance(addr1, 1000);
        storage.set_balance(addr2, 500);
        
        assert_eq!(storage.get_balance(&addr1), 1000);
        assert_eq!(storage.get_balance(&addr2), 500);
        
        storage.transfer(addr1, addr2, 300).unwrap();
        
        assert_eq!(storage.get_balance(&addr1), 700);
        assert_eq!(storage.get_balance(&addr2), 800);
    }

    #[test]
    fn test_nonce_operations() {
        let mut storage = Storage::new();
        let addr = [1u8; 20];
        
        assert_eq!(storage.get_nonce(&addr), 0);
        storage.increment_nonce(addr);
        assert_eq!(storage.get_nonce(&addr), 1);
        storage.set_nonce(addr, 10);
        assert_eq!(storage.get_nonce(&addr), 10);
    }
}
