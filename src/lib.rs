//! RVM - The Rust Virtual Machine
//! 
//! A robust, extensible, and secure virtual machine engine built in Rust.
//! Designed for blockchain, agent, and cloud-native systems with deterministic execution.

pub mod core;
pub mod runtime;
pub mod revm;
pub mod opcodes;
pub mod gas;
pub mod ghostchain_gas;
pub mod storage;
pub mod crypto;
pub mod ghostchain_crypto;
pub mod ghostchain_services;
pub mod error;
pub mod wasm_lite;

pub use core::*;
pub use runtime::*;
pub use revm::*;
pub use wasm_lite::*;
pub use error::RvmError;
pub use ghostchain_gas::*;
pub use ghostchain_crypto::*;
pub use ghostchain_services::*;

/// RVM version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default gas limit for execution
pub const DEFAULT_GAS_LIMIT: u64 = 21_000_000;

/// Maximum stack size
pub const MAX_STACK_SIZE: usize = 1024;

/// Maximum call depth for contracts
pub const MAX_CALL_DEPTH: usize = 256;

/// WASM-lite specific constants
pub const WASM_LITE_MAX_MEMORY: usize = 16 * 1024 * 1024; // 16MB max memory
pub const WASM_LITE_PAGE_SIZE: usize = 64 * 1024; // 64KB pages
pub const WASM_LITE_MAX_FUNCTIONS: usize = 1024; // Max functions per module
