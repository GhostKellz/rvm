//! WASM-lite Module
//!
//! Lightweight, secure, and performant WebAssembly-inspired execution environment.
//! Designed specifically for blockchain and agent use cases with deterministic execution.

use crate::{
    error::RvmError,
    gas::GasMeter,
    core::{ExecutionResult, ExecutionEnvironment},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WASM-lite instruction set - simplified and optimized for blockchain use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmLiteInstruction {
    // Control Flow
    Nop = 0x00,
    Block = 0x02,
    Loop = 0x03,
    If = 0x04,
    Else = 0x05,
    End = 0x0b,
    Br = 0x0c,
    BrIf = 0x0d,
    Return = 0x0f,
    Call = 0x10,
    CallIndirect = 0x11,

    // Memory Operations
    LocalGet = 0x20,
    LocalSet = 0x21,
    LocalTee = 0x22,
    GlobalGet = 0x23,
    GlobalSet = 0x24,
    
    // Load/Store
    I32Load = 0x28,
    I64Load = 0x29,
    I32Store = 0x36,
    I64Store = 0x37,
    
    // Constants
    I32Const = 0x41,
    I64Const = 0x42,
    
    // Arithmetic
    I32Add = 0x6a,
    I32Sub = 0x6b,
    I32Mul = 0x6c,
    I32DivS = 0x6d,
    I32DivU = 0x6e,
    I32RemS = 0x6f,
    I32RemU = 0x70,
    I32And = 0x71,
    I32Or = 0x72,
    I32Xor = 0x73,
    I32Shl = 0x74,
    I32ShrS = 0x75,
    I32ShrU = 0x76,
    
    // Comparison
    I32Eqz = 0x45,
    I32Eq = 0x46,
    I32Ne = 0x47,
    I32LtS = 0x48,
    I32LtU = 0x49,
    I32GtS = 0x4a,
    I32GtU = 0x4b,
    I32LeS = 0x4c,
    I32LeU = 0x4d,
    I32GeS = 0x4e,
    I32GeU = 0x4f,
    
    // WASM-lite specific extensions for blockchain
    Keccak256 = 0xf0,
    EcRecover = 0xf1,
    Blake2b = 0xf2,
    Ed25519Verify = 0xf3,
    GetCaller = 0xf4,
    GetValue = 0xf5,
    GetGasRemaining = 0xf6,
    StorageLoad = 0xf7,
    StorageStore = 0xf8,
    EmitLog = 0xf9,
    Transfer = 0xfa,
}

/// WASM-lite value types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmLiteValueType {
    I32,
    I64,
    Bytes,
}

/// WASM-lite runtime value
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmLiteValue {
    I32(i32),
    I64(i64),
    Bytes(Vec<u8>),
}

/// WASM-lite function signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmLiteFunction {
    /// Function name
    pub name: String,
    /// Parameter types
    pub params: Vec<WasmLiteValueType>,
    /// Return types
    pub returns: Vec<WasmLiteValueType>,
    /// Function body (bytecode)
    pub body: Vec<u8>,
    /// Local variables
    pub locals: Vec<WasmLiteValueType>,
}

/// WASM-lite module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmLiteModule {
    /// Module version
    pub version: u32,
    /// Functions
    pub functions: Vec<WasmLiteFunction>,
    /// Global variables
    pub globals: Vec<WasmLiteValue>,
    /// Memory pages
    pub memory_pages: u32,
    /// Export table
    pub exports: HashMap<String, usize>, // function name -> function index
    /// Import table
    pub imports: HashMap<String, WasmLiteFunction>,
}

/// WASM-lite execution context
#[derive(Debug, Clone)]
pub struct WasmLiteContext {
    /// Value stack
    stack: Vec<WasmLiteValue>,
    /// Local variables
    locals: Vec<WasmLiteValue>,
    /// Global variables
    globals: Vec<WasmLiteValue>,
    /// Memory
    memory: Vec<u8>,
    /// Program counter
    pc: usize,
    /// Call stack
    call_stack: Vec<CallFrame>,
    /// Gas meter
    gas: GasMeter,
    /// Execution environment
    env: ExecutionEnvironment,
}

/// Call frame for function calls
#[derive(Debug, Clone)]
struct CallFrame {
    /// Function index
    function_index: usize,
    /// Return address
    return_pc: usize,
    /// Local variable offset
    locals_offset: usize,
    /// Stack pointer at call time
    stack_pointer: usize,
}

/// WASM-lite virtual machine
pub struct WasmLiteVM {
    /// Loaded modules
    modules: HashMap<String, WasmLiteModule>,
    /// Execution context
    context: Option<WasmLiteContext>,
}

impl WasmLiteInstruction {
    /// Convert byte to instruction
    pub fn from_byte(byte: u8) -> Result<Self, RvmError> {
        match byte {
            0x00 => Ok(WasmLiteInstruction::Nop),
            0x02 => Ok(WasmLiteInstruction::Block),
            0x03 => Ok(WasmLiteInstruction::Loop),
            0x04 => Ok(WasmLiteInstruction::If),
            0x05 => Ok(WasmLiteInstruction::Else),
            0x0b => Ok(WasmLiteInstruction::End),
            0x0c => Ok(WasmLiteInstruction::Br),
            0x0d => Ok(WasmLiteInstruction::BrIf),
            0x0f => Ok(WasmLiteInstruction::Return),
            0x10 => Ok(WasmLiteInstruction::Call),
            0x11 => Ok(WasmLiteInstruction::CallIndirect),
            0x20 => Ok(WasmLiteInstruction::LocalGet),
            0x21 => Ok(WasmLiteInstruction::LocalSet),
            0x22 => Ok(WasmLiteInstruction::LocalTee),
            0x23 => Ok(WasmLiteInstruction::GlobalGet),
            0x24 => Ok(WasmLiteInstruction::GlobalSet),
            0x28 => Ok(WasmLiteInstruction::I32Load),
            0x29 => Ok(WasmLiteInstruction::I64Load),
            0x36 => Ok(WasmLiteInstruction::I32Store),
            0x37 => Ok(WasmLiteInstruction::I64Store),
            0x41 => Ok(WasmLiteInstruction::I32Const),
            0x42 => Ok(WasmLiteInstruction::I64Const),
            0x45 => Ok(WasmLiteInstruction::I32Eqz),
            0x46 => Ok(WasmLiteInstruction::I32Eq),
            0x47 => Ok(WasmLiteInstruction::I32Ne),
            0x48 => Ok(WasmLiteInstruction::I32LtS),
            0x49 => Ok(WasmLiteInstruction::I32LtU),
            0x4a => Ok(WasmLiteInstruction::I32GtS),
            0x4b => Ok(WasmLiteInstruction::I32GtU),
            0x4c => Ok(WasmLiteInstruction::I32LeS),
            0x4d => Ok(WasmLiteInstruction::I32LeU),
            0x4e => Ok(WasmLiteInstruction::I32GeS),
            0x4f => Ok(WasmLiteInstruction::I32GeU),
            0x6a => Ok(WasmLiteInstruction::I32Add),
            0x6b => Ok(WasmLiteInstruction::I32Sub),
            0x6c => Ok(WasmLiteInstruction::I32Mul),
            0x6d => Ok(WasmLiteInstruction::I32DivS),
            0x6e => Ok(WasmLiteInstruction::I32DivU),
            0x6f => Ok(WasmLiteInstruction::I32RemS),
            0x70 => Ok(WasmLiteInstruction::I32RemU),
            0x71 => Ok(WasmLiteInstruction::I32And),
            0x72 => Ok(WasmLiteInstruction::I32Or),
            0x73 => Ok(WasmLiteInstruction::I32Xor),
            0x74 => Ok(WasmLiteInstruction::I32Shl),
            0x75 => Ok(WasmLiteInstruction::I32ShrS),
            0x76 => Ok(WasmLiteInstruction::I32ShrU),
            0xf0 => Ok(WasmLiteInstruction::Keccak256),
            0xf1 => Ok(WasmLiteInstruction::EcRecover),
            0xf2 => Ok(WasmLiteInstruction::Blake2b),
            0xf3 => Ok(WasmLiteInstruction::Ed25519Verify),
            0xf4 => Ok(WasmLiteInstruction::GetCaller),
            0xf5 => Ok(WasmLiteInstruction::GetValue),
            0xf6 => Ok(WasmLiteInstruction::GetGasRemaining),
            0xf7 => Ok(WasmLiteInstruction::StorageLoad),
            0xf8 => Ok(WasmLiteInstruction::StorageStore),
            0xf9 => Ok(WasmLiteInstruction::EmitLog),
            0xfa => Ok(WasmLiteInstruction::Transfer),
            _ => Err(RvmError::InvalidWasmLiteInstruction(byte)),
        }
    }

    /// Get gas cost for instruction
    pub fn gas_cost(&self) -> u64 {
        match self {
            WasmLiteInstruction::Nop => 0,
            WasmLiteInstruction::Block | WasmLiteInstruction::Loop | 
            WasmLiteInstruction::If | WasmLiteInstruction::Else | 
            WasmLiteInstruction::End => 1,
            
            WasmLiteInstruction::Br | WasmLiteInstruction::BrIf => 2,
            WasmLiteInstruction::Return => 1,
            WasmLiteInstruction::Call | WasmLiteInstruction::CallIndirect => 5,
            
            WasmLiteInstruction::LocalGet | WasmLiteInstruction::LocalSet | 
            WasmLiteInstruction::LocalTee => 1,
            WasmLiteInstruction::GlobalGet | WasmLiteInstruction::GlobalSet => 3,
            
            WasmLiteInstruction::I32Load | WasmLiteInstruction::I64Load => 3,
            WasmLiteInstruction::I32Store | WasmLiteInstruction::I64Store => 3,
            
            WasmLiteInstruction::I32Const | WasmLiteInstruction::I64Const => 1,
            
            WasmLiteInstruction::I32Add | WasmLiteInstruction::I32Sub | 
            WasmLiteInstruction::I32And | WasmLiteInstruction::I32Or | 
            WasmLiteInstruction::I32Xor => 3,
            
            WasmLiteInstruction::I32Mul => 5,
            WasmLiteInstruction::I32DivS | WasmLiteInstruction::I32DivU | 
            WasmLiteInstruction::I32RemS | WasmLiteInstruction::I32RemU => 8,
            
            WasmLiteInstruction::I32Shl | WasmLiteInstruction::I32ShrS | 
            WasmLiteInstruction::I32ShrU => 3,
            
            WasmLiteInstruction::I32Eqz | WasmLiteInstruction::I32Eq | 
            WasmLiteInstruction::I32Ne | WasmLiteInstruction::I32LtS | 
            WasmLiteInstruction::I32LtU | WasmLiteInstruction::I32GtS | 
            WasmLiteInstruction::I32GtU | WasmLiteInstruction::I32LeS | 
            WasmLiteInstruction::I32LeU | WasmLiteInstruction::I32GeS | 
            WasmLiteInstruction::I32GeU => 3,
            
            // Blockchain-specific operations
            WasmLiteInstruction::Keccak256 => 30,
            WasmLiteInstruction::EcRecover => 3000,
            WasmLiteInstruction::Blake2b => 60,
            WasmLiteInstruction::Ed25519Verify => 2000,
            WasmLiteInstruction::GetCaller | WasmLiteInstruction::GetValue | 
            WasmLiteInstruction::GetGasRemaining => 2,
            WasmLiteInstruction::StorageLoad => 100,
            WasmLiteInstruction::StorageStore => 5000,
            WasmLiteInstruction::EmitLog => 375,
            WasmLiteInstruction::Transfer => 25000,
        }
    }
}

impl WasmLiteValue {
    /// Get the type of the value
    pub fn value_type(&self) -> WasmLiteValueType {
        match self {
            WasmLiteValue::I32(_) => WasmLiteValueType::I32,
            WasmLiteValue::I64(_) => WasmLiteValueType::I64,
            WasmLiteValue::Bytes(_) => WasmLiteValueType::Bytes,
        }
    }

    /// Convert to i32 if possible
    pub fn as_i32(&self) -> Result<i32, RvmError> {
        match self {
            WasmLiteValue::I32(v) => Ok(*v),
            _ => Err(RvmError::WasmLiteTypeError),
        }
    }

    /// Convert to i64 if possible
    pub fn as_i64(&self) -> Result<i64, RvmError> {
        match self {
            WasmLiteValue::I64(v) => Ok(*v),
            WasmLiteValue::I32(v) => Ok(*v as i64),
            _ => Err(RvmError::WasmLiteTypeError),
        }
    }

    /// Convert to bytes if possible
    pub fn as_bytes(&self) -> Result<&[u8], RvmError> {
        match self {
            WasmLiteValue::Bytes(v) => Ok(v),
            _ => Err(RvmError::WasmLiteTypeError),
        }
    }
}

impl WasmLiteVM {
    /// Create a new WASM-lite VM
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            context: None,
        }
    }

    /// Load a WASM-lite module
    pub fn load_module(&mut self, name: String, module: WasmLiteModule) -> Result<(), RvmError> {
        // Validate module
        if module.version != 1 {
            return Err(RvmError::UnsupportedWasmLiteVersion(module.version));
        }

        // Check memory constraints
        if module.memory_pages > (crate::WASM_LITE_MAX_MEMORY / crate::WASM_LITE_PAGE_SIZE) as u32 {
            return Err(RvmError::WasmLiteMemoryLimit);
        }

        if module.functions.len() > crate::WASM_LITE_MAX_FUNCTIONS {
            return Err(RvmError::WasmLiteFunctionLimit);
        }

        self.modules.insert(name, module);
        Ok(())
    }

    /// Execute a function in a loaded module
    pub async fn execute_function(
        &mut self,
        module_name: &str,
        function_name: &str,
        args: Vec<WasmLiteValue>,
        gas_limit: u64,
        env: ExecutionEnvironment,
    ) -> Result<ExecutionResult, RvmError> {
        let module = self.modules.get(module_name)
            .ok_or_else(|| RvmError::WasmLiteModuleNotFound(module_name.to_string()))?;

        let function_index = module.exports.get(function_name)
            .ok_or_else(|| RvmError::WasmLiteFunctionNotFound(function_name.to_string()))?;

        let function = &module.functions[*function_index];

        // Validate arguments
        if args.len() != function.params.len() {
            return Err(RvmError::WasmLiteArgumentMismatch);
        }

        for (arg, expected_type) in args.iter().zip(&function.params) {
            if arg.value_type() != *expected_type {
                return Err(RvmError::WasmLiteTypeError);
            }
        }

        // Create execution context
        let mut context = WasmLiteContext {
            stack: Vec::new(),
            locals: args,
            globals: module.globals.clone(),
            memory: vec![0; module.memory_pages as usize * crate::WASM_LITE_PAGE_SIZE],
            pc: 0,
            call_stack: Vec::new(),
            gas: GasMeter::new(gas_limit),
            env,
        };

        // Execute function
        let result = self.execute_function_body(&function.body, &mut context).await?;

        Ok(ExecutionResult {
            return_data: result,
            gas_used: context.gas.used(),
            success: true,
            error: None,
        })
    }

    /// Execute function body
    async fn execute_function_body(
        &self,
        bytecode: &[u8],
        context: &mut WasmLiteContext,
    ) -> Result<Vec<u8>, RvmError> {
        while context.pc < bytecode.len() {
            let instruction = WasmLiteInstruction::from_byte(bytecode[context.pc])?;
            
            // Charge gas
            context.gas.consume(instruction.gas_cost())?;
            
            match self.execute_instruction(instruction, bytecode, context).await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        // Return top stack value as bytes
        if let Some(value) = context.stack.pop() {
            match value {
                WasmLiteValue::I32(v) => Ok(v.to_le_bytes().to_vec()),
                WasmLiteValue::I64(v) => Ok(v.to_le_bytes().to_vec()),
                WasmLiteValue::Bytes(v) => Ok(v),
            }
        } else {
            Ok(vec![])
        }
    }

    /// Execute a single instruction
    async fn execute_instruction(
        &self,
        instruction: WasmLiteInstruction,
        bytecode: &[u8],
        context: &mut WasmLiteContext,
    ) -> Result<bool, RvmError> {
        match instruction {
            WasmLiteInstruction::Nop => {
                context.pc += 1;
            }
            
            WasmLiteInstruction::I64Const => {
                if context.pc + 8 >= bytecode.len() {
                    return Err(RvmError::InvalidWasmLiteBytecode);
                }
                let value = i64::from_le_bytes([
                    bytecode[context.pc + 1],
                    bytecode[context.pc + 2],
                    bytecode[context.pc + 3],
                    bytecode[context.pc + 4],
                    bytecode[context.pc + 5],
                    bytecode[context.pc + 6],
                    bytecode[context.pc + 7],
                    bytecode[context.pc + 8],
                ]);
                context.stack.push(WasmLiteValue::I64(value));
                context.pc += 9;
            }
            
            WasmLiteInstruction::I32Add => {
                let b = context.stack.pop().ok_or(RvmError::WasmLiteStackUnderflow)?.as_i32()?;
                let a = context.stack.pop().ok_or(RvmError::WasmLiteStackUnderflow)?.as_i32()?;
                context.stack.push(WasmLiteValue::I32(a.wrapping_add(b)));
                context.pc += 1;
            }
            
            WasmLiteInstruction::I32Sub => {
                let b = context.stack.pop().ok_or(RvmError::WasmLiteStackUnderflow)?.as_i32()?;
                let a = context.stack.pop().ok_or(RvmError::WasmLiteStackUnderflow)?.as_i32()?;
                context.stack.push(WasmLiteValue::I32(a.wrapping_sub(b)));
                context.pc += 1;
            }
            
            WasmLiteInstruction::I32Mul => {
                let b = context.stack.pop().ok_or(RvmError::WasmLiteStackUnderflow)?.as_i32()?;
                let a = context.stack.pop().ok_or(RvmError::WasmLiteStackUnderflow)?.as_i32()?;
                context.stack.push(WasmLiteValue::I32(a.wrapping_mul(b)));
                context.pc += 1;
            }
            
            WasmLiteInstruction::Return => {
                return Ok(false);
            }

            // Blockchain-specific instructions
            WasmLiteInstruction::Keccak256 => {
                let value = context.stack.pop().ok_or(RvmError::WasmLiteStackUnderflow)?;
                let data = value.as_bytes()?;
                let hash = crate::crypto::RvmCrypto::keccak256(data);
                context.stack.push(WasmLiteValue::Bytes(hash.to_vec()));
                context.pc += 1;
            }
            
            WasmLiteInstruction::GetCaller => {
                context.stack.push(WasmLiteValue::Bytes(context.env.caller.to_vec()));
                context.pc += 1;
            }
            
            WasmLiteInstruction::GetValue => {
                context.stack.push(WasmLiteValue::I64(context.env.value as i64));
                context.pc += 1;
            }
            
            WasmLiteInstruction::LocalGet => {
                if context.pc + 1 >= bytecode.len() {
                    return Err(RvmError::InvalidWasmLiteBytecode);
                }
                let local_index = bytecode[context.pc + 1] as usize;
                if local_index >= context.locals.len() {
                    return Err(RvmError::WasmLiteStackUnderflow);
                }
                context.stack.push(context.locals[local_index].clone());
                context.pc += 2;
            }
            
            WasmLiteInstruction::GetGasRemaining => {
                context.stack.push(WasmLiteValue::I64(context.gas.remaining() as i64));
                context.pc += 1;
            }

            _ => {
                // TODO: Implement remaining instructions
                context.pc += 1;
            }
        }
        
        Ok(true)
    }

    /// Create a simple WASM-lite demo module
    pub fn create_demo_module() -> WasmLiteModule {
        // Simple function that adds two numbers: (a: i32, b: i32) -> i32
        // This function takes two parameters from the stack and adds them
        let add_function = WasmLiteFunction {
            name: "add".to_string(),
            params: vec![WasmLiteValueType::I32, WasmLiteValueType::I32],
            returns: vec![WasmLiteValueType::I32],
            body: vec![
                0x20, 0x00,  // local.get 0 (first parameter)
                0x20, 0x01,  // local.get 1 (second parameter)
                0x6a,        // i32.add
                0x0f,        // return
            ],
            locals: vec![],
        };

        let mut exports = HashMap::new();
        exports.insert("add".to_string(), 0);

        WasmLiteModule {
            version: 1,
            functions: vec![add_function],
            globals: vec![],
            memory_pages: 1,
            exports,
            imports: HashMap::new(),
        }
    }
}

impl Default for WasmLiteVM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wasm_lite_basic() {
        let mut vm = WasmLiteVM::new();
        let module = WasmLiteVM::create_demo_module();
        
        vm.load_module("demo".to_string(), module).unwrap();
        
        let args = vec![WasmLiteValue::I32(10), WasmLiteValue::I32(20)];
        let env = ExecutionEnvironment::default();
        
        let result = vm.execute_function("demo", "add", args, 1000, env).await.unwrap();
        assert!(result.success);
        assert!(result.gas_used > 0);
    }

    #[test]
    fn test_wasm_lite_value_types() {
        let val_i32 = WasmLiteValue::I32(42);
        let val_i64 = WasmLiteValue::I64(100);
        let val_bytes = WasmLiteValue::Bytes(vec![1, 2, 3]);

        assert_eq!(val_i32.value_type(), WasmLiteValueType::I32);
        assert_eq!(val_i64.value_type(), WasmLiteValueType::I64);
        assert_eq!(val_bytes.value_type(), WasmLiteValueType::Bytes);
        
        assert_eq!(val_i32.as_i32().unwrap(), 42);
        assert_eq!(val_i64.as_i64().unwrap(), 100);
        assert_eq!(val_bytes.as_bytes().unwrap(), &[1, 2, 3]);
    }

    #[test]
    fn test_instruction_gas_costs() {
        assert_eq!(WasmLiteInstruction::Nop.gas_cost(), 0);
        assert_eq!(WasmLiteInstruction::I32Add.gas_cost(), 3);
        assert_eq!(WasmLiteInstruction::I32Mul.gas_cost(), 5);
        assert_eq!(WasmLiteInstruction::Keccak256.gas_cost(), 30);
        assert_eq!(WasmLiteInstruction::EcRecover.gas_cost(), 3000);
    }
}
