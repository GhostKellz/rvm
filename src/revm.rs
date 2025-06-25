//! rEVM - Rust Ethereum Virtual Machine
//!
//! Ethereum Virtual Machine compatibility layer for RVM.
//! Provides full EVM opcode compatibility while running on the RVM runtime.

use crate::{
    core::{RvmCore, ExecutionEnvironment, ExecutionResult, Contract},
    crypto::{RvmCrypto, Precompiles},
    error::RvmError,
    gas::GasMeter,
    opcodes::Opcode,
    runtime::{RvmRuntime, RuntimeConfig},
    storage::Storage,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// EVM-compatible execution environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmEnvironment {
    /// Block coinbase (miner address)
    pub coinbase: [u8; 20],
    /// Block timestamp
    pub timestamp: u64,
    /// Block number
    pub block_number: u64,
    /// Block difficulty
    pub difficulty: u64,
    /// Block gas limit
    pub gas_limit: u64,
    /// Chain ID
    pub chain_id: u64,
    /// Base fee per gas (EIP-1559)
    pub base_fee: u64,
}

/// EVM transaction context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmTransaction {
    /// Transaction hash
    pub hash: [u8; 32],
    /// Sender address
    pub from: [u8; 20],
    /// Recipient address (None for contract creation)
    pub to: Option<[u8; 20]>,
    /// Transaction value
    pub value: u64,
    /// Transaction data/input
    pub data: Vec<u8>,
    /// Gas limit
    pub gas_limit: u64,
    /// Gas price
    pub gas_price: u64,
    /// Transaction nonce
    pub nonce: u64,
}

/// EVM account state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmAccount {
    /// Account balance
    pub balance: u64,
    /// Account nonce
    pub nonce: u64,
    /// Code hash
    pub code_hash: [u8; 32],
    /// Storage root
    pub storage_root: [u8; 32],
}

/// rEVM - Rust Ethereum Virtual Machine
pub struct REvm {
    /// Underlying RVM runtime
    runtime: RvmRuntime,
    /// EVM environment
    env: EvmEnvironment,
    /// EVM accounts
    accounts: HashMap<[u8; 20], EvmAccount>,
    /// Transaction history
    transactions: Vec<EvmTransaction>,
    /// Block history
    blocks: Vec<EvmBlock>,
}

/// EVM block information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmBlock {
    /// Block number
    pub number: u64,
    /// Block hash
    pub hash: [u8; 32],
    /// Parent hash
    pub parent_hash: [u8; 32],
    /// Block timestamp
    pub timestamp: u64,
    /// Block coinbase
    pub coinbase: [u8; 20],
    /// Block difficulty
    pub difficulty: u64,
    /// Gas limit
    pub gas_limit: u64,
    /// Gas used
    pub gas_used: u64,
    /// Transactions in block
    pub transactions: Vec<[u8; 32]>,
}

/// EVM execution result with additional EVM-specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmResult {
    /// Base execution result
    pub result: ExecutionResult,
    /// Logs generated during execution
    pub logs: Vec<EvmLog>,
    /// State changes
    pub state_changes: Vec<StateChange>,
    /// Transaction receipt
    pub receipt: TransactionReceipt,
}

/// EVM log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmLog {
    /// Contract address that generated the log
    pub address: [u8; 20],
    /// Log topics
    pub topics: Vec<[u8; 32]>,
    /// Log data
    pub data: Vec<u8>,
}

/// State change during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    /// Account address
    pub address: [u8; 20],
    /// Change type
    pub change_type: StateChangeType,
    /// Previous value
    pub previous_value: Vec<u8>,
    /// New value
    pub new_value: Vec<u8>,
}

/// Type of state change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateChangeType {
    /// Balance change
    Balance,
    /// Nonce change
    Nonce,
    /// Code change
    Code,
    /// Storage change
    Storage([u8; 32]), // Storage key
}

/// Transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction hash
    pub transaction_hash: [u8; 32],
    /// Block number
    pub block_number: u64,
    /// Gas used
    pub gas_used: u64,
    /// Success status
    pub success: bool,
    /// Contract address (for contract creation)
    pub contract_address: Option<[u8; 20]>,
    /// Logs
    pub logs: Vec<EvmLog>,
}

impl REvm {
    /// Create a new rEVM instance
    pub fn new(chain_id: u64) -> Self {
        let config = RuntimeConfig {
            max_gas_limit: 30_000_000, // EVM block gas limit
            enable_precompiles: true,
            enable_agent_apis: false, // Disable for pure EVM compatibility
            enable_crypto_hooks: true,
            debug_mode: false,
        };

        Self {
            runtime: RvmRuntime::new(config),
            env: EvmEnvironment {
                coinbase: [0u8; 20],
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                block_number: 1,
                difficulty: 1000000,
                gas_limit: 30_000_000,
                chain_id,
                base_fee: 1_000_000_000, // 1 Gwei
            },
            accounts: HashMap::new(),
            transactions: Vec::new(),
            blocks: Vec::new(),
        }
    }

    /// Execute an EVM transaction
    pub async fn execute_transaction(&mut self, tx: EvmTransaction) -> Result<EvmResult, RvmError> {
        // Convert EVM transaction to RVM execution environment
        let env = ExecutionEnvironment {
            contract_address: tx.to.unwrap_or([0u8; 20]),
            caller: tx.from,
            value: tx.value,
            gas_price: tx.gas_price,
            block_number: self.env.block_number,
            timestamp: self.env.timestamp,
        };

        // Execute transaction
        let result = if let Some(to) = tx.to {
            // Call existing contract
            self.runtime.call_contract(to, tx.data.clone(), tx.from, tx.value, tx.gas_limit).await?
        } else {
            // Contract creation
            let deployment_request = crate::runtime::DeploymentRequest {
                bytecode: tx.data.clone(),
                constructor_params: vec![],
                initial_balance: tx.value,
                gas_limit: tx.gas_limit,
            };
            
            let contract_address = self.runtime.deploy_contract(deployment_request, tx.from).await?;
            
            ExecutionResult {
                return_data: contract_address.to_vec(),
                gas_used: 21000, // Base contract creation cost
                success: true,
                error: None,
            }
        };

        // Create transaction receipt
        let receipt = TransactionReceipt {
            transaction_hash: tx.hash,
            block_number: self.env.block_number,
            gas_used: result.gas_used,
            success: result.success,
            contract_address: if tx.to.is_none() && result.success {
                Some(self.bytes_to_address(&result.return_data))
            } else {
                None
            },
            logs: vec![], // TODO: Extract logs from execution
        };

        // Store transaction
        self.transactions.push(tx);

        Ok(EvmResult {
            result,
            logs: vec![], // TODO: Extract logs
            state_changes: vec![], // TODO: Track state changes
            receipt,
        })
    }

    /// Deploy an EVM contract
    pub async fn deploy_contract(
        &mut self,
        bytecode: Vec<u8>,
        deployer: [u8; 20],
        value: u64,
        gas_limit: u64,
    ) -> Result<[u8; 20], RvmError> {
        let tx = EvmTransaction {
            hash: RvmCrypto::keccak256(&bytecode),
            from: deployer,
            to: None,
            value,
            data: bytecode,
            gas_limit,
            gas_price: 1_000_000_000, // 1 Gwei
            nonce: self.get_account_nonce(&deployer),
        };

        let result = self.execute_transaction(tx).await?;
        
        if result.result.success {
            Ok(result.receipt.contract_address.unwrap())
        } else {
            Err(RvmError::ExecutionFailed(
                result.result.error.unwrap_or_else(|| "Contract deployment failed".to_string())
            ))
        }
    }

    /// Call an EVM contract
    pub async fn call_contract(
        &mut self,
        contract_address: [u8; 20],
        call_data: Vec<u8>,
        caller: [u8; 20],
        value: u64,
        gas_limit: u64,
    ) -> Result<EvmResult, RvmError> {
        let tx = EvmTransaction {
            hash: RvmCrypto::keccak256(&call_data),
            from: caller,
            to: Some(contract_address),
            value,
            data: call_data,
            gas_limit,
            gas_price: 1_000_000_000,
            nonce: self.get_account_nonce(&caller),
        };

        self.execute_transaction(tx).await
    }

    /// Execute EVM bytecode directly
    pub async fn execute_bytecode(
        &mut self,
        bytecode: &[u8],
        caller: [u8; 20],
        value: u64,
        gas_limit: u64,
    ) -> Result<ExecutionResult, RvmError> {
        let env = ExecutionEnvironment {
            contract_address: [0u8; 20],
            caller,
            value,
            gas_price: 1_000_000_000,
            block_number: self.env.block_number,
            timestamp: self.env.timestamp,
        };

        self.runtime.execute(bytecode, env).await
    }

    /// Get account information
    pub fn get_account(&self, address: &[u8; 20]) -> EvmAccount {
        self.accounts.get(address).cloned().unwrap_or_else(|| EvmAccount {
            balance: 0,
            nonce: 0,
            code_hash: [0u8; 32],
            storage_root: [0u8; 32],
        })
    }

    /// Get account nonce
    pub fn get_account_nonce(&self, address: &[u8; 20]) -> u64 {
        self.get_account(address).nonce
    }

    /// Set account balance
    pub fn set_account_balance(&mut self, address: [u8; 20], balance: u64) {
        let mut account = self.get_account(&address);
        account.balance = balance;
        self.accounts.insert(address, account);
    }

    /// Mine a new block
    pub fn mine_block(&mut self) -> EvmBlock {
        let block_hash = RvmCrypto::keccak256(&self.env.block_number.to_be_bytes());
        let parent_hash = if self.env.block_number > 1 {
            self.blocks.last().map(|b| b.hash).unwrap_or([0u8; 32])
        } else {
            [0u8; 32]
        };

        let block = EvmBlock {
            number: self.env.block_number,
            hash: block_hash,
            parent_hash,
            timestamp: self.env.timestamp,
            coinbase: self.env.coinbase,
            difficulty: self.env.difficulty,
            gas_limit: self.env.gas_limit,
            gas_used: 0, // TODO: Calculate from transactions
            transactions: self.transactions.iter().map(|tx| tx.hash).collect(),
        };

        self.blocks.push(block.clone());
        self.env.block_number += 1;
        self.env.timestamp += 12; // 12 second block time
        
        block
    }

    /// Get block by number
    pub fn get_block(&self, block_number: u64) -> Option<&EvmBlock> {
        self.blocks.iter().find(|b| b.number == block_number)
    }

    /// Get current block number
    pub fn block_number(&self) -> u64 {
        self.env.block_number
    }

    /// Get chain ID
    pub fn chain_id(&self) -> u64 {
        self.env.chain_id
    }

    /// Convert bytes to address
    fn bytes_to_address(&self, bytes: &[u8]) -> [u8; 20] {
        let mut address = [0u8; 20];
        if bytes.len() >= 20 {
            address.copy_from_slice(&bytes[0..20]);
        }
        address
    }

    /// EVM demo: Simple arithmetic computation
    pub async fn evm_demo(&mut self) -> Result<EvmResult, RvmError> {
        // EVM bytecode: PUSH1 15, PUSH1 25, ADD, PUSH1 2, DIV, STOP
        // This computes (15 + 25) / 2 = 20
        let bytecode = vec![
            0x60, 0x0f, // PUSH1 15
            0x60, 0x19, // PUSH1 25
            0x01,       // ADD
            0x60, 0x02, // PUSH1 2
            0x04,       // DIV
            0x00,       // STOP
        ];

        let caller = [1u8; 20];
        let result = self.execute_bytecode(&bytecode, caller, 0, 100000).await?;

        Ok(EvmResult {
            result,
            logs: vec![],
            state_changes: vec![],
            receipt: TransactionReceipt {
                transaction_hash: RvmCrypto::keccak256(&bytecode),
                block_number: self.env.block_number,
                gas_used: 0,
                success: true,
                contract_address: None,
                logs: vec![],
            },
        })
    }
}

impl Default for EvmEnvironment {
    fn default() -> Self {
        Self {
            coinbase: [0u8; 20],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            block_number: 1,
            difficulty: 1000000,
            gas_limit: 30_000_000,
            chain_id: 1337, // Default to local testnet
            base_fee: 1_000_000_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_revm_execution() {
        let mut revm = REvm::new(1337);
        
        // Test EVM demo
        let result = revm.evm_demo().await.unwrap();
        assert!(result.result.success);
        assert!(result.result.gas_used > 0);
    }

    #[tokio::test]
    async fn test_contract_deployment() {
        let mut revm = REvm::new(1337);
        
        let bytecode = vec![0x60, 0x01, 0x60, 0x02, 0x01, 0x00]; // Simple ADD
        let deployer = [1u8; 20];
        
        let address = revm.deploy_contract(bytecode, deployer, 0, 100000).await.unwrap();
        assert_ne!(address, [0u8; 20]);
    }

    #[test]
    fn test_block_mining() {
        let mut revm = REvm::new(1337);
        
        let initial_block = revm.block_number();
        let block = revm.mine_block();
        
        assert_eq!(block.number, initial_block);
        assert_eq!(revm.block_number(), initial_block + 1);
    }
}
