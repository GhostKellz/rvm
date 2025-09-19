//! RVM Opcodes
//!
//! Complete opcode definitions for RVM bytecode execution

use crate::error::RvmError;

/// RVM opcodes with gas costs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // Arithmetic Operations
    STOP = 0x00,        // Gas: 0
    ADD = 0x01,         // Gas: 3
    MUL = 0x02,         // Gas: 5
    SUB = 0x03,         // Gas: 3
    DIV = 0x04,         // Gas: 5
    SDIV = 0x05,        // Gas: 5
    MOD = 0x06,         // Gas: 5
    SMOD = 0x07,        // Gas: 5
    ADDMOD = 0x08,      // Gas: 8
    MULMOD = 0x09,      // Gas: 8
    EXP = 0x0a,         // Gas: 10
    SIGNEXTEND = 0x0b,  // Gas: 5

    // Comparison Operations
    LT = 0x10,          // Gas: 3
    GT = 0x11,          // Gas: 3
    SLT = 0x12,         // Gas: 3
    SGT = 0x13,         // Gas: 3
    EQ = 0x14,          // Gas: 3
    ISZERO = 0x15,      // Gas: 3
    AND = 0x16,         // Gas: 3
    OR = 0x17,          // Gas: 3
    XOR = 0x18,         // Gas: 3
    NOT = 0x19,         // Gas: 3
    BYTE = 0x1a,        // Gas: 3

    // Crypto Operations
    KECCAK256 = 0x20,   // Gas: 30

    // Environment Information
    ADDRESS = 0x30,     // Gas: 2
    BALANCE = 0x31,     // Gas: 100
    ORIGIN = 0x32,      // Gas: 2
    CALLER = 0x33,      // Gas: 2
    CALLVALUE = 0x34,   // Gas: 2
    CALLDATALOAD = 0x35, // Gas: 3
    CALLDATASIZE = 0x36, // Gas: 2
    CALLDATACOPY = 0x37, // Gas: 3
    CODESIZE = 0x38,    // Gas: 2
    CODECOPY = 0x39,    // Gas: 3
    GASPRICE = 0x3a,    // Gas: 2
    EXTCODESIZE = 0x3b, // Gas: 100
    EXTCODECOPY = 0x3c, // Gas: 100

    // Block Information
    BLOCKHASH = 0x40,   // Gas: 20
    COINBASE = 0x41,    // Gas: 2
    TIMESTAMP = 0x42,   // Gas: 2
    NUMBER = 0x43,      // Gas: 2
    DIFFICULTY = 0x44,  // Gas: 2
    GASLIMIT = 0x45,    // Gas: 2

    // Stack Operations
    POP = 0x50,         // Gas: 2
    MLOAD = 0x51,       // Gas: 3
    MSTORE = 0x52,      // Gas: 3
    MSTORE8 = 0x53,     // Gas: 3
    SLOAD = 0x54,       // Gas: 100
    SSTORE = 0x55,      // Gas: 100/5000
    JUMP = 0x56,        // Gas: 8
    JUMPI = 0x57,       // Gas: 10
    PC = 0x58,          // Gas: 2
    MSIZE = 0x59,       // Gas: 2
    GAS = 0x5a,         // Gas: 2
    JUMPDEST = 0x5b,    // Gas: 1

    // Push Operations
    PUSH1 = 0x60,       // Gas: 3
    PUSH2 = 0x61,       // Gas: 3
    PUSH3 = 0x62,       // Gas: 3
    PUSH4 = 0x63,       // Gas: 3
    PUSH5 = 0x64,       // Gas: 3
    PUSH6 = 0x65,       // Gas: 3
    PUSH7 = 0x66,       // Gas: 3
    PUSH8 = 0x67,       // Gas: 3
    PUSH9 = 0x68,       // Gas: 3
    PUSH10 = 0x69,      // Gas: 3
    PUSH11 = 0x6a,      // Gas: 3
    PUSH12 = 0x6b,      // Gas: 3
    PUSH13 = 0x6c,      // Gas: 3
    PUSH14 = 0x6d,      // Gas: 3
    PUSH15 = 0x6e,      // Gas: 3
    PUSH16 = 0x6f,      // Gas: 3
    PUSH17 = 0x70,      // Gas: 3
    PUSH18 = 0x71,      // Gas: 3
    PUSH19 = 0x72,      // Gas: 3
    PUSH20 = 0x73,      // Gas: 3
    PUSH21 = 0x74,      // Gas: 3
    PUSH22 = 0x75,      // Gas: 3
    PUSH23 = 0x76,      // Gas: 3
    PUSH24 = 0x77,      // Gas: 3
    PUSH25 = 0x78,      // Gas: 3
    PUSH26 = 0x79,      // Gas: 3
    PUSH27 = 0x7a,      // Gas: 3
    PUSH28 = 0x7b,      // Gas: 3
    PUSH29 = 0x7c,      // Gas: 3
    PUSH30 = 0x7d,      // Gas: 3
    PUSH31 = 0x7e,      // Gas: 3
    PUSH32 = 0x7f,      // Gas: 3

    // Duplication Operations
    DUP1 = 0x80,        // Gas: 3
    DUP2 = 0x81,        // Gas: 3
    DUP3 = 0x82,        // Gas: 3
    DUP4 = 0x83,        // Gas: 3
    DUP5 = 0x84,        // Gas: 3
    DUP6 = 0x85,        // Gas: 3
    DUP7 = 0x86,        // Gas: 3
    DUP8 = 0x87,        // Gas: 3
    DUP9 = 0x88,        // Gas: 3
    DUP10 = 0x89,       // Gas: 3
    DUP11 = 0x8a,       // Gas: 3
    DUP12 = 0x8b,       // Gas: 3
    DUP13 = 0x8c,       // Gas: 3
    DUP14 = 0x8d,       // Gas: 3
    DUP15 = 0x8e,       // Gas: 3
    DUP16 = 0x8f,       // Gas: 3

    // Exchange Operations
    SWAP1 = 0x90,       // Gas: 3
    SWAP2 = 0x91,       // Gas: 3
    SWAP3 = 0x92,       // Gas: 3
    SWAP4 = 0x93,       // Gas: 3
    SWAP5 = 0x94,       // Gas: 3
    SWAP6 = 0x95,       // Gas: 3
    SWAP7 = 0x96,       // Gas: 3
    SWAP8 = 0x97,       // Gas: 3
    SWAP9 = 0x98,       // Gas: 3
    SWAP10 = 0x99,      // Gas: 3
    SWAP11 = 0x9a,      // Gas: 3
    SWAP12 = 0x9b,      // Gas: 3
    SWAP13 = 0x9c,      // Gas: 3
    SWAP14 = 0x9d,      // Gas: 3
    SWAP15 = 0x9e,      // Gas: 3
    SWAP16 = 0x9f,      // Gas: 3

    // Logging Operations
    LOG0 = 0xa0,        // Gas: 375
    LOG1 = 0xa1,        // Gas: 750
    LOG2 = 0xa2,        // Gas: 1125
    LOG3 = 0xa3,        // Gas: 1500
    LOG4 = 0xa4,        // Gas: 1875

    // System Operations
    CREATE = 0xf0,      // Gas: 32000
    CALL = 0xf1,        // Gas: 100
    CALLCODE = 0xf2,    // Gas: 100
    RETURN = 0xf3,      // Gas: 0
    DELEGATECALL = 0xf4, // Gas: 100
    CREATE2 = 0xf5,     // Gas: 32000
    STATICCALL = 0xfa,  // Gas: 100
    REVERT = 0xfd,      // Gas: 0
    INVALID = 0xfe,     // Gas: 0
    SELFDESTRUCT = 0xff, // Gas: 5000

    // GhostChain-specific opcodes (custom range)
    // Identity operations
    GHOST_ID_VERIFY = 0xc0,     // Gas: 1000 - Verify GhostID signature
    GHOST_ID_RESOLVE = 0xc1,    // Gas: 500 - Resolve GhostID to address
    GHOST_ID_CREATE = 0xc2,     // Gas: 2000 - Create new GhostID

    // Token operations (4-token economy)
    TOKEN_BALANCE = 0xc3,       // Gas: 100 - Get token balance (GCC/SPIRIT/MANA/GHOST)
    TOKEN_TRANSFER = 0xc4,      // Gas: 5000 - Transfer tokens between accounts
    TOKEN_MINT = 0xc5,          // Gas: 10000 - Mint new tokens (restricted)
    TOKEN_BURN = 0xc6,          // Gas: 5000 - Burn tokens

    // CNS operations
    CNS_RESOLVE = 0xc7,         // Gas: 300 - Resolve domain to address
    CNS_REGISTER = 0xc8,        // Gas: 20000 - Register new domain
    CNS_UPDATE = 0xc9,          // Gas: 5000 - Update domain records
    CNS_OWNER = 0xca,           // Gas: 100 - Get domain owner

    // L2 operations
    L2_SUBMIT = 0xcb,           // Gas: 2000 - Submit transaction to L2
    L2_BATCH_VERIFY = 0xcc,     // Gas: 50000 - Verify L2 batch proof
    L2_STATE_SYNC = 0xcd,       // Gas: 10000 - Sync L1/L2 state

    // Cross-chain operations
    BRIDGE_SEND = 0xce,         // Gas: 15000 - Send cross-chain transaction
    BRIDGE_RECEIVE = 0xcf,      // Gas: 10000 - Receive cross-chain transaction

    // AI/Agent operations (Jarvis integration)
    AGENT_CALL = 0xd0,          // Gas: 5000 - Call AI agent function
    AGENT_DEPLOY = 0xd1,        // Gas: 50000 - Deploy AI agent
    AGENT_QUERY = 0xd2,         // Gas: 1000 - Query agent state
}

impl Opcode {
    /// Convert byte to opcode
    pub fn from_byte(byte: u8) -> Result<Self, RvmError> {
        match byte {
            0x00 => Ok(Opcode::STOP),
            0x01 => Ok(Opcode::ADD),
            0x02 => Ok(Opcode::MUL),
            0x03 => Ok(Opcode::SUB),
            0x04 => Ok(Opcode::DIV),
            0x05 => Ok(Opcode::SDIV),
            0x06 => Ok(Opcode::MOD),
            0x07 => Ok(Opcode::SMOD),
            0x08 => Ok(Opcode::ADDMOD),
            0x09 => Ok(Opcode::MULMOD),
            0x0a => Ok(Opcode::EXP),
            0x0b => Ok(Opcode::SIGNEXTEND),
            0x10 => Ok(Opcode::LT),
            0x11 => Ok(Opcode::GT),
            0x12 => Ok(Opcode::SLT),
            0x13 => Ok(Opcode::SGT),
            0x14 => Ok(Opcode::EQ),
            0x15 => Ok(Opcode::ISZERO),
            0x16 => Ok(Opcode::AND),
            0x17 => Ok(Opcode::OR),
            0x18 => Ok(Opcode::XOR),
            0x19 => Ok(Opcode::NOT),
            0x1a => Ok(Opcode::BYTE),
            0x20 => Ok(Opcode::KECCAK256),
            0x30 => Ok(Opcode::ADDRESS),
            0x31 => Ok(Opcode::BALANCE),
            0x32 => Ok(Opcode::ORIGIN),
            0x33 => Ok(Opcode::CALLER),
            0x34 => Ok(Opcode::CALLVALUE),
            0x35 => Ok(Opcode::CALLDATALOAD),
            0x36 => Ok(Opcode::CALLDATASIZE),
            0x37 => Ok(Opcode::CALLDATACOPY),
            0x38 => Ok(Opcode::CODESIZE),
            0x39 => Ok(Opcode::CODECOPY),
            0x3a => Ok(Opcode::GASPRICE),
            0x3b => Ok(Opcode::EXTCODESIZE),
            0x3c => Ok(Opcode::EXTCODECOPY),
            0x40 => Ok(Opcode::BLOCKHASH),
            0x41 => Ok(Opcode::COINBASE),
            0x42 => Ok(Opcode::TIMESTAMP),
            0x43 => Ok(Opcode::NUMBER),
            0x44 => Ok(Opcode::DIFFICULTY),
            0x45 => Ok(Opcode::GASLIMIT),
            0x50 => Ok(Opcode::POP),
            0x51 => Ok(Opcode::MLOAD),
            0x52 => Ok(Opcode::MSTORE),
            0x53 => Ok(Opcode::MSTORE8),
            0x54 => Ok(Opcode::SLOAD),
            0x55 => Ok(Opcode::SSTORE),
            0x56 => Ok(Opcode::JUMP),
            0x57 => Ok(Opcode::JUMPI),
            0x58 => Ok(Opcode::PC),
            0x59 => Ok(Opcode::MSIZE),
            0x5a => Ok(Opcode::GAS),
            0x5b => Ok(Opcode::JUMPDEST),
            0x60 => Ok(Opcode::PUSH1),
            0x61 => Ok(Opcode::PUSH2),
            0x62 => Ok(Opcode::PUSH3),
            0x63 => Ok(Opcode::PUSH4),
            0x64 => Ok(Opcode::PUSH5),
            0x65 => Ok(Opcode::PUSH6),
            0x66 => Ok(Opcode::PUSH7),
            0x67 => Ok(Opcode::PUSH8),
            0x68 => Ok(Opcode::PUSH9),
            0x69 => Ok(Opcode::PUSH10),
            0x6a => Ok(Opcode::PUSH11),
            0x6b => Ok(Opcode::PUSH12),
            0x6c => Ok(Opcode::PUSH13),
            0x6d => Ok(Opcode::PUSH14),
            0x6e => Ok(Opcode::PUSH15),
            0x6f => Ok(Opcode::PUSH16),
            0x70 => Ok(Opcode::PUSH17),
            0x71 => Ok(Opcode::PUSH18),
            0x72 => Ok(Opcode::PUSH19),
            0x73 => Ok(Opcode::PUSH20),
            0x74 => Ok(Opcode::PUSH21),
            0x75 => Ok(Opcode::PUSH22),
            0x76 => Ok(Opcode::PUSH23),
            0x77 => Ok(Opcode::PUSH24),
            0x78 => Ok(Opcode::PUSH25),
            0x79 => Ok(Opcode::PUSH26),
            0x7a => Ok(Opcode::PUSH27),
            0x7b => Ok(Opcode::PUSH28),
            0x7c => Ok(Opcode::PUSH29),
            0x7d => Ok(Opcode::PUSH30),
            0x7e => Ok(Opcode::PUSH31),
            0x7f => Ok(Opcode::PUSH32),
            0x80 => Ok(Opcode::DUP1),
            0x81 => Ok(Opcode::DUP2),
            0x82 => Ok(Opcode::DUP3),
            0x83 => Ok(Opcode::DUP4),
            0x84 => Ok(Opcode::DUP5),
            0x85 => Ok(Opcode::DUP6),
            0x86 => Ok(Opcode::DUP7),
            0x87 => Ok(Opcode::DUP8),
            0x88 => Ok(Opcode::DUP9),
            0x89 => Ok(Opcode::DUP10),
            0x8a => Ok(Opcode::DUP11),
            0x8b => Ok(Opcode::DUP12),
            0x8c => Ok(Opcode::DUP13),
            0x8d => Ok(Opcode::DUP14),
            0x8e => Ok(Opcode::DUP15),
            0x8f => Ok(Opcode::DUP16),
            0x90 => Ok(Opcode::SWAP1),
            0x91 => Ok(Opcode::SWAP2),
            0x92 => Ok(Opcode::SWAP3),
            0x93 => Ok(Opcode::SWAP4),
            0x94 => Ok(Opcode::SWAP5),
            0x95 => Ok(Opcode::SWAP6),
            0x96 => Ok(Opcode::SWAP7),
            0x97 => Ok(Opcode::SWAP8),
            0x98 => Ok(Opcode::SWAP9),
            0x99 => Ok(Opcode::SWAP10),
            0x9a => Ok(Opcode::SWAP11),
            0x9b => Ok(Opcode::SWAP12),
            0x9c => Ok(Opcode::SWAP13),
            0x9d => Ok(Opcode::SWAP14),
            0x9e => Ok(Opcode::SWAP15),
            0x9f => Ok(Opcode::SWAP16),
            0xa0 => Ok(Opcode::LOG0),
            0xa1 => Ok(Opcode::LOG1),
            0xa2 => Ok(Opcode::LOG2),
            0xa3 => Ok(Opcode::LOG3),
            0xa4 => Ok(Opcode::LOG4),
            0xf0 => Ok(Opcode::CREATE),
            0xf1 => Ok(Opcode::CALL),
            0xf2 => Ok(Opcode::CALLCODE),
            0xf3 => Ok(Opcode::RETURN),
            0xf4 => Ok(Opcode::DELEGATECALL),
            0xf5 => Ok(Opcode::CREATE2),
            0xfa => Ok(Opcode::STATICCALL),
            0xfd => Ok(Opcode::REVERT),
            0xfe => Ok(Opcode::INVALID),
            0xff => Ok(Opcode::SELFDESTRUCT),

            // GhostChain-specific opcodes
            0xc0 => Ok(Opcode::GHOST_ID_VERIFY),
            0xc1 => Ok(Opcode::GHOST_ID_RESOLVE),
            0xc2 => Ok(Opcode::GHOST_ID_CREATE),
            0xc3 => Ok(Opcode::TOKEN_BALANCE),
            0xc4 => Ok(Opcode::TOKEN_TRANSFER),
            0xc5 => Ok(Opcode::TOKEN_MINT),
            0xc6 => Ok(Opcode::TOKEN_BURN),
            0xc7 => Ok(Opcode::CNS_RESOLVE),
            0xc8 => Ok(Opcode::CNS_REGISTER),
            0xc9 => Ok(Opcode::CNS_UPDATE),
            0xca => Ok(Opcode::CNS_OWNER),
            0xcb => Ok(Opcode::L2_SUBMIT),
            0xcc => Ok(Opcode::L2_BATCH_VERIFY),
            0xcd => Ok(Opcode::L2_STATE_SYNC),
            0xce => Ok(Opcode::BRIDGE_SEND),
            0xcf => Ok(Opcode::BRIDGE_RECEIVE),
            0xd0 => Ok(Opcode::AGENT_CALL),
            0xd1 => Ok(Opcode::AGENT_DEPLOY),
            0xd2 => Ok(Opcode::AGENT_QUERY),

            _ => Err(RvmError::InvalidOpcode(byte)),
        }
    }

    /// Get gas cost for opcode
    pub fn gas_cost(&self) -> u64 {
        match self {
            Opcode::STOP => 0,
            Opcode::ADD | Opcode::SUB | Opcode::LT | Opcode::GT | Opcode::SLT | 
            Opcode::SGT | Opcode::EQ | Opcode::ISZERO | Opcode::AND | Opcode::OR | 
            Opcode::XOR | Opcode::NOT | Opcode::BYTE => 3,
            
            Opcode::MUL | Opcode::DIV | Opcode::SDIV | Opcode::MOD | Opcode::SMOD | 
            Opcode::SIGNEXTEND => 5,
            
            Opcode::ADDMOD | Opcode::MULMOD | Opcode::JUMP => 8,
            Opcode::EXP => 10,
            Opcode::JUMPI => 10,
            
            Opcode::KECCAK256 => 30,
            
            Opcode::ADDRESS | Opcode::ORIGIN | Opcode::CALLER | Opcode::CALLVALUE | 
            Opcode::CALLDATASIZE | Opcode::CODESIZE | Opcode::GASPRICE | 
            Opcode::COINBASE | Opcode::TIMESTAMP | Opcode::NUMBER | 
            Opcode::DIFFICULTY | Opcode::GASLIMIT | Opcode::PC | Opcode::MSIZE | 
            Opcode::GAS => 2,
            
            Opcode::CALLDATALOAD | Opcode::CODECOPY | Opcode::MLOAD | Opcode::MSTORE | 
            Opcode::MSTORE8 | Opcode::CALLDATACOPY => 3,
            
            Opcode::POP => 2,
            Opcode::JUMPDEST => 1,
            
            Opcode::PUSH1 | Opcode::PUSH2 | Opcode::PUSH3 | Opcode::PUSH4 |
            Opcode::PUSH5 | Opcode::PUSH6 | Opcode::PUSH7 | Opcode::PUSH8 |
            Opcode::PUSH9 | Opcode::PUSH10 | Opcode::PUSH11 | Opcode::PUSH12 |
            Opcode::PUSH13 | Opcode::PUSH14 | Opcode::PUSH15 | Opcode::PUSH16 |
            Opcode::PUSH17 | Opcode::PUSH18 | Opcode::PUSH19 | Opcode::PUSH20 |
            Opcode::PUSH21 | Opcode::PUSH22 | Opcode::PUSH23 | Opcode::PUSH24 |
            Opcode::PUSH25 | Opcode::PUSH26 | Opcode::PUSH27 | Opcode::PUSH28 |
            Opcode::PUSH29 | Opcode::PUSH30 | Opcode::PUSH31 | Opcode::PUSH32 => 3,
            
            Opcode::DUP1 | Opcode::DUP2 | Opcode::DUP3 | Opcode::DUP4 |
            Opcode::DUP5 | Opcode::DUP6 | Opcode::DUP7 | Opcode::DUP8 |
            Opcode::DUP9 | Opcode::DUP10 | Opcode::DUP11 | Opcode::DUP12 |
            Opcode::DUP13 | Opcode::DUP14 | Opcode::DUP15 | Opcode::DUP16 => 3,
            
            Opcode::SWAP1 | Opcode::SWAP2 | Opcode::SWAP3 | Opcode::SWAP4 |
            Opcode::SWAP5 | Opcode::SWAP6 | Opcode::SWAP7 | Opcode::SWAP8 |
            Opcode::SWAP9 | Opcode::SWAP10 | Opcode::SWAP11 | Opcode::SWAP12 |
            Opcode::SWAP13 | Opcode::SWAP14 | Opcode::SWAP15 | Opcode::SWAP16 => 3,
            
            Opcode::BALANCE | Opcode::EXTCODESIZE | Opcode::EXTCODECOPY | 
            Opcode::CALL | Opcode::CALLCODE | Opcode::DELEGATECALL | 
            Opcode::STATICCALL => 100,
            
            Opcode::SLOAD => 100,
            Opcode::SSTORE => 100, // Simplified, actual cost depends on storage state
            
            Opcode::BLOCKHASH => 20,
            
            Opcode::LOG0 => 375,
            Opcode::LOG1 => 750,
            Opcode::LOG2 => 1125,
            Opcode::LOG3 => 1500,
            Opcode::LOG4 => 1875,
            
            Opcode::CREATE | Opcode::CREATE2 => 32000,
            Opcode::SELFDESTRUCT => 5000,

            Opcode::RETURN | Opcode::REVERT | Opcode::INVALID => 0,

            // GhostChain-specific opcode gas costs
            Opcode::GHOST_ID_VERIFY => 1000,
            Opcode::GHOST_ID_RESOLVE => 500,
            Opcode::GHOST_ID_CREATE => 2000,

            Opcode::TOKEN_BALANCE | Opcode::CNS_OWNER => 100,
            Opcode::CNS_RESOLVE => 300,
            Opcode::AGENT_QUERY => 1000,

            Opcode::L2_SUBMIT => 2000,
            Opcode::TOKEN_TRANSFER | Opcode::TOKEN_BURN | Opcode::CNS_UPDATE |
            Opcode::AGENT_CALL => 5000,

            Opcode::TOKEN_MINT | Opcode::L2_STATE_SYNC | Opcode::BRIDGE_RECEIVE => 10000,
            Opcode::CNS_REGISTER | Opcode::BRIDGE_SEND => 20000,

            Opcode::L2_BATCH_VERIFY | Opcode::AGENT_DEPLOY => 50000,
        }
    }

    /// Check if opcode is a PUSH instruction
    pub fn is_push(&self) -> bool {
        matches!(*self, 
            Opcode::PUSH1 | Opcode::PUSH2 | Opcode::PUSH3 | Opcode::PUSH4 |
            Opcode::PUSH5 | Opcode::PUSH6 | Opcode::PUSH7 | Opcode::PUSH8 |
            Opcode::PUSH9 | Opcode::PUSH10 | Opcode::PUSH11 | Opcode::PUSH12 |
            Opcode::PUSH13 | Opcode::PUSH14 | Opcode::PUSH15 | Opcode::PUSH16 |
            Opcode::PUSH17 | Opcode::PUSH18 | Opcode::PUSH19 | Opcode::PUSH20 |
            Opcode::PUSH21 | Opcode::PUSH22 | Opcode::PUSH23 | Opcode::PUSH24 |
            Opcode::PUSH25 | Opcode::PUSH26 | Opcode::PUSH27 | Opcode::PUSH28 |
            Opcode::PUSH29 | Opcode::PUSH30 | Opcode::PUSH31 | Opcode::PUSH32
        )
    }

    /// Get the number of bytes pushed by PUSH instruction
    pub fn push_bytes(&self) -> usize {
        match self {
            Opcode::PUSH1 => 1,
            Opcode::PUSH2 => 2,
            Opcode::PUSH3 => 3,
            Opcode::PUSH4 => 4,
            Opcode::PUSH5 => 5,
            Opcode::PUSH6 => 6,
            Opcode::PUSH7 => 7,
            Opcode::PUSH8 => 8,
            Opcode::PUSH9 => 9,
            Opcode::PUSH10 => 10,
            Opcode::PUSH11 => 11,
            Opcode::PUSH12 => 12,
            Opcode::PUSH13 => 13,
            Opcode::PUSH14 => 14,
            Opcode::PUSH15 => 15,
            Opcode::PUSH16 => 16,
            Opcode::PUSH17 => 17,
            Opcode::PUSH18 => 18,
            Opcode::PUSH19 => 19,
            Opcode::PUSH20 => 20,
            Opcode::PUSH21 => 21,
            Opcode::PUSH22 => 22,
            Opcode::PUSH23 => 23,
            Opcode::PUSH24 => 24,
            Opcode::PUSH25 => 25,
            Opcode::PUSH26 => 26,
            Opcode::PUSH27 => 27,
            Opcode::PUSH28 => 28,
            Opcode::PUSH29 => 29,
            Opcode::PUSH30 => 30,
            Opcode::PUSH31 => 31,
            Opcode::PUSH32 => 32,
            _ => 0,
        }
    }
}
