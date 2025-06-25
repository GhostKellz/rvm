//! Core VM Engine
//! 
//! Stack-based bytecode interpreter with gas metering and state management.

use crate::{opcodes::Opcode, gas::GasMeter, storage::Storage, error::RvmError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Core RVM execution context and state
#[derive(Debug, Clone)]
pub struct RvmCore {
    /// Execution stack
    pub stack: Vec<u64>,
    /// Program counter
    pub pc: usize,
    /// Gas meter for execution costs
    pub gas: GasMeter,
    /// Contract storage
    pub storage: Storage,
    /// Call depth tracking
    pub call_depth: usize,
    /// Execution environment
    pub env: ExecutionEnvironment,
}

/// Execution environment containing context data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEnvironment {
    /// Contract address being executed
    pub contract_address: [u8; 20],
    /// Caller address
    pub caller: [u8; 20],
    /// Call value
    pub value: u64,
    /// Gas price
    pub gas_price: u64,
    /// Block number
    pub block_number: u64,
    /// Block timestamp
    pub timestamp: u64,
}

/// Contract deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Contract bytecode
    pub bytecode: Vec<u8>,
    /// Contract address
    pub address: [u8; 20],
    /// Contract storage
    pub storage: HashMap<[u8; 32], [u8; 32]>,
    /// Contract balance
    pub balance: u64,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Return data
    pub return_data: Vec<u8>,
    /// Gas used
    pub gas_used: u64,
    /// Execution success
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl RvmCore {
    /// Create a new RVM core instance
    pub fn new(gas_limit: u64) -> Self {
        Self {
            stack: Vec::with_capacity(1024),
            pc: 0,
            gas: GasMeter::new(gas_limit),
            storage: Storage::new(),
            call_depth: 0,
            env: ExecutionEnvironment::default(),
        }
    }

    /// Execute bytecode with the given environment
    pub async fn execute(&mut self, bytecode: &[u8], env: ExecutionEnvironment) -> Result<ExecutionResult, RvmError> {
        self.env = env;
        self.pc = 0;
        self.stack.clear();

        while self.pc < bytecode.len() {
            let opcode = Opcode::from_byte(bytecode[self.pc])?;
            
            // Charge base gas for opcode
            self.gas.consume(opcode.gas_cost())?;
            
            match self.execute_opcode(opcode, bytecode).await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    return Ok(ExecutionResult {
                        return_data: vec![],
                        gas_used: self.gas.used(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(ExecutionResult {
            return_data: vec![], // Will be set by RETURN opcode
            gas_used: self.gas.used(),
            success: true,
            error: None,
        })
    }

    /// Get gas cost for opcode
    async fn execute_opcode(&mut self, opcode: Opcode, bytecode: &[u8]) -> Result<bool, RvmError> {
        match opcode {
            Opcode::PUSH1 => {
                self.pc += 1;
                if self.pc >= bytecode.len() {
                    return Err(RvmError::InvalidBytecode("PUSH1 without data".into()));
                }
                self.stack_push(bytecode[self.pc] as u64)?;
                self.pc += 1;
            }
            Opcode::PUSH32 => {
                if self.pc + 32 >= bytecode.len() {
                    return Err(RvmError::InvalidBytecode("PUSH32 without enough data".into()));
                }
                let mut value = 0u64;
                for i in 1..=8 { // Take first 8 bytes for u64
                    if self.pc + i < bytecode.len() {
                        value = (value << 8) | (bytecode[self.pc + i] as u64);
                    }
                }
                self.stack_push(value)?;
                self.pc += 33;
            }
            Opcode::ADD => {
                let b = self.stack_pop()?;
                let a = self.stack_pop()?;
                self.stack_push(a.wrapping_add(b))?;
                self.pc += 1;
            }
            Opcode::SUB => {
                let b = self.stack_pop()?;
                let a = self.stack_pop()?;
                self.stack_push(a.wrapping_sub(b))?;
                self.pc += 1;
            }
            Opcode::MUL => {
                let b = self.stack_pop()?;
                let a = self.stack_pop()?;
                self.stack_push(a.wrapping_mul(b))?;
                self.pc += 1;
            }
            Opcode::DIV => {
                let b = self.stack_pop()?;
                let a = self.stack_pop()?;
                if b == 0 {
                    self.stack_push(0)?;
                } else {
                    self.stack_push(a / b)?;
                }
                self.pc += 1;
            }
            Opcode::DUP1 => {
                let top = self.stack_top()?;
                self.stack_push(top)?;
                self.pc += 1;
            }
            Opcode::SWAP1 => {
                let len = self.stack.len();
                if len < 2 {
                    return Err(RvmError::StackUnderflow);
                }
                self.stack.swap(len - 1, len - 2);
                self.pc += 1;
            }
            Opcode::SSTORE => {
                let key = self.stack_pop()?;
                let value = self.stack_pop()?;
                self.storage.set(key, value).await?;
                self.pc += 1;
            }
            Opcode::SLOAD => {
                let key = self.stack_pop()?;
                let value = self.storage.get(key).await?;
                self.stack_push(value)?;
                self.pc += 1;
            }
            Opcode::JUMP => {
                let dest = self.stack_pop()? as usize;
                if dest >= bytecode.len() {
                    return Err(RvmError::InvalidJump(dest));
                }
                self.pc = dest;
            }
            Opcode::JUMPI => {
                let dest = self.stack_pop()? as usize;
                let condition = self.stack_pop()?;
                if condition != 0 {
                    if dest >= bytecode.len() {
                        return Err(RvmError::InvalidJump(dest));
                    }
                    self.pc = dest;
                } else {
                    self.pc += 1;
                }
            }
            Opcode::STOP => {
                return Ok(false);
            }
            Opcode::RETURN => {
                // For now, just stop execution
                return Ok(false);
            }
            _ => {
                self.pc += 1;
            }
        }
        Ok(true)
    }

    /// Push value onto stack
    fn stack_push(&mut self, value: u64) -> Result<(), RvmError> {
        if self.stack.len() >= crate::MAX_STACK_SIZE {
            return Err(RvmError::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop value from stack
    fn stack_pop(&mut self) -> Result<u64, RvmError> {
        self.stack.pop().ok_or(RvmError::StackUnderflow)
    }

    /// Get top stack value without popping
    fn stack_top(&self) -> Result<u64, RvmError> {
        self.stack.last().copied().ok_or(RvmError::StackUnderflow)
    }

    /// Deploy a contract
    pub async fn deploy_contract(&mut self, bytecode: Vec<u8>, env: ExecutionEnvironment) -> Result<[u8; 20], RvmError> {
        // Simple address generation (in production, use CREATE2 or similar)
        let mut address = [0u8; 20];
        address[0..8].copy_from_slice(&env.block_number.to_be_bytes());
        address[8..16].copy_from_slice(&env.timestamp.to_be_bytes());
        
        let contract = Contract {
            bytecode,
            address,
            storage: HashMap::new(),
            balance: env.value,
        };

        // Store contract (simplified storage)
        self.storage.set_contract(address, contract).await?;
        
        Ok(address)
    }
}

impl Default for ExecutionEnvironment {
    fn default() -> Self {
        Self {
            contract_address: [0u8; 20],
            caller: [0u8; 20],
            value: 0,
            gas_price: 1,
            block_number: 1,
            timestamp: 1640995200, // 2022-01-01
        }
    }
}

impl ExecutionEnvironment {
    /// Create a new execution environment
    pub fn new(contract_address: [u8; 20], caller: [u8; 20], value: u64) -> Self {
        Self {
            contract_address,
            caller,
            value,
            gas_price: 1,
            block_number: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}
