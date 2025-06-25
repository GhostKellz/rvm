//! Runtime System
//!
//! Plugin-friendly runtime with hooks for storage, crypto, and agent APIs.

use crate::{
    core::{RvmCore, ExecutionEnvironment, ExecutionResult, Contract},
    crypto::{RvmCrypto, Precompiles},
    error::RvmError,
    gas::GasMeter,
    storage::Storage,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Maximum gas limit
    pub max_gas_limit: u64,
    /// Enable precompiled contracts
    pub enable_precompiles: bool,
    /// Enable agent API hooks
    pub enable_agent_apis: bool,
    /// Enable crypto hooks
    pub enable_crypto_hooks: bool,
    /// Debug mode
    pub debug_mode: bool,
}

/// Runtime hooks for extending functionality
#[derive(Debug, Clone)]
pub struct RuntimeHooks {
    /// Storage hooks
    pub storage_hooks: Vec<fn(&mut Storage) -> Result<(), RvmError>>,
    /// Crypto hooks
    pub crypto_hooks: Vec<fn(&[u8]) -> Result<Vec<u8>, RvmError>>,
    /// Agent API hooks
    pub agent_hooks: Vec<fn(&str, &[u8]) -> Result<Vec<u8>, RvmError>>,
}

/// RVM Runtime with plugin support
pub struct RvmRuntime {
    /// Core VM instances (pool for concurrent execution)
    core_pool: Vec<RvmCore>,
    /// Shared storage
    storage: Arc<RwLock<Storage>>,
    /// Runtime configuration
    config: RuntimeConfig,
    /// Runtime hooks
    hooks: RuntimeHooks,
    /// Deployed contracts
    contracts: HashMap<[u8; 20], Contract>,
    /// Execution statistics
    stats: ExecutionStats,
}

/// Execution statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// Total executions
    pub total_executions: u64,
    /// Total gas used
    pub total_gas_used: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average gas per execution
    pub avg_gas_per_execution: u64,
}

/// Agent API call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCall {
    /// Agent function name
    pub function: String,
    /// Call parameters
    pub params: Vec<u8>,
    /// Expected return type
    pub return_type: String,
}

/// Contract deployment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRequest {
    /// Contract bytecode
    pub bytecode: Vec<u8>,
    /// Constructor parameters
    pub constructor_params: Vec<u8>,
    /// Initial balance
    pub initial_balance: u64,
    /// Gas limit for deployment
    pub gas_limit: u64,
}

impl RvmRuntime {
    /// Create a new runtime instance
    pub fn new(config: RuntimeConfig) -> Self {
        let mut core_pool = Vec::new();
        for _ in 0..4 { // Create a pool of 4 cores
            core_pool.push(RvmCore::new(config.max_gas_limit));
        }

        Self {
            core_pool,
            storage: Arc::new(RwLock::new(Storage::new())),
            config,
            hooks: RuntimeHooks {
                storage_hooks: Vec::new(),
                crypto_hooks: Vec::new(),
                agent_hooks: Vec::new(),
            },
            contracts: HashMap::new(),
            stats: ExecutionStats::default(),
        }
    }

    /// Execute bytecode with the runtime
    pub async fn execute(
        &mut self,
        bytecode: &[u8],
        env: ExecutionEnvironment,
    ) -> Result<ExecutionResult, RvmError> {
        // Get a core from the pool
        let mut core = self.core_pool.pop().unwrap_or_else(|| RvmCore::new(self.config.max_gas_limit));
        
        // Set up storage
        {
            let storage = self.storage.read().await;
            core.storage = storage.clone();
        }

        // Execute with hooks
        let result = self.execute_with_hooks(&mut core, bytecode, env).await;

        // Update storage
        {
            let mut storage = self.storage.write().await;
            *storage = core.storage.clone();
        }

        // Update statistics
        match &result {
            Ok(execution_result) => {
                self.update_stats(execution_result);
            }
            Err(_) => {
                // Update failed execution stats
                self.stats.total_executions += 1;
                self.stats.failed_executions += 1;
            }
        }

        // Return core to pool
        if self.core_pool.len() < 4 {
            core.pc = 0;
            core.stack.clear();
            self.core_pool.push(core);
        }

        result
    }

    /// Execute with runtime hooks
    async fn execute_with_hooks(
        &mut self,
        core: &mut RvmCore,
        bytecode: &[u8],
        env: ExecutionEnvironment,
    ) -> Result<ExecutionResult, RvmError> {
        // Pre-execution hooks
        if self.config.enable_crypto_hooks {
            for hook in &self.hooks.crypto_hooks {
                hook(bytecode)?;
            }
        }

        // Execute the bytecode
        let mut result = core.execute(bytecode, env).await?;

        // Post-execution hooks
        if self.config.enable_agent_apis {
            result = self.process_agent_calls(result).await?;
        }

        Ok(result)
    }

    /// Process agent API calls
    async fn process_agent_calls(&mut self, mut result: ExecutionResult) -> Result<ExecutionResult, RvmError> {
        // Check if return data contains agent call markers
        if result.return_data.starts_with(b"AGENT_CALL:") {
            let call_data = &result.return_data[11..]; // Skip "AGENT_CALL:" prefix
            
            if let Ok(agent_call) = serde_json::from_slice::<AgentCall>(call_data) {
                // Execute agent hook
                for hook in &self.hooks.agent_hooks {
                    if let Ok(agent_result) = hook(&agent_call.function, &agent_call.params) {
                        result.return_data = agent_result;
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    /// Deploy a contract
    pub async fn deploy_contract(
        &mut self,
        request: DeploymentRequest,
        deployer: [u8; 20],
    ) -> Result<[u8; 20], RvmError> {
        // Create deployment environment
        let env = ExecutionEnvironment {
            contract_address: [0u8; 20], // Will be set by deployment
            caller: deployer,
            value: request.initial_balance,
            gas_price: 1,
            block_number: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Get a core for deployment
        let mut core = self.core_pool.pop().unwrap_or_else(|| RvmCore::new(self.config.max_gas_limit));
        
        // Set up storage
        {
            let storage = self.storage.read().await;
            core.storage = storage.clone();
        }

        // Deploy the contract
        let contract_address = core.deploy_contract(request.bytecode.clone(), env).await?;

        // Update storage
        {
            let mut storage = self.storage.write().await;
            *storage = core.storage.clone();
        }

        // Store contract in runtime
        let contract = Contract {
            bytecode: request.bytecode,
            address: contract_address,
            storage: HashMap::new(),
            balance: request.initial_balance,
        };
        self.contracts.insert(contract_address, contract);

        // Return core to pool
        if self.core_pool.len() < 4 {
            self.core_pool.push(core);
        }

        Ok(contract_address)
    }

    /// Call a deployed contract
    pub async fn call_contract(
        &mut self,
        contract_address: [u8; 20],
        call_data: Vec<u8>,
        caller: [u8; 20],
        value: u64,
        gas_limit: u64,
    ) -> Result<ExecutionResult, RvmError> {
        // Get contract bytecode
        let contract = self.contracts.get(&contract_address)
            .ok_or_else(|| RvmError::ContractNotFound(contract_address))?;

        let bytecode = contract.bytecode.clone();

        // Create execution environment
        let env = ExecutionEnvironment {
            contract_address,
            caller,
            value,
            gas_price: 1,
            block_number: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Execute the contract
        self.execute(&bytecode, env).await
    }

    /// Execute a precompiled contract
    pub async fn execute_precompile(
        &self,
        address: u8,
        input: &[u8],
        gas_limit: u64,
    ) -> Result<ExecutionResult, RvmError> {
        if !self.config.enable_precompiles {
            return Err(RvmError::PrecompilesDisabled);
        }

        let mut gas_meter = GasMeter::new(gas_limit);
        
        // Charge gas for precompile execution
        let gas_cost = match address {
            1 => 3000,  // ECRECOVER
            2 => 60 + 12 * ((input.len() + 31) / 32) as u64, // SHA256
            3 => 600 + 120 * ((input.len() + 31) / 32) as u64, // RIPEMD160
            4 => 15 + 3 * ((input.len() + 31) / 32) as u64, // IDENTITY
            _ => return Err(RvmError::InvalidPrecompile(address)),
        };

        gas_meter.consume(gas_cost)?;

        match Precompiles::execute(address, input) {
            Ok(output) => Ok(ExecutionResult {
                return_data: output,
                gas_used: gas_meter.used(),
                success: true,
                error: None,
            }),
            Err(e) => Ok(ExecutionResult {
                return_data: vec![],
                gas_used: gas_meter.used(),
                success: false,
                error: Some(e.to_string()),
            }),
        }
    }

    /// Add a storage hook
    pub fn add_storage_hook(&mut self, hook: fn(&mut Storage) -> Result<(), RvmError>) {
        self.hooks.storage_hooks.push(hook);
    }

    /// Add a crypto hook
    pub fn add_crypto_hook(&mut self, hook: fn(&[u8]) -> Result<Vec<u8>, RvmError>) {
        self.hooks.crypto_hooks.push(hook);
    }

    /// Add an agent API hook
    pub fn add_agent_hook(&mut self, hook: fn(&str, &[u8]) -> Result<Vec<u8>, RvmError>) {
        self.hooks.agent_hooks.push(hook);
    }

    /// Get execution statistics
    pub fn get_stats(&self) -> &ExecutionStats {
        &self.stats
    }

    /// Get deployed contracts
    pub fn get_contracts(&self) -> &HashMap<[u8; 20], Contract> {
        &self.contracts
    }

    /// Update execution statistics
    fn update_stats(&mut self, result: &ExecutionResult) {
        self.stats.total_executions += 1;
        self.stats.total_gas_used += result.gas_used;
        
        if result.success {
            self.stats.successful_executions += 1;
        } else {
            self.stats.failed_executions += 1;
        }

        self.stats.avg_gas_per_execution = if self.stats.total_executions > 0 {
            self.stats.total_gas_used / self.stats.total_executions
        } else {
            0
        };
    }

    /// Create a simple demo execution
    pub async fn demo_execution(&mut self) -> Result<ExecutionResult, RvmError> {
        // Simple bytecode: PUSH1 10, PUSH1 20, ADD, PUSH1 5, MUL, STOP
        // This computes (10 + 20) * 5 = 150
        let bytecode = vec![
            0x60, 0x0a, // PUSH1 10
            0x60, 0x14, // PUSH1 20  
            0x01,       // ADD
            0x60, 0x05, // PUSH1 5
            0x02,       // MUL
            0x00,       // STOP
        ];

        let env = ExecutionEnvironment::default();
        self.execute(&bytecode, env).await
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_gas_limit: crate::DEFAULT_GAS_LIMIT,
            enable_precompiles: true,
            enable_agent_apis: true,
            enable_crypto_hooks: true,
            debug_mode: false,
        }
    }
}

impl Default for RuntimeHooks {
    fn default() -> Self {
        Self {
            storage_hooks: Vec::new(),
            crypto_hooks: Vec::new(),
            agent_hooks: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_execution() {
        let mut runtime = RvmRuntime::new(RuntimeConfig::default());
        
        // Test demo execution
        let result = runtime.demo_execution().await.unwrap();
        assert!(result.success);
        assert!(result.gas_used > 0);
    }

    #[tokio::test]
    async fn test_contract_deployment() {
        let mut runtime = RvmRuntime::new(RuntimeConfig::default());
        
        let request = DeploymentRequest {
            bytecode: vec![0x60, 0x01, 0x60, 0x02, 0x01, 0x00], // Simple ADD contract
            constructor_params: vec![],
            initial_balance: 1000,
            gas_limit: 100000,
        };

        let deployer = [1u8; 20];
        let address = runtime.deploy_contract(request, deployer).await.unwrap();
        
        assert_ne!(address, [0u8; 20]);
        assert!(runtime.contracts.contains_key(&address));
    }

    #[tokio::test]
    async fn test_precompile_execution() {
        let runtime = RvmRuntime::new(RuntimeConfig::default());
        
        // Test identity precompile
        let input = b"test data";
        let result = runtime.execute_precompile(4, input, 1000).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.return_data, input);
    }
}
