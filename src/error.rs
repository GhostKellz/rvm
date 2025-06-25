//! RVM Error Types
//!
//! Comprehensive error handling for the RVM ecosystem including core VM,
//! rEVM compatibility, WASM-lite, and runtime errors.

use thiserror::Error;

/// Main RVM error type
#[derive(Error, Debug, Clone)]
pub enum RvmError {
    // Core VM Errors
    #[error("Stack overflow")]
    StackOverflow,
    
    #[error("Stack underflow")]
    StackUnderflow,
    
    #[error("Out of gas: needed {needed}, available {available}")]
    OutOfGas { needed: u64, available: u64 },
    
    #[error("Invalid opcode: 0x{0:02x}")]
    InvalidOpcode(u8),
    
    #[error("Invalid bytecode: {0}")]
    InvalidBytecode(String),
    
    #[error("Invalid jump destination: {0}")]
    InvalidJump(usize),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    // Storage Errors
    #[error("Insufficient balance: available {available}, required {required}")]
    InsufficientBalance { available: u64, required: u64 },
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Contract not found: {0:02x?}")]
    ContractNotFound([u8; 20]),
    
    // Cryptography Errors
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Invalid precompile address: {0}")]
    InvalidPrecompile(u8),
    
    #[error("Precompiles are disabled")]
    PrecompilesDisabled,
    
    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),
    
    // rEVM Errors
    #[error("EVM execution failed: {0}")]
    EvmExecutionFailed(String),
    
    #[error("Invalid EVM transaction: {0}")]
    InvalidEvmTransaction(String),
    
    #[error("EVM state error: {0}")]
    EvmStateError(String),
    
    #[error("Block not found: {0}")]
    BlockNotFound(u64),
    
    // WASM-lite Errors
    #[error("Invalid WASM-lite instruction: 0x{0:02x}")]
    InvalidWasmLiteInstruction(u8),
    
    #[error("Invalid WASM-lite bytecode")]
    InvalidWasmLiteBytecode,
    
    #[error("WASM-lite stack underflow")]
    WasmLiteStackUnderflow,
    
    #[error("WASM-lite type error")]
    WasmLiteTypeError,
    
    #[error("WASM-lite module not found: {0}")]
    WasmLiteModuleNotFound(String),
    
    #[error("WASM-lite function not found: {0}")]
    WasmLiteFunctionNotFound(String),
    
    #[error("WASM-lite argument mismatch")]
    WasmLiteArgumentMismatch,
    
    #[error("Unsupported WASM-lite version: {0}")]
    UnsupportedWasmLiteVersion(u32),
    
    #[error("WASM-lite memory limit exceeded")]
    WasmLiteMemoryLimit,
    
    #[error("WASM-lite function limit exceeded")]
    WasmLiteFunctionLimit,
    
    // Runtime Errors
    #[error("Runtime not initialized")]
    RuntimeNotInitialized,
    
    #[error("Hook execution failed: {0}")]
    HookExecutionFailed(String),
    
    #[error("Agent API error: {0}")]
    AgentApiError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    // Gas Metering Errors
    #[error("Gas limit too low: {0}")]
    GasLimitTooLow(u64),
    
    #[error("Gas price too low: {0}")]
    GasPriceTooLow(u64),
    
    // Memory Errors
    #[error("Memory access out of bounds: offset {offset}, size {size}, memory_size {memory_size}")]
    MemoryOutOfBounds { offset: usize, size: usize, memory_size: usize },
    
    #[error("Memory allocation failed: {0}")]
    MemoryAllocationFailed(String),
    
    // Call Stack Errors
    #[error("Call stack overflow: depth {0}")]
    CallStackOverflow(usize),
    
    #[error("Call stack underflow")]
    CallStackUnderflow,
    
    #[error("Invalid call target: {0:02x?}")]
    InvalidCallTarget([u8; 20]),
    
    // Serialization Errors
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    // Network/Communication Errors
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    // Development/Debug Errors
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    #[error("Debug assertion failed: {0}")]
    DebugAssertionFailed(String),
    
    // External Integration Errors
    #[error("External call failed: {0}")]
    ExternalCallFailed(String),
    
    #[error("GhostChain integration error: {0}")]
    GhostChainError(String),
    
    #[error("Tokio runtime error: {0}")]
    TokioError(String),
    
    // Custom/Generic Errors
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for RVM operations
pub type RvmResult<T> = Result<T, RvmError>;
