//! Cryptographic Operations
//!
//! Provides cryptographic primitives for RVM including hashing, signature verification,
//! and other crypto operations needed for blockchain functionality.

use crate::error::RvmError;
use k256::{ecdsa::{RecoveryId, Signature, VerifyingKey}, elliptic_curve::sec1::ToEncodedPoint};
use sha3::{Digest, Keccak256};
use serde::{Deserialize, Serialize};

/// Cryptographic operations for RVM
pub struct RvmCrypto;

/// ECDSA signature with recovery information
#[derive(Debug, Clone)]
pub struct RecoverableSignature {
    /// The signature
    pub signature: [u8; 64],
    /// Recovery ID
    pub recovery_id: u8,
}

impl RvmCrypto {
    /// Compute Keccak256 hash
    pub fn keccak256(data: &[u8]) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    /// Compute Keccak256 hash and return as u64 (for simple cases)
    pub fn keccak256_u64(data: &[u8]) -> u64 {
        let hash = Self::keccak256(data);
        u64::from_be_bytes([
            hash[0], hash[1], hash[2], hash[3],
            hash[4], hash[5], hash[6], hash[7],
        ])
    }

    /// Recover public key from ECDSA signature
    pub fn ecrecover(
        hash: &[u8; 32],
        signature: &[u8; 64],
        recovery_id: u8,
    ) -> Result<[u8; 64], RvmError> {
        // Create signature from bytes
        let sig = Signature::try_from(signature.as_slice())
            .map_err(|_| RvmError::InvalidSignature)?;

        // Create recovery ID
        let recovery_id = RecoveryId::try_from(recovery_id)
            .map_err(|_| RvmError::InvalidSignature)?;

        // Recover the verifying key
        let recovered_key = VerifyingKey::recover_from_prehash(hash, &sig, recovery_id)
            .map_err(|_| RvmError::InvalidSignature)?;

        // Convert to uncompressed public key bytes (64 bytes without prefix)
        let encoded_point = recovered_key.to_encoded_point(false);
        let public_key_bytes = encoded_point.as_bytes();
        
        if public_key_bytes.len() != 65 || public_key_bytes[0] != 0x04 {
            return Err(RvmError::InvalidSignature);
        }

        // Return the 64-byte public key (without the 0x04 prefix)
        let mut public_key = [0u8; 64];
        public_key.copy_from_slice(&public_key_bytes[1..]);
        
        Ok(public_key)
    }

    /// Derive Ethereum address from public key
    pub fn public_key_to_address(public_key: &[u8; 64]) -> [u8; 20] {
        let hash = Self::keccak256(public_key);
        let mut address = [0u8; 20];
        address.copy_from_slice(&hash[12..]);
        address
    }

    /// Verify ECDSA signature
    pub fn verify_signature(
        hash: &[u8; 32],
        signature: &[u8; 64],
        recovery_id: u8,
        expected_address: &[u8; 20],
    ) -> Result<bool, RvmError> {
        let recovered_pubkey = Self::ecrecover(hash, signature, recovery_id)?;
        let recovered_address = Self::public_key_to_address(&recovered_pubkey);
        Ok(recovered_address == *expected_address)
    }

    /// Create a deterministic address from creator and nonce (CREATE opcode)
    pub fn create_address(creator: &[u8; 20], nonce: u64) -> [u8; 20] {
        use std::io::Write;
        
        let mut data = Vec::new();
        data.write_all(creator).unwrap();
        data.write_all(&nonce.to_be_bytes()).unwrap();
        
        let hash = Self::keccak256(&data);
        let mut address = [0u8; 20];
        address.copy_from_slice(&hash[12..]);
        address
    }

    /// Create a deterministic address from creator, salt, and init code hash (CREATE2 opcode)
    pub fn create2_address(
        creator: &[u8; 20],
        salt: &[u8; 32],
        init_code_hash: &[u8; 32],
    ) -> [u8; 20] {
        use std::io::Write;
        
        let mut data = Vec::new();
        data.write_all(&[0xff]).unwrap(); // CREATE2 prefix
        data.write_all(creator).unwrap();
        data.write_all(salt).unwrap();
        data.write_all(init_code_hash).unwrap();
        
        let hash = Self::keccak256(&data);
        let mut address = [0u8; 20];
        address.copy_from_slice(&hash[12..]);
        address
    }

    /// Simple hash function for general use
    pub fn hash(data: &[u8]) -> u64 {
        Self::keccak256_u64(data)
    }

    /// Generate a pseudo-random number (for testing purposes)
    pub fn pseudo_random(seed: u64) -> u64 {
        // Simple LCG for deterministic randomness
        seed.wrapping_mul(1103515245).wrapping_add(12345)
    }

    /// Merkle root calculation for simple binary tree
    pub fn merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
        if leaves.is_empty() {
            return [0u8; 32];
        }
        
        if leaves.len() == 1 {
            return leaves[0];
        }

        let mut current_level = leaves.to_vec();
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in current_level.chunks(2) {
                let hash = if chunk.len() == 2 {
                    // Combine two hashes
                    let mut combined = Vec::new();
                    combined.extend_from_slice(&chunk[0]);
                    combined.extend_from_slice(&chunk[1]);
                    Self::keccak256(&combined)
                } else {
                    // Odd number, hash with itself
                    chunk[0]
                };
                next_level.push(hash);
            }
            
            current_level = next_level;
        }
        
        current_level[0]
    }

    /// Verify Merkle proof
    pub fn verify_merkle_proof(
        leaf: &[u8; 32],
        proof: &[[u8; 32]],
        root: &[u8; 32],
        index: usize,
    ) -> bool {
        let mut current_hash = *leaf;
        let mut current_index = index;
        
        for proof_element in proof {
            let mut combined = Vec::new();
            
            if current_index % 2 == 0 {
                // Current hash is left, proof element is right
                combined.extend_from_slice(&current_hash);
                combined.extend_from_slice(proof_element);
            } else {
                // Current hash is right, proof element is left
                combined.extend_from_slice(proof_element);
                combined.extend_from_slice(&current_hash);
            }
            
            current_hash = Self::keccak256(&combined);
            current_index /= 2;
        }
        
        current_hash == *root
    }
}

/// Precompiled contracts for common crypto operations
pub struct Precompiles;

impl Precompiles {
    /// ECRECOVER precompile (address 0x01)
    pub fn ecrecover(input: &[u8]) -> Result<Vec<u8>, RvmError> {
        if input.len() != 128 {
            return Ok(vec![0u8; 32]); // Return zero on invalid input
        }

        let mut hash = [0u8; 32];
        let mut v = [0u8; 32];
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];

        hash.copy_from_slice(&input[0..32]);
        v.copy_from_slice(&input[32..64]);
        r.copy_from_slice(&input[64..96]);
        s.copy_from_slice(&input[96..128]);

        // Extract v as recovery ID
        let recovery_id = if v[31] >= 27 && v[31] <= 28 {
            v[31] - 27
        } else {
            return Ok(vec![0u8; 32]); // Invalid recovery ID
        };

        // Combine r and s into signature
        let mut signature = [0u8; 64];
        signature[0..32].copy_from_slice(&r);
        signature[32..64].copy_from_slice(&s);

        match RvmCrypto::ecrecover(&hash, &signature, recovery_id) {
            Ok(pubkey) => {
                let address = RvmCrypto::public_key_to_address(&pubkey);
                let mut result = vec![0u8; 32];
                result[12..32].copy_from_slice(&address);
                Ok(result)
            }
            Err(_) => Ok(vec![0u8; 32]), // Return zero on error
        }
    }

    /// SHA256 precompile (address 0x02)
    pub fn sha256(input: &[u8]) -> Result<Vec<u8>, RvmError> {
        use sha3::Sha3_256;
        let mut hasher = Sha3_256::new();
        hasher.update(input);
        Ok(hasher.finalize().to_vec())
    }

    /// RIPEMD160 precompile (address 0x03)
    pub fn ripemd160(input: &[u8]) -> Result<Vec<u8>, RvmError> {
        // For now, return SHA256 hash padded to 32 bytes
        // In a full implementation, use actual RIPEMD160
        let sha_hash = Self::sha256(input)?;
        let mut result = vec![0u8; 32];
        result[12..32].copy_from_slice(&sha_hash[0..20]);
        Ok(result)
    }

    /// Identity precompile (address 0x04)
    pub fn identity(input: &[u8]) -> Result<Vec<u8>, RvmError> {
        Ok(input.to_vec())
    }

    /// Execute a precompiled contract
    pub fn execute(address: u8, input: &[u8]) -> Result<Vec<u8>, RvmError> {
        match address {
            1 => Self::ecrecover(input),
            2 => Self::sha256(input),
            3 => Self::ripemd160(input),
            4 => Self::identity(input),
            _ => Err(RvmError::InvalidPrecompile(address)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keccak256() {
        let data = b"hello world";
        let hash = RvmCrypto::keccak256(data);
        
        // Known hash for "hello world"
        let expected = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad")
            .unwrap();
        assert_eq!(hash.to_vec(), expected);
    }

    #[test]
    fn test_create_address() {
        let creator = [1u8; 20];
        let nonce = 42;
        let address = RvmCrypto::create_address(&creator, nonce);
        
        // Should be deterministic
        let address2 = RvmCrypto::create_address(&creator, nonce);
        assert_eq!(address, address2);
        
        // Different nonce should give different address
        let address3 = RvmCrypto::create_address(&creator, nonce + 1);
        assert_ne!(address, address3);
    }

    #[test]
    fn test_merkle_root() {
        let leaves = vec![
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
            [4u8; 32],
        ];
        
        let root = RvmCrypto::merkle_root(&leaves);
        assert_ne!(root, [0u8; 32]);
        
        // Should be deterministic
        let root2 = RvmCrypto::merkle_root(&leaves);
        assert_eq!(root, root2);
    }

    #[test]
    fn test_precompiles() {
        // Test identity precompile
        let input = b"test data";
        let result = Precompiles::identity(input).unwrap();
        assert_eq!(result, input);
        
        // Test SHA256 precompile
        let sha_result = Precompiles::sha256(input).unwrap();
        assert_eq!(sha_result.len(), 32);
    }
}
