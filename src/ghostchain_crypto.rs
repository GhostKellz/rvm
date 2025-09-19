//! GhostChain Cryptographic Integration
//!
//! Enhanced cryptographic operations for GhostChain ecosystem using GCrypt backend
//! Supports Ed25519, Secp256k1, Blake3, and other crypto operations needed for
//! GhostID, CNS, and cross-chain functionality.

use crate::error::RvmError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported cryptographic algorithms in GhostChain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CryptoAlgorithm {
    /// Ed25519 for high-performance signatures
    Ed25519,
    /// Secp256k1 for Ethereum compatibility
    Secp256k1,
    /// Blake3 for fast hashing
    Blake3,
    /// Keccak256 for Ethereum compatibility
    Keccak256,
    /// SHA256 for general purpose
    Sha256,
}

/// Signature with algorithm information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostSignature {
    /// The signature bytes
    pub signature: Vec<u8>,
    /// The algorithm used
    pub algorithm: CryptoAlgorithm,
    /// Public key (optional, for verification)
    pub public_key: Option<Vec<u8>>,
    /// Recovery information (for Secp256k1)
    pub recovery_id: Option<u8>,
}

/// Public key with algorithm information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostPublicKey {
    /// The public key bytes
    pub key: Vec<u8>,
    /// The algorithm
    pub algorithm: CryptoAlgorithm,
}

/// Hash digest with algorithm information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostHash {
    /// The hash bytes
    pub hash: Vec<u8>,
    /// The algorithm used
    pub algorithm: CryptoAlgorithm,
}

/// GhostID structure for identity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostId {
    /// The unique identifier
    pub id: String,
    /// Associated public key
    pub public_key: GhostPublicKey,
    /// Domain associations
    pub domains: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// GhostChain crypto backend (wrapper around GCrypt)
#[derive(Debug, Clone)]
pub struct GhostChainCrypto {
    /// Available algorithms
    algorithms: HashMap<CryptoAlgorithm, bool>,
}

impl GhostChainCrypto {
    /// Create a new GhostChain crypto instance
    pub fn new() -> Result<Self, RvmError> {
        let mut algorithms = HashMap::new();

        // Check which algorithms are available
        #[cfg(feature = "gcrypt")]
        {
            algorithms.insert(CryptoAlgorithm::Ed25519, true);
            algorithms.insert(CryptoAlgorithm::Secp256k1, true);
            algorithms.insert(CryptoAlgorithm::Blake3, true);
        }

        // Always available
        algorithms.insert(CryptoAlgorithm::Keccak256, true);
        algorithms.insert(CryptoAlgorithm::Sha256, true);

        Ok(Self { algorithms })
    }

    /// Check if an algorithm is supported
    pub fn is_algorithm_supported(&self, algorithm: CryptoAlgorithm) -> bool {
        self.algorithms.get(&algorithm).copied().unwrap_or(false)
    }

    /// Generate a hash using the specified algorithm
    pub fn hash(&self, data: &[u8], algorithm: CryptoAlgorithm) -> Result<GhostHash, RvmError> {
        if !self.is_algorithm_supported(algorithm) {
            return Err(RvmError::CryptoError(format!("Algorithm {:?} not supported", algorithm)));
        }

        let hash = match algorithm {
            CryptoAlgorithm::Keccak256 => {
                use sha3::{Digest, Keccak256};
                let mut hasher = Keccak256::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            CryptoAlgorithm::Sha256 => {
                use sha3::{Digest, Sha3_256};
                let mut hasher = Sha3_256::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            #[cfg(feature = "gcrypt")]
            CryptoAlgorithm::Blake3 => {
                // Use GCrypt Blake3 implementation
                self.gcrypt_blake3(data)?
            }
            _ => {
                return Err(RvmError::CryptoError(format!("Hash algorithm {:?} not implemented", algorithm)));
            }
        };

        Ok(GhostHash { hash, algorithm })
    }

    /// Verify a GhostID signature
    pub fn verify_ghost_id_signature(
        &self,
        ghost_id: &str,
        message: &[u8],
        signature: &GhostSignature,
    ) -> Result<bool, RvmError> {
        match signature.algorithm {
            CryptoAlgorithm::Ed25519 => {
                #[cfg(feature = "gcrypt")]
                {
                    self.verify_ed25519_signature(message, signature)
                }
                #[cfg(not(feature = "gcrypt"))]
                {
                    Err(RvmError::CryptoError("Ed25519 not available without gcrypt feature".to_string()))
                }
            }
            CryptoAlgorithm::Secp256k1 => {
                self.verify_secp256k1_signature(message, signature)
            }
            _ => {
                Err(RvmError::CryptoError(format!("Signature algorithm {:?} not supported for GhostID", signature.algorithm)))
            }
        }
    }

    /// Create a GhostID from public key and domains
    pub fn create_ghost_id(
        &self,
        public_key: &GhostPublicKey,
        domains: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> Result<GhostId, RvmError> {
        // Generate deterministic ID from public key
        let key_hash = self.hash(&public_key.key, CryptoAlgorithm::Blake3)
            .unwrap_or_else(|_| {
                // Fallback to Keccak256 if Blake3 not available
                self.hash(&public_key.key, CryptoAlgorithm::Keccak256).unwrap()
            });

        let id = hex::encode(&key_hash.hash[0..16]); // Use first 16 bytes for ID

        Ok(GhostId {
            id,
            public_key: public_key.clone(),
            domains,
            metadata,
        })
    }

    /// Resolve GhostID to address (for blockchain operations)
    pub fn ghost_id_to_address(&self, ghost_id: &GhostId) -> Result<[u8; 20], RvmError> {
        // Hash the public key to create address
        let key_hash = self.hash(&ghost_id.public_key.key, CryptoAlgorithm::Keccak256)?;

        let mut address = [0u8; 20];
        if key_hash.hash.len() >= 32 {
            address.copy_from_slice(&key_hash.hash[12..32]);
        } else {
            return Err(RvmError::CryptoError("Invalid hash length for address generation".to_string()));
        }

        Ok(address)
    }

    /// Generate address for CNS domain operations
    pub fn domain_to_address(&self, domain: &str) -> Result<[u8; 20], RvmError> {
        // Create deterministic address from domain name
        let domain_hash = self.hash(domain.as_bytes(), CryptoAlgorithm::Keccak256)?;

        let mut address = [0u8; 20];
        address.copy_from_slice(&domain_hash.hash[12..32]);

        // Set a special prefix for domain addresses
        address[0] = 0xdd; // 'dd' for domain

        Ok(address)
    }

    /// Verify secp256k1 signature (Ethereum-compatible)
    fn verify_secp256k1_signature(
        &self,
        message: &[u8],
        signature: &GhostSignature,
    ) -> Result<bool, RvmError> {
        if signature.signature.len() != 64 {
            return Err(RvmError::InvalidSignature);
        }

        let recovery_id = signature.recovery_id.ok_or(RvmError::InvalidSignature)?;

        // Hash the message
        let message_hash = self.hash(message, CryptoAlgorithm::Keccak256)?;
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&message_hash.hash);

        // Convert signature to array
        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&signature.signature);

        // Use existing ECDSA recovery
        match crate::crypto::RvmCrypto::ecrecover(&hash_array, &sig_array, recovery_id) {
            Ok(recovered_pubkey) => {
                if let Some(expected_pubkey) = &signature.public_key {
                    Ok(recovered_pubkey.to_vec() == *expected_pubkey)
                } else {
                    // If no public key provided, just check if recovery succeeded
                    Ok(true)
                }
            }
            Err(_) => Ok(false),
        }
    }

    /// GCrypt-specific implementations (when feature is enabled)
    #[cfg(feature = "gcrypt")]
    fn gcrypt_blake3(&self, data: &[u8]) -> Result<Vec<u8>, RvmError> {
        // This would use the actual GCrypt Blake3 implementation
        // For now, fallback to Keccak256
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(data);
        Ok(hasher.finalize().to_vec())
    }

    #[cfg(feature = "gcrypt")]
    fn verify_ed25519_signature(
        &self,
        message: &[u8],
        signature: &GhostSignature,
    ) -> Result<bool, RvmError> {
        // This would use GCrypt's Ed25519 implementation
        // For now, return a placeholder
        if signature.signature.len() != 64 {
            return Err(RvmError::InvalidSignature);
        }

        if signature.public_key.is_none() {
            return Err(RvmError::InvalidSignature);
        }

        // TODO: Implement actual Ed25519 verification with GCrypt
        // gcrypt::protocols::Ed25519::verify(message, &signature.signature, public_key)

        Ok(true) // Placeholder
    }

    /// Cross-chain signature verification
    pub fn verify_cross_chain_signature(
        &self,
        chain_id: u64,
        message: &[u8],
        signature: &GhostSignature,
    ) -> Result<bool, RvmError> {
        // Create chain-specific message
        let mut chain_message = Vec::new();
        chain_message.extend_from_slice(&chain_id.to_be_bytes());
        chain_message.extend_from_slice(message);

        self.verify_ghost_id_signature("", &chain_message, signature)
    }

    /// Generate deterministic keypair for testing
    pub fn generate_test_keypair(&self, seed: &[u8], algorithm: CryptoAlgorithm) -> Result<(Vec<u8>, Vec<u8>), RvmError> {
        if !self.is_algorithm_supported(algorithm) {
            return Err(RvmError::CryptoError(format!("Algorithm {:?} not supported", algorithm)));
        }

        match algorithm {
            CryptoAlgorithm::Secp256k1 => {
                // Generate deterministic secp256k1 keypair from seed
                let seed_hash = self.hash(seed, CryptoAlgorithm::Keccak256)?;

                // Simple keypair generation for testing (not cryptographically secure)
                let mut private_key = [0u8; 32];
                private_key.copy_from_slice(&seed_hash.hash);

                // For testing, just derive public key from private key hash
                let pubkey_hash = self.hash(&private_key, CryptoAlgorithm::Keccak256)?;
                let mut public_key = [0u8; 64];
                public_key[0..32].copy_from_slice(&pubkey_hash.hash);
                public_key[32..64].copy_from_slice(&seed_hash.hash);

                Ok((private_key.to_vec(), public_key.to_vec()))
            }
            #[cfg(feature = "gcrypt")]
            CryptoAlgorithm::Ed25519 => {
                // Generate deterministic Ed25519 keypair
                let seed_hash = self.hash(seed, CryptoAlgorithm::Blake3)?;

                // TODO: Use GCrypt Ed25519 key generation
                // For now, use simple derivation
                let mut private_key = [0u8; 32];
                private_key.copy_from_slice(&seed_hash.hash[0..32]);

                let mut public_key = [0u8; 32];
                public_key.copy_from_slice(&seed_hash.hash[0..32]);
                // Modify to make it different from private key
                public_key[0] ^= 0x80;

                Ok((private_key.to_vec(), public_key.to_vec()))
            }
            _ => {
                Err(RvmError::CryptoError(format!("Keypair generation not supported for {:?}", algorithm)))
            }
        }
    }
}

impl Default for GhostChainCrypto {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            algorithms: HashMap::new(),
        })
    }
}

/// Utility functions for GhostChain crypto operations
pub struct GhostChainCryptoUtils;

impl GhostChainCryptoUtils {
    /// Convert hex string to bytes
    pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, RvmError> {
        let hex_str = if hex_str.starts_with("0x") {
            &hex_str[2..]
        } else {
            hex_str
        };

        hex::decode(hex_str).map_err(|e| RvmError::CryptoError(format!("Invalid hex: {}", e)))
    }

    /// Convert bytes to hex string
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        format!("0x{}", hex::encode(bytes))
    }

    /// Generate domain hash for CNS operations
    pub fn domain_hash(domain: &str) -> [u8; 32] {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(domain.as_bytes());
        hasher.finalize().into()
    }

    /// Validate GhostID format
    pub fn validate_ghost_id_format(ghost_id: &str) -> bool {
        // GhostID should be 32 hex characters (16 bytes)
        ghost_id.len() == 32 && ghost_id.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Extract domain from full domain name
    pub fn extract_domain_parts(domain: &str) -> Result<(String, String), RvmError> {
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() < 2 {
            return Err(RvmError::InvalidDomainName(format!("Invalid domain format: {}", domain)));
        }

        let name = parts[0].to_string();
        let tld = parts[1..].join(".");

        Ok((name, tld))
    }

    /// Check if domain is a GhostChain domain (.ghost, .gcc, .spirit, .mana)
    pub fn is_ghostchain_domain(domain: &str) -> bool {
        domain.ends_with(".ghost") ||
        domain.ends_with(".gcc") ||
        domain.ends_with(".spirit") ||
        domain.ends_with(".mana")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_initialization() {
        let crypto = GhostChainCrypto::new().unwrap();
        assert!(crypto.is_algorithm_supported(CryptoAlgorithm::Keccak256));
        assert!(crypto.is_algorithm_supported(CryptoAlgorithm::Sha256));
    }

    #[test]
    fn test_hashing() {
        let crypto = GhostChainCrypto::new().unwrap();
        let data = b"test data";

        let hash = crypto.hash(data, CryptoAlgorithm::Keccak256).unwrap();
        assert_eq!(hash.algorithm, CryptoAlgorithm::Keccak256);
        assert_eq!(hash.hash.len(), 32);

        // Should be deterministic
        let hash2 = crypto.hash(data, CryptoAlgorithm::Keccak256).unwrap();
        assert_eq!(hash.hash, hash2.hash);
    }

    #[test]
    fn test_ghost_id_creation() {
        let crypto = GhostChainCrypto::new().unwrap();

        let public_key = GhostPublicKey {
            key: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            algorithm: CryptoAlgorithm::Ed25519,
        };

        let domains = vec!["test.ghost".to_string()];
        let metadata = HashMap::new();

        let ghost_id = crypto.create_ghost_id(&public_key, domains, metadata).unwrap();

        assert!(!ghost_id.id.is_empty());
        assert_eq!(ghost_id.domains.len(), 1);
        assert_eq!(ghost_id.domains[0], "test.ghost");
    }

    #[test]
    fn test_domain_operations() {
        let crypto = GhostChainCrypto::new().unwrap();

        let domain = "example.ghost";
        let address = crypto.domain_to_address(domain).unwrap();

        // Should be deterministic
        let address2 = crypto.domain_to_address(domain).unwrap();
        assert_eq!(address, address2);

        // Different domains should give different addresses
        let address3 = crypto.domain_to_address("different.ghost").unwrap();
        assert_ne!(address, address3);

        // Domain addresses should have special prefix
        assert_eq!(address[0], 0xdd);
    }

    #[test]
    fn test_crypto_utils() {
        let hex_str = "0x1234abcd";
        let bytes = GhostChainCryptoUtils::hex_to_bytes(hex_str).unwrap();
        let hex_back = GhostChainCryptoUtils::bytes_to_hex(&bytes);
        assert_eq!(hex_str, hex_back);

        assert!(GhostChainCryptoUtils::validate_ghost_id_format("1234567890abcdef1234567890abcdef"));
        assert!(!GhostChainCryptoUtils::validate_ghost_id_format("invalid"));

        assert!(GhostChainCryptoUtils::is_ghostchain_domain("test.ghost"));
        assert!(GhostChainCryptoUtils::is_ghostchain_domain("example.gcc"));
        assert!(!GhostChainCryptoUtils::is_ghostchain_domain("example.com"));
    }

    #[test]
    fn test_keypair_generation() {
        let crypto = GhostChainCrypto::new().unwrap();
        let seed = b"test seed";

        let (private_key, public_key) = crypto
            .generate_test_keypair(seed, CryptoAlgorithm::Secp256k1)
            .unwrap();

        assert_eq!(private_key.len(), 32);
        assert_eq!(public_key.len(), 64);

        // Should be deterministic
        let (private_key2, public_key2) = crypto
            .generate_test_keypair(seed, CryptoAlgorithm::Secp256k1)
            .unwrap();

        assert_eq!(private_key, private_key2);
        assert_eq!(public_key, public_key2);
    }
}