//! GhostChain Services Integration
//!
//! Implementation of GhostID identity verification and CNS domain resolution
//! for RVM opcodes. Integrates with GhostChain services running on ports 8552-8555.

use crate::{
    error::RvmError,
    ghostchain_crypto::{GhostChainCrypto, GhostId, GhostSignature, GhostChainCryptoUtils},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// GhostID service client (Port 8552)
#[derive(Debug, Clone)]
pub struct GhostIdService {
    /// Local cache of verified GhostIDs
    cache: HashMap<String, GhostId>,
    /// Crypto backend
    crypto: GhostChainCrypto,
}

/// CNS (Crypto Name Service) client (Port 8553)
#[derive(Debug, Clone)]
pub struct CnsService {
    /// Domain registry cache
    domains: HashMap<String, DomainRecord>,
    /// Reverse lookup cache (address -> domain)
    reverse_lookup: HashMap<[u8; 20], String>,
}

/// Domain record structure for CNS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRecord {
    /// Domain name (e.g., "example.ghost")
    pub name: String,
    /// Associated address
    pub address: [u8; 20],
    /// Owner address
    pub owner: [u8; 20],
    /// Registration timestamp
    pub registered_at: u64,
    /// Expiration timestamp
    pub expires_at: u64,
    /// Associated GhostID (optional)
    pub ghost_id: Option<String>,
    /// Additional records (A, CNAME, etc.)
    pub records: HashMap<String, String>,
}

/// Response from GhostID verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostIdVerificationResult {
    /// Whether the verification succeeded
    pub verified: bool,
    /// The verified GhostID (if successful)
    pub ghost_id: Option<GhostId>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Verification timestamp
    pub timestamp: u64,
}

/// Response from CNS resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CnsResolutionResult {
    /// Whether the resolution succeeded
    pub resolved: bool,
    /// The resolved address
    pub address: Option<[u8; 20]>,
    /// Full domain record
    pub record: Option<DomainRecord>,
    /// Error message (if failed)
    pub error: Option<String>,
}

impl GhostIdService {
    /// Create a new GhostID service instance
    pub fn new() -> Result<Self, RvmError> {
        let crypto = GhostChainCrypto::new()?;
        Ok(Self {
            cache: HashMap::new(),
            crypto,
        })
    }

    /// Verify a GhostID signature (GHOST_ID_VERIFY opcode implementation)
    pub fn verify_ghost_id_signature(
        &mut self,
        ghost_id: &str,
        message: &[u8],
        signature_data: &[u8],
    ) -> Result<GhostIdVerificationResult, RvmError> {
        // Validate GhostID format
        if !GhostChainCryptoUtils::validate_ghost_id_format(ghost_id) {
            return Ok(GhostIdVerificationResult {
                verified: false,
                ghost_id: None,
                error: Some("Invalid GhostID format".to_string()),
                timestamp: self.current_timestamp(),
            });
        }

        // Try to get GhostID from cache first
        if let Some(cached_ghost_id) = self.cache.get(ghost_id) {
            return self.verify_with_ghost_id(cached_ghost_id, message, signature_data);
        }

        // If not in cache, try to resolve from service (simulated for now)
        match self.fetch_ghost_id_from_service(ghost_id) {
            Ok(ghost_id_obj) => {
                // Cache the result
                self.cache.insert(ghost_id.to_string(), ghost_id_obj.clone());
                self.verify_with_ghost_id(&ghost_id_obj, message, signature_data)
            }
            Err(e) => Ok(GhostIdVerificationResult {
                verified: false,
                ghost_id: None,
                error: Some(format!("Failed to fetch GhostID: {}", e)),
                timestamp: self.current_timestamp(),
            }),
        }
    }

    /// Resolve GhostID to blockchain address (GHOST_ID_RESOLVE opcode implementation)
    pub fn resolve_ghost_id_to_address(&mut self, ghost_id: &str) -> Result<Option<[u8; 20]>, RvmError> {
        // Validate format
        if !GhostChainCryptoUtils::validate_ghost_id_format(ghost_id) {
            return Err(RvmError::InvalidGhostIdFormat(ghost_id.to_string()));
        }

        // Check cache first
        if let Some(cached_ghost_id) = self.cache.get(ghost_id) {
            let address = self.crypto.ghost_id_to_address(cached_ghost_id)?;
            return Ok(Some(address));
        }

        // Fetch from service
        match self.fetch_ghost_id_from_service(ghost_id) {
            Ok(ghost_id_obj) => {
                let address = self.crypto.ghost_id_to_address(&ghost_id_obj)?;
                self.cache.insert(ghost_id.to_string(), ghost_id_obj);
                Ok(Some(address))
            }
            Err(_) => Ok(None),
        }
    }

    /// Create a new GhostID (GHOST_ID_CREATE opcode implementation)
    pub fn create_ghost_id(
        &mut self,
        public_key_data: &[u8],
        domains: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> Result<String, RvmError> {
        // Parse public key (assume Ed25519 for now)
        let public_key = crate::ghostchain_crypto::GhostPublicKey {
            key: public_key_data.to_vec(),
            algorithm: crate::ghostchain_crypto::CryptoAlgorithm::Ed25519,
        };

        // Create GhostID
        let ghost_id = self.crypto.create_ghost_id(&public_key, domains, metadata)?;

        // Cache the new GhostID
        self.cache.insert(ghost_id.id.clone(), ghost_id.clone());

        Ok(ghost_id.id)
    }

    /// Verify signature with known GhostID
    fn verify_with_ghost_id(
        &self,
        ghost_id: &GhostId,
        message: &[u8],
        signature_data: &[u8],
    ) -> Result<GhostIdVerificationResult, RvmError> {
        // Parse signature data (format: algorithm[1] + signature[64] + recovery_id[1])
        if signature_data.len() < 66 {
            return Ok(GhostIdVerificationResult {
                verified: false,
                ghost_id: None,
                error: Some("Invalid signature data length".to_string()),
                timestamp: self.current_timestamp(),
            });
        }

        let algorithm = match signature_data[0] {
            0 => crate::ghostchain_crypto::CryptoAlgorithm::Ed25519,
            1 => crate::ghostchain_crypto::CryptoAlgorithm::Secp256k1,
            _ => {
                return Ok(GhostIdVerificationResult {
                    verified: false,
                    ghost_id: None,
                    error: Some("Unsupported signature algorithm".to_string()),
                    timestamp: self.current_timestamp(),
                });
            }
        };

        let signature = GhostSignature {
            signature: signature_data[1..65].to_vec(),
            algorithm,
            public_key: Some(ghost_id.public_key.key.clone()),
            recovery_id: if signature_data.len() > 65 { Some(signature_data[65]) } else { None },
        };

        // Verify the signature
        match self.crypto.verify_ghost_id_signature(&ghost_id.id, message, &signature) {
            Ok(verified) => Ok(GhostIdVerificationResult {
                verified,
                ghost_id: if verified { Some(ghost_id.clone()) } else { None },
                error: None,
                timestamp: self.current_timestamp(),
            }),
            Err(e) => Ok(GhostIdVerificationResult {
                verified: false,
                ghost_id: None,
                error: Some(format!("Verification failed: {}", e)),
                timestamp: self.current_timestamp(),
            }),
        }
    }

    /// Simulate fetching GhostID from external service
    fn fetch_ghost_id_from_service(&self, ghost_id: &str) -> Result<GhostId, RvmError> {
        // In a real implementation, this would make an HTTP request to port 8552
        // For now, simulate with test data

        if ghost_id == "1234567890abcdef1234567890abcdef" {
            // Return a test GhostID
            let public_key = crate::ghostchain_crypto::GhostPublicKey {
                key: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                         17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32],
                algorithm: crate::ghostchain_crypto::CryptoAlgorithm::Ed25519,
            };

            Ok(GhostId {
                id: ghost_id.to_string(),
                public_key,
                domains: vec!["test.ghost".to_string()],
                metadata: HashMap::new(),
            })
        } else {
            Err(RvmError::GhostIdNotFound(ghost_id.to_string()))
        }
    }

    /// Get current timestamp
    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl CnsService {
    /// Create a new CNS service instance
    pub fn new() -> Self {
        let mut service = Self {
            domains: HashMap::new(),
            reverse_lookup: HashMap::new(),
        };

        // Initialize with some test domains
        service.initialize_test_domains();
        service
    }

    /// Resolve domain to address (CNS_RESOLVE opcode implementation)
    pub fn resolve_domain(&self, domain: &str) -> Result<CnsResolutionResult, RvmError> {
        // Validate domain format
        if domain.is_empty() || !domain.contains('.') {
            return Ok(CnsResolutionResult {
                resolved: false,
                address: None,
                record: None,
                error: Some("Invalid domain format".to_string()),
            });
        }

        // Check if it's a GhostChain domain
        if !GhostChainCryptoUtils::is_ghostchain_domain(domain) {
            return Ok(CnsResolutionResult {
                resolved: false,
                address: None,
                record: None,
                error: Some("Domain not in GhostChain namespace".to_string()),
            });
        }

        // Look up domain
        if let Some(record) = self.domains.get(domain) {
            // Check if domain has expired
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if record.expires_at > current_time {
                Ok(CnsResolutionResult {
                    resolved: true,
                    address: Some(record.address),
                    record: Some(record.clone()),
                    error: None,
                })
            } else {
                Ok(CnsResolutionResult {
                    resolved: false,
                    address: None,
                    record: None,
                    error: Some("Domain has expired".to_string()),
                })
            }
        } else {
            Ok(CnsResolutionResult {
                resolved: false,
                address: None,
                record: None,
                error: Some("Domain not found".to_string()),
            })
        }
    }

    /// Register a new domain (CNS_REGISTER opcode implementation)
    pub fn register_domain(
        &mut self,
        domain: &str,
        owner: [u8; 20],
        target_address: [u8; 20],
        ghost_id: Option<String>,
    ) -> Result<bool, RvmError> {
        // Validate domain
        if !GhostChainCryptoUtils::is_ghostchain_domain(domain) {
            return Err(RvmError::InvalidDomainName(domain.to_string()));
        }

        // Check if domain already exists
        if self.domains.contains_key(domain) {
            return Err(RvmError::DomainRegistrationFailed(
                format!("Domain {} already registered", domain)
            ));
        }

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Create domain record
        let record = DomainRecord {
            name: domain.to_string(),
            address: target_address,
            owner,
            registered_at: current_time,
            expires_at: current_time + 365 * 24 * 60 * 60, // 1 year
            ghost_id,
            records: HashMap::new(),
        };

        // Register domain
        self.domains.insert(domain.to_string(), record);
        self.reverse_lookup.insert(target_address, domain.to_string());

        Ok(true)
    }

    /// Update domain records (CNS_UPDATE opcode implementation)
    pub fn update_domain(
        &mut self,
        domain: &str,
        owner: [u8; 20],
        new_address: Option<[u8; 20]>,
        new_records: Option<HashMap<String, String>>,
    ) -> Result<bool, RvmError> {
        // Get existing record
        let record = self.domains.get_mut(domain)
            .ok_or_else(|| RvmError::DomainNotFound(domain.to_string()))?;

        // Check ownership
        if record.owner != owner {
            return Err(RvmError::UnauthorizedDomainOperation(
                format!("Not owner of domain {}", domain)
            ));
        }

        // Update address if provided
        if let Some(new_addr) = new_address {
            // Remove old reverse lookup
            self.reverse_lookup.remove(&record.address);
            // Update record
            record.address = new_addr;
            // Add new reverse lookup
            self.reverse_lookup.insert(new_addr, domain.to_string());
        }

        // Update additional records if provided
        if let Some(records) = new_records {
            record.records.extend(records);
        }

        Ok(true)
    }

    /// Get domain owner (CNS_OWNER opcode implementation)
    pub fn get_domain_owner(&self, domain: &str) -> Result<Option<[u8; 20]>, RvmError> {
        if let Some(record) = self.domains.get(domain) {
            Ok(Some(record.owner))
        } else {
            Ok(None)
        }
    }

    /// Reverse lookup: get domain from address
    pub fn reverse_lookup(&self, address: &[u8; 20]) -> Option<String> {
        self.reverse_lookup.get(address).cloned()
    }

    /// Initialize test domains for development
    fn initialize_test_domains(&mut self) {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Test domains
        let test_domains = vec![
            ("test.ghost", [0x11; 20], [0x22; 20]),
            ("example.gcc", [0x33; 20], [0x44; 20]),
            ("demo.spirit", [0x55; 20], [0x66; 20]),
            ("sample.mana", [0x77; 20], [0x88; 20]),
        ];

        for (domain, owner, address) in test_domains {
            let record = DomainRecord {
                name: domain.to_string(),
                address,
                owner,
                registered_at: current_time,
                expires_at: current_time + 365 * 24 * 60 * 60, // 1 year
                ghost_id: None,
                records: HashMap::new(),
            };

            self.domains.insert(domain.to_string(), record);
            self.reverse_lookup.insert(address, domain.to_string());
        }
    }
}

/// Main GhostChain services coordinator
#[derive(Debug, Clone)]
pub struct GhostChainServices {
    /// GhostID service
    pub ghost_id: GhostIdService,
    /// CNS service
    pub cns: CnsService,
}

impl GhostChainServices {
    /// Create new GhostChain services instance
    pub fn new() -> Result<Self, RvmError> {
        Ok(Self {
            ghost_id: GhostIdService::new()?,
            cns: CnsService::new(),
        })
    }

    /// Execute GhostID verification (for GHOST_ID_VERIFY opcode)
    pub fn execute_ghost_id_verify(
        &mut self,
        ghost_id: &str,
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool, RvmError> {
        let result = self.ghost_id.verify_ghost_id_signature(ghost_id, message, signature)?;
        Ok(result.verified)
    }

    /// Execute GhostID resolution (for GHOST_ID_RESOLVE opcode)
    pub fn execute_ghost_id_resolve(&mut self, ghost_id: &str) -> Result<Option<[u8; 20]>, RvmError> {
        self.ghost_id.resolve_ghost_id_to_address(ghost_id)
    }

    /// Execute GhostID creation (for GHOST_ID_CREATE opcode)
    pub fn execute_ghost_id_create(
        &mut self,
        public_key: &[u8],
        domains: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> Result<String, RvmError> {
        self.ghost_id.create_ghost_id(public_key, domains, metadata)
    }

    /// Execute CNS resolution (for CNS_RESOLVE opcode)
    pub fn execute_cns_resolve(&self, domain: &str) -> Result<Option<[u8; 20]>, RvmError> {
        let result = self.cns.resolve_domain(domain)?;
        Ok(result.address)
    }

    /// Execute CNS registration (for CNS_REGISTER opcode)
    pub fn execute_cns_register(
        &mut self,
        domain: &str,
        owner: [u8; 20],
        target_address: [u8; 20],
        ghost_id: Option<String>,
    ) -> Result<bool, RvmError> {
        self.cns.register_domain(domain, owner, target_address, ghost_id)
    }

    /// Execute CNS update (for CNS_UPDATE opcode)
    pub fn execute_cns_update(
        &mut self,
        domain: &str,
        owner: [u8; 20],
        new_address: Option<[u8; 20]>,
        new_records: Option<HashMap<String, String>>,
    ) -> Result<bool, RvmError> {
        self.cns.update_domain(domain, owner, new_address, new_records)
    }

    /// Execute CNS owner lookup (for CNS_OWNER opcode)
    pub fn execute_cns_owner(&self, domain: &str) -> Result<Option<[u8; 20]>, RvmError> {
        self.cns.get_domain_owner(domain)
    }
}

impl Default for GhostChainServices {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            ghost_id: GhostIdService {
                cache: HashMap::new(),
                crypto: GhostChainCrypto::default(),
            },
            cns: CnsService::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ghost_id_service() {
        let mut service = GhostIdService::new().unwrap();

        // Test GhostID creation
        let public_key = vec![1u8; 32];
        let domains = vec!["test.ghost".to_string()];
        let metadata = HashMap::new();

        let ghost_id = service.create_ghost_id(&public_key, domains, metadata).unwrap();
        assert!(!ghost_id.is_empty());

        // Test GhostID resolution
        let address = service.resolve_ghost_id_to_address(&ghost_id).unwrap();
        assert!(address.is_some());
    }

    #[test]
    fn test_cns_service() {
        let mut service = CnsService::new();

        // Test domain resolution (should find test domain)
        let result = service.resolve_domain("test.ghost").unwrap();
        assert!(result.resolved);
        assert!(result.address.is_some());

        // Test non-existent domain
        let result = service.resolve_domain("nonexistent.ghost").unwrap();
        assert!(!result.resolved);

        // Test domain registration
        let new_domain = "newdomain.ghost";
        let owner = [0x99; 20];
        let target = [0xaa; 20];

        let success = service.register_domain(new_domain, owner, target, None).unwrap();
        assert!(success);

        // Verify registration
        let result = service.resolve_domain(new_domain).unwrap();
        assert!(result.resolved);
        assert_eq!(result.address.unwrap(), target);
    }

    #[test]
    fn test_ghostchain_services() {
        let mut services = GhostChainServices::new().unwrap();

        // Test CNS operations
        let resolved = services.execute_cns_resolve("test.ghost").unwrap();
        assert!(resolved.is_some());

        let owner = services.execute_cns_owner("test.ghost").unwrap();
        assert!(owner.is_some());

        // Test GhostID operations
        let public_key = vec![1u8; 32];
        let domains = vec!["newtest.ghost".to_string()];
        let metadata = HashMap::new();

        let ghost_id = services.execute_ghost_id_create(&public_key, domains, metadata).unwrap();
        assert!(!ghost_id.is_empty());

        let address = services.execute_ghost_id_resolve(&ghost_id).unwrap();
        assert!(address.is_some());
    }

    #[test]
    fn test_domain_validation() {
        assert!(GhostChainCryptoUtils::is_ghostchain_domain("test.ghost"));
        assert!(GhostChainCryptoUtils::is_ghostchain_domain("example.gcc"));
        assert!(GhostChainCryptoUtils::is_ghostchain_domain("demo.spirit"));
        assert!(GhostChainCryptoUtils::is_ghostchain_domain("sample.mana"));
        assert!(!GhostChainCryptoUtils::is_ghostchain_domain("example.com"));

        assert!(GhostChainCryptoUtils::validate_ghost_id_format("1234567890abcdef1234567890abcdef"));
        assert!(!GhostChainCryptoUtils::validate_ghost_id_format("invalid"));
    }
}